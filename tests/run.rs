use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use flexi_logger::Logger;
use log::info;
use tempfile::tempdir;

use catj::{CatBoxParams, run};

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
  Logger::try_with_str("info")?.start()?;
  Ok(())
}

#[test]
fn it_should_run() {
  setup_logger().unwrap();

  let dir = tempdir().unwrap();
  let source = Path::new("./fixtures/aplusb/source/ac.cpp").to_path_buf();
  let executable = dir.path().join("Main.out");

  let mut command = Command::new("g++");
  command.arg(source.to_str().unwrap()).arg("-o").arg(executable.to_str().unwrap());
  command.output().expect("Compile should be ok");

  info!("Start running ./fixtures/aplusb/ac.cpp");

  for i in 1..4 {
    let executable = executable.to_string_lossy().to_string();

    let mut params = CatBoxParams::new(executable.clone(), vec![]);
    let sub_in = PathBuf::from(format!("./fixtures/aplusb/testcases/{}.in", i));
    let sub_in = sub_in.to_string_lossy().to_string();
    let sub_out = dir.path().join("sub.out");
    let sub_out = sub_out.to_string_lossy().to_string();
    params.stdin(sub_in.clone()).stdout(sub_out.clone());
    run(params).unwrap();

    let out = fs::read_to_string(sub_out.clone()).unwrap();
    let ans = fs::read_to_string(PathBuf::from(format!("./fixtures/aplusb/testcases/{}.ans", i))).unwrap();

    info!("Testcase #{}. out: {}", i, out.trim_end());
    info!("Testcase #{}. ans: {}", i, ans.trim_end());
    assert_eq!(out, ans);

    fs::remove_file(Path::new(sub_out.as_str())).unwrap();
  }

  info!("Running ./fixtures/aplusb/ac.cpp ok");
}
