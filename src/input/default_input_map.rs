use bevy::prelude::*;

use crate::input::*;

use super::input_map::Input;

pub(crate) const DEFAULT_INPUT_MAP: &[(ActionType, Input)] = &[
    (ActionType::Up, Input::KeyBoard(KeyCode::ArrowUp)),
    (ActionType::Down, Input::KeyBoard(KeyCode::ArrowDown)),
    (ActionType::Left, Input::KeyBoard(KeyCode::ArrowLeft)),
    (ActionType::Right, Input::KeyBoard(KeyCode::ArrowRight)),
    (ActionType::Action, Input::KeyBoard(KeyCode::Space)),
    (ActionType::Action, Input::KeyBoard(KeyCode::Enter)),
    (ActionType::Cancel, Input::KeyBoard(KeyCode::Escape)),
    (ActionType::Up, Input::GamepadButton(GamepadButton::DPadUp)),
    (
        ActionType::Down,
        Input::GamepadButton(GamepadButton::DPadDown),
    ),
    (
        ActionType::Left,
        Input::GamepadButton(GamepadButton::DPadLeft),
    ),
    (
        ActionType::Right,
        Input::GamepadButton(GamepadButton::DPadRight),
    ),
    (
        ActionType::Action,
        Input::GamepadButton(GamepadButton::South),
    ),
    (
        ActionType::Cancel,
        Input::GamepadButton(GamepadButton::East),
    ),
    (
        ActionType::Up,
        Input::GamepadAxis(GamepadStick::LeftStickUp),
    ),
    (
        ActionType::Down,
        Input::GamepadAxis(GamepadStick::LeftStickDown),
    ),
    (
        ActionType::Left,
        Input::GamepadAxis(GamepadStick::LeftStickLeft),
    ),
    (
        ActionType::Right,
        Input::GamepadAxis(GamepadStick::LeftStickRight),
    ),
];
