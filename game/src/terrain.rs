use crate::{
    components::EntityData,
    spatial::{Layer, Location},
    world::World,
};
use entity_table::Entity;
use grid_2d::{Coord, Size};
use rgb_int::Rgb24;

pub struct Terrain {
    pub world: World,
    pub player: Entity,
}

#[allow(dead_code)]
pub fn from_str(s: &str, player_data: EntityData) -> Terrain {
    let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
    let mut world = World::new(size);
    let mut player_data = Some(player_data);
    let mut player = None;
    let mut prev_wall = false;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch.is_control() {
                continue;
            }
            let coord = Coord::new(x as i32, y as i32);
            match ch {
                ',' => {
                    world.spawn_floor(coord);
                }
                '.' => {
                    world.spawn_ground(coord);
                }
                '&' => {
                    world.spawn_ground(coord);
                    world.spawn_tree(coord);
                }
                'L' => {
                    world.spawn_floor(coord);
                    world.spawn_light(coord, Rgb24::new(255, 50, 0));
                }
                'M' => {
                    world.spawn_floor(coord);
                    world.spawn_light(coord, Rgb24::new(255, 255, 255));
                }
                '#' => {
                    world.spawn_floor(coord);
                    world.spawn_wall(coord);
                    prev_wall = true;
                    continue;
                }
                '+' => {
                    world.spawn_floor(coord);
                    if prev_wall {
                        world.spawn_door(coord, direction::Axis::Y);
                    } else {
                        world.spawn_door(coord, direction::Axis::X);
                    }
                }
                '%' => {
                    world.spawn_floor(coord);
                    if prev_wall {
                        world.spawn_window(coord, direction::Axis::Y);
                    } else {
                        world.spawn_window(coord, direction::Axis::X);
                    }
                }
                '@' => {
                    world.spawn_ground(coord);
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                }
                ' ' => (),
                _ => log::warn!(
                    "unexpected char in terrain: {} ({})",
                    ch.escape_unicode(),
                    ch
                ),
            }
            prev_wall = false;
        }
    }
    let player = player.expect("didn't create player");
    Terrain { world, player }
}
