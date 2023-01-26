use std::cmp::max;
use std::path::PathBuf;
use std::slice::Iter;

use nix::libc::STDOUT_FILENO;
use nix::sys::signal::Signal;
use nix::unistd::{isatty, Gid, Uid};

use crate::cgroup::CatBoxUsage;
use crate::syscall::SyscallFilter;
use crate::utils::mount::MountPoint;
use crate::utils::{MemoryLimitType, TimeLimitType};
use crate::CatBoxError;

pub use builder::{CatBoxBuilder, CatBoxOptionBuilder};

mod builder;

/// CatBox top structure for context and multiple commands
pub struct CatBox {
  context: Box<dyn CatBoxContext>,
  options: Vec<CatBoxOption>,
}

/// CatBoxContext for storing running result
pub trait CatBoxContext {
  fn add_result(&mut self, label: &String, result: CatBoxResult);

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

pub struct CatBoxRunContext {
  max_time: TimeLimitType,
  max_memory: MemoryLimitType,
  sum_time: TimeLimitType,
  sum_memory: MemoryLimitType,
  results: Vec<CatBoxResult>,
}

pub struct CatBoxCompileContext {}

pub struct CatBoxJudgeContext {}

/// CatBox running params that can config its behavior
#[derive(Debug, Clone)]
pub struct CatBoxOption {
  /// Used to identify command
  label: String,
  /// Time limit
  time_limit: TimeLimitType,
  /// Memory limit
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
  /// Run all the commands
  pub fn start(&mut self) -> Result<(), CatBoxError> {
    for option in self.options.iter() {
      let result = crate::run(&option)?;
      self.context.add_result(&option.label.clone(), result);
    }
    Ok(())
  }

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

impl CatBoxRunContext {
  pub fn new() -> Self {
    CatBoxRunContext {
      max_time: 0,
      max_memory: 0,
      sum_time: 0,
      sum_memory: 0,
      results: vec![],
    }
  }
}

impl CatBoxContext for CatBoxRunContext {
  fn add_result(&mut self, _label: &String, result: CatBoxResult) {
    self.max_time = max(self.max_time, result.time);
    self.max_memory = max(self.max_memory, result.memory);
    self.sum_time += result.time;
    self.sum_memory += result.memory;
    self.results.push(result);
  }

  fn report_human(&self) {
    if self.results.len() == 1 {
      let result = self.results.first().unwrap();
      let status = result.status().map_or_else(
        || "\x1b[91m×\x1b[39m".to_string(),
        |v| format!("\x1b[9{}m{}\x1b[39m", if v == 0 { 2 } else { 1 }, v),
      );
      let signal = result.signal().map_or_else(
        || "\x1b[92m✓\x1b[39m".to_string(),
        |v| format!("\x1b[91m{}\x1b[39m", v),
      );

      println!();
      println!("\x1b[1mStatus\x1b[22m     {}", status);
      println!("\x1b[1mSignal\x1b[22m     {}", signal);
      println!("\x1b[1mTime\x1b[22m       {} ms", result.time());
      println!("\x1b[1mTime user\x1b[22m  {} ms", result.time_user());
      println!("\x1b[1mTime sys\x1b[22m   {} ms", result.time_sys());
      println!("\x1b[1mMemory\x1b[22m     {} KB", result.memory());
      println!();
    } else {
      todo!()
    }
  }

  fn report_json(&self) {
    if self.results.len() == 1 {
      let result = self.results.first().unwrap();
      let status = result
        .status()
        .map_or_else(|| "null".to_string(), |v| v.to_string());
      let signal = result
        .signal()
        .map_or_else(|| "null".to_string(), |v| format!("\"{}\"", v));

      println!("{{");
      println!("  \"ok\": true,");
      println!("  \"status\": {},", status);
      println!("  \"signal\": {},", signal);
      println!("  \"time\": {},", result.time());
      println!("  \"time_user\": {},", result.time_user());
      println!("  \"time_sys\": {},", result.time_sys());
      println!("  \"memory\": {}", result.memory());
      println!("}}");
    } else {
      todo!()
    }
  }
}

impl CatBoxContext for CatBoxCompileContext {
  fn add_result(&mut self, _label: &String, result: CatBoxResult) {
    todo!()
  }

  fn report_human(&self) {
    todo!()
  }

  fn report_json(&self) {
    todo!()
  }
}

impl CatBoxContext for CatBoxJudgeContext {
  fn add_result(&mut self, _label: &String, result: CatBoxResult) {
    todo!()
  }

  fn report_human(&self) {
    todo!()
  }

  fn report_json(&self) {
    todo!()
  }
}
