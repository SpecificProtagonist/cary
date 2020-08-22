use crate::World;
use crate::math::*;
use crate::components::*;

pub fn load() -> World {
    let level_string = include_str!("../levels/test");

    let mut world = World::new();
   
    let mut x = 0;
    let mut y = 0;
    // TODO: spawn_batched() would be faster
    for c in level_string.chars() {
        match c {
            'P' => {
                let mut player_pos = world.entities.get_mut::<Pos>(world.player).unwrap();
                player_pos.curr = Vec2(x as f32, y as f32);
            },
            '#' => {
                world.entities.spawn(make_tile_solid(x, y));
            },
            ' ' => {
                world.entities.spawn(make_tile_background(x, y));
            },
            '.' => (),
            '\n' => {
                y -= 1;
                x = -1;
            },
            '\r' => (),
            _ => panic!()
        }
        x += 1;
    }

    world
}