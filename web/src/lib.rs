use chargrid_web::{Context, Size};
use general_storage_static::{backend::LocalStorage, StaticStorage};
use rainforest_app::{app, AppArgs, AppStorage, InitialRngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";
const CONTROLS_KEY: &str = "controls";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let storage = StaticStorage::new(LocalStorage::new());
    let context = Context::new(Size::new(80, 60), "content");
    let args = AppArgs {
        storage: AppStorage {
            handle: storage,
            save_game_key: SAVE_KEY.to_string(),
            controls_key: CONTROLS_KEY.to_string(),
        },
        initial_rng_seed: InitialRngSeed::Random,
        omniscient: false,
        new_game: false,
    };
    context.run(app(args));
    Ok(())
}
