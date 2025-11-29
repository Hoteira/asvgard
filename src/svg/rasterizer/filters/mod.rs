pub mod gaussian;
pub mod offset;
pub mod merge;

use crate::svg::parser::tags::Tag;
use std::collections::HashMap;

pub fn apply_filter(
    source_graphic: &Vec<u32>,
    width: usize,
    height: usize,
    filter_tag: &Tag,
    _defs: &HashMap<String, Tag>,
) -> Vec<u32> {
    let mut results: HashMap<String, Vec<u32>> = HashMap::new();
    
    // 1. Initialize Standard Inputs
    results.insert("SourceGraphic".to_string(), source_graphic.clone());
    
    let source_alpha: Vec<u32> = source_graphic.iter().map(|&c| {
        // Extract Alpha, set RGB to 0 (black)
        let a = (c >> 24) & 0xFF;
        (a << 24) // 0xAA000000
    }).collect();
    results.insert("SourceAlpha".to_string(), source_alpha);


    let mut last_result_name = "SourceGraphic".to_string();

    for primitive in &filter_tag.children {
        // Determine Input
        let in_attr = primitive.params.get("in").cloned();
        let input_name = in_attr.as_deref().unwrap_or(&last_result_name);
        
        // Get Input Buffer (Default to transparent black if missing - though SVG spec says otherwise, this is safe)
        let input_buffer = results.get(input_name).cloned().unwrap_or_else(|| vec![0; width * height]);

        let output_buffer = match primitive.name.as_str() {
            "feGaussianBlur" => {
                gaussian::apply(&input_buffer, width, height, primitive)
            }
            "feOffset" => {
                offset::apply(&input_buffer, width, height, primitive)
            }
            "feMerge" => {
                merge::apply(&input_buffer, width, height, primitive, &results, source_graphic)
            }
            _ => {
                // Pass-through for unsupported filters to maintain chain continuity if possible, 
                // or just ignore.
                input_buffer
            }
        };
        
        // Store Result
        if let Some(result_name) = primitive.params.get("result") {
            results.insert(result_name.clone(), output_buffer.clone());
            last_result_name = result_name.clone();
        } else {
            // If no 'result' name, this output becomes the implicit input for the next
            // We can use a temporary name or just track the buffer. 
            // SVG Spec: If 'result' is omitted, the result is only available as the implicit input to the next filter primitive.
            // We simulate this by updating the "last_result" to point to a hidden entry or just updating the default.
            // Ideally, we insert it with a reserved name.
            let temp_name = format!("__internal_{}", primitive.name); // simplistic unique-ish name
            results.insert(temp_name.clone(), output_buffer.clone());
            last_result_name = temp_name;
        }
    }
    
    results.get(&last_result_name).cloned().unwrap_or_else(|| source_graphic.clone())
}
