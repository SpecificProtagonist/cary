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

pub struct Collider {
    pub bounds: Bounds
}

pub struct Physics {
    // TODO: maybe smallvec?
    /// Bounds relative to position
    pub bounds: Bounds,
    pub vel: Vec2,
    pub acc: Vec2,
    pub mass: f32,
    pub gravity: bool
}

pub struct ChildOf {
    pub parent: hecs::Entity,
    pub offset: Vec2
}

pub struct Killable {}

pub struct Walking {
    pub max_speed: f32,
    pub acc: f32,
    pub jump_time_max: f32,
    pub jump_time: f32,
}

pub struct Flying {
    pub max_speed: f32,
    pub acc: f32,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HControl { None, Left, Right }
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VControl { None, Up, Down }


// TODO: maybe smallvec?
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

    pub fn timer(&self) -> f32 {
        if self.repeat {
            self.timer % (self.frame_duration * self.tex.len() as f32)
        } else {
            self.timer
        }
    }
}

pub struct Player {
    
}

pub fn make_player(pos: Vec2) -> (Player, Pos, Physics, Controllable, Killable, Sprite){
    const SIZE: f32 = 0.65;
    (
        Player {},
        pos.into(),
        Physics {
            bounds: Bounds::around(Vec2::zero(), Vec2(SIZE, SIZE)),
            vel: Vec2::zero(),
            acc: Vec2::zero(),
            mass: 0.25,
            gravity: true
        },
        Controllable::default(),
        Killable {},
        Sprite::ani(crate::textures::PLAYER_FLY, TexAnchor::Center, Layer::Foreground, 0.08, true),
    )
}