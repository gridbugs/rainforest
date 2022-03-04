use crate::{colour, controls::Controls, AppStorage, Config, InitialRngSeed};
use chargrid::{control_flow::boxed::*, prelude::*};
use rainforest_game::{
    witness::{self, RunningGame, Witness},
    Config as GameConfig, Game,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

/// An interactive, renderable process yielding a value of type `T`
pub type CF<T> = BoxedCF<Option<T>, GameLoopData>;
pub type State = GameLoopData;

struct RngSeedSource {
    next_seed: u64,
    seed_rng: Isaac64Rng,
}

impl RngSeedSource {
    fn new(initial_rng_seed: InitialRngSeed) -> Self {
        let mut seed_rng = Isaac64Rng::from_entropy();
        let next_seed = match initial_rng_seed {
            InitialRngSeed::U64(seed) => seed,
            InitialRngSeed::Random => seed_rng.gen(),
        };
        Self {
            next_seed,
            seed_rng,
        }
    }

    fn next_seed(&mut self) -> u64 {
        let seed = self.next_seed;
        self.next_seed = self.seed_rng.gen();
        #[cfg(feature = "print_stdout")]
        println!("RNG Seed: {}", seed);
        #[cfg(feature = "print_log")]
        log::info!("RNG Seed: {}", seed);
        seed
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
}

impl GameInstanceStorable {
    fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self { running_game } = self;
        let (game, running) = running_game.into_game();
        (GameInstance { game }, running)
    }
}

struct GameInstance {
    game: Game,
}

impl GameInstance {
    pub fn new<R: Rng>(config: &GameConfig, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        (GameInstance { game }, running)
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self { game } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable { running_game }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        crate::game::render_game_with_visibility(&self.game, ctx, fb);
    }
}

pub enum GameLoopState {
    Paused(witness::Running),
    Playing(Witness),
    MainMenu,
}

pub struct GameLoopData {
    instance: Option<GameInstance>,
    controls: Controls,
    game_config: GameConfig,
    storage: AppStorage,
    rng_seed_source: RngSeedSource,
    config: Config,
}

fn new_game(
    rng_seed_source: &mut RngSeedSource,
    game_config: &GameConfig,
) -> (GameInstance, witness::Running) {
    let mut rng = Isaac64Rng::seed_from_u64(rng_seed_source.next_seed());
    GameInstance::new(game_config, &mut rng)
}

impl GameLoopData {
    pub fn new(
        game_config: GameConfig,
        mut storage: AppStorage,
        initial_rng_seed: InitialRngSeed,
        force_new_game: bool,
    ) -> (Self, GameLoopState) {
        let mut rng_seed_source = RngSeedSource::new(initial_rng_seed);
        let (instance, state) = match storage.load_game() {
            Some(instance) => {
                let (instance, running) = instance.into_game_instance();
                (
                    Some(instance),
                    GameLoopState::Playing(running.into_witness()),
                )
            }
            None => {
                if force_new_game {
                    let (instance, running) = new_game(&mut rng_seed_source, &game_config);
                    (
                        Some(instance),
                        GameLoopState::Playing(running.into_witness()),
                    )
                } else {
                    (None, GameLoopState::MainMenu)
                }
            }
        };
        let controls = if let Some(controls) = storage.load_controls() {
            controls
        } else {
            let controls = Controls::default();
            storage.save_controls(&controls);
            controls
        };
        let config = storage.load_config().unwrap_or_default();
        (
            Self {
                instance,
                controls,
                game_config,
                storage,
                rng_seed_source,
                config,
            },
            state,
        )
    }

    fn render(&self, cursor_colour: Rgba32, ctx: Ctx, fb: &mut FrameBuffer) {
        self.instance.as_ref().unwrap().render(ctx, fb);
    }

    fn update(&mut self, event: Event, running: witness::Running) -> GameLoopState {
        GameLoopState::Playing(running.into_witness())
    }
}

struct GameInstanceComponent(Option<witness::Running>);

impl GameInstanceComponent {
    fn new(running: witness::Running) -> Self {
        Self(Some(running))
    }
}

impl Component for GameInstanceComponent {
    type Output = GameLoopState;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(colour::CURSOR, ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = self.0.take().unwrap();
        if event.is_escape_or_start() {
            GameLoopState::Paused(running)
        } else {
            state.update(event, running)
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn game_instance_component(running: witness::Running) -> CF<GameLoopState> {
    boxed_cf(GameInstanceComponent::new(running))
        .some()
        .no_peek()
}

pub fn game_loop_component(initial_state: GameLoopState) -> CF<()> {
    use GameLoopState::*;
    loop_(initial_state, |state| match state {
        Playing(witness) => match witness {
            Witness::Running(running) => game_instance_component(running).continue_(),
        },
        Paused(running) => todo!(),
        MainMenu => todo!(),
    })
    .bound_size(Size::new_u16(80, 60))
}
