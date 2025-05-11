use bevy::prelude::*;

use crate::input::*;

pub(crate) const DEFAULT_INPUT_MAP: &[InputMapping] = &[
    InputMapping::Key {
        keycode: KeyCode::ArrowUp,
        action: ActionType::Up,
    },
    InputMapping::Key {
        keycode: KeyCode::ArrowDown,
        action: ActionType::Down,
    },
    InputMapping::Key {
        keycode: KeyCode::ArrowLeft,
        action: ActionType::Left,
    },
    InputMapping::Key {
        keycode: KeyCode::ArrowRight,
        action: ActionType::Right,
    },
    InputMapping::Key {
        keycode: KeyCode::Space,
        action: ActionType::Action,
    },
    InputMapping::Key {
        keycode: KeyCode::Enter,
        action: ActionType::Action,
    },
    InputMapping::Key {
        keycode: KeyCode::Escape,
        action: ActionType::Cancel,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::DPadUp,
        action: ActionType::Up,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::DPadDown,
        action: ActionType::Down,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::DPadLeft,
        action: ActionType::Left,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::DPadRight,
        action: ActionType::Right,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::South,
        action: ActionType::Action,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButton::East,
        action: ActionType::Cancel,
    },
    InputMapping::GamepadAxes {
        gamepad: None,
        stick: GamepadStick::Left,
    },
];
