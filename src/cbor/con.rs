use minicbor::data::Tag;

// https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml (private tags)

pub(crate) const RAW_BYTES: Tag = Tag::new(80000);
pub(crate) const EVAL: Tag = Tag::new(80001);
pub(crate) const EVAL_FORMAT: Tag = Tag::new(80002);
pub(crate) const JSON: Tag = Tag::new(80003);
