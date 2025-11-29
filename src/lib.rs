//! # Asvgard
//!
//! `asvgard` is a lightweight, dependency-free (mostly) graphics rendering library written in Rust.
//! It provides a unified interface for loading, decoding, and rasterizing various image formats
//! including **SVG**, **PNG**, and **TGA**.
//!
//! ## Features
//!
//! - **SVG Support**: Custom-built XML parser and rasterizer supporting paths, shapes, strokes, and fills.
//! - **PNG Support**: From-scratch implementation of DEFLATE (zlib) decompression and filtering.
//! - **TGA Support**: Native support for uncompressed and RLE-compressed TGA files.
//! - **No External Image Crates**: All decoding logic is implemented internally for educational and lightweight purposes.
//!
//! ## Usage
//!
//! ```rust
//! use asvgard::prelude::*;
//!
//! // Load an image from bytes (detects format automatically)
//! let data = include_bytes!("../test.png");
//! let width = 800;
//! let height = 600;
//!
//! match load_image(data, width, height) {
//!     Ok(buffer) => {
//!         println!("Image loaded! Buffer size: {}", buffer.len());
//!         // buffer contains ARGB u32 pixels
//!     }
//!     Err(e) => eprintln!("Failed to load image: {}", e),
//! }
//! ```

pub mod svg;
pub mod png;
pub mod tga;
pub mod utils;

use std::cmp::min;

/// Common types and functions for easy import.
pub mod prelude {
    pub use crate::load_image;
    pub use crate::detect_type;
    pub use crate::ImageType;
    // Exporting the Canvas might be useful if moved to a shared location
    // pub use crate::svg::rasterizer::canva::Canvas; 
}

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Svg,
    Png,
    Tga,
    Unknown,
}

/// Detects the image format based on magic bytes or file heuristics.
///
/// # Arguments
///
/// * `data` - A byte slice containing the file data.
pub fn detect_type(data: &[u8]) -> ImageType {
    // PNG Signature: 89 50 4E 47 0D 0A 1A 0A
    if data.len() >= 8 && &data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
        return ImageType::Png;
    }

    // TGA Footer Signature (New TGA)
    if data.len() >= 26 {
        let footer_start = data.len() - 26;
        if &data[footer_start..footer_start + 16] == b"TRUEVISION-XFILE" {
            return ImageType::Tga;
        }
    }
    
    // TGA Header Heuristic (Old TGA fallback)
    if data.len() >= 18 {
        let color_map_type = data[1];
        let image_type = data[2];
        if (color_map_type == 0 || color_map_type == 1) && 
           (image_type == 2 || image_type == 3 || image_type == 10 || image_type == 11) {
               return ImageType::Tga;
        }
    }

    // SVG Heuristic (XML-like text check)
    let check_len = min(data.len(), 4096);
    if let Ok(s) = std::str::from_utf8(&data[0..check_len]) {
        if s.contains("<svg") || s.trim_start().starts_with("<?xml") {
            return ImageType::Svg;
        }
    }
    
    ImageType::Unknown
}

/// Loads and rasterizes an image from raw bytes into a pixel buffer.
///
/// The output is a `Vec<u32>` where each pixel is `0xAARRGGBB` (ARGB).
/// The image will be resized to fit the specified `width` and `height`.
///
/// # Arguments
///
/// * `data` - Raw file bytes.
/// * `width` - Desired output width.
/// * `height` - Desired output height.
pub fn load_image(data: &[u8], width: usize, height: usize) -> Result<Vec<u32>, String> {
    match detect_type(data) {
        ImageType::Svg => crate::svg::render(data, width, height),
        ImageType::Png => crate::png::render(data, width, height),
        ImageType::Tga => crate::tga::render(data, width, height),
        ImageType::Unknown => Err("Unknown or unsupported file type".to_string()),
    }
}