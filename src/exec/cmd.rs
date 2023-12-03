use serde::Serialize;
use serde_json::Value;

use crate::{command::WithCommands, context::Context};

use super::Executable;

#[derive(Debug, Clone, knuffel::Decode)]
pub struct Cmd {
    #[knuffel(arguments)]
    name: Vec<String>,

    #[knuffel(children(name = "argument"))]
    pub arguments: Vec<CmdValue>,

    #[knuffel(children(name = "flag"))]
    pub flag: Vec<CmdValue>,
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct CmdValue {
    #[knuffel(argument)]
    name: String,

    #[knuffel(argument)]
    value: String,
}

#[async_trait::async_trait]
impl Executable for Cmd {
    async fn execute(&self, context: &Context) {
        let command = context.bakefile.command(&self.name.join(" "));
        let mut context = context.clone();

        for argument in &self.arguments {
            context
                .data
                .a
                .insert(argument.name.clone(), Value::String(argument.value.clone()));
        }

        if let Some(command) = command {
            command.execute(&context).await;
        };
    }
}
