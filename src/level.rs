use crate::World;
use crate::math::*;
use crate::components::*;

const LEVELS: &[&str] = &[
    include_str!("../levels/level_0"),
    include_str!("../levels/level_1"),
    include_str!("../levels/level_2"),
    include_str!("../levels/level_3"),
];

pub fn level_count() -> usize {
    LEVELS.len()
}

pub fn load(level: usize) -> World {
    let level_string = LEVELS[level];

    let mut world = World::new();
   
    let mut x = 0;
    let mut y = 0;
    // TODO: spawn_batched() would be faster
    for c in level_string.chars() {
        let mut background = true;
        match c {
            'P' => {
                let mut player_pos = world.entities.get_mut::<Pos>(world.player).unwrap();
                player_pos.curr = Vec2(x as f32, y as f32);
            },
            'C' => {
                let mut cary_pos = world.entities.get_mut::<Pos>(world.cary).unwrap();
                cary_pos.curr = Vec2(x as f32, y as f32);
            },
            'E' => {
                world.entities.spawn(make_exit(x, y));
            },
            '#' => {
                background = false;
                world.entities.spawn(make_tile_solid(x, y));
            },
            ' ' => (),
            'M' => {
                world.entities.spawn(make_tile_movable(x, y));
            },
            '^' => {
                world.entities.spawn(make_spikes(x, y));
            },
            '-' => {
                world.entities.spawn(make_divider(x, y, false));
            },
            '|' => {
                world.entities.spawn(make_divider(x, y, true));
            },
            'u' => {
                world.entities.spawn(make_trap_ceiling(x, y));
            },
            '.' => {
                background = false;
            },
            '\n' => {
                background = false;
                y -= 1;
                x = -1;
            },
            '\r' => {
                background = false;
            },
            _ => panic!(format!("Unknown entity: {}", c))
        }
        if background {
            world.entities.spawn(make_tile_background(x, y));
        }
        x += 1;
    }

    let cary_pos = world.entities.get_mut::<Pos>(world.cary).unwrap().curr;
    world.camera.pos = cary_pos;

    world
}