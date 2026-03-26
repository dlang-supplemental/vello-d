# vello-d

High-performance, GPU-accelerated 2D vector graphics for **Dlang**, powered by **Vello** (Rust).

## 🚀 Key Features
- **GPU Backends**: Native support for **Vulkan** and **DX12** via `wgpu`.
- **Multi-Window Rendering**: Create multiple windows running on different GPU backends within a single process.
- **Threaded Rendering**: Support for decoupled background rendering to bypass the Win32 main-loop move/resize blocking issue.
- **RAII Bindings**: Ergonomic Dlang wrappers for Vello's scene and context management.

## 🛠️ Getting Started

### 1. Build the Rust Bridge
Vello is a Rust engine, so you first need to build the C-compatible bridge:
```powershell
cd vello_bridge
cargo build --release
```

### 2. Run the Dlang Demos
The `tests/` directory contains several configurations to demonstrate different rendering architectures:

| Configuration | Command | Description |
| :--- | :--- | :--- |
| **Simple** | `dub run -c simple` | A basic single-window Vulkan renderer. |
| **Multi-Backend** | `dub run -c multi` | **Vulkan + DX12** windows in one process (Shared Main Thread). |
| **Threaded** | `dub run -c threaded` | **Vulkan + DX12** windows (Background Threaded). **Highly Recommended.** |

### 💡 Why Threaded Rendering?
On Windows, standard single-threaded main loops (like GLFW's `glfwPollEvents`) block when a user moves or resizes a window. By using the `-c threaded` configuration, each window renders in its own background thread, ensuring **silky smooth animation** even while the window is being dragged.

## 📄 Documentation
- **[Integration Challenges](file:///C:/Users/rjamd/.gemini/antigravity/brain/b9e39486-8d48-4212-a36a-956137da2bd4/vello_d_integration_challenges.md)**: A technical retrospective on the staging texture strategy and sRGB format compatibility.
- **[Dlang Lessons Learned](file:///Z:/code/github.com/dlang-supplemental/vello-d/source/vello/LESSONS.md)**: Tips for FFI safety and RAII.
- **[Rust Lessons Learned](file:///Z:/code/github.com/dlang-supplemental/vello-d/vello_bridge/LESSONS.md)**: Insights into `wgpu` feature selection and cross-adapter compatibility.

## ⚖️ License
MIT / Apache 2.0
