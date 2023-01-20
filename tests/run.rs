use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

use catbox::sandbox::{CatBoxParams, run};

#[test]
fn it_should_run() {
  let dir = tempdir().unwrap();
  let source = Path::new("./fixtures/aplusb/ac.cpp").to_path_buf();
  let executable = dir.path().join("Main.out");

  let mut command = Command::new("g++");
  command.arg(source.to_str().unwrap()).arg("-o").arg(executable.to_str().unwrap());
  command.output().expect("Compile should be ok");

  run(CatBoxParams {
    time_limit: 1000,
    memory_limit: 65536,
    program: executable,
    arguments: Vec::new(),
  });
}
