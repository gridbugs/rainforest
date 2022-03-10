use chargrid::input::{GamepadButton, Input, KeyboardInput};
use direction::CardinalDirection;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInput {
    Direction(CardinalDirection),
    DirectionLong(CardinalDirection),
    Wait,
    WaitLong,
    Examine,
    Get,
    Map,
    WeatherReport,
    Lantern,
}

#[derive(Serialize, Deserialize)]
pub struct Controls {
    keys: BTreeMap<KeyboardInput, AppInput>,
    gamepad: BTreeMap<GamepadButton, AppInput>,
}

impl Default for Controls {
    fn default() -> Self {
        let keys = btreemap![
            KeyboardInput::Left => AppInput::Direction(CardinalDirection::West),
            KeyboardInput::Right => AppInput::Direction(CardinalDirection::East),
            KeyboardInput::Up => AppInput::Direction(CardinalDirection::North),
            KeyboardInput::Down => AppInput::Direction(CardinalDirection::South),
            KeyboardInput::Char('a') => AppInput::Direction(CardinalDirection::West),
            KeyboardInput::Char('d') => AppInput::Direction(CardinalDirection::East),
            KeyboardInput::Char('w') => AppInput::Direction(CardinalDirection::North),
            KeyboardInput::Char('s') => AppInput::Direction(CardinalDirection::South),
            KeyboardInput::Char('h') => AppInput::Direction(CardinalDirection::West),
            KeyboardInput::Char('l') => AppInput::Direction(CardinalDirection::East),
            KeyboardInput::Char('k') => AppInput::Direction(CardinalDirection::North),
            KeyboardInput::Char('j') => AppInput::Direction(CardinalDirection::South),
            KeyboardInput::Char('A') => AppInput::DirectionLong(CardinalDirection::West),
            KeyboardInput::Char('D') => AppInput::DirectionLong(CardinalDirection::East),
            KeyboardInput::Char('W') => AppInput::DirectionLong(CardinalDirection::North),
            KeyboardInput::Char('S') => AppInput::DirectionLong(CardinalDirection::South),
            KeyboardInput::Char('H') => AppInput::DirectionLong(CardinalDirection::West),
            KeyboardInput::Char('L') => AppInput::DirectionLong(CardinalDirection::East),
            KeyboardInput::Char('K') => AppInput::DirectionLong(CardinalDirection::North),
            KeyboardInput::Char('J') => AppInput::DirectionLong(CardinalDirection::South),
            KeyboardInput::Char('x') => AppInput::Examine,
            KeyboardInput::Char('g') => AppInput::Get,
            KeyboardInput::Char('m') => AppInput::Map,
            KeyboardInput::Char('r') => AppInput::WeatherReport,
            KeyboardInput::Char(' ') => AppInput::Wait,
            KeyboardInput::Char('.') => AppInput::WaitLong,
            KeyboardInput::Char('f') => AppInput::Lantern,
        ];
        let gamepad = btreemap![
            GamepadButton::DPadLeft => AppInput::Direction(CardinalDirection::West),
            GamepadButton::DPadRight => AppInput::Direction(CardinalDirection::East),
            GamepadButton::DPadUp => AppInput::Direction(CardinalDirection::North),
            GamepadButton::DPadDown => AppInput::Direction(CardinalDirection::South),
            GamepadButton::Select => AppInput::Wait,
            GamepadButton::North => AppInput::Get,
            GamepadButton::RightBumper => AppInput::Examine,
        ];
        Self { keys, gamepad }
    }
}

impl Controls {
    pub fn get(&self, input: Input) -> Option<AppInput> {
        match input {
            Input::Keyboard(keyboard_input) => self.keys.get(&keyboard_input).cloned(),
            Input::Gamepad(gamepad_input) => self.gamepad.get(&gamepad_input.button).cloned(),
            Input::Mouse(_) => None,
        }
    }

    pub fn get_direction(&self, input: Input) -> Option<CardinalDirection> {
        self.get(input).and_then(|app_input| match app_input {
            AppInput::Direction(direction) => Some(direction),
            _ => None,
        })
    }
}
