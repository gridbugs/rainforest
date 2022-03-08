use crate::{
    components::EntityData,
    spatial::{Layer, Location},
    world::World,
};
use direction::CardinalDirection;
use entity_table::Entity;
use grid_2d::{Coord, Grid, Size};
use perlin2::Perlin2;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

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
                    world.spawn_floor(coord, 0.);
                }
                '.' => {
                    world.spawn_ground(coord, 0.);
                }
                '1' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_gumboots(coord);
                }
                '2' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_umbrella(coord);
                }
                '3' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_shovel(coord);
                }
                '4' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_map(coord);
                }
                '5' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_weather_report(coord);
                }
                '&' => {
                    world.spawn_ground(coord, 0.);
                    world.spawn_tree(coord, rng);
                }
                'L' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_light(coord);
                }
                'M' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_light(coord);
                }
                '#' => {
                    world.spawn_floor(coord, 0.);
                    world.spawn_wall(coord);
                    prev_wall = true;
                    continue;
                }
                '+' => {
                    world.spawn_floor(coord, 0.);
                    if prev_wall {
                        world.spawn_door(coord, direction::Axis::Y);
                    } else {
                        world.spawn_door(coord, direction::Axis::X);
                    }
                }
                '%' => {
                    world.spawn_floor(coord, 0.);
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
                    world.spawn_ground(coord, 0.);
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

const SIZE: Size = Size::new_u16(140, 140);

fn try_generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Result<Terrain, &str> {
    let size = SIZE;
    let topography = Perlin2::new(rng);
    let topography_spread = 0.05;
    let tree_chance = Perlin2::new(rng);
    let tree_chance_spread = 0.05;
    let tree_chance_scale = 0.3;
    let mut world = World::new(size);
    let mut no_trees = Grid::new_copy(size, false);
    let mut player_data = Some(player_data);
    let topography_grid = Grid::new_fn(size, |coord| {
        topography.noise01((
            coord.x as f64 * topography_spread,
            coord.y as f64 * topography_spread,
        ))
    });
    let padding = 35;
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
    world.spawn_bed(cabin_coord);
    world.spawn_light(cabin_coord);
    let cabin_size = Size::new(9, 9);
    let cabin_top_left = cabin_coord - (cabin_size.to_coord().unwrap() / 2);
    for coord in cabin_size.coord_iter_row_major() {
        world.spawn_floor(
            coord + cabin_top_left,
            *topography_grid.get_checked(coord + cabin_top_left),
        );
    }
    let door_direction = rng.gen::<CardinalDirection>();
    let door_coord =
        cabin_coord + door_direction.coord() * cabin_size.get(door_direction.axis()) as i32 / 2;
    *no_trees.get_checked_mut(door_coord + door_direction.coord()) = true;
    *no_trees.get_checked_mut(door_coord + door_direction.coord() * 2) = true;
    world.spawn_door(door_coord, door_direction.axis());
    world.spawn_light(door_coord - door_direction.coord());
    let lamp_coord =
        door_coord + (door_direction.coord() * 5) + door_direction.left90().coord() * 1;
    world.spawn_lamp(lamp_coord);
    let bulletin_board_coord =
        door_coord + (door_direction.coord() * 3) + door_direction.right90().coord();
    world.spawn_bulletin_board(bulletin_board_coord);
    *no_trees.get_checked_mut(bulletin_board_coord + Coord::new(0, 1)) = true;
    *no_trees.get_checked_mut(bulletin_board_coord + Coord::new(0, 2)) = true;
    let mut chair_candidates = Vec::new();
    for coord in [
        door_direction.left90().coord(),
        door_direction.right90().coord(),
    ] {
        let window_coord =
            door_coord + coord * cabin_size.get(door_direction.axis().other()) as i32 / 4;
        world.spawn_window(window_coord, door_direction.axis());
        world.spawn_light(window_coord - door_direction.coord());
        chair_candidates.push(window_coord - door_direction.coord());
    }
    for direction in [
        door_direction.left90(),
        door_direction.opposite(),
        door_direction.right90(),
    ] {
        let mid_coord =
            cabin_coord + direction.coord() * cabin_size.get(direction.axis()) as i32 / 2;
        world.spawn_window(mid_coord, direction.axis());
        world.spawn_light(mid_coord - direction.coord());
        chair_candidates.push(mid_coord - direction.coord());
        for coord in [direction.left90().coord(), direction.right90().coord()] {
            let window_coord =
                mid_coord + coord * cabin_size.get(direction.axis().other()) as i32 / 4;
            world.spawn_window(window_coord, direction.axis());
            world.spawn_light(window_coord - direction.coord());
            chair_candidates.push(window_coord - direction.coord());
        }
    }
    let &chair_coord = chair_candidates.choose(rng).unwrap();
    if chair_coord.x < cabin_coord.x {
        world.spawn_chair_left_facing(chair_coord);
    } else {
        world.spawn_chair_right_facing(chair_coord);
    }
    let chair_window_direction = CardinalDirection::all()
        .find(|d| {
            world
                .spatial_table
                .layers_at_checked(chair_coord + d.coord())
                .feature
                .is_some()
        })
        .unwrap();
    let teapot_coord = chair_coord
        + if rng.gen() {
            chair_window_direction.left90().coord()
        } else {
            chair_window_direction.right90().coord()
        };
    world.spawn_teapot(teapot_coord);
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
                && coord.manhattan_distance(cabin_coord) > 20
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
                    world.spawn_ruins_floor(out_coord, *topography_grid.get_checked(out_coord));
                    world.spawn_ruins_wall(out_coord);
                }
                '.' => {
                    world.spawn_ruins_floor(out_coord, *topography_grid.get_checked(out_coord));
                }
                '?' => {
                    world.spawn_ruins_floor(out_coord, *topography_grid.get_checked(out_coord));
                    world.spawn_altar(out_coord);
                }
                other => panic!("unexpected char {}", other),
            }
        }
    }
    let num_extra_lamps = 3;
    let mut lamp_coords = vec![lamp_coord];
    for _ in 0..num_extra_lamps {
        let coord = size
            .coord_iter_row_major()
            .filter(|coord| {
                coord.x > padding
                    && coord.y > padding
                    && coord.x < size.x() as i32 - padding
                    && coord.y < size.y() as i32 - padding
                    && coord.manhattan_distance(cabin_coord) > 20
                    && coord.manhattan_distance(ruins_coord) > 20
                    && lamp_coords
                        .iter()
                        .all(|&c| coord.manhattan_distance(c) > 20)
            })
            .choose(rng)
            .ok_or("no lamp coord")?;
        lamp_coords.push(coord);
        world.spawn_lamp(coord);
        *no_trees.get_checked_mut(lamp_coord + Coord::new(0, 1)) = true;
        *no_trees.get_checked_mut(lamp_coord + Coord::new(0, 2)) = true;
    }
    let lake_direction = rng.gen::<CardinalDirection>();
    let lake_mid = (size.to_coord().unwrap() / 2)
        + (lake_direction.coord() * (((size.get(lake_direction.axis()) as i32) / 2) - 24));
    let pier_offset = rng.gen_range(-40..40) + lake_mid.get(lake_direction.axis().other());
    let lake_edge = lake_mid.set(lake_direction.axis().other(), 0);
    let mut lake_offset = 0;
    let lake_offset_step_pool = [0, 0, 0, 0, 0, 0, 0, 1, -1];
    for i in 0..(size.get(lake_direction.axis().other()) as i32) {
        lake_offset += lake_offset_step_pool.choose(rng).unwrap();
        let mut j = lake_edge.get(lake_direction.axis()) + lake_offset;
        if i == pier_offset {
            let pier_coord =
                Coord::new_axis(j - lake_direction.sign() * 2, i, lake_direction.axis());
            *no_trees.get_checked_mut(pier_coord - Coord::new_axis(1, 0, lake_direction.axis())) =
                true;
            let pier_length = 8;
            for i in 0..pier_length {
                let coord = pier_coord
                    + Coord::new_axis(i * lake_direction.sign(), 0, lake_direction.axis());
                world.spawn_pier_floor(coord, *topography_grid.get_checked(coord));
                *no_trees.get_checked_mut(coord) = true;
            }
            let pier_lamp_coord = pier_coord
                + (lake_direction.coord() * (pier_length - 1))
                + if rng.gen() {
                    lake_direction.left90().coord()
                } else {
                    lake_direction.right90().coord()
                };
            if let Some(entity) = world.spatial_table.layers_at_checked(pier_lamp_coord).floor {
                world.spatial_table.remove(entity);
                world.components.remove_entity(entity);
            }
            if let Some(entity) = world
                .spatial_table
                .layers_at_checked(pier_lamp_coord)
                .feature
            {
                world.spatial_table.remove(entity);
                world.components.remove_entity(entity);
            }

            world.spawn_pier_floor(pier_lamp_coord, 1.);
            world.spawn_lamp(pier_lamp_coord);
        }
        let mut count = 0;
        loop {
            let coord = Coord::new_axis(j, i, lake_direction.axis());
            if !coord.is_valid(size) {
                break;
            }
            if world.spatial_table.layers_at_checked(coord).floor.is_none() {
                if coord == player_location.coord {
                    return Err("player spawned in lake");
                }
                world.spawn_lake_water(coord, rng);
                let noise = tree_chance.noise01((
                    coord.x as f64 * tree_chance_spread,
                    coord.y as f64 * tree_chance_spread,
                ));
                if rng.gen::<f64>() < (noise - (count as f64 / 10.)) {
                    world.spawn_grass(coord);
                }
            }
            *no_trees.get_checked_mut(coord) = true;
            j += lake_direction.sign();
            count += 1;
        }
    }
    let flower_patch_coord = topography_grid
        .enumerate()
        .filter_map(|(coord, &f)| {
            if coord.x > padding
                && coord.y > padding
                && coord.x < size.x() as i32 - padding
                && coord.y < size.y() as i32 - padding
                && coord.manhattan_distance(cabin_coord) > 20
                && coord.manhattan_distance(ruins_coord) > 20
                && f > 0.65
            {
                Some(coord)
            } else {
                None
            }
        })
        .choose(rng)
        .ok_or("no flower patch coord")?;
    let flower_patch_radius = 5;
    let mut flower_candidates = Vec::new();
    for offset in Size::new(flower_patch_radius * 2, flower_patch_radius * 2).coord_iter_row_major()
    {
        let rel_coord = offset - Coord::new(flower_patch_radius as i32, flower_patch_radius as i32);
        if rel_coord.magnitude2() < (flower_patch_radius * flower_patch_radius) {
            let abs_coord = rel_coord + flower_patch_coord;
            let layers = world.spatial_table.layers_at_checked(abs_coord);
            if layers.item.is_none() && layers.feature.is_none() {
                flower_candidates.push(abs_coord);
            }
        }
    }
    let num_flowers = 12;
    if flower_candidates.len() < num_flowers {
        return Err("not enough flower candidates");
    }
    for &coord in flower_candidates.choose_multiple(rng, num_flowers) {
        world.spawn_flower(coord);
        *no_trees.get_checked_mut(coord) = true;
    }
    let tea_patch_coord = topography_grid
        .enumerate()
        .filter_map(|(coord, &f)| {
            if coord.x > padding
                && coord.y > padding
                && coord.x < size.x() as i32 - padding
                && coord.y < size.y() as i32 - padding
                && coord.manhattan_distance(cabin_coord) > 20
                && coord.manhattan_distance(ruins_coord) > 20
                && coord.manhattan_distance(flower_patch_coord) > 20
                && f > 0.65
            {
                Some(coord)
            } else {
                None
            }
        })
        .choose(rng)
        .ok_or("no tea patch coord")?;
    let tea_patch_radius = 5;
    let mut tea_candidates = Vec::new();
    for offset in Size::new(tea_patch_radius * 2, tea_patch_radius * 2).coord_iter_row_major() {
        let rel_coord = offset - Coord::new(tea_patch_radius as i32, tea_patch_radius as i32);
        if rel_coord.magnitude2() < (tea_patch_radius * tea_patch_radius) {
            let abs_coord = rel_coord + tea_patch_coord;
            let layers = world.spatial_table.layers_at_checked(abs_coord);
            if layers.item.is_none() && layers.feature.is_none() {
                tea_candidates.push(abs_coord);
            }
        }
    }
    let num_tea = 12;
    if tea_candidates.len() < num_tea {
        return Err("not enough tea candidates");
    }
    for &coord in tea_candidates.choose_multiple(rng, num_tea) {
        world.spawn_tea(coord);
        *no_trees.get_checked_mut(coord) = true;
    }
    let close_to_edge = |padding: i32, coord: Coord| {
        coord.x < padding
            || coord.y < padding
            || coord.x > size.width() as i32 - padding
            || coord.y > size.height() as i32 - padding
    };
    let mut rock_candidates = Vec::new();
    for coord in size.coord_iter_row_major() {
        if world.spatial_table.layers_at_checked(coord).floor.is_some() {
            continue;
        }
        if coord == player_location.coord {
            world.spawn_ground(coord, *topography_grid.get_checked(coord));
            continue;
        }
        if !no_trees.get_checked(coord)
            && world
                .spatial_table
                .layers_at_checked(coord)
                .feature
                .is_none()
        {
            if close_to_edge(3, coord)
                || (close_to_edge(10, coord) && rng.gen::<f64>() < 0.5)
                || rng.gen::<f64>()
                    < (tree_chance.noise01((
                        coord.x as f64 * tree_chance_spread,
                        coord.y as f64 * tree_chance_spread,
                    )) - 0.2)
                        * tree_chance_scale
            {
                world.spawn_tree(coord, rng);
            } else {
                let noise = tree_chance.noise01((
                    coord.x as f64 * tree_chance_spread,
                    coord.y as f64 * tree_chance_spread,
                ));
                if (noise < 0.4 && rng.gen::<f64>() * 0.8 > noise) || rng.gen::<f64>() < 0.1 {
                    world.spawn_grass(coord);
                } else {
                    let layers = world.spatial_table.layers_at_checked(coord);
                    if layers.item.is_none() && layers.feature.is_none() {
                        rock_candidates.push(coord);
                    }
                }
            }
        }
        world.spawn_ground(coord, *topography_grid.get_checked(coord));
    }
    let num_rocks = 60;
    if rock_candidates.len() < num_rocks {
        return Err("not enough rock rock candidate");
    }
    for &coord in rock_candidates.choose_multiple(rng, num_rocks) {
        world.spawn_rock(coord);
    }
    Ok(Terrain { world, player })
}

pub fn generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Terrain {
    loop {
        match try_generate(player_data.clone(), rng) {
            Ok(terrain) => break terrain,
            Err(message) => log::warn!("{}", message),
        }
    }
}
