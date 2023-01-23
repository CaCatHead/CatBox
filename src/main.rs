use std::env;
use std::error::Error;
use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::{error, info};

use crate::catbox::run;
use crate::context::CatBoxParams;
use crate::preset::make_compile_params;
use crate::utils::default_format;

mod catbox;
mod cgroup;
mod context;
mod preset;
mod syscall;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[arg(short, long, help = "Time limit")]
  time: Option<u64>,

  #[arg(short, long, help = "Memory limit")]
  memory: Option<u64>,

  #[arg(long, help = "Pass environment variables (key=value)")]
  env: Vec<String>,

  #[structopt(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[command(about = "Run user program")]
  Run {
    #[arg(help = "Program")]
    program: String,

    #[arg(help = "Arguments")]
    arguments: Vec<String>,

    #[arg(short = 'i', long, default_value = "/dev/null", help = "Redirect stdin")]
    stdin: String,

    #[arg(short = 'o', long, default_value = "/dev/null", help = "Redirect stdout")]
    stdout: String,

    #[arg(short = 'e', long, default_value = "/dev/null", help = "Redirect stderr")]
    stderr: String,

    #[arg(short = 'R', long, help = "Mount read-only directory")]
    read: Vec<String>,

    #[arg(short = 'W', long, help = "Mount read-write directory")]
    write: Vec<String>,
  },

  #[command(about = "Compile user code")]
  Compile {
    #[arg(help = "Submission code file")]
    submission: String,

    #[arg(short, long, help = "Language")]
    language: String,

    #[arg(short, long, help = "Output file")]
    output: String,
  },

  #[command(about = "Run validator")]
  Validate {
    #[arg(help = "Validator")]
    validator: PathBuf,
  },

  #[command(about = "Run checker")]
  Check {
    #[arg(help = "Checker")]
    checker: PathBuf,
  },
}

impl Cli {
  fn resolve(self) -> Result<Vec<CatBoxParams>, Box<dyn Error>> {
    let mut command = match self.command {
      Commands::Compile {
        language,
        submission,
        output,
      } => make_compile_params(language, submission, output),
      Commands::Run {
        program,
        arguments,
        stdin,
        stdout,
        stderr,
        read,
        write,
      } => {
        let mut params = CatBoxParams::new(program, arguments);

        for text in read {
          if let Err(_) = params.parse_mount_read(text) {
            error!("Parse mount string fails");
          }
        }
        for text in write {
          if let Err(_) = params.parse_mount_write(text) {
            error!("Parse mount string fails");
          }
        }

        params
          .stdin(stdin)
          .stdout(stdout)
          .stderr(stderr)
          .chroot(true);

        params
      }
      Commands::Validate { .. } => {
        unimplemented!()
      }
      Commands::Check { .. } => {
        unimplemented!()
      }
    };

    if let Some(time) = self.time {
      command.time_limit(time);
    }
    if let Some(memory) = self.memory {
      command.memory_limit(memory);
    }

    for env in self.env {
      if let Err(_) = command.parse_env(env) {
        error!("Parse environment variable string fails");
      }
    }

    Ok(vec![command])
  }
}

fn start(tasks: &Vec<CatBoxParams>) -> Result<(), Box<dyn Error>> {
  for param in tasks {
    run(&param)?;
  }
  Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
  Logger::try_with_str("catj=info")?
    .log_to_file(
      FileSpec::default()
        .directory(env::var("LOG_DIR").unwrap_or("./logs/".into()))
        .basename("catj")
        .discriminant(format!(
          "{}",
          chrono::offset::Local::now().format("%Y-%m-%d")
        ))
        .suppress_timestamp(),
    )
    .append()
    .duplicate_to_stderr(Duplicate::Warn)
    .format_for_files(default_format)
    .print_message()
    .start()?;

  let cli = Cli::parse();
  let params = cli.resolve()?;

  info!("Start running catbox");

  let result = start(&params);
  for param in params {
    param.close();
  }

  match result {
    Ok(_) => {
      info!("Running catj finished");
      Ok(())
    }
    Err(err) => {
      error!("Running catj failed");
      Err(err)
    }
  }
}
