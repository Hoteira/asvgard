pub mod gaussian;
pub mod offset;
pub mod merge;

use crate::svg::parser::tags::Tag;
use crate::utils::compat::{HashMap, String, ToString, Vec, format, vec};

pub fn apply_filter(
    source_graphic: &Vec<u32>,
    width: usize,
    height: usize,
    filter_tag: &Tag,
    _defs: &HashMap<String, Tag>,
) -> Vec<u32> {
    let mut results: HashMap<String, Vec<u32>> = HashMap::new();
    
    results.insert("SourceGraphic".to_string(), source_graphic.clone());
    
    let source_alpha: Vec<u32> = source_graphic.iter().map(|&c| {

        let a = (c >> 24) & 0xFF;
        (a << 24) // 0xAA000000
    }).collect();
    results.insert("SourceAlpha".to_string(), source_alpha);


    let mut last_result_name = "SourceGraphic".to_string();

    for primitive in &filter_tag.children {

        let in_attr = primitive.params.get("in").cloned();
        let input_name = in_attr.as_deref().unwrap_or(&last_result_name);
        
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
                input_buffer
            }
        };
        
        if let Some(result_name) = primitive.params.get("result") {
            results.insert(result_name.clone(), output_buffer.clone());
            last_result_name = result_name.clone();
        } else {
            let temp_name = format!("__internal_{}", primitive.name);
            results.insert(temp_name.clone(), output_buffer.clone());
            last_result_name = temp_name;
        }
    }
    
    results.get(&last_result_name).cloned().unwrap_or_else(|| source_graphic.clone())
}
