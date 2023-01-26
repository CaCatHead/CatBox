use std::env;
use std::ffi::CString;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::slice::Iter;

use log::{debug, error, info};
use nix::libc;
use nix::libc::{gid_t, rlim_t, uid_t, STDOUT_FILENO};
use nix::mount::{umount2, MntFlags};
use nix::sys::signal::Signal;
use nix::unistd::{isatty, Gid, Group, Uid, User};
use tempfile::tempdir;

use crate::syscall::SyscallFilter;
use crate::utils::mount::MountPoint;
use crate::utils::{into_c_string, parse_env};
use crate::CatBoxError;
use crate::cgroup::CatBoxUsage;

pub type TimeLimitType = u64;

pub type MemoryLimitType = u64;

pub type UidType = uid_t;

pub type GidType = gid_t;

/// Build CatBox
pub struct CatBoxBuilder {
  context: Box<dyn CatBoxContext>,
  options: Vec<CatBoxOption>,
  env: Vec<(String, String)>,
  force: Option<bool>,
  time_limit: Option<TimeLimitType>,
  memory_limit: Option<MemoryLimitType>,
  uid: Option<UidType>,
  gid: Option<GidType>,
}

/// CatBox top structure for context and multiple commands
pub struct CatBox {
  context: Box<dyn CatBoxContext>,
  options: Vec<CatBoxOption>,
}

/// CatBoxContext for storing running result
pub trait CatBoxContext {
  fn report(&self) {
    let is_tty = isatty(STDOUT_FILENO).unwrap_or(false);
    if is_tty {
      self.report_human();
    } else {
      self.report_json();
    }
  }

  fn report_human(&self);

  fn report_json(&self);
}

pub struct CatBoxRunContext {}

pub struct CatBoxCompileContext {}

pub struct CatBoxJudgeContext {}

/// Build CatBox running option
pub struct CatBoxOptionBuilder {
  parent: CatBoxBuilder,
  option: CatBoxOption,
}

/// CatBox running params that can config its behavior
#[derive(Debug, Clone)]
pub struct CatBoxOption {
  time_limit: TimeLimitType,
  memory_limit: MemoryLimitType,
  program: String,
  arguments: Vec<String>,
  uid: Uid,
  gid: Gid,
  cgroup: String,
  process: u64,
  ptrace: Option<SyscallFilter>,
  stack_size: u64,
  chroot: Option<PathBuf>,
  cwd: PathBuf,
  mounts: Vec<MountPoint>,
  env: Vec<(String, String)>,
  stdin: Option<String>,
  stdout: Option<String>,
  stderr: Option<String>,
  force: bool,
  debug: bool,
}

/// CatBox running result
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CatBoxResult {
  status: Option<i32>,
  signal: Option<Signal>,
  time: TimeLimitType,
  time_user: TimeLimitType,
  time_sys: TimeLimitType,
  memory: MemoryLimitType,
}

impl CatBox {
  /// List all the commands
  pub fn commands(&self) -> Iter<CatBoxOption> {
    self.options.iter()
  }

  /// Return the only command when there is just one command
  pub fn single(&self) -> Option<&CatBoxOption> {
    if self.options.len() == 1 {
      self.options.first()
    } else {
      None
    }
  }

  /// Add result
  pub fn add_result(&self) {

  }

  /// Report usage
  pub fn report(&self) {
    self.context.report();
  }

  /// Report json format usage
  pub fn report_json(&self) {
    self.context.report_json();
  }

  /// Close all the CatBoxes
  pub fn close(self) {
    for option in self.options.into_iter() {
      option.close();
    }
  }
}

impl CatBoxBuilder {
  /// Create a new CatBox with a context
  pub fn new(context: Box<dyn CatBoxContext>) -> Self {
    CatBoxBuilder {
      context,
      options: vec![],
      env: vec![],
      force: None,
      time_limit: None,
      memory_limit: None,
      uid: None,
      gid: None,
    }
  }

