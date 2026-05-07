mod bind;
mod define;
pub mod parser;

pub use define::Config;

pub trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> Self;
}
