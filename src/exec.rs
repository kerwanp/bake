use futures::future::join_all;
use handlebars::Handlebars;
use serde::Serialize;

use crate::{
    hb::{self},
    Exec, Task,
};

#[async_trait::async_trait]
pub trait Executable {
    async fn execute<C>(&self, context: &C)
    where
        C: Serialize + Send + Sync;
}

#[async_trait::async_trait]
impl Executable for String {
    async fn execute<C>(&self, context: &C)
    where
        C: Serialize + Send + Sync,
    {
        let mut reg = Handlebars::new();

        reg.register_helper("flag", Box::new(hb::flag));
        let res = reg.render_template(&self, context).unwrap();

        let mut child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(res)
            .spawn()
            .expect("Failed to spawn");

        let _ = child.wait().await;
    }
}

#[async_trait::async_trait]
impl Executable for Exec {
    async fn execute<C>(&self, context: &C)
    where
        C: Serialize + Send + Sync,
    {
        let futures: Vec<_> = self
            .tasks
            .iter()
            .map(|t| match t {
                Task::Run(run) => run.execute(context),
                Task::Exec(exec) => exec.execute(context),
            })
            .collect();

        if self.parallel {
            join_all(futures).await;
        } else {
            for future in futures {
                future.await
            }
        }
    }
}
