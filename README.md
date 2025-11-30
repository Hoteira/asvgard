<div align="center">
  <br>
  <img src="https://raw.githubusercontent.com/Hoteira/asvgard/refs/heads/master/icon/icon.svg" alt="Asvgard Logo" width="120" height="120">

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

## Fonts & Licenses

### Embedded Font
This library includes **Cascadia Code Nerd Font Mono** for text rendering in SVGs.

- **Font**: Cascadia Code Nerd Font Mono Regular
- **License**: SIL Open Font License 1.1 (OFL)
- **Copyright**:
    - © 2019 Microsoft Corporation (Cascadia Code)
    - © 2021 Ryan L McIntyre (Nerd Fonts patches)
- **License File**: See `fonts/LICENSE-OFL.txt`

The SIL OFL allows free use, modification, and redistribution. The font is bundled with this library for convenience and does not affect the MIT license of the code.

### Links
- [Cascadia Code](https://github.com/microsoft/cascadia-code) (Original font)
- [Nerd Fonts](https://www.nerdfonts.com/) (Patched version)

## License

Licensed under the [MIT License](LICENSE).

