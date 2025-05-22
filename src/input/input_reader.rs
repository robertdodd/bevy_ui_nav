use bevy::{ecs::system::SystemParam, prelude::*};

use super::{input_map::Input, GamepadDevice, GamepadStick};

/// Input state for actions.
///
/// Actions can read input values and optionally consume them without affecting Bevy input resources.
#[derive(SystemParam)]
pub(crate) struct InputReader<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    gamepads: Query<'w, 's, &'static Gamepad>,
    gamepad_device: Local<'s, GamepadDevice>,
}

impl InputReader<'_, '_> {
    /// Assigns a gamepad from which [`Self::value`] should read input.
    pub(crate) fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        *self.gamepad_device = gamepad.into();
    }

    /// Returns the [`ActionValue`] for the given [`Input`].
    ///
    /// See also [`Self::consume`] and [`Self::set_gamepad`].
    pub(crate) fn value(&self, input: Input) -> bool {
        match input {
            Input::KeyBoard(key) => self.keys.pressed(key),
            Input::GamepadButton(button) => {
                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self
                        .gamepads
                        .iter()
                        .filter_map(|gamepad| gamepad.get(button))
                        .find(|&value| value != 0.0),
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .ok()
                        .and_then(|gamepad| gamepad.get(button)),
                };

                value.is_some_and(|v| v > 0.)
            }
            Input::GamepadAxis(stick) => match *self.gamepad_device {
                GamepadDevice::Any => self
                    .gamepads
                    .iter()
                    .any(|gamepad| get_input_axis(gamepad, stick)),
                GamepadDevice::Single(entity) => self
                    .gamepads
                    .get(entity)
                    .ok()
                    .is_some_and(|gamepad| get_input_axis(gamepad, stick)),
            },
        }
    }
}

fn get_input_axis(gamepad: &Gamepad, stick: GamepadStick) -> bool {
    let axis = match stick {
        GamepadStick::LeftStickUp => GamepadAxis::LeftStickY,
        GamepadStick::LeftStickDown => GamepadAxis::LeftStickY,
        GamepadStick::LeftStickLeft => GamepadAxis::LeftStickX,
        GamepadStick::LeftStickRight => GamepadAxis::LeftStickX,
        GamepadStick::RightStickUp => GamepadAxis::RightStickY,
        GamepadStick::RightStickDown => GamepadAxis::RightStickY,
        GamepadStick::RightStickLeft => GamepadAxis::RightStickX,
        GamepadStick::RightStickRight => GamepadAxis::RightStickX,
    };
    let value = gamepad.get_unclamped(axis);
    if let Some(value) = value {
        match stick {
            GamepadStick::LeftStickLeft => value < 0.,
            GamepadStick::LeftStickRight => value > 0.,
            GamepadStick::RightStickLeft => value < 0.,
            GamepadStick::RightStickRight => value > 0.,
            GamepadStick::LeftStickUp => value < 0.,
            GamepadStick::LeftStickDown => value > 0.,
            GamepadStick::RightStickUp => value < 0.,
            GamepadStick::RightStickDown => value > 0.,
        }
    } else {
        false
    }
}
