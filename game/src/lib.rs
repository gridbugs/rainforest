use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod witness;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: bool,
    pub demo: bool,
    pub debug: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Game {}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        Self {}
    }

    pub fn into_running_game(self, running: witness::Running) -> witness::RunningGame {
        witness::RunningGame::new(self, running)
    }
}
