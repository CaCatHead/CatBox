use std::env;
use std::ffi::{c_uint, CString};

use libc_stdhandle::{stderr, stdin, stdout};
use log::{debug, error, info};
use nix::libc;
use nix::libc::{freopen, RLIM_INFINITY};
use nix::sys::ptrace;
use nix::sys::resource::{Resource, setrlimit};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{alarm, execvpe, fork, ForkResult};

use crate::CatBoxParams;
use crate::cgroup::CatBoxCgroup;
use crate::context::CatBoxResult;
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


pub fn run(params: CatBoxParams) -> Result<CatBoxResult, String> {
  match unsafe { fork() } {
    Ok(ForkResult::Parent { child, .. }) => {
      info!("Start running child process (pid = {})", child);

      // 设置 cgroup
      let cgroup = CatBoxCgroup::new(&params, child);

      // 复制 SyscallFilter
      let mut filter = params.ptrace.clone();

      let (status, signal) = loop {
        let status = waitpid(child, None).unwrap();
        match status {
          WaitStatus::Exited(pid, status) => {
            info!("Child process #{}. exited with status {}", pid, status);
            break (Some(status), None);
          }
          WaitStatus::Signaled(pid, signal, _) => {
            info!("Child process #{}. is signaled by {}", pid, signal);
            break (None, Some(signal));
          }
          WaitStatus::Stopped(pid, signal) => {
            // 完整 Signal 定义见：https://man7.org/linux/man-pages/man7/signal.7.html
            match signal {
              // 可能是超时了
              Signal::SIGALRM | Signal::SIGVTALRM | Signal::SIGXCPU => {
                info!("Child process #{}. is stopped by {} (may be time limit exceeded)", pid, signal);
                ptrace::kill(pid).unwrap();
                break (None, Some(signal));
              }
              // 处理系统调用
              Signal::SIGTRAP => {
                let user_regs = ptrace::getregs(pid).unwrap();
                let syscall_id = user_regs.orig_rax;
                debug!("Child process #{}. performed a syscall: {}", pid, syscall_id);

                if let Some(filter) = &mut filter {
                  if filter.filter(&pid, &user_regs) {
                    ptrace::syscall(pid, None).unwrap();
                  } else {
                    info!("Child process #{}. is stopped for forbidden syscall (id = {})", pid, user_regs.orig_rax);
                    ptrace::kill(pid).unwrap();
                    break (None, Some(signal));
                  }
                } else {
                  ptrace::syscall(pid, None).unwrap();
                }
              }
              // 因为各种原因 RE
              Signal::SIGKILL | Signal::SIGBUS | Signal::SIGFPE | Signal::SIGILL | Signal::SIGSEGV | Signal::SIGSYS | Signal::SIGXFSZ => {
                info!("Child process #{}. is stopped by {}", pid, signal);
                ptrace::kill(pid).unwrap();
                break (None, Some(signal));
              }
              // 未捕获 SIGCONT，不是终端
              Signal::SIGCONT | Signal::SIGHUP | Signal::SIGINT => {
                unreachable!()
              }
              _ => {
                info!("Child process #{}. is stopped by an unhandled signal {}", pid, signal);
                unimplemented!()
              }
            }
          }
          WaitStatus::PtraceSyscall(pid) => { unreachable!() }
          WaitStatus::PtraceEvent(_, _, _) => { unreachable!() }
          WaitStatus::Continued(_) => { unreachable!() }
          WaitStatus::StillAlive => { unreachable!() }
        }
      };

      let usage = cgroup.usage();
      info!("{:?}", usage);

      Ok(CatBoxResult {
        status,
        signal,
        time: usage.time(),
        time_user: usage.time_user(),
        time_sys: usage.time_sys(),
        memory: usage.memory_swap(),
      })
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

      // 启动 ptrace 追踪子进程
      if params.ptrace.is_some() {
        ptrace::traceme().unwrap();
      }

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
