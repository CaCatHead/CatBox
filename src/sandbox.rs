use std::ffi::{c_uint, CString};

use libc_stdhandle::{stderr, stdin, stdout};
use log::{error, info};
use nix::libc;
use nix::libc::freopen;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{alarm, execve, fork, ForkResult};

use crate::CatBoxParams;
use crate::utils::into_c_string;

/// 设置子进程时钟 signal，运行时限 + 1 秒
fn set_alarm(params: &CatBoxParams) {
  let time_limit = (params.time_limit as f64 / 1000.0 as f64).ceil() as c_uint;
  alarm::set(time_limit + 1);
}


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


/// 获取环境变量
fn get_env(params: &CatBoxParams) -> Vec<CString> {
  let mut envs = vec![];
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

      loop {
        let status = waitpid(child, None).unwrap();
        match status {
          WaitStatus::Exited(_, status) => {
            info!("Child process exited with status {}", status);
            break Ok(());
          }
          WaitStatus::Signaled(_, signal, _) => {
            info!("Child process is signaled by {}", signal);
            break Ok(());
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
    }
    Ok(ForkResult::Child) => {
      // 重定向输入输出
      redirect_io(&params);

      // 设置时钟
      set_alarm(&params);

      // execve 运行用户程序
      let program = into_c_string(&params.program);
      let path = program.clone();
      let path = path.as_ref();
      let args = params.arguments.iter().map(|p| into_c_string(p)).collect::<Vec<CString>>();
      let args = [vec![program], args].concat();
      let args = args.as_slice();
      let env = get_env(&params);

      let result = execve(path, &args, env.as_slice());
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
