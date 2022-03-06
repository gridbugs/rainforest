use crate::{
    colour,
    controls::{AppInput, Controls},
    examine,
    fields::{GroundField, LogField},
    text, AppStorage, InitialRngSeed,
};
use chargrid::{border::BorderStyle, control_flow::boxed::*, menu, prelude::*, text::StyledString};
use rainforest_game::{
    witness::{self, RunningGame, Witness},
    ActionError, Config as GameConfig, Game,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

const GAME_VIEW_SIZE: Size = Size::new_u16(25, 19);
const GAME_VIEW_OFFSET: Coord = Coord::new(0, 2);

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

fn action_error_message(action_error: ActionError) -> StyledString {
    let style = Style::plain_text();
    let string = match action_error {
        ActionError::Message(s) => s,
    };
    StyledString { string, style }
}

#[derive(Serialize, Deserialize)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
    ground_field: GroundField,
    log_field: LogField,
}

impl GameInstanceStorable {
    fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self {
            running_game,
            ground_field,
            log_field,
        } = self;
        let (game, running) = running_game.into_game();
        (
            GameInstance {
                game,
                ground_field,
                log_field,
            },
            running,
        )
    }
}

struct GameInstance {
    game: Game,
    ground_field: GroundField,
    log_field: LogField,
}

impl GameInstance {
    pub fn new<R: Rng>(config: &GameConfig, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        let ground_field = GroundField::new(game.world_size(), rng);
        let log_field = LogField::new(game.world_size(), rng);
        (
            GameInstance {
                game,
                ground_field,
                log_field,
            },
            running,
        )
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self {
            game,
            ground_field,
            log_field,
        } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable {
            running_game,
            ground_field,
            log_field,
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let offset = self.game.player_coord() - (GAME_VIEW_SIZE / 2);
        crate::game::render_game_with_visibility(
            &self.game,
            offset,
            GAME_VIEW_SIZE,
            &self.ground_field,
            &self.log_field,
            ctx.add_offset(GAME_VIEW_OFFSET),
            fb,
        );
    }
}

pub enum GameLoopState {
    Examine(witness::Running),
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
    context_message: Option<StyledString>,
    examine_message: Option<StyledString>,
    cursor: Option<Coord>,
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
        (
            Self {
                instance,
                controls,
                game_config,
                storage,
                rng_seed_source,
                context_message: None,
                examine_message: None,
                cursor: None,
            },
            state,
        )
    }

    fn game(&self) -> &Game {
        &self.instance.as_ref().unwrap().game
    }

    fn render(&self, cursor_colour: Rgba32, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = self.instance.as_ref().unwrap();
        instance.render(ctx, fb);
        if let Some(cursor) = self.cursor {
            if cursor.is_valid(GAME_VIEW_SIZE + Size::new_u16(1, 1)) {
                let screen_cursor = GAME_VIEW_OFFSET + (cursor * 3);
                for offset in Size::new_u16(3, 3).coord_iter_row_major() {
                    fb.set_cell_relative_to_ctx(
                        ctx,
                        screen_cursor + offset,
                        10,
                        RenderCell::BLANK.with_background(cursor_colour),
                    );
                }
            }
        }
        self.render_text(ctx, fb);
    }

    fn render_text(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        if let Some(context_message) = self.context_message.as_ref() {
            context_message.render(&(), ctx.add_y(1), fb);
        }
        if let Some(top_text) = self.examine_message.as_ref() {
            top_text.clone().wrap_word().render(&(), ctx, fb);
        }
    }

