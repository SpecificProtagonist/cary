#![allow(unused)]

// Maybe reuse euler crate?
#[derive(Copy, Clone)]
pub struct Sprite {
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32
}

impl Sprite {
    pub fn left(&self) -> f32 {
        self.center_x - 0.5*self.width
    }

    pub fn right(&self) -> f32 {
        self.center_x + 0.5*self.width
    }

    pub fn bottom(&self) -> f32 {
        self.center_y - 0.5*self.height
    }

    pub fn top(&self) -> f32 {
        self.center_y + 0.5*self.height
    }
}

include!(concat!(env!("OUT_DIR"), "/uv-coords"));