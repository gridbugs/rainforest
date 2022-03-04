use grid_2d::Coord;
use rand::Rng;
use serde::{Deserialize, Serialize};
use shadowcast::Context as ShadowcastContext;

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
use world::World;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: bool,
    pub demo: bool,
    pub debug: bool,
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

    pub fn visibility_grid(&self) -> &VisibilityGrid {
        &self.visibility_grid
    }

    pub fn contains_floor(&self, coord: Coord) -> bool {
        self.world.is_floor_at_coord(coord)
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
}
