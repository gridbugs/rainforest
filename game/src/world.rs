use crate::{
    components::{Components, EntityData, Tile},
    spatial::{Location, SpatialTable},
    visibility::Light,
};
use entity_table::{Entity, EntityAllocator};
use grid_2d::{Coord, Size};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub spatial_table: SpatialTable,
}

impl World {
    pub fn new(size: Size) -> Self {
        let entity_allocator = EntityAllocator::default();
        let components = Components::default();
        let spatial_table = SpatialTable::new(size);
        Self {
            entity_allocator,
            components,
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

    pub fn entity_coord(&self, entity: Entity) -> Option<Coord> {
        self.spatial_table.coord_of(entity)
    }

    pub fn open_door(&mut self, door: Entity) {
        self.components.solid.remove(door);
        self.components.opacity.remove(door);
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
}
