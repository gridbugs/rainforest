use crate::{components::Tile, world::World};
use entity_table::Entity;
use grid_2d::{Coord, CoordIter, Grid, GridEnumerate, Size};
use rational::Rational;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, Context as ShadowcastContext, DirectionBitmap, InputGrid};

const AMBIENT_COL: Rgb24 = Rgb24::new_grey(31);
const VISION_DISTANCE_SQUARED: u32 = 400;
pub const VISION_DISTANCE: vision_distance::Circle =
    vision_distance::Circle::new_squared(VISION_DISTANCE_SQUARED);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Light {
    pub colour: Rgb24,
    pub vision_distance: vision_distance::Circle,
    pub diminish: Rational,
}

pub struct Visibility;

impl InputGrid for Visibility {
    type Grid = World;
    type Opacity = u8;
    fn size(&self, world: &Self::Grid) -> Size {
        world.size()
    }
    fn get_opacity(&self, world: &Self::Grid, coord: Coord) -> Self::Opacity {
        world.get_opacity_at_coord(coord)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Omniscient;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct EntityTile {
    pub entity: Entity,
    pub tile: Tile,
}

#[derive(Serialize, Deserialize)]
pub struct TileLayers {
    pub floor: Option<EntityTile>,
    pub feature: Option<EntityTile>,
    pub character: Option<EntityTile>,
    pub item: Option<EntityTile>,
}

#[derive(Serialize, Deserialize)]
pub struct VisibilityCell {
    last_seen: u64,
    last_seen_next: u64,
    last_lit: u64,
    visible_directions: DirectionBitmap,
    light_colour: Rgb24,
    tile_layers: TileLayers,
}

impl Default for VisibilityCell {
    fn default() -> Self {
        Self {
            last_seen: 0,
            last_seen_next: 0,
            last_lit: 0,
            visible_directions: DirectionBitmap::empty(),
            light_colour: Rgb24::new(0, 0, 0),
            tile_layers: TileLayers {
                floor: None,
                feature: None,
                character: None,
                item: None,
            },
        }
    }
}

impl VisibilityCell {
    pub fn tile_layers(&self) -> &TileLayers {
        &self.tile_layers
    }
    pub fn visibility(&self, count: u64) -> CellVisibility {
        if self.last_seen == count {
            let light_colour = if self.last_lit == count {
                Some(self.light_colour)
            } else {
                None
            };
            CellVisibility::CurrentlyVisibleWithLightColour(light_colour)
        } else if self.last_seen == 0 {
            CellVisibility::NeverVisible
        } else {
            CellVisibility::PreviouslyVisible
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VisibilityGrid {
    grid: Grid<VisibilityCell>,
    count: u64,
}

pub enum CellVisibility {
    NeverVisible,
    PreviouslyVisible,
    CurrentlyVisibleWithLightColour(Option<Rgb24>),
}

impl VisibilityGrid {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            count: 1,
        }
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn enumerate(&self) -> GridEnumerate<VisibilityCell> {
        self.grid.enumerate()
    }
    pub fn get_cell(&self, coord: Coord) -> Option<&VisibilityCell> {
        self.grid.get(coord)
    }
    pub fn cell_visibility(&self, coord: Coord) -> CellVisibility {
        if let Some(cell) = self.grid.get(coord) {
            if cell.last_seen == self.count {
                let light_colour = if cell.last_lit == self.count {
                    Some(cell.light_colour)
                } else {
                    None
                };
                CellVisibility::CurrentlyVisibleWithLightColour(light_colour)
            } else if cell.last_seen == 0 {
                CellVisibility::NeverVisible
            } else {
                CellVisibility::PreviouslyVisible
            }
        } else {
            CellVisibility::NeverVisible
        }
    }
    pub fn is_coord_currently_visible(&self, coord: Coord) -> bool {
        match self.cell_visibility(coord) {
            CellVisibility::CurrentlyVisibleWithLightColour(Some(_)) => true,
            _ => false,
        }
    }
    pub fn is_coord_never_visible(&self, coord: Coord) -> bool {
        match self.cell_visibility(coord) {
            CellVisibility::NeverVisible => true,
            _ => false,
        }
    }
    pub fn update(
        &mut self,
        player_coord: Coord,
        world: &World,
        shadowcast_context: &mut ShadowcastContext<u8>,
        omniscient: Option<Omniscient>,
    ) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        if let Some(Omniscient) = omniscient {
            for coord in CoordIter::new(world.size()) {
                let cell = grid.get_checked_mut(coord);
                cell.last_seen_next = count;
                cell.last_seen = count;
                cell.visible_directions = DirectionBitmap::all();
                cell.last_lit = count;
                cell.light_colour = Rgb24::new_grey(255);
                let layers = world.spatial_table.layers_at_checked(coord);
                if let Some(entity) = layers.floor {
                    if let Some(&tile) = world.components.tile.get(entity) {
                        cell.tile_layers.floor = Some(EntityTile { entity, tile });
                    }
                } else {
                    cell.tile_layers.floor = None;
                }
                if let Some(entity) = layers.feature {
                    if let Some(&tile) = world.components.tile.get(entity) {
                        cell.tile_layers.feature = Some(EntityTile { entity, tile });
                    }
                } else {
                    cell.tile_layers.feature = None;
                }
                if let Some(entity) = layers.character {
                    if let Some(&tile) = world.components.tile.get(entity) {
                        cell.tile_layers.character = Some(EntityTile { entity, tile });
                    }
                } else {
                    cell.tile_layers.character = None;
                }
                if let Some(entity) = layers.item {
                    if let Some(&tile) = world.components.tile.get(entity) {
                        cell.tile_layers.item = Some(EntityTile { entity, tile });
                    }
                } else {
                    cell.tile_layers.item = None;
                }
            }
        } else {
            shadowcast_context.for_each_visible(
                player_coord,
                &Visibility,
                world,
                VISION_DISTANCE,
                255,
                |coord, visible_directions, _visibility| {
                    let cell = grid.get_checked_mut(coord);
                    cell.last_seen_next = count;
                    cell.visible_directions = visible_directions;
                    cell.last_lit = count;
                    cell.light_colour = AMBIENT_COL;
                },
            );
        }
        for (light_coord, light) in world.all_lights_by_coord() {
            shadowcast_context.for_each_visible(
                light_coord,
                &Visibility,
                world,
                light.vision_distance,
                255,
                |cell_coord, visible_directions, visibility| {
                    let cell = grid.get_checked_mut(cell_coord);
                    if cell.last_seen_next == count
                        && !(visible_directions & cell.visible_directions).is_empty()
                    {
                        let distance_squared = (light_coord - cell_coord).magnitude2();
                        let inverse_light_intensity = (distance_squared * light.diminish.numerator)
                            / light.diminish.denominator;
                        let light_colour = light.colour.scalar_div(inverse_light_intensity.max(1));
                        cell.light_colour = cell
                            .light_colour
                            .saturating_add(light_colour.normalised_scalar_mul(visibility));
                        if cell.light_colour.saturating_channel_total() > 31 {
                            let layers = world.spatial_table.layers_at_checked(cell_coord);
                            if let Some(entity) = layers.floor {
                                if let Some(&tile) = world.components.tile.get(entity) {
                                    cell.tile_layers.floor = Some(EntityTile { entity, tile });
                                }
                            } else {
                                cell.tile_layers.floor = None;
                            }
                            if let Some(entity) = layers.feature {
                                if let Some(&tile) = world.components.tile.get(entity) {
                                    cell.tile_layers.feature = Some(EntityTile { entity, tile });
                                }
                            } else {
                                cell.tile_layers.feature = None;
                            }
                            if let Some(entity) = layers.character {
                                if let Some(&tile) = world.components.tile.get(entity) {
                                    cell.tile_layers.character = Some(EntityTile { entity, tile });
                                }
                            } else {
                                cell.tile_layers.character = None;
                            }
                            if let Some(entity) = layers.item {
                                if let Some(&tile) = world.components.tile.get(entity) {
                                    cell.tile_layers.item = Some(EntityTile { entity, tile });
                                }
                            } else {
                                cell.tile_layers.item = None;
                            }
                            cell.last_seen = count;
                        }
                    }
                },
            );
        }
    }
}
