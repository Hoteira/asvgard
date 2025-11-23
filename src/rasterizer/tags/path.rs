use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::dda::Rasterizer;
use crate::rasterizer::raster::{PathRasterizer, Point};
use crate::utils::color::{Paint, get_fill, get_stroke};
use crate::rasterizer::tags::clippath::{get_clip_path_id, ClipMask};
use crate::utils::transform::Transform;
use std::collections::HashMap;
use crate::rasterizer::map::Map;

#[derive(Debug)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CubicBezier(Point, Point, Point),
    QuadraticBezier(Point, Point),
    Arc {
        rx: f32,
        ry: f32,
        x_axis_rotation: f32,
        large_arc_flag: bool,
        sweep_flag: bool,
        end: Point,
    },
    ClosePath,
}

pub fn draw_path(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    map: &mut Map,
    transform: &Transform
) {
    let mut fill = get_fill(tag).resolve(defs);
    let stroke = get_stroke(tag).resolve(defs);

    let d = match tag.params.get("d") {
        Some(d) => d,
        None => {
            return;
        }
    };

    let d_path = parse_path_data(d);

    let transformed_path = apply_transform_to_path(&d_path, transform);

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

    let mut color_map = generate_color_map(
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

    let draw_x = path_rasterizer.bounds.x.floor() as usize;
    let draw_y = path_rasterizer.bounds.y.floor() as usize;

    map.add_buffer(
        &color_map,
        draw_x,
        draw_y,
        r_w,
        r_h
    );
}

pub fn parse_path_data(d: &str) -> Vec<PathCommand> {
    let mut d_path: Vec<PathCommand> = Vec::new();
    let mut current_command = ' ';
    let mut current_number = String::new();
    let mut args: Vec<f32> = Vec::new();
    let mut current_pos = Point { x: 0.0, y: 0.0 };
    let mut subpath_start = Point { x: 0.0, y: 0.0 };

    for c in d.chars() {
        if c.is_whitespace() || c == ',' {
            if !current_number.is_empty() {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
            }
            continue;
        }

        if c.is_alphabetic() {
            if !current_number.is_empty() {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
            }

            if current_command != ' ' && !args.is_empty() {
                process_command(current_command, &mut args, &mut d_path, &mut current_pos, &mut subpath_start);
            }

            current_command = c;
            continue;
        }

        if (c == '-' || c == '+') && !current_number.is_empty() {
            let last_char = current_number.chars().last().unwrap();
            if last_char == 'e' || last_char == 'E' {
                current_number.push(c);
            } else {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
                current_number.push(c);
            }
        } else {
            current_number.push(c);
        }
    }

    if !current_number.is_empty() {
        args.push(current_number.parse::<f32>().unwrap_or(0.0));
    }

    if current_command != ' ' {
        if current_command == 'Z' || current_command == 'z' || !args.is_empty() {
            process_command(current_command, &mut args, &mut d_path, &mut current_pos, &mut subpath_start);
        }
    }

    d_path
}

fn process_command(
    command: char,
    args: &mut Vec<f32>,
    d_path: &mut Vec<PathCommand>,
    current_pos: &mut Point,
    subpath_start: &mut Point
) {
    let is_relative = command.is_lowercase();
    let cmd = command.to_ascii_uppercase();

    match cmd {
        'M' => {
            let mut i = 0;
            while i + 1 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i], current_pos.y + args[i + 1])
                } else {
                    (args[i], args[i + 1])
                };

                if i == 0 {
                    d_path.push(PathCommand::MoveTo(Point { x, y }));
                    *subpath_start = Point { x, y };
                } else {
                    d_path.push(PathCommand::LineTo(Point { x, y }));
                }

                *current_pos = Point { x, y };
                i += 2;
            }
        }
        'L' => {
            let mut i = 0;
            while i + 1 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i], current_pos.y + args[i + 1])
                } else {
                    (args[i], args[i + 1])
                };

                d_path.push(PathCommand::LineTo(Point { x, y }));
                *current_pos = Point { x, y };
                i += 2;
            }
        }
        'H' => {
            for &val in args.iter() {
                let x = if is_relative { current_pos.x + val } else { val };
                d_path.push(PathCommand::LineTo(Point { x, y: current_pos.y }));
                current_pos.x = x;
            }
        }
        'V' => {
            for &val in args.iter() {
                let y = if is_relative { current_pos.y + val } else { val };
                d_path.push(PathCommand::LineTo(Point { x: current_pos.x, y }));
                current_pos.y = y;
            }
        }
        'C' => {
            let mut i = 0;
            while i + 5 < args.len() {
                let (x1, y1, x2, y2, x, y) = if is_relative {
                    (
                        current_pos.x + args[i], current_pos.y + args[i + 1],
                        current_pos.x + args[i + 2], current_pos.y + args[i + 3],
                        current_pos.x + args[i + 4], current_pos.y + args[i + 5]
                    )
                } else {
                    (args[i], args[i + 1], args[i + 2], args[i + 3], args[i + 4], args[i + 5])
                };

                d_path.push(PathCommand::CubicBezier(
                    Point { x: x1, y: y1 },
                    Point { x: x2, y: y2 },
                    Point { x, y }
                ));
                *current_pos = Point { x, y };
                i += 6;
            }
        }
        'Q' => {
            let mut i = 0;
            while i + 3 < args.len() {
                let (x1, y1, x, y) = if is_relative {
                    (
                        current_pos.x + args[i], current_pos.y + args[i + 1],
                        current_pos.x + args[i + 2], current_pos.y + args[i + 3]
                    )
                } else {
                    (args[i], args[i + 1], args[i + 2], args[i + 3])
                };

                d_path.push(PathCommand::QuadraticBezier(
                    Point { x: x1, y: y1 },
                    Point { x, y }
                ));
                *current_pos = Point { x, y };
                i += 4;
            }
        }
        'A' => {
            let mut i = 0;
            while i + 6 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i + 5], current_pos.y + args[i + 6])
                } else {
                    (args[i + 5], args[i + 6])
                };

                d_path.push(PathCommand::Arc {
                    rx: args[i],
                    ry: args[i + 1],
                    x_axis_rotation: args[i + 2],
                    large_arc_flag: args[i + 3] != 0.0,
                    sweep_flag: args[i + 4] != 0.0,
                    end: Point { x, y },
                });
                *current_pos = Point { x, y };
                i += 7;
            }
        }
        'Z' => {
            d_path.push(PathCommand::ClosePath);
            *current_pos = *subpath_start;
        }
        _ => {}
    }

    args.clear();
}

