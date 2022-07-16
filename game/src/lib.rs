use gridbugs::{
    direction::CardinalDirection,
    grid_2d::{Coord, Grid, Size},
    rgb_int::Rgb24,
    shadowcast::{self, Context as ShadowcastContext},
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod components;
mod realtime;
mod spatial;
mod spawn;
mod terrain;
mod visibility;
pub mod witness;
mod world;

use components::EntityData;
pub use components::{DoorState, Equipment, Item, Tile};
pub use gridbugs::entity_table::Entity;
use realtime::AnimationContext;
pub use spatial::Layer;
use spatial::Location;
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
    pub fn err_msg<T>(s: &str) -> Result<T, Self> {
        Err(Self::Message(s.to_string()))
    }
    fn err_cant_walk_there<T>() -> Result<T, Self> {
        Self::err_msg("You can't walk there!")
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Equipped {
    pub shovel: bool,
    pub map: bool,
    pub weather_report: bool,
    pub umbrella: bool,
    pub gumboots: bool,
    pub crowbar: bool,
    pub lantern: bool,
}

impl Equipped {
    fn all() -> Self {
        Equipped {
            shovel: true,
            map: true,
            weather_report: true,
            umbrella: true,
            gumboots: true,
            crowbar: true,
            lantern: true,
        }
    }
}

mod motivation {
    use super::RainLevel;

    pub const SLEEP: i32 = 250;
    pub const LAKE: i32 = 250;
    pub const TEA: i32 = 250;
    pub const FLOWER: i32 = 250;

    pub fn chair(rain_level: RainLevel) -> i32 {
        match rain_level {
            RainLevel::Light => 125,
            RainLevel::Medium => 250,
            RainLevel::Heavy => 500,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MotivationModifier {
    PassageOfTime,
    OutsideInRain(RainLevel),
    UnderTree,
    InFloodWater,
    Tired,
    OnSteppingStone,
    InTheDark,
    FlattenedGrass,
    Gumboots,
    Umbrella,
}

impl MotivationModifier {
    pub fn value(&self) -> i32 {
        match self {
            Self::PassageOfTime => -1,
            Self::OutsideInRain(RainLevel::Light) => -2,
            Self::OutsideInRain(RainLevel::Medium) => -3,
            Self::OutsideInRain(RainLevel::Heavy) => -4,
            Self::UnderTree => 2,
            Self::Umbrella => 2,
            Self::InFloodWater => -8,
            Self::OnSteppingStone => 4,
            Self::Gumboots => 4,
            Self::Tired => -5,
            Self::InTheDark => -10,
            Self::FlattenedGrass => 1,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::PassageOfTime => "Passage of Time",
            Self::OutsideInRain(RainLevel::Light) => "Outside in Light Rain",
            Self::OutsideInRain(RainLevel::Medium) => "Outside in Medium Rain",
            Self::OutsideInRain(RainLevel::Heavy) => "Outside in Heavy Rain",
            Self::UnderTree => "Under a Tree",
            Self::InFloodWater => "In Flood Water",
            Self::OnSteppingStone => "On Stepping Stone",
            Self::Tired => "Tired",
            Self::InTheDark => "In the Dark",
            Self::FlattenedGrass => "Flattened some Grass",
            Self::Gumboots => "Gumboots",
            Self::Umbrella => "Umbrella",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TopographyCell {
    Height(f64),
    Water,
    Unknown,
    Player,
    Ruins,
    Flowers,
    Tea,
    Cabin,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct RainSchedule {
    per_day: Vec<Vec<RainLevel>>,
}

impl RainSchedule {
    fn new<R: Rng>(rng: &mut R) -> Self {
        use RainLevel::*;
        let mut per_day = vec![
            vec![Light, Light, Light, Light, Light, Light],
            vec![Medium, Medium, Light, Light, Light, Light],
            vec![Medium, Medium, Medium, Medium, Light, Light],
            vec![Heavy, Medium, Medium, Medium, Light, Light],
            vec![Heavy, Heavy, Medium, Medium, Medium, Medium],
            vec![Heavy, Heavy, Heavy, Heavy, Medium, Medium],
        ];
        for v in &mut per_day {
            v.shuffle(rng);
        }
        Self { per_day }
    }

    fn at_time(&self, time: Time) -> RainLevel {
        self.per_day
            .get(time.day() as usize)
            .and_then(|a| a.get(time.hour() as usize / 4).cloned())
            .unwrap_or(RainLevel::Heavy)
    }

    pub fn get(&self, day: usize, time: usize) -> RainLevel {
        self.per_day[day][time]
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

#[derive(Serialize, Deserialize, Default)]
struct MotivationFlags {
    lake: bool,
    chair: bool,
    tea: bool,
    flower: bool,
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
    num_flooded: f64,
    rng: Isaac64Rng,
    last_sleep: Option<u32>,
    motivation: i32,
    last_motivation_modifiers: Vec<MotivationModifier>,
    motivation_flags: MotivationFlags,
    player_item: Option<EntityData>,
    player_lantern: bool,
    player_pushing: bool,
    flattened_grass: bool,
    equipped: Equipped,
    first: bool,
    cabin_direction: CardinalDirection,
}

impl Game {
    const INITIAL_MOTIVATION: i32 = 1000;
    pub const MAX_MOTIVATION: i32 = 1000;
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::from_rng(base_rng).unwrap();
        let player_data = components::make_player();
        let (
            Terrain {
                world,
                player,
                cabin_direction,
            },
            equipped,
        ) = if config.debug {
            (
                terrain::from_str(include_str!("demo_terrain.txt"), player_data, &mut rng),
                Equipped::all(),
            )
        } else {
            (
                terrain::generate(player_data, &mut rng),
                Equipped::default(),
            )
        };
        let visibility_grid = VisibilityGrid::new(world.size());
        let mut game = Self {
            visibility_grid,
            shadowcast_context: ShadowcastContext::default(),
            world,
            player,
            animation_context: AnimationContext::default(),
            time: Time::new(0, 23, 18, 00),
            rain_schedule: RainSchedule::new(&mut rng),
            num_flooded: 0.,
            rng,
            last_sleep: None,
            motivation: Self::INITIAL_MOTIVATION,
            last_motivation_modifiers: Vec::new(),
            motivation_flags: MotivationFlags::default(),
            player_item: None,
            player_lantern: false,
            player_pushing: false,
            flattened_grass: false,
            equipped,
            first: true,
            cabin_direction,
        };
        game.after_turn(0, config);
        game.update_motivation();
        game.motivation = Self::INITIAL_MOTIVATION;
        game
    }

    pub fn pushing(&self) -> bool {
        self.player_pushing
    }

    fn is_won(&self) -> bool {
        self.time.day() > 5
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

    pub fn rain_schedule(&self) -> RainSchedule {
        self.rain_schedule.clone()
    }

    pub fn topography_grid(&self) -> Grid<TopographyCell> {
        let mut flowers = false;
        let mut tea = false;
        Grid::new_fn(self.world_size(), |coord| {
            let layers = self.world.spatial_table.layers_at_checked(coord);
            if let Some(character) = layers.character {
                if self.world.components.player.contains(character) {
                    return TopographyCell::Player;
                }
            }
            if let Some(feature) = layers.feature {
                if let Some(tile) = self.world.components.tile.get(feature) {
                    match tile {
                        Tile::Altar => return TopographyCell::Ruins,
                        Tile::Bed => return TopographyCell::Cabin,
                        _ => (),
                    }
                }
            }
            if let Some(item) = layers.item {
                if let Some(tile) = self.world.components.tile.get(item) {
                    match tile {
                        Tile::Tea => {
                            if !tea {
                                tea = true;
                                return TopographyCell::Tea;
                            }
                        }
                        Tile::Flower => {
                            if !flowers {
                                flowers = true;
                                return TopographyCell::Flowers;
                            }
                        }
                        _ => (),
                    }
                }
            }
            if let Some(floor) = layers.floor {
                if self.world.components.water.contains(floor) {
                    TopographyCell::Water
                } else if let Some(&height) = self.world.components.height.get(floor) {
                    TopographyCell::Height(height)
                } else {
                    TopographyCell::Unknown
                }
            } else {
                TopographyCell::Unknown
            }
        })
    }

    pub fn motivation(&self) -> i32 {
        self.motivation
    }

    fn update_motivation_mod(&mut self) {
        let player_coord = self.player_coord();
        self.last_motivation_modifiers.clear();
        self.last_motivation_modifiers
            .push(MotivationModifier::PassageOfTime);
        if !self.should_hide_rain(player_coord) {
            self.last_motivation_modifiers
                .push(MotivationModifier::OutsideInRain(self.rain_level()));
            if self.equipped.umbrella {
                self.last_motivation_modifiers
                    .push(MotivationModifier::Umbrella);
            } else if self.is_player_next_to_tree() {
                self.last_motivation_modifiers
                    .push(MotivationModifier::UnderTree);
            }
        }
        if self.is_player_in_flood_water() {
            self.last_motivation_modifiers
                .push(MotivationModifier::InFloodWater);
            if self.equipped.gumboots {
                self.last_motivation_modifiers
                    .push(MotivationModifier::Gumboots);
            } else if self.is_player_on_stepping_stone() {
                self.last_motivation_modifiers
                    .push(MotivationModifier::OnSteppingStone);
            }
        }
        if let Some(last_sleep) = self.last_sleep {
            if self.time.seconds - last_sleep > 3600 * 20 {
                self.last_motivation_modifiers
                    .push(MotivationModifier::Tired);
            }
        } else {
            if self.time.seconds > Time::new(1, 1, 0, 0).seconds {
                self.last_motivation_modifiers
                    .push(MotivationModifier::Tired);
            }
        }
        if let Some(cell) = self.visibility_grid.get_cell(player_coord) {
            if cell.light_colour().max_channel() < 112 {
                self.last_motivation_modifiers
                    .push(MotivationModifier::InTheDark);
            }
        }
        if self.flattened_grass {
            self.flattened_grass = false;
            self.last_motivation_modifiers
                .push(MotivationModifier::FlattenedGrass);
        }
    }

    fn update_motivation(&mut self) {
        self.update_motivation_mod();
        for m in &self.last_motivation_modifiers {
            self.motivation = (self.motivation + m.value()).min(Self::MAX_MOTIVATION);
        }
    }

    fn increase_motivation(&mut self, by: i32) {
        self.motivation = (self.motivation + by).min(Self::MAX_MOTIVATION);
    }

    pub fn last_motivation_modifiers(&self) -> &[MotivationModifier] {
        &self.last_motivation_modifiers
    }

    fn is_player_next_to_tree(&self) -> bool {
        let player_coord = self.player_coord();
        for d in CardinalDirection::all() {
            if let Some(feature) = self
                .world
                .spatial_table
                .layers_at_checked(player_coord + d.coord())
                .feature
            {
                if self.world.components.tree.contains(feature) {
                    return true;
                }
            }
        }
        false
    }

    fn is_player_in_flood_water(&self) -> bool {
        let player_coord = self.player_coord();
        if let Some(floor) = self
            .world
            .spatial_table
            .layers_at_checked(player_coord)
            .floor
        {
            if self.world.components.water.contains(floor) {
                return true;
            }
        }
        false
    }

    fn is_player_on_stepping_stone(&self) -> bool {
        let player_coord = self.player_coord();
        let cell = self.world.spatial_table.layers_at_checked(player_coord);
        if let Some(floor) = cell.floor {
            if self.world.components.water.contains(floor) {
                if let Some(item) = cell.item {
                    if self.world.components.rock.contains(item) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn player_item(&self) -> Option<Item> {
        self.player_item.as_ref().map(|i| i.item.unwrap())
    }

    pub fn player_lantern(&self) -> bool {
        self.player_lantern
    }

    pub fn tick(
        &mut self,
        _since_previous: Duration,
        _config: &Config,
        running: witness::Running,
    ) -> Witness {
        self.animation_context.tick(&mut self.world);
        if self.first {
            self.first = false;
            let direction = match self.cabin_direction {
                CardinalDirection::North => "north",
                CardinalDirection::East => "east",
                CardinalDirection::South => "south",
                CardinalDirection::West => "west",
            };
            let text = format!(
                "You've booked five days at a cabin in the forest. You arrive, exhausted, looking forward to falling asleep to the sound of rain. You see the lights of the cabin through the trees to the {}.", direction);
            running.prompt(text)
        } else {
            running.into_witness()
        }
    }

    const FLOOD_STEP: usize = 1200;

    fn after_turn(&mut self, time_delta: u32, config: &Config) {
        let old_time = self.time;
        self.time.seconds += time_delta;
        if old_time.is_night() && !self.time.is_night() {
            self.world.turn_lamps_off();
        } else if !old_time.is_night() && self.time.is_night() {
            self.world.turn_lamps_on();
        }
        let (mut player_light_colour, mut player_light_distance) = match self.time.hour() {
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
        if self.player_lantern {
            player_light_colour = player_light_colour.saturating_add(World::LANTERN_LIGHT.colour);
            if player_light_distance < World::LANTERN_LIGHT.vision_distance.distance_squared() {
                player_light_distance = World::LANTERN_LIGHT.vision_distance.distance_squared();
            }
        }
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
        self.num_flooded += (Self::FLOOD_STEP as f64 / 86400.) * time_delta as f64;
        self.world
            .flood(self.num_flooded.floor() as usize, &mut self.rng);
        if old_time.day() != self.time.day() {
            self.motivation_flags = MotivationFlags::default();
        }
        for _ in 0..(time_delta / Self::TURN_TIME) {
            self.update_motivation();
        }
        self.update_visibility(config);
    }

    fn to_push(&self, start: Coord, direction: CardinalDirection) -> Vec<Entity> {
        let mut ret = vec![];
        let mut coord = start;
        loop {
            if let Some(layers) = self.world.spatial_table.layers_at(coord) {
                if let Some(item) = layers.item {
                    if self.world.components.push.contains(item) {
                        ret.push(item);
                        coord += direction.coord();
                        continue;
                    }
                } else if let Some(feature) = layers.feature {
                    if self.world.components.solid.contains(feature) {
                        ret.clear();
                        break;
                    }
                }
                break;
            } else {
                ret.clear();
                break;
            }
        }
        ret.reverse();
        ret
    }

    pub fn equipped(&self) -> &Equipped {
        &self.equipped
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
        if let Some(&layers) = self.world.spatial_table.layers_at(destination) {
            if let Some(item) = layers.item {
                if let Some(equipment) = self.world.components.equipment.get(item) {
                    match equipment {
                        Equipment::Shovel => self.equipped.shovel = true,
                        Equipment::Map => self.equipped.map = true,
                        Equipment::WeatherReport => self.equipped.weather_report = true,
                        Equipment::Umbrella => self.equipped.umbrella = true,
                        Equipment::Gumboots => self.equipped.gumboots = true,
                        Equipment::Crowbar => self.equipped.crowbar = true,
                        Equipment::Lantern => {
                            self.equipped.lantern = true;
                            self.player_lantern = true;
                        }
                    }
                    let text = match equipment {
                        Equipment::Shovel => "You equip the shovel. You can now dig ditches by pressing 'e'.",
                        Equipment::Map => "You equip the topographic map. View it by pressing 'm'.",
                        Equipment::WeatherReport => "You equip the weather report. View it by pressing 'r'.",
                        Equipment::Umbrella => "You equip the umbrella. Motivation loss by rain is reduced.",
                        Equipment::Gumboots => "You equip gumboots. Motivation loss by flood water is reduced.",
                        Equipment::Crowbar => "You equip the crowbar. You can now push rocks. Toggle pushing mode by pressing 'p'.",
                        Equipment::Lantern => "You equip the lantern. Toggle the light by pressing 'f'.",
                    };
                    self.world.components.remove_entity(item);
                    self.world.spatial_table.remove(item);
                    return (running.prompt(format!("{}", text)), Ok(()));
                }
            }
            if self.player_pushing {
                if let Some(item) = layers.item {
                    if self.world.components.push.contains(item) {
                        for e in self.to_push(destination, direction) {
                            let coord = self.world.spatial_table.coord_of(e).unwrap();
                            let _ = self
                                .world
                                .spatial_table
                                .update_coord(e, coord + direction.coord());
                        }
                    }
                }
            }
            if let Some(floor) = layers.floor {
                if self.world.components.lake.contains(floor) {
                    return (
                        running.into_witness(),
                        ActionError::err_msg("Refusing to walk into the lake"),
                    );
                }
            }
            if let Some(feature) = layers.feature {
                if self.world.components.chair.contains(feature) {
                    if self.motivation_flags.chair {
                        return (
                            running.prompt(format!("You've already sat in your chair today.")),
                            Ok(()),
                        );
                    } else {
                        self.motivation_flags.chair = true;
                        self.increase_motivation(motivation::chair(self.rain_level()));
                        let level = match self.rain_level() {
                            RainLevel::Light => "light",
                            RainLevel::Medium => "medium",
                            RainLevel::Heavy => "heavy",
                        };
                        return (running.prompt(format!("You get comfortable in the cozy chair and enjoy the {} rain.\n\nMotivation increased by {}.", level, motivation::chair(self.rain_level()))), Ok(()));
                    }
                }
                if self.world.components.altar.contains(feature) {
                    if let Some(item) = self.player_item.as_ref() {
                        if item.item.unwrap() == Item::Flower {
                            if self.motivation_flags.flower {
                                return (
                                    running.prompt(format!(
                                        "You've already placed a flower here today."
                                    )),
                                    Ok(()),
                                );
                            } else {
                                self.player_item = None;
                                self.motivation_flags.flower = true;
                                self.increase_motivation(motivation::FLOWER);
                                return (running.prompt(format!("You place a flower on the long-abandoned altar.\n\nMotivation increased by {}.", motivation::FLOWER)), Ok(()));
                            }
                        } else {
                            return (
                                running.prompt(format!("An altar. You could leave an offering...")),
                                Ok(()),
                            );
                        }
                    } else {
                        return (
                            running.prompt(format!("An altar. You could leave an offering...")),
                            Ok(()),
                        );
                    }
                }
                if self.world.components.tea_pot.contains(feature) {
                    if let Some(item) = self.player_item.as_ref() {
                        if item.item.unwrap() == Item::Tea {
                            if self.motivation_flags.tea {
                                return (
                                    running.prompt(format!("You've already had tea today!")),
                                    Ok(()),
                                );
                            } else {
                                self.player_item = None;
                                self.motivation_flags.tea = true;
                                self.increase_motivation(motivation::TEA);
                                return (running.prompt(format!("Mmm...a nice relaxing cup of tea.\n\nMotivation increased by {}.", motivation::TEA)), Ok(()));
                            }
                        } else {
                            return (
                                running.prompt(format!(
                                "A teapot. You could make tea, if only you had some tea leaves..."
                            )),
                                Ok(()),
                            );
                        }
                    } else {
                        return (
                            running.prompt(format!(
                                "A teapot. You could make tea, if only you had some tea leaves..."
                            )),
                            Ok(()),
                        );
                    }
                }
                if self.world.components.bulletin_board.contains(feature) {
                    return (
                        running.prompt(format!("\"Enjoy your stay in our cabin!\"")),
                        Ok(()),
                    );
                }
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
                    if self.world.flatten_grass(feature) {
                        self.flattened_grass = true;
                    }
                }
            }
            let _ = self
                .world
                .spatial_table
                .update_coord(self.player, destination);
            if let Some(floor) = self
                .world
                .spatial_table
                .layers_at_checked(destination)
                .floor
            {
                if self.world.components.end_of_pier.contains(floor) && !self.motivation_flags.lake
                {
                    self.motivation_flags.lake = true;
                    self.increase_motivation(motivation::LAKE);
                    return (running.prompt(format!("Contemplating the vastness of this lake puts your life into perspective.\n\nMotivation increased by {}.", motivation::LAKE)), Ok(()));
                }
            }
        } else {
            return (running.into_witness(), ActionError::err_cant_walk_there());
        }
        (running.into_witness(), Ok(()))
    }

    const TURN_TIME: u32 = 120;

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
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        (witness, result)
    }

    pub fn player_walk_until_collide(
        &mut self,
        direction: CardinalDirection,
        config: &Config,
        mut running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        let ret = loop {
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
                if let Some(floor) = layers.floor {
                    if self.world.components.lake.contains(floor) {
                        break (running.into_witness(), Ok(()));
                    }
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
        };
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        ret
    }

    pub fn player_wait(&mut self, config: &Config, running: witness::Running) -> Witness {
        self.after_turn(Self::TURN_TIME, config);
        if self.is_won() {
            return Witness::Win;
        }
        if self.motivation <= 0 {
            return Witness::GameOver;
        }
        running.into_witness()
    }

    pub fn player_wait_long(&mut self, config: &Config, running: witness::Running) -> Witness {
        self.after_turn(3600, config);
        if self.is_won() {
            return Witness::Win;
        }
        if self.motivation <= 0 {
            return Witness::GameOver;
        }

        running.into_witness()
    }

    pub fn player_sleep(&mut self, config: &Config, sleep: witness::Sleep) -> Witness {
        let motivation = self.motivation;
        self.after_turn(3600 * 8, config);
        self.last_sleep = Some(self.time.seconds);
        self.update_motivation_mod();
        self.motivation = motivation; // don't lose motivation while asleep
        self.increase_motivation(motivation::SLEEP);
        if self.is_won() {
            return Witness::Win;
        }
        sleep.prompt(format!(
            "You sleep for 8 hours.\n\nMotivation increased by {}.",
            motivation::SLEEP
        ))
    }

    pub fn player_get(
        &mut self,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        let player_coord = self
            .world
            .spatial_table
            .coord_of(self.player)
            .expect("can't get coord of player");
        let ret = if let Some(item) = self
            .world
            .spatial_table
            .layers_at_checked(player_coord)
            .item
        {
            let item_data = self.world.components.remove_entity_data(item);
            self.world.spatial_table.remove(item);
            let message = if let Some(current_item) = self.player_item.take() {
                let current_item_item = current_item.item.unwrap();
                let entity = self.world.entity_allocator.alloc();
                self.world
                    .components
                    .insert_entity_data(entity, current_item);
                let _ = self.world.spatial_table.update(
                    entity,
                    Location {
                        coord: player_coord,
                        layer: Some(Layer::Item),
                    },
                );
                format!(
                    "You put down the {} and pick up the {}.",
                    current_item_item.to_string(),
                    item_data.item.unwrap().to_string()
                )
            } else {
                format!("You pick up the {}.", item_data.item.unwrap().to_string())
            };
            self.player_item = Some(item_data);
            (running.prompt(message), Ok(()))
        } else {
            if let Some(current_item) = self.player_item.take() {
                let item = current_item.item.unwrap();
                let entity = self.world.entity_allocator.alloc();
                self.world
                    .components
                    .insert_entity_data(entity, current_item);
                let _ = self.world.spatial_table.update(
                    entity,
                    Location {
                        coord: player_coord,
                        layer: Some(Layer::Item),
                    },
                );
                (
                    running.prompt(format!("You put down the {}.", item.to_string())),
                    Ok(()),
                )
            } else {
                return (
                    running.into_witness(),
                    ActionError::err_msg("There is no item here!"),
                );
            }
        };
        self.after_turn(Self::TURN_TIME, config);
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        ret
    }

    pub fn player_toggle_lantern(
        &mut self,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        if !self.equipped.lantern {
            return (
                running.into_witness(),
                ActionError::err_msg("You don't have the lantern equipped!"),
            );
        }
        self.player_lantern = !self.player_lantern;
        self.after_turn(0, config);
        self.update_motivation_mod(); // remove the "InTheDark" modifier
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        (running.into_witness(), Ok(()))
    }

    pub fn player_toggle_pushing(
        &mut self,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        if !self.equipped.crowbar {
            return (
                running.into_witness(),
                ActionError::err_msg("You don't have the crowbar equipped!"),
            );
        }
        self.player_pushing = !self.player_pushing;
        self.after_turn(0, config);
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        (running.into_witness(), Ok(()))
    }

    pub fn player_dig(
        &mut self,
        config: &Config,
        running: witness::Running,
    ) -> (Witness, Result<(), ActionError>) {
        if !self.equipped.shovel {
            return (
                running.into_witness(),
                ActionError::err_msg("You don't have the shovel equipped!"),
            );
        }
        self.world.dig(self.player_coord());
        self.after_turn(Self::TURN_TIME, config);
        if self.is_won() {
            return (Witness::Win, Ok(()));
        }
        if self.motivation <= 0 {
            return (Witness::GameOver, Ok(()));
        }
        (running.into_witness(), Ok(()))
    }
}
