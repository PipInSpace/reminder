use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};
use wgpu_text::*;
use wgpu_text::glyph_brush::{
    BuiltInLineBreaker, Layout, Section, Text,
};

use crate::Theme;

fn resize(
    config: &mut wgpu::SurfaceConfiguration,
    surface: &mut wgpu::Surface,
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) {
    config.width = size.width;
    config.height = size.height;
    surface.configure(device, &config);
}

async fn run(event_loop: EventLoop<()>, window: Window, reminder_string: &str, theme: Theme) {
    // COLOR IS IN LINEAR SRGB COLOR SPACE
    let (background_color, font_color) = match theme {
        Theme::Dark => (wgpu::Color {
            r: 0.01298,
            g: 0.01298,
            b: 0.01298,
            a: 0.98,
        }, [0.87137, 0.87137, 0.87137, 1.0]),
        Theme::Light => (wgpu::Color {
            r: 0.87137,
            g: 0.87137,
            b: 0.87137,
            a: 0.95,
        }, [0.01298, 0.01298, 0.01298, 1.0]),
    };
    let instance = wgpu::Instance::default();
    let mut surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let size = window.inner_size();
    let caps = surface.get_capabilities(&adapter);
    let mut config: wgpu::SurfaceConfiguration = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: caps.formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Set up text rendering
    // All wgpu-text related below:
    let moderat_black: &[u8] = include_bytes!("../fonts/Moderat-Black.ttf");
    let moderat_medium: &[u8] = include_bytes!("../fonts/Moderat-Medium.ttf");
    let mut brush_m_black = BrushBuilder::using_font_bytes(moderat_black).unwrap().build(
        &device,
        config.width,
        config.height,
        config.format,
    );
    let mut brush_m_medium = BrushBuilder::using_font_bytes(moderat_medium).unwrap().build(
        &device,
        config.width,
        config.height,
        config.format,
    );
    let font_size = 40.;
    let text_title = Section::default()
        .add_text(
            Text::new("Reminder")
            .with_scale(font_size)
            .with_color(font_color),
        )
        .with_bounds((config.width as f32 * 0.9, config.height as f32))
        .with_layout(
            Layout::default()
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((15.0, 10.0))
        .to_owned();
    let font_size = 25.;
    let text_remider = Section::default()
        .add_text(
            Text::new(reminder_string)
            .with_scale(font_size)
            .with_color(font_color),
        )
        .with_bounds((config.width as f32 * 0.9, config.height as f32))
        .with_layout(
            Layout::default()
                .line_breaker(BuiltInLineBreaker::UnicodeLineBreaker),
        )
        .with_screen_position((15.0, 55.0))
        .to_owned();

    event_loop
        .run(move |event, target| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (&instance, &adapter);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(new_size) => {
                        // Recreate the swap chain with the new size
                        resize(&mut config, &mut surface, &device, new_size);
                        // On macos the window needs to be redrawn manually after resizing
                        window.request_redraw();

                        //section.bounds = (config.width as f32 * 0.4, config.height as _);
                        //section.screen_position.1 = config.height as f32 * 0.5;

                        //brush.resize_view(config.width as f32, config.height as f32, &queue);
                    }
                    WindowEvent::RedrawRequested => {
                        match brush_m_black.queue(&device, &queue, vec![&text_title]) {
                            Ok(_) => (),
                            Err(err) => {
                                panic!("{err}");
                            }
                        };
                        match brush_m_medium.queue(&device, &queue, vec![&text_remider]) {
                            Ok(_) => (),
                            Err(err) => {
                                panic!("{err}");
                            }
                        };

                        let frame = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        {
                            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(background_color),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            brush_m_black.draw(&mut rpass);
                            brush_m_medium.draw(&mut rpass);
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::MouseInput {
                        device_id: _,
                        state: _,
                        button: _,
                    } => target.exit(),
                    _ => {}
                }
            }
        })
        .unwrap();
}

pub fn create_alert(string: &str, theme: Theme) {
    const WINDOW_WIDTH: u32 = 400;
    const WINDOW_HEIGHT: u32 = 260;
    const WINDOW_PADDING: u32 = 24;
    const TASKBAR: u32 = 40;

    let event_loop = EventLoop::new().unwrap();

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

    env_logger::init();
    //run(event_loop, window, theme);
    pollster::block_on(run(event_loop, window, string, theme));
}
