use knuffel::{traits::ErrorSpan, Decode};

use crate::bakefile::Bakefile;

#[derive(Debug, Clone)]
pub struct Include(pub Bakefile);

impl<S: ErrorSpan> Decode<S> for Include {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, knuffel::errors::DecodeError<S>> {
        let raw: RawInclude = knuffel::Decode::decode_node(node, ctx)?;

        Ok(Bakefile::from_path(&raw.path).map(Include).unwrap())
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
struct RawInclude {
    #[knuffel(argument)]
    name: Option<String>,

    #[knuffel(child, unwrap(argument))]
    path: String,
}
