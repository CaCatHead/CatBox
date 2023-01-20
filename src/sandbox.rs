use std::path::{PathBuf};

#[derive(Debug)]
pub struct CatBoxParams {
  pub time_limit: u64,
  pub memory_limit: u64,
  pub program: PathBuf,
  pub arguments: Vec<String>,
}

// impl RunCommand {
//   fn get(&self) -> std::process::Command {
//     let input = match self {
//       RunCommand::List(v) => v,
//     };
//     let program = input.first().unwrap();
//     let args = input.iter().skip(1).collect::<Vec<&String>>();
//     let mut command = std::process::Command::new(program.clone());
//     command.args(args.clone());
//     command
//   }
// }

pub fn run(params: CatBoxParams) {
  dbg!(&params.program);
  dbg!(&params.arguments);
}
