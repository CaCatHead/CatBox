use cgroups_rs::{Cgroup, CgroupPid, Controller, MaxValue};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::cpuacct::CpuAcctController;
use cgroups_rs::memory::MemController;
use cgroups_rs::pid::PidController;
use log::{debug, info, warn};
use nix::unistd::Pid;

use crate::CatBoxParams;

pub struct CatBoxCgroup {
  pid: Pid,
  cgroup: Cgroup,
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

    let builder = CgroupBuilder::new(params.cgroup.as_str());
    let builder = if enable_memory {
      builder.memory()
        .memory_soft_limit(params.memory_limit as i64)
        .memory_hard_limit(params.memory_limit as i64)
        .memory_swap_limit(params.memory_limit as i64)
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

    let cgroup = builder.build(hierarchy).unwrap();
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
      pid: child,
      cgroup,
      enable_cpuacct,
      enable_memory,
    }
  }

  pub fn usage(&self) -> CatBoxUsage {
    let (time, time_user, time_sys) = if self.enable_cpuacct {
      let cpuacct: &cgroups_rs::cpuacct::CpuAcctController = self.cgroup.controller_of().unwrap();
      let acct = cpuacct.cpuacct();
      debug!("usage: {}", acct.usage);
      debug!("usage_sys: {}", acct.usage_sys);
      debug!("usage_user: {}", acct.usage_user);
      cpuacct.reset().unwrap();
      (acct.usage, acct.usage_user, acct.usage_sys)
    } else {
      (0, 0, 0)
    };

    let memory_swap = if self.enable_memory {
      let memory: &cgroups_rs::memory::MemController = self.cgroup.controller_of().unwrap();
      let memswap = memory.memswap();
      debug!("memswap.max_usage_in_bytes: {}", memswap.max_usage_in_bytes);
      memory.reset_max_usage().unwrap();
      memswap.max_usage_in_bytes
    } else {
      0
    };

    CatBoxUsage {
      time,
      time_user,
      time_sys,
      memory_swap,
    }
  }
}
