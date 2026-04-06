mod algorithms;
mod app;
mod benchmark;
mod geometry;
mod input;
mod renderer;
mod ui;

fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut application = app::Application::default();
    event_loop
        .run_app(&mut application)
        .expect("Event loop failed");
}
