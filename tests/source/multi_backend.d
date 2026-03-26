import std.stdio;
import std.math;
import vello;
import glfw3.api;
import core.sys.windows.windows;

extern(C) HWND glfwGetWin32Window(GLFWwindow* window);

void main() {
    writeln("Initializing Multi-Backend Vello Demo...");
    
    if (!glfwInit()) {
        writeln("Failed to init GLFW");
        return;
    }
    scope(exit) glfwTerminate();

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    
    // Window 1: Vulkan
    auto window1 = glfwCreateWindow(600, 400, "Vello - VULKAN Backend", null, null);
    if (!window1) return;
    
    // Window 2: DX12
    auto window2 = glfwCreateWindow(600, 400, "Vello - DX12 Backend", null, null);
    if (!window2) return;

    HWND hwnd1 = glfwGetWin32Window(window1);
    HWND hwnd2 = glfwGetWin32Window(window2);
    HINSTANCE hinstance = GetModuleHandleA(null);
    
    initVello();
    
    writeln("Creating VULKAN context...");
    glfwPollEvents(); // Keep windows responsive during init
    auto ctx1 = new Context(hwnd1, hinstance, 600, 400, VelloBackend.Vulkan);
    
    writeln("Creating DX12 context...");
    glfwPollEvents();
    auto ctx2 = new Context(hwnd2, hinstance, 600, 400, VelloBackend.Dx12);

    auto scene = new Scene();

    while (!glfwWindowShouldClose(window1) && !glfwWindowShouldClose(window2)) {
        glfwPollEvents();

        float time = cast(float)glfwGetTime();
        
        // Render to Vulkan window
        scene.reset();
        scene.clear(Color(0.1f, 0.1f, 0.2f, 1.0f));
        scene.fillRect(50 + sin(time) * 50, 50, 100, 100, 255, 0, 0, 255); // Red Moving Rect
        ctx1.render(scene);

        // Render to DX12 window
        scene.reset();
        scene.clear(Color(0.2f, 0.1f, 0.1f, 1.0f));
        scene.fillRect(50, 50 + cos(time) * 50, 100, 100, 0, 255, 0, 255); // Green Moving Rect
        ctx2.render(scene);
    }
}
