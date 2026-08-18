module vello.bindings;

extern (C) {
    /// Opaque handle for a Vello scene (path builder + transform included).
    struct VelloScene;

    /// Opaque handle for a Vello rendering context.
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

    void vello_init_logging();

    VelloContext* vello_context_new_for_hwnd(
        void* hwnd, void* hinstance, uint width, uint height, VelloBackend backend
    );
    VelloContext* vello_context_new_for_x11(
        void* display, ulong window, int screen, uint width, uint height, VelloBackend backend
    );
    VelloContext* vello_context_new_for_wayland(
        void* display, void* surface, uint width, uint height, VelloBackend backend
    );

    void vello_context_free(VelloContext* ctx);
    void vello_context_render(VelloContext* ctx, VelloScene* scene);
    void vello_context_resize(VelloContext* ctx, uint width, uint height);

    VelloScene* vello_scene_new();
    void vello_scene_free(VelloScene* scene);
    void vello_scene_reset(VelloScene* scene);
    void vello_scene_set_transform(VelloScene* scene, double a, double b, double c, double d, double e, double f);
    void vello_scene_reset_transform(VelloScene* scene);

    void vello_scene_fill_rect(
        VelloScene* scene, double x, double y, double w, double h,
        ubyte r, ubyte g, ubyte b, ubyte a
    );
    void vello_scene_fill_rounded_rect(
        VelloScene* scene, double x, double y, double w, double h, double radius,
        ubyte r, ubyte g, ubyte b, ubyte a
    );
    void vello_scene_stroke_rect(
        VelloScene* scene, double x, double y, double w, double h, double width,
        ubyte r, ubyte g, ubyte b, ubyte a
    );
    void vello_scene_fill_circle(
        VelloScene* scene, double cx, double cy, double radius,
        ubyte r, ubyte g, ubyte b, ubyte a
    );
    void vello_scene_stroke_line(
        VelloScene* scene, double x0, double y0, double x1, double y1, double width,
        ubyte r, ubyte g, ubyte b, ubyte a
    );
    void vello_scene_fill_linear_rect(
        VelloScene* scene, double x, double y, double w, double h,
        double x0, double y0, double x1, double y1,
        ubyte r0, ubyte g0, ubyte b0, ubyte a0,
        ubyte r1, ubyte g1, ubyte b1, ubyte a1
    );
    void vello_scene_fill_radial_rect(
        VelloScene* scene, double x, double y, double w, double h,
        double cx, double cy, float radius,
        ubyte r0, ubyte g0, ubyte b0, ubyte a0,
        ubyte r1, ubyte g1, ubyte b1, ubyte a1
    );

    void vello_scene_push_clip_rect(VelloScene* scene, double x, double y, double w, double h);
    void vello_scene_pop_layer(VelloScene* scene);

    void vello_scene_path_begin(VelloScene* scene);
    void vello_scene_path_move_to(VelloScene* scene, double x, double y);
    void vello_scene_path_line_to(VelloScene* scene, double x, double y);
    void vello_scene_path_quad_to(VelloScene* scene, double c1x, double c1y, double x, double y);
    void vello_scene_path_cubic_to(
        VelloScene* scene, double c1x, double c1y, double c2x, double c2y, double x, double y
    );
    void vello_scene_path_close(VelloScene* scene);
    void vello_scene_path_fill(VelloScene* scene, ubyte r, ubyte g, ubyte b, ubyte a);
    void vello_scene_path_stroke(VelloScene* scene, double width, ubyte r, ubyte g, ubyte b, ubyte a);

    /// RGBA8 unpremultiplied, row-major, `srcW * srcH * 4` bytes.
    void vello_scene_draw_image(
        VelloScene* scene, double x, double y, double dstW, double dstH,
        uint srcW, uint srcH, const(ubyte)* pixels
    );

    struct VelloGlyph {
        uint id;
        float x;
        float y;
    }

    void vello_scene_draw_glyphs(
        VelloScene* scene,
        const(ubyte)* fontBytes, size_t fontLen, uint fontIndex,
        float fontSize, double tx, double ty,
        ubyte r, ubyte g, ubyte b, ubyte a,
        const(VelloGlyph)* glyphs, size_t glyphCount
    );
}
