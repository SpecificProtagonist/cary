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
        let mut background = false;
        match c {
            'P' => {
                background = true;
                let mut player_pos = world.entities.get_mut::<Pos>(world.player).unwrap();
                player_pos.curr = Vec2(x as f32, y as f32);
            },
            'C' => {
                background = true;
                let mut cary_pos = world.entities.get_mut::<Pos>(world.cary).unwrap();
                cary_pos.curr = Vec2(x as f32, y as f32);
            },
            'E' => {
                background = true;
                world.entities.spawn(make_exit(x, y));
            },
            '#' => {
                world.entities.spawn(make_tile_solid(x, y));
            },
            ' ' => {
                background = true;
            },
            'M' => {
                world.entities.spawn(make_tile_movable(x, y));
            },
            '^' => {
                background = true;
                world.entities.spawn(make_spikes(x, y));
            },
            'u' => {
                background = true;
                world.entities.spawn(make_trap_ceiling(x, y));
            },
            '.' => (),
            '\n' => {
                y -= 1;
                x = -1;
            },
            '\r' => (),
            _ => panic!(format!("Unknown entity: {}", c))
        }
        if background {
            world.entities.spawn(make_tile_background(x, y));
        }
        x += 1;
    }

    world
}