mod renderer;
mod textures;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

// Temporary
#[derive(Debug, Clone, Copy)]
pub struct Vec2(f32, f32);
impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0+rhs.0, self.1+rhs.1)
    }
}
impl std::ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0-rhs.0, self.1-rhs.1)
    }
}
impl std::ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0*rhs.0, self.1*rhs.1)
    }
}
impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2(self.0*rhs, self.1*rhs)
    }
}
impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2(self*rhs.0, self*rhs.1)
    }
}
impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2(self.0/rhs, self.1/rhs)
    }
}




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
