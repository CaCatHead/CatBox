use crate::CatBoxError;
use log::error;
use std::fs::canonicalize;
use std::path::PathBuf;

/// Mount point
#[derive(Debug, Clone)]
pub struct MountPoint {
  write: bool,
  src: PathBuf,
  dst: PathBuf,
}

impl MountPoint {
  pub fn defaults() -> Vec<Self> {
    vec![
      Self::read(PathBuf::from("/bin"), PathBuf::from("/bin")),
      Self::read(PathBuf::from("/sbin"), PathBuf::from("/sbin")),
      Self::read(PathBuf::from("/usr"), PathBuf::from("/usr")),
      Self::read(PathBuf::from("/etc"), PathBuf::from("/etc")),
      Self::read(PathBuf::from("/lib"), PathBuf::from("/lib")),
      Self::read(PathBuf::from("/lib64"), PathBuf::from("/lib64")),
    ]
  }

  fn canonicalize<PS: Into<PathBuf>>(path: PS) -> Result<PathBuf, String> {
    let path: PathBuf = path.into();
    if path.is_absolute() {
      Ok(path)
    } else {
      canonicalize(path).or_else(|e| Err(e.to_string()))
    }
  }

  fn parse(write: bool, text: String) -> Result<Self, CatBoxError> {
    let arr = text.split(":").collect::<Vec<&str>>();
    if arr.len() == 1 {
      let p = arr.get(0).unwrap();
      Ok(MountPoint {
        write,
        src: Self::canonicalize(p)?,
        dst: Self::canonicalize(p)?,
      })
    } else if arr.len() == 2 {
      let src = arr.get(0).unwrap();
      let dst = arr.get(1).unwrap();
      Ok(MountPoint {
        write,
        src: Self::canonicalize(*src)?,
        dst: Self::canonicalize(*dst)?,
      })
    } else {
      error!("Parse mount input string ({}) fails", &text);
      Err(CatBoxError::cli("Wrong mount string format"))
    }
  }

  pub fn parse_read(text: String) -> Result<Self, CatBoxError> {
    Self::parse(false, text)
  }

  pub fn parse_write(text: String) -> Result<Self, CatBoxError> {
    Self::parse(true, text)
  }

  pub fn read(src: PathBuf, dst: PathBuf) -> Self {
    MountPoint {
      write: false,
      src: Self::canonicalize(src).unwrap(),
      dst: Self::canonicalize(dst).unwrap(),
    }
  }

  pub fn write(src: PathBuf, dst: PathBuf) -> Self {
    MountPoint {
      write: true,
      src: Self::canonicalize(src).unwrap(),
      dst: Self::canonicalize(dst).unwrap(),
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
