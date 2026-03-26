import std.stdio;
import std.math;
import vello;
import glfw3.api;
import core.sys.windows.windows;

// Externs for GLFW native access
extern(C) HWND glfwGetWin32Window(GLFWwindow* window);

void main() {
    writeln("Initializing Vello Graphical Demo...");
    
    // 1. Init GLFW
    if (!glfwInit()) {
        writeln("Failed to init GLFW");
        return;
    }
    scope(exit) glfwTerminate();

    // 2. Create Window
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API); // Important: No OpenGL context
    auto window = glfwCreateWindow(800, 600, "Vello-D Graphical Demo", null, null);
    if (!window) {
        writeln("Failed to create window");
        return;
    }

    // 3. Get native handles (Windows specific)
    HWND hwnd = glfwGetWin32Window(window);
    HINSTANCE hinstance = GetModuleHandleA(null);
    
    // 4. Init Vello
    initVello();
    auto ctx = new Context(hwnd, hinstance, 800, 600);
    auto scene = new Scene();

    writeln("Running main loop...");
    while (!glfwWindowShouldClose(window)) {
        glfwPollEvents();

        // Check for resize
        int w, h;
        glfwGetFramebufferSize(window, &w, &h);
        ctx.resize(cast(uint)w, cast(uint)h);
        scene.reset();

        float time = cast(float)glfwGetTime();
        scene.clear(Color(
            0.1f + 0.1f * sin(time),
            0.1f + 0.1f * cos(time),
            0.2f,
            1.0f
        ));
        
        // Draw some shapes
        scene.fillRect(50, 50, 200, 200, 255, 0, 0, 255); // Red Rect
        scene.fillRect(300, 100, 150, 150, 0, 255, 0, 128); // Green Semi-Transparent Rect
        scene.fillRect(100, 300, 400, 50, 0, 0, 255, 255); // Blue Bar

        // Render to window
        ctx.render(scene);
    }
    
    writeln("Demo finished.");
}
