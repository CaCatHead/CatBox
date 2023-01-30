use crate::context::CatBoxOption;
use crate::syscall::RestrictedSyscall;
use crate::utils::mount::MountPoint;
use crate::utils::{MemoryLimitType, TimeLimitType};

pub(crate) struct LanguagePreset {
  pub(crate) compile: CompileOption,
  pub(crate) execute: ExecuteOption,
}

pub(crate) struct CompileOption {
  pub(crate) extension: String,
  pub(crate) commands: Vec<ExecuteCommand>,
}

pub(crate) struct ExecuteOption {
  pub(crate) commands: Vec<ExecuteCommand>,
}

pub(crate) enum UserType {
  Nobody,
  Current,
  Root,
}

pub struct ExecuteCommand {
  program: String,
  argument: Vec<String>,
  time_limit: TimeLimitType,
  memory_limit: MemoryLimitType,
  user: UserType,
  process: u64,
  ptrace: Vec<RestrictedSyscall>,
  chroot: bool,
  mounts: Vec<MountPoint>,
  env: Vec<(String, String)>,
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
      argument: arguments.into_iter().map(|a| a.into()).collect(),
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

  pub(crate) fn append_read_mount(mut self) -> Self {
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
