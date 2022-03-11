use rgb_int::Rgba32;

pub const CURSOR: Rgba32 = Rgba32::new(255, 255, 0, 64);
pub const FLOOR_BACKGROUND: Rgba32 = Rgba32::new_grey(0);
pub const FLOOR_FOREGROUND: Rgba32 = Rgba32::hex_rgb(0x8b602c);
pub const WALL_FRONT: Rgba32 = Rgba32::new_rgb(0xD0, 0x8C, 0x15);
pub const WINDOWS: Rgba32 = Rgba32::new_rgb(0xBE, 0xED, 0xFF);
pub const DOOR: Rgba32 = Rgba32::hex_rgb(0x634015);
pub const DOOR_BORDER: Rgba32 = Rgba32::hex_rgb(0x362d11);
pub const PLAYER: Rgba32 = Rgba32::new_grey(255);
pub const WOOD: Rgba32 = Rgba32::hex_rgb(0x362d11);
pub const LEAF: Rgba32 = Rgba32::hex_rgb(0x376315);
pub const RAIN: Rgba32 = Rgba32::new_rgb(50, 120, 150);
pub const RAIN_REMEMBERED: Rgba32 = Rgba32::new_rgb(80, 90, 100);
pub const RUINS_WALL_TOP: Rgba32 = Rgba32::new_rgb(31, 31, 31);
pub const RUINS_WALL_FRONT_BACKGROUND: Rgba32 = Rgba32::new_rgb(63, 63, 63);
pub const RUINS_WALL_FRONT_FOREGROUND: Rgba32 = Rgba32::new_rgb(95, 95, 95);
pub const RUINS_FLOOR_BACKGROUND: Rgba32 = Rgba32::new_rgb(0, 0, 0);
pub const RUINS_FLOOR_FOREGROUND: Rgba32 = Rgba32::new_rgb(127, 127, 127);
pub const ALTAR_FOREGROUND: Rgba32 = Rgba32::new_rgb(185, 185, 185);
pub const ALTAR_TOP_FOREGROUND: Rgba32 = Rgba32::new_rgb(185, 185, 185);
pub const BULLETIN_TEXT: Rgba32 = Rgba32::new_rgb(0, 0, 0);
pub const LAMP_BASE: Rgba32 = Rgba32::new_rgb(50, 50, 50);
pub const LAMP_LIGHT: Rgba32 = Rgba32::new_rgb(185, 185, 0);
pub const LAMP_OFF: Rgba32 = Rgba32::new_rgb(63, 63, 63);
pub const PIER_FLOOR_BACKGROUND: Rgba32 = WOOD.saturating_scalar_mul_div(1, 2);
pub const PIER_FLOOR_FOREGROUND: Rgba32 = WOOD;
pub const GRASS: Rgba32 = Rgba32::hex_rgb(0x154712);
pub const ROCK: Rgba32 = Rgba32::hex_rgb(0x2d1d16);
pub const FLOWER: Rgba32 = Rgba32::new_rgb(255, 255, 255);
pub const BED_MATRESS: Rgba32 = Rgba32::new_rgb(185, 0, 0);
pub const BED_HEAD: Rgba32 = Rgba32::new_rgb(185, 185, 185);
pub const BED_LEGS: Rgba32 = Rgba32::new_rgb(63, 63, 63);
pub const CHAIR: Rgba32 = Rgba32::hex_rgb(0x0c3a44);
pub const TEAPOT: Rgba32 = Rgba32::hex_rgb(0x8c0bcb);
pub const TEA: Rgba32 = Rgba32::hex_rgb(0x2ec110);
pub const GUMBOOTS: Rgba32 = Rgba32::new_rgb(255, 255, 0);
pub const UMBRELLA: Rgba32 = Rgba32::new_rgb(185, 0, 255);
pub const UMBRELLA_HANDLE: Rgba32 = Rgba32::new_rgb(63, 63, 63);
pub const SHOVEL_BLADE: Rgba32 = Rgba32::new_grey(185);
pub const SHOVEL_HANDLE: Rgba32 = WOOD;
pub const MAP_BACKGROUND: Rgba32 = Rgba32::hex_rgb(0xceb168);
pub const MAP_FOREGROUND: Rgba32 = Rgba32::new_rgb(0, 0, 0);
pub const WEATHER_REPORT_BACKGROUND: Rgba32 = Rgba32::new_grey(185);
pub const WEATHER_REPORT_FOREGROUND: Rgba32 = Rgba32::new_rgb(0, 0, 0);
pub const LANTERN_HANDLE: Rgba32 = Rgba32::new_rgb(255, 255, 255);
pub const LANTERN_LIGHT: Rgba32 = Rgba32::new_rgb(255, 255, 0);
pub const CROWBAR_SHAFT: Rgba32 = Rgba32::new_rgb(185, 63, 63);
pub const CROWBAR_TIP: Rgba32 = Rgba32::new_rgb(127, 127, 127);
pub const DITCH_FOREGROUND: Rgba32 = Rgba32::hex_rgb(0x372405);
pub const DITCH_BACKGROUND: Rgba32 = Rgba32::hex_rgb(0x291b04);
