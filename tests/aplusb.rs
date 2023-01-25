use catj::{run, CatBoxParams, CatBoxResult};
use log::info;
use std::env::current_dir;
use std::fs::{self, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
use nix::sys::signal::Signal;
use tempfile::tempdir;

mod common;

fn compile_cpp(dir: &PathBuf, file: &String) -> String {
  let source_dir = PathBuf::from("./fixtures/aplusb/source/");
  let source = format!("{}{}", source_dir.to_string_lossy(), file);
  let source = Path::new(&source).to_path_buf();
  let source = source.to_string_lossy();
  let executable = dir.join("Main.out");
  let executable = executable.to_string_lossy();

  let program = if source.ends_with(".cpp") { String::from("g++") } else { String::from("gcc") };
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
    // .stdout(Some("/dev/null"))
    // .stderr(Some("/dev/null"))
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

fn run_aplusb(dir: &PathBuf, executable: &String, ok: bool, time: u64, memory: u64) -> Option<(String, CatBoxResult)> {
  for i in 1..4 {
    let sub_in = PathBuf::from(format!("./fixtures/aplusb/testcases/{}.in", i));
    let sub_in = sub_in.to_string_lossy().to_string();
    let sub_out = dir.join("sub.out");
    let sub_out = sub_out.to_string_lossy().to_string();

    let mut params = CatBoxParams::new(executable.clone(), vec![]);
    params
      // .debug()
      .time_limit(time)
      .memory_limit(memory)
      .stdin(Some(sub_in.clone()))
      .stdout(Some(sub_out.clone()))
      .stderr(Some("/dev/null"))
      .chroot(true)
      .env("ONLINE_JUDGE", "true")
      .cwd("/")
      .mount_read(&dir, &dir);
    let result = run(&params).unwrap();

    let out = fs::read_to_string(sub_out.clone()).unwrap();
    let ans = fs::read_to_string(PathBuf::from(format!(
      "./fixtures/aplusb/testcases/{}.ans",
      i
    )))
    .unwrap();

    remove_file(Path::new(sub_out.as_str())).unwrap();
    params.close();

    if ok {
      info!("Testcase #{}. out: {}", i, out.trim_end());
      info!("Testcase #{}. ans: {}", i, ans.trim_end());
      assert_eq!(out, ans);
    } else {
      return Some((out, result));
    }
  }
  None
}

fn run_fail_cpp(file: &str, time: u64, memory: u64) -> CatBoxResult {
  let file = file.to_string();
  let dir = tempdir().unwrap();
  let dir = dir.into_path();

  info!("Start running {} at {}", file, dir.to_string_lossy());
  let executable = compile_cpp(&dir, &file);
  info!("Compile {} -> {} ok", &file, &executable);
  let (_, result) = run_aplusb(&dir, &executable, false, time, memory).unwrap();
  info!("Running {} ok at {}", &file, dir.to_string_lossy());
  remove_dir_all(dir).unwrap();

  result
}

fn run_fail_cpp_stdout(file: &str, time: u64, memory: u64) -> (String, CatBoxResult) {
  let file = file.to_string();
  let dir = tempdir().unwrap();
  let dir = dir.into_path();

  info!("Start running {} at {}", file, dir.to_string_lossy());
  let executable = compile_cpp(&dir, &file);
  info!("Compile {} -> {} ok", &file, &executable);
  let result = run_aplusb(&dir, &executable, false, time, memory).unwrap();
  info!("Running {} ok at {}", &file, dir.to_string_lossy());
  remove_dir_all(dir).unwrap();

  result
}

fn run_ok_cpp(file: &str, time: u64, memory: u64) {
  let file = file.to_string();
  let dir = tempdir().unwrap();
  let dir = dir.into_path();

  info!("Start running {} at {}", file, dir.to_string_lossy());
  let executable = compile_cpp(&dir, &file);
  info!("Compile {} -> {} ok", &file, &executable);
  let result = run_aplusb(&dir, &executable, true, time, memory);
  assert!(result.is_none());
  info!("Running {} ok at {}", &file, dir.to_string_lossy());
  remove_dir_all(dir).unwrap();
}

#[test]
fn it_should_run_cpp_ac() {
  common::setup();
  run_ok_cpp("ac.cpp", 1000, 262144);
}

#[test]
fn it_should_run_small_stack() {
  common::setup();
  run_ok_cpp("small_stack.cpp", 1000, 262144);
}

#[test]
fn it_should_not_run_tle() {
  common::setup();
  let result = run_fail_cpp("tle.cpp", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert!(result.time() > 1000);
}

#[test]
fn it_should_not_run_mle() {
  common::setup();
  let result = run_fail_cpp("mle.cpp", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert!(result.memory() > 262144);
}

#[test]
fn it_should_not_run_malloc() {
  common::setup();
  let result = run_fail_cpp("malloc.c", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert!(result.memory() > 262144);
}

#[test]
fn it_should_not_run_big_stack() {
  common::setup();
  let result = run_fail_cpp("big_stack.cpp", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert!(result.memory() > 262144);
}

#[test]
fn it_should_not_run_fork() {
  common::setup();
  let result = run_fail_cpp("fork.cpp", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert_eq!(*result.signal(), Some(Signal::SIGKILL));
}

#[test]
fn it_should_not_run_sleep() {
  common::setup();
  let result = run_fail_cpp("sleep.c", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert_eq!(*result.signal(), Some(Signal::SIGALRM));
}

#[test]
fn it_should_not_run_while1() {
  common::setup();
  let result = run_fail_cpp("while1.c", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert_eq!(*result.signal(), Some(Signal::SIGALRM));
}

#[test]
fn it_should_not_run_output_size() {
  common::setup();
  let result = run_fail_cpp("output_size.c", 5000, 262144);
  assert_eq!(*result.status(), None);
  assert_eq!(*result.signal(), Some(Signal::SIGXFSZ));
}

#[test]
fn it_should_run_uid() {
  common::setup();
  let (_, result) = run_fail_cpp_stdout("uid.c", 1000, 262144);
  assert_eq!(*result.status(), Some(0));
}

#[test]
fn it_should_not_run_env() {
  common::setup();
  std::env::set_var("test", "value");
  let (text, result) = run_fail_cpp_stdout("env.c", 1000, 262144);
  assert_eq!(text.trim_end(), "true,null");
  assert_eq!(*result.status(), Some(0));
}

#[test]
fn it_should_not_run_re1() {
  common::setup();
  let result = run_fail_cpp("re1.c", 1000, 262144);
  assert_eq!(*result.status(), Some(42));
}

#[test]
fn it_should_not_run_re2() {
  common::setup();
  let result = run_fail_cpp("re2.c", 1000, 262144);
  assert_eq!(*result.status(), None);
  assert_eq!(*result.signal(), Some(Signal::SIGSEGV));
}

