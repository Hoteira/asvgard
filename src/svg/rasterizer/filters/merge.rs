use crate::svg::parser::tags::Tag;
use crate::utils::compat::{HashMap, String, ToString, Vec, vec};
use crate::utils::compat::FloatExt;

pub fn apply(
    _input: &[u32],
    width: usize,
    height: usize,
    tag: &Tag,
    results: &HashMap<String, Vec<u32>>,
    source_graphic: &[u32]
) -> Vec<u32> {
    let mut output = vec![0u32; width * height];

    // Iterate over <feMergeNode> children
    for child in &tag.children {
        if child.name == "feMergeNode" {
            let in_name = child.params.get("in").map(|s| s.as_str()).unwrap_or("");
            
            let layer = if in_name == "SourceGraphic" {
                 Some(source_graphic)
            } else if let Some(buf) = results.get(in_name) {
                Some(buf.as_slice())
            } else {
                None
            };

            if let Some(layer_data) = layer {
                blend_layer(&mut output, layer_data);
            }
        }
    }

    output
}

fn blend_layer(dest: &mut [u32], src: &[u32]) {
    crate::svg::rasterizer::simd::blend_scanline(dest, src);
}
