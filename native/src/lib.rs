use general_storage_static::{
    backend::{FileStorage, IfDirectoryMissing},
    StaticStorage,
};
pub use meap;
use rainforest_app::{AppStorage, InitialRngSeed};

const DEFAULT_SAVE_FILE: &str = "save";
const DEFAULT_NEXT_TO_EXE_STORAGE_DIR: &str = "save";
const DEFAULT_CONTROLS_FILE: &str = "controls.json";

pub struct NativeCommon {
    pub storage: AppStorage,
    pub initial_rng_seed: InitialRngSeed,
    pub omniscient: bool,
    pub new_game: bool,
}

impl NativeCommon {
    pub fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                rng_seed = opt_opt::<u64, _>("INT", 'r').name("rng-seed").desc("rng seed to use for first new game");
                save_file = opt_opt("PATH", 's').name("save-file").desc("save file")
                    .with_default(DEFAULT_SAVE_FILE.to_string());
                controls_file = opt_opt("PATH", "controls-file").desc("controls file")
                    .with_default(DEFAULT_CONTROLS_FILE.to_string());
                storage_dir = opt_opt("PATH", 'd').name("storage-dir")
                    .desc("directory that will contain state")
                    .with_default(DEFAULT_NEXT_TO_EXE_STORAGE_DIR.to_string());
                delete_save = flag("delete-save").desc("delete save game file");
                new_game = flag("new-game").desc("start a new game, skipping the menu");
                omniscient = flag("omniscient").desc("enable omniscience");
            } in {{
                let initial_rng_seed = rng_seed.map(InitialRngSeed::U64).unwrap_or(InitialRngSeed::Random);
                let mut file_storage = StaticStorage::new(
                    FileStorage::next_to_exe(storage_dir, IfDirectoryMissing::Create)
                    .expect("failed to open directory"),
                );
                if delete_save {
                    let result = file_storage.remove(&save_file);
                    if result.is_err() {
                        log::warn!("couldn't find save file to delete");
                    }
                }
                let storage = AppStorage {
                    handle: file_storage,
                    save_game_key: save_file,
                    controls_key: controls_file,
                };
                Self {
                    initial_rng_seed,
                    storage,
                    omniscient,
                    new_game,
                }
            }}
        }
    }
}
