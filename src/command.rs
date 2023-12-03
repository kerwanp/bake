use crate::{
    argument::Argument,
    context::Context,
    exec::{
        run::{self, Run},
        Executable,
    },
    flag::Flag,
    include::Include,
};

#[derive(Debug, Clone, knuffel::Decode)]
pub struct Command {
    #[knuffel(argument)]
    pub name: String,

    #[knuffel(child, default)]
    pub internal: bool,

    #[knuffel(children(name = "include"))]
    pub includes: Vec<Include>,

    #[knuffel(child, unwrap(argument))]
    pub help: Option<String>,

    #[knuffel(children(name = "flag"))]
    pub flags: Vec<Flag>,

    #[knuffel(children(name = "argument"))]
    pub arguments: Vec<Argument>,

    #[knuffel(children(name = "command"))]
    pub subcommands: Vec<Command>,

    #[knuffel(child)]
    pub run: Option<Run>,
}

pub trait WithCommands {
    fn commands(&self) -> Vec<&Command>;
    fn command(&self, name: &str) -> Option<&Command>;
}

impl WithCommands for Command {
    // TODO: Find a better way to avoid cloning
    fn commands(&self) -> Vec<&Command> {
        let cmds: Vec<&Command> = self.includes.iter().flat_map(|f| f.0.commands()).collect();
        [self.subcommands.iter().collect(), cmds].concat()
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

impl From<&Command> for clap::Command {
    fn from(val: &Command) -> Self {
        // TODO: Maybe avoid leaking?
        let name: &'static str = val.name.clone().leak();
        let mut cmd = clap::Command::new(name);

        if let Some(description) = &val.help {
            cmd = cmd.about(description)
        }

        // TODO: Maybe avoid cloning?
        cmd = cmd.subcommands(val.includes.iter().flat_map(|f| f.0.commands()));
        cmd = cmd.subcommands(&val.subcommands);
        cmd = cmd.args(&val.flags);
        cmd = cmd.args(&val.arguments);
        cmd
    }
}

#[async_trait::async_trait]
impl Executable for Command {
    async fn execute(&self, context: &Context) {
        if let Some(run) = &self.run {
            run.execute(context).await;
        };
    }
}
