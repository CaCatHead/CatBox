use std::env;
use std::path::PathBuf;

use nix::sys::signal::Signal;
use nix::unistd::{Gid, Group, Uid, User};
use tempfile::tempdir;

use crate::syscall::SyscallFilter;

#[allow(unused)]
#[derive(Debug, Clone)]
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
  pub(crate) chroot: Option<PathBuf>,
  pub(crate) cwd: PathBuf,
  pub(crate) mounts: Vec<MountPoint>,
  pub(crate) env: Vec<(String, String)>,
  pub(crate) stdin: String,
  pub(crate) stdout: String,
  pub(crate) stderr: String,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MountPoint {
  write: bool,
  src: PathBuf,
  dst: PathBuf,
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

impl CatBoxParams {
  pub fn new(program: String, arguments: Vec<String>) -> Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    let cgroup = env::var("CATJ_CGROUP").unwrap_or(current_user.name);

    let catbox_user = User::from_name("nobody").unwrap().unwrap();
    let catbox_group = Group::from_name("nogroup").unwrap().unwrap();

    CatBoxParams {
      time_limit: 1000,
      memory_limit: 262144,
      program,
      arguments,
      uid: catbox_user.uid,
      gid: catbox_group.gid,
      cgroup,
      process: 1,
      ptrace: Some(SyscallFilter::default()),
      stack_size: u64::MAX,
      chroot: None,
      cwd: env::current_dir().unwrap(),
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
    if flag {
      let temp = tempdir().unwrap();
      self.chroot = Some(temp.into_path());
    } else {
      self.chroot = None;
    }
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

impl MountPoint {
  pub fn read(src: PathBuf, dst: PathBuf) -> Self {
    MountPoint {
      write: false,
      src,
      dst,
    }
  }

  pub fn write(src: PathBuf, dst: PathBuf) -> Self {
    MountPoint {
      write: true,
      src,
      dst,
    }
  }

  pub fn read_only(&self) -> bool {
    !self.write
  }

  pub fn src(&self) -> &PathBuf {
    &self.src
  }

  pub fn dst(&self) -> &PathBuf {
    &self.dst
  }
}
