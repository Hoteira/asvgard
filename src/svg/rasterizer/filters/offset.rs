use crate::svg::parser::tags::Tag;
use crate::utils::compat::{Vec, vec, FloatExt};

pub fn apply(input: &[u32], width: usize, height: usize, tag: &Tag) -> Vec<u32> {
    let dx = tag.params.get("dx").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let dy = tag.params.get("dy").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);

    let idx = dx.round() as isize;
    let idy = dy.round() as isize;

    if idx == 0 && idy == 0 {
        return input.to_vec();
    }

    let mut output = vec![0u32; width * height];

    for y in 0..height as isize {
        for x in 0..width as isize {
            let src_x = x - idx;
            let src_y = y - idy;

            if src_x >= 0 && src_x < width as isize && src_y >= 0 && src_y < height as isize {
                let src_index = (src_y as usize) * width + (src_x as usize);
                let dst_index = (y as usize) * width + (x as usize);
                output[dst_index] = input[src_index];
            }
        }
    }

    output
}
