use lazy_static::lazy_static;

use crate::preset::preset::{
  CompileOption, ExecuteCommand, ExecuteOption, LanguagePreset, UserType,
};

lazy_static! {
  pub(crate) static ref CPP_PRESET: LanguagePreset = LanguagePreset {
    compile: CompileOption::new("cpp").command(
      ExecuteCommand::new(
        "g++",
        vec![
          "${source}",
          "-o",
          "${executable}",
          "-fdiagnostics-color=always",
          "-Wall",
          "-Wextra",
          "-Wno-unused-result",
          "-static",
          "-lm",
          "--std=c++20",
          "-O2",
          "-DONLINE_JUDGE",
          "-Wall"
        ]
      )
      .default_time_limit(10 * 1000)
      .default_memory_limit(1024 * 1024)
      .default_user(UserType::Current)
      .default_process(10)
      .default_ptrace(vec![])
      .default_chroot(true)
    ),
    execute: ExecuteOption::new()
      .command(ExecuteCommand::new::<&str, String>("${executable}", vec![])),
  };
}
