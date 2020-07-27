#![allow(unused)]
use super::Vec2;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub center: Vec2,
    pub size: Vec2
}


include!(concat!(env!("OUT_DIR"), "/uv-coords.rs"));