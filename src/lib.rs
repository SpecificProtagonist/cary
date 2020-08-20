mod textures;
mod math;
mod components;
mod renderer;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use math::*;
use components::*;
use renderer::{Renderer, Rgb, Layer};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;



const TIME_BETWEEN_UPDATES: f32 = 1.0 / 25.0;
const MIN_TIME_BETWEEN_FRAMES: f32 = 1.0 / 60.0;



#[cfg(target_arch="wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    main()
}

pub fn main() {
    
    let mut world = World::new();

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
            Event::MainEventsCleared 
                => {
                    if last_update.elapsed() >= TIME_BETWEEN_UPDATES {
                        last_update.add(TIME_BETWEEN_UPDATES);
                        update_world(&mut world);
                    }
                    let since_last_frame = last_frame.elapsed();
                    if since_last_frame >= MIN_TIME_BETWEEN_FRAMES {
                        last_frame.add(MIN_TIME_BETWEEN_FRAMES);
                        render(&world, &mut renderer, since_last_frame / TIME_BETWEEN_UPDATES);
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

struct World {
    entities: hecs::World,
    /// In-game time in seconds
    time: f32
}

impl World {
    fn new() -> Self {
        let mut entities = hecs::World::new();

        // Test
        entities.spawn(components::make_bat(Vec2(0.0, 0.0)));
        for x in -10..11 {
            for y in -10..11 {
                entities.spawn((
                    Pos::from(Vec2(x as f32, y as f32)), 
                    Sprite::single(textures::TILE_DUNGEON_BRICK_BG, textures::TexAnchor::Center, Layer::BackgroundTile)));
            }
        }
        entities.spawn((Pos::from(Vec2(0.0, 0.0)), Light::simple(8.0, Rgb(1.0, 0.8, 0.3))));
        entities.spawn((Pos::from(Vec2(3.5, 1.0)), Light::simple(8.0, Rgb(0.7, 0.7, 1.0))));

        World {
            entities,
            time: 0.0
        }
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.entities.query::<Q>()
    }
}

fn update_world(world: &mut World) {
    update_position_interpol(world);
    world.time += TIME_BETWEEN_UPDATES;
    update_physics(world);
    update_animations(world);
}

fn update_position_interpol(world: &World) {
    for (_, pos) in world.query::<&mut Pos>().iter() {
        pos.prev_interpol = pos.curr
    }
}

fn update_physics(world: &World) {
    for (_, (pos, physics)) in world.query::<(&mut Pos, &mut Physics)>().iter() {
        physics.vel += physics.acc * TIME_BETWEEN_UPDATES;
        pos.curr += physics.vel * TIME_BETWEEN_UPDATES;

        // Test
        physics.vel.0 = (world.time * 0.5).sin();
        physics.vel.1 = (world.time * 0.5).cos();
    }
}

fn update_animations(world: &World) {
    // We need to update this instead of just calculating the time elapsed
    // in render() because the game may be paused (or time-dilated for special effects)
    for (_, sprite) in world.query::<&mut Sprite>().iter() {
        sprite.timer += TIME_BETWEEN_UPDATES
    }
}

fn render(world: &World, renderer: &mut Renderer, lerp: f32) {
    for (_, (pos, light)) in world.query::<(&Pos, &Light)>().iter() {
        renderer.draw_light(
            pos.prev_interpol.lerp(pos.curr, lerp), 
            light.tex, 
            light.size.map(|size|Vec2(size,size)).unwrap_or(light.tex.size / textures::PIXELS_PER_TILE), 
            light.color
        )
    }
    for (_, (pos, sprite)) in world.query::<(&Pos, &Sprite)>().iter() {
        let pos = pos.prev_interpol.lerp(pos.curr, lerp);
        let index_base = (sprite.timer / sprite.frame_duration) as usize;
        let tex = &sprite.tex[
            if sprite.repeat {
                index_base % sprite.tex.len()
            } else {
                index_base.min(sprite.tex.len()-1)
            }
        ];
        renderer.draw(pos, sprite.tex_anchor, tex, sprite.layer)
    }
    renderer.render()
}