use flexi_logger::DeferredNow;
use log::Record;
use std::ffi::CString;

/// A logline-formatter that produces log lines like <br>
/// ```[datetime: INFO] Task successfully read from conf.json```
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

pub fn into_c_string(string: &String) -> CString {
  let string = string.as_str();
  CString::new(string).expect("Convert &str to CString should work")
}
