# Lessons Learned: vello-d (Dlang)

## 1. FFI Safety & RAII
*   **The Issue**: Manual FFI pointers are prone to memory leaks.
*   **The Lesson**: Use the RAII pattern in D classes. Wrap Rust pointers in `struct VelloHandle` or classes with `~this()` to automatically call `vello_context_free` and `vello_scene_free`.

## 2. D-Rust Struct Mapping
*   **The Issue**: Mismatched struct layouts (like `Color`) lead to garbage values on the Rust side.
*   **The Lesson**: For simple values (colors, points, rects), use D-side POD structs that EXACTLY mirror the Rust equivalents (e.g., `float r, g, b, a;`).

## 3. Graphics Loop Stability
*   **The Issue**: Appending to a Vello scene without resetting it causes memory explosions.
*   **The Lesson**: Always call `scene.reset()` at the start of every frame in your D loop. Vello's `Scene` is a command buffer, not a persistent object.

## 4. Multi-Threaded Rendering
*   **The Issue**: Windows' modal loop blocks drawing during move/resize.
*   **The Lesson**: Move `ctx.render(scene)` to a dedicated `Thread`. Dlang's `core.thread.osthread` is perfect for this. Decoupling ensures your app stays fluid (60fps) even while the OS is busy dragging the window.

## 5. The "GPU Warmup" Splash
*   **The Issue**: Modern backends like DX12 can take 10+ seconds to initialize/compile shaders, leaving a "frozen white screen."
*   **The Lesson**: Revealing the window immediately and using **Native GDI** (`GetDC`, `BitBlt`) to draw a simple "Loading..." text/progress bar provides instant feedback while the heavy GPU work happens in a background thread. This is a crucial UX pattern for high-end graphics libraries.

## 6. Startup Responsiveness
*   **The Issue**: Initializing multiple `wgpu` contexts sequentially on the main thread causes a "not responding" cursor.
*   **The Lesson**: Always interleave `glfwPollEvents()` during initialization OR move `new Context()` into the background thread. This keeps the Windows message queue processing from the very first frame.

## 7. Build Simplification
*   **The Issue**: End-users shouldn't need to configure include paths manually.
*   **The Lesson**: Use `pragma(lib, "vello_bridge.dll")` and include the DLL in the repository or a central build folder to simplify the `dub` experience.
