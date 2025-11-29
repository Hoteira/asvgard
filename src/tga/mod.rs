mod header;

use crate::tga::header::{TgaHeader, ImageType};
use crate::utils::image::resize_image;
use crate::utils::compat::{Vec, String, ToString, vec, format};

pub fn render(data: &[u8], width: usize, height: usize) -> Result<Vec<u32>, String> {
    let header = TgaHeader::parse(data)?;

    let mut offset = 18 + header.id_length as usize;
    if header.color_map_type == 1 {
        let entry_size = (header.color_map_depth + 7) / 8;
        offset += header.color_map_length as usize * entry_size as usize;
    }

    if offset > data.len() {
        return Err("Invalid TGA offsets".to_string());
    }

    let pixel_data = &data[offset..];
    
    let native_w = header.width as usize;
    let native_h = header.height as usize;
    
    // Decode based on type
    let buffer = match header.image_type {
        ImageType::TrueColor => decode_uncompressed(pixel_data, &header, native_w, native_h)?,
        ImageType::RleTrueColor => decode_rle(pixel_data, &header, native_w, native_h)?,
        _ => return Err(format!("Unsupported TGA type: {:?}", header.image_type)),
    };

    // Resize if needed
    if native_w != width || native_h != height {
        Ok(resize_image(&buffer, native_w, native_h, width, height))
    } else {
        Ok(buffer)
    }
}

fn decode_uncompressed(data: &[u8], header: &TgaHeader, width: usize, height: usize) -> Result<Vec<u32>, String> {
    let bytes_per_pixel = (header.pixel_depth / 8) as usize;
    let mut buffer = vec![0u32; width * height];
    let is_top_left = header.is_top_left();

    // Expected data size check
    if data.len() < width * height * bytes_per_pixel {
        return Err("Not enough pixel data".to_string());
    }

    for y in 0..height {
        let target_y = if is_top_left { y } else { height - 1 - y };
        
        for x in 0..width {
            let i = (y * width + x) * bytes_per_pixel;
            let color = parse_pixel(&data[i..], bytes_per_pixel);
            buffer[target_y * width + x] = color;
        }
    }
    
    Ok(buffer)
}

fn decode_rle(data: &[u8], header: &TgaHeader, width: usize, height: usize) -> Result<Vec<u32>, String> {
    let bytes_per_pixel = (header.pixel_depth / 8) as usize;
    let mut buffer = vec![0u32; width * height];
    let is_top_left = header.is_top_left();
    
    let mut pixel_idx = 0;
    let mut offset = 0;
    let total_pixels = width * height;

    while pixel_idx < total_pixels {
        if offset >= data.len() {
            return Err("Unexpected EOF in RLE data".to_string());
        }

        let packet_header = data[offset];
        offset += 1;

        let count = (packet_header & 0x7F) as usize + 1;
        let is_rle = (packet_header & 0x80) != 0;

        if is_rle {
            // RLE packet: Read 1 pixel value, repeat `count` times
            if offset + bytes_per_pixel > data.len() { return Err("EOF reading RLE pixel".to_string()); }
            let color = parse_pixel(&data[offset..], bytes_per_pixel);
            offset += bytes_per_pixel;

            for _ in 0..count {
                if pixel_idx >= total_pixels { break; }
                let (x, y) = (pixel_idx % width, pixel_idx / width);
                let target_y = if is_top_left { y } else { height - 1 - y };
                buffer[target_y * width + x] = color;
                pixel_idx += 1;
            }
        } else {
            if offset + count * bytes_per_pixel > data.len() { return Err("EOF reading Raw packet".to_string()); }
            
            for _ in 0..count {
                if pixel_idx >= total_pixels { break; }
                let color = parse_pixel(&data[offset..], bytes_per_pixel);
                offset += bytes_per_pixel;
                
                let (x, y) = (pixel_idx % width, pixel_idx / width);
                let target_y = if is_top_left { y } else { height - 1 - y };
                buffer[target_y * width + x] = color;
                pixel_idx += 1;
            }
        }
    }

    Ok(buffer)
}

#[inline]
fn parse_pixel(data: &[u8], bpp: usize) -> u32 {
    match bpp {
        3 => {
            // BGR -> ARGB
            let b = data[0];
            let g = data[1];
            let r = data[2];
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        },
        4 => {
            // BGRA -> ARGB
            let b = data[0];
            let g = data[1];
            let r = data[2];
            let a = data[3];
            ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        },
        _ => 0xFF000000,
    }
}