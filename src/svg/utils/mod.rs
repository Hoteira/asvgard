use crate::svg::parser::tags::Tag;

pub(crate) mod coords;
pub(crate) mod color;
pub(crate) mod effects;
pub mod transform;

pub fn get_id(tag: &Tag) -> Option<&String> {
    tag.params.get("id")
}