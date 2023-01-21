use std::ffi::CString;
use std::path::PathBuf;

pub fn into_c_string(path: &PathBuf) -> CString {
  let path = path.to_str().expect("Convert PathBuf to &str should work");
  CString::new(path).expect("Convert &str to CString should work")
}
