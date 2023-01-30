use std::collections::HashMap;
use std::path::PathBuf;

use lazy_static::lazy_static;
use log::info;
use path_absolutize::*;

use crate::context::CatBoxBuilder;
use crate::error::CatBoxError;
use crate::preset::default::{CPP_PRESET, C_PRESET, JAVA_PRESET};
use crate::preset::preset::UserType;
use crate::Commands;

mod default;
mod preset;

lazy_static! {
  static ref DETECT_LANGUAGE_MAP: HashMap<&'static str, &'static str> = {
    let mut map = HashMap::new();
    map.insert("c", "c");
    map.insert("cc", "cpp");
    map.insert("c++", "cpp");
    map.insert("cpp", "cpp");
    map.insert("java", "java");
    map.insert("py", "python3");
    map.insert("python", "python3");
    map.insert("python3", "python3");
    map.insert("py2", "python2");
    map.insert("python2", "python2");
    map
  };
}

fn detect_language(language: &Option<String>, submission: &String) -> Option<String> {
  if let Some(language) = language {
    return if let Some(language) = DETECT_LANGUAGE_MAP.get(language.as_str()) {
      Some(language.to_string())
    } else {
      Some(language.to_string())
    };
  }

  if let Some((_, ext)) = submission.rsplit_once(".") {
    let value = DETECT_LANGUAGE_MAP.get(ext);
    value.map(|v| v.to_string())
  } else {
    None
  }
}

pub(crate) fn make_compile_params(
  mut builder: CatBoxBuilder,
  command: Commands,
) -> Result<CatBoxBuilder, CatBoxError> {
  if let Commands::Compile {
    language,
    submission,
    output,
    ..
  } = command
  {
    let language = detect_language(&language, &submission)
      .ok_or(CatBoxError::cli("Can not detect submission language"))?;

    let preset = match language.as_str() {
      "c" => C_PRESET.clone(),
      "cpp" => CPP_PRESET.clone(),
      "java" => JAVA_PRESET.clone(),
      _ => return Err(CatBoxError::cli("Can not find language preset")),
    };

    info!("Compile language {}", &language);

    // let root_dir = tempdir().unwrap();
    // let root_dir = root_dir.into_path();

    let submission = PathBuf::from(&submission);
    let submission = submission.absolutize().unwrap();
    let submission_dir = submission.parent().unwrap();
    let output = PathBuf::from(&output);
    let output = output.absolutize().unwrap();
    let output_dir = output.parent().unwrap();

    for command in preset.compile.commands.iter() {
      let option_builder = builder
        .command(
          command.apply_program(submission.to_str().unwrap(), output.to_str().unwrap()),
          command.apply_arguments(submission.to_str().unwrap(), output.to_str().unwrap()),
        )
        .time_limit(command.time_limit)
        .memory_limit(command.memory_limit)
        .set_process(Some(command.process))
        .set_chroot(command.chroot)
        .mount_read(submission_dir, submission_dir)
        .mount_write(output_dir, output_dir)
        .cwd(&output_dir)
        .disable_ptrace();

      let mut option_builder = match command.user {
        UserType::Nobody => option_builder,
        UserType::Current => option_builder.current_user(),
        UserType::Root => {
          unimplemented!()
        }
      };

      for feat in command.ptrace.iter() {
        option_builder = option_builder.ptrace(feat.clone())
      }
      for mount_point in command.mounts.iter() {
        option_builder = option_builder.mount(mount_point.clone())
      }
      for (key, value) in command.env.iter() {
        option_builder = option_builder.env(key, value);
      }

      builder = option_builder.done();
    }

    Ok(builder)
  } else {
    Err(CatBoxError::cli("unreachable"))
  }
}
