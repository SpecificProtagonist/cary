use crate::World;
use crate::math::*;
use crate::components::*;

const DEFAULT_LEVELS: &[&str] = &[
    include_str!("../levels/level_0"),
    include_str!("../levels/level_1"),
    include_str!("../levels/level_2"),
    include_str!("../levels/level_3"),
    include_str!("../levels/level_4"),
    include_str!("../levels/level_5"),
];

pub struct Levels {
    level: usize,
    pub level_strings: Vec<String>
}


impl Levels {
    pub fn default() -> Self {
        Self::new(DEFAULT_LEVELS.iter().map(|s|s.to_string()).collect())
    }

    pub fn new(level_strings: Vec<String>) -> Self {
        Levels {
            level: 0,
            level_strings
        }
    }

    pub fn next(&mut self) -> bool {
        if self.level + 1 < self.level_strings.len() {
            self.level += 1;
            true
        } else {
            false
        }
    }

    pub fn load(&self) -> World {
        let level_string = &self.level_strings[self.level];

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
                    world.entities.spawn(make_spikes(x, y, 0));
                },
                '>' => {
                    world.entities.spawn(make_spikes(x, y, 1));
                },
                'v' => {
                    world.entities.spawn(make_spikes(x, y, 2));
                },
                '<' => {
                    world.entities.spawn(make_spikes(x, y, 3));
                },
                '-' => {
                    world.entities.spawn(make_divider(x, y, false));
                },
                '|' => {
                    world.entities.spawn(make_divider(x, y, true));
                },
                'u' => {
                    world.entities.spawn(make_trap(x, y, 0));
                },
                '⊂' => {
                    world.entities.spawn(make_trap(x, y, 1));
                },
                'n' => {
                    world.entities.spawn(make_trap(x, y, 2));
                },
                '⊃' => {
                    world.entities.spawn(make_trap(x, y, 3));
                },
                'S' => {
                    background = false;
                    world.entities.spawn(make_shooter(x, y));
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
                _ => {
                    println!("Failed to load level - Unknown entity: {}", c);
                    std::process::exit(2);
                }
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
}