use crate::{
    components::{DoorState, Equipment, Item, Tile},
    realtime::flicker,
    spatial::{Layer, Location},
    visibility::Light,
    world::World,
};
use gridbugs::{
    entity_table::Entity,
    grid_2d::coord_2d::{Axis, Coord},
    rgb_int::Rgb24,
    shadowcast::vision_distance::Circle,
};
use rand::{seq::SliceRandom, Rng};
use rand_range::UniformInclusiveRange;
use rational::Rational;

impl World {
    pub fn spawn_floor(&mut self, coord: Coord, height: f64) -> Entity {
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
        self.components.height.insert(entity, height);
        entity
    }

    pub fn spawn_ground(&mut self, coord: Coord, height: f64) -> Entity {
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
        self.components.height.insert(entity, height);
        self.components.ground.insert(entity, ());
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
        self.components.tree.insert(entity, ());
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

    pub const HOUSE_LIGHT: Light = Light {
        colour: Rgb24::new(255, 255, 255),
        vision_distance: Circle::new_squared(200),
        diminish: Rational {
            numerator: 1,
            denominator: 10,
        },
    };

    pub fn spawn_light(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(entity, Location { coord, layer: None })
            .unwrap();
        self.components.light.insert(entity, Self::HOUSE_LIGHT);
        self.components.house_light.insert(entity, ());
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
        self.become_water(entity, rng);
        entity
    }
    pub fn become_water<R: Rng>(&mut self, entity: Entity, rng: &mut R) {
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
        self.components.water.insert(entity, ());
    }

    pub fn spawn_ruins_floor(&mut self, coord: Coord, height: f64) -> Entity {
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
        self.components.tile.insert(entity, Tile::RuinsFloor);
        self.components.height.insert(entity, height);
        entity
    }

    pub fn spawn_ruins_wall(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::RuinsWall);
        self.components.solid.insert(entity, ());
        self.components.opacity.insert(entity, 255);
        entity
    }

    pub fn spawn_altar(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Altar);
        self.components.solid.insert(entity, ());
        self.components.altar.insert(entity, ());
        entity
    }

    pub fn spawn_bulletin_board(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::BulletinBoard);
        self.components.solid.insert(entity, ());
        self.components.bulletin_board.insert(entity, ());
        entity
    }

    pub const LAMP_LIGHT: Light = Light {
        colour: Rgb24::new(255, 255, 185),
        vision_distance: Circle::new_squared(100),
        diminish: Rational {
            numerator: 1,
            denominator: 20,
        },
    };

    pub fn spawn_lamp(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Lamp);
        self.components.solid.insert(entity, ());
        self.components.light.insert(entity, Self::LAMP_LIGHT);
        self.components.lamp.insert(entity, ());
        entity
    }

    pub fn spawn_pier_floor(&mut self, coord: Coord, height: f64) -> Entity {
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
        self.components.tile.insert(entity, Tile::PierFloor);
        self.components.height.insert(entity, height);
        entity
    }

    pub fn spawn_lake_water<R: Rng>(&mut self, coord: Coord, rng: &mut R) -> Entity {
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
        self.become_water(entity, rng);
        self.components.lake.insert(entity, ());
        entity
    }

    pub fn spawn_grass(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Grass);
        self.components.opacity.insert(entity, 100);
        self.components.grass.insert(entity, ());
        entity
    }

    pub fn spawn_rock(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Rock);
        self.components.push.insert(entity, ());
        self.components.rock.insert(entity, ());
        entity
    }

    pub fn spawn_flower(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Flower);
        self.components.item.insert(entity, Item::Flower);
        entity
    }

    pub fn spawn_bed(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Bed);
        self.components.bed.insert(entity, ());
        entity
    }

    pub fn spawn_chair_left_facing(&mut self, coord: Coord) -> Entity {
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
        self.components.solid.insert(entity, ());
        self.components.chair.insert(entity, ());
        self.components.tile.insert(entity, Tile::ChairLeftFacing);
        entity
    }

    pub fn spawn_chair_right_facing(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::ChairRightFacing);
        self.components.solid.insert(entity, ());
        self.components.chair.insert(entity, ());
        entity
    }

    pub fn spawn_teapot(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Teapot);
        self.components.tea_pot.insert(entity, ());
        self.components.solid.insert(entity, ());
        entity
    }

    pub fn spawn_tea(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Tea);
        self.components.item.insert(entity, Item::Tea);
        entity
    }

    pub fn spawn_gumboots(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Gumboots);
        self.components
            .equipment
            .insert(entity, Equipment::Gumboots);
        entity
    }

    pub fn spawn_umbrella(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Umbrella);
        self.components
            .equipment
            .insert(entity, Equipment::Umbrella);
        entity
    }

    pub fn spawn_shovel(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Shovel);
        self.components.equipment.insert(entity, Equipment::Shovel);
        entity
    }

    pub fn spawn_map(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Map);
        self.components.equipment.insert(entity, Equipment::Map);
        entity
    }

    pub fn spawn_weather_report(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::WeatherReport);
        self.components
            .equipment
            .insert(entity, Equipment::WeatherReport);
        entity
    }

    pub const LANTERN_LIGHT: Light = Light {
        colour: Rgb24::new(255, 255, 185),
        vision_distance: Circle::new_squared(18),
        diminish: Rational {
            numerator: 1,
            denominator: 10,
        },
    };

    pub fn spawn_lantern(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Lantern);
        self.components.equipment.insert(entity, Equipment::Lantern);
        self.components.light.insert(entity, Self::LANTERN_LIGHT);
        entity
    }

    pub fn spawn_crowbar(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Crowbar);
        self.components.equipment.insert(entity, Equipment::Crowbar);
        entity
    }
}
