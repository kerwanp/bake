use std::str::FromStr;

use clap::ArgAction;

#[derive(Debug, Clone, knuffel::Decode)]
pub struct Flag {
    #[knuffel(type_name)]
    pub mode: FlagMode,

    #[knuffel(argument)]
    pub name: String,

    #[knuffel(argument)]
    pub short: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub help: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub enum FlagMode {
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

impl From<&Flag> for clap::Arg {
    fn from(val: &Flag) -> Self {
        let name: &'static str = val.name.clone().leak();
        let mut arg = clap::Arg::new(name).long(name);

        arg = match val.mode {
            FlagMode::Boolean => arg.action(ArgAction::SetTrue),
            FlagMode::Value => arg.action(ArgAction::Set),
        };

        // TODO : Handle error properly
        if let Some(short) = &val.short {
            let chars: Vec<char> = short.chars().collect();

            if chars.len() > 1 {
                panic!("Ohoho flag issue");
            }

            let char = chars.first().unwrap();
            arg = arg.short(char.to_owned());
        }

        if let Some(description) = &val.help {
            arg = arg.help(description);
        }

        arg
    }
}
