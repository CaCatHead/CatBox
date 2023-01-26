use std::fs;

use catj::{run, CatBoxBuilder, CatBoxOption};

mod common;

#[test]
fn it_should_build_cat_box() {
  CatBoxBuilder::run()
    .command("g++", vec!["a.cpp", "-o", "a.out"])
    .build();
}

#[test]
fn it_should_echo() {
  common::setup();
  let text = "123";
  let output_path = "./echo.out";

  let catbox = CatBoxBuilder::run()
    .command("echo", vec![text])
    .stdout(output_path)
    .build();
  run(catbox.single().unwrap()).unwrap();

  let output = fs::read_to_string(output_path).unwrap();
  assert_eq!(output.trim(), text)
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
