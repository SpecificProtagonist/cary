mod renderer;


use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};



async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = renderer::Renderer::create(&window).await;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => renderer.render(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } 
                => renderer.resize(size.width, size.height),
            Event::WindowEvent { event: WindowEvent::CloseRequested, ..}
                => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}



fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    futures::executor::block_on(run(event_loop, window));
}
