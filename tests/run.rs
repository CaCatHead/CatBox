use std::path::Path;
use std::process::Command;

use flexi_logger::Logger;
use log::info;
use tempfile::tempdir;

use catj::sandbox::{CatBoxParams, run};

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
  Logger::try_with_str("catbox=info")?.start()?;
  Ok(())
}

#[test]
fn it_should_run() {
  setup_logger().unwrap();

  let dir = tempdir().unwrap();
  let source = Path::new("./fixtures/aplusb/ac.cpp").to_path_buf();
  let executable = dir.path().join("Main.out");

  let mut command = Command::new("g++");
  command.arg(source.to_str().unwrap()).arg("-o").arg(executable.to_str().unwrap());
  command.output().expect("Compile should be ok");

  info!("Start running ./fixtures/aplusb/ac.cpp");

  run(CatBoxParams {
    time_limit: 1000,
    memory_limit: 65536,
    program: executable,
    arguments: Vec::new(),
  }).unwrap();

  info!("Running ./fixtures/aplusb/ac.cpp ok");
}
