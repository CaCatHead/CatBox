use std::error::Error;
use std::ffi::{c_uint, CString};
use std::fs::create_dir_all;

use std::path::{Path, PathBuf};

use libc_stdhandle::{stderr, stdin, stdout};
use log::{debug, error, info};

use nix::libc::RLIM_INFINITY;
use nix::libc::{self, freopen};
use nix::mount::{mount, MsFlags};
use nix::sys::ptrace;
use nix::sys::resource::{setrlimit, Resource};
use nix::sys::signal::Signal;

use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{alarm, chdir, chroot, execvpe, fork, setgid, setuid, ForkResult};

use crate::cgroup::CatBoxCgroup;
use crate::context::CatBoxResult;
use crate::pipe::CatBoxPipe;
use crate::utils::into_c_string;
use crate::CatBoxParams;

/// 重定向输出输出
fn redirect_io(params: &CatBoxParams) -> Result<(), Box<dyn Error>> {
  unsafe {
    if let Some(in_path) = &params.stdin {
      let in_path = into_c_string(&in_path);
      let mode = CString::new("r").unwrap();
      freopen(in_path.as_ptr(), mode.as_ptr(), stdin());
    }

    if let Some(out_path) = &params.stdout {
      let out_path = into_c_string(&out_path);
      let mode = CString::new("w").unwrap();
      freopen(out_path.as_ptr(), mode.as_ptr(), stdout());
    }

    if let Some(err_path) = &params.stderr {
      let err_path = into_c_string(&err_path);
      let mode = CString::new("w").unwrap();
      freopen(err_path.as_ptr(), mode.as_ptr(), stderr());
    }
  }

  // debug!("Redirect /dev/null");
  // let null_fd = open(
  //   "/dev/null",
  //   OFlag::O_RDONLY | OFlag::O_WRONLY,
  //   Mode::empty(),
  // )
  // .unwrap();

  // info!("Redirect child process stdin to {}", &params.stdin);
  // let stdin_fd = if params.stdin != "/dev/null" {
  //   let file = Path::new(params.stdin.as_str());
  //   open(file, OFlag::O_RDONLY, Mode::empty()).unwrap()
  // } else {
  //   null_fd.clone()
  // };
  // match dup2(stdin_fd, STDIN_FILENO) {
  //   Ok(_) => {
  //     if stdin_fd != null_fd {
  //       close(stdin_fd)?;
  //     }
  //   }
  //   Err(err) => {
  //     error!("Redirect stdin fails: {}", err);
  //   }
  // };

  // info!("Redirect child process stdout to {}", &params.stdout);
  // let stdout_fd = if params.stdin != "/dev/null" {
  //   let file = Path::new(params.stdout.as_str());
  //   open(
  //     file,
  //     OFlag::O_WRONLY | OFlag::O_TRUNC | OFlag::O_CREAT,
  //     Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IWGRP,
  //   )
  //   .unwrap()
  // } else {
  //   null_fd.clone()
  // };
  // match dup2(stdout_fd, STDOUT_FILENO) {
  //   Ok(_) => {
  //     if stdout_fd != null_fd {
  //       close(stdout_fd)?;
  //     }
  //   }
  //   Err(err) => {
  //     error!("Redirect stdout fails: {}", err);
  //   }
  // };

  // info!("Redirect child process stderr to {}", &params.stderr);
  // let stderr_fd = if params.stdin != "/dev/null" {
  //   let file = Path::new(params.stderr.as_str());
  //   open(
  //     file,
  //     OFlag::O_WRONLY | OFlag::O_TRUNC | OFlag::O_CREAT,
  //     Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IWGRP,
  //   )
  //   .unwrap()
  // } else {
  //   null_fd.clone()
  // };
  // // debug 状态下不重定向 stderr 到 /dev/null，否则子进程看不到输出
  // if params.stderr != "/dev/null" || !params.debug {
  //   match dup2(stderr_fd, STDERR_FILENO) {
  //     Ok(_) => {
  //       if stderr_fd != null_fd {
  //         close(stderr_fd)?;
  //       }
  //     }
  //     Err(err) => {
  //       error!("Redirect stderr fails: {}", err);
  //     }
  //   }
  // }

  // close(null_fd)?;

  info!("Redirect child process IO ok");

  Ok(())
}

