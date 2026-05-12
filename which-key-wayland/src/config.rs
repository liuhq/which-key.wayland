mod bind;
mod define;
pub(crate) mod parser;

pub(crate) use define::Config;

pub(crate) trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
}
