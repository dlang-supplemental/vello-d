use vello::wgpu::{
    Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    Instance, InstanceDescriptor, PowerPreference, PresentMode, Queue, RequestAdapterOptions,
    Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureFormat, TextureUsages,
};
use vello::kurbo::{Rect};
use vello::peniko::{Color, Brush};
use vello::{Scene, Renderer, RendererOptions};
use std::os::raw::{c_void};
use raw_window_handle::{
    RawWindowHandle, Win32WindowHandle, RawDisplayHandle, WindowsDisplayHandle,
};
use core::num::NonZeroIsize;

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
    vello_init_logging();

    let backend_mask = match backend {
        VelloBackend::All => Backends::all(),
        VelloBackend::Vulkan => Backends::VULKAN,
        VelloBackend::Dx12 => Backends::DX12,
        VelloBackend::Dx11 => Backends::all(), // Fallback
        VelloBackend::Metal => Backends::METAL,
        VelloBackend::Gl => Backends::GL,
        VelloBackend::BrowserWebGpu => Backends::BROWSER_WEBGPU,
    };

    let instance = Instance::new(&InstanceDescriptor {
        backends: backend_mask,
        ..Default::default()
    });

    let mut win_handle = Win32WindowHandle::new(
        NonZeroIsize::new(hwnd as isize).expect("Invalid HWND"),
    );
    win_handle.hinstance = NonZeroIsize::new(hinstance as isize);
    let rwh = RawWindowHandle::Win32(win_handle);
    let rdh = RawDisplayHandle::Windows(WindowsDisplayHandle::new());

    let surface = unsafe {
        instance.create_surface_unsafe(vello::wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: rdh,
            raw_window_handle: rwh,
        })
    }.expect("Failed to create surface");

    let adapter = pollster::block_on(instance.request_adapter(
        &RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    )).expect("Failed to request adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("Vello Device"),
            ..Default::default()
        },
    )).expect("Failed to request device");

    let caps = surface.get_capabilities(&adapter);
    let mut format = caps.formats[0];
    for f in &caps.formats {
        if *f == TextureFormat::Rgba8Unorm || *f == TextureFormat::Rgba8UnormSrgb {
            format = *f;
            break;
        }
    }

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST, // Surface needs COPY_DST now
        format,
        width,
        height,
        present_mode: PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Create the staging texture vello will render to
    let target_texture = device.create_texture(&TextureDescriptor {
        label: Some("Vello Target Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: vello::wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm, // Always use linear for storage
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: std::num::NonZeroUsize::new(1),
            ..Default::default()
        },
    ).expect("Failed to create renderer");

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

#[no_mangle]
pub unsafe extern "C" fn vello_context_free(ctx: *mut VelloContext) {
    if !ctx.is_null() {
        let _ = Box::from_raw(ctx);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_context_render(ctx: *mut VelloContext, scene: *mut Scene) {
    if let (Some(ctx), Some(scene)) = (ctx.as_mut(), scene.as_ref()) {
        let surface_texture = ctx.surface.get_current_texture()
            .expect("Failed to get current texture");
        
        // Render to staging texture
        let view = ctx.target_texture.create_view(&vello::wgpu::TextureViewDescriptor::default());

        ctx.renderer.render_to_texture(
             &ctx.device,
             &ctx.queue,
             scene,
             &view,
             &vello::RenderParams {
                base_color: Color::BLACK,
                width: ctx.config.width,
                height: ctx.config.height,
                antialiasing_method: vello::AaConfig::Area,
             },
        ).expect("Failed to render");

        // Copy staging texture to surface
        let mut encoder = ctx.device.create_command_encoder(&CommandEncoderDescriptor {
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
        if ctx.config.width == width && ctx.config.height == height {
            return;
        }
        ctx.config.width = width;
        ctx.config.height = height;
        ctx.surface.configure(&ctx.device, &ctx.config);
        
        // Recreate staging texture
        ctx.target_texture = ctx.device.create_texture(&TextureDescriptor {
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
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_new() -> *mut Scene {
    Box::into_raw(Box::new(Scene::new()))
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_free(scene: *mut Scene) {
    if !scene.is_null() {
        let _ = Box::from_raw(scene);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_reset(scene: *mut Scene) {
    if let Some(scene) = scene.as_mut() {
        scene.reset();
    }
}

#[no_mangle]
pub unsafe extern "C" fn vello_scene_fill_rect(
    scene: *mut Scene,
    x: f64, y: f64, w: f64, h: f64,
    r: u8, g: u8, b: u8, a: u8
) {
    if let Some(scene) = scene.as_mut() {
        let rect = Rect::new(x, y, x + w, y + h);
        let color = Color::from_rgba8(r, g, b, a);
        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &rect,
        );
    }
}
