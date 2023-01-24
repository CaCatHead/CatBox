use std::{
  error::Error,
  fmt::{Debug, Display},
};

use flexi_logger::FlexiLoggerError;
use nix::errno::Errno;

pub enum CatBoxError {
  Fork(String),
  Cgroup(String),
  Exec(String),
  Nix(Errno),
  Fs(String),
  Cli(String),
  Logger(FlexiLoggerError),
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
