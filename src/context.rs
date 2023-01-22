use std::env;

use nix::sys::signal::Signal;
use nix::unistd::{Gid, Uid, User};

use crate::syscall::SyscallFilter;

#[allow(unused)]
#[derive(Debug)]
pub struct CatBoxParams {
  pub time_limit: u64,
  pub memory_limit: u64,
  pub program: String,
  pub arguments: Vec<String>,
  pub(crate) uid: Uid,
  pub(crate) gid: Gid,
  pub(crate) cgroup: String,
  pub(crate) process: u64,
  pub(crate) ptrace: Option<SyscallFilter>,
  pub(crate) stack_size: u64,
  pub(crate) chroot: bool,
  pub(crate) mounts: Vec<MountPoint>,
  pub(crate) env: Vec<(String, String)>,
  pub(crate) stdin: String,
  pub(crate) stdout: String,
  pub(crate) stderr: String,
}

#[allow(unused)]
#[derive(Debug)]
pub struct MountPoint {
  write: bool,
  src: String,
  dst: String,
}

impl CatBoxParams {
  pub fn new(program: String, arguments: Vec<String>) -> Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    let cgroup = env::var("CATJ_CGROUP").unwrap_or(current_user.name);

    // let catbox_user = User::from_name("nobody").unwrap().unwrap();

    CatBoxParams {
      time_limit: 1000,
      memory_limit: 262144,
      program,
      arguments,
      uid: current_user.uid,
      gid: current_user.gid,
      cgroup,
      process: 1,
      ptrace: Some(SyscallFilter::default()),
      stack_size: u64::MAX,
      chroot: false,
      mounts: vec![],
      env: vec![],
      stdin: String::from("/dev/null"),
      stdout: String::from("/dev/null"),
      stderr: String::from("/dev/null"),
    }
  }

  pub fn stdin(self: &mut Self, path: String) -> &mut Self {
    self.stdin = path;
    self
  }

  pub fn stdout(self: &mut Self, path: String) -> &mut Self {
    self.stdout = path;
    self
  }

  pub fn stderr(self: &mut Self, path: String) -> &mut Self {
    self.stdin = path;
    self
  }

  pub fn chroot(self: &mut Self, flag: bool) -> &mut Self {
    self.chroot = flag;
    self
  }

  pub fn env(self: &mut Self, key: String, value: String) -> &mut Self {
    self.env.push((key, value));
    self
  }

  pub fn ptrace(self: &mut Self, syscall_filter: Option<SyscallFilter>) -> &mut Self {
    self.ptrace = syscall_filter;
    self
  }
}

#[allow(unused)]
pub struct CatBoxResult {
  pub(crate) status: Option<i32>,
  pub(crate) signal: Option<Signal>,
  pub(crate) time: u64,
  pub(crate) time_user: u64,
  pub(crate) time_sys: u64,
  pub(crate) memory: u64,
}
