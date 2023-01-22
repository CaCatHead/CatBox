use std::env;
use std::ffi::{c_uint, CString};

use cgroups_rs::{Cgroup, CgroupPid, Controller, MaxValue};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use libc_stdhandle::{stderr, stdin, stdout};
use log::{debug, error, info};
use nix::libc;
use nix::libc::{freopen, RLIM_INFINITY};
use nix::sys::resource::{Resource, setrlimit};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{alarm, execvpe, fork, ForkResult, Pid};

use crate::CatBoxParams;
use crate::utils::into_c_string;

/// 重定向输出输出
fn redirect_io(params: &CatBoxParams) {
  unsafe {
    let in_path = into_c_string(&params.stdin);
    let mode = CString::new("r").unwrap();
    freopen(in_path.as_ptr(), mode.as_ptr(), stdin());

    let out_path = into_c_string(&params.stdout);
    let mode = CString::new("w").unwrap();
    freopen(out_path.as_ptr(), mode.as_ptr(), stdout());

    let err_path = into_c_string(&params.stderr);
    let mode = CString::new("w").unwrap();
    freopen(err_path.as_ptr(), mode.as_ptr(), stderr());
  }
}

/// 设置子进程时钟 signal，运行时限 + 1 秒
fn set_alarm(params: &CatBoxParams) {
  let time_limit = (params.time_limit as f64 / 1000.0 as f64).ceil() as c_uint;
  alarm::set(time_limit + 1);
  debug!("Set alarm {} seconds", time_limit + 1);
}

/// 调用 setrlimit
fn set_resource_limit(params: &CatBoxParams) {
  let stack_size = if params.stack_size == u64::MAX {
    RLIM_INFINITY
  } else {
    params.stack_size
  };
  setrlimit(Resource::RLIMIT_STACK, stack_size, stack_size).expect("setrlimit stack should be ok");
}

/// 设置 cgroup
fn set_cgroup(params: &CatBoxParams, child: Pid) -> Cgroup {
  let hierarchy = cgroups_rs::hierarchies::auto();
  let cg = CgroupBuilder::new("catj")
    .memory()
    .memory_soft_limit(params.memory_limit as i64)
    .memory_hard_limit(params.memory_limit as i64)
    .memory_swap_limit(params.memory_limit as i64)
    .done()
    .pid()
    .maximum_number_of_processes(MaxValue::Value(params.process as i64))
    .done();

  let cgroup = cg.build(hierarchy).unwrap();
  let child = CgroupPid::from(child.as_raw() as u64);

  let cpuacct: &cgroups_rs::cpuacct::CpuAcctController = cgroup.controller_of().unwrap();
  cpuacct.reset().unwrap();
  cpuacct.add_task(&child).unwrap();

  let memory: &cgroups_rs::memory::MemController = cgroup.controller_of().unwrap();
  memory.reset_max_usage().unwrap();
  memory.add_task(&child).unwrap();

  let pid: &cgroups_rs::pid::PidController = cgroup.controller_of().unwrap();
  pid.add_task(&child).unwrap();

  cgroup
}


/// 获取环境变量
/// 默认只传递 PATH 环境变量
fn get_env(params: &CatBoxParams) -> Vec<CString> {
  let path = format!("PATH={}", env::var("PATH").unwrap_or("".to_string()));
  let mut envs = vec![into_c_string(&path)];
  for (key, value) in params.env.iter() {
    let pair = format!("{}={}", key, value);
    envs.push(into_c_string(&pair));
  }
  envs
}


pub fn run(params: CatBoxParams) -> Result<(), String> {
  match unsafe { fork() } {
    Ok(ForkResult::Parent { child, .. }) => {
      info!("Start running child process (pid = {})", child);

      let cgroup = set_cgroup(&params, child);

      loop {
        let status = waitpid(child, None).unwrap();
        match status {
          WaitStatus::Exited(_, status) => {
            info!("Child process exited with status {}", status);
            break;
          }
          WaitStatus::Signaled(_, signal, _) => {
            info!("Child process is signaled by {}", signal);
            break;
          }
          WaitStatus::Stopped(_, signal) => {
            info!("Child process is stopped by {}", signal);
          }
          WaitStatus::PtraceEvent(_, _, _) => {}
          WaitStatus::PtraceSyscall(_) => {}
          WaitStatus::Continued(_) => {}
          WaitStatus::StillAlive => {}
        }
      }

      let cpuacct: &cgroups_rs::cpuacct::CpuAcctController = cgroup.controller_of().unwrap();
      let acct = cpuacct.cpuacct();

      info!("usage: {}", acct.usage);
      info!("usage_sys: {}", acct.usage_sys);
      info!("usage_user: {}", acct.usage_user);

      let memory: &cgroups_rs::memory::MemController = cgroup.controller_of().unwrap();
      let memswap = memory.memswap();
      info!("memory swap: {}", memswap.max_usage_in_bytes);

      cpuacct.reset().unwrap();

      Ok(())
    }
    Ok(ForkResult::Child) => {
      // 重定向输入输出
      redirect_io(&params);

      // setrlimit
      set_resource_limit(&params);

      // 设置时钟
      set_alarm(&params);

      // execvpe 运行用户程序
      let program = into_c_string(&params.program);
      let path = program.clone();
      let path = path.as_ref();
      let args = params.arguments.iter().map(|p| into_c_string(p)).collect::<Vec<CString>>();
      let args = [vec![program], args].concat();
      let args = args.as_slice();
      let env = get_env(&params);

      let result = execvpe(path, &args, env.as_slice());
      if let Err(e) = result {
        error!("Execve user submission fails: {}", e.desc());
        info!("Submission path: {}", params.program);
        let args = args.iter().map(|cstr| cstr.to_string_lossy().into()).collect::<Vec<Box<str>>>();
        info!("Submission args: {}", args.join(" "));
      }

      unsafe { libc::_exit(0) };
    }
    Err(_) => Err("Fork failed".into()),
  }
}
