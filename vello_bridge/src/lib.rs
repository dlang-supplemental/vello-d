#[cfg(target_os = "windows")]
use core::num::NonZeroIsize;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
#[cfg(unix)]
use raw_window_handle::{
    WaylandDisplayHandle, WaylandWindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
#[cfg(target_os = "windows")]
use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
use std::os::raw::c_void;
use std::ptr;
#[cfg(unix)]
use std::ptr::NonNull;
use std::sync::Arc;
use vello::kurbo::{Affine, BezPath, Circle, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{
    Blob, Brush, Color, Fill, FontData, Gradient, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
};
use vello::wgpu::{
    Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Instance,
    InstanceDescriptor, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTargetUnsafe, Texture, TextureDescriptor, TextureFormat,
    TextureUsages,
};
use vello::{AaConfig, AaSupport, Glyph, RenderParams, Renderer, RendererOptions, Scene};

#[repr(C)]
pub enum VelloBackend {
    All = 0,
    Vulkan = 1,
    Dx12 = 2,
    Dx11 = 3,
    Metal = 4,
    Gl = 5,
    BrowserWebGpu = 6,
}

pub struct VelloContext {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub renderer: Renderer,
    pub config: SurfaceConfiguration,
    pub target_texture: Texture,
}

/// Scene plus path builder / current transform. Opaque to D as `VelloScene`.
pub struct VelloSceneHost {
    pub scene: Scene,
    pub path: BezPath,
    pub transform: Affine,
}

fn color_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

fn brush_solid(r: u8, g: u8, b: u8, a: u8) -> Brush {
    Brush::Solid(color_rgba(r, g, b, a))
}

fn backend_mask(backend: VelloBackend) -> Backends {
    match backend {
        VelloBackend::All => Backends::all(),
        VelloBackend::Vulkan => Backends::VULKAN,
        VelloBackend::Dx12 => Backends::DX12,
        VelloBackend::Dx11 => Backends::all(),
        VelloBackend::Metal => Backends::METAL,
        VelloBackend::Gl => Backends::GL,
        VelloBackend::BrowserWebGpu => Backends::BROWSER_WEBGPU,
    }
}

unsafe fn context_from_raw_handles(
    rwh: RawWindowHandle,
    rdh: RawDisplayHandle,
    width: u32,
    height: u32,
    backend: VelloBackend,
) -> *mut VelloContext {
    vello_init_logging();

    let instance = Instance::new(&InstanceDescriptor {
        backends: backend_mask(backend),
        ..Default::default()
    });

    let surface = match instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: rdh,
        raw_window_handle: rwh,
    }) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let adapter = match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })) {
        Ok(a) => a,
        Err(_) => return ptr::null_mut(),
    };

    let (device, queue) = match pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: Some("Vello Device"),
        ..Default::default()
    })) {
        Ok(pair) => pair,
        Err(_) => return ptr::null_mut(),
    };

    let caps = surface.get_capabilities(&adapter);
    if caps.formats.is_empty() || width == 0 || height == 0 {
        return ptr::null_mut();
    }
    let mut format = caps.formats[0];
    for f in &caps.formats {
        if *f == TextureFormat::Rgba8Unorm || *f == TextureFormat::Rgba8UnormSrgb {
            format = *f;
            break;
        }
    }

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
        format,
        width,
        height,
        present_mode: PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let target_texture = create_target(&device, width, height);
    let renderer = match Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::all(),
            num_init_threads: std::num::NonZeroUsize::new(1),
            ..Default::default()
        },
    ) {
        Ok(r) => r,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(VelloContext {
        instance,
        surface,
        device,
        queue,
        renderer,
        config,
        target_texture,
    }))
}

