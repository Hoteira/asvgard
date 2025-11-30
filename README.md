<div align="center">
  <img src="icon/icon.svg" alt="Asvgard Logo" width="120" height="120">

# Asvgard

**A Lightweight, Embeddable Vector Graphics Rasterizer**

[![Crates.io](https://img.shields.io/crates/v/asvgard.svg?style=flat-square)](https://crates.io/crates/asvgard)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg?style=flat-square)](https://docs.rust-embedded.org/book/)

<sub>SVG • PNG • TGA • Pure Rust • Zero External Dependencies</sub>
</div>

<br>

## 📖 Overview

**Asvgard** is a compact graphics rendering library designed for high-performance parsing and rasterization of **SVG**, **PNG**, and **TGA** formats. 

Built with a focus on educational value and portability, it implements all decoding logic from scratch—including a full **DEFLATE/zlib** implementation and **XML** parser—avoiding the bloat of heavy external crates. This makes it uniquely suited for bare-metal environments, kernel development, and lightweight GUI applications.

## ✨ Key Features

-   **🖼️ Multi-Format Support:**
    -   **SVG:** Supports paths, shapes (rect, circle, polygon), strokes, fills, and transforms.
    -   **PNG:** Custom DEFLATE decompression (Huffman coding, LZ77), adaptive filtering, and interlacing.
    -   **TGA:** Uncompressed and RLE-compressed TrueColor/Grayscale support.
-   **🛠️ Dependency Free:** No `image`, `flate2`, or `xml-rs`. Every byte of logic is internal.
-   **⚙️ no_std Compatible:** Designed to run in environments without an operating system (allocator required).
-   **⚡ SIMD Optimized:** Contains hand-written SIMD routines for critical filtering and blending operations.
-   **📐 Smart Transforms:** Automatically handles SVG `viewBox` scaling and centering.

## 🏗️ Architecture

```
asvgard/
├── src/
│   ├── svg/           # XML Parser & SVG Tag Logic
│   │   ├── parser/    # Lexer and Tag Tree builder
│   │   └── rasterizer/ # Scanline rasterizer & Filters (Blur, Offset)
│   ├── png/           # DEFLATE decompressor & Filter reconstruction
│   ├── tga/           # TGA Header parsing & RLE decoding
│   └── utils/         # Math, Transforms, and Compatibility layers
```

## 🚀 Usage

Asvgard provides a unified interface for loading images. It automatically detects the file format from the byte header.

```rust
use asvgard::prelude::*;

fn main() {
    // 1. Load raw bytes
    let data = include_bytes!("../assets/image.svg");
    
    // 2. Define target resolution
    let width = 800;
    let height = 600;

    // 3. Rasterize
    // Returns a Result<Vec<u32>, String> buffer in 0xAARRGGBB format
    match load_image(data, width, height) {
        Ok(buffer) => {
            println!("Successfully rendered {} pixels!", buffer.len());
        },
        Err(e) => eprintln!("Error rendering image: {}", e),
    }
}
```

## 📦 Installation

```toml
[dependencies]
asvgard = "0.1.0"
```

To use in a **no_std** environment, disable default features:

```toml
[dependencies]
asvgard = { version = "0.1.0", default-features = false }
```

## 📜 License

Distributed under the [MIT](LICENSE) license.