/// 设置子进程时钟 signal，运行时限 + 1 秒
fn set_alarm(params: &CatBoxParams) {
  let time_limit = (params.time_limit as f64 / 1000.0 as f64).ceil() as c_uint;
  alarm::set(time_limit + 1);
  debug!("Set alarm {} seconds", time_limit + 1);
}

/// 调用 setrlimit
fn set_resource_limit(params: &CatBoxParams) -> Result<(), Box<dyn Error>> {
  // 运行时限
  let time_limit = (params.time_limit as f64 / 1000.0 as f64).ceil() as u64;
  setrlimit(Resource::RLIMIT_CPU, time_limit + 1, time_limit + 1)?;

  // 地址空间无限
  setrlimit(Resource::RLIMIT_AS, RLIM_INFINITY, RLIM_INFINITY)?;

  let stack_size = if params.stack_size == u64::MAX {
    RLIM_INFINITY
  } else {
    params.stack_size
  };
  debug!("Set stack size {} bytes", stack_size);
  setrlimit(Resource::RLIMIT_STACK, stack_size, stack_size)?;

  // 输出大小 256 MB
  let fsize = 256 * 1024 * 1024 as u64;
  setrlimit(Resource::RLIMIT_FSIZE, fsize, fsize)?;

  Ok(())
}

/// chroot
fn change_root(new_root: &PathBuf, params: &CatBoxParams) -> Result<(), Box<dyn Error>> {
  info!("Mount new root: {}", new_root.to_string_lossy());

  mount::<PathBuf, PathBuf, PathBuf, PathBuf>(
    Some(new_root),
    new_root,
    None,
    MsFlags::MS_BIND | MsFlags::MS_REC,
    None,
  )?;

  mount::<PathBuf, PathBuf, PathBuf, PathBuf>(
    None,
    new_root,
    None,
    MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_REC,
    None,
  )?;

  for mount_point in &params.mounts {
    if !mount_point.dst().is_absolute() {
      error!(
        "The dst path {} in mounts should be absolute",
        mount_point.dst().to_string_lossy()
      );
      continue;
    }
    if !mount_point.dst().is_dir() {
      error!(
        "The dst path {} in mounts should be a directory",
        mount_point.dst().to_string_lossy()
      );
      continue;
    }

    let target = mount_point.dst().strip_prefix(Path::new("/")).unwrap();
    let target = new_root.join(target);
    create_dir_all(&target)?;
    debug!("Mount directory {:?} -> {:?}", mount_point.src(), &target);

    mount::<PathBuf, PathBuf, PathBuf, PathBuf>(
      Some(mount_point.src()),
      &target,
      None,
      MsFlags::MS_BIND | MsFlags::MS_REC,
      None,
    )?;
    if mount_point.read_only() {
      mount::<PathBuf, PathBuf, PathBuf, PathBuf>(
        None,
        &target,
        None,
        MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_REC,
        None,
      )?;
    }
  }

  chroot(new_root)?;
  chdir(Path::new("/"))?;

  Ok(())
}

/// 获取环境变量
/// 默认只传递 PATH 环境变量
fn get_env(params: &CatBoxParams) -> Vec<CString> {
  let mut envs = vec![];
  for (key, value) in params.env.iter() {
    let pair = format!("{}={}", key, value);
    envs.push(into_c_string(&pair));
  }
  envs
}

