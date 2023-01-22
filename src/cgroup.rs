use std::fmt::format;

use cgroups_rs::{Cgroup, CgroupPid, Controller, MaxValue};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::cpuacct::CpuAcctController;
use cgroups_rs::memory::MemController;
use cgroups_rs::pid::PidController;
use log::{debug, error, warn};
use nix::sys::resource::{getrusage, UsageWho};
use nix::sys::time::TimeVal;
use nix::unistd::Pid;

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
  memory_swap: u64,
}

impl CatBoxCgroup {
  pub fn new(params: &CatBoxParams, child: Pid) -> Self {
    debug!("Init cgroup {}", params.cgroup);

    let hierarchy = cgroups_rs::hierarchies::auto();

    let enable_cpuacct = hierarchy.subsystems().iter().any(|subsystem| subsystem.controller_name() == "cpuacct");
    let enable_memory = hierarchy.subsystems().iter().any(|subsystem| subsystem.controller_name() == "memory");
    let enable_pids = hierarchy.subsystems().iter().any(|subsystem| subsystem.controller_name() == "pids");

    let cgroup_name = format!("{}/{}.{}", params.cgroup, params.cgroup, child.as_raw());
    let builder = CgroupBuilder::new(cgroup_name.as_str());
    let builder = if enable_memory {
      let memory_limit = params.memory_limit as i64 * 1024 + 4 * 1024;
      builder.memory()
        .memory_soft_limit(memory_limit)
        .memory_hard_limit(memory_limit)
        .memory_swap_limit(memory_limit)
        .done()
    } else {
      builder
    };
    let builder = if enable_pids {
      builder.pid()
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
    if enable_pids {
      supported_controller.push("pids".to_string());
    }
    let builder = builder.set_specified_controllers(supported_controller);

    let cgroup = match builder.build(hierarchy) {
      Ok(cgroup) => cgroup,
      Err(err) => {
        error!("Build cgroup fails: {}", err);
        return CatBoxCgroup {
          name: cgroup_name,
          cgroup: None,
          enable_cpuacct: false,
          enable_memory: false,
        };
      }
    };
    let task = CgroupPid::from(child.as_raw() as u64);

    if enable_cpuacct {
      let cpuacct: &CpuAcctController = cgroup.controller_of().unwrap();
      cpuacct.reset().unwrap();
      cpuacct.add_task(&task).unwrap();
    }
    if enable_memory {
      let memory: &MemController = cgroup.controller_of().unwrap();
      memory.reset_max_usage().unwrap();
      memory.add_task(&task).unwrap();
    }
    if enable_pids {
      let pid: &PidController = cgroup.controller_of().unwrap();
      pid.add_task(&task).unwrap();
    }

    if !enable_cpuacct {
      warn!("cgroup cpuacct is not supported");
    }
    if !enable_memory {
      warn!("cgroup memory is not supported");
    }
    if !enable_pids {
      warn!("cgroup pids is not supported");
    }

    CatBoxCgroup {
      name: cgroup_name,
      cgroup: Some(cgroup),
      enable_cpuacct,
      enable_memory,
    }
  }

  pub fn usage(&self) -> CatBoxUsage {
    let mut rusage = None;

    let is_cgroup = self.cgroup.is_some();
    let (time, time_user, time_sys) = if is_cgroup && self.enable_cpuacct {
      let cgroup = self.cgroup.as_ref().unwrap();
      let cpuacct: &CpuAcctController = cgroup.controller_of().unwrap();
      let acct = cpuacct.cpuacct();
      debug!("usage: {}", acct.usage);
      debug!("usage_sys: {}", acct.usage_sys);
      debug!("usage_user: {}", acct.usage_user);
      cpuacct.reset().unwrap();
      (acct.usage / 1000000, acct.usage_user / 1000000, acct.usage_sys / 1000000)
    } else {
      let usage = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();
      rusage = Some(usage);
      debug!("usage.user_time: {}", usage.user_time());
      debug!("usage.system_time: {}", usage.system_time());
      let time_user = usage.user_time();
      let time_sys = usage.system_time();
      (microseconds(time_user + time_sys), microseconds(time_user), microseconds(time_sys))
    };

    let memory_swap = if is_cgroup && self.enable_memory {
      let cgroup = self.cgroup.as_ref().unwrap();
      let memory: &MemController = cgroup.controller_of().unwrap();
      let memswap = memory.memswap();
      debug!("memswap.max_usage_in_bytes: {}", memswap.max_usage_in_bytes);
      memory.reset_max_usage().unwrap();
      memswap.max_usage_in_bytes / 1024
    } else {
      let usage = rusage.unwrap_or_else(|| getrusage(UsageWho::RUSAGE_CHILDREN).unwrap());
      debug!("usage.max_rss: {}", usage.max_rss());
      usage.max_rss() as u64
    };

    CatBoxUsage {
      time,
      time_user,
      time_sys,
      memory_swap,
    }
  }
}

impl Drop for CatBoxCgroup {
  fn drop(&mut self) {
    if let Some(cgroup) = &self.cgroup {
      debug!("Delete created cgroup {}", self.name);
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

  pub fn memory_swap(&self) -> u64 {
    self.memory_swap
  }
}

fn microseconds(val: TimeVal) -> u64 {
  (val.tv_sec() * 1000 + val.tv_usec() / 1000) as u64
}
