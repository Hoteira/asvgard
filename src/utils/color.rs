use crate::parser::tags::Tag;

#[inline]
pub fn get_fill(tag: &mut Tag) -> u32 {
    let c = tag.params.get("fill");

    if c.is_none() {
        0x0
    } else {
        match c.unwrap().to_string().as_str() {
            "white" => 0xFF_FFFFFF,
            "black" => 0xFF_000000,
            "red" => 0xFF_FF0000,
            "green" => 0xFF_00FF00,
            "blue" => 0xFF_0000FF,

            _ => 0x0,
        }

    }
}

#[inline]
pub fn get_stroke(tag: &mut Tag) -> u32 {
    let c = tag.params.get("stroke");

    if c.is_none() {
        0x0
    } else {
        match c.unwrap().to_string().as_str() {
            "none" => 0x0,
            "white" => 0xFF_FFFFFF,
            "black" => 0xFF_000000,
            "red" => 0xFF_FF0000,
            "green" => 0xFF_00FF00,
            "blue" => 0xFF_0000FF,

            _ => 0x0,
        }

    }
}