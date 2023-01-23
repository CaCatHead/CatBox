use std::{error::Error, os::unix::prelude::RawFd};

use nix::{
  fcntl::OFlag,
  unistd::{self, close, pipe2},
};

pub struct CatBoxPipe(RawFd, RawFd);

pub struct CatBoxReadPipe(RawFd);

pub struct CatBoxWritePipe(RawFd);

impl CatBoxPipe {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let result = pipe2(OFlag::O_CLOEXEC | OFlag::O_NONBLOCK)?;
    Ok(CatBoxPipe(result.0, result.1))
  }

  pub fn read(self) -> Result<CatBoxReadPipe, Box<dyn Error>> {
    close(self.1)?;
    Ok(CatBoxReadPipe(self.0))
  }

  pub fn write(self) -> Result<CatBoxWritePipe, Box<dyn Error>> {
    close(self.0)?;
    Ok(CatBoxWritePipe(self.1))
  }
}

impl CatBoxReadPipe {
  pub fn read(self: &Self) -> Result<String, Box<dyn Error>> {
    let mut buf = Vec::new();
    buf.reserve(1024);
    unistd::read(self.0, buf.as_mut_slice())?;
    Ok(String::from_utf8(buf)?)
  }

  pub fn close(self: Self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
}

impl Drop for CatBoxReadPipe {
  fn drop(&mut self) {
    close(self.0).unwrap();
  }
}

impl CatBoxWritePipe {
  pub fn write<S: Into<String>>(self: &Self, text: S) -> Result<usize, Box<dyn Error>> {
    let text: String = text.into();
    let size = unistd::write(self.0, text.as_bytes())?;
    Ok(size)
  }

  pub fn close(self: Self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
}

impl Drop for CatBoxWritePipe {
  fn drop(&mut self) {
    close(self.0).unwrap();
  }
}
