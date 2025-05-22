use bevy::prelude::*;

use super::GamepadStick;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum Input {
    KeyBoard(KeyCode),
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadStick),
}