fn create_target(device: &Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Vello Target Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: vello::wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[no_mangle]
pub unsafe extern "C" fn vello_init_logging() {
    let _ = env_logger::try_init();
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_new_for_hwnd(
    hwnd: *mut c_void,
    hinstance: *mut c_void,
    width: u32,
    height: u32,
    backend: VelloBackend,
) -> *mut VelloContext {
    #[cfg(target_os = "windows")]
    {
        let Some(hwnd_nz) = NonZeroIsize::new(hwnd as isize) else {
            return ptr::null_mut();
        };
        let mut win_handle = Win32WindowHandle::new(hwnd_nz);
        win_handle.hinstance = NonZeroIsize::new(hinstance as isize);
        context_from_raw_handles(
            RawWindowHandle::Win32(win_handle),
            RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
            width,
            height,
            backend,
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (hwnd, hinstance, width, height, backend);
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_new_for_x11(
    display: *mut c_void,
    window: u64,
    screen: i32,
    width: u32,
    height: u32,
    backend: VelloBackend,
) -> *mut VelloContext {
    #[cfg(unix)]
    {
        let Some(display_nn) = NonNull::new(display) else {
            return ptr::null_mut();
        };
        let win_handle = XlibWindowHandle::new(window as std::os::raw::c_ulong);
        let display_handle = XlibDisplayHandle::new(Some(display_nn), screen);
        context_from_raw_handles(
            RawWindowHandle::Xlib(win_handle),
            RawDisplayHandle::Xlib(display_handle),
            width,
            height,
            backend,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (display, window, screen, width, height, backend);
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_new_for_wayland(
    display: *mut c_void,
    surface: *mut c_void,
    width: u32,
    height: u32,
    backend: VelloBackend,
) -> *mut VelloContext {
    #[cfg(unix)]
    {
        let Some(display_nn) = NonNull::new(display) else {
            return ptr::null_mut();
        };
        let Some(surface_nn) = NonNull::new(surface) else {
            return ptr::null_mut();
        };
        context_from_raw_handles(
            RawWindowHandle::Wayland(WaylandWindowHandle::new(surface_nn)),
            RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display_nn)),
            width,
            height,
            backend,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (display, surface, width, height, backend);
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_free(ctx: *mut VelloContext) {
    if !ctx.is_null() {
        let _ = Box::from_raw(ctx);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_render(ctx: *mut VelloContext, scene: *mut VelloSceneHost) {
    if let (Some(ctx), Some(host)) = (ctx.as_mut(), scene.as_ref()) {
        let Ok(surface_texture) = ctx.surface.get_current_texture() else {
            return;
        };

        let view = ctx
            .target_texture
            .create_view(&vello::wgpu::TextureViewDescriptor::default());

        if ctx
            .renderer
            .render_to_texture(
                &ctx.device,
                &ctx.queue,
                &host.scene,
                &view,
                &RenderParams {
                    base_color: Color::BLACK,
                    width: ctx.config.width,
                    height: ctx.config.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .is_err()
        {
            return;
        }

        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });

        encoder.copy_texture_to_texture(
            ctx.target_texture.as_image_copy(),
            surface_texture.texture.as_image_copy(),
            Extent3d {
                width: ctx.config.width,
                height: ctx.config.height,
                depth_or_array_layers: 1,
            },
        );

        ctx.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_resize(ctx: *mut VelloContext, width: u32, height: u32) {
    if let Some(ctx) = ctx.as_mut() {
        if width == 0 || height == 0 {
            return;
        }
        if ctx.config.width == width && ctx.config.height == height {
            return;
        }
        ctx.config.width = width;
        ctx.config.height = height;
        ctx.surface.configure(&ctx.device, &ctx.config);
        ctx.target_texture = create_target(&ctx.device, width, height);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_new() -> *mut VelloSceneHost {
    Box::into_raw(Box::new(VelloSceneHost {
        scene: Scene::new(),
        path: BezPath::new(),
        transform: Affine::IDENTITY,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_free(scene: *mut VelloSceneHost) {
    if !scene.is_null() {
        let _ = Box::from_raw(scene);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_reset(scene: *mut VelloSceneHost) {
    if let Some(host) = scene.as_mut() {
        host.scene.reset();
        host.path = BezPath::new();
        host.transform = Affine::IDENTITY;
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_set_transform(
    scene: *mut VelloSceneHost,
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
) {
    if let Some(host) = scene.as_mut() {
        host.transform = Affine::new([a, b, c, d, e, f]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_reset_transform(scene: *mut VelloSceneHost) {
    if let Some(host) = scene.as_mut() {
        host.transform = Affine::IDENTITY;
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        let rect = Rect::new(x, y, x + w, y + h);
        host.scene.fill(
            Fill::NonZero,
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &rect,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_rounded_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    radius: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        let shape = RoundedRect::new(x, y, x + w, y + h, radius.max(0.0));
        host.scene.fill(
            Fill::NonZero,
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &shape,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_stroke_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    width: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        if width <= 0.0 {
            return;
        }
        let rect = Rect::new(x, y, x + w, y + h);
        host.scene.stroke(
            &Stroke::new(width),
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &rect,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_circle(
    scene: *mut VelloSceneHost,
    cx: f64,
    cy: f64,
    radius: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        host.scene.fill(
            Fill::NonZero,
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &Circle::new((cx, cy), radius.max(0.0)),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_stroke_line(
    scene: *mut VelloSceneHost,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    width: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        if width <= 0.0 {
            return;
        }
        host.scene.stroke(
            &Stroke::new(width),
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &Line::new((x0, y0), (x1, y1)),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_linear_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    r0: u8,
    g0: u8,
    b0: u8,
    a0: u8,
    r1: u8,
    g1: u8,
    b1: u8,
    a1: u8,
) {
    if let Some(host) = scene.as_mut() {
        let rect = Rect::new(x, y, x + w, y + h);
        let grad = Gradient::new_linear((x0, y0), (x1, y1)).with_stops([
            color_rgba(r0, g0, b0, a0),
            color_rgba(r1, g1, b1, a1),
        ]);
        host.scene
            .fill(Fill::NonZero, host.transform, &grad, None, &rect);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_radial_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    cx: f64,
    cy: f64,
    radius: f32,
    r0: u8,
    g0: u8,
    b0: u8,
    a0: u8,
    r1: u8,
    g1: u8,
    b1: u8,
    a1: u8,
) {
    if let Some(host) = scene.as_mut() {
        let rect = Rect::new(x, y, x + w, y + h);
        let grad = Gradient::new_radial((cx, cy), radius.max(0.0)).with_stops([
            color_rgba(r0, g0, b0, a0),
            color_rgba(r1, g1, b1, a1),
        ]);
        host.scene
            .fill(Fill::NonZero, host.transform, &grad, None, &rect);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_push_clip_rect(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) {
    if let Some(host) = scene.as_mut() {
        let rect = Rect::new(x, y, x + w, y + h);
        host.scene
            .push_clip_layer(Fill::NonZero, host.transform, &rect);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_pop_layer(scene: *mut VelloSceneHost) {
    if let Some(host) = scene.as_mut() {
        host.scene.pop_layer();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_begin(scene: *mut VelloSceneHost) {
    if let Some(host) = scene.as_mut() {
        host.path = BezPath::new();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_move_to(scene: *mut VelloSceneHost, x: f64, y: f64) {
    if let Some(host) = scene.as_mut() {
        host.path.move_to((x, y));
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_line_to(scene: *mut VelloSceneHost, x: f64, y: f64) {
    if let Some(host) = scene.as_mut() {
        host.path.line_to((x, y));
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_quad_to(
    scene: *mut VelloSceneHost,
    c1x: f64,
    c1y: f64,
    x: f64,
    y: f64,
) {
    if let Some(host) = scene.as_mut() {
        host.path.quad_to((c1x, c1y), (x, y));
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_cubic_to(
    scene: *mut VelloSceneHost,
    c1x: f64,
    c1y: f64,
    c2x: f64,
    c2y: f64,
    x: f64,
    y: f64,
) {
    if let Some(host) = scene.as_mut() {
        host.path.curve_to((c1x, c1y), (c2x, c2y), (x, y));
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_close(scene: *mut VelloSceneHost) {
    if let Some(host) = scene.as_mut() {
        host.path.close_path();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_fill(
    scene: *mut VelloSceneHost,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        let path = host.path.clone();
        host.scene.fill(
            Fill::NonZero,
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &path,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_path_stroke(
    scene: *mut VelloSceneHost,
    width: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    if let Some(host) = scene.as_mut() {
        if width <= 0.0 {
            return;
        }
        let path = host.path.clone();
        host.scene.stroke(
            &Stroke::new(width),
            host.transform,
            &brush_solid(r, g, b, a),
            None,
            &path,
        );
    }
}

/// Draw an RGBA8 (unpremultiplied) image. `pixels` is width*height*4 bytes, row-major.
#[no_mangle]
pub unsafe extern "C" fn vello_scene_draw_image(
    scene: *mut VelloSceneHost,
    x: f64,
    y: f64,
    dst_w: f64,
    dst_h: f64,
    src_w: u32,
    src_h: u32,
    pixels: *const u8,
) {
    if scene.is_null() || pixels.is_null() || src_w == 0 || src_h == 0 {
        return;
    }
    let host = &mut *scene;
    let nbytes = (src_w as usize).saturating_mul(src_h as usize).saturating_mul(4);
    let slice = std::slice::from_raw_parts(pixels, nbytes);
    let data: Box<[u8]> = slice.to_vec().into_boxed_slice();
    let image = ImageData {
        data: Blob::new(Arc::new(data)),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: src_w,
        height: src_h,
    };
    let sx = if src_w == 0 { 1.0 } else { dst_w / src_w as f64 };
    let sy = if src_h == 0 { 1.0 } else { dst_h / src_h as f64 };
    let transform = host.transform * Affine::new([sx, 0.0, 0.0, sy, x, y]);
    let brush = ImageBrush::new(image);
    host.scene.draw_image(&brush, transform);
}

#[repr(C)]
pub struct VelloGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// Draw a glyph run from a TrueType/OpenType blob (`font_bytes`, `font_len`).
#[no_mangle]
pub unsafe extern "C" fn vello_scene_draw_glyphs(
    scene: *mut VelloSceneHost,
    font_bytes: *const u8,
    font_len: usize,
    font_index: u32,
    font_size: f32,
    tx: f64,
    ty: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    glyphs: *const VelloGlyph,
    glyph_count: usize,
) {
    if scene.is_null() || font_bytes.is_null() || font_len == 0 || glyphs.is_null() || glyph_count == 0
    {
        return;
    }
    let host = &mut *scene;
    let bytes = std::slice::from_raw_parts(font_bytes, font_len);
    let blob = Blob::new(Arc::new(bytes.to_vec()));
    let font = FontData::new(blob, font_index);
    let run = std::slice::from_raw_parts(glyphs, glyph_count);
    let iter = run.iter().map(|g| Glyph {
        id: g.id,
        x: g.x,
        y: g.y,
    });
    host.scene
        .draw_glyphs(&font)
        .font_size(font_size.max(1.0))
        .transform(host.transform * Affine::translate((tx, ty)))
        .brush(color_rgba(r, g, b, a))
        .draw(Fill::NonZero, iter);
}
