mod textures;
mod math;
mod components;
mod renderer;
mod level;

use std::collections::HashMap;
use winit::{
    event::{Event, WindowEvent, VirtualKeyCode, },
    event_loop::{ControlFlow, EventLoop},
};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
use hecs::Entity;
use math::*;
use components::*;
use renderer::{Renderer, Layer};



const TIME_BETWEEN_UPDATES: f32 = 1.0 / 25.0;
const MIN_TIME_BETWEEN_FRAMES: f32 = 1.0 / 60.0;

const GAME_END_WAIT_TIME: f32 = 1.5;
const UI_CAMERA: Camera = Camera {
    pos: Vec2(0.0, 0.0),
    size: 7.0
};

enum GameState {
    ShowControls,
    WorldLoaded(World),
    Victory
}

#[cfg(target_arch="wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    main()
}

pub fn main() {

    let event_loop = EventLoop::new();
    // There will be only one window -> ignore window ids in events
    let window = winit::window::WindowBuilder::new()
        .with_title("Cary").with_window_icon(Some(get_icon())).build(&event_loop).unwrap();
    // Web: Add canvas to page
    #[cfg(target_arch="wasm32")] {
        use winit::platform::web::WindowExtWebSys;
        let web_window = web_sys::window().unwrap();
        let document = web_window.document().unwrap();
        let body = document.body().unwrap();
        body.append_child(&window.canvas()).unwrap();
    }
    let mut renderer = Renderer::create(&window);

    let mut level = 0;
    let mut game_state = GameState::ShowControls;

    // std::time's not available in wasm?
    // Also, maybe explicit requestAnimationFrame would be usefull on the web
    let mut last_update = Time::now();
    let mut last_frame = Time::now();

    event_loop.run(move |event, _, control_flow| {
        // Immediately restart loop; WaitUntil would suspend the thread
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } 
                => renderer.resize(size.width, size.height),
            Event::WindowEvent { event: WindowEvent::CloseRequested, ..}
                => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. }
                => {
                match &mut game_state {
                    GameState::WorldLoaded(world) => {
                        if (Some(VirtualKeyCode::J) == input.virtual_keycode) 
                        & (input.state == winit::event::ElementState::Pressed)
                        {
                            match world.state {
                                WorldState::Victory(_, time) if time > GAME_END_WAIT_TIME => {
                                    level += 1;
                                    if level < level::level_count() {
                                        *world = level::load(level)
                                    } else {
                                        game_state = GameState::Victory
                                    }
                                },
                                WorldState::Loss(_, time) if time > GAME_END_WAIT_TIME
                                => *world = level::load(level),
                                _ => world.input(input)
                            }
                        } else {
                            world.input(input)
                        }
                    },
                    GameState::ShowControls => {
                        if input.virtual_keycode.is_some() & (input.state == winit::event::ElementState::Pressed) {
                            game_state = GameState::WorldLoaded(level::load(level))
                        }
                    },
                    GameState::Victory => ()
                }
            },
            Event::MainEventsCleared 
                => {
                    if last_update.elapsed() >= TIME_BETWEEN_UPDATES {
                        last_update.add(TIME_BETWEEN_UPDATES);
                        if let GameState::WorldLoaded(world) = &mut game_state {
                            world.update();
                        }
                    }
                    let since_last_frame = last_frame.elapsed();
                    if since_last_frame >= MIN_TIME_BETWEEN_FRAMES {
                        last_frame.add(MIN_TIME_BETWEEN_FRAMES);
                        match &game_state {
                            GameState::WorldLoaded(world) 
                                => world.render(&mut renderer, since_last_frame / TIME_BETWEEN_UPDATES),
                            GameState::ShowControls => render_show_controls(&mut renderer),
                            GameState::Victory => render_victory(&mut renderer)
                        }
                    }
                },
            _ => {}
        }
    })
}

