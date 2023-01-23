use std::collections::HashMap;
use std::error::Error;

use lazy_static::lazy_static;

use super::context::CatBoxParams;

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
) -> Result<CatBoxParams, Box<dyn Error>> {
  let language = detect_language(&language, &submission)
    .ok_or(Box::<dyn Error>::from("Can not detect submission language"))?;

  match language.as_str() {
    "c" => {
      let args = vec![];
      let params = CatBoxParams::new("g++", args);
      Ok(params)
    }
    "cpp" => {
      let args = vec![];
      let params = CatBoxParams::new("g++", args);
      Ok(params)
    }
    _ => {
      unimplemented!()
    }
  }
}

struct CompileOption {
  time_limit: u64,
  memory_limit: u64,
  process: u64,
  commands: Vec<CompileCommand>,
}

struct CompileCommand {
  program: String,
  argument: Vec<String>,
}

impl CompileOption {
  fn new() -> Self {
    CompileOption {
      time_limit: 10000,
      memory_limit: 1048576,
      process: 10,
      commands: vec![],
    }
  }

  fn command<PS: Into<String>, AS: Into<String>>(
    self: &mut Self,
    program: PS,
    arguments: Vec<AS>,
  ) -> &mut Self {
    let command = CompileCommand::new(program, arguments);
    self.commands.push(command);
    self
  }

  fn resolve(self: Self) -> Vec<CatBoxParams> {
    let mut commands = vec![];
    for command in self.commands {
      let mut params = CatBoxParams::new(command.program, command.argument);
      params
        .time_limit(self.time_limit)
        .memory_limit(self.memory_limit)
        .process(self.process);
      commands.push(params);
    }
    commands
  }
}

impl CompileCommand {
  fn new<PS: Into<String>, AS: Into<String>>(program: PS, arguments: Vec<AS>) -> Self {
    CompileCommand {
      program: program.into(),
      argument: arguments.into_iter().map(|a| a.into()).collect(),
    }
  }
}
