use std::fmt::format;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

use flexi_logger::Logger;
use log::info;
use tempfile::tempdir;

use catj::{CatBoxParams, run};

static INIT: Once = Once::new();

fn setup_logger() {
  INIT.call_once(|| {
    Logger::try_with_str("catj=debug,info").unwrap().start().unwrap();
  });
}

fn run_cpp(file: &str, ok: bool) {
  let file = format!("./fixtures/aplusb/source/{}", file);
  let dir = tempdir().unwrap();
  let source = Path::new(&file).to_path_buf();
  let executable = dir.path().join("Main.out");

  let mut command = Command::new("g++");
  command.arg(source.to_str().unwrap()).arg("-o").arg(executable.to_str().unwrap());
  command.output().expect("Compile should be ok");

  info!("Start running {}", file);

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

    if ok {
      info!("Testcase #{}. ans: {}", i, ans.trim_end());
      assert_eq!(out, ans);
    } else {
      break;
    }

    fs::remove_file(Path::new(sub_out.as_str())).unwrap();
  }

  info!("Running {} ok", file);
}

#[test]
fn it_should_run_ac() {
  setup_logger();
  run_cpp("ac.cpp", true);
}

#[test]
fn it_should_run_tle() {
  setup_logger();
  run_cpp("tle.cpp", false);
}

#[test]
fn it_should_run_mle() {
  setup_logger();
  run_cpp("mle.cpp", false);
}

