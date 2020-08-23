use crate::Vec2;

// Eventually I'll remove the WebGL backend. I yearn for that day.
#[cfg(target_arch="wasm32")]
mod backend_webgl;
#[cfg(target_arch="wasm32")]
pub use backend_webgl::Renderer;

#[cfg(not(target_arch="wasm32"))]
mod backend_wgpu;
#[cfg(not(target_arch="wasm32"))]
pub use backend_wgpu::Renderer;



#[derive(Copy, Clone, Debug)]
pub enum Layer {
    ForegroundTile = 3,
    ForegroundPlayer = 4,
    ForegroundPickupHint = 5,
    Foreground = 7,
    Background = 8,
    BackgroundTile = 9,
}

impl From<Layer> for f32 {
    fn from(layer: Layer) -> Self {
        layer as i32 as f32 / 10.0
    }
}