    fn examine_mouse(&mut self, event: Event) {
        match event {
            Event::Input(Input::Mouse(mouse_input)) => match mouse_input {
                MouseInput::MouseMove { button: _, coord } => {
                    self.cursor = Some((coord - GAME_VIEW_OFFSET) / 3);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn update_examine_text(&mut self) {
        self.examine_message = self.cursor.and_then(|coord| {
            let world_coord = self.game().player_coord() - (GAME_VIEW_SIZE / 2) + coord;
            examine::examine(self.game(), world_coord)
        });
    }

    fn update(&mut self, event: Event, running: witness::Running) -> GameLoopState {
        let instance = self.instance.as_mut().unwrap();
        let witness = match event {
            Event::Input(input) => {
                if let Some(app_input) = self.controls.get(input) {
                    let (witness, action_result) = match app_input {
                        AppInput::Direction(direction) => {
                            instance
                                .game
                                .player_walk(direction, &self.game_config, running)
                        }
                        AppInput::Wait => (running.into_witness(), Ok(())),
                        AppInput::Get => (running.into_witness(), Ok(())),
                        AppInput::Examine => {
                            return GameLoopState::Examine(running);
                        }
                    };
                    if let Err(action_error) = action_result {
                        self.context_message = Some(action_error_message(action_error));
                    } else {
                        self.context_message = None;
                    }
                    witness
                } else {
                    running.into_witness()
                }
            }
            Event::Tick(since_previous) => {
                instance
                    .game
                    .tick(since_previous, &self.game_config, running)
            }
            _ => Witness::Running(running),
        };
        self.examine_mouse(event);
        self.update_examine_text();
        GameLoopState::Playing(witness)
    }

    fn new_game(&mut self) -> witness::Running {
        let (instance, running) = new_game(&mut self.rng_seed_source, &self.game_config);
        self.instance = Some(instance);
        running
    }

    fn save_instance(&mut self, running: witness::Running) -> witness::Running {
        let instance = self.instance.take().unwrap().into_storable(running);
        self.storage.save_game(&instance);
        let (instance, running) = instance.into_game_instance();
        self.instance = Some(instance);
        running
    }

    fn clear_saved_game(&mut self) {
        self.storage.clear_game();
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

struct GameExamineComponent;

impl Component for GameExamineComponent {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(colour::CURSOR.with_a(128), ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        if let Some(input) = event.input() {
            state.controls.get_direction(input).map(|direction| {
                let cursor = state.cursor.unwrap_or_else(|| state.game().player_coord());
                state.cursor = Some(cursor + direction.coord());
            });
            if let Some(AppInput::Examine) = state.controls.get(input) {
                return Some(());
            }
        }
        state.examine_mouse(event);
        state.update_examine_text();
        None
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn game_examine_component() -> CF<()> {
    on_state_then(|state: &mut State| {
        state.context_message = Some(StyledString {
            string: "Examining (escape/start to return to game)".to_string(),
            style: Style::plain_text().with_foreground(Rgba32::new_grey(100)),
        });
        let cursor = state.cursor.unwrap_or_else(|| state.game().player_coord());
        state.cursor = Some(cursor);
        boxed_cf(GameExamineComponent)
            .catch_escape_or_start()
            .map_val(|| ())
            .side_effect(|state: &mut State| {
                state.context_message = None;
                state.cursor = None;
            })
    })
}

const MENU_FADE_SPEC: menu::identifier::fade_spec::FadeSpec = {
    use menu::identifier::fade_spec::*;
    FadeSpec {
        on_select: Fade {
            to: To {
                rgba32: Layers {
                    foreground: Rgba32::new_grey(0),
                    background: Rgba32::new_grey(255),
                },
                bold: true,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
        on_deselect: Fade {
            to: To {
                rgba32: Layers {
                    foreground: Rgba32::new_grey(128),
                    background: Rgba32::hex(0),
                },
                bold: false,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
    }
};

fn menu_style<T: 'static>(menu: CF<T>) -> CF<T> {
    menu.border(BorderStyle::default())
        .fill(Rgba32::new_grey(0))
        .centre()
        .overlay_tint(
            render_state(|state: &State, ctx, fb| state.render(colour::CURSOR, ctx, fb)),
            chargrid::core::TintDim(63),
            10,
        )
}

#[derive(Clone)]
enum MainMenuEntry {
    NewGame,
    Help,
    Quit,
}

enum MainMenuOutput {
    NewGame { new_running: witness::Running },
    Quit,
}

fn main_menu() -> CF<MainMenuEntry> {
    use menu::builder::*;
    use MainMenuEntry::*;
    let mut builder = menu_builder().vi_keys();
    let mut add_item = |entry, name, ch: char| {
        let identifier =
            MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
        builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
    };
    add_item(NewGame, "New Game", 'n');
    add_item(Help, "Help", 'h');
    add_item(Quit, "Quit", 'q');
    builder.build_boxed_cf()
}

fn title_decorate<T: 'static>(cf: CF<T>) -> CF<T> {
    cf.with_title(
        styled_string("Rain Forest".to_string(), Style::plain_text()),
        2,
    )
    .centre()
}

const MAIN_MENU_TEXT_WIDTH: u32 = 40;
fn main_menu_loop() -> CF<MainMenuOutput> {
    use MainMenuEntry::*;
    title_decorate(main_menu()).repeat_unit(move |entry| match entry {
        NewGame => on_state(|state: &mut State| MainMenuOutput::NewGame {
            new_running: state.new_game(),
        })
        .break_(),
        Help => text::help(MAIN_MENU_TEXT_WIDTH).centre().continue_(),
        Quit => val_once(MainMenuOutput::Quit).break_(),
    })
}

#[derive(Clone)]
enum PauseMenuEntry {
    Resume,
    SaveQuit,
    Save,
    NewGame,
    Help,
    Clear,
}

enum PauseOutput {
    ContinueGame { running: witness::Running },
    MainMenu,
    Quit,
}

fn pause_menu() -> CF<PauseMenuEntry> {
    use menu::builder::*;
    use PauseMenuEntry::*;
    let mut builder = menu_builder().vi_keys();
    let mut add_item = |entry, name, ch: char| {
        let identifier =
            MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
        builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
    };
    add_item(Resume, "Resume", 'r');
    add_item(SaveQuit, "Save and Quit", 'q');
    add_item(Save, "Save", 's');
    add_item(NewGame, "New Game", 'n');
    add_item(Help, "Help", 'h');
    add_item(Clear, "Clear", 'c');
    builder.build_boxed_cf()
}

fn pause_menu_loop(running: witness::Running) -> CF<PauseOutput> {
    use PauseMenuEntry::*;
    let text_width = 64;
    menu_style(
        pause_menu()
            .menu_harness()
            .repeat(
                running,
                move |running, entry_or_escape| match entry_or_escape {
                    Ok(entry) => match entry {
                        Resume => break_(PauseOutput::ContinueGame { running }),
                        SaveQuit => on_state(|state: &mut State| {
                            state.save_instance(running);
                            PauseOutput::Quit
                        })
                        .break_(),
                        Save => on_state(|state: &mut State| PauseOutput::ContinueGame {
                            running: state.save_instance(running),
                        })
                        .break_(),
                        NewGame => on_state(|state: &mut State| PauseOutput::ContinueGame {
                            running: state.new_game(),
                        })
                        .break_(),
                        Help => text::help(text_width).continue_with(running),
                        Clear => on_state(|state: &mut State| {
                            state.clear_saved_game();
                            PauseOutput::MainMenu
                        })
                        .break_(),
                    },
                    Err(_escape_or_start) => break_(PauseOutput::ContinueGame { running }),
                },
            ),
    )
}

pub fn game_loop_component(initial_state: GameLoopState) -> CF<()> {
    use GameLoopState::*;
    loop_(initial_state, |state| match state {
        Playing(witness) => match witness {
            Witness::Running(running) => game_instance_component(running).continue_(),
        },
        Paused(running) => pause_menu_loop(running).map(|pause_output| match pause_output {
            PauseOutput::ContinueGame { running } => {
                LoopControl::Continue(Playing(running.into_witness()))
            }
            PauseOutput::MainMenu => LoopControl::Continue(MainMenu),
            PauseOutput::Quit => LoopControl::Break(()),
        }),
        Examine(running) => game_examine_component()
            .map_val(|| Playing(running.into_witness()))
            .continue_(),
        MainMenu => main_menu_loop().map(|main_menu_output| match main_menu_output {
            MainMenuOutput::NewGame { new_running } => {
                LoopControl::Continue(Playing(new_running.into_witness()))
            }
            MainMenuOutput::Quit => LoopControl::Break(()),
        }),
    })
}
