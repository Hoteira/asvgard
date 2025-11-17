use crate::parser::tags::Tag;

#[inline]
pub fn get_width(tag: &mut Tag) -> usize {
    let w = tag.params.get("width");

    if w.is_none() {
        0
    } else {
        w.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_height(tag: &mut Tag) -> usize {
    let h = tag.params.get("height");

    if h.is_none() {
        0
    } else {
        h.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_x(tag: &mut Tag) -> usize {
    let x = tag.params.get("x");

    if x.is_none() {
        0
    } else {
        x.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_y(tag: &mut Tag) -> usize {
    let y = tag.params.get("y");

    if y.is_none() {
        0
    } else {
        y.unwrap().parse::<usize>().unwrap()
    }
}