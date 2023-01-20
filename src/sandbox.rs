use std::ffi::{c_uint, CString};
use std::path::PathBuf;

use log::{error, info};
use nix::libc;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{alarm, execve, fork, ForkResult};

#[derive(Debug)]
pub struct CatBoxParams {
  pub time_limit: u64,
  pub memory_limit: u64,
  pub program: PathBuf,
  pub arguments: Vec<String>,
}

// impl RunCommand {
//   fn get(&self) -> std::process::Command {
//     let input = match self {
//       RunCommand::List(v) => v,
//     };
//     let program = input.first().unwrap();
//     let args = input.iter().skip(1).collect::<Vec<&String>>();
//     let mut command = std::process::Command::new(program.clone());
//     command.args(args.clone());
//     command
//   }
// }

pub fn run(params: CatBoxParams) -> Result<(), String> {
  match unsafe { fork() } {
    Ok(ForkResult::Parent { child, .. }) => {
      info!("Start running child process (pid = {})", child);

      loop {
        let status = waitpid(child, None).unwrap();
        match status {
          WaitStatus::Exited(_, _) => {
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
      // 设置时钟，time_limit + 1 秒后，结束
      let time_limit = (params.time_limit as f64 / 1000.0 as f64).ceil() as c_uint;
      alarm::set(time_limit + 1);

      // execve 运行用户程序
      let program = CString::new(params.program.to_str().unwrap()).unwrap();
      let path = program.clone();
      let path = path.as_ref();
      let args = params.arguments.iter().map(|p| CString::new(p.as_str()).unwrap()).collect::<Vec<CString>>();
      let args = [vec![program], args].concat();
      let args = args.as_slice();
      let env: [&CString; 0] = [];

      let result = execve(path, &args, &env);
      if let Err(e) = result {
        error!("Execve user submission fails: {}", e.desc());
        info!("Submission path: {}", params.program.to_string_lossy());
        let args = args.iter().map(|cstr| cstr.to_string_lossy().into()).collect::<Vec<Box<str>>>();
        info!("Submission args: {}", args.join(" "));
      }

      unsafe { libc::_exit(0) };
    }
    Err(_) => Err("Fork failed".into()),
  }
}
