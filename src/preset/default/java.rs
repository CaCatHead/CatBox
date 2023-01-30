use lazy_static::lazy_static;

use crate::preset::preset::{
  CompileOption, ExecuteCommand, ExecuteOption, LanguagePreset, UserType,
};

lazy_static! {
  pub(crate) static ref JAVA_PRESET: LanguagePreset = LanguagePreset {
    compile: CompileOption::new("java")
      .command(
        ExecuteCommand::new("javac", vec!["-encoding", "utf8", "-d", ".", "${source}",])
          .default_time_limit(10 * 1000)
          .default_memory_limit(1024 * 1024)
          .default_user(UserType::Current)
          .default_process(10)
          .default_ptrace(vec![])
          .default_chroot(true)
          .append_read_mount("/proc", "/proc")
          .append_read_mount("/dev", "/dev")
      )
      .command(
        // Use bash to expand *.class
        ExecuteCommand::new("bash", vec!["-c", "jar -cvf ${executable} *.class"])
          .default_time_limit(10 * 1000)
          .default_memory_limit(1024 * 1024)
          .default_user(UserType::Current)
          .default_process(10)
          .default_ptrace(vec![])
          .default_chroot(true)
          .append_read_mount("/proc", "/proc")
          .append_read_mount("/dev", "/dev")
      ),
    execute: ExecuteOption::new().command(ExecuteCommand::new(
      "java",
      vec!["-cp", "${executable}", "Main"]
    )),
  };
}
