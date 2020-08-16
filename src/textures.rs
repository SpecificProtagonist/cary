#![allow(unused)]
use super::Vec2;

pub const PIXELS_PER_TILE: f32 = 16.0;

#[derive(Copy, Clone, Debug)]
pub struct TexCoords {
    pub center: Vec2,
    pub size: Vec2
}

#[derive(Copy, Clone, Debug)]
pub enum TexAnchor {
    Top,
    Center,
    Bottom
}

include!(concat!(env!("OUT_DIR"), "/uv-coords.rs"));