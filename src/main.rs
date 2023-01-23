use std::env;
use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use flexi_logger::{DeferredNow, Duplicate, FileSpec, Logger};
use log::{info, Record};

use crate::context::CatBoxParams;
use crate::preset::make_compile_params;
use crate::sandbox::run;

mod cgroup;
mod context;
mod preset;
mod sandbox;
mod syscall;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[arg(short, long, default_value_t = 1000)]
  time: u64,

  #[arg(short, long, default_value_t = 262144)]
  memory: u64,

  #[arg(long, default_value_t = false)]
  verbose: bool,

  #[arg(long, default_value = "/dev/null")]
  stdin: String,

  #[arg(long, default_value = "/dev/null")]
  stdout: String,

  #[arg(long, default_value = "/dev/null")]
  stderr: String,

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
  fn resolve(self) -> Vec<CatBoxParams> {
    let mut command = match self.command {
      Commands::Compile {
        language,
        submission,
        ..
      } => make_compile_params(language, submission),
      Commands::Run { program, arguments } => {
        let mut params = CatBoxParams::new(program, arguments);
        params.chroot(true);
        params
      }
      Commands::Validate { .. } => {
        unimplemented!()
      }
      Commands::Check { .. } => {
        unimplemented!()
      }
    };

    command
      .stdin(self.stdin)
      .stdout(self.stdout)
      .stderr(self.stderr);

    vec![command]
  }
}

/// A logline-formatter that produces log lines like <br>
/// ```[datetime: INFO] Task successfully read from conf.json```
pub fn default_format(
  w: &mut dyn std::io::Write,
  now: &mut DeferredNow,
  record: &Record,
) -> Result<(), std::io::Error> {
  write!(
    w,
    "[{}: {:5}] {}",
    now.format("%Y-%m-%d %H:%M:%S"),
    record.level(),
    record.args()
  )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  let params = cli.resolve();

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

  info!("Start running catbox");

  for param in params {
    run(param)?;
  }

  info!("Running catbox finished");

  Ok(())
}
