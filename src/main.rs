use std::{thread, time::Duration};

use winit::event_loop::EventLoop;

mod alert;

pub enum Theme {
    Dark,
    Light,
}

fn main() {
    println!("Hello, world!");
    let event_loop = EventLoop::new().unwrap();
    
    loop {
        alert::create_alert("The first method is the simplest, and will give you default values for everything.", Theme::Dark);
    }
}
