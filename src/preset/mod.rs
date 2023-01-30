use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::context::CatBoxOption;
use crate::error::CatBoxError;

mod default;
mod preset;

lazy_static! {
  static ref DETECT_LANGUAGE_MAP: HashMap<&'static str, &'static str> = {
    let mut map = HashMap::new();
    map.insert("c", "c");
    map.insert("cc", "cpp");
    map.insert("c++", "cpp");
    map.insert("cpp", "cpp");
    map.insert("java", "java");
    map.insert("py", "python3");
    map.insert("python", "python3");
    map.insert("python3", "python3");
    map.insert("py2", "python2");
    map.insert("python2", "python2");
    map
  };
}

fn detect_language(language: &Option<String>, submission: &String) -> Option<String> {
  if let Some(language) = language {
    return if let Some(language) = DETECT_LANGUAGE_MAP.get(language.as_str()) {
      Some(language.to_string())
    } else {
      Some(language.to_string())
    };
  }

  if let Some((_, ext)) = submission.rsplit_once(".") {
    let value = DETECT_LANGUAGE_MAP.get(ext);
    value.map(|v| v.to_string())
  } else {
    None
  }
}

pub fn make_compile_params(
  language: Option<String>,
  submission: String,
  _output: String,
) -> Result<CatBoxOption, CatBoxError> {
  let language = detect_language(&language, &submission)
    .ok_or(CatBoxError::cli("Can not detect submission language"))?;

  unimplemented!()
  // match language.as_str() {
  //   "c" => {
  //     let args = vec![];
  //     let params = CatBoxOption::new("g++", args);
  //     Ok(params)
  //   }
  //   "cpp" => {
  //     let args = vec![];
  //     let params = CatBoxOption::new("g++", args);
  //     Ok(params)
  //   }
  //   _ => {
  //     unimplemented!()
  //   }
  // }
}
