#![allow(dead_code)]

use std::env;
use std::error::Error;
use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use context::CatBoxResult;
use flexi_logger::{FileSpec, Logger};
use log::{error, info};
use nix::libc::STDOUT_FILENO;
use nix::unistd::isatty;

use crate::catbox::run;
use crate::context::CatBoxParams;
use crate::preset::make_compile_params;
use crate::utils::default_format;

mod catbox;
mod cgroup;
mod context;
mod error;
mod pipe;
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

  #[arg(short, long, help = "Time limit (unit: ms)")]
  time: Option<u64>,

  #[arg(short, long, help = "Memory limit (unit: KB)")]
  memory: Option<u64>,

  #[arg(long, value_name = "KEY=VALUE", help = "Pass environment variables")]
  env: Vec<String>,

  #[arg(long, help = "Child process uid")]
  uid: Option<u32>,

  #[arg(long, help = "Child process gid")]
  gid: Option<u32>,

  #[arg(long, help = "Run in current user")]
  user: bool,

  #[arg(short, long, help = "Force security control")]
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

    #[arg(long, help = "The number of processes")]
    process: Option<u64>,

    #[arg(long, help = "Disable chroot")]
    no_chroot: bool,

    #[arg(long, help = "Disable ptrace")]
    no_ptrace: bool,
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
  fn resolve(self) -> Result<Vec<CatBoxParams>, Box<dyn Error>> {
    let mut command = match self.command {
      Commands::Compile {
        language,
        submission,
        output,
        ..
      } => make_compile_params(language, submission, output)?,
      Commands::Run {
        program,
        arguments,
        stdin,
        stdout,
        stderr,
        read,
        write,
        process,
        no_chroot,
        no_ptrace,
      } => {
        let mut params = CatBoxParams::new(program, arguments);

        for text in read {
          if let Err(msg) = params.parse_mount_read(text) {
            error!("Parse mount string fails");
            return Err(Box::<dyn Error>::from(msg));
          }
        }
        for text in write {
          if let Err(msg) = params.parse_mount_write(text) {
            error!("Parse mount string fails");
            return Err(Box::<dyn Error>::from(msg));
          }
        }

        params
          .stdin(stdin)
          .stdout(stdout)
          .stderr(stderr)
          .chroot(!no_chroot);

        if no_ptrace {
          params.ptrace(None);
        }
        if let Some(process) = process {
          params.process(process);
        }

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

    if self.force {
      command.force();
    }

    if self.user {
      command.current_user();
    }
    if let Some(uid) = self.uid {
      command.uid(uid);
    }
    if let Some(gid) = self.gid {
      command.gid(gid);
    }

    for env in self.env {
      if let Err(msg) = command.parse_env(env) {
        error!("Parse environment variable string fails");
        return Err(Box::<dyn Error>::from(msg));
      }
    }

    Ok(vec![command])
  }
}

fn start(tasks: &Vec<CatBoxParams>) -> Result<Vec<CatBoxResult>, Box<dyn Error>> {
  let mut results = vec![];
  for param in tasks {
    let result = run(&param)?;
    results.push(result);
  }
  Ok(results)
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
    // .duplicate_to_stderr(Duplicate::Warn)
    .format_for_files(default_format)
    .start()?;

  let cli = Cli::parse();
  let report = cli.report;
  let json_format = cli.json;
  let params = cli.resolve()?;

  info!("Start running catj");

  let result = start(&params);
  for param in params {
    param.close();
  }

  match result {
    Ok(results) => {
      info!("Running catj finished");
      if report {
        if results.len() == 1 {
          let is_tty = isatty(STDOUT_FILENO).unwrap_or(false);

          let result = results.first().unwrap();

          if json_format || !is_tty {
            let status = result
              .status
              .map_or_else(|| "null".to_string(), |v| v.to_string());
            let signal = result
              .signal
              .map_or_else(|| "null".to_string(), |v| format!("\"{}\"", v));

            println!("{{");
            println!("  \"status\": {},", status);
            println!("  \"signal\": {},", signal);
            println!("  \"time\": {},", result.time);
            println!("  \"time_user\": {},", result.time_user);
            println!("  \"time_sys\": {},", result.time_sys);
            println!("  \"memory\": {}", result.memory);
            println!("}}");
          } else {
            let status = result.status.map_or_else(
              || "\x1b[91m×\x1b[39m".to_string(),
              |v| format!("\x1b[9{}m{}\x1b[39m", if v == 0 { 2 } else { 1 }, v),
            );
            let signal = result.signal.map_or_else(
              || "\x1b[92m✓\x1b[39m".to_string(),
              |v| format!("\x1b[91m{}\x1b[39m", v),
            );

            println!("\x1b[1mStatus\x1b[22m     {}", status);
            println!("\x1b[1mSignal\x1b[22m     {}", signal);
            println!("\x1b[1mTime\x1b[22m       {} ms", result.time);
            println!("\x1b[1mTime user\x1b[22m  {} ms", result.time_user);
            println!("\x1b[1mTime sys\x1b[22m   {} ms", result.time_sys);
            println!("\x1b[1mMemory\x1b[22m     {} KB", result.memory);
          }
        }
      }
      Ok(())
    }
    Err(err) => {
      error!("Running catj failed");
      Err(err)
    }
  }
}
