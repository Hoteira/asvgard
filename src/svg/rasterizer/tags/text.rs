use crate::utils::compat::{HashMap, VecDeque, Vec, String, ToString};
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::utils::transform::Transform;
use crate::svg::utils::color::{get_fill, get_stroke, Paint};
use crate::svg::utils::coords::parse_length;
use crate::svg::utils::effects::get_stroke_width;
use crate::svg::rasterizer::tags::path::generate_color_map;
use crate::svg::rasterizer::raster::{Line as RasterLine, PathRasterizer};
use crate::svg::rasterizer::dda::Rasterizer;
use crate::svg::rasterizer::stroke::draw_stroke;
use titanf::TrueTypeFont;
use crate::utils::compat::FloatExt;

pub fn draw_text(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let text = tag.text_content.clone();
    if text.trim().is_empty() {
        return;
    }

    let x = tag.params.get("x").map(|s| parse_length(s, 0.0, canvas.width as f32)).unwrap_or(0.0);
    let y = tag.params.get("y").map(|s| parse_length(s, 0.0, canvas.height as f32)).unwrap_or(0.0);
    let font_size = tag.params.get("font-size").and_then(|s| s.parse::<f32>().ok()).unwrap_or(16.0);

    // Embedded font for no_std compatibility
    let font_data = include_bytes!("../../../../CaskaydiaMonoNerdFontMono-Regular.ttf");
    
    let mut font = match TrueTypeFont::load_font(font_data) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut fill = get_fill(tag).resolve(defs);
    if tag.params.get("fill").is_none() {
        fill = Paint::Solid(0xFF000000);
    }
    
    let stroke = get_stroke(tag).resolve(defs);
    let stroke_width = get_stroke_width(tag);

    let (sx, sy) = transform.get_scale();
    
    let mut cursor_x = x;
    let cursor_y = y;

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    
    let half_stroke = if stroke_width > 0.0 { stroke_width / 2.0 } else { 0.0 };

    // Pass 1: Calculate Text Bounding Box (Screen Space)
    for c in text.chars() {
        let (metrics, _, _, _) = font.get_char_lines(c, font_size);
        if metrics.width > 0 && metrics.height > 0 {
            let glyph_origin_x = cursor_x + metrics.left_side_bearing as f32;
            let glyph_origin_y = cursor_y + metrics.base_line as f32; // Revert to +
            
            let gx = glyph_origin_x + half_stroke;
            let gy = glyph_origin_y + half_stroke;
            
            // Revert to +y bounding box logic
            let corners = [
                (gx - half_stroke, gy - half_stroke),
                (gx + metrics.width as f32 + half_stroke, gy - half_stroke),
                (gx - half_stroke, gy + metrics.height as f32 + half_stroke),
                (gx + metrics.width as f32 + half_stroke, gy + metrics.height as f32 + half_stroke),
            ];

            for (cx, cy) in corners {
                let (tx, ty) = transform.apply(cx, cy);
                if tx < min_x { min_x = tx; }
                if ty < min_y { min_y = ty; }
                if tx > max_x { max_x = tx; }
                if ty > max_y { max_y = ty; }
            }
        }
        cursor_x += metrics.advance_width as f32 + half_stroke;
    }
    
    let bbox_w = if max_x > min_x { max_x - min_x } else { 0.0 };
    let bbox_h = if max_y > min_y { max_y - min_y } else { 0.0 };
    let bbox_x = if min_x == f32::MAX { 0.0 } else { min_x };
    let bbox_y = if min_y == f32::MAX { 0.0 } else { min_y };

    // Pass 2: Draw
    cursor_x = x;

    for c in text.chars() {
        let (metrics, v_lines, m_lines, stroke_lines) = font.get_char_lines(c, font_size);
        
        let glyph_origin_x = cursor_x + metrics.left_side_bearing as f32;
        let glyph_origin_y = cursor_y + metrics.base_line as f32; // Revert to +
        
        let gx = glyph_origin_x + half_stroke;
        let gy = glyph_origin_y + half_stroke;

        // Transform Fill Lines (Revert to +y)
        let mut t_v: Vec<RasterLine> = Vec::new();
        let mut t_m: Vec<RasterLine> = Vec::new();

        for l in v_lines.iter().chain(m_lines.iter()) {
            let (p0x, p0y) = transform.apply(gx + l.x0, gy + l.y0);
            let (p1x, p1y) = transform.apply(gx + l.x1, gy + l.y1);
            let line = RasterLine::new(p0x, p0y, p1x, p1y, 1.0); // Revert dir to 1.0

            if line.dx_is_zero {
                t_v.push(line);
            } else {
                t_m.push(line);
            }
        }

        // Draw Fill
        if !fill.is_none() && metrics.width > 0 {
            let mut f_min_x = f32::MAX;
            let mut f_min_y = f32::MAX;
            let mut f_max_x = f32::MIN;
            let mut f_max_y = f32::MIN;
            
            for l in t_v.iter().chain(t_m.iter()) {
                f_min_x = f_min_x.min(l.x0).min(l.x1);
                f_min_y = f_min_y.min(l.y0).min(l.y1);
                f_max_x = f_max_x.max(l.x0).max(l.x1);
                f_max_y = f_max_y.max(l.y0).max(l.y1);
            }
            
            if f_max_x > f_min_x && f_max_y > f_min_y {
                let draw_x = f_min_x.floor() as isize;
                let draw_y = f_min_y.floor() as isize;
                
                let offset_x = draw_x as f32;
                let offset_y = draw_y as f32;
                
                let r_w = (f_max_x - offset_x).ceil() as usize + 1;
                let r_h = (f_max_y - offset_y).ceil() as usize + 1;
                
                // Safety check
                if r_w < 20000 && r_h < 20000 {
                    let shift_lines = |lines: &[RasterLine]| -> Vec<RasterLine> {
                        lines.iter().map(|l| RasterLine::new(l.x0 - offset_x, l.y0 - offset_y, l.x1 - offset_x, l.y1 - offset_y, 1.0)).collect()
                    };
                    
                    let local_v = shift_lines(&t_v);
                    let local_m = shift_lines(&t_m);
                    
                    let renderer = Rasterizer::new(r_w, r_h);
                    let bitmap = renderer.draw(&local_v, &local_m).to_bitmap();
                    
                    let color_map = generate_color_map(
                        &bitmap,
                        &fill,
                        r_w,
                        r_h,
                        offset_x,
                        offset_y,
                        bbox_x,
                        bbox_y,
                        bbox_w,
                        bbox_h
                    );
                    
                    canvas.add_buffer(&color_map, draw_x, draw_y, r_w, r_h);
                }
            }
        }

        // Draw Stroke
        if !stroke.is_none() && stroke_width > 0.0 {
            let t_stroke: Vec<RasterLine> = stroke_lines.iter()
                .filter(|l| (l.x0 - l.x1).abs() > 1e-4 || (l.y0 - l.y1).abs() > 1e-4)
                .map(|l| {
                    let (p0x, p0y) = transform.apply(gx + l.x0, gy + l.y0); // Revert to +y
                    let (p1x, p1y) = transform.apply(gx + l.x1, gy + l.y1);
                    RasterLine::new(p0x, p0y, p1x, p1y, 1.0)
                }).collect();
            
            if !t_stroke.is_empty() {
                let contours = group_connected_lines(t_stroke);
                
                let (sx, sy) = transform.get_scale();
                let scaled_stroke_width = stroke_width * (sx + sy) / 2.0;

                for contour in contours {
                    let mut stroke_rasterizer = PathRasterizer::new();
                    for l in contour {
                        stroke_rasterizer.insert_line(l.x0, l.y0, l.x1, l.y1, 1.0);
                    }
                    draw_stroke(canvas, &stroke_rasterizer, stroke.clone(), scaled_stroke_width, bbox_x, bbox_y, bbox_w, bbox_h);
                }
            }
        }
        
        cursor_x += metrics.advance_width as f32 + half_stroke;
    }
}

