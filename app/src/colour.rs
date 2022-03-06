use rgb_int::Rgba32;

pub const CURSOR: Rgba32 = Rgba32::new(255, 255, 0, 64);
pub const GROUND_BACKGROUND: Rgba32 = Rgba32::new_grey(0);
pub const FLOOR_BACKGROUND: Rgba32 = Rgba32::new_grey(0);
pub const FLOOR_FOREGROUND: Rgba32 = Rgba32::hex_rgb(0x8b602c);
pub const WALL_FRONT: Rgba32 = Rgba32::new_rgb(0xD0, 0x8C, 0x15);
pub const WINDOWS: Rgba32 = Rgba32::new_rgb(0xBE, 0xED, 0xFF);
pub const DOOR: Rgba32 = Rgba32::hex_rgb(0x634015);
pub const DOOR_BORDER: Rgba32 = Rgba32::hex_rgb(0x362d11);
pub const PLAYER: Rgba32 = Rgba32::new_grey(255);
pub const WOOD: Rgba32 = Rgba32::hex_rgb(0x362d11);
pub const LEAF: Rgba32 = Rgba32::hex_rgb(0x376315);