  /// Create a run CatBox
  pub fn run() -> Self {
    Self::new(Box::new(CatBoxRunContext {}))
  }

  /// Create a compile CatBox
  pub fn compile() -> Self {
    Self::new(Box::new(CatBoxCompileContext {}))
  }

  /// Create a new command to be run
  pub fn command<PS: Into<String>, AS: Into<String>>(
    self,
    program: PS,
    arguments: Vec<AS>,
  ) -> CatBoxOptionBuilder {
    let mut option = CatBoxOption::default(
      program.into(),
      arguments.into_iter().map(|a| a.into()).collect(),
    );

    // Set default time limit
    if let Some(time_limit) = self.time_limit {
      option.time_limit = time_limit;
    }
    // Set default memory limit
    if let Some(memory_limit) = self.memory_limit {
      option.memory_limit = memory_limit;
    }
    // Set default force mode
    if let Some(force) = self.force {
      option.force = force;
    }
    // Set default uid
    if let Some(uid) = self.uid {
      option.uid = Uid::from(uid);
    }
    // Set default uid
    if let Some(gid) = self.gid {
      option.gid = Gid::from(gid);
    }
    // Set default env
    for env_pair in self.env.iter() {
      option.env.push(env_pair.clone());
    }

    CatBoxOptionBuilder {
      parent: self,
      option,
    }
  }

  /// Build CatBox after setting all the options
  pub fn build(mut self) -> CatBox {
    CatBox {
      context: self.context,
      options: self.options,
    }
  }

  /// Set default time limit
  pub fn set_default_time_limit(mut self, value: Option<TimeLimitType>) -> Self {
    self.time_limit = value;
    self
  }

  /// Set default memory limit
  pub fn set_default_memory_limit(mut self, value: Option<MemoryLimitType>) -> Self {
    self.memory_limit = value;
    self
  }

  /// Set default force mode
  pub fn set_default_force(mut self, flag: Option<bool>) -> Self {
    self.force = flag;
    self
  }

  /// Set default uid
  pub fn set_default_uid(mut self, uid: Option<UidType>) -> Self {
    self.uid = uid;
    self
  }

  /// Set default gid
  pub fn set_default_gid(mut self, gid: Option<GidType>) -> Self {
    self.gid = gid;
    self
  }

  /// Set current user
  pub fn set_current_user(mut self, flag: bool) -> Self {
    if flag {
      let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
      self.uid = Some(current_user.uid.as_raw());
      self.gid = Some(current_user.gid.as_raw());
    }
    self
  }

  /// Parse default env list
  pub fn parse_env_list(mut self, list: Vec<String>) -> Result<Self, CatBoxError> {
    for env_var in list {
      self.env.push(parse_env(env_var)?);
    }
    Ok(self)
  }
}

impl CatBoxContext for CatBoxRunContext {
  fn report_human(&self) {
    todo!()
  }

  fn report_json(&self) {
    todo!()
  }
}

impl CatBoxContext for CatBoxCompileContext {
  fn report_human(&self) {
    todo!()
  }

  fn report_json(&self) {
    todo!()
  }
}

impl CatBoxOptionBuilder {
  /// Finish building, return CatBoxBuilder
  pub fn done(mut self) -> CatBoxBuilder {
    let mut builder = self.parent;
    builder.options.push(self.option);
    builder
  }

  /// Finish building, return CatBox
  pub fn build(mut self) -> CatBox {
    let builder = self.done();
    builder.build()
  }

  /// Set time limit (unit: ms)
  pub fn time_limit(mut self, value: TimeLimitType) -> Self {
    self.option.time_limit = value;
    self
  }

  /// Set memory limit (unit: KB)
  pub fn memory_limit(mut self, value: MemoryLimitType) -> Self {
    self.option.memory_limit = value;
    self
  }

  /// Set uid
  pub fn uid(mut self, uid: UidType) -> Self {
    self.option.uid = Uid::from(uid);
    self
  }

