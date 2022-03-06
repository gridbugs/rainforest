use direction::CardinalDirection;
use grid_2d::{Coord, Size};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shadowcast::Context as ShadowcastContext;
use std::time::Duration;

mod components;
mod spatial;
mod spawn;
mod terrain;
mod visibility;
pub mod witness;
mod world;

pub use components::Tile;
pub use entity_table::Entity;
pub use spatial::Layer;
use terrain::Terrain;
pub use visibility::{CellVisibility, EntityTile, Omniscient, VisibilityCell, VisibilityGrid};
use witness::Witness;
use world::World;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: bool,
    pub demo: bool,
    pub debug: bool,
}

pub enum ActionError {
    Message(String),
}

impl ActionError {
    fn err_msg<T>(s: &str) -> Result<T, Self> {
        Err(Self::Message(s.to_string()))
    }
    fn err_cant_walk_there<T>() -> Result<T, Self> {
        Self::err_msg("You can't walk there!")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    visibility_grid: VisibilityGrid,
    shadowcast_context: ShadowcastContext<u8>,
    world: World,
    player: Entity,
}

impl Game {
    pub fn new<R: Rng>(config: &Config, _base_rng: &mut R) -> Self {
        let Terrain { world, player } =
            terrain::from_str(include_str!("demo_terrain.txt"), components::make_player());
        let visibility_grid = VisibilityGrid::new(world.size());
        let mut game = Self {
            visibility_grid,
            shadowcast_context: ShadowcastContext::default(),
            world,
            player,
        };
        game.update_visibility(config);
        game
    }

    pub fn into_running_game(self, running: witness::Running) -> witness::RunningGame {
        witness::RunningGame::new(self, running)
    }

    pub fn world_size(&self) -> Size {
        self.world.size()
    }

    pub fn visibility_grid(&self) -> &VisibilityGrid {
        &self.visibility_grid
    }

    pub fn contains_floor(&self, coord: Coord) -> bool {
        self.world.is_floor_at_coord(coord)
    }

    pub fn player_coord(&self) -> Coord {
        self.world
            .spatial_table
            .coord_of(self.player)
            .expect("can't find coord of player")
    }

    fn update_visibility(&mut self, config: &Config) {
        if let Some(player_coord) = self.world.entity_coord(self.player) {
            self.visibility_grid.update(
                player_coord,
                &self.world,
                &mut self.shadowcast_context,
                if config.omniscient {
                    Some(Omniscient)
                } else {
                    None
                },
            );
        }
    }

    pub fn tick(
        &mut self,
        _since_previous: Duration,
        _config: &Config,
        running: witness::Running,
    ) -> Witness {
        running.into_witness()
    }

    pub fn player_walk_inner(
        &mut self,
        direction: CardinalDirection,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        let player_coord = self
            .world
            .spatial_table
            .coord_of(self.player)
            .expect("can't get coord of player");
        let destination = player_coord + direction.coord();
        if let Some(layers) = self.world.spatial_table.layers_at(destination) {
            if let Some(feature) = layers.feature {
                if self.world.components.solid.contains(feature) {
                    if self.world.components.door_state.contains(feature) {
                        self.world.open_door(feature);
                        return (running.into_witness(), Ok(()));
                    } else {
                        return (running.into_witness(), ActionError::err_cant_walk_there());
                    }
                }
            }
            let _ = self
                .world
                .spatial_table
                .update_coord(self.player, destination);
        } else {
            return (running.into_witness(), ActionError::err_cant_walk_there());
        }
        (running.into_witness(), Ok(()))
    }

    pub fn player_walk(
        &mut self,
        direction: CardinalDirection,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        let (witness, result) = self.player_walk_inner(direction, running);
        if result.is_ok() {
            self.update_visibility(config);
        }
        (witness, result)
    }
}
