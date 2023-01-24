use log::info;
use std::env::current_dir;
use std::fs::{self, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use catj::{run, CatBoxParams};

mod common;

fn compile_cpp(dir: &PathBuf, file: &String) -> String {
  let source_dir = PathBuf::from("./fixtures/aplusb/source/");
  let source = format!("{}{}", source_dir.to_string_lossy(), file);
  let source = Path::new(&source).to_path_buf();
  let source = source.to_string_lossy();
  let executable = dir.join("Main.out");
  let executable = executable.to_string_lossy();

  let program = String::from("g++");
  let arguments = vec![
    source.to_string(),
    String::from("-o"),
    executable.to_string(),
    String::from("-lm"),
  ];

  let mut params = CatBoxParams::new(program.clone(), arguments);
  params
    .time_limit(10 * 1000)
    .stdin(Some("/dev/null"))
    .stdout(Some("/dev/null"))
    .stderr(Some("/dev/null"))
    .chroot(true)
    .current_user()
    .ptrace(None)
    .process(10)
    .cwd(current_dir().unwrap())
    .mount_read(&source_dir, &source_dir)
    .mount_write(&dir, &dir);
  run(&params).unwrap();

  executable.to_string()
}

fn run_aplusb(dir: &PathBuf, executable: &String) {
  for i in 1..4 {
    let sub_in = PathBuf::from(format!("./fixtures/aplusb/testcases/{}.in", i));
    let sub_in = sub_in.to_string_lossy().to_string();
    let sub_out = dir.join("sub.out");
    let sub_out = sub_out.to_string_lossy().to_string();

    let mut params = CatBoxParams::new(executable.clone(), vec![]);
    params
      // .debug()
      .stdin(Some(sub_in.clone()))
      .stdout(Some(sub_out.clone()))
      .chroot(true)
      .cwd("/")
      .mount_read(&dir, &dir);
    run(&params).unwrap();

    let out = fs::read_to_string(sub_out.clone()).unwrap();
    let ans = fs::read_to_string(PathBuf::from(format!(
      "./fixtures/aplusb/testcases/{}.ans",
      i
    )))
    .unwrap();

    info!("Testcase #{}. out: {}", i, out.trim_end());

    info!("Testcase #{}. ans: {}", i, ans.trim_end());
    assert_eq!(out, ans);

    remove_file(Path::new(sub_out.as_str())).unwrap();
    params.close();
  }
}

#[test]
fn it_should_run_cpp_ac() {
  common::setup();

  let file = "ac.cpp".to_string();
  let dir = tempdir().unwrap();
  let dir = dir.into_path();

  info!("Start running {} at {}", file, dir.to_string_lossy());
  let executable = compile_cpp(&dir, &file);
  info!("Compile {} -> {} ok", &file, &executable);
  run_aplusb(&dir, &executable);
  info!("Running {} ok at {}", &file, dir.to_string_lossy());
  remove_dir_all(dir).unwrap();
}

// #[test]
// fn it_should_run_tle() {
//   setup_logger();
//   run_cpp("tle.cpp", false);
// }

// #[test]
// fn it_should_run_mle() {
//   setup_logger();
//   run_cpp("mle.cpp", false);
// }

// #[test]
// fn it_should_not_run_fork() {
//   setup_logger();
//   run_cpp("fork.cpp", false);
// }