fn group_connected_lines(lines: Vec<RasterLine>) -> Vec<Vec<RasterLine>> {
    let mut contours: Vec<Vec<RasterLine>> = Vec::new();
    let mut remaining_lines = lines;

    const EPSILON: f32 = 1e-3;

    while let Some(current_line) = remaining_lines.pop() {
        let mut current_contour: VecDeque<RasterLine> = VecDeque::new();
        current_contour.push_back(current_line);
        
        let mut changed = true;

        while changed {
            changed = false;
            let mut next_remaining = Vec::new();
            
            let start_pt = if let Some(l) = current_contour.front() { (l.x0, l.y0) } else { break };
            let end_pt = if let Some(l) = current_contour.back() { (l.x1, l.y1) } else { break };
            
            for line in remaining_lines.drain(..) {
                if (line.x0 - end_pt.0).abs() < EPSILON && (line.y0 - end_pt.1).abs() < EPSILON {
                    current_contour.push_back(line);
                    changed = true;
                } else if (line.x1 - start_pt.0).abs() < EPSILON && (line.y1 - start_pt.1).abs() < EPSILON {
                    current_contour.push_front(line);
                    changed = true;
                } else {
                    next_remaining.push(line);
                }
            }
            remaining_lines = next_remaining;
        }
        contours.push(current_contour.into_iter().collect());
    }
    contours
}
