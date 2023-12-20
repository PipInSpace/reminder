#![windows_subsystem = "windows"]
use std::time::Instant;
use std::{collections::HashMap, thread, time::Duration};
use wgpu_text::glyph_brush::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::*;
use winit::event_loop::EventLoopBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowId,
};

use crate::{parser::read_from_file, alert::Theme};
use alert::AlertWindow;

mod alert;
mod parser;



pub struct Reminder {
    string: String, // Reminder message
    repeating: bool, // Reminder is repeating
    triggered: bool, // Reminder has been triggered/disabled
    every: u32, // Reminder Intervall in seconds
    duration_until: Duration,
}

//#[derive(Debug, Clone)]
pub enum CustomEvents {
    CreateAlert(String, Theme),
}

pub async fn run(event_loop: EventLoop<CustomEvents>) {
    let mut alert_windows: HashMap<WindowId, AlertWindow> = HashMap::new();
    let instance = wgpu::Instance::default();
    //let window = alert::create_alert(string, theme, &event_loop);
    //windows.insert(window.id(), window);

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            // Request an adapter which can render to our surface (Using Defaults is bad practice but oh well)
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
    let moderat_black: &[u8] = include_bytes!("../fonts/Moderat-Black.ttf");
    let moderat_medium: &[u8] = include_bytes!("../fonts/Moderat-Medium.ttf");

    //window.set_visible(true);
    event_loop
        .run(move |event, target| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (&instance, &adapter);

            match event {
                Event::UserEvent(user_event) => match user_event {
                    CustomEvents::CreateAlert(string, theme) => {
                        let alert_window = alert::create_alert(
                            string, theme, target, &adapter, &device, &instance,
                        );
                        alert_windows.insert(alert_window.window.id(), alert_window);
                    }
                },
                Event::WindowEvent { window_id, event } => {
                    match event {
                        WindowEvent::Resized(new_size) => {
                            let alert_window = alert_windows
                                .get_mut(&window_id)
                                .expect("Could not retrieve alert_window");
                            alert::resize(
                                &mut alert_window.config,
                                &mut alert_window.surface,
                                &device,
                                new_size,
                            );
                            // On macos the window needs to be redrawn manually after resizing
                            alert_window.window.request_redraw();

                            //section.bounds = (config.width as f32 * 0.4, config.height as _);
                            //section.screen_position.1 = config.height as f32 * 0.5;

                            //brush.resize_view(config.width as f32, config.height as f32, &queue);
                        }
                        WindowEvent::RedrawRequested => {
                            let alert_window = alert_windows
                                .get(&window_id)
                                .expect("Could not retrieve alert_window");
                            let config = &alert_window.config;

                            // Setup text brushes for all typefaces
                            let mut brush_m_black = BrushBuilder::using_font_bytes(moderat_black)
                                .unwrap()
                                .build(&device, config.width, config.height, config.format);
                            let mut brush_m_medium = BrushBuilder::using_font_bytes(moderat_medium)
                                .unwrap()
                                .build(&device, config.width, config.height, config.format);

                            // Setup text sections
                            let font_size = 40.;
                            let text_title = Section::default()
                                .add_text(
                                    Text::new("Reminder")
                                        .with_scale(font_size)
                                        .with_color(alert_window.font_color),
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
                                    Text::new(&alert_window.reminder_string)
                                        .with_scale(font_size)
                                        .with_color(alert_window.font_color),
                                )
                                .with_bounds((config.width as f32 * 0.9, config.height as f32))
                                .with_layout(
                                    Layout::default()
                                        .line_breaker(BuiltInLineBreaker::UnicodeLineBreaker),
                                )
                                .with_screen_position((15.0, 55.0))
                                .to_owned();

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

                            let frame = alert_window
                                .surface
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
                                let mut rpass =
                                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: None,
                                        color_attachments: &[Some(
                                            wgpu::RenderPassColorAttachment {
                                                view: &view,
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(
                                                        alert_window.background_color,
                                                    ),
                                                    store: wgpu::StoreOp::Store,
                                                },
                                            },
                                        )],
                                        depth_stencil_attachment: None,
                                        timestamp_writes: None,
                                        occlusion_query_set: None,
                                    });

                                // Draw text
                                brush_m_black.draw(&mut rpass);
                                brush_m_medium.draw(&mut rpass);
                            }

                            queue.submit(Some(encoder.finish()));
                            frame.present();
                        }
                        WindowEvent::CloseRequested => {
                            alert_windows.remove(&window_id);
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state: _,
                            button: _,
                        } => {
                            alert_windows.remove(&window_id);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

#[rustfmt::skip]
fn main() {
    println!("Reminder v0.1");

    let mut reminders = read_from_file();
    let now = Instant::now();

    let event_loop = EventLoopBuilder::<CustomEvents>::with_user_event().build().unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        loop {
            for reminder in &mut reminders {
                if now.elapsed() > reminder.duration_until && !reminder.triggered {
                    event_loop_proxy.send_event(CustomEvents::CreateAlert(reminder.string.clone(),Theme::adaptive(),)).ok();
                    reminder.duration_until += Duration::from_secs(reminder.every.into());
                    if !reminder.repeating {
                        // make sure one-time reminders are disabled
                        reminder.triggered = true
                    };
                }
            }
            thread::sleep(Duration::from_secs(5))
        }
    });

    pollster::block_on(run(event_loop))
}
