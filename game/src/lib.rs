use direction::CardinalDirection;
use grid_2d::{Coord, Grid, Size};
use rand::{seq::SliceRandom, Rng, SeedableRng};
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

#[derive(Debug, Clone, Copy)]
pub enum TopographyCell {
    Height(f64),
    Water,
    Unknown,
    Player,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RainLevel {
    Light,
    Medium,
    Heavy,
}

impl RainLevel {
    pub fn to_string(&self) -> String {
        match self {
            Self::Light => "Light Rain",
            Self::Medium => "Medium Rain",
            Self::Heavy => "Heavy Rain",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize)]
struct RainSchedule {
    per_day: Vec<Vec<RainLevel>>,
}

impl RainSchedule {
    fn new<R: Rng>(rng: &mut R) -> Self {
        use RainLevel::*;
        let mut per_day = vec![
            vec![Light, Light, Light],
            vec![Medium, Light, Light],
            vec![Medium, Medium, Light],
            vec![Heavy, Medium, Light],
            vec![Heavy, Medium, Medium],
            vec![Heavy, Heavy, Medium],
        ];
        for v in &mut per_day {
            v.shuffle(rng);
        }
        Self { per_day }
    }

    pub fn at_time(&self, time: Time) -> RainLevel {
        self.per_day
            .get(time.day() as usize)
            .and_then(|a| a.get(time.hour() as usize / 8).cloned())
            .unwrap_or(RainLevel::Heavy)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Time {
    seconds: u32,
}

impl Time {
    pub fn new(day: u32, hour: u32, minute: u32, second: u32) -> Self {
        let seconds = (day * 86400) + (hour * 3600) + (minute * 60) + second;
        Self { seconds }
    }

    pub fn second(&self) -> u32 {
        self.seconds % 60
    }

    pub fn minute(&self) -> u32 {
        (self.seconds % 3600) / 60
    }

    pub fn hour(&self) -> u32 {
        (self.seconds % 86400) / 3600
    }

    pub fn day(&self) -> u32 {
        self.seconds / 86400
    }

    pub fn to_string(&self) -> String {
        if false {
            // 12 hr time
            let (am_pm, h) = match self.hour() {
                0 => ("am", 12),
                h @ 1..=11 => ("am", h),
                12 => ("pm", 12),
                h @ 13.. => ("pm", h - 12),
            };
            format!(
                "Day {}, {}:{:02}:{:02}{}",
                self.day(),
                h,
                self.minute(),
                self.second(),
                am_pm
            )
        } else {
            format!("Day {}, {}:{:02}", self.day(), self.hour(), self.minute(),)
        }
    }

    fn is_night(&self) -> bool {
        let h = self.hour();
        h < 5 || h > 17
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    visibility_grid: VisibilityGrid,
    shadowcast_context: ShadowcastContext<u8>,
    world: World,
    player: Entity,
    animation_context: AnimationContext,
    time: Time,
    rain_schedule: RainSchedule,
    num_flooded: usize,
    rng: Isaac64Rng,
    last_sleep: Option<u32>,
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
            time: Time::new(0, 23, 17, 30),
            rain_schedule: RainSchedule::new(&mut rng),
            num_flooded: 0,
            rng,
            last_sleep: None,
        };
        game.after_turn(0, config);
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

    pub fn contains_wall(&self, coord: Coord) -> bool {
        self.world.is_wall_at_coord(coord)
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

    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn rain_level(&self) -> RainLevel {
        self.rain_schedule.at_time(self.time)
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

    pub fn topography_grid(&self) -> Grid<TopographyCell> {
        Grid::new_fn(self.world_size(), |coord| {
            if let Some(character) = self.world.spatial_table.layers_at_checked(coord).character {
                if self.world.components.player.contains(character) {
                    return TopographyCell::Player;
                }
            }
            if let Some(floor) = self.world.spatial_table.layers_at_checked(coord).floor {
                if let Some(&height) = self.world.components.height.get(floor) {
                    TopographyCell::Height(height)
                } else if self.world.components.lake.contains(floor) {
                    TopographyCell::Water
                } else {
                    TopographyCell::Unknown
                }
            } else {
                TopographyCell::Unknown
            }
        })
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

    const FLOOD_STEP: usize = 400;

    fn after_turn(&mut self, time_delta: u32, config: &Config) {
        let old_time = self.time;
        self.time.seconds += time_delta;
        if old_time.is_night() && !self.time.is_night() {
            self.world.turn_lamps_off();
        } else if !old_time.is_night() && self.time.is_night() {
            self.world.turn_lamps_on();
        }
        let (player_light_colour, player_light_distance) = match self.time.hour() {
            0 => (Rgb24::new(64, 64, 80), 25),
            1 => (Rgb24::new(64, 64, 80), 25),
            2 => (Rgb24::new(64, 64, 80), 25),
            3 => (Rgb24::new(64, 64, 80), 25),
            4 => (Rgb24::new(64, 64, 80), 25),
            5 => (Rgb24::new(120, 100, 80), 50),
            6 => (Rgb24::new(130, 110, 100), 80),
            7 => (Rgb24::new(140, 120, 120), 120),
            8 => (Rgb24::new(150, 140, 140), 160),
            9 => (Rgb24::new(160, 160, 160), 200),
            10 => (Rgb24::new(180, 180, 180), 200),
            11 => (Rgb24::new(200, 200, 200), 200),
            12 => (Rgb24::new(200, 200, 200), 200),
            13 => (Rgb24::new(200, 200, 200), 200),
            14 => (Rgb24::new(200, 200, 200), 200),
            15 => (Rgb24::new(200, 200, 200), 200),
            16 => (Rgb24::new(200, 200, 150), 200),
            17 => (Rgb24::new(200, 150, 20), 120),
            18 => (Rgb24::new(150, 120, 80), 80),
            19 => (Rgb24::new(80, 80, 80), 25),
            20 => (Rgb24::new(80, 80, 80), 25),
            21 => (Rgb24::new(80, 80, 80), 25),
            22 => (Rgb24::new(80, 80, 80), 25),
            23 => (Rgb24::new(80, 80, 80), 25),
            _ => panic!(),
        };
        {
            let light = self
                .world
                .components
                .light
                .get_mut(self.player)
                .expect("player lacks light");
            light.colour = player_light_colour;
            light.vision_distance =
                shadowcast::vision_distance::Circle::new_squared(player_light_distance);
        }
        if old_time.day() != self.time.day() {
            self.num_flooded += Self::FLOOD_STEP;
            self.world.flood(self.num_flooded, &mut self.rng);
        }
        self.update_visibility(config);
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
                if self.world.components.bed.contains(feature) {
                    if let Some(last_sleep) = self.last_sleep {
                        if self.time.seconds - last_sleep < 8 * 3600 {
                            return (
                                running.into_witness(),
                                ActionError::err_msg("You don't feel like sleeping yet"),
                            );
                        }
                    }
                    return (running.sleep(), Ok(()));
                }
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

    const TURN_TIME: u32 = 60;

    pub fn player_walk(
        &mut self,
        direction: CardinalDirection,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        let (witness, result) = self.player_walk_inner(direction, running);
        if result.is_ok() {
            self.after_turn(Self::TURN_TIME, config);
        }
        (witness, result)
    }

    pub fn player_walk_until_collide(
        &mut self,
        direction: CardinalDirection,
        config: &Config,
        mut running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        loop {
            let player_coord = self
                .world
                .spatial_table
                .coord_of(self.player)
                .expect("can't get coord of player");
            let destination = player_coord + direction.coord();
            if let Some(layers) = self.world.spatial_table.layers_at(destination) {
                if layers.feature.is_some() {
                    break (running.into_witness(), Ok(()));
                }
            }
            let (witness, result) = self.player_walk_inner(direction, running);
            if result.is_ok() {
                self.after_turn(Self::TURN_TIME, config);
            }
            if let Witness::Running(next_running) = witness {
                running = next_running;
            } else {
                break (witness, result);
            }
        }
    }

    pub fn player_wait(&mut self, config: &Config, running: witness::Running) -> Witness {
        self.after_turn(Self::TURN_TIME, config);
        running.into_witness()
    }

    pub fn player_wait_long(&mut self, config: &Config, running: witness::Running) -> Witness {
        self.after_turn(3600, config);
        running.into_witness()
    }

    pub fn player_sleep(&mut self, config: &Config, sleep: witness::Sleep) -> Witness {
        self.after_turn(3600 * 8, config);
        self.last_sleep = Some(self.time.seconds);
        sleep.running()
    }
}
