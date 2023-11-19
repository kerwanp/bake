mod exec;
mod hb;

use std::{collections::HashMap, fs, str::FromStr};

use clap::{ArgAction, ArgMatches};
use exec::Executable;
use serde::{de::value, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, knuffel::Decode)]
struct Bakefile {
    #[knuffel(child, unwrap(argument))]
    name: String,

    #[knuffel(child, unwrap(argument))]
    author: Option<String>,

    #[knuffel(child, unwrap(argument))]
    version: Option<String>,

    #[knuffel(child, unwrap(argument))]
    about: Option<String>,

    #[knuffel(children(name = "flag"))]
    flags: Vec<Flag>,

    #[knuffel(children(name = "command"))]
    commands: Vec<Command>,
}

impl Into<clap::Command> for Bakefile {
    fn into(self) -> clap::Command {
        let name: &'static str = self.name.leak();
        let mut cmd = clap::Command::new(name);

        if let Some(author) = self.author {
            let author: &'static str = author.leak();
            cmd = cmd.version(author)
        }

        if let Some(version) = self.version {
            let version: &'static str = version.leak();
            cmd = cmd.version(version)
        }

        if let Some(about) = self.about {
            cmd = cmd.about(about)
        }

        cmd = cmd.subcommands(self.commands);
        cmd = cmd.args(self.flags);

        cmd
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
struct Command {
    #[knuffel(argument)]
    name: String,

    #[knuffel(child, unwrap(argument))]
    help: Option<String>,

    #[knuffel(children(name = "flag"))]
    flags: Vec<Flag>,

    #[knuffel(children(name = "argument"))]
    arguments: Vec<Argument>,

    #[knuffel(children(name = "command"))]
    subcommands: Vec<Command>,

    #[knuffel(child)]
    exec: Exec,
}

impl Into<clap::Command> for Command {
    fn into(self) -> clap::Command {
        let name: &'static str = self.name.leak();
        let mut cmd = clap::Command::new(name);

        if let Some(description) = self.help {
            cmd = cmd.about(description)
        }

        cmd = cmd.subcommands(self.subcommands);
        cmd = cmd.args(self.flags);
        cmd = cmd.args(self.arguments);

        cmd
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
struct Argument {
    #[knuffel(argument)]
    name: String,

    #[knuffel(child, unwrap(argument))]
    help: Option<String>,
}

impl Into<clap::Arg> for Argument {
    fn into(self) -> clap::Arg {
        let name: &'static str = self.name.leak();
        let mut arg = clap::Arg::new(name);

        if let Some(description) = self.help {
            arg = arg.help(description);
        }

        arg
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
struct Flag {
    #[knuffel(type_name)]
    mode: FlagMode,

    #[knuffel(argument)]
    name: String,

    #[knuffel(argument)]
    short: Option<String>,

    #[knuffel(child, unwrap(argument))]
    help: Option<String>,
}

#[derive(Debug, Clone, Default)]
enum FlagMode {
    #[default]
    Boolean,
    Value,
}

impl FromStr for FlagMode {
    type Err = Box<dyn std::error::Error + Send + Sync + 'static>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "value" => Ok(FlagMode::Value),
            "bool" | "boolean" => Ok(FlagMode::Boolean),
            _ => Err("Flag mode must be `boolean` or `value`")?,
        }
    }
}

impl Into<clap::Arg> for Flag {
    fn into(self) -> clap::Arg {
        let name: &'static str = self.name.leak();
        let mut arg = clap::Arg::new(name).long(name);

        arg = match self.mode {
            FlagMode::Boolean => arg.action(ArgAction::SetTrue),
            FlagMode::Value => arg.action(ArgAction::Set),
        };

        // TODO : Handle error properly
        if let Some(short) = self.short {
            let chars: Vec<char> = short.chars().collect();

            if chars.len() > 1 {
                panic!("Ohoho flag issue");
            }

            let char = chars.get(0).unwrap();
            arg = arg.short(char.to_owned());
        }

        if let Some(description) = self.help {
            arg = arg.help(description);
        }

        arg
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
struct Exec {
    #[knuffel(argument)]
    name: Option<String>,

    #[knuffel(child)]
    parallel: bool,

    #[knuffel(children)]
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, knuffel::Decode)]
enum Task {
    Exec(Exec),
    Run(#[knuffel(argument)] String),
}

#[derive(Debug, Clone, Serialize)]
struct Context {
    f: HashMap<String, Value>,
    a: HashMap<String, Value>,
    d: HashMap<String, Value>,
}

impl Command {
    pub fn context(&self, matches: &ArgMatches) -> Context {
        let ids = matches.ids();
        let mut f: HashMap<String, Value> = HashMap::new();
        let mut a: HashMap<String, Value> = HashMap::new();
        let mut d: HashMap<String, Value> = HashMap::new();

        for id in ids {
            let flag = self.flags.iter().find(|f| f.name == id.to_string());

            if let Some(flag) = flag {
                let value: Value = match flag.mode {
                    FlagMode::Boolean => Value::Bool(matches.get_flag(&flag.name)),
                    FlagMode::Value => {
                        Value::String(matches.get_one::<String>(&flag.name).unwrap().to_owned())
                    }
                };

                f.insert(id.to_string(), value.clone());
                d.insert(id.to_string(), value);
            }

            let arg = self.arguments.iter().find(|f| f.name == id.to_string());

            if let Some(arg) = arg {
                let value: Value =
                    Value::String(matches.get_one::<String>(&arg.name).unwrap().to_owned());

                a.insert(id.to_string(), value.clone());
                d.insert(id.to_string(), value);
            }
        }

        Context { f, a, d }
    }
}

#[tokio::main]
async fn main() {
    let content = fs::read_to_string("bake.kdl").unwrap();
    let bakefile = knuffel::parse::<Bakefile>("bake.kdl", &content);

    match bakefile {
        Ok(config) => {
            let cmd: clap::Command = config.clone().into();
            let matches = cmd.get_matches();

            if let Some((name, subcommand)) = matches.subcommand() {
                let command = &config.commands.iter().find(|c| c.name == name).unwrap();

                let data = command.context(&subcommand);

                command.exec.execute(&data).await;
            }
        }
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    }
}
