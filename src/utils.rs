use std::ffi::CString;

pub fn into_c_string(string: &String) -> CString {
  let string = string.as_str();
  CString::new(string).expect("Convert &str to CString should work")
}
