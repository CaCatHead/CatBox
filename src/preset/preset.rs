use std::path::PathBuf;
use crate::syscall::RestrictedSyscall;
use crate::utils::mount::MountPoint;
use crate::utils::{MemoryLimitType, TimeLimitType};

#[derive(Debug, Clone)]
pub(crate) struct LanguagePreset {
  pub(crate) compile: CompileOption,
  pub(crate) execute: ExecuteOption,
}

#[derive(Debug, Clone)]
pub(crate) struct CompileOption {
  pub(crate) extension: String,
  pub(crate) commands: Vec<ExecuteCommand>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExecuteOption {
  pub(crate) commands: Vec<ExecuteCommand>,
}

#[derive(Debug, Clone)]
pub(crate) enum UserType {
  Nobody,
  Current,
  Root,
}

#[derive(Debug, Clone)]
pub struct ExecuteCommand {
  pub(crate) program: String,
  pub(crate) arguments: Vec<String>,
  pub(crate) time_limit: TimeLimitType,
  pub(crate) memory_limit: MemoryLimitType,
  pub(crate) user: UserType,
  pub(crate) process: u64,
  pub(crate) ptrace: Vec<RestrictedSyscall>,
  pub(crate) chroot: bool,
  pub(crate) mounts: Vec<MountPoint>,
  pub(crate) env: Vec<(String, String)>,
}

impl CompileOption {
  pub fn new<ES: Into<String>>(extension: ES) -> Self {
    CompileOption {
      extension: extension.into(),
      commands: vec![],
    }
  }

  pub fn command(mut self, command: ExecuteCommand) -> Self {
    self.commands.push(command);
    self
  }
}

impl ExecuteOption {
  pub fn new() -> Self {
    ExecuteOption { commands: vec![] }
  }

  pub fn command(mut self, command: ExecuteCommand) -> Self {
    self.commands.push(command);
    self
  }
}

impl ExecuteCommand {
  pub(crate) fn new<PS: Into<String>, AS: Into<String>>(program: PS, arguments: Vec<AS>) -> Self {
    ExecuteCommand {
      program: program.into(),
      arguments: arguments.into_iter().map(|a| a.into()).collect(),
      time_limit: 1000,
      memory_limit: 262144,
      user: UserType::Nobody,
      process: 1,
      ptrace: vec![RestrictedSyscall::Net, RestrictedSyscall::Process],
      chroot: true,
      mounts: vec![],
      env: vec![],
    }
  }

  fn apply(text: &str, source: &str, executable: &str) -> String {
    text
      .replace("${source}", source)
      .replace("${executable}", executable)
  }

  pub(crate) fn apply_program(&self, source: &str, executable: &str) -> String {
    Self::apply(self.program.as_str(), source, executable)
  }

  pub(crate) fn apply_arguments(&self, source: &str, executable: &str) -> Vec<String> {
    self
      .arguments
      .iter()
      .map(|a| Self::apply(a, source, executable))
      .collect()
  }

  pub(crate) fn default_time_limit(mut self, value: TimeLimitType) -> Self {
    self.time_limit = value;
    self
  }

  pub(crate) fn default_memory_limit(mut self, value: MemoryLimitType) -> Self {
    self.memory_limit = value;
    self
  }

  pub(crate) fn default_user(mut self, user_type: UserType) -> Self {
    self.user = user_type;
    self
  }

  pub(crate) fn default_process(mut self, value: u64) -> Self {
    self.process = value;
    self
  }

  pub(crate) fn default_ptrace(mut self, features: Vec<RestrictedSyscall>) -> Self {
    self.ptrace = features;
    self
  }

  pub(crate) fn default_chroot(mut self, flag: bool) -> Self {
    self.chroot = flag;
    self
  }

  pub(crate) fn append_read_mount(mut self, src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Self {
    let point = MountPoint::read(src.into(), dst.into());
    self.mounts.push(point);
    self
  }

  pub(crate) fn append_write_mount(mut self) -> Self {
    self
  }

  pub(crate) fn append_env(mut self, key: String, value: String) -> Self {
    self.env.push((key, value));
    self
  }
}
