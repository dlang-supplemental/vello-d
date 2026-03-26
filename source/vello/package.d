module vello;

public import vello.bindings;

/**
 * Simplified color structure for D-side management.
 */
struct Color {
    float r, g, b, a;
}

/**
 * High-level wrapper for a Vello rendering context.
 */
class Context {
    private VelloContext* _handle;

    /**
     * Create a context for a Windows window.
     */
    this(void* hwnd, void* hinstance, uint width, uint height, VelloBackend backend = VelloBackend.All) {
        _handle = vello_context_new_for_hwnd(hwnd, hinstance, width, height, backend);
    }

    ~this() {
        if (_handle) {
            vello_context_free(_handle);
        }
    }

    /**
     * Render a scene onto the window's surface.
     */
    void render(Scene scene) {
        vello_context_render(_handle, scene.handle);
    }

    /**
     * Update internal surfaces when the window is resized.
     */
    void resize(uint width, uint height) {
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
        }
    }

    /// Reset the scene to its initial empty state.
    void reset() {
        vello_scene_reset(_handle);
    }

    /// Set a background color for the upcoming frame.
    void clear(Color color) {
        fillRect(-1, -1, 1e9, 1e9, 
            cast(ubyte)(color.r * 255), cast(ubyte)(color.g * 255), 
            cast(ubyte)(color.b * 255), cast(ubyte)(color.a * 255));
    }

    /// Fill a rectangle with a solid color.
    void fillRect(double x, double y, double w, double h, ubyte r, ubyte g, ubyte b, ubyte a = 255) {
        vello_scene_fill_rect(_handle, x, y, w, h, r, g, b, a);
    }

    /// Access the raw underlying handle.
    @property VelloScene* handle() { return _handle; }
}

/**
 * Global initialization for the vello bridge.
 */
void initVello() {
    vello_init_logging();
}
