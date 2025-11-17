use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::utils::color::{get_fill, get_stroke};
use crate::utils::coords::{get_height, get_width, get_x, get_y};

pub fn draw_rect(tag: &mut Tag, canvas: &mut Canvas) {
    let width = get_width(tag);
    let height = get_height(tag);
    let x = get_x(tag);
    let y = get_y(tag);
    let color = get_fill(tag);

    let stroke = get_stroke(tag);

    println!("{:?}", color);

    for i in y..y + height {
        for j in x..x + width {
            canvas.data[i * canvas.width + j] = color;
        }
    }
}