  /// Set gid
  pub fn gid(mut self, gid: GidType) -> Self {
    self.option.gid = Gid::from(gid);
    self
  }

  /// Set uid / gid with current user
  pub fn current_user(mut self) -> Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    self.option.uid = current_user.uid;
    self.option.gid = current_user.gid;
    self
  }

  /// Set the max number of processes
  pub fn process(mut self, value: u64) -> Self {
    self.option.process = value;
    self
  }

  /// Set the max number of processes or do nothing
  pub fn set_process(mut self, value: Option<u64>) -> Self {
    if let Some(value) = value {
      self.option.process = value;
    }
    self
  }

  /// Set stdin redirection or not
  pub fn set_stdin<PS: Into<String>>(mut self, path: Option<PS>) -> Self {
    self.option.stdin = path.map(|p| p.into());
    self
  }

  /// Set stdin redirection
  pub fn stdin<PS: Into<String>>(mut self, path: PS) -> Self {
    self.option.stdin = Some(path.into());
    self
  }

  /// Set stdout redirection or not
  pub fn set_stdout<PS: Into<String>>(mut self, path: Option<PS>) -> Self {
    self.option.stdout = path.map(|p| p.into());
    self
  }

  /// Set stdout redirection
  pub fn stdout<PS: Into<String>>(mut self, path: PS) -> Self {
    self.option.stdout = Some(path.into());
    self
  }

  /// Set stderr redirection or not
  pub fn set_stderr<PS: Into<String>>(mut self, path: Option<PS>) -> Self {
    self.option.stderr = path.map(|p| p.into());
    self
  }

  /// Set stderr redirection
  pub fn stderr<PS: Into<String>>(mut self, path: PS) -> Self {
    self.option.stderr = Some(path.into());
    self
  }

  /// Set chroot or not
  pub fn set_chroot(mut self, flag: bool) -> Self {
    if flag {
      self.chroot()
    } else {
      self.option.chroot = None;
      self
    }
  }

  /// Set ptrace syscall filter
  pub fn set_ptrace(mut self, flag: bool) -> Self {
    if flag {
      self.option.ptrace = Some(SyscallFilter::default());
    } else {
      self.option.ptrace = None;
    }
    self
  }

  /// Enable chroot
  pub fn chroot(mut self) -> Self {
    let temp = tempdir().unwrap();
    let temp = temp.into_path();
    self.option.chroot = Some(temp);
    self
  }

  /// Set work directory in chroot
  pub fn cwd<P: Into<PathBuf>>(mut self, path: P) -> Self {
    self.option.cwd = path.into();
    self
  }

  /// Mount read directory
  pub fn mount_read<SP: Into<PathBuf>, DP: Into<PathBuf>>(mut self, src: SP, dst: DP) -> Self {
    self
      .option
      .mounts
      .push(MountPoint::read(src.into(), dst.into()));
    self
  }

  /// Mount write directory
  pub fn mount_write<SP: Into<PathBuf>, DP: Into<PathBuf>>(mut self, src: SP, dst: DP) -> Self {
    self
      .option
      .mounts
      .push(MountPoint::write(src.into(), dst.into()));
    self
  }

  /// Parse read mount points
  pub fn parse_mount_read(mut self, list: Vec<String>) -> Result<Self, CatBoxError> {
    for text in list {
      let mount_point = MountPoint::parse_read(text)?;
      self.option.mounts.push(mount_point);
    }
    Ok(self)
  }

  /// Parse write mount points
  pub fn parse_mount_write(mut self, list: Vec<String>) -> Result<Self, CatBoxError> {
    for text in list {
      let mount_point = MountPoint::parse_write(text)?;
      self.option.mounts.push(mount_point);
    }
    Ok(self)
  }

  /// Pass env
  pub fn env<KS: Into<String>, VS: Into<String>>(mut self, key: KS, value: VS) -> Self {
    self.option.env.push((key.into(), value.into()));
    self
  }
}

