pub use crate::game::ActionError;
use crate::game::{self, Config, ControlFlow, TickOutput};
use gridbugs::direction::CardinalDirection;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The `Witness` type defined in this module is intended as the sole means of mutating the game
/// state. Depending on the current state of the game, different types of mutation may be valid or
/// invalid. For example, if the game is in a state where the user is expected to choose an option
/// from a menu, such as an ability to use, it is invalid for the game to receive an update which
/// moves the player character. One solution to this problem would be to have all mutating methods
/// of the `Game` type take ownership of `self` and return an `enum` encoding the different types
/// of interaction the game could expect. This would be inconvenient to clients of this code as
/// it will prevent mutating the game state through a `mut ref`. The `Witness` type encodes the
/// currently-expected interaction externally to the game's state itself, and exposes methods that
/// mutate the game state through a `mut ref`, and take ownership of, and return witness values to
/// model changes in the currently-expected updates and prevent invalid updates with the type
/// system.

/// A private unit type which prevents witnesses being minted other than in the approved ways.
/// Importantly, this type is not `Clone` or `Copy`, and neither are any witness types, similarly
/// to control the construction of witnesses.
#[derive(Debug)]
struct Private;

/// Represents the fact that the game is currently running, expecting inputs that interact with the
/// game world by manipulating the player character
#[derive(Debug)]
pub struct Running(Private);

impl Running {
    pub fn running_game(self, game: Game) -> RunningGame {
        RunningGame(game)
    }

    /// Convenience method for wrapping `self` in `Witness::Running(...)`
    pub fn into_witness(self) -> Witness {
        Witness::Running(self)
    }

    /// Helper for turning self into a prompt with a given message
    fn into_prompt_witness(self, message: String) -> Witness {
        Witness::Prompt(Prompt {
            message,
            private: self.0,
        })
    }

    fn handle_control_flow(self, cf: Option<ControlFlow>) -> Witness {
        match cf {
            None => self.into_witness(),
            Some(control_flow) => match control_flow {
                ControlFlow::Prompt(message) => self.into_prompt_witness(message),
                ControlFlow::Sleep => Witness::Sleep(Sleep(self.0)),
                ControlFlow::Win => Witness::Win,
                ControlFlow::GameOver => Witness::GameOver,
            },
        }
    }

    /// Common logic for handling the common return type of methods that update the game state
    fn handle_control_flow_result(
        self,
        cfr: Result<Option<ControlFlow>, ActionError>,
    ) -> (Witness, Result<(), ActionError>) {
        match cfr {
            Ok(maybe_control_flow) => (self.handle_control_flow(maybe_control_flow), Ok(())),
            Err(e) => (self.into_witness(), Err(e)),
        }
    }

    /// Called periodically, once per frame
    pub fn tick(self, game: &mut Game, since_previous: Duration, config: &Config) -> Witness {
        let _ = since_previous;
        let _ = config;
        match game.0.tick() {
            None => self.into_witness(),
            Some(TickOutput::Prompt(message)) => self.into_prompt_witness(message),
        }
    }

    pub fn player_dig(
        self,
        game: &mut Game,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_dig(config))
    }

    pub fn player_toggle_pushing(
        self,
        game: &mut Game,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_toggle_pushing(config))
    }

    pub fn player_toggle_lantern(
        self,
        game: &mut Game,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_toggle_lantern(config))
    }

    pub fn player_get(
        self,
        game: &mut Game,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_get(config))
    }

    pub fn player_wait(self, game: &mut Game, config: &Config) -> Witness {
        self.handle_control_flow(game.0.player_wait(config))
    }

    pub fn player_wait_long(self, game: &mut Game, config: &Config) -> Witness {
        self.handle_control_flow(game.0.player_wait_long(config))
    }

    pub fn player_walk(
        self,
        game: &mut Game,
        direction: CardinalDirection,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_walk(direction, config))
    }

    pub fn player_walk_until_collide(
        self,
        game: &mut Game,
        direction: CardinalDirection,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        self.handle_control_flow_result(game.0.player_walk_until_collide(direction, config))
    }
}

/// Represents the fact that the game is waiting for an answer about whether the player character
/// goes to sleep
#[derive(Debug)]
pub struct Sleep(Private);

impl Sleep {
    pub fn cancel(self) -> Witness {
        Witness::Running(Running(self.0))
    }

    pub fn commit(self, game: &mut Game, config: &Config) -> Witness {
        game.0.player_sleep(config);
        if game.0.is_won() {
            Witness::Win
        } else {
            Witness::Prompt(Prompt {
                message: game::prompts::sleep(),
                private: self.0,
            })
        }
    }
}

#[derive(Debug)]
pub struct Prompt {
    message: String,
    private: Private,
}

impl Prompt {
    pub fn into_witness(self) -> Witness {
        Witness::Prompt(self)
    }

    pub fn running(self) -> Witness {
        Running(self.private).into_witness()
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug)]
pub enum Witness {
    Running(Running),
    Sleep(Sleep),
    Prompt(Prompt),
    GameOver,
    Win,
}

pub fn new_game<R: Rng>(config: &Config, base_rng: &mut R) -> (Game, Running) {
    let g = game::Game::new(config, base_rng);
    (Game(g), Running(Private))
}

/// Wraps a `Game`, and can only be constructed from a `Running`, serving as proof that the wrapped
/// game is in the state represented by the `Running` witness
#[derive(Serialize, Deserialize)]
pub struct RunningGame(Game);

impl RunningGame {
    pub fn into_game(self) -> (Game, Running) {
        (self.0, Running(Private))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game(game::Game);

mod game_interface {
    use super::Game;
    use crate::{
        components::Item,
        game::{Equipped, MotivationModifier, RainLevel, RainSchedule, Time, TopographyCell},
        visibility::VisibilityGrid,
    };
    use gridbugs::{
        coord_2d::{Coord, Size},
        entity_table::Entity,
        grid_2d::Grid,
        rgb_int::Rgb24,
    };

    impl Game {
        pub fn contains_wall(&self, coord: Coord) -> bool {
            self.0.contains_wall(coord)
        }

        pub fn colour_hint(&self, entity: Entity) -> Option<Rgb24> {
            self.0.colour_hint(entity)
        }

        pub fn visibility_grid(&self) -> &VisibilityGrid {
            self.0.visibility_grid()
        }

        pub fn should_hide_rain(&self, coord: Coord) -> bool {
            self.0.should_hide_rain(coord)
        }

        pub fn world_size(&self) -> Size {
            self.0.world_size()
        }

        pub fn topography_grid(&self) -> Grid<TopographyCell> {
            self.0.topography_grid()
        }

        pub fn rain_schedule(&self) -> RainSchedule {
            self.0.rain_schedule()
        }

        pub fn player_coord(&self) -> Coord {
            self.0.player_coord()
        }

        pub fn rain_level(&self) -> RainLevel {
            self.0.rain_level()
        }

        pub fn equipped(&self) -> &Equipped {
            self.0.equipped()
        }

        pub fn player_lantern(&self) -> bool {
            self.0.player_lantern()
        }

        pub fn pushing(&self) -> bool {
            self.0.pushing()
        }

        pub fn player_item(&self) -> Option<Item> {
            self.0.player_item()
        }

        pub fn last_motivation_modifiers(&self) -> &[MotivationModifier] {
            self.0.last_motivation_modifiers()
        }

        pub fn time(&self) -> &Time {
            self.0.time()
        }

        pub fn motivation(&self) -> i32 {
            self.0.motivation()
        }
    }
}
