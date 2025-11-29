//! PNG decoding and rendering.
//!
//! This module provides a custom implementation of PNG decoding, including
//! raw chunk parsing, DEFLATE decompression (via `zlib`), and inverse filtering.

pub mod chunks;
pub mod zlib;
mod filter;

use core::convert::TryInto;
use chunks::{IhdrChunk, ColorType, InterlaceMethod, PlteChunk, Rgb};
use crate::utils::image::resize_image;
use crate::utils::compat::{Vec, String, ToString, format, vec};

const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// Renders a PNG byte stream into a pixel buffer.
///
/// # Arguments
///
/// * `data` - Raw PNG file bytes.
/// * `width` - Target width.
/// * `height` - Target height.
pub fn render(data: &[u8], width: usize, height: usize) -> Result<Vec<u32>, String> {
    if data.len() < 8 || &data[..8] != SIGNATURE {
        return Err("Invalid PNG signature".to_string());
    }

    let mut pos = 8;
    let mut compressed_data = Vec::new();
    let mut ihdr_chunk: Option<IhdrChunk> = None;
    let mut plte_chunk: Option<PlteChunk> = None;

    while pos < data.len() {
        if pos + 4 > data.len() { break; }
        let length = u32::from_be_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
        pos += 4;

        if pos + 4 > data.len() { break; }
        let chunk_type = &data[pos..pos+4];
        pos += 4;

        if pos + length > data.len() { 
            return Err("Unexpected EOF while reading chunk data".to_string());
        }
        
        let chunk_data = &data[pos..pos+length];
        
        match chunk_type {
            b"IHDR" => ihdr_chunk = Some(parse_ihdr(chunk_data)?),
            b"PLTE" => plte_chunk = Some(parse_plte(chunk_data)?),
            b"IDAT" => compressed_data.extend_from_slice(chunk_data),
            b"IEND" => break,
            _ => {}
        }
        
        pos += length;

        if pos + 4 > data.len() { 
            return Err("Unexpected EOF while reading CRC".to_string());
        }
        pos += 4; // Skip CRC
    }

    let Some(ihdr) = ihdr_chunk else {
        return Err("IHDR chunk not found".to_string());
    };

    // Decompress the IDAT stream using our custom zlib implementation
    let decompressed = zlib::decompress(&compressed_data)?;
    
    // Apply inverse filtering to reconstruct raw pixel bytes
    let raw_pixels = filter::unfilter(&decompressed, &ihdr)?;
    
    // Convert raw bytes (RGB/RGBA/Indexed) to native u32 ARGB buffer
    let native_buffer = convert_to_native_buffer(&raw_pixels, &ihdr, plte_chunk.as_ref())?;
    
    let native_w = ihdr.width as usize;
    let native_h = ihdr.height as usize;

    // Resize if requested dimensions differ from native dimensions
    if native_w != width || native_h != height {
        Ok(resize_image(&native_buffer, native_w, native_h, width, height))
    } else {
        Ok(native_buffer)
    }
}

fn convert_to_native_buffer(data: &[u8], ihdr: &IhdrChunk, palette: Option<&PlteChunk>) -> Result<Vec<u32>, String> {
    let w = ihdr.width as usize;
    let h = ihdr.height as usize;
    let mut buffer = vec![0u32; w * h];

    match ihdr.color_type {
        ColorType::RGB => {
            if ihdr.bit_depth != 8 { return Err("Only 8-bit RGB supported for now".to_string()); }
            for i in 0..w*h {
                let src_idx = i * 3;
                let r = data[src_idx];
                let g = data[src_idx+1];
                let b = data[src_idx+2];
                buffer[i] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        },
        ColorType::RGBA => {
            if ihdr.bit_depth != 8 { return Err("Only 8-bit RGBA supported for now".to_string()); }
            for i in 0..w*h {
                let src_idx = i * 4;
                let r = data[src_idx];
                let g = data[src_idx+1];
                let b = data[src_idx+2];
                let a = data[src_idx+3];
                buffer[i] = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        },
        ColorType::Grayscale => {
             if ihdr.bit_depth != 8 { return Err("Only 8-bit Grayscale supported for now".to_string()); }
             for i in 0..w*h {
                let l = data[i];
                buffer[i] = 0xFF000000 | ((l as u32) << 16) | ((l as u32) << 8) | (l as u32);
            }
        },
        ColorType::Indexed => {
            if ihdr.bit_depth != 8 { return Err("Only 8-bit Indexed color supported for now".to_string()); }
            let palette = palette.ok_or("Indexed color image missing PLTE chunk")?;
            for i in 0..w*h {
                let idx = data[i] as usize;
                if idx >= palette.palette.len() {
                    return Err(format!("Palette index {} out of bounds", idx));
                }
                let color = palette.palette[idx];
                buffer[i] = 0xFF000000 | ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
            }
        },
        _ => return Err(format!("Color type {:?} not yet supported for rendering", ihdr.color_type)),
    }

    Ok(buffer)
}

fn parse_ihdr(data: &[u8]) -> Result<IhdrChunk, String> {
    if data.len() != 13 {
        return Err(format!("IHDR chunk has invalid length: {}", data.len()));
    }

    let width = u32::from_be_bytes(data[0..4].try_into().unwrap());
    let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
    let bit_depth = data[8];
    let color_type_byte = data[9];
    let compression = data[10];
    let filter = data[11];
    let interlace_byte = data[12];

    let color_type: ColorType = color_type_byte.try_into()
        .map_err(|e| format!("Invalid ColorType in IHDR: {}", e))?;
    
    let interlace: InterlaceMethod = interlace_byte.try_into()
        .map_err(|e| format!("Invalid InterlaceMethod in IHDR: {}", e))?;

    if compression != 0 { return Err(format!("Unsupported compression: {}", compression)); }
    if filter != 0 { return Err(format!("Unsupported filter: {}", filter)); }

    Ok(IhdrChunk {
        width,
        height,
        bit_depth,
        color_type,
        compression,
        filter,
        interlace,
    })
}

fn parse_plte(data: &[u8]) -> Result<PlteChunk, String> {
    if data.len() % 3 != 0 {
        return Err(format!("PLTE chunk length {} is not divisible by 3", data.len()));
    }
    
    let mut palette = Vec::with_capacity(data.len() / 3);
    for chunk in data.chunks(3) {
        palette.push(Rgb {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
        });
    }
    
    Ok(PlteChunk { palette })
}