pub(crate) fn apply_transform_to_path(commands: &[PathCommand], transform: &Transform) -> Vec<PathCommand> {
    commands.iter().map(|cmd| {
        match cmd {
            PathCommand::MoveTo(p) => {
                let (x, y) = transform.apply(p.x, p.y);
                PathCommand::MoveTo(Point { x, y })
            }
            PathCommand::LineTo(p) => {
                let (x, y) = transform.apply(p.x, p.y);
                PathCommand::LineTo(Point { x, y })
            }
            PathCommand::CubicBezier(cp1, cp2, end) => {
                let (x1, y1) = transform.apply(cp1.x, cp1.y);
                let (x2, y2) = transform.apply(cp2.x, cp2.y);
                let (x, y) = transform.apply(end.x, end.y);
                PathCommand::CubicBezier(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }, Point { x, y })
            }
            PathCommand::QuadraticBezier(cp, end) => {
                let (x1, y1) = transform.apply(cp.x, cp.y);
                let (x, y) = transform.apply(end.x, end.y);
                PathCommand::QuadraticBezier(Point { x: x1, y: y1 }, Point { x, y })
            }
            PathCommand::Arc { rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end } => {
                let (x, y) = transform.apply(end.x, end.y);
                let (sx, sy) = transform.get_scale();
                PathCommand::Arc { rx: rx * sx, ry: ry * sy, x_axis_rotation: *x_axis_rotation, large_arc_flag: *large_arc_flag, sweep_flag: *sweep_flag, end: Point { x, y } }
            }
            PathCommand::ClosePath => PathCommand::ClosePath,
        }
    }).collect()
}

pub(crate) fn generate_color_map(
    bitmap: &[u8],
    paint: &Paint,
    width: usize,
    height: usize,
    bounds_x: f32,
    bounds_y: f32,

) -> Vec<u32> {
    let mut color_map = Vec::with_capacity(bitmap.len());

    match paint {
        Paint::Solid(color) => {
            let rgb = color & 0x00FFFFFF;
            for &coverage in bitmap {
                color_map.push(((coverage as u32) << 24) | rgb);
            }
        }
        
        Paint::LinearGradient(_) => {
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let coverage = bitmap[idx];
                    let svg_x = bounds_x + x as f32;
                    let svg_y = bounds_y + y as f32;
                    let color = paint.get_color_at(svg_x, svg_y);
                    color_map.push(((coverage as u32) << 24) | (color & 0xFFFFFF));
                }
            }
        }

        Paint::None | Paint::Reference(_) => {
            for _ in 0..bitmap.len() {
                color_map.push(0x00000000);
            }
        }
    }

    color_map
}
