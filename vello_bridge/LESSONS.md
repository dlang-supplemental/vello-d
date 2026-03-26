# Lessons Learned: vello_bridge (Rust)

### 1. WGPU Surface Format Mismatch
*   **The Issue**: Surfaces often want `Rgba8UnormSrgb`. Vello storage bindings want `Rgba8Unorm`.
*   **The Lesson**: Do NOT render directly to the surface. Use a **staging texture** with `Rgba8Unorm` and then `copy_texture_to_texture` to the surface swapchain. This avoids all storage validation issues with sRGB formats.

### 2. Device Feature Selection
*   **The Issue**: Requesting `Features::all()` causes `ExperimentalFeaturesNotEnabled` failures on many drivers.
*   **The Lesson**: Use `Features::empty()` and only add what is strictly necessary. Vello 0.8.0 is surprisingly capable with minimal features on modern Vulkan/DX12.

### 3. FFI Callback Handling
*   **The Issue**: Rust's async nature conflicts with D's synchronous expectations.
*   **The Lesson**: Use `pollster::block_on` for one-off setup (like device creation) but try to keep the `render` call non-blocking by using `queue.submit()` and immediately returning to the host.

### 4. Raw Window Handles
*   **The Issue**: `raw-window-handle` versions (v0.5 vs v0.6) are a major source of ecosystem fragmentation.
*   **The Lesson**: Expect to manually reconstruct structs from raw pointers. Do not rely on intermediate wrapper libraries for windowing when doing FFI.
