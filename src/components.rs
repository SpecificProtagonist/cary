use crate::{Vec2, Bounds};
use crate::textures::*;
use crate::renderer::{Rgb, Layer};

// This file will be split later



pub struct Pos {
    pub curr: Vec2,
    /// Position during the previous update,
    /// used to interpolate between the frames.
    /// Set to curr to avoid interpolation to this update.
    pub prev_interpol: Vec2
}

impl From<Vec2> for Pos {
    fn from(pos: Vec2) -> Self {
        Pos {
            curr: pos,
            prev_interpol: pos
        }
    }
}

pub struct Physics {
    /// Bounds relative to position
    pub bounds: Bounds,
    pub vel: Vec2,
    pub acc: Vec2,
    pub mass: f32,
    pub flying: bool
}

pub struct Child {
    pub parent: hecs::Entity,
    pub offset: Vec2
}

pub struct Health {
    pub current: u16,
    pub max: u16
}

impl From<u16> for Health {
    fn from(max: u16) -> Self {
        Health {
            current: max,
            max
        }
    }
}

pub struct Burning {
    pub duration: f32
}

pub struct Walking {
    pub max_speed: f32,
    pub acc: f32,
    pub jump_time_max: f32,
    pub jump_time: f32,
}


pub struct Controllable {
    pub horizontal: HControl,
    pub vertical: VControl,
    pub attack: bool,
    pub special_ability: bool
}

impl Default for Controllable {
    fn default() -> Self {
        Controllable {
            horizontal: HControl::None,
            vertical: VControl::None,
            attack: false,
            special_ability: false
        }
    }
}

pub enum HControl { None, Left, Right }
pub enum VControl { None, Up, Down }


pub struct Sprite {
    pub offset: Vec2,
    pub tex_anchor: TexAnchor,
    pub layer: Layer,
    pub tex: &'static [TexCoords],
    pub frame_duration: f32,
    pub repeat: bool,
    pub timer: f32
}

impl Sprite {
    pub fn single(coords: &'static [TexCoords], tex_anchor: TexAnchor, layer: Layer) -> Self {
        Sprite {
            offset: Vec2::zero(),
            tex_anchor,
            tex: coords,
            frame_duration: f32::INFINITY,
            repeat: false,
            timer: 0.0,
            layer
        }
    }

    pub fn ani(coords: &'static [TexCoords], tex_anchor: TexAnchor, layer: Layer, frame_duration: f32, repeat: bool) -> Self {
        Sprite {
            offset: Vec2::zero(),
            tex_anchor,
            tex: coords,
            frame_duration,
            repeat,
            timer: 0.0,
            layer
        }
    }
}


pub struct Light {
    pub offset: Vec2,
    pub tex: &'static TexCoords,
    pub size: Vec2,
    pub color: Rgb
}

impl Light {
    pub fn simple(size: f32, color: Rgb) -> Self {
        Light {
            offset: Vec2::zero(),
            tex: &DEFAULT_LIGHT[0],
            size: Vec2(size, size),
            color
        }
    }
}


pub struct BatComponent { 
    pub rush_time: f32,
    pub rush_cooldown: f32
}

pub type Bat = (Pos, Physics, Health, Controllable, BatComponent, Sprite);

pub fn make_bat(pos: Vec2) -> Bat {
    const SIZE: f32 = 0.65;
    const HEALTH: u16 = 2;
    (
        pos.into(),
        Physics {
            bounds: Bounds::around(Vec2::zero(), Vec2(SIZE, SIZE)),
            vel: Vec2::zero(),
            acc: Vec2::zero(),
            mass: 0.25,
            flying: true
        },
        HEALTH.into(),
        Controllable::default(),
        BatComponent {
            rush_time: 0.0,
            rush_cooldown: 0.0
        },
        Sprite::ani(crate::textures::BAT_FLY, TexAnchor::Center, Layer::Foreground, 0.08, true)
    )
}


pub struct HumanComponent {}

pub struct PlayerComponent {}
