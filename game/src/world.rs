use crate::{
    components::{Components, DoorState, EntityData, Tile},
    realtime::RealtimeComponents,
    spatial::{Location, SpatialTable},
    visibility::Light,
};
use entity_table::{Entity, EntityAllocator};
use grid_2d::{Coord, Size};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub realtime_components: RealtimeComponents,
    pub spatial_table: SpatialTable,
}

impl World {
    pub fn new(size: Size) -> Self {
        let entity_allocator = EntityAllocator::default();
        let components = Components::default();
        let spatial_table = SpatialTable::new(size);
        let realtime_components = RealtimeComponents::default();
        Self {
            entity_allocator,
            components,
            realtime_components,
            spatial_table,
        }
    }

    pub fn size(&self) -> Size {
        self.spatial_table.grid_size()
    }

    pub fn get_opacity_at_coord(&self, coord: Coord) -> u8 {
        self.spatial_table
            .layers_at(coord)
            .and_then(|c| c.feature)
            .and_then(|e| self.components.opacity.get(e).cloned())
            .unwrap_or(0)
    }

    pub fn all_lights_by_coord<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &'a Light)> {
        self.components
            .light
            .iter()
            .filter_map(move |(entity, light)| {
                self.spatial_table
                    .coord_of(entity)
                    .map(|coord| (coord, light))
            })
    }

    pub fn insert_entity_data(&mut self, location: Location, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table.update(entity, location).unwrap();
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    pub fn is_floor_at_coord(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_table.layers_at(coord) {
            if let Some(entity) = spatial_cell.floor {
                self.components.tile.get(entity) == Some(&Tile::Floor)
                    || self.components.tile.get(entity) == Some(&Tile::Ground)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_wall_at_coord(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_table.layers_at(coord) {
            if let Some(entity) = spatial_cell.feature {
                if let Some(tile) = self.components.tile.get(entity) {
                    match tile {
                        Tile::Wall
                        | Tile::RuinsWall
                        | Tile::Window(_)
                        | Tile::DoorClosed(_)
                        | Tile::DoorOpen(_) => return true,
                        _ => (),
                    }
                }
            }
        }
        false
    }

    pub fn should_hide_rain(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_table.layers_at(coord) {
            if let Some(entity) = spatial_cell.floor {
                let is_floor = self.components.tile.get(entity) == Some(&Tile::Floor);
                let ground_below = if let Some(spatial_cell) =
                    self.spatial_table.layers_at(coord + Coord { x: 0, y: 1 })
                {
                    if let Some(entity) = spatial_cell.floor {
                        self.components.tile.get(entity) == Some(&Tile::Ground)
                    } else {
                        false
                    }
                } else {
                    false
                };
                is_floor && !ground_below
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn entity_coord(&self, entity: Entity) -> Option<Coord> {
        self.spatial_table.coord_of(entity)
    }

    pub fn open_door(&mut self, door: Entity) {
        self.components.solid.remove(door);
        self.components.opacity.remove(door);
        self.components.door_state.insert(door, DoorState::Open);
        let axis = match self
            .components
            .tile
            .get(door)
            .expect("door lacks tile component")
        {
            Tile::DoorClosed(axis) | Tile::DoorOpen(axis) => *axis,
            _ => panic!("unexpected tile on door"),
        };
        self.components.tile.insert(door, Tile::DoorOpen(axis));
    }

    pub fn close_door(&mut self, door: Entity) {
        self.components.solid.insert(door, ());
        self.components.opacity.insert(door, 255);
        self.components.door_state.insert(door, DoorState::Closed);
        let axis = match self
            .components
            .tile
            .get(door)
            .expect("door lacks tile component")
        {
            Tile::DoorClosed(axis) | Tile::DoorOpen(axis) => *axis,
            _ => panic!("unexpected tile on door"),
        };
        self.components.tile.insert(door, Tile::DoorClosed(axis));
    }

    pub fn flatten_grass(&mut self, grass: Entity) {
        self.components.opacity.remove(grass);
        self.components.tile.insert(grass, Tile::FlatGrass);
    }

    pub fn turn_lamps_on(&mut self) {
        for lamp in self.components.lamp.entities() {
            self.components.light.insert(lamp, Self::LAMP_LIGHT);
            self.components.tile.insert(lamp, Tile::Lamp);
        }
        for house_light in self.components.house_light.entities() {
            self.components.light.insert(house_light, Self::HOUSE_LIGHT);
        }
    }

    pub fn turn_lamps_off(&mut self) {
        for lamp in self.components.lamp.entities() {
            self.components.light.remove(lamp);
            self.components.tile.insert(lamp, Tile::LampOff);
        }
        for house_light in self.components.house_light.entities() {
            self.components.light.remove(house_light);
        }
    }

    pub fn flood<R: Rng>(&mut self, n: usize, rng: &mut R) {
        let mut entities_by_height = self
            .components
            .height
            .iter()
            .filter(|(a, _)| self.components.ground.contains(*a))
            .map(|(a, &b)| (a, b))
            .collect::<Vec<_>>();
        entities_by_height.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for &(entity, _) in entities_by_height.iter().take(n) {
            self.become_water(entity, rng);
        }
        for &(entity, _) in entities_by_height.iter().skip(n) {
            self.components.tile.insert(entity, Tile::Ground);
            self.components.realtime.remove(entity);
        }
    }
}
