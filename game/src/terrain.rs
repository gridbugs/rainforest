use crate::{
    components::EntityData,
    spatial::{Layer, Location},
    world::World,
};
use direction::CardinalDirection;
use entity_table::Entity;
use grid_2d::{Coord, Grid, Size};
use perlin2::Perlin2;
use rand::{seq::IteratorRandom, Rng};
use rgb_int::Rgb24;

pub struct Terrain {
    pub world: World,
    pub player: Entity,
}

#[allow(dead_code)]
pub fn from_str<R: Rng>(s: &str, player_data: EntityData, rng: &mut R) -> Terrain {
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
                    world.spawn_tree(coord, rng);
                }
                'L' => {
                    world.spawn_floor(coord);
                    world.spawn_light(coord, Rgb24::new(255, 185, 100));
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
                '~' => {
                    world.spawn_water(coord, rng);
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

const SIZE: Size = Size::new_u16(100, 100);

fn try_generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Result<Terrain, &str> {
    let size = SIZE;
    let topography = Perlin2::new(rng);
    let topography_spread = 0.05;
    let tree_chance = Perlin2::new(rng);
    let tree_chance_spread = 0.05;
    let tree_chance_scale = 0.3;
    let light_colour = Rgb24::new(255, 185, 100);
    let mut world = World::new(size);
    let mut player_data = Some(player_data);
    let topography_grid = Grid::new_fn(size, |coord| {
        topography.noise01((
            coord.x as f64 * topography_spread,
            coord.y as f64 * topography_spread,
        ))
    });
    let padding = 30;
    let cabin_coord = topography_grid
        .enumerate()
        .filter_map(|(coord, &f)| {
            if coord.x > padding
                && coord.y > padding
                && coord.x < size.x() as i32 - padding
                && coord.y < size.y() as i32 - padding
                && f > 0.65
            {
                Some(coord)
            } else {
                None
            }
        })
        .choose(rng)
        .ok_or("no cabin coord")?;
    world.spawn_light(cabin_coord, Rgb24::new(255, 255, 255));
    let cabin_size = Size::new(9, 9);
    let cabin_top_left = cabin_coord - (cabin_size.to_coord().unwrap() / 2);
    for coord in cabin_size.coord_iter_row_major() {
        world.spawn_floor(coord + cabin_top_left);
    }
    let door_direction = rng.gen::<CardinalDirection>();
    let door_coord =
        cabin_coord + door_direction.coord() * cabin_size.get(door_direction.axis()) as i32 / 2;
    world.spawn_door(door_coord, door_direction.axis());
    world.spawn_light(door_coord - door_direction.coord(), light_colour);
    for coord in [
        door_direction.left90().coord(),
        door_direction.right90().coord(),
    ] {
        let window_coord =
            door_coord + coord * cabin_size.get(door_direction.axis().other()) as i32 / 4;
        world.spawn_window(window_coord, door_direction.axis());
        world.spawn_light(window_coord - door_direction.coord(), light_colour);
    }
    for direction in [
        door_direction.left90(),
        door_direction.opposite(),
        door_direction.right90(),
    ] {
        let mid_coord =
            cabin_coord + direction.coord() * cabin_size.get(direction.axis()) as i32 / 2;
        world.spawn_window(mid_coord, direction.axis());
        world.spawn_light(mid_coord - direction.coord(), light_colour);
        for coord in [direction.left90().coord(), direction.right90().coord()] {
            let window_coord =
                mid_coord + coord * cabin_size.get(direction.axis().other()) as i32 / 4;
            world.spawn_window(window_coord, direction.axis());
            world.spawn_light(window_coord - direction.coord(), light_colour);
        }
    }
    for offset in cabin_size.coord_iter_row_major() {
        let coord = cabin_top_left + offset;
        if world
            .spatial_table
            .layers_at_checked(coord)
            .feature
            .is_none()
            && CardinalDirection::all().any(|d| {
                world
                    .spatial_table
                    .layers_at_checked(d.coord() + coord)
                    .floor
                    .is_none()
            })
        {
            world.spawn_wall(coord);
        }
    }
    let player_location = Location {
        coord: cabin_coord
            + door_direction.coord() * cabin_size.get(door_direction.axis()) as i32 * 2,
        layer: Some(Layer::Character),
    };
    let player = world.insert_entity_data(player_location, player_data.take().unwrap());
    let ruins_coord = topography_grid
        .enumerate()
        .filter_map(|(coord, &f)| {
            if coord.x > padding
                && coord.y > padding
                && coord.x < size.x() as i32 - padding
                && coord.y < size.y() as i32 - padding
                && coord.manhattan_distance(cabin_coord) > 30
                && f > 0.65
            {
                Some(coord)
            } else {
                None
            }
        })
        .choose(rng)
        .ok_or("no ruins coord")?;
    let ruins_txt = include_str!("ruins_prefab.txt");
    let ruins_num_rotations = rng.gen_range(0..4);
    for (y, row) in ruins_txt.split('\n').filter(|s| !s.is_empty()).enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let mut out_coord = Coord::new(x as i32, y as i32);
            for _ in 0..ruins_num_rotations {
                out_coord = out_coord.left90();
            }
            out_coord += ruins_coord;
            match ch {
                '#' => {
                    world.spawn_ruins_floor(out_coord);
                    world.spawn_ruins_wall(out_coord);
                }
                '.' => {
                    world.spawn_ruins_floor(out_coord);
                }
                '?' => {
                    world.spawn_ruins_floor(out_coord);
                    world.spawn_altar(out_coord);
                }
                other => panic!("unexpected char {}", other),
            }
        }
    }
    for coord in size.coord_iter_row_major() {
        if world.spatial_table.layers_at_checked(coord).floor.is_some() {
            continue;
        }
        if coord == player_location.coord {
            world.spawn_ground(coord);
            continue;
        }
        if rng.gen::<f64>()
            < tree_chance.noise01((
                coord.x as f64 * tree_chance_spread,
                coord.y as f64 * tree_chance_spread,
            )) * tree_chance_scale
        {
            world.spawn_tree(coord, rng);
        }
        world.spawn_ground(coord);
    }
    Ok(Terrain { world, player })
}

pub fn generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Terrain {
    loop {
        if let Ok(terrain) = try_generate(player_data.clone(), rng) {
            return terrain;
        }
    }
}
