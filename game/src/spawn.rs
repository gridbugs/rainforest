use crate::{
    components::{DoorState, Tile},
    spatial::{Layer, Location},
    visibility::Light,
    world::World,
};
use entity_table::Entity;
use grid_2d::coord_2d::{Axis, Coord};
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

    pub fn spawn_tree(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Tree);
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
}
