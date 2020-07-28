#![allow(unused)]
use super::Vec2;

pub const PIXELS_PER_TILE: f32 = 16.0;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub center: Vec2,
    pub size: Vec2
}


include!(concat!(env!("OUT_DIR"), "/uv-coords.rs"));