fn get_icon() -> winit::window::Icon {
    use image::GenericImageView;
    let image = image::load_from_memory(include_bytes!("../icon.png")).unwrap();
    let rgba = image.as_rgba8().unwrap().to_vec();
    winit::window::Icon::from_rgba(rgba, image.dimensions().0 as u32, image.dimensions().1 as u32).unwrap()
}

#[derive(Copy, Clone)]
struct Time (
    #[cfg(target_arch="wasm32")]
    f64,
    #[cfg(not(target_arch="wasm32"))]
    std::time::Instant
);

#[cfg(target_arch="wasm32")]
impl Time {
    fn now() -> Self {
        Self(js_sys::Date::now() / 1000.0)
    }

    fn elapsed(self: Self) -> f32 {
        (Self::now().0 - self.0) as f32
    }

    fn add(&mut self, duration: f32) {
        self.0 += duration as f64;
    }
}


#[cfg(not(target_arch="wasm32"))]
impl Time {
    fn now() -> Self {
        Self(std::time::Instant::now())
    }

    fn elapsed(&self) -> f32 {
        self.0.elapsed().as_secs_f32()
    }

    fn add(&mut self, duration: f32) {
        self.0 += std::time::Duration::from_secs_f32(duration);
    }
}

pub struct Camera {
    pos: Vec2,
    size: f32
}

pub struct World {
    state: WorldState,
    pressed_keys: HashMap<VirtualKeyCode, bool>,
    entities: hecs::World,
    player: Entity,
    cary: Entity,
    camera: Camera,
    /// In-game time in seconds
    time: f32
}

impl World {
    fn new() -> Self {
        let mut entities = hecs::World::new();
        let player = entities.spawn(make_player(Vec2(0.0, 0.0)));
        let cary = entities.spawn(make_cary(Vec2(0.0, 0.0)));

        World {
            state: WorldState::Running,
            pressed_keys: HashMap::new(),
            entities,
            player,
            cary,
            camera: Camera { pos: Vec2::zero(), size: 7.0 },
            time: 0.0
        }
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.entities.query::<Q>()
    }

