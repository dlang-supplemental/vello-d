import std.stdio;
import std.math;
import core.thread;
import core.time;
import vello;
import glfw3.api;
import core.sys.windows.windows;

extern(C) HWND glfwGetWin32Window(GLFWwindow* window);

/**
 * A dedicated thread that manages a single Vello context's render loop.
 * This ensures rendering continues even if the main thread is blocked by 
 * Win32 window move/resize modal loops.
 */
class RenderThread : Thread {
    GLFWwindow* _window;
    HWND _hwnd;
    HINSTANCE _hinstance;
    VelloBackend _backend;
    Context ctx;
    Scene scene;
    Color bgColor;
    ubyte rectR, rectG, rectB;
    bool useSin;
    shared bool running = true;
    shared bool ready = false;

    this(
        GLFWwindow* window, HWND hwnd, HINSTANCE hinstance, 
        VelloBackend backend, Color bgColor, ubyte r, ubyte g, ubyte b, bool useSin
    ) {
        this._window = window;
        this._hwnd = hwnd;
        this._hinstance = hinstance;
        this._backend = backend;
        this.bgColor = bgColor;
        this.rectR = r;
        this.rectG = g;
        this.rectB = b;
        this.useSin = useSin;
        super(&run);
    }

    void run() {
        // Initialize context here in the background thread
        try {
            this.ctx = new Context(_hwnd, _hinstance, 600, 400, _backend);
            this.scene = new Scene();
            this.ready = true; // Signal main thread that we are ready
        } catch (Exception e) {
            writeln("Failed to init backend: ", e.msg);
        }

        while (running) {
            static double time = 0;
            time += 0.016; 

            scene.reset();
            scene.clear(bgColor);
            
            double offset = useSin ? sin(time) : cos(time);
            scene.fillRect(50 + offset * 50, 50, 100, 100, rectR, rectG, rectB, 255);
            
            ctx.render(scene);
        }
    }
}

void main() {
    writeln("Initializing Threaded Multi-Backend Vello Demo...");
    writeln("Each window has its own render thread. Animation should stay smooth during move.");
    
    if (!glfwInit()) {
        writeln("Failed to init GLFW");
        return;
    }
    scope(exit) glfwTerminate();

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_VISIBLE, GLFW_TRUE); // Show immediately
    
    // Window 1: Vulkan
    auto window1 = glfwCreateWindow(600, 400, "Vello - VULKAN", null, null);
    if (!window1) return;
    
    // Window 2: DX12
    auto window2 = glfwCreateWindow(600, 400, "Vello - DX12", null, null);
    if (!window2) return;

    HWND hwnd1 = glfwGetWin32Window(window1);
    HWND hwnd2 = glfwGetWin32Window(window2);
    HINSTANCE hinstance = GetModuleHandleA(null);
    
    initVello();
    
    auto t1 = new RenderThread(
        window1, hwnd1, hinstance, VelloBackend.Vulkan, 
        Color(0.1f, 0.1f, 0.2f, 1.0f), 255, 0, 0, true
    );
    auto t2 = new RenderThread(
        window2, hwnd2, hinstance, VelloBackend.Dx12, 
        Color(0.2f, 0.1f, 0.1f, 1.0f), 0, 255, 0, false
    );

    t1.start();
    t2.start();

    while (!glfwWindowShouldClose(window1) && !glfwWindowShouldClose(window2)) {
        glfwPollEvents(); 

        // Draw GDI splash for DX12 while it warms up (Smooth Double-Buffered)
        if (!t2.ready) {
            static double splashStartTime = 0;
            if (splashStartTime == 0) splashStartTime = glfwGetTime();
            
            double currentTime = glfwGetTime();
            double elapsed = currentTime - splashStartTime;

            auto hdc = GetDC(hwnd2);
            RECT rect;
            GetClientRect(hwnd2, &rect);

            // Double buffering to prevent flicker at high frame rates
            auto hdcMem = CreateCompatibleDC(hdc);
            auto hbmMem = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
            auto hOld = SelectObject(hdcMem, hbmMem);

            // Paint black background to back buffer
            HBRUSH blackBrush = CreateSolidBrush(RGB(0, 0, 0));
            FillRect(hdcMem, &rect, blackBrush);
            DeleteObject(blackBrush);

            // Draw text
            SetTextColor(hdcMem, RGB(220, 220, 255));
            SetBkMode(hdcMem, TRANSPARENT);
            DrawTextA(hdcMem, "Loading DX12 Backend... (Compiling Shaders)", -1, &rect, 
                DT_CENTER | DT_VCENTER | DT_SINGLELINE);

            // Draw progress bar
            int barW = rect.right / 2;
            int barH = 6;
            int barX = rect.right / 4;
            int barY = rect.bottom / 2 + 30;
            
            RECT barRect = { barX, barY, barX + barW, barY + barH };
            HBRUSH grayBrush = CreateSolidBrush(RGB(40, 40, 40));
            FillRect(hdcMem, &barRect, grayBrush);
            DeleteObject(grayBrush);

            float progress = cast(float)elapsed / 10.0f; // Fake 10s load
            if (progress > 0.95f) progress = 0.95f; // Slow down at the end
            RECT fillRect = { barX, barY, cast(int)(barX + barW * progress), barY + barH };
            HBRUSH highlightBrush = CreateSolidBrush(RGB(50, 150, 255));
            FillRect(hdcMem, &fillRect, highlightBrush);
            DeleteObject(highlightBrush);

            // Flip back buffer to screen
            BitBlt(hdc, 0, 0, rect.right, rect.bottom, hdcMem, 0, 0, SRCCOPY);

            SelectObject(hdcMem, hOld);
            DeleteObject(hbmMem);
            DeleteDC(hdcMem);
            ReleaseDC(hwnd2, hdc);
        }
    }

    t1.running = false;
    t2.running = false;
    t1.join();
    t2.join();
}
