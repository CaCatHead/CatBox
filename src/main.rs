#![allow(dead_code)]

use std::env;
use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use flexi_logger::{FileSpec, Logger};
use log::{error, info};

use crate::catbox::run;
use crate::context::{CatBox, CatBoxBuilder, CatBoxOption};
use crate::error::{CatBoxError, CatBoxExit};
// use crate::preset::make_compile_params;
use crate::utils::{default_format, GidType, MemoryLimitType, TimeLimitType, UidType};

mod catbox;
mod cgroup;
mod context;
mod error;
mod preset;
mod syscall;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[arg(short, long, help = "Output report")]
  report: bool,

  #[arg(long, requires = "report", help = "Output JSON format report")]
  json: bool,

  #[arg(short, long, help = "Time limit (unit: ms) [default: 1000]")]
  time: Option<TimeLimitType>,

  #[arg(short, long, help = "Memory limit (unit: KB) [default: 262144]")]
  memory: Option<MemoryLimitType>,

  #[arg(long, value_name = "KEY=VALUE", help = "Pass environment variables [default: PATH]")]
  env: Vec<String>,

  #[arg(long, help = "Current working directory [default: ./]")]
  cwd: Option<PathBuf>,

  #[arg(long, help = "Child process uid [default: Nobody]")]
  uid: Option<UidType>,

  #[arg(long, help = "Child process gid [default: nogroup]")]
  gid: Option<GidType>,

  #[arg(long, help = "Run in current user [default: false]")]
  user: bool,

  #[arg(short, long, help = "Force security control [default: false]")]
  force: bool,

  #[structopt(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[command(about = "Run user program")]
  Run {
    #[arg(help = "Program to be executed")]
    program: String,

    #[arg(help = "Arguments")]
    arguments: Vec<String>,

    #[arg(short = 'i', long, help = "Redirect stdin [default: PIPE]")]
    stdin: Option<String>,

    #[arg(short = 'o', long, help = "Redirect stdout [default: PIPE]")]
    stdout: Option<String>,

    #[arg(short = 'e', long, help = "Redirect stderr [default: PIPE]")]
    stderr: Option<String>,

    #[arg(short = 'R', long, value_name = "SRC:DST", help = "Mount read-only directory")]
    read: Vec<String>,

    #[arg(short = 'W', long, value_name = "SRC:DST", help = "Mount read-write directory")]
    write: Vec<String>,

    #[arg(long, help = "The number of processes [default: 1]")]
    process: Option<u64>,

    #[arg(
      long,
      value_name = "PRESET",
      help = "Enable ptrace presets [support: none|net|process|all]"
    )]
    ptrace: Option<Vec<String>>,

    #[arg(long, help = "Disable chroot [default: false]")]
    no_chroot: bool,
  },

  #[command(about = "Compile user code")]
  Compile {
    #[arg(help = "Submission code file")]
    submission: String,

    #[arg(short, long, help = "Language")]
    language: Option<String>,

    #[arg(short, long, help = "Output file")]
    output: String,

    #[arg(long, default_value = "/dev/null", help = "Redirect stdout")]
    stdout: String,

    #[arg(long, default_value = "/dev/null", help = "Redirect stderr")]
    stderr: String,
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
  fn resolve(self) -> Result<CatBox, CatBoxError> {
    let builder = match self.command {
      Commands::Run { .. } => CatBoxBuilder::run(),
      Commands::Compile { .. } => CatBoxBuilder::compile(),
      Commands::Validate { .. } => {
        unimplemented!()
      }
      Commands::Check { .. } => {
        unimplemented!()
      }
    }
    .set_default_time_limit(self.time)
    .set_default_memory_limit(self.memory)
    .set_default_force(self.force)
    .set_current_user(self.user)
    .set_default_uid(self.uid)
    .set_default_gid(self.gid)
    .set_default_cwd(self.cwd)
    .parse_env_list(self.env)?;

    let catbox = match self.command {
      Commands::Run {
        program,
        arguments,
        stdin,
        stdout,
        stderr,
        read,
        write,
        process,
        ptrace,
        no_chroot,
      } => builder
        .command(program, arguments)
        .set_process(process)
        .set_stdin(stdin)
        .set_stdout(stdout)
        .set_stderr(stderr)
        .set_chroot(!no_chroot)
        .parse_ptrace_presets(ptrace)?
        .parse_mount_read(read)?
        .parse_mount_write(write)?
        .done(),
      Commands::Compile {
        language,
        submission,
        output,
        ..
      } => {
        // make_compile_params(language, submission, output)?
        unimplemented!()
      }
      Commands::Validate { .. } => {
        unimplemented!()
      }
      Commands::Check { .. } => {
        unimplemented!()
      }
    };

    Ok(catbox.build())
  }
}

fn bootstrap() -> Result<(), CatBoxError> {
  Logger::try_with_str("catj=info")?
    .log_to_file(
      FileSpec::default()
        .directory(env::var("CATJ_LOG").unwrap_or("./logs/".into()))
        .basename("catj")
        .discriminant(format!(
          "{}",
          chrono::offset::Local::now().format("%Y-%m-%d")
        ))
        .suppress_timestamp(),
    )
    .append()
    // .duplicate_to_stderr(Duplicate::Warn)
    .format_for_files(default_format)
    .start()?;

  info!("Start running catj");

  let cli = Cli::parse();
  let report = cli.report;
  let json_format = cli.json;
  let mut catbox = cli.resolve()?;

  let result = match catbox.start() {
    Ok(results) => {
      info!("Running catj finished");
      if report {
        if !json_format {
          catbox.report();
        } else {
          catbox.report_json();
        }
      }
      Ok(())
    }
    Err(err) => {
      error!("Running catj failed: {}", err);
      Err(err)
    }
  };

  catbox.close();

  result
}

fn main() -> CatBoxExit {
  match bootstrap() {
    Ok(_) => CatBoxExit::Ok,
    Err(err) => CatBoxExit::Err(err),
  }
}
