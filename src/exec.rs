pub mod cmd;
pub mod run;

use handlebars::Handlebars;
use serde::Serialize;

use crate::{
    context::Context,
    hb::{self},
};

use self::{cmd::Cmd, run::Run};

#[derive(Debug, Clone, knuffel::Decode)]
pub enum Exec {
    Cmd(Cmd),
    Run(Run),
}

#[async_trait::async_trait]
impl Executable for Exec {
    async fn execute(&self, context: &Context) {
        match self {
            Self::Cmd(cmd) => cmd.execute(context).await,
            Self::Run(task_name) => todo!("Must find task to execute {:?}", task_name),
        }
    }
}

#[async_trait::async_trait]
pub trait Executable {
    async fn execute(&self, context: &Context);
}

#[async_trait::async_trait]
impl Executable for String {
    async fn execute(&self, context: &Context) {
        let mut reg = Handlebars::new();

        reg.register_helper("flag", Box::new(hb::flag));
        let res = reg.render_template(self, &context.data).unwrap();

        let mut child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(res)
            .spawn()
            .expect("Failed to spawn");

        let _ = child.wait().await;
    }
}
