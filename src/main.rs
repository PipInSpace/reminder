use std::{thread, time::Duration};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowId},
};

fn resize(config: &mut wgpu::SurfaceConfiguration, surface: &mut wgpu::Surface, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) {
    config.width = size.width;
    config.height = size.height;
    surface.configure(device, &config);
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    // COLOR IS IN LINEAR SRGB COLOR SPACE
    let background_color = wgpu::Color {
        r: 0.01096,
        g: 0.00913,
        b: 0.007,
        a: 0.98
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

    event_loop
        .run(move |event, target| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (&instance, &adapter);

            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(new_size) => {
                        // Recreate the swap chain with the new size
                        resize(&mut config, &mut surface, &device, new_size);
                        // On macos the window needs to be redrawn manually after resizing
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                            let frame = surface.get_current_texture().expect("Failed to acquire next swap chain texture");
                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            let mut encoder =
                                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: None,
                                });
                            {
                                let _rpass =
                                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: None,
                                        color_attachments: &[Some(
                                            wgpu::RenderPassColorAttachment {
                                                view: &view,
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(
                                                        background_color,
                                                    ),
                                                    store: wgpu::StoreOp::Store,
                                                },
                                            },
                                        )],
                                        depth_stencil_attachment: None,
                                        timestamp_writes: None,
                                        occlusion_query_set: None,
                                    });
                            }

                            queue.submit(Some(encoder.finish()));
                            frame.present();
                    }
                    WindowEvent::CloseRequested => {
                        target.exit()
                    }
                    WindowEvent::MouseInput { device_id: _, state: _, button: _ } => {
                        target.exit()
                    }
                    _ => {}
                }
            }
        })
        .unwrap();
}


fn main() {
    println!("Hello, world!");
    const WINDOW_WIDTH: u32 = 400;
    const WINDOW_HEIGHT: u32 = 260;
    const WINDOW_PADDING: u32 = 32;
    const WINDOW_TITLEBAR: u32 = 32;

    let event_loop = EventLoop::new().unwrap();

    let window = winit::window::WindowBuilder::new()
        .with_blur(true)
        .with_transparent(true)
        .with_title("Reminder")
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();
    window.set_outer_position(winit::dpi::PhysicalPosition::new(
        WINDOW_PADDING,
        WINDOW_PADDING + WINDOW_TITLEBAR,
    ));
    window.set_decorations(false);

    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
