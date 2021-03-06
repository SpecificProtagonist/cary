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

pub struct RemoveOnImpact {}

pub struct Shooter {
    pub cooldown: f32
}

// Not neccessary with way the game turned out
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
    pub rotation: u8,
    pub frame_duration: f32,
    pub repeat: bool,
    pub timer: f32,
    pub running: bool
}

impl Sprite {
    pub fn single(coords: &'static [TexCoords], tex_anchor: TexAnchor, layer: Layer, rotation: u8) -> Self {
        Sprite {
            offset: Vec2::zero(),
            tex_anchor,
            tex: coords,
            mirror: false,
            rotation,
            frame_duration: f32::INFINITY,
            repeat: false,
            timer: 0.0,
            layer,
            running: false
        }
    }

    pub fn ani(coords: &'static [TexCoords], tex_anchor: TexAnchor, layer: Layer, frame_duration: f32, repeat: bool, rotation: u8) -> Self {
        Sprite {
            offset: Vec2::zero(),
            tex_anchor,
            tex: coords,
            mirror: false,
            rotation,
            frame_duration,
            repeat,
            timer: 0.0,
            layer,
            running: true
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
            rotation: 0,
            frame_duration: f32::INFINITY,
            repeat: false,
            timer: 0.0,
            layer: Layer::Background,
            running: false
        }
    )
}

// Integer coordinates refer to the bottom center of the tile. This was a mistake.
pub fn make_tile_solid(x: i32, y: i32) -> (Pos, Sprite, Collider) {
    (
        Vec2(x as f32, y as f32).into(),
        Sprite::single(TILE_SOLID, TexAnchor::Bottom, Layer::ForegroundTile, 0),
        Collider {
            bounds: Bounds::around(Vec2(0.0, 0.5), Vec2(1.0, 1.0))
        }
    )
}

pub fn make_tile_movable(x: i32, y: i32) -> (Pos, Sprite, Collider, Physics, Carryable) {
    let bounds = Bounds::around(Vec2(0.0, 0.5), Vec2(0.95, 0.95));
    (
        Vec2(x as f32, y as f32).into(),
        Sprite::single(TILE_MOVEABLE, TexAnchor::Bottom, Layer::ForegroundTile, 1),
        Collider {
            bounds
        },
        Physics {
            bounds,
            vel: Vec2::zero(),
            gravity: true,
            collided: (Horizontal::None, Vertical::None)
        },
        Carryable {
            detect_bounds: Bounds::around(Vec2(0.0, 1.4), Vec2(1.2, 0.8)),
            carry_offset: Vec2(0.0, -(bounds.size().1 + 0.55/2.0)),
            carried: false
        },
    )
}

pub fn make_spikes(x: i32, y: i32, rotation: u8) -> (Pos, Hazzard, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Hazzard {
            bounds: Bounds::around(
                Vec2(0.0, 0.5) + Vec2(0.0, -0.1).rotated(rotation), 
                if rotation % 2 == 0 {Vec2(1.0, 0.8)} else {Vec2(0.8, 1.0)})
        },
        Sprite::ani(SPIKES, TexAnchor::Bottom, Layer::Foreground, 0.27, true, rotation),
    )
}

pub fn make_divider(x: i32, y: i32, vertical: bool) -> (Pos, Hazzard, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Hazzard {
            bounds: Bounds::around(Vec2(0.0, 0.5), 
            if vertical {Vec2(0.28, 1.0)} else { Vec2(1.0, 0.375) })
        },
        Sprite::ani(DIVIDER, TexAnchor::Bottom, Layer::Foreground, 
            1.0/25.0, true,if vertical {0} else {1}),
    )
}

pub fn make_trap(x: i32, y: i32, rotation: u8) -> (Pos, Hazzard, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Hazzard {
            bounds: Bounds::around(
                Vec2(0.0, 0.5) + Vec2(0.0, 0.25).rotated(rotation), 
                if rotation % 2 == 0 {Vec2(0.8, 0.4)} else {Vec2(0.4, 0.8)})
        },
        Sprite::ani(TRAP_CEIL, TexAnchor::Bottom, Layer::Foreground, 0.13, true, rotation),
    )
}

