use std::time::Duration;

use structopt::StructOpt;
use wait_timeout::ChildExt;

#[derive(Debug, StructOpt)]
#[structopt(about = "A simple sandbox program used for competitive programming contest")]
struct CliOption {
    #[structopt(short, long, default_value = "1")]
    time: u64,

    // #[structopt(short, long, default_value = "65536")]
    // memory: u64,

    #[structopt(subcommand)]
    command: RunCommand,
}

#[derive(Debug, StructOpt)]
enum RunCommand {
    #[structopt(external_subcommand)]
    List(Vec<String>)
}

impl RunCommand {
    fn get(&self) -> std::process::Command {
        let input = match self {
            RunCommand::List(v) => v,
        };
        let program = input.first().unwrap();
        let args = input.iter().skip(1).collect::<Vec<&String>>();
        let mut command = std::process::Command::new(program.clone());
        command.args(args.clone());
        command
    }
}

fn main() {
    let option = CliOption::from_args();
    println!("Option: {:?}", option);

    let time_limit = Duration::from_secs(option.time);
    let mut child = option.command.get().spawn().unwrap();

    println!("Id: {}", child.id());

    let status = match child.wait_timeout(time_limit).unwrap() {
        Some(status) => {
            status.code().unwrap()
        },
        None => {
            child.kill().unwrap();
            child.wait().unwrap().code().unwrap()
        }
    };

    println!("Return: {}", status);
}
