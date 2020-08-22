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
use renderer::{Renderer, Rgb, Layer};



const TIME_BETWEEN_UPDATES: f32 = 1.0 / 25.0;
const MIN_TIME_BETWEEN_FRAMES: f32 = 1.0 / 60.0;



#[cfg(target_arch="wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    main()
}

pub fn main() {
    
    let mut world = level::load();

    let event_loop = EventLoop::new();
    // There will be only one window -> ignore window ids in events
    let window = winit::window::Window::new(&event_loop).unwrap();
    // Web: Add canvas to page
    #[cfg(target_arch="wasm32")] {
        use winit::platform::web::WindowExtWebSys;
        let web_window = web_sys::window().unwrap();
        let document = web_window.document().unwrap();
        let body = document.body().unwrap();
        body.append_child(&window.canvas()).unwrap();
    }
    let mut renderer = Renderer::create(&window);

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
                => world.input(input),
            Event::MainEventsCleared 
                => {
                    if last_update.elapsed() >= TIME_BETWEEN_UPDATES {
                        last_update.add(TIME_BETWEEN_UPDATES);
                        world.update();
                    }
                    let since_last_frame = last_frame.elapsed();
                    if since_last_frame >= MIN_TIME_BETWEEN_FRAMES {
                        last_frame.add(MIN_TIME_BETWEEN_FRAMES);
                        world.render(&mut renderer, since_last_frame / TIME_BETWEEN_UPDATES);
                    }
                },
            _ => {}
        }
    })
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

struct Camera {
    pos: Vec2,
    size: f32
}

pub struct World {
    pressed_keys: HashMap<VirtualKeyCode, bool>,
    entities: hecs::World,
    player: Entity,
    camera: Camera,
    /// In-game time in seconds
    time: f32
}

impl World {
    fn new() -> Self {
        let mut entities = hecs::World::new();
        let player = entities.spawn(components::make_player(Vec2(0.0, 0.0)));

        World {
            pressed_keys: HashMap::new(),
            entities,
            player,
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
        self.update_position_interpol();
        self.time += TIME_BETWEEN_UPDATES;
        self.update_player_input();
        self.update_player();
        self.update_physics();
        self.update_animations();
        self.update_camera();
    }

    fn update_player_input(&mut self) {
        // TODO: figure out scancodes & use them instead
        use VirtualKeyCode::*;
        let mut control = self.entities.get_mut::<Controllable>(self.player).unwrap();
        control.vertical = match (self.pressed_keys.get(&W), self.pressed_keys.get(&S)) {
            (Some(true), Some(true)) => VControl::None,
            (Some(false), Some(false)) => VControl::None,
            (Some(_), _) => VControl::Up,
            (_, Some(_)) => VControl::Down,
            (None, None) => VControl::None
        };
        control.horizontal = match (self.pressed_keys.get(&A), self.pressed_keys.get(&D)) {
            (Some(true), Some(true)) => HControl::None,
            (Some(false), Some(false)) => HControl::None,
            (Some(_), _) => HControl::Left,
            (_, Some(_)) => HControl::Right,
            (None, None) => HControl::None
        };
        control.attack = self.pressed_keys.contains_key(&J);

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
        for (_, (player, physics, control, sprite)) in self.query::<(&mut Player, &mut Physics, &Controllable, &mut Sprite)>().iter() {
            // Flap
            let flap_acc = 5.6;
            let max_speed_upwards = 10.0;
            let idle_acc = 5.0;
            let dive_strenght = 25.0;
            let horizontal_acc = 6.0;
            let max_horizontal_speed = 5.5;
            match control.vertical {
                VControl::Up if sprite.finished() => {
                    physics.vel.1 = (physics.vel.1 + flap_acc).min(max_speed_upwards);
                    sprite.tex = textures::PLAYER_FLY;
                    sprite.timer = 0.0;
                },
                VControl::None if sprite.finished() => {
                    sprite.tex = if physics.vel.1 < -0.3*dive_strenght {textures::PLAYER_DIVE} else {textures::PLAYER_IDLE};
                    physics.vel.1 += idle_acc * TIME_BETWEEN_UPDATES;
                },
                VControl::Down => {
                    sprite.tex = textures::PLAYER_DIVE;
                    if physics.vel.1 > -0.5*dive_strenght {
                        physics.vel.0 *= 1.0 - 0.2*dive_strenght * TIME_BETWEEN_UPDATES;
                        physics.vel.1 -= dive_strenght * TIME_BETWEEN_UPDATES;
                    }
                },
                _ => ()
            }
            if (control.horizontal == HControl::Left) & (physics.vel.0 > -max_horizontal_speed) {
                physics.vel.0 -= horizontal_acc * TIME_BETWEEN_UPDATES;
            }
            if (control.horizontal == HControl::Right) & (physics.vel.0 < max_horizontal_speed) {
                physics.vel.0 += horizontal_acc * TIME_BETWEEN_UPDATES;
            }
            player.flap_cooldown -= TIME_BETWEEN_UPDATES;
            // TODO: idle, standing & dive animations
        }
    }

    fn update_physics(&mut self) {
        const GRAVITY: f32 = 10.0;
        const TERMINAL_VELOCITY: f32 = 12.0;
        const GROUND_FRICTION: f32 = 4.5;
        for (_, (pos, physics)) in self.query::<(&mut Pos, &mut Physics)>().iter() {
            if physics.vel.1 > -TERMINAL_VELOCITY {
                physics.vel.1 -= GRAVITY * TIME_BETWEEN_UPDATES;
            }

            let bounds = physics.bounds.moved(pos.curr);
            let mut movement = physics.vel * TIME_BETWEEN_UPDATES;
            for (_, (collision_pos, collider)) in self.query::<(&Pos, &Collider)>().iter() {
                let collision = collider.bounds.moved(collision_pos.curr);
                if bounds.moved(movement).overlapps(collision) {
                    if bounds.moved(Vec2(movement.0, 0.0)).overlapps(collision) {
                        physics.vel.0 = 0.0;
                        movement.0 = 0.0;
                    }
                    if bounds.moved(Vec2(0.0, movement.1)).overlapps(collision) {
                        physics.vel.1 = 0.0;
                        physics.vel.0 *= 1.0 - GROUND_FRICTION * TIME_BETWEEN_UPDATES;
                        movement.1 = 0.0;
                    }
                }
            }
            pos.curr += movement;
        }
    }

    fn update_animations(&mut self) {
        // We need to update this instead of just calculating the time elapsed
        // in render() because the game may be paused (or time-dilated for special effects)
        for (_, sprite) in self.query::<&mut Sprite>().iter() {
            sprite.timer += TIME_BETWEEN_UPDATES
        }
    }

    fn update_camera(&mut self) {
        let player_pos = self.entities.get::<Pos>(self.player).unwrap();
        self.camera.pos += (player_pos.curr-self.camera.pos) * TIME_BETWEEN_UPDATES;
    }

    fn render(&self, renderer: &mut Renderer, lerp: f32) {
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
            renderer.draw(&self.camera, pos, sprite.tex_anchor, tex, sprite.layer)
        }
        renderer.render()
    }
}