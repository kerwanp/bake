mod argument;
mod bakefile;
mod command;
mod context;
mod exec;
mod flag;
mod hb;
mod include;

use bakefile::Bakefile;
use command::WithCommands;
use exec::Executable;
use serde_json::json;

#[tokio::main]
async fn main() {
    let bakefile = Bakefile::from_path("bake.kdl");

    match bakefile {
        Ok(bakefile) => {
            let cmd = bakefile.configure(clap::Command::new("bake"));

            let matches = &cmd.get_matches();
            let data = bakefile.context(matches);

            let Some((name, mut matches)) = matches.subcommand() else {
                panic!("COMMAND NOT FOUND");
            };

            let mut command = bakefile.command(name).unwrap();

            while let Some((name, subcommand)) = matches.subcommand() {
                command = command.command(name).unwrap();
                matches = subcommand;
            }

            command.execute(&data).await;
        }
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    }
}
