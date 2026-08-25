<a id="readme-top"></a>
<div align="center">
  <a href="https://github.com/dlang-supplemental/vello-d/actions/workflows/build.yml"><img src="https://github.com/dlang-supplemental/vello-d/actions/workflows/build.yml/badge.svg" alt="Build"></a>
  <a href="https://dlang-supplemental.github.io/docs/vello-d/index.html"><img src="https://img.shields.io/badge/docs-DLang%20Supplemental-22c55e?style=for-the-badge" alt="Docs | DLang Supplemental"></a>
  <a href="https://github.com/dlang-supplemental/vello-d/graphs/contributors"><img src="https://img.shields.io/github/contributors/dlang-supplemental/vello-d.svg?style=for-the-badge" alt="Contributors"></a>
  <a href="https://github.com/dlang-supplemental/vello-d/network/members"><img src="https://img.shields.io/github/forks/dlang-supplemental/vello-d.svg?style=for-the-badge" alt="Forks"></a>
  <a href="https://github.com/dlang-supplemental/vello-d/stargazers"><img src="https://img.shields.io/github/stars/dlang-supplemental/vello-d.svg?style=for-the-badge" alt="Stargazers"></a>
  <a href="https://github.com/dlang-supplemental/vello-d/issues"><img src="https://img.shields.io/github/issues/dlang-supplemental/vello-d.svg?style=for-the-badge" alt="Issues"></a>

  <h1 align="center">vello-d</h1>

  <p align="center">
    High-performance, GPU-accelerated 2D vector graphics for D, powered by Vello (Rust).
    <br />
    <br />
    <a href="https://dlang-supplemental.github.io/docs/vello-d/index.html"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/dlang-supplemental/vello-d/issues">Report Bug</a>
    &middot;
    <a href="https://github.com/dlang-supplemental/vello-d/issues">Request Feature</a>
  </p>
</div>

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about-the-project">About The Project</a></li>
    <li><a href="#getting-started">Getting Started</a></li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#changelog">Changelog</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>

## About The Project

Install: `dub add vello-d` (after building the Rust bridge below).

### Key Features

- **GPU Backends**: Native support for **Vulkan** and **DX12** via `wgpu`.
- **Surfaces**: Windows `HWND`, X11 (`Display*` + window id), Wayland (`wl_display*` + `wl_surface*`).
- **Scene API**: fills, strokes, rounded rects, circles, Bézier paths, clip layers, linear/radial gradients, RGBA8 images, TrueType glyph runs, UTF-8 text (`drawText` / `measureText`).
- **Multi-Window Rendering**: Create multiple windows running on different GPU backends within a single process.
- **Threaded Rendering**: Support for decoupled background rendering to bypass the Win32 main-loop move/resize blocking issue.
- **RAII Bindings**: Ergonomic Dlang wrappers for Vello's scene and context management.

## Getting Started

### 1. Build the Rust Bridge

Vello is a Rust engine, so you first need to build the C-compatible bridge:

```powershell
cd vello_bridge
cargo build --release
```

See the [Quick start](https://dlang-supplemental.github.io/docs/vello-d/quick-start.html) guide for a full walkthrough.

## Usage

The `demos/` directory contains several configurations to demonstrate different rendering architectures:

| Configuration | Command | Description |
| :--- | :--- | :--- |
| **Simple** | `dub run -c simple` | A basic single-window Vulkan renderer. |
| **Multi-Backend** | `dub run -c multi` | **Vulkan + DX12** windows in one process (Shared Main Thread). |
| **Threaded** | `dub run -c threaded` | **Vulkan + DX12** windows (Background Threaded). **Highly Recommended.** |

### Text rendering (0.1.4+)

```d
ubyte[] font = /* load .ttf */;
float w = scene.drawText(font, 0, 32.0f, 40.0, 120.0, 255, 255, 255, 255, "Hello!");
```

See [Text rendering](https://dlang-supplemental.github.io/docs/vello-d/text-rendering.html) for `drawGlyphs`, constraints, and C ABI details.

### Why Threaded Rendering?

On Windows, standard single-threaded main loops (like GLFW's `glfwPollEvents`) block when a user moves or resizes a window. By using the `-c threaded` configuration, each window renders in its own background thread, ensuring **silky smooth animation** even while the window is being dragged.

### Documentation

- **[Antora docs](https://dlang-supplemental.github.io/docs/vello-d/index.html)**: scene API, demos, troubleshooting, changelog
- **[Dlang Lessons Learned](source/vello/LESSONS.md)**: Tips for FFI safety and RAII.
- **[Rust Lessons Learned](vello_bridge/LESSONS.md)**: Insights into `wgpu` feature selection and cross-adapter compatibility.
- **[TUTORIAL.md](TUTORIAL.md)**: Recreate the threaded multi-backend demo.

## Changelog

See [CHANGELOG.adoc](CHANGELOG.adoc) and the [docs changelog index](https://dlang-supplemental.github.io/docs/vello-d/changelog.html).

## License

MIT / Apache 2.0

## Contact

DLang Supplemental — dlang@devcentr.org

Project Link: https://github.com/dlang-supplemental/vello-d

Site: https://dlang-supplemental.github.io

<p align="right">(<a href="#readme-top">back to top</a>)</p>
