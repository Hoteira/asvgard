pub mod color;
pub mod coords;
pub mod effects;
pub mod transform;

use crate::svg::parser::tags::Tag;
use crate::utils::compat::String;

pub fn get_id(tag: &Tag) -> Option<&String> {
    tag.params.get("id")
}