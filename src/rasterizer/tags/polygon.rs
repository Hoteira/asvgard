use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::dda::Rasterizer;
use crate::rasterizer::raster::{PathRasterizer, Point};
use crate::rasterizer::tags::clippath::{get_clip_path_id, ClipMask};
use crate::rasterizer::tags::path::PathCommand;
use crate::rasterizer::tags::path::PathCommand::MoveTo;
use crate::utils::color::get_fill;
use crate::utils::transform::Transform;

pub fn draw_polygon(tag: &Tag, defs: &HashMap<String, Tag>, map: &mut Canvas, transform: &Transform) {
    let mut fill = get_fill(tag).resolve(defs);
    let mut points = get_points(tag);

    if points.is_empty() { return; }


    let cx = points.iter().map(|p| p.x).sum::<f32>() / points.len() as f32;
    let cy = points.iter().map(|p| p.y).sum::<f32>() / points.len() as f32;

    let mut commands = Vec::new();

    commands.push(MoveTo(points[0]));
    for p in &points[1..] {
        commands.push(PathCommand::LineTo(*p))
    }
    commands.push(PathCommand::ClosePath);

    let transformed_path = crate::rasterizer::tags::path::apply_transform_to_path(&commands, transform);

    let mut path_rasterizer = PathRasterizer::new();
    path_rasterizer.build_lines_from_path(&transformed_path, 1.0, 1.0);

    let renderer = Rasterizer::new(
        path_rasterizer.bounds.width.ceil() as usize + 1,
        path_rasterizer.bounds.height.ceil() as usize,
    );

    let r_w = renderer.width;
    let r_h = renderer.height;
    let bitmap = renderer.draw(&path_rasterizer.v_lines, &path_rasterizer.m_lines).to_bitmap();

    let (sx, sy) = transform.get_scale();

    fill.scale(sx.min(sy));

    let mut color_map = crate::rasterizer::tags::path::generate_color_map(
        &bitmap,
        &fill,
        r_w,
        r_h,
        path_rasterizer.bounds.x,
        path_rasterizer.bounds.y,
    );

    if let Some(clip_id) = get_clip_path_id(tag) {
        if let Some(clippath_tag) = defs.get(&clip_id) {
            if clippath_tag.name == "clipPath" {
                if let Some(clip_mask) = ClipMask::from_clippath_tag(clippath_tag, sx.min(sy)) {
                    clip_mask.apply_to_bitmap(
                        &mut color_map,
                        path_rasterizer.bounds.x,
                        path_rasterizer.bounds.y,
                        r_w,
                        r_h
                    );
                }
            }
        }
    }

    let draw_x = path_rasterizer.bounds.x.round() as usize;
    let draw_y = path_rasterizer.bounds.y.round() as usize;

    map.add_buffer(
        &color_map,
        draw_x,
        draw_y,
        r_w,
        r_h
    );
}

#[inline]
pub fn get_points(tag: &Tag) -> Vec<Point> {
    let Some(points_str) = tag.params.get("points") else {
        return Vec::new();
    };

    let nums: Vec<f32> = points_str
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

    nums.chunks_exact(2)
        .map(|chunk| Point {
            x: chunk[0],
            y: chunk[1]
        })
        .collect()
}