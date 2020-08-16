mod renderer;
mod textures;
mod math;
mod components;

use std::time::{Instant, Duration};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use math::*;
use renderer::{Renderer, Rgb, Layer};
use components::*;



const UPDATES_PER_SECOND: f32 = 25.0;
const FRAMES_PER_SECOND_CAP: f32 = 60.0;


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


fn main() {
    
    let mut world = World::new();

    let event_loop = EventLoop::new();
    // There will be only one window -> ignore window ids in events
    let window = winit::window::Window::new(&event_loop).unwrap();
    let mut renderer = Renderer::create(&window);

    let time_between_updates = Duration::from_secs_f32(1.0 / UPDATES_PER_SECOND);
    let mut last_update = Instant::now();
    let min_time_between_frames = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND_CAP);
    let mut last_frame = Instant::now();

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
                    if last_update.elapsed() >= time_between_updates {
                        last_update += time_between_updates;
                        update_world(&mut world);
                    }
                    if last_update.elapsed() >= min_time_between_frames {
                        last_frame += min_time_between_frames;
                        render(&world, &mut renderer, last_update.elapsed().as_secs_f32() * UPDATES_PER_SECOND);
                    }
                },
            _ => {}
        }
    })
}

fn update_world(world: &mut World) {
    update_position_interpol(world);
    world.time += 1.0/UPDATES_PER_SECOND;
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
        physics.vel += physics.acc / UPDATES_PER_SECOND;
        pos.curr += physics.vel / UPDATES_PER_SECOND;

        // Test
        physics.vel.0 = (world.time * 0.5).sin();
        physics.vel.1 = (world.time * 0.5).cos();
    }
}

fn update_animations(world: &World) {
    // We need to update this instead of just calculating the time elapsed
    // in render() because the game may be paused (or time-dilated for special effects)
    for (_, sprite) in world.query::<&mut Sprite>().iter() {
        sprite.timer += 1.0/UPDATES_PER_SECOND
    }
}

fn render(world: &World, renderer: &mut Renderer, lerp: f32) {
    for (_, (pos, light)) in world.query::<(&Pos, &Light)>().iter() {
        renderer.draw_light(
            pos.prev_interpol.lerp(pos.curr, lerp), 
            light.tex, 
            light.size, 
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