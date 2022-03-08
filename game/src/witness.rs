use crate::{Config, Game};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Private;

#[derive(Debug)]
pub struct Running(Private);

impl Running {
    pub fn into_witness(self) -> Witness {
        Witness::Running(self)
    }

    // TODO: The client should interact with the game by calling methods on the witness directly
    // to remove the need for the pub(crate)s in this file
    pub(crate) fn sleep(self) -> Witness {
        Sleep(self.0).into_witness()
    }
}

#[derive(Debug)]
pub struct Sleep(Private);

impl Sleep {
    pub fn into_witness(self) -> Witness {
        Witness::Sleep(self)
    }

    pub(crate) fn running(self) -> Witness {
        Running(self.0).into_witness()
    }

    pub fn cancel(self) -> Witness {
        Running(self.0).into_witness()
    }
}

#[derive(Debug)]
pub enum Witness {
    Running(Running),
    Sleep(Sleep),
}

pub fn new_game<R: Rng>(config: &Config, base_rng: &mut R) -> (Game, Running) {
    let g = Game::new(config, base_rng);
    (g, Running(Private))
}

#[derive(Serialize, Deserialize)]
pub struct RunningGame {
    game: Game,
}

impl RunningGame {
    pub fn new(game: Game, running: Running) -> Self {
        let _ = running;
        Self { game }
    }

    pub fn into_game(self) -> (Game, Running) {
        (self.game, Running(Private))
    }
}