impl CatBoxOption {
  pub fn default<PS: Into<String>, AS: Into<String>>(program: PS, arguments: Vec<AS>) -> Self {
    let current_user = User::from_uid(Uid::current()).unwrap().unwrap();
    let cgroup = env::var("CATJ_CGROUP").unwrap_or(current_user.name);

    let catbox_user = User::from_name("nobody").unwrap().unwrap();
    let catbox_group = Group::from_name("nogroup").unwrap().unwrap();

    CatBoxOption {
      time_limit: 1000,
      memory_limit: 262144,
      program: program.into(),
      arguments: arguments.into_iter().map(|a| a.into()).collect(),
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
      force: false,
      debug: false,
    }
  }

  pub fn time_limit(&self) -> TimeLimitType {
    self.time_limit
  }

  pub fn memory_limit(&self) -> MemoryLimitType {
    self.memory_limit
  }

  pub fn program(&self) -> CString {
    into_c_string(&self.program)
  }

  pub fn arguments(&self) -> Vec<CString> {
    self.arguments.iter().map(|p| into_c_string(p)).collect()
  }

  pub fn uid(&self) -> Uid {
    self.uid
  }

  pub fn gid(&self) -> Gid {
    self.gid
  }

  pub fn cgroup(&self) -> &str {
    &self.cgroup
  }

  pub fn process(&self) -> u64 {
    self.process
  }

  pub fn ptrace(&self) -> &Option<SyscallFilter> {
    &self.ptrace
  }

  pub fn stack_size(&self) -> rlim_t {
    if self.stack_size == u64::MAX {
      libc::RLIM_INFINITY
    } else {
      self.stack_size
    }
  }

  pub fn chroot(&self) -> &Option<PathBuf> {
    &self.chroot
  }

  pub fn cwd(&self) -> &PathBuf {
    &self.cwd
  }

  pub fn mounts(&self) -> &Vec<MountPoint> {
    &self.mounts
  }

  pub fn env(&self) -> &Vec<(String, String)> {
    &self.env
  }

  pub fn stdin(&self) -> &Option<String> {
    &self.stdin
  }

  pub fn stdout(&self) -> &Option<String> {
    &self.stdout
  }

  pub fn stderr(&self) -> &Option<String> {
    &self.stderr
  }

  pub fn force(&self) -> bool {
    self.force
  }

  pub fn debug(&self) -> bool {
    self.debug
  }

  // pub fn ptrace(self: &mut Self, syscall_filter: Option<SyscallFilter>) -> &mut Self {
  //   self.ptrace = syscall_filter;
  //   self
  // }
  // pub fn force(self: &mut Self) -> &mut Self {
  //   self.force = true;
  //   self
  // }
  // #[allow(unused)]
  // pub fn debug(self: &mut Self) -> &mut Self {
  //   self.debug = true;
  //   self
  // }

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

          // match remove_dir_all(&new_root) {
          //   Ok(_) => {
          //     info!("Remove new root: {}", new_root.to_string_lossy());
          //   }
          //   Err(err) => {
          //     error!(
          //       "Fails removing new root: {} ({})",
          //       new_root.to_string_lossy(),
          //       err
          //     );
          //   }
          // }
        }
      }
    }
  }
}

impl CatBoxResult {
  pub(crate) fn new(status: Option<i32>, signal: Option<Signal>, usage: CatBoxUsage) -> Self {
    CatBoxResult {
      status,
      signal,
      time: usage.time(),
      time_user: usage.time_user(),
      time_sys: usage.time_sys(),
      memory: usage.memory(),
    }
  }

  pub fn status(&self) -> &Option<i32> {
    &self.status
  }

  pub fn signal(&self) -> &Option<Signal> {
    &self.signal
  }

  pub fn time(&self) -> TimeLimitType {
    self.time
  }

  pub fn time_user(&self) -> TimeLimitType {
    self.time_user
  }

  pub fn time_sys(&self) -> TimeLimitType {
    self.time_sys
  }

  pub fn memory(&self) -> MemoryLimitType {
    self.memory
  }
}
