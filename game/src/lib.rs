use direction::CardinalDirection;
use grid_2d::{Coord, Size};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::Context as ShadowcastContext;
use std::time::Duration;

mod components;
mod realtime;
mod spatial;
mod spawn;
mod terrain;
mod visibility;
pub mod witness;
mod world;

pub use components::{DoorState, Tile};
pub use entity_table::Entity;
use realtime::AnimationContext;
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
    animation_context: AnimationContext,
}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::from_rng(base_rng).unwrap();
        let player_data = components::make_player();
        let Terrain { world, player } = if config.debug {
            terrain::from_str(include_str!("demo_terrain.txt"), player_data, &mut rng)
        } else {
            terrain::generate(player_data, &mut rng)
        };
        let visibility_grid = VisibilityGrid::new(world.size());
        let mut game = Self {
            visibility_grid,
            shadowcast_context: ShadowcastContext::default(),
            world,
            player,
            animation_context: AnimationContext::default(),
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

    pub fn should_hide_rain(&self, coord: Coord) -> bool {
        self.world.should_hide_rain(coord)
    }

    pub fn colour_hint(&self, entity: Entity) -> Option<Rgb24> {
        self.world.components.colour_hint.get(entity).cloned()
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
        self.animation_context.tick(&mut self.world);
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
            if let Some(floor) = layers.floor {
                if self.world.components.lake.contains(floor) {
                    return (
                        running.into_witness(),
                        ActionError::err_msg("Refusing to walk into the lake"),
                    );
                }
            }
            if let Some(feature) = layers.feature {
                if self.world.components.solid.contains(feature) {
                    if self.world.components.door_state.contains(feature) {
                        self.world.open_door(feature);
                        return (running.into_witness(), Ok(()));
                    } else {
                        for d in [direction.left90(), direction.right90()] {
                            if let Some(layers) =
                                self.world.spatial_table.layers_at(destination + d.coord())
                            {
                                if let Some(feature) = layers.feature {
                                    if let Some(DoorState::Open) =
                                        self.world.components.door_state.get(feature)
                                    {
                                        self.world.close_door(feature);
                                        return (running.into_witness(), Ok(()));
                                    }
                                }
                            }
                        }
                        return (running.into_witness(), ActionError::err_cant_walk_there());
                    }
                }
                if self.world.components.grass.contains(feature) {
                    self.world.flatten_grass(feature);
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
