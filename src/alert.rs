use winit::{
    event_loop::EventLoop,
    window::Window,
};
use crate::Theme;

pub struct AlertWindow {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub background_color: wgpu::Color,
    pub font_color: [f32; 4],
    pub reminder_string: String,
}

pub fn resize(
    config: &mut wgpu::SurfaceConfiguration,
    surface: &mut wgpu::Surface,
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) {
    config.width = size.width;
    config.height = size.height;
    surface.configure(device, &config);
}

pub fn create_alert(
    string: String,
    theme: Theme,
    event_loop: &winit::event_loop::EventLoopWindowTarget<crate::CustomEvents>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    instance: &wgpu::Instance,
) -> AlertWindow {
    const WINDOW_WIDTH: u32 = 400;
    const WINDOW_HEIGHT: u32 = 260;
    const WINDOW_PADDING: u32 = 24;
    const TASKBAR: u32 = 40;

    let (background_color, font_color) = match theme {
        Theme::Dark => (
            wgpu::Color {
                r: 0.01298,
                g: 0.01298,
                b: 0.01298,
                a: 0.98,
            },
            [0.87137, 0.87137, 0.87137, 1.0],
        ),
        Theme::Light => (
            wgpu::Color {
                r: 0.87137,
                g: 0.87137,
                b: 0.87137,
                a: 0.95,
            },
            [0.01298, 0.01298, 0.01298, 1.0],
        ),
    };

    let window = winit::window::WindowBuilder::new()
        .with_blur(true)
        .with_transparent(true)
        .with_title(format!("Reminder: {}", string))
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();
    window.set_outer_position(winit::dpi::PhysicalPosition::new(
        WINDOW_PADDING,
        WINDOW_PADDING + TASKBAR,
    ));
    window.set_decorations(false);

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let size = window.inner_size();

    let caps = surface.get_capabilities(adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: caps.formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(device, &config);

    AlertWindow {
        window,
        surface,
        config,
        background_color,
        font_color,
        reminder_string: string,
    }
    //run(event_loop, window, theme);
    //pollster::block_on(run(event_loop, window, string, theme));
}
