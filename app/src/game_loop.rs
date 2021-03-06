use crate::{
    colour,
    controls::{AppInput, Controls},
    examine,
    fields::{GroundField, LogField, TeaField},
    mist::Mist,
    rain::{Rain, RainDirection},
    text, AppStorage, InitialRngSeed,
};
use gridbugs::{
    chargrid::{
        self, border::BorderStyle, control_flow::*, menu, pad_by::Padding, prelude::*,
        text::StyledString,
    },
    grid_2d::Grid,
};
use rainforest_game::{
    witness::{self, RunningGame, Witness},
    ActionError, Config as GameConfig, Game, Item, RainLevel, RainSchedule, TopographyCell,
    MAX_MOTIVATION,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

const GAME_VIEW_SIZE: Size = Size::new_u16(26, 18);
const GAME_VIEW_OFFSET: Coord = Coord::new(1, 2);

/// An interactive, renderable process yielding a value of type `T`
pub type AppCF<T> = CF<Option<T>, GameLoopData>;
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
    tea_field: TeaField,
    rain: Rain,
    mist: Mist,
}

impl GameInstanceStorable {
    fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self {
            running_game,
            ground_field,
            log_field,
            tea_field,
            rain,
            mist,
        } = self;
        let (game, running) = running_game.into_game();
        (
            GameInstance {
                game,
                ground_field,
                log_field,
                tea_field,
                rain,
                mist,
            },
            running,
        )
    }
}

struct GameInstance {
    game: Game,
    ground_field: GroundField,
    log_field: LogField,
    tea_field: TeaField,
    rain: Rain,
    mist: Mist,
}

impl GameInstance {
    pub fn new<R: Rng>(config: &GameConfig, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        let ground_field = GroundField::new(game.world_size(), rng);
        let log_field = LogField::new(game.world_size(), rng);
        let tea_field = TeaField::new(game.world_size(), rng);
        let rain = Rain::new(&game, 10000, RainDirection::Diagonal, rng);
        let mist = Mist::new(rng);
        (
            GameInstance {
                game,
                ground_field,
                log_field,
                tea_field,
                rain,
                mist,
            },
            running,
        )
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self {
            game,
            ground_field,
            log_field,
            tea_field,
            rain,
            mist,
        } = self;
        let running_game = running.running_game(game);
        GameInstanceStorable {
            running_game,
            ground_field,
            log_field,
            tea_field,
            rain,
            mist,
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.render_top_ui(ctx.add_depth(10), fb);
        let offset = self.game.player_coord() - (GAME_VIEW_SIZE / 2);
        let ctx = ctx.add_offset(GAME_VIEW_OFFSET);
        crate::game::render_game_with_visibility(
            &self.game,
            offset,
            GAME_VIEW_SIZE,
            &self.ground_field,
            &self.log_field,
            &self.tea_field,
            &self.mist,
            ctx,
            fb,
        );
        self.rain
            .render(&self.game, offset, GAME_VIEW_SIZE, ctx, fb);
        self.render_bottom_ui(ctx.add_y(GAME_VIEW_SIZE.y() as i32 * 3), fb);
    }

    fn render_top_ui(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let time = StyledString {
            string: self.game.time().to_string(),
            style: Style::plain_text(),
        };
        time.render(&(), ctx.add_x(67), fb);
        let weather = StyledString {
            string: self.game.rain_level().to_string(),
            style: Style::plain_text(),
        };
        weather.render(&(), ctx.add_xy(67, 1), fb);
    }
    fn render_bottom_ui(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let motivation = self.game.motivation().max(0);
        let motivation_colour = if motivation < 200 {
            Rgba32::new_rgb(255, 0, 0)
        } else {
            Rgba32::new_grey(255)
        };
        StyledString {
            string: format!("Motivation: {}/{}", motivation, MAX_MOTIVATION),
            style: Style::plain_text()
                .with_bold(true)
                .with_foreground(motivation_colour),
        }
        .render(&(), ctx.add_y(2), fb);
        let motivation_modifiers = self
            .game
            .last_motivation_modifiers()
            .iter()
            .map(|m| {
                let sign = if m.value() >= 0 { "+" } else { "" };
                format!("{} ({}{})", m.to_string(), sign, m.value())
            })
            .collect::<Vec<_>>()
            .join(", ");
        let motivation_change: i32 = self
            .game
            .last_motivation_modifiers()
            .iter()
            .map(|m| m.value())
            .sum();
        StyledString {
            string: format!(
                "Motivation Change: {} = {}",
                motivation_change, motivation_modifiers
            ),
            style: Style::plain_text().with_foreground(Rgba32::new_grey(185)),
        }
        .wrap_word()
        .render(&(), ctx.add_y(0), fb);
        let item_str = match self.game.player_item() {
            None => "(nothing)",
            Some(Item::Flower) => "a flower (drop with g)",
            Some(Item::Tea) => "some tea leaves (drop with g)",
            Some(Item::Rock) => "a rock (drop with g)",
        };
        StyledString {
            string: (format!("Holding: {}", item_str)),
            style: Style::plain_text().with_bold(true),
        }
        .wrap_word()
        .render(&(), ctx.add_xy(40, 2), fb);
        let rock_behaviour = if self.game.pushing() { "push" } else { "walk" };
        StyledString {
            string: (format!("Rocks: {}", rock_behaviour)),
            style: Style::plain_text().with_bold(true),
        }
        .wrap_word()
        .render(&(), ctx.add_xy(25, 2), fb);
        let mut equipped_text = Vec::new();
        {
            let equipped = self.game.equipped();
            if equipped.umbrella {
                equipped_text.push("Umbrella");
            }
            if equipped.gumboots {
                equipped_text.push("Gumboots");
            }
            if equipped.lantern {
                if self.game.player_lantern() {
                    equipped_text.push("Lantern (on)");
                } else {
                    equipped_text.push("Lantern (off)");
                }
            }
            if equipped.crowbar {
                equipped_text.push("Crowbar");
            }
            if equipped.shovel {
                equipped_text.push("Shovel");
            }
            if equipped.map {
                equipped_text.push("Map");
            }
            if equipped.weather_report {
                equipped_text.push("Weather Report");
            }
        }
        let equipped_string = if equipped_text.is_empty() {
            "(nothing)".to_string()
        } else {
            equipped_text.join(", ")
        };
        StyledString {
            string: (format!("Equipped: {}", equipped_string)),
            style: Style::plain_text().with_bold(true),
        }
        .wrap_word()
        .render(&(), ctx.add_xy(0, 3), fb);
    }
}

pub enum GameLoopState {
    Examine(witness::Running),
    Paused(witness::Running),
    Playing(Witness),
    MainMenu,
    Map(witness::Running),
    WeatherReport(witness::Running),
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
        self.render_text(ctx.add_depth(20), fb);
    }

