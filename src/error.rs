use std::{error::Error, fmt::Display};

use nix::errno::Errno;

#[derive(Debug)]
pub enum CatBoxError {
  Fork(String),
  Cgroup(String),
  Exec(String),
  Nix(Errno),
  Fs(String),
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
}

impl Display for CatBoxError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      CatBoxError::Fork(msg) => f.write_fmt(format_args!("CatBox Fork Error: {}", msg)),
      CatBoxError::Cgroup(msg) => f.write_fmt(format_args!("CatBox Cgroup Error: {}", msg)),
      CatBoxError::Exec(msg) => f.write_fmt(format_args!("CatBox Exec Error: {}", msg)),
      CatBoxError::Nix(errno) => f.write_fmt(format_args!("CatBox Nix Error: {}", errno)),
      CatBoxError::Fs(msg) => f.write_fmt(format_args!("CatBox File System Error: {}", msg)),
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

impl Error for CatBoxError {}
