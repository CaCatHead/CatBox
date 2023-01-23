use log::{debug, error, info};
use std::env;
use std::fs::remove_dir_all;
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
  pub(crate) debug: bool,
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
      mounts: MountPoint::defaults(),
      env: vec![],
      stdin: String::from("/dev/null"),
      stdout: String::from("/dev/null"),
      stderr: String::from("/dev/null"),
      debug: false,
    }
  }

  pub fn time_limit(self: &mut Self, value: u64) -> &mut Self {
    self.time_limit = value;
    self
  }

  pub fn memory_limit(self: &mut Self, value: u64) -> &mut Self {
    self.memory_limit = value;
    self
  }

  pub fn stdin<PS: Into<String>>(self: &mut Self, path: PS) -> &mut Self {
    self.stdin = path.into();
    self
  }

  pub fn stdout<PS: Into<String>>(self: &mut Self, path: PS) -> &mut Self {
    self.stdout = path.into();
    self
  }

  pub fn stderr<PS: Into<String>>(self: &mut Self, path: PS) -> &mut Self {
    self.stdin = path.into();
    self
  }

  pub fn chroot(self: &mut Self, enable: bool) -> &mut Self {
    if enable {
      let temp = tempdir().unwrap();
      let temp = temp.into_path();
      self.chroot = Some(temp);
    } else {
      self.chroot = None;
    }
    self
  }

  pub fn mount_read<SP: Into<PathBuf>, DP: Into<PathBuf>>(
    self: &mut Self,
    src: SP,
    dst: DP,
  ) -> &mut Self {
    self.mounts.push(MountPoint::read(src.into(), dst.into()));
    self
  }

  pub fn mount_write<SP: Into<PathBuf>, DP: Into<PathBuf>>(
    self: &mut Self,
    src: SP,
    dst: DP,
  ) -> &mut Self {
    self.mounts.push(MountPoint::write(src.into(), dst.into()));
    self
  }

  pub fn env<KS: Into<String>, VS: Into<String>>(self: &mut Self, key: KS, value: VS) -> &mut Self {
    self.env.push((key.into(), value.into()));
    self
  }

  pub fn ptrace(self: &mut Self, syscall_filter: Option<SyscallFilter>) -> &mut Self {
    self.ptrace = syscall_filter;
    self
  }

  #[allow(unused)]
  pub fn debug(self: &mut Self) -> &mut Self {
    self.debug = true;
    self
  }

  pub fn close(self: Self) {
    if let Some(chroot) = self.chroot {
      if self.debug {
        debug!("Persist new root: {}", chroot.to_string_lossy());
      } else {
        match remove_dir_all(&chroot) {
          Ok(_) => {
            info!("Remov new root: {}", chroot.to_string_lossy())
          }
          Err(_) => {
            error!("Fails removing new root: {}", chroot.to_string_lossy())
          }
        }
      }
    }
  }
}

impl MountPoint {
  pub fn defaults() -> Vec<Self> {
    vec![
      Self::read(PathBuf::from("/bin"), PathBuf::from("/bin")),
      Self::read(PathBuf::from("/usr"), PathBuf::from("/usr")),
      Self::read(PathBuf::from("/lib"), PathBuf::from("/lib")),
      Self::read(PathBuf::from("/lib64"), PathBuf::from("/lib64")),
    ]
  }

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
