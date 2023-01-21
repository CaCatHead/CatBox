use std::path::PathBuf;

#[derive(Debug)]
pub struct CatBoxParams {
  pub time_limit: u64,
  pub memory_limit: u64,
  pub program: PathBuf,
  pub arguments: Vec<String>,
  pub(crate) stdin: PathBuf,
  pub(crate) stdout: PathBuf,
  pub(crate) stderr: PathBuf,
}

impl CatBoxParams {
  pub fn new(program: PathBuf, arguments: Vec<String>) -> Self {
    CatBoxParams {
      time_limit: 1000,
      memory_limit: 262144,
      program,
      arguments,
      stdin: PathBuf::from("/dev/null"),
      stdout: PathBuf::from("/dev/null"),
      stderr: PathBuf::from("/dev/null"),
    }
  }

  pub fn stdin(self: &mut Self, path: PathBuf) -> &mut Self {
    self.stdin = path;
    self
  }

  pub fn stdout(self: &mut Self, path: PathBuf) -> &mut Self {
    self.stdout = path;
    self
  }

  pub fn stderr(self: &mut Self, path: PathBuf) -> &mut Self {
    self.stdin = path;
    self
  }
}
