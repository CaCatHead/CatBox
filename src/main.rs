use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::sandbox::{CatBoxParams, run};

mod sandbox;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[arg(short, long, default_value_t = 1000)]
  time: u64,

  #[arg(short, long, default_value_t = 65536)]
  memory: u64,

  #[arg(long, default_value_t = false)]
  verbose: bool,

  #[arg(long, default_value = "/dev/null")]
  stdin: PathBuf,

  #[arg(long, default_value = "/dev/null")]
  stdout: PathBuf,

  #[arg(long, default_value = "/dev/null")]
  stderr: PathBuf,

  #[structopt(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[command(about = "Run user program")]
  Run {
    #[arg(help = "Program")]
    program: PathBuf,

    #[arg(help = "Arguments")]
    arguments: Vec<String>,
  },

  #[command(about = "Compile user code")]
  Compile {
    #[arg(help = "Compiler")]
    compiler: PathBuf,

    #[arg(help = "Arguments")]
    arguments: Vec<String>,
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
    let command = match self.command {
      Commands::Compile { compiler, arguments } => { unimplemented!() }
      Commands::Run { program, arguments } => { (program, arguments) }
      Commands::Validate { validator } => { unimplemented!() }
      Commands::Check { checker } => { unimplemented!() }
    };

    vec![
      CatBoxParams {
        time_limit: self.time,
        memory_limit: self.memory,
        program: command.0,
        arguments: command.1,
      }
    ]
  }
}

fn main() {
  let cli = Cli::parse();
  let params = cli.resolve();
  for param in params {
    run(param);
  }
}
