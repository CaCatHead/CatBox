use std::{
  error::Error,
  fmt::{Debug, Display},
  process::{ExitCode, Termination},
};

use flexi_logger::FlexiLoggerError;
use nix::{errno::Errno, libc::STDOUT_FILENO, unistd::isatty};

pub enum CatBoxError {
  Fork(String),
  Cgroup(String),
  Exec(String),
  Nix(Errno),
  Fs(String),
  Cli(String),
  Logger(FlexiLoggerError),
}

#[allow(unused)]
pub enum CatBoxExit {
  Ok,
  Err(CatBoxError),
}

impl CatBoxError {
  pub fn fork<MS: Into<String>>(msg: MS) -> CatBoxError {
    CatBoxError::Fork(msg.into())
  }

  pub fn cgroup<MS: Into<String>>(msg: MS) -> CatBoxError {
    CatBoxError::Cgroup(msg.into())
  }

  pub fn exec<MS: Into<String>>(msg: MS) -> CatBoxError {
    CatBoxError::Exec(msg.into())
  }

  pub fn cli<MS: Into<String>>(msg: MS) -> CatBoxError {
    CatBoxError::Cli(msg.into())
  }
}

impl Debug for CatBoxError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self, f)
  }
}

impl Display for CatBoxError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      CatBoxError::Fork(msg) => f.write_fmt(format_args!("CatBox Fork Error: {}", msg)),
      CatBoxError::Cgroup(msg) => f.write_fmt(format_args!("CatBox Cgroup Error: {}", msg)),
      CatBoxError::Exec(msg) => f.write_fmt(format_args!("CatBox Exec Error: {}", msg)),
      CatBoxError::Nix(errno) => f.write_fmt(format_args!("CatBox Nix Error: {}", errno)),
      CatBoxError::Fs(msg) => f.write_fmt(format_args!("CatBox File System Error: {}", msg)),
      CatBoxError::Cli(msg) => f.write_fmt(format_args!("CatBox CLI Error: {}", msg)),
      CatBoxError::Logger(err) => f.write_fmt(format_args!("CatBox Logger Error: {}", err)),
    }
  }
}

impl From<Errno> for CatBoxError {
  fn from(errno: Errno) -> Self {
    CatBoxError::Nix(errno)
  }
}

impl From<std::io::Error> for CatBoxError {
  fn from(err: std::io::Error) -> Self {
    CatBoxError::Fs(err.to_string())
  }
}

impl From<FlexiLoggerError> for CatBoxError {
  fn from(err: FlexiLoggerError) -> Self {
    CatBoxError::Logger(err)
  }
}

impl Error for CatBoxError {}

impl Termination for CatBoxExit {
  fn report(self) -> ExitCode {
    match self {
      CatBoxExit::Ok => ExitCode::SUCCESS.report(),
      CatBoxExit::Err(err) => {
        let text = format!("{}", err);
        let text = match text.split_once(": ") {
          Some((prefix, message)) => {
            let is_tty = isatty(STDOUT_FILENO).unwrap_or(false);
            if is_tty {
              format!("\x1b[1m\x1b[91m{}\x1b[39m\x1b[22m  {}", prefix, message)
            } else {
              format!(
                "{{\n  \"ok\": false,\n  \"type\": \"{}\",\n  \"message\": \"{}\"\n}}",
                prefix, message
              )
            }
          }
          None => {
            format!("{}", err)
          }
        };
        eprintln!("{}", text);
        ExitCode::FAILURE.report()
      }
    }
  }
}
