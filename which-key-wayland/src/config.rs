mod bind;
mod define;
pub mod parser;

pub use define::{Config, Footer, SYMBOL_INDICATOR};

pub trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
}
