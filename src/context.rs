use std::env;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

use log::{debug, error, info};
use nix::mount::{umount2, MntFlags};
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
  pub(crate) stdin: Option<String>,
  pub(crate) stdout: Option<String>,
  pub(crate) stderr: Option<String>,
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
  pub fn new<PS: Into<String>>(program: PS, arguments: Vec<String>) -> Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    let cgroup = env::var("CATJ_CGROUP").unwrap_or(current_user.name);

    let catbox_user = User::from_name("nobody").unwrap().unwrap();
    let catbox_group = Group::from_name("nogroup").unwrap().unwrap();

    CatBoxParams {
      time_limit: 1000,
      memory_limit: 262144,
      program: program.into(),
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
      env: vec![(
        "PATH".to_string(),
        env::var("PATH").unwrap_or("".to_string()),
      )],
      stdin: None,
      stdout: None,
      stderr: None,
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

  pub fn current_user(self: &mut Self) -> &mut Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    self.uid = current_user.uid;
    self.gid = current_user.gid;
    self
  }

  pub fn uid(self: &mut Self, uid: u32) -> &mut Self {
    self.uid = Uid::from(uid);
    self
  }

  pub fn gid(self: &mut Self, gid: u32) -> &mut Self {
    self.gid = Gid::from(gid);
    self
  }

  pub fn process(self: &mut Self, value: u64) -> &mut Self {
    self.process = value;
    self
  }

  pub fn stdin<PS: Into<String>>(self: &mut Self, path: Option<PS>) -> &mut Self {
    self.stdin = path.map(|p| p.into());
    self
  }

  pub fn stdout<PS: Into<String>>(self: &mut Self, path: Option<PS>) -> &mut Self {
    self.stdout = path.map(|p| p.into());
    self
  }

  pub fn stderr<PS: Into<String>>(self: &mut Self, path: Option<PS>) -> &mut Self {
    self.stderr = path.map(|p| p.into());
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

  pub fn parse_mount_read(self: &mut Self, text: String) -> Result<&mut Self, String> {
    let mount_point = MountPoint::parse_read(text)?;
    self.mounts.push(mount_point);
    Ok(self)
  }

  pub fn parse_mount_write(self: &mut Self, text: String) -> Result<&mut Self, String> {
    let mount_point = MountPoint::parse_write(text)?;
    self.mounts.push(mount_point);
    Ok(self)
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

  pub fn parse_env(self: &mut Self, text: String) -> Result<&mut Self, String> {
    let arr = text.split("=").collect::<Vec<&str>>();
    if arr.len() == 2 {
      let key = arr.get(0).unwrap();
      let value = arr.get(1).unwrap();
      Ok(self.env(*key, *value))
    } else {
      Err("Wrong environment variable string format".to_string())
    }
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
    if let Some(new_root) = self.chroot {
      if self.debug {
        debug!("Persist new root: {}", new_root.to_string_lossy());
      } else {
        let mut has_mount = false;
        for mount_point in &self.mounts {
          let target = mount_point.dst().strip_prefix(Path::new("/")).unwrap();
          let target = new_root.join(target);
          if target.exists() {
            debug!("Unmount directory {:?}", &target);
            if let Err(err) = umount2(&target, MntFlags::MNT_FORCE | MntFlags::MNT_DETACH) {
              error!("Fails umount {}: {}", target.to_string_lossy(), err);
            } else {
              has_mount = true;
            }
          }
        }
        if new_root.exists() {
          if has_mount {
            if let Err(err) = umount2(&new_root, MntFlags::MNT_FORCE | MntFlags::MNT_DETACH) {
              error!("Fails umount {}: {}", new_root.to_string_lossy(), err);
            }
          }

          match remove_dir_all(&new_root) {
            Ok(_) => {
              info!("Remove new root: {}", new_root.to_string_lossy());
            }
            Err(err) => {
              error!(
                "Fails removing new root: {} ({})",
                new_root.to_string_lossy(),
                err
              );
            }
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

  fn parse(write: bool, text: String) -> Result<Self, String> {
    let arr = text.split(":").collect::<Vec<&str>>();
    if arr.len() == 1 {
      let p = arr.get(0).unwrap();
      Ok(MountPoint {
        write,
        src: PathBuf::from(p),
        dst: PathBuf::from(p),
      })
    } else if arr.len() == 2 {
      let src = arr.get(0).unwrap();
      let dst = arr.get(1).unwrap();
      Ok(MountPoint {
        write,
        src: PathBuf::from(src),
        dst: PathBuf::from(dst),
      })
    } else {
      Err("Wrong mount string format".to_string())
    }
  }

  pub fn parse_read(text: String) -> Result<Self, String> {
    Self::parse(false, text)
  }

  pub fn parse_write(text: String) -> Result<Self, String> {
    Self::parse(true, text)
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
