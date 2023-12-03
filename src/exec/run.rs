use futures::future::join_all;
use serde::Serialize;

use crate::context::Context;

use super::{Exec, Executable};

#[derive(Debug, Clone, knuffel::Decode)]
pub struct Run {
    #[knuffel(argument)]
    name: Option<String>,

    #[knuffel(child, default)]
    parallel: bool,

    #[knuffel(children)]
    exec: Vec<Exec>,
}

#[async_trait::async_trait]
impl Executable for Run {
    async fn execute(&self, context: &Context) {
        if !self.exec.is_empty() {
            let futures: Vec<_> = self.exec.iter().map(|t| t.execute(context)).collect();

            if self.parallel {
                join_all(futures).await;
            } else {
                for future in futures {
                    future.await
                }
            }
        } else if let Some(run) = &self.name {
            run.execute(context).await
        } else {
            dbg!(self);
            panic!("You must specify a command to run");
        }
    }
}
