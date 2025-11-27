<div align="center">
  <br>
  <img src="icon/icon.svg" alt="aSVGard Logo" width="140" height="140">

# aSVGard

**High-performance SVG rasterizer written in pure Rust**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/asvgard)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](https://docs.rust-embedded.org/book/)
[![crates.io](https://img.shields.io/crates/v/asvgard.svg)](https://crates.io/crates/asvgard)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/asvgard)

<sub>🦀 Pure Rust  • 🔧 Real-time Preview • 💎 Production Ready</sub>

---

</div>

<br>

## Features

- 🦀 **Pure Rust** — Memory-safe rendering with zero unsafe code
- 🎨 **Full SVG Support** — Paths, shapes, gradients, transforms, opacity
- 🔧 **Real-time Preview** — Built-in minifb integration for live debugging
- 📦 **`no_std` Ready** — Works in bare-metal environments (requires `alloc`)
- 💯 **Stable Rust** — No nightly features required
- ✨ **Anti-aliasing** — Smooth edges with configurable multisampling
- 🎯 **High Precision** — Sub-pixel accurate path rendering

## Supported Elements

- ✅ **Basic Shapes** — `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`
- ✅ **Complex Paths** — `M`, `L`, `H`, `V`, `C`, `S`, `Q`, `T`, `A`, `Z` commands
- ✅ **Transformations** — translate, rotate, scale, skew, matrix operations
- ✅ **Gradients** — Linear and radial gradients with multiple stops
- ✅ **Styling** — Fill, stroke, opacity, fill-rule (evenodd, nonzero)
- ✅ **Stroke Properties** — Width, linecap, linejoin, miterlimit, dasharray
- ✅ **Groups** — Nested `<g>` elements with cascading transforms
- 🔄 **Text Rendering** — In progress (TrueType font integration)
- 🔄 **Filters** — In progress (blur, drop-shadow, color matrix)
- ✅ **Clipping/Masking**
- ⏳ **Animations**

## no_std Support

aSVGard works in `no_std` environments with `alloc`:
```toml
[dependencies]
asvgard = { version = "x.x.0", default-features = false }
```

Perfect for:
- 🔧 **Embedded Systems** — ARM Cortex-M, RISC-V
- 💾 **Bootloaders** — Custom graphics in early boot stages
- 🖥️ **OS Kernels** — Native vector graphics without OS dependencies
- 🌐 **WebAssembly** — Browser-based rendering without JS overhead
- 🎮 **Game Engines** — Custom UI rendering pipelines

When `no_std` is enabled, the preview window (minifb) is automatically disabled.

## Future Features

- [x] Basic shape rendering
- [x] Path support with all commands
- [x] Transforms and gradients
- [x] Real-time preview window
- [x] Text rendering with TrueType fonts (via TiTanFont)
- [ ] SVG filters (blur, drop-shadow, color-matrix, etc.)
- [x] Clipping paths and masks
- [ ] Pattern fills
- [ ] Animation support (SMIL)
- [ ] Advanced typography (text-on-path, vertical text)

## License

Licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Open an issue or PR on GitHub.

---

<div align="center">


<br>

<sub>Built with 🦀 Rust and 📈</sub>

<br>


</div>