pub fn make_bullet(pos: Vec2, target: Vec2) -> (Pos, Sprite, Physics, Hazzard, RemoveOnImpact) {
    (
        pos.into(),
        Sprite::ani(BULLET, TexAnchor::Center, Layer::ForegroundTile, 0.3, true, 0),
        Physics {
            bounds: Bounds::around(Vec2::zero(), Vec2(0.6, 0.6)),
            vel: (target-pos).norm() * 3.0,
            gravity: false,
            collided: (Horizontal::None, Vertical::None)
        },
        Hazzard {
            bounds: Bounds::around(Vec2::zero(), Vec2(0.4, 0.4))
        },
        RemoveOnImpact {}
    )
}

pub fn make_shooter(x: i32, y: i32) -> (Pos, Sprite, Collider, Shooter) {
    (
        Vec2(x as f32, y as f32).into(),
        Sprite {
            offset: Vec2(0.0, 0.5),
            tex: SHOOTER,
            tex_anchor: TexAnchor::Center,
            layer: Layer::ForegroundTile,
            mirror: false,
            rotation: 0,
            frame_duration: f32::INFINITY,
            timer: 0.0,
            repeat: false,
            running: false
        },
        Collider {
            bounds: Bounds::around(Vec2(0.0, 0.5), Vec2(1.0, 1.0))
        },
        Shooter {
            cooldown: 2.0
        }
    )
}

pub struct Player {
    pub flap_cooldown: f32,
    pub carrying: Option<Entity>,
    pub stamina: f32 // max: 1.0
}

pub fn make_player(pos: Vec2) -> (Player, Pos, Physics, Controllable, Children, Killable, Sprite){
    let bounds = Bounds::around(Vec2::zero(), Vec2(0.55, 0.55));
    (
        Player { 
            flap_cooldown: 0.0,
            carrying: None,
            stamina: 1.0
        },
        pos.into(),
        Physics {
            bounds,
            vel: Vec2::zero(),
            gravity: true,
            collided: (Horizontal::None, Vertical::None)
        },
        Controllable::default(),
        Children(Vec::new()),
        Killable {
            bounds,
            loss_on_death: true
        },
        Sprite::ani(PLAYER_FLY, TexAnchor::Center, Layer::Foreground, 0.08, false, 0),
    )
}

pub struct Cary {
    pub walk_right: bool
}

pub fn make_cary(pos: Vec2) -> (Cary, Pos, Physics, Killable, Carryable, Sprite) { 
    (
        Cary {
            walk_right: true
        },
        pos.into(),
        Physics {
            bounds: Bounds::around(Vec2(0.0, 0.6), Vec2(0.7, 1.2)),
            vel: Vec2::zero(),
            gravity: true,
            collided: (Horizontal::None, Vertical::None)
        },
        Killable {
            bounds: Bounds::around(Vec2(0.0, 0.6), Vec2(0.3, 1.2)),
            loss_on_death: true
        },
        Carryable {
            detect_bounds: Bounds::around(Vec2(0.0, 1.5), Vec2(1.2, 1.0)),
            carry_offset: Vec2(0.0, -1.30),
            carried: false
        },
        Sprite::ani(CARY_WALK, TexAnchor::Bottom, Layer::ForegroundPlayer, 0.2, true, 0)
    )
}

pub struct Exit(pub Bounds);

pub fn make_exit(x: i32, y: i32) -> (Pos, Exit, Sprite) {
    (
        Vec2(x as f32, y as f32).into(),
        Exit( Bounds::around(Vec2(0.0, 0.6), Vec2(0.3, 1.6))),
        Sprite::single(EXIT, TexAnchor::Bottom, Layer::Foreground, 0)
    )
}