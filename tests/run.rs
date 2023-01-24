use std::fs::{self, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

use flexi_logger::Logger;
use log::info;
use tempfile::tempdir;

use catj::{run, CatBoxParams};

static INIT: Once = Once::new();

fn setup_logger() {
  INIT.call_once(|| {
    Logger::try_with_str("catj=debug,info")
      .unwrap()
      .start()
      .unwrap();
  });
}

fn run_cpp(file: &str, ok: bool) {
  let file = format!("./fixtures/aplusb/source/{}", file);
  let dir = tempdir().unwrap();
  let dir = dir.into_path();
  let source = Path::new(&file).to_path_buf();
  let executable = dir.join("Main.out");

  let mut command = Command::new("g++");
  command
    .arg(source.to_str().unwrap())
    .arg("-o")
    .arg(executable.to_str().unwrap());
  command.output().expect("Compile should be ok");

  info!("Start running {}", file);

  for i in 1..4 {
    let executable = executable.to_string_lossy().to_string();

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

    if ok {
      info!("Testcase #{}. ans: {}", i, ans.trim_end());
      assert_eq!(out, ans);
    } else {
      break;
    }

    remove_file(Path::new(sub_out.as_str())).unwrap();
    params.close();
  }

  info!("Running {} ok", file);
  remove_dir_all(dir).unwrap();
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

#[test]
fn it_should_not_run_fork() {
  setup_logger();
  run_cpp("fork.cpp", false);
}

#[test]
fn it_should_echo() {
  setup_logger();
  let mut params = CatBoxParams::new("echo", vec!["123".to_string()]);
  params
    // .debug()
    .stdout(Some("./logs/echo.txt"));
  run(&params).unwrap();
}

// #[test]
// fn it_should_dup() {
//   match unsafe { fork() } {
//     Ok(ForkResult::Parent { child, .. }) => {
//       waitpid(child, None).unwrap();
//     }
//     Ok(ForkResult::Child { .. }) => {
//       let null_fd = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .open("/dev/null")
//         .unwrap()
//         .into_raw_fd();

//       let file = Path::new("a.txt");
//       let file = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .truncate(true)
//         .mode(S_IWUSR | S_IRUSR | S_IRGRP | S_IWGRP)
//         .open(file)
//         .unwrap();
//       let fd = File::into_raw_fd(file);

//       println!("fd: {}", fd);
//       println!("null: {}", null_fd);

//       dup2(null_fd, STDIN_FILENO).unwrap();
//       dup2(fd, STDOUT_FILENO).unwrap();
//       dup2(null_fd, STDERR_FILENO).unwrap();

//       close(fd).unwrap();
//       close(null_fd).unwrap();

//       write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();
//       execvp(
//         into_c_string("echo").as_c_str(),
//         &[into_c_string("echo"), into_c_string("1234444")],
//       )
//       .unwrap();
//     }
//     Err(_) => {}
//   };
// }

// fn into_c_string<S: Into<String>>(string: S) -> CString {
//   let string = string.into();
//   let string = string.as_str();
//   CString::new(string).expect("Convert &str to CString should work")
// }
