use std::{collections::HashMap, fs};

use clap::ArgMatches;
use serde_json::Value;

use crate::{
    command::{Command, WithCommands},
    context::{Context, ContextData},
    flag::{Flag, FlagMode},
    include::Include,
};

#[derive(Debug, Clone, knuffel::Decode)]
pub struct Bakefile {
    #[knuffel(child, unwrap(argument))]
    pub name: String,

    #[knuffel(children(name = "include"))]
    pub includes: Vec<Include>,

    #[knuffel(child, unwrap(argument))]
    pub author: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub version: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub help: Option<String>,

    #[knuffel(children(name = "flag"))]
    pub flags: Vec<Flag>,

    #[knuffel(children(name = "command"))]
    pub commands: Vec<Command>,
}

impl<'a> Bakefile {
    pub fn from_path(path: &str) -> Result<Bakefile, knuffel::Error> {
        let content = fs::read_to_string(path).unwrap();
        knuffel::parse::<Self>(path, &content)
    }

    pub fn configure(&self, mut command: clap::Command) -> clap::Command {
        // TODO: Find a cleaner way (without static leak and clone?)
        if let Some(author) = self.author.clone() {
            let author: &'static str = author.leak();
            command = command.version(author)
        }

        // TODO: Find a cleaner way (without static leak and clone?)
        if let Some(version) = self.version.clone() {
            let version: &'static str = version.leak();
            command = command.version(version);
        }

        if let Some(help) = &self.help {
            command = command.about(help);
        }

        command = command.subcommands(self.commands().into_iter().filter(|c| !c.internal));

        command
    }

    pub fn context(&'a self, matches: &'a ArgMatches) -> Context {
        let ids = matches.ids();
        let mut f: HashMap<String, Value> = HashMap::new();
        let mut a: HashMap<String, Value> = HashMap::new();
        let mut d: HashMap<String, Value> = HashMap::new();

        for id in ids {
            let flag = self.flags.iter().find(|f| f.name == *id);

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

            // let arg = self.arguments.iter().find(|f| f.name == *id);
            //
            // if let Some(arg) = arg {
            //     let value: Value =
            //         Value::String(matches.get_one::<String>(&arg.name).unwrap().to_owned());
            //
            //     a.insert(id.to_string(), value.clone());
            //     d.insert(id.to_string(), value);
            // }
        }

        Context {
            matches,
            bakefile: self,
            data: ContextData { f, a, d },
        }
    }
}

impl WithCommands for Bakefile {
    // TODO: Find a better way to avoid cloning
    fn commands(&self) -> Vec<&Command> {
        let test: Vec<_> = self.includes.iter().flat_map(|i| i.0.commands()).collect();
        [self.commands.iter().collect(), test].concat()
    }

    fn command(&self, name: &str) -> Option<&Command> {
        let mut v: Vec<&str> = name.split_whitespace().collect();
        let name = v.remove(0);
        let curr = self.commands().into_iter().find(|c| c.name == name)?;

        if v.is_empty() {
            return Some(curr);
        }

        curr.command(&v.join(" "))
    }
}
