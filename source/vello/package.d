module vello;

public import vello.bindings;

/**
 * Simplified color structure for D-side management.
 * Channels are 0..1, standard (non-inverted) alpha: 1 = opaque.
 */
struct Color {
    float r, g, b, a = 1;

    static Color fromBytes(ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        return Color(r / 255.0f, g / 255.0f, b / 255.0f, a / 255.0f);
    }
}

private void rgbaBytes(Color color, out ubyte r, out ubyte g, out ubyte b, out ubyte a) {
    r = cast(ubyte)(color.r * 255);
    g = cast(ubyte)(color.g * 255);
    b = cast(ubyte)(color.b * 255);
    a = cast(ubyte)(color.a * 255);
}

/**
 * High-level wrapper for a Vello rendering context.
 */
class Context {
    private VelloContext* _handle;

    /**
     * Create a context for a Windows HWND.
     */
    this(void* hwnd, void* hinstance, uint width, uint height, VelloBackend backend = VelloBackend.All) {
        _handle = vello_context_new_for_hwnd(hwnd, hinstance, width, height, backend);
    }

    private this(VelloContext* handle) {
        _handle = handle;
    }

    /// X11: `display` is `Display*`, `window` is the XID.
    static Context forX11(void* display, ulong window, int screen, uint width, uint height,
            VelloBackend backend = VelloBackend.All) {
        return new Context(vello_context_new_for_x11(display, window, screen, width, height, backend));
    }

    /// Wayland: `display` is `wl_display*`, `surface` is `wl_surface*`.
    static Context forWayland(void* display, void* surface, uint width, uint height,
            VelloBackend backend = VelloBackend.All) {
        return new Context(vello_context_new_for_wayland(display, surface, width, height, backend));
    }

    @property bool valid() const { return _handle !is null; }

    ~this() {
        if (_handle) {
            vello_context_free(_handle);
            _handle = null;
        }
    }

    void render(Scene scene) {
        if (_handle)
            vello_context_render(_handle, scene.handle);
    }

    void resize(uint width, uint height) {
        if (_handle)
            vello_context_resize(_handle, width, height);
    }

    @property VelloContext* handle() { return _handle; }
}

/**
 * High-level wrapper for a Vello scene.
 * Manages handle lifetimes automatically using RAII.
 */
class Scene {
    private VelloScene* _handle;

    this() {
        _handle = vello_scene_new();
    }

    ~this() {
        if (_handle) {
            vello_scene_free(_handle);
            _handle = null;
        }
    }

    /// Reset the scene to its initial empty state.
    void reset() {
        vello_scene_reset(_handle);
    }

    /// Affine matrix [a, b, c, d, e, f] as in kurbo (`| a c e ; b d f |`).
    void setTransform(double a, double b, double c, double d, double e, double f) {
        vello_scene_set_transform(_handle, a, b, c, d, e, f);
    }

    void resetTransform() {
        vello_scene_reset_transform(_handle);
    }

    /// Set a background color for the upcoming frame (huge rect fill).
    void clear(Color color) {
        ubyte r, g, b, a;
        rgbaBytes(color, r, g, b, a);
        fillRect(-1, -1, 1e9, 1e9, r, g, b, a);
    }

    void fillRect(double x, double y, double w, double h, ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_fill_rect(_handle, x, y, w, h, r, g, b, a);
    }

    void fillRoundedRect(double x, double y, double w, double h, double radius,
            ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_fill_rounded_rect(_handle, x, y, w, h, radius, r, g, b, a);
    }

    void strokeRect(double x, double y, double w, double h, double width,
            ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_stroke_rect(_handle, x, y, w, h, width, r, g, b, a);
    }

    void fillCircle(double cx, double cy, double radius, ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_fill_circle(_handle, cx, cy, radius, r, g, b, a);
    }

    void strokeLine(double x0, double y0, double x1, double y1, double width,
            ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_stroke_line(_handle, x0, y0, x1, y1, width, r, g, b, a);
    }

    void fillLinearRect(double x, double y, double w, double h,
            double x0, double y0, double x1, double y1,
            ubyte r0, ubyte g0, ubyte b0, ubyte a0,
            ubyte r1, ubyte g1, ubyte b1, ubyte a1) {
        vello_scene_fill_linear_rect(_handle, x, y, w, h, x0, y0, x1, y1, r0, g0, b0, a0, r1, g1, b1, a1);
    }

    void fillRadialRect(double x, double y, double w, double h,
            double cx, double cy, float radius,
            ubyte r0, ubyte g0, ubyte b0, ubyte a0,
            ubyte r1, ubyte g1, ubyte b1, ubyte a1) {
        vello_scene_fill_radial_rect(_handle, x, y, w, h, cx, cy, radius, r0, g0, b0, a0, r1, g1, b1, a1);
    }

    void pushClipRect(double x, double y, double w, double h) {
        vello_scene_push_clip_rect(_handle, x, y, w, h);
    }

    void popLayer() {
        vello_scene_pop_layer(_handle);
    }

    void beginPath() { vello_scene_path_begin(_handle); }
    void moveTo(double x, double y) { vello_scene_path_move_to(_handle, x, y); }
    void lineTo(double x, double y) { vello_scene_path_line_to(_handle, x, y); }
    void quadTo(double c1x, double c1y, double x, double y) {
        vello_scene_path_quad_to(_handle, c1x, c1y, x, y);
    }
    void cubicTo(double c1x, double c1y, double c2x, double c2y, double x, double y) {
        vello_scene_path_cubic_to(_handle, c1x, c1y, c2x, c2y, x, y);
    }
    void closePath() { vello_scene_path_close(_handle); }
    void fillPath(ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_path_fill(_handle, r, g, b, a);
    }
    void strokePath(double width, ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_path_stroke(_handle, width, r, g, b, a);
    }

    /// RGBA8 unpremultiplied pixels, row-major.
    void drawImage(double x, double y, double dstW, double dstH, uint srcW, uint srcH, const(ubyte)[] pixels) {
        if (!pixels.length || srcW == 0 || srcH == 0)
            return;
        vello_scene_draw_image(_handle, x, y, dstW, dstH, srcW, srcH, pixels.ptr);
    }

    /// TrueType/OpenType outlines. `glyphs[i].x/y` are offsets from `(tx, ty)` (baseline origin).
    void drawGlyphs(const(ubyte)[] fontBytes, uint fontIndex, float fontSize,
            double tx, double ty, ubyte r, ubyte g, ubyte b, ubyte a, const(VelloGlyph)[] glyphs) {
        if (!fontBytes.length || !glyphs.length)
            return;
        vello_scene_draw_glyphs(_handle, fontBytes.ptr, fontBytes.length, fontIndex,
            fontSize, tx, ty, r, g, b, a, glyphs.ptr, glyphs.length);
    }

    @property VelloScene* handle() { return _handle; }
}

void initVello() {
    vello_init_logging();
}
