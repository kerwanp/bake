use std::collections::HashMap;

use clap::ArgMatches;
use serde::Serialize;
use serde_json::Value;

use crate::bakefile::Bakefile;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub matches: &'a ArgMatches,
    pub bakefile: &'a Bakefile,
    pub data: ContextData,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextData {
    pub f: HashMap<String, Value>,
    pub a: HashMap<String, Value>,
    pub d: HashMap<String, Value>,
}

trait WithContext {
    fn context(&self, matches: &ArgMatches, context: &mut Context) -> Context;
}
