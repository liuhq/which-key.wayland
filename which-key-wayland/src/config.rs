mod bind;
mod define;
pub(crate) mod parser;

pub(crate) use define::{Config, Footer, SYMBOL_INDICATOR};

pub(crate) trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
}
