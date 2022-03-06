use crate::{
    components::{DoorState, Tile},
    realtime::flicker,
    spatial::{Layer, Location},
    visibility::Light,
    world::World,
};
use entity_table::Entity;
use grid_2d::coord_2d::{Axis, Coord};
use rand::{seq::SliceRandom, Rng};
use rand_range::UniformInclusiveRange;
use rational::Rational;
use rgb_int::Rgb24;
use shadowcast::vision_distance::Circle;

impl World {
    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Floor);
        entity
    }

    pub fn spawn_ground(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Ground);
        entity
    }

    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Wall);
        self.components.solid.insert(entity, ());
        self.components.opacity.insert(entity, 255);
        entity
    }

    pub fn spawn_tree<R: Rng>(&mut self, coord: Coord, rng: &mut R) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        let &tile = [Tile::Tree0, Tile::Tree1, Tile::Tree2].choose(rng).unwrap();
        self.components.tile.insert(entity, tile);
        self.components.solid.insert(entity, ());
        self.components.opacity.insert(entity, 127);
        entity
    }

    pub fn spawn_door(&mut self, coord: Coord, axis: Axis) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::DoorClosed(axis));
        self.components.opacity.insert(entity, 255);
        self.components.solid.insert(entity, ());
        self.components.door_state.insert(entity, DoorState::Closed);
        entity
    }

    pub fn spawn_window(&mut self, coord: Coord, axis: Axis) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Window(axis));
        self.components.solid.insert(entity, ());
        entity
    }

    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(entity, Location { coord, layer: None })
            .unwrap();
        self.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(200),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        entity
    }

    pub fn spawn_water<R: Rng>(&mut self, coord: Coord, rng: &mut R) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Water);
        let colour_range = UniformInclusiveRange {
            low: Rgb24::new(10, 40, 100),
            high: Rgb24::new(20, 90, 150),
        };
        self.realtime_components.flicker.insert(entity, {
            use flicker::spec::*;
            Flicker {
                colour_hint: Some(colour_range),
                light_colour: None,
                tile: None,
                until_next_event: UniformInclusiveRange {
                    low: Duration::from_millis(200),
                    high: Duration::from_millis(1000),
                },
            }
            .build(rng)
        });
        self.components
            .colour_hint
            .insert(entity, colour_range.choose(rng));
        self.components.realtime.insert(entity, ());
        entity
    }
}