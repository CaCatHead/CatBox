use std::error::Error;

use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::cpu::CpuController;
use cgroups_rs::cpuacct::{CpuAcct, CpuAcctController};
use cgroups_rs::memory::{MemController, MemSwap, Memory};
use cgroups_rs::pid::PidController;
use cgroups_rs::{Cgroup, CgroupPid, Controller, MaxValue};
use log::{debug, error, warn};
use nix::sys::resource::{getrusage, UsageWho};
use nix::sys::time::TimeVal;
use nix::unistd::Pid;

use crate::error::CatBoxError;
use crate::CatBoxParams;

pub struct CatBoxCgroup {
  name: String,
  cgroup: Option<Cgroup>,
  enable_cpuacct: bool,
  enable_memory: bool,
}

#[derive(Debug)]
pub struct CatBoxUsage {
  time: u64,
  time_user: u64,
  time_sys: u64,
  memory: u64,
}

impl CatBoxCgroup {
  pub fn new(params: &CatBoxParams, child: Pid) -> Result<Self, CatBoxError> {
    let hierarchy = cgroups_rs::hierarchies::auto();

    let mut enable_cpuacct = hierarchy
      .subsystems()
      .iter()
      .any(|subsystem| subsystem.controller_name() == "cpuacct");
    let mut enable_memory = hierarchy
      .subsystems()
      .iter()
      .any(|subsystem| subsystem.controller_name() == "memory");

    let enable_cpu = hierarchy
      .subsystems()
      .iter()
      .any(|subsystem| subsystem.controller_name() == "cpu");
    let enable_pids = hierarchy
      .subsystems()
      .iter()
      .any(|subsystem| subsystem.controller_name() == "pids");

    let cgroup_name = format!("{}/{}.{}", params.cgroup, params.cgroup, child.as_raw());

    debug!("Init cgroup {}", cgroup_name);

    let builder = CgroupBuilder::new(cgroup_name.as_str());
    let builder = if enable_memory {
      let memory_limit = params.memory_limit as i64 * 1024 + 4 * 1024;
      builder
        .memory()
        .memory_soft_limit(memory_limit)
        .memory_hard_limit(memory_limit)
        .memory_swap_limit(memory_limit)
        .done()
    } else {
      builder
    };
    let builder = if enable_cpu {
      builder.cpu().quota(1000000).period(1000000).done()
    } else {
      builder
    };
    let builder = if enable_pids {
      builder
        .pid()
        .maximum_number_of_processes(MaxValue::Value(params.process as i64))
        .done()
    } else {
      builder
    };

    let mut supported_controller = vec![];
    if enable_cpuacct {
      supported_controller.push("cpuacct".to_string());
    }
    if enable_memory {
      supported_controller.push("memory".to_string());
    }
    if enable_cpu {
      supported_controller.push("cpu".to_string());
    }
    if enable_pids {
      supported_controller.push("pids".to_string());
    }
    let builder = builder.set_specified_controllers(supported_controller);

    let cgroup = match builder.build(hierarchy) {
      Ok(cgroup) => cgroup,
      Err(err) => {
        error!("Build cgroup fails: {}", err);
        if params.force {
          return Err(CatBoxError::cgroup(err.to_string()));
        } else {
          return Ok(CatBoxCgroup {
            name: cgroup_name,
            cgroup: None,
            enable_cpuacct: false,
            enable_memory: false,
          });
        }
      }
    };
    let task = CgroupPid::from(child.as_raw() as u64);

    if enable_cpuacct {
      let add_task = || -> Result<(), Box<dyn Error>> {
        let cpuacct: &CpuAcctController = cgroup
          .controller_of()
          .ok_or(Box::<dyn Error>::from("Get cpu controller fails"))?;
        cpuacct.reset()?;
        cpuacct.add_task(&task)?;
        Ok(())
      };
      if add_task().is_err() {
        enable_cpuacct = false;
      }
    }
    if enable_memory {
      let add_task = || -> Result<(), Box<dyn Error>> {
        let memory: &MemController = cgroup
          .controller_of()
          .ok_or(Box::<dyn Error>::from("Get memory controller fails"))?;
        memory.reset_max_usage()?;
        memory.add_task(&task)?;
        Ok(())
      };
      if add_task().is_err() {
        enable_memory = false;
      }
    }
    if enable_cpu {
      if let Some(cpu) = cgroup.controller_of::<CpuController>() {
        if let Err(err) = cpu.add_task(&task) {
          error!("Add cgroup cpu task fails: {}", err)
        }
      } else {
        error!("Get cpu cgroup controller fails")
      }
    }
    if enable_pids {
      if let Some(pid) = cgroup.controller_of::<PidController>() {
        if let Err(err) = pid.add_task(&task) {
          error!("Add cgroup pids task fails: {}", err)
        }
      } else {
        error!("Get pids cgroup controller fails")
      }
    }

    // 默认回退到不使用 cgroup，force 模式下报错
    if !enable_cpuacct {
      if params.force {
        return Err(CatBoxError::cgroup("cgroup cpuacct is not supported"));
      } else {
        warn!("cgroup cpuacct is not supported");
      }
    }
    if !enable_memory {
      if params.force {
        return Err(CatBoxError::cgroup("cgroup memory is not supported"));
      } else {
        warn!("cgroup memory is not supported");
      }
    }
    if !enable_cpu {
      if params.force {
        return Err(CatBoxError::cgroup("cgroup cpu is not supported"));
      } else {
        warn!("cgroup cpu is not supported");
      }
    }
    if !enable_pids {
      if params.force {
        return Err(CatBoxError::cgroup("cgroup pids is not supported"));
      } else {
        warn!("cgroup pids is not supported");
      }
    }

    Ok(CatBoxCgroup {
      name: cgroup_name,
      cgroup: Some(cgroup),
      enable_cpuacct,
      enable_memory,
    })
  }

