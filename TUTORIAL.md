# Tutorial: Recreating the Vello-D Demo suite

This tutorial walks you through recreating the advanced **Threaded Multi-Backend Demo** from scratch.

## Prerequisites

- **DMD/LDC** (Dlang Compiler)
- **Rust/Cargo** (for the bridge)
- **GLFW** (via `glfw-d` or `bindbc-glfw`)

---

## Step 1: The Rust FFI Bridge

Create a Rust library that exports `extern "C"` functions for Vello.

1.  **Staging Texture Strategy**: Vello requires `Rgba8Unorm` storage textures. Windows surfaces often want `Bgra8UnormSrgb`.
    - **Tip**: Create an internal `Texture` in Rust with `Rgba8Unorm`. Render Vello to that texture, then use `copy_texture_to_texture` to blit it to the final swapchain if the formats are compatible.
2.  **Backend Selection**: Allow passing an `enum` for **Vulkan** vs **DX12**. This is essential for testing cross-driver stability.

## Step 2: Dlang RAII Wrappers

Wrap the raw pointers from Rust in D classes to ensure proper memory cleanup.

```d
class Context {
    private VelloContext* _handle;
    this(...) { _handle = vello_context_new_for_hwnd(...); }
    ~this() { vello_context_free(_handle); }
    void render(Scene s) { vello_context_render(_handle, s.handle); }
}
```

## Step 3: Threaded Rendering (The High-Perf Pattern)

Windows blocks the main thread during window move/resize. To fix this, move your render loop to a separate thread.

1.  **Create a RenderThread**: Subclass `core.thread.Thread`.
2.  **Decouple Context**: Pass the `HWND` to the thread and have the thread call `new Context()`.
3.  **The Warpup Delay**: Since DX12 takes 10s to compile shaders, the thread handles this in the background while the main thread stays responsive.

## Step 4: The GDI Splash Screen (The UX Polish)

To avoid a white frozen screen during the 10-second DX12 startup:

1.  **Immediate Show**: Show the window immediately with `glfwShowWindow`.
2.  **Main Loop GDI**: In your main `while(!glfwWindowShouldClose)` loop, detect if the render thread is `ready`.
3.  **Draw Loading Status**: Use `GetDC(hwnd)` and `BitBlt` to draw a "Loading..." message.

    ```d
    if (!t1.ready) {
        auto hdc = GetDC(hwnd);
        DrawTextA(hdc, "Compiling Shaders...", ...);
        ReleaseDC(hwnd);
    }
    ```

4.  **Handoff**: Once `ready` becomes true, stop the GDI draws. Vello will automatically take over the surface.

---

## Conclusion

By following these steps, you build a "professional grade" graphics interface that handles complex backend synchronization, format mismatches, and OS-level blocking behaviors.

**Happy Coding with Vello + Dlang!**

## Building the Demos

```powershell
cd demos
dub build -c threaded
dub run -c threaded
```
