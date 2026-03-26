module vello.bindings;

extern(C) {
    /// Opaque handle for a vello::Scene
    struct VelloScene;

    /// Opaque handle for a Vello rendering context
    struct VelloContext;

    enum VelloBackend {
        All = 0,
        Vulkan = 1,
        Dx12 = 2,
        Dx11 = 3,
        Metal = 4,
        Gl = 5,
        BrowserWebGpu = 6,
    }

    /// Initialize logging for the bridge.
    void vello_init_logging();
    VelloContext* vello_context_new_for_hwnd(
        void* hwnd, void* hinstance, uint width, uint height, VelloBackend backend
    );

    /// Free a rendering context.
    void vello_context_free(VelloContext* ctx);

    /// Render a scene to the context.
    void vello_context_render(VelloContext* ctx, VelloScene* scene);

    /// Resize the context's internal buffers.
    void vello_context_resize(VelloContext* ctx, uint width, uint height);

    /// Create a new scene.
    VelloScene* vello_scene_new();

    /// Free a previously created scene.
    void vello_scene_free(VelloScene* scene);

    /// Reset a scene for the next frame.
    void vello_scene_reset(VelloScene* scene);

    /// Draw a filled rectangle.
    void vello_scene_fill_rect(
        VelloScene* scene, double x, double y, double w, double h, 
        ubyte r, ubyte g, ubyte b, ubyte a
    );
}
