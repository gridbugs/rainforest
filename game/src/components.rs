use crate::visibility::Light;
use entity_table::declare_entity_module;
use grid_2d::coord_2d::Axis;
use rational::Rational;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::vision_distance::Circle;

declare_entity_module! {
    components {
        tile: Tile,
        opacity: u8,
        solid: (),
        player: Player,
        door_state: DoorState,
        light: Light,
        colour_hint: Rgb24,
        realtime: (),
        bulletin_board: (),
        lake: (),
        water: (),
        grass: (),
        lamp: (),
        house_light: (),
        bed: (),
        height: f64,
        ground: (),
        tree: (),
        end_of_pier: (),
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Ground,
    Tree0,
    Tree1,
    Tree2,
    DoorClosed(Axis),
    DoorOpen(Axis),
    Window(Axis),
    Water,
    RuinsWall,
    RuinsFloor,
    Altar,
    BulletinBoard,
    Lamp,
    LampOff,
    PierFloor,
    Grass,
    FlatGrass,
    Rock,
    Flower,
    Bed,
    ChairLeftFacing,
    ChairRightFacing,
    Teapot,
    Tea,
    Gumboots,
    Umbrella,
    Shovel,
    Map,
    WeatherReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {}

pub fn make_player() -> EntityData {
    EntityData {
        tile: Some(Tile::Player),
        player: Some(Player {}),
        light: Some(Light {
            colour: Rgb24::new_grey(185),
            vision_distance: Circle::new_squared(200),
            diminish: Rational {
                numerator: 1,
                denominator: 50,
            },
        }),
        ..Default::default()
    }
}