  fn get_cpuacct(&self) -> Result<CpuAcct, Box<dyn Error>> {
    if self.enable_cpuacct {
      match &self.cgroup {
        None => Err(Box::<dyn Error>::from("cgroup is None")),
        Some(cgroup) => {
          let cpuacct: &CpuAcctController = cgroup
            .controller_of()
            .ok_or(Box::<dyn Error>::from("Get cpu controller fails"))?;
          let acct = cpuacct.cpuacct();
          debug!("usage: {}", acct.usage);
          debug!("usage_sys: {}", acct.usage_sys);
          debug!("usage_user: {}", acct.usage_user);
          cpuacct.reset()?;
          Ok(acct)
        }
      }
    } else {
      Err(Box::<dyn Error>::from("cpuacct is disabled"))
    }
  }

  fn get_memory(&self) -> Result<(Memory, MemSwap), Box<dyn Error>> {
    if self.enable_memory {
      match &self.cgroup {
        None => Err(Box::<dyn Error>::from("cgroup is None")),
        Some(cgroup) => {
          let memory: &MemController = cgroup
            .controller_of()
            .ok_or(Box::<dyn Error>::from("Get memory controller fails"))?;
          let mem = memory.memory_stat();
          debug!("mem.max_usage_in_bytes: {}", mem.max_usage_in_bytes);
          let memswap = memory.memswap();
          debug!("memswap.max_usage_in_bytes: {}", memswap.max_usage_in_bytes);
          memory.reset_max_usage()?;
          Ok((mem, memswap))
        }
      }
    } else {
      Err(Box::<dyn Error>::from("memory is disabled"))
    }
  }

  pub fn usage(&self) -> CatBoxUsage {
    let mut rusage = None;

    let (time, time_user, time_sys) = match self.get_cpuacct() {
      Ok(acct) => (
        acct.usage / 1000000,
        acct.usage_user / 1000000,
        acct.usage_sys / 1000000,
      ),
      Err(_) => {
        let usage = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();
        rusage = Some(usage);
        debug!("usage.user_time: {}", usage.user_time());
        debug!("usage.system_time: {}", usage.system_time());
        let time_user = usage.user_time();
        let time_sys = usage.system_time();
        (
          microseconds(time_user + time_sys),
          microseconds(time_user),
          microseconds(time_sys),
        )
      }
    };

    let memory = match self.get_memory() {
      Ok((mem, memswap)) => {
        std::cmp::max(mem.max_usage_in_bytes, memswap.max_usage_in_bytes) / 1024
      }
      Err(_) => {
        let usage = rusage.unwrap_or_else(|| getrusage(UsageWho::RUSAGE_CHILDREN).unwrap());
        debug!("usage.max_rss: {}", usage.max_rss());
        usage.max_rss() as u64
      }
    };

    CatBoxUsage {
      time,
      time_user,
      time_sys,
      memory,
    }
  }
}

impl Drop for CatBoxCgroup {
  fn drop(&mut self) {
    if let Some(cgroup) = &self.cgroup {
      debug!("Delete cgroup {}", self.name);
      cgroup.delete().unwrap();
    }
  }
}

impl CatBoxUsage {
  pub fn time(&self) -> u64 {
    self.time
  }

  pub fn time_user(&self) -> u64 {
    self.time_user
  }

  pub fn time_sys(&self) -> u64 {
    self.time_sys
  }

  pub fn memory(&self) -> u64 {
    self.memory
  }
}

fn microseconds(val: TimeVal) -> u64 {
  (val.tv_sec() * 1000 + val.tv_usec() / 1000) as u64
}
