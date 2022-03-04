use entity_table::declare_entity_module;
use grid_2d::coord_2d::Axis;
use serde::{Deserialize, Serialize};

declare_entity_module! {
    components {
        tile: Tile,
        opacity: u8,
        solid: (),
        player: Player,
        door_state: DoorState,
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    DoorClosed(Axis),
    DoorOpen(Axis),
    Stairs,
    Window(Axis),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {}
