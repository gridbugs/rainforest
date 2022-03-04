use rgb_int::Rgba32;

pub const CURSOR: Rgba32 = Rgba32::new(255, 255, 0, 64);
pub const GROUND_BACKGROUND: Rgba32 = Rgba32::new_rgb(0x10, 0x80, 0x10);
pub const GROUND_FOREGROUND: Rgba32 = Rgba32::new_rgb(0x10, 0x10, 0x10);
pub const FLOOR_BACKGROUND: Rgba32 = Rgba32::new_rgb(0xD4, 0xB8, 0x88);
pub const FLOOR_FOREGROUND: Rgba32 = Rgba32::new_rgb(0xB0, 0x8C, 0x4C);
pub const WALL_TOP: Rgba32 = Rgba32::new_rgb(0x49, 0x2E, 0x00);
pub const WALL_FRONT: Rgba32 = Rgba32::new_rgb(0xD0, 0x8C, 0x15);
pub const WINDOWS: Rgba32 = Rgba32::new_rgb(0xBE, 0xED, 0xFF);
pub const STRIPE: Rgba32 = Rgba32::new_rgb(0xFF, 0xBE, 0x4C);
pub const DOOR: Rgba32 = Rgba32::new_rgb(0x88, 0x88, 0x88);
pub const DOOR_BORDER: Rgba32 = Rgba32::new_grey(0x33);
pub const PLAYER: Rgba32 = Rgba32::new_grey(0x00);
