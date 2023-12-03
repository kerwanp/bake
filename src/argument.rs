#[derive(Debug, Clone, knuffel::Decode)]
pub struct Argument {
    #[knuffel(argument)]
    pub name: String,

    #[knuffel(child, unwrap(argument))]
    pub help: Option<String>,
}

impl From<&Argument> for clap::Arg {
    fn from(val: &Argument) -> Self {
        let name: &'static str = val.name.clone().leak();
        let mut arg = clap::Arg::new(name);

        if let Some(help) = &val.help {
            arg = arg.help(help);
        }

        arg
    }
}
