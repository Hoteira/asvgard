<div align="center">
  <br>
  <img src="icon/icon.svg" alt="Asvgard Logo" width="120" height="120">

# Asvgard

**High-performance SVG rasterizer written in Rust**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<sub>🦀 Pure Rust • ⚡ Fast Rendering • 🖼️ Vector Graphics</sub>
</div>

<br>

## Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/asvgard.git
cd asvgard

# Run the demo (renders built-in bunny.svg)
cargo run --release
```

## Features

- 🚀 **Fast Rasterization** — Optimized rendering pipeline for vector graphics
- 📐 **Smart Transforms** — Automatically handles `viewBox` scaling and centering
- 🧩 **Modular Architecture** — Clean separation between `parser` and `rasterizer`
- 🖥️ **Live Preview** — Built-in windowed viewer using `minifb`
- 🔗 **Defs & ID Support** — Resolves shared resources and definitions

## Architecture

```
src/
├── parser/        # XML parsing and tag structure
├── rasterizer/    # Drawing logic and pixel buffer management
├── utils/         # Math helpers (Matrices, Transforms)
└── main.rs        # Window management and event loop
```

## License

Licensed under the [MIT License](LICENSE).

---

<div align="center">
  <sub>Painting pixels with vectors 🎨</sub>
</div>