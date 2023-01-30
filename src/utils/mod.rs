use std::env;
use std::ffi::CString;

use flexi_logger::DeferredNow;
use log::{error, info, Record};
use nix::libc::{gid_t, uid_t};

pub use pipe::{CatBoxPipe, CatBoxReadPipe, CatBoxWritePipe};

use crate::CatBoxError;

pub mod mount;
pub mod pipe;

pub type TimeLimitType = u64;

pub type MemoryLimitType = u64;

pub type UidType = uid_t;

pub type GidType = gid_t;

/// A logline-formatter that produces log lines like <br>
/// ```[datetime: INFO] Task successfully read from conf.json```
#[allow(unused)]
pub fn default_format(
  w: &mut dyn std::io::Write,
  now: &mut DeferredNow,
  record: &Record,
) -> Result<(), std::io::Error> {
  write!(
    w,
    "[{}: {:5}] {}",
    now.format("%Y-%m-%d %H:%M:%S"),
    record.level(),
    record.args()
  )
}

pub(crate) fn into_c_string(string: &String) -> CString {
  let string = string.as_str();
  CString::new(string).expect("Convert &str to CString should work")
}

pub(crate) fn parse_env(text: String) -> Result<(String, String), CatBoxError> {
  let arr = text.split("=").collect::<Vec<&str>>();
  if arr.len() == 2 {
    let key = arr.get(0).unwrap();
    let value = arr.get(1).unwrap();
    Ok((key.to_string(), value.to_string()))
  } else if arr.len() == 1 {
    let key = arr.get(0).unwrap();
    let value = env::var(key).unwrap_or("".to_string());
    info!("Read environment variable {} = {}", key, value);
    Ok((key.to_string(), value.to_string()))
  } else {
    error!("Wrong environment variable string ({}) format", &text);
    Err(CatBoxError::cli("Wrong environment variable string format"))
  }
}
