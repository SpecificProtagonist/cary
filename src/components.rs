use hecs::Entity;
use crate::{Vec2, Bounds};
use crate::textures::*;
use crate::renderer::Layer;

// This file will be split later


// TODO: merge bounds into pos
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
    pub mass: f32,
    pub gravity: bool,
    pub collided: (Horizontal, Vertical)
}

// Carefull: Allways use both ChildOf and Children 
pub struct ChildOf {
    pub parent: Entity,
    pub offset: Vec2,
    pub collision: Bounds // Neccessary because we can't borrow physics
}
pub struct Children(pub Vec<Entity>);

pub struct Killable {
    pub bounds: Bounds,
    pub loss_on_death: bool
}

pub struct Hazzard {
    pub bounds: Bounds
}

pub struct Carryable {
    pub detect_bounds: Bounds,
    pub carry_offset: Vec2,
    pub carried: bool
}

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
    pub horizontal: Horizontal,
    pub vertical: Vertical,
    pub pick_up: bool,
}

impl Default for Controllable {
    fn default() -> Self {
        Controllable {
            horizontal: Horizontal::None,
            vertical: Vertical::None,
            pick_up: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Horizontal { None, Left, Right }
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Vertical { None, Up, Down }


// TODO: maybe smallvec?
pub struct Sprite {
    pub offset: Vec2,
    pub tex_anchor: TexAnchor,
    pub layer: Layer,
    pub tex: &'static [TexCoords],
    pub mirror: bool,
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
            mirror: false,
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
            mirror: false,
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

    pub fn finished(&self) -> bool {
        if self.repeat {
            false
        } else {
            self.timer >= self.frame_duration * self.tex.len() as f32
        }
    }
}

pub fn make_tile_background(x: i32, y: i32) -> (Pos, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Sprite {
            offset: Vec2(0.0, -1.0),
            tex_anchor: TexAnchor::Bottom,
            tex: TILE_FREE,
            mirror: false,
            frame_duration: f32::INFINITY,
            repeat: false,
            timer: 0.0,
            layer: Layer::Background
        }
    )
}

pub fn make_tile_solid(x: i32, y: i32) -> (Pos, Sprite, Collider) {
    (
        Vec2(x as f32, y as f32).into(),
        Sprite::single(TILE_SOLID, TexAnchor::Bottom, Layer::ForegroundTile),
        Collider {
            bounds: Bounds::around(Vec2(0.0, 0.5), Vec2(1.0, 1.0))
        }
    )
}

pub fn make_spikes(x: i32, y: i32) -> (Pos, Hazzard, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Hazzard {
            bounds: Bounds::around(Vec2(0.0, 0.4), Vec2(1.0, 0.8))
        },
        Sprite::single(SPIKES, TexAnchor::Bottom, Layer::Foreground),
    )
}

pub fn make_trap_ceiling(x: i32, y: i32) -> (Pos, Hazzard, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Hazzard {
            bounds: Bounds::around(Vec2(0.0, 0.7), Vec2(1.0, 0.6))
        },
        Sprite::ani(TRAP_CEIL, TexAnchor::Bottom, Layer::Foreground, 0.13, true),
    )
}

pub struct Player {
    pub flap_cooldown: f32,
    pub carrying: Option<Entity>
}

pub fn make_player(pos: Vec2) -> (Player, Pos, Physics, Controllable, Children, Killable, Sprite){
    const SIZE: f32 = 0.65;
    let bounds = Bounds::around(Vec2::zero(), Vec2(SIZE, SIZE));
    (
        Player { 
            flap_cooldown: 0.0,
            carrying: None
        },
        pos.into(),
        Physics {
            bounds,
            vel: Vec2::zero(),
            mass: 0.25,
            gravity: true,
            collided: (Horizontal::None, Vertical::None)
        },
        Controllable::default(),
        Children(Vec::new()),
        Killable {
            bounds,
            loss_on_death: true
        },
        Sprite::ani(PLAYER_FLY, TexAnchor::Center, Layer::Foreground, 0.08, false),
    )
}

pub struct Cary {
    pub walk_right: bool
}

pub fn make_cary(pos: Vec2) -> (Cary, Pos, Physics, Killable, Carryable, Sprite) {
    let bounds = Bounds::around(Vec2(0.0, 0.6), Vec2(0.6, 1.2));
    (
        Cary {
            walk_right: true
        },
        pos.into(),
        Physics {
            bounds,
            vel: Vec2::zero(),
            mass: 1.0,
            gravity: true,
            collided: (Horizontal::None, Vertical::None)
        },
        Killable {
            bounds,
            loss_on_death: true
        },
        Carryable {
            detect_bounds: Bounds::around(Vec2(0.0, 1.6), Vec2(1.2, 1.0)),
            carry_offset: Vec2(0.0, -1.3),
            carried: false
        },
        Sprite::ani(CARY_WALK, TexAnchor::Bottom, Layer::ForegroundPlayer, 0.2, true)
    )
}

pub struct Exit(pub Bounds);

pub fn make_exit(x: i32, y: i32) -> (Pos, Exit, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Exit( Bounds::around(Vec2(0.0, 0.7), Vec2(0.3, 1.4))),
        Sprite::single(EXIT, TexAnchor::Bottom, Layer::Foreground)
    )
}