pub fn run(params: &CatBoxParams) -> Result<CatBoxResult, Box<dyn Error>> {
  let pipe = CatBoxPipe::new()?;

  match unsafe { fork() } {
    Ok(ForkResult::Parent { child, .. }) => {
      let pipe = pipe.read()?;

      // 设置 cgroup
      let cgroup = CatBoxCgroup::new(&params, child);

      // 复制 SyscallFilter
      let mut filter = params.ptrace.clone();

      let (status, signal) = loop {
        let status = waitpid(child, None)?;

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
                info!(
                  "Child process #{}. is stopped by {} (may be time limit exceeded)",
                  pid, signal
                );
                ptrace::kill(pid)?;
                break (None, Some(signal));
              }
              // 处理系统调用
              Signal::SIGTRAP => {
                let user_regs = ptrace::getregs(pid)?;
                // let syscall_id = user_regs.orig_rax;
                // debug!(
                //   "Child process #{}. performed a syscall: {}",
                //   pid, syscall_id
                // );

                if let Some(filter) = &mut filter {
                  if filter.filter(&pid, &user_regs) {
                    ptrace::syscall(pid, None)?;
                  } else {
                    info!(
                      "Child process #{}. is stopped for forbidden syscall (id = {})",
                      pid, user_regs.orig_rax
                    );
                    ptrace::kill(pid)?;
                    break (None, Some(signal));
                  }
                } else {
                  ptrace::syscall(pid, None)?;
                }
              }
              // 因为各种原因 RE
              Signal::SIGKILL
              | Signal::SIGBUS
              | Signal::SIGFPE
              | Signal::SIGILL
              | Signal::SIGSEGV
              | Signal::SIGSYS
              | Signal::SIGXFSZ => {
                info!("Child process #{}. is stopped by {}", pid, signal);
                ptrace::kill(pid)?;
                break (None, Some(signal));
              }
              // 未捕获 SIGCONT，不是终端
              Signal::SIGCONT | Signal::SIGHUP | Signal::SIGINT => {
                unreachable!()
              }
              _ => {
                info!(
                  "Child process #{}. is stopped by an unhandled signal {}",
                  pid, signal
                );
                unimplemented!()
              }
            }
          }
          WaitStatus::PtraceSyscall(_) => {
            unreachable!()
          }
          WaitStatus::PtraceEvent(_, _, _) => {
            unreachable!()
          }
          WaitStatus::Continued(_) => {
            unreachable!()
          }
          WaitStatus::StillAlive => {
            unreachable!()
          }
        }
      };

      let message = pipe.read().ok();
      info!("Recv message: {:?}", message);
      pipe.close()?;

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
      info!("Child process is running");

      let pipe = pipe.write()?;

      // 重定向输入输出
      redirect_io(&params)?;

      // chroot
      if let Some(chroot) = &params.chroot {
        match change_root(chroot, &params) {
          Ok(_) => {
            debug!("Chroot ok: {}", chroot.to_string_lossy());
          }
          Err(err) => {
            error!("Chroot fails: {}", err);
          }
        }
      }

      // 设置时钟
      set_alarm(&params);

      // setrlimit
      set_resource_limit(&params)?;

      // 设置用户
      if let Err(err) = setgid(params.gid) {
        error!("Set gid {} fails: {}", params.gid, err);
      }
      if let Err(err) = setuid(params.uid) {
        error!("Set uid {} fails: {}", params.uid, err);
      }

      // execvpe 运行用户程序
      let program = into_c_string(&params.program);
      let path = program.clone();
      let path = path.as_ref();
      let args = params
        .arguments
        .iter()
        .map(|p| into_c_string(p))
        .collect::<Vec<CString>>();
      let args = [vec![program], args].concat();
      let args = args.as_slice();
      let env = get_env(&params);

      {
        let args = args
          .iter()
          .map(|cstr| cstr.to_string_lossy().into())
          .collect::<Vec<Box<str>>>();
        info!("Start running program {}", args.join(" "));
      }

      // 启动 ptrace 追踪子进程
      if params.ptrace.is_some() {
        ptrace::traceme().unwrap();
      }

      let result = execvpe(path, &args, env.as_slice());
      if let Err(e) = result {
        let sz = pipe.write(format!("Execvpe fails: {}", e.desc()))?;
        info!("Write sz: {}", sz);

        error!("Execvpe fails: {}", e.desc());
        info!("Submission path: {}", params.program);
        let args = args
          .iter()
          .map(|cstr| cstr.to_string_lossy().into())
          .collect::<Vec<Box<str>>>();
        info!("Submission args: {}", args.join(" "));

        pipe.close()?;
      }

      unsafe { libc::_exit(0) };
    }
    Err(_) => Err(Box::<dyn Error>::from("Fork failed")),
  }
}