    fn input(&mut self, input: winit::event::KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            if input.state == winit::event::ElementState::Pressed {
                self.pressed_keys.insert(keycode, true);
            } else {
                self.pressed_keys.remove(&keycode);
            }
        }
    }

    fn update(&mut self) {
        match self.state {
            WorldState::Running => {
                self.update_position_interpol();
                self.time += TIME_BETWEEN_UPDATES;
                self.update_player_input();
                self.update_player();
                self.update_cary();
                self.update_physics();
                self.update_hazzards();
                self.update_exits();
                self.update_animations();
                self.update_camera();
            },
            WorldState::Loss(_, ref mut time) => {
                *time += TIME_BETWEEN_UPDATES;
            },
            WorldState::Victory(_, ref mut time) => {
                *time += TIME_BETWEEN_UPDATES;
            }
        }
    }

    fn update_player_input(&mut self) {
        // TODO: figure out scancodes & use them instead
        use VirtualKeyCode::*;
        let mut control = self.entities.get_mut::<Controllable>(self.player).unwrap();
        control.vertical = match (self.pressed_keys.get(&W), self.pressed_keys.get(&S)) {
            (Some(true), Some(true)) => Vertical::None,
            (Some(false), Some(false)) => Vertical::None,
            (Some(_), _) => Vertical::Up,
            (_, Some(_)) => Vertical::Down,
            (None, None) => Vertical::None
        };
        control.horizontal = match (self.pressed_keys.get(&A), self.pressed_keys.get(&D)) {
            (Some(true), Some(true)) => Horizontal::None,
            (Some(false), Some(false)) => Horizontal::None,
            (Some(_), _) => Horizontal::Left,
            (_, Some(_)) => Horizontal::Right,
            (None, None) => Horizontal::None
        };
        control.pick_up = *self.pressed_keys.get(&J).unwrap_or(&false);

        for value in self.pressed_keys.values_mut() {
            *value = false;
        }
    }

    fn update_position_interpol(&mut self) {
        for (_, pos) in self.query::<&mut Pos>().iter() {
            pos.prev_interpol = pos.curr
        }
    }

    fn update_player(&mut self) {
        let mut player_query = 
            self.entities.query_one::<(&mut Player, &mut Physics, &Controllable, &mut Children, &mut Sprite)>(self.player).unwrap();
        let (player, physics, control, children, sprite) = player_query.get().unwrap();
    
        // Flap
        let flap_acc = 5.6;
        let flap_fall_decell = 9.0;
        let max_speed_upwards = 10.0;
        let idle_acc = 5.0;
        let dive_strenght = 25.0;
        let horizontal_acc = 7.8;
        let max_horizontal_speed = 4.0;
        match control.vertical {
            Vertical::Up if sprite.finished() => {
                if physics.vel.1 < 0.0 {
                    physics.vel.1 *= 1.0 - flap_fall_decell * TIME_BETWEEN_UPDATES;
                }
                physics.vel.1 = (physics.vel.1 + flap_acc).min(max_speed_upwards);
                sprite.tex = textures::PLAYER_FLY;
                sprite.timer = 0.0;
            },
            Vertical::None if sprite.finished() => {
                sprite.tex = if physics.vel.1 < -0.3*dive_strenght {textures::PLAYER_DIVE} else {textures::PLAYER_IDLE};
                physics.vel.1 += idle_acc * TIME_BETWEEN_UPDATES;
            },
            Vertical::Down => {
                sprite.tex = textures::PLAYER_DIVE;
                if physics.vel.1 > -0.5*dive_strenght {
                    physics.vel.0 *= 1.0 - 0.2*dive_strenght * TIME_BETWEEN_UPDATES;
                    physics.vel.1 -= dive_strenght * TIME_BETWEEN_UPDATES;
                }
            },
            _ => ()
        }
        if (control.horizontal == Horizontal::Left) & (physics.vel.0 > -max_horizontal_speed) {
            physics.vel.0 -= horizontal_acc * TIME_BETWEEN_UPDATES;
        }
        if (control.horizontal == Horizontal::Right) & (physics.vel.0 < max_horizontal_speed) {
            physics.vel.0 += horizontal_acc * TIME_BETWEEN_UPDATES;
        }
        player.flap_cooldown -= TIME_BETWEEN_UPDATES;
        // TODO: standing sprite

        let stamina_regen_rate = 0.3;
        let stamina_drain_rate = 0.18;
        if player.carrying.is_some() {
            player.stamina = (player.stamina - stamina_drain_rate * TIME_BETWEEN_UPDATES).max(0.0)
        } else {
            player.stamina = (player.stamina + stamina_regen_rate * TIME_BETWEEN_UPDATES).min(1.0)
        }

        if control.pick_up | (player.stamina == 0.0) {
            if let Some(carried) = player.carrying {
                player.carrying = None;
                children.0.retain(|child| *child != carried);
                drop(player_query);
                self.entities.get_mut::<Carryable>(carried).unwrap().carried = false;
                self.entities.remove_one::<ChildOf>(carried).unwrap();
            } else if let Some(to_be_carried) = self.find_pickupable() {
                player.carrying = Some(to_be_carried);
                children.0.push(to_be_carried);
                drop(player_query);
                let mut carryable = self.entities.get_mut::<Carryable>(to_be_carried).unwrap();
                carryable.carried = true;
                let carry_offset = carryable.carry_offset;
                drop(carryable);
                let child_bounds = self.entities.get_mut::<Physics>(to_be_carried).unwrap().bounds;
                self.entities.insert_one(to_be_carried,
                    ChildOf {
                        parent: self.player,
                        offset: carry_offset,
                        collision: child_bounds
                    }
                ).unwrap();
            }
        }
    }

    fn find_pickupable(&self) -> Option<Entity> {
        let player_pos = self.entities.get::<Pos>(self.player).unwrap();
        for (entity, (pos, carryable)) in self.query::<(&Pos, &Carryable)>().iter() {
            if (carryable.detect_bounds + pos.curr).contains(player_pos.curr) {
                return Some(entity)
            }
        }
        None
    }

    fn update_cary(&mut self) {
        if !self.entities.get::<Carryable>(self.cary).unwrap().carried {
            let mut cary = self.entities.get_mut::<Cary>(self.cary).unwrap();
            let pos = self.entities.get_mut::<Pos>(self.cary).unwrap();
            let mut physics = self.entities.get_mut::<Physics>(self.cary).unwrap();

            let walk_speed = 1.8;
            let jump_speed = 6.7;
            let jump_speed_low = 5.0;
            // Allow movement during junp
            if (physics.collided.1 == Vertical::Down) | (physics.vel.1 > 0.0) {
                physics.vel.0 = if cary.walk_right { walk_speed} else { -walk_speed };
            }
            if (physics.collided.1 == Vertical::Down) & 
                (((physics.collided.0 == Horizontal::Left) & !cary.walk_right) 
                |((physics.collided.0 == Horizontal::Right) & cary.walk_right))
            {
                // Decide whether to jump or to turn around
                let check_offset_x = if cary.walk_right {0.2} else {-0.2};
                if self.is_free(&(physics.bounds + pos.curr + Vec2(check_offset_x, 0.1))) {
                    physics.vel.1 = 1.5;
                } else if self.is_free(&(physics.bounds + pos.curr + Vec2(check_offset_x, 1.1))) {
                    physics.vel.1 = jump_speed_low;
                } else if self.is_free(&(physics.bounds + pos.curr + Vec2(check_offset_x, 2.1))) {
                    physics.vel.1 = jump_speed;
                } else {
                    let mut sprite = self.entities.get_mut::<Sprite>(self.cary).unwrap();
                    sprite.mirror = cary.walk_right;
                    cary.walk_right ^= true;
                }
            }
        }
    }

    fn is_free(&self, bounds: &Bounds) -> bool {
        for (_, (collision_pos, collider)) in self.query::<(&Pos, &Collider)>().iter() {
            if bounds.overlapps(collider.bounds + collision_pos.curr) {
                return false
            }
        }
        true
    }

    fn update_physics(&mut self) {
        const GRAVITY: f32 = 10.0;
        const TERMINAL_VELOCITY: f32 = 12.0;
        const GROUND_FRICTION: f32 = 4.5;
        for (entity, physics) in self.query::<&mut Physics>().iter() {
            if physics.vel.1 > -TERMINAL_VELOCITY {
                physics.vel.1 -= GRAVITY * TIME_BETWEEN_UPDATES;
            }

            let mut movement = physics.vel * TIME_BETWEEN_UPDATES;
            physics.collided = (Horizontal::None, Vertical::None);
            let bounds = physics.bounds + self.entities.get::<Pos>(entity).unwrap().curr;
            let children = self.entities.get::<Children>(entity).ok();
            let entities = &self.entities;
            for (collision_entity, (collision_pos, collider)) in self.query::<(&Pos, &Collider)>().iter() {
                let collision = collider.bounds + collision_pos.curr;
                for bounds in Some(bounds).into_iter().chain(children.iter().flat_map(
                    |children| children.0.iter().filter_map(
                        |child| Some(entities.get::<ChildOf>(*child).ok()?.collision
                                            + entities.get::<Pos>(*child).ok()?.curr)
                    ))) 
                {
                    if self.entities.get::<Carryable>(collision_entity).map_or(true, |carryable|!carryable.carried) {
                        if !bounds.overlapps(collision) & (bounds + movement).overlapps(collision) {
                            if (bounds + Vec2(movement.0, 0.0)).overlapps(collision) {
                                physics.collided.0 = if movement.0 > 0.0 { Horizontal::Right } else { Horizontal::Left };
                                physics.vel.0 = 0.0;
                                movement.0 = 0.0;
                            }
                            if (bounds + Vec2(0.0, movement.1)).overlapps(collision) {
                                physics.collided.1 = if movement.1 > 0.0 { Vertical::Up } else { Vertical::Down };
                                physics.vel.1 = 0.0;
                                physics.vel.0 *= 1.0 - GROUND_FRICTION * TIME_BETWEEN_UPDATES;
                                movement.1 = 0.0;
                            }
                            if (movement.0 != 0.0) & (movement.1 != 0.0) {
                                // Corner hit head on
                                movement = Vec2::zero()
                            }
                        }
                    }
                }
            }
            self.entities.get_mut::<Pos>(entity).unwrap().curr += movement;
        }

        // Children
        for (_, (pos, child_of)) in self.query::<(&mut Pos, &ChildOf)>().iter() {
            let parent_pos = self.entities.get::<Pos>(child_of.parent).unwrap();
            pos.curr = parent_pos.curr + child_of.offset;
        }
    }

    fn update_hazzards(&mut self) {
        let mut loss = None;
        for (_, (pos, killable)) in self.query::<(&Pos, &Killable)>().iter() {
            for (_, (hazzard_pos, hazzard)) in self.query::<(&Pos, &Hazzard)>().iter() {
                if (killable.bounds + pos.curr).overlapps(hazzard.bounds + hazzard_pos.curr) {
                    if killable.loss_on_death {
                        loss = Some(pos.curr);
                    } else {
                        todo!();
                    }
                }
            }
        }
        if let Some(pos) = loss {
            self.state = WorldState::Loss(pos, 0.0)
        }
    }

    fn update_exits(&mut self) {
        let cary_pos = self.entities.get::<Pos>(self.cary).unwrap().curr;
        let mut reached = false;
        for (_, (pos, exit)) in self.query::<(&Pos, &Exit)>().iter() {
            if (exit.0 + pos.curr).contains(cary_pos) {
                reached = true;
            }
        }
        if reached {
            self.state = WorldState::Victory(cary_pos, 0.0);
        }
    }

    fn update_animations(&mut self) {
        // We need to update this instead of just calculating the time elapsed
        // in render() because the game may be paused (or time-dilated for special effects)
        for (_, sprite) in self.query::<&mut Sprite>().iter() {
            if sprite.running {
                sprite.timer += TIME_BETWEEN_UPDATES
            }
        }
    }

    fn update_camera(&mut self) {
        let player_pos = self.entities.get::<Pos>(self.player).unwrap().curr;
        let cary_pos = self.entities.get::<Pos>(self.cary).unwrap().curr;

        let max_x_diff = 1.55 * self.camera.size; // todo: make dependant on aspect ratio
        let x_diff = (cary_pos.0 - player_pos.0).max(-max_x_diff).min(max_x_diff);
        let max_y_diff = 1.25 * self.camera.size;
        let y_diff = (cary_pos.1 - player_pos.1).max(-max_y_diff).min(max_y_diff);

        let target = Vec2(
            player_pos.0 + 0.5 * x_diff,
            player_pos.1 + 0.5 * y_diff
        );

        self.camera.pos += (target-self.camera.pos) * 1.5 * TIME_BETWEEN_UPDATES;

        fn zoom_dist(distance: Vec2) -> f32 {
            (distance.0.abs() / 3.0 /* arbitrarily chosen */).max(distance.1.abs())
        }
        let size_min: f32 = 7.0;
        let size_max: f32 = 12.0;
        let grow_target = zoom_dist(cary_pos - target) + 0.8;
        let shrink_target = zoom_dist(cary_pos - target) + 4.0;
        if grow_target > self.camera.size {
            self.camera.size = size_max.min(self.camera.size + (grow_target-self.camera.size) * 0.6 * TIME_BETWEEN_UPDATES);
        } else if shrink_target < self.camera.size {
            self.camera.size = size_min.max(self.camera.size + (shrink_target-self.camera.size) * 0.6 * TIME_BETWEEN_UPDATES);
        }
    }

    fn render(&self, renderer: &mut Renderer, lerp: f32) {
        // Sprites
        for (_, (pos, sprite)) in self.query::<(&Pos, &Sprite)>().iter() {
            let pos = pos.prev_interpol.lerp(pos.curr, lerp) + sprite.offset;
            let index_base = (sprite.timer / sprite.frame_duration) as usize;
            let tex = &sprite.tex[
                if sprite.repeat {
                    index_base % sprite.tex.len()
                } else {
                    index_base.min(sprite.tex.len()-1)
                }
            ];
            renderer.draw(&self.camera, pos, sprite.tex_anchor, tex, sprite.layer, sprite.mirror)
        }

        // Pickup hint
        if let Some(carryable) = self.find_pickupable() {
            let player_pos = self.entities.get::<Pos>(self.player).unwrap();
            let carryable = self.entities.get::<Carryable>(carryable).unwrap();
            if !carryable.carried {
                renderer.draw(&self.camera, player_pos.curr + carryable.carry_offset * 0.5, 
                    textures::TexAnchor::Center, &textures::PICKUP_HINT[0], Layer::ForegroundPickupHint, false);
            }
        }

        // Stamina bar
        let player_pos = self.entities.get::<Pos>(self.player).unwrap().curr;
        let player = self.entities.get::<Player>(self.player).unwrap();
        if match self.state { WorldState::Running => player.stamina < 1.0, _ => false } {
            // TODO: independant of camera.size
            renderer.draw(&self.camera, player_pos + Vec2(0.0, 0.7), textures::TexAnchor::Bottom,
                &textures::STAMINA_BAR[(player.stamina * textures::STAMINA_BAR.len() as f32) as usize], Layer::UI, false);
        }

        // Transition
        let transition_speed = 1.3;
        match self.state {
            WorldState::Running => {
                renderer.set_transition(&self.camera, Vec2::zero(), 0.0, false);
            },
            WorldState::Loss(center, time) => {
                renderer.set_transition(&self.camera, center, transition_speed * time, false);
                if time > GAME_END_WAIT_TIME {
                    renderer.draw(&UI_CAMERA, Vec2::zero(), textures::TexAnchor::Center, 
                        &textures::TEXT_RETRY[0], Layer::UI, false);
                }
            },
            WorldState::Victory(center, time) => {
                renderer.set_transition(&self.camera, center, transition_speed * time, true);
                if time > GAME_END_WAIT_TIME {
                    renderer.draw(&UI_CAMERA, Vec2::zero(), textures::TexAnchor::Center, 
                        &textures::TEXT_NEXT[0], Layer::UI, false);
                }
            }
        }

        renderer.render()
    }
}

enum WorldState {
    Running,
    Loss(Vec2, f32),
    Victory(Vec2, f32)
}

fn render_show_controls(renderer: &mut Renderer) {
    for x in -30..31 {
        for y in -10..10 {
            renderer.draw(&UI_CAMERA, Vec2(x as f32, y as f32), textures::TexAnchor::Center, 
                &textures::CYAN[0], Layer::ForegroundTile, false);
        }
    }
    renderer.draw(&UI_CAMERA, Vec2::zero(), textures::TexAnchor::Center, 
        &textures::CONTROLS[0], Layer::UI, false);
    renderer.render();
}

fn render_victory(renderer: &mut Renderer) {
    for x in -30..31 {
        for y in -10..10 {
            renderer.draw(&UI_CAMERA, Vec2(x as f32, y as f32), textures::TexAnchor::Center, 
                &textures::CYAN[0], Layer::ForegroundTile, false);
        }
    }
    renderer.draw(&UI_CAMERA, Vec2::zero(), textures::TexAnchor::Center, 
        &textures::VICTORY[0], Layer::UI, false);
    renderer.render();
}