    fn render_text(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        if let Some(context_message) = self.context_message.as_ref() {
            context_message.render(&(), ctx.add_xy(1, 1), fb);
        }
        if let Some(top_text) = self.examine_message.as_ref() {
            top_text.clone().wrap_word().render(&(), ctx.add_x(1), fb);
        } else if let Some(mut here) = examine::examine_under_player(self.game()) {
            here.string = format!("Here: {}", here.string);
            here.wrap_word().render(&(), ctx.add_x(1), fb);
        }
    }

    fn examine_mouse(&mut self, event: Event) {
        match event {
            Event::Input(Input::Mouse(mouse_input)) => match mouse_input {
                MouseInput::MouseMove { button: _, coord } => {
                    let cursor = (coord - GAME_VIEW_OFFSET) / 3;
                    if cursor.is_valid(GAME_VIEW_SIZE) {
                        self.cursor = Some(cursor);
                    } else {
                        self.cursor = None;
                    }
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
                    self.cursor = None;
                    let (witness, action_result) = match app_input {
                        AppInput::Direction(direction) => {
                            running.player_walk(&mut instance.game, direction, &self.game_config)
                        }
                        AppInput::DirectionLong(direction) => running.player_walk_until_collide(
                            &mut instance.game,
                            direction,
                            &self.game_config,
                        ),
                        AppInput::Wait => (
                            running.player_wait(&mut instance.game, &self.game_config),
                            Ok(()),
                        ),
                        AppInput::WaitLong => (
                            running.player_wait_long(&mut instance.game, &self.game_config),
                            Ok(()),
                        ),
                        AppInput::Get => running.player_get(&mut instance.game, &self.game_config),
                        AppInput::Lantern => {
                            running.player_toggle_lantern(&mut instance.game, &self.game_config)
                        }
                        AppInput::Pushing => {
                            running.player_toggle_pushing(&mut instance.game, &self.game_config)
                        }
                        AppInput::Dig => running.player_dig(&mut instance.game, &self.game_config),
                        AppInput::Map => {
                            if instance.game.equipped().map {
                                return GameLoopState::Map(running);
                            } else {
                                (
                                    running.into_witness(),
                                    ActionError::err_msg("You don't have the map equipped!"),
                                )
                            }
                        }
                        AppInput::WeatherReport => {
                            if instance.game.equipped().weather_report {
                                return GameLoopState::WeatherReport(running);
                            } else {
                                (
                                    running.into_witness(),
                                    ActionError::err_msg(
                                        "You don't have the weather report equipped!",
                                    ),
                                )
                            }
                        }
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
                match instance.game.rain_level() {
                    RainLevel::Light => instance.rain.update(4000, RainDirection::Vertical),
                    RainLevel::Medium => instance.rain.update(10000, RainDirection::Diagonal),
                    RainLevel::Heavy => instance.rain.update(30000, RainDirection::Diagonal),
                }
                instance.rain.tick();
                instance.mist.tick();
                running.tick(&mut instance.game, since_previous, &self.game_config)
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

fn game_instance_component(running: witness::Running) -> AppCF<GameLoopState> {
    cf(GameInstanceComponent::new(running)).some().no_peek()
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

fn game_examine_component() -> AppCF<()> {
    on_state_then(|state: &mut State| {
        state.context_message = Some(StyledString {
            string: "Examining (escape/start to return to game)".to_string(),
            style: Style::plain_text(),
        });
        let cursor = state.cursor.unwrap_or_else(|| state.game().player_coord());
        state.cursor = Some(cursor);
        cf(GameExamineComponent)
            .catch_escape_or_start()
            .map_val(|| ())
            .side_effect(|state: &mut State| {
                state.context_message = None;
                state.cursor = None;
            })
    })
}

fn weather_report_component() -> AppCF<()> {
    on_state_then(|state: &mut State| {
        state.context_message = Some(StyledString {
            string: "Weather Report (escape/start to return to game)".to_string(),
            style: Style::plain_text(),
        });
        let rain_schedule = state.game().rain_schedule();
        state.examine_message = None;
        cf(WeatherReportComponent(rain_schedule))
            .catch_escape_or_start()
            .map_val(|| ())
            .side_effect(|state: &mut State| {
                state.context_message = None;
                state.cursor = None;
            })
    })
}

struct WeatherReportComponent(RainSchedule);
impl Component for WeatherReportComponent {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render_text(ctx, fb);
        let mut ctx = ctx.add_xy(1, 3);
        StyledString::plain_text(format!(
            "              |{:^11}|{:^11}|{:^11}|{:^11}|{:^11}|",
            "Day 1", "Day 2", "Day 3", "Day 4", "Day 5",
        ))
        .render(&(), ctx, fb);
        let mut line = |name: &str, time| {
            ctx = ctx.add_y(1);
            StyledString::plain_text(format!(
                "--------------+{:-^11}+{:-^11}+{:-^11}+{:-^11}+{:-^11}+",
                "", "", "", "", ""
            ))
            .render(&(), ctx, fb);
            ctx = ctx.add_y(1);
            StyledString::plain_text(format!(
                "              |{:^11}|{:^11}|{:^11}|{:^11}|{:^11}|",
                "", "", "", "", ""
            ))
            .render(&(), ctx, fb);
            ctx = ctx.add_y(1);
            StyledString::plain_text(format!(
                "{} |{:^11}|{:^11}|{:^11}|{:^11}|{:^11}|",
                name,
                self.0.get(1, time).to_string(),
                self.0.get(2, time).to_string(),
                self.0.get(3, time).to_string(),
                self.0.get(4, time).to_string(),
                self.0.get(5, time).to_string(),
            ))
            .render(&(), ctx, fb);
            ctx = ctx.add_y(1);
            StyledString::plain_text(format!(
                "              |{:^11}|{:^11}|{:^11}|{:^11}|{:^11}|",
                "", "", "", "", ""
            ))
            .render(&(), ctx, fb);
        };
        line("00:00 - 04:00", 0);
        line("04:00 - 08:00", 1);
        line("08:00 - 12:00", 2);
        line("12:00 - 16:00", 3);
        line("16:00 - 20:00", 4);
        line("20:00 - 00:00", 5);
        StyledString::plain_text(format!(
            "              |{:^11}|{:^11}|{:^11}|{:^11}|{:^11}|",
            "", "", "", "", ""
        ))
        .render(&(), ctx, fb);
        StyledString::plain_text(format!(
            "--------------+{:-^11}+{:-^11}+{:-^11}+{:-^11}+{:-^11}+",
            "", "", "", "", ""
        ))
        .render(&(), ctx.add_y(1), fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input) => {
                if let Some(app_input) = state.controls.get(input) {
                    match app_input {
                        AppInput::Map => return Some(()),
                        _ => (),
                    }
                }
            }
            _ => (),
        }

        None
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

struct MapComponent(Grid<TopographyCell>);
impl Component for MapComponent {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render_text(ctx, fb);
        let ctx = ctx.add_xy(17, 6);
        for (coord, &cell) in self.0.enumerate() {
            let (render_cell, depth) = match cell {
                TopographyCell::Height(height) => (
                    RenderCell::default()
                        .with_character(' ')
                        .with_background(Rgba32::new_grey((height.clamp(0., 1.) * 255.) as u8)),
                    0,
                ),
                TopographyCell::Water => (
                    RenderCell::default()
                        .with_character('~')
                        .with_background(Rgba32::new_grey(0)),
                    0,
                ),
                TopographyCell::Unknown => (RenderCell::default().with_character('?'), 0),
                TopographyCell::Player => (
                    RenderCell::default()
                        .with_character('@')
                        .with_bold(true)
                        .with_foreground(Rgba32::new_rgb(255, 0, 0)),
                    2,
                ),
                TopographyCell::Ruins => {
                    StyledString::plain_text("Ruins".to_string()).render(
                        &(),
                        ctx.add_offset(coord / 3).add_depth(1),
                        fb,
                    );
                    continue;
                }
                TopographyCell::Cabin => {
                    StyledString::plain_text("Cabin".to_string()).render(
                        &(),
                        ctx.add_offset(coord / 3).add_depth(1),
                        fb,
                    );
                    continue;
                }
                TopographyCell::Tea => {
                    StyledString::plain_text("Tea".to_string()).render(
                        &(),
                        ctx.add_offset(coord / 3).add_depth(1),
                        fb,
                    );
                    continue;
                }
                TopographyCell::Flowers => {
                    StyledString::plain_text("Flowers".to_string()).render(
                        &(),
                        ctx.add_offset(coord / 3).add_depth(1),
                        fb,
                    );
                    continue;
                }
            };
            fb.set_cell_relative_to_ctx(ctx, coord / 3, depth, render_cell);
        }
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input) => {
                if let Some(app_input) = state.controls.get(input) {
                    match app_input {
                        AppInput::Map => return Some(()),
                        _ => (),
                    }
                }
            }
            _ => (),
        }

        None
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn map_component() -> AppCF<()> {
    on_state_then(|state: &mut State| {
        state.context_message = Some(StyledString {
            string: "Topographic Map (escape/start to return to game)".to_string(),
            style: Style::plain_text(),
        });
        let topography_grid = state.game().topography_grid();
        state.examine_message = None;
        cf(MapComponent(topography_grid))
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

fn menu_style<T: 'static>(menu: AppCF<T>) -> AppCF<T> {
    menu.border(BorderStyle::default())
        .fill(Rgba32::new_grey(0))
        .centre()
        .overlay_tint(
            render_state(|state: &State, ctx, fb| state.render(colour::CURSOR, ctx, fb)),
            chargrid::core::TintDim(63),
            10,
        )
}

fn popup_style<T: 'static>(menu: AppCF<T>) -> AppCF<T> {
    menu.border(BorderStyle::default())
        .fill(Rgba32::new_grey(0))
        .centre()
        .add_y(30)
        .overlay_tint(
            render_state(|state: &State, ctx, fb| state.render(colour::CURSOR, ctx, fb)),
            chargrid::core::TintDim(255),
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

fn main_menu() -> AppCF<MainMenuEntry> {
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
    builder.build_cf()
}

fn title_decorate<T: 'static>(cf: AppCF<T>) -> AppCF<T> {
    cf.with_title(
        styled_string(
            "Rain Forest".to_string(),
            Style::plain_text()
                .with_foreground(Rgba32::hex_rgb(0x1b6f16))
                .with_bold(true),
        ),
        2,
    )
    .centre()
}

const MAIN_MENU_TEXT_WIDTH: u32 = 40;
fn main_menu_loop() -> AppCF<MainMenuOutput> {
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

fn pause_menu() -> AppCF<PauseMenuEntry> {
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
    builder.build_cf()
}

fn pause_menu_loop(running: witness::Running) -> AppCF<PauseOutput> {
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

fn yes_no_menu() -> AppCF<bool> {
    use menu::builder::*;
    menu_builder()
        .vi_keys()
        .add_item(
            item(
                true,
                MENU_FADE_SPEC.identifier(move |b| write!(b, "(y) Yes").unwrap()),
            )
            .add_hotkey_char('y')
            .add_hotkey_char('Y'),
        )
        .add_item(
            item(
                false,
                MENU_FADE_SPEC.identifier(move |b| write!(b, "(n) No").unwrap()),
            )
            .add_hotkey_char('n')
            .add_hotkey_char('N'),
        )
        .build_cf()
}

fn yes_no(message: String) -> AppCF<bool> {
    menu_style(
        yes_no_menu().with_title(
            cf(StyledString {
                string: message,
                style: Style::plain_text(),
            }
            .wrap_word())
            .ignore_state()
            .bound_width(40),
            1,
        ),
    )
}

fn sleep_menu(sleep: witness::Sleep) -> AppCF<Witness> {
    yes_no("Go to sleep?".to_string()).map_side_effect(|yes, state: &mut State| {
        if yes {
            let instance = state.instance.as_mut().unwrap();
            sleep.commit(&mut instance.game, &state.game_config)
        } else {
            sleep.cancel()
        }
    })
}

fn popup_delay(string: String) -> AppCF<()> {
    popup_style(
        StyledString {
            string: string.clone(),
            style: Style::new()
                .with_bold(false)
                .with_underline(false)
                .with_foreground(Rgba32::new_grey(255)),
        }
        .wrap_word()
        .cf()
        .bound_width(50)
        .pad_by(Padding::all(1))
        .delay(Duration::from_secs(2)),
    )
    .then(|| popup(string))
}

fn popup(string: String) -> AppCF<()> {
    popup_style(
        StyledString {
            string,
            style: Style::new()
                .with_bold(false)
                .with_underline(false)
                .with_foreground(Rgba32::new_grey(255)),
        }
        .wrap_word()
        .cf()
        .bound_width(50)
        .pad_by(Padding::all(1))
        .press_any_key(),
    )
}

fn prompt(prompt_witness: witness::Prompt) -> AppCF<Witness> {
    on_state_then(move |state: &mut State| {
        state.examine_message = None;
        state.cursor = None;
        popup(prompt_witness.message().to_string()).map_val(|| prompt_witness.running())
    })
}

fn game_over() -> AppCF<()> {
    on_state_then(move |state: &mut State| {
        state.examine_message = None;
        state.cursor = None;
        state.clear_saved_game();
        popup_delay("You tire of trudging through the flooded forest in the rain. You pack up your belongings and return home.".to_string())
    })
}

fn win() -> AppCF<()> {
    on_state_then(move |state: &mut State| {
        state.examine_message = None;
        state.cursor = None;
        state.clear_saved_game();
        popup_delay("After five days enjoying the forest in the rain, it's time to return to your life. This break was just what you needed.".to_string())
    })
}

pub fn game_loop_component(initial_state: GameLoopState) -> AppCF<()> {
    use GameLoopState::*;
    loop_(initial_state, |state| match state {
        Playing(witness) => match witness {
            Witness::Running(running) => game_instance_component(running).continue_(),
            Witness::Sleep(sleep) => sleep_menu(sleep).map(Playing).continue_(),
            Witness::Prompt(prompt_witness) => prompt(prompt_witness).map(Playing).continue_(),
            Witness::GameOver => game_over().map_val(|| MainMenu).continue_(),
            Witness::Win => win().map_val(|| MainMenu).continue_(),
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
        Map(running) => map_component()
            .map_val(|| Playing(running.into_witness()))
            .continue_(),
        WeatherReport(running) => weather_report_component()
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
