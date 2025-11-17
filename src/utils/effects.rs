use crate::parser::tags::Tag;

#[inline]
pub fn get_stroke_width(tag: &mut Tag) -> usize {
    let w = tag.params.get("stroke");

    if let Some(w) = w {
        return w.parse::<usize>().unwrap();
    }

    0
}