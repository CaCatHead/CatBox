#[derive(Debug)]
pub struct CatBoxParams {
  pub time_limit: u64,
  pub memory_limit: u64,
  pub program: String,
  pub arguments: Vec<String>,
  pub(crate) chroot: bool,
  pub(crate) env: Vec<(String, String)>,
  pub(crate) stdin: String,
  pub(crate) stdout: String,
  pub(crate) stderr: String,
}

impl CatBoxParams {
  pub fn new(program: String, arguments: Vec<String>) -> Self {
    CatBoxParams {
      time_limit: 1000,
      memory_limit: 262144,
      program,
      arguments,
      chroot: false,
      env: vec![],
      stdin: String::from("/dev/null"),
      stdout: String::from("/dev/null"),
      stderr: String::from("/dev/null"),
    }
  }

  pub fn stdin(self: &mut Self, path: String) -> &mut Self {
    self.stdin = path;
    self
  }

  pub fn stdout(self: &mut Self, path: String) -> &mut Self {
    self.stdout = path;
    self
  }

  pub fn stderr(self: &mut Self, path: String) -> &mut Self {
    self.stdin = path;
    self
  }

  pub fn chroot(self: &mut Self, flag: bool) -> &mut Self {
    self.chroot = flag;
    self
  }

  pub fn env(self: &mut Self, key: String, value: String) -> &mut Self {
    self.env.push((key, value));
    self
  }
}
