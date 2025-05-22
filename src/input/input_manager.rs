use bevy::{platform::collections::HashMap, prelude::*};

use crate::prelude::{PressType, UiNavDirection};

use super::{input_map::Input, GamepadDevice, InputReader, DEFAULT_INPUT_MAP};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum ActionType {
    Up,
    Down,
    Left,
    Right,
    Action,
    Cancel,
}

impl ActionType {
    pub fn to_direction(&self) -> Option<UiNavDirection> {
        match self {
            ActionType::Up => Some(UiNavDirection::Up),
            ActionType::Down => Some(UiNavDirection::Down),
            ActionType::Left => Some(UiNavDirection::Left),
            ActionType::Right => Some(UiNavDirection::Right),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum GamepadStick {
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource, Debug)]
pub struct UiNavInputManager {
    pub(crate) input_map: HashMap<ActionType, Vec<Input>>,
    pub(crate) current_state: HashMap<ActionType, bool>,
    pub(crate) previous_state: HashMap<ActionType, bool>,
    pub(crate) stick_tolerance: f32,
    pub(crate) stick_snap_tolerance: f32,
    pub(crate) gamepad: GamepadDevice,
}

impl Default for UiNavInputManager {
    fn default() -> Self {
        Self::from_input_map(DEFAULT_INPUT_MAP, 0.1, 0.9)
    }
}

impl UiNavInputManager {
    /// Sets which gamepad device to use.
    pub fn set_gamepad(&mut self, gamepad: GamepadDevice) {
        self.gamepad = gamepad;
    }

    pub(crate) fn update(&mut self, input_reader: &InputReader) {
        // update the previous state, and clear current state
        self.reset();

        // update the state of each action from the input reader
        for (action, inputs) in &mut self.input_map {
            let is_pressed = inputs.iter().any(|input| input_reader.value(*input));
            self.current_state.insert(*action, is_pressed);
        }
    }

    /// update the previous state, and clear current state
    pub fn reset(&mut self) {
        self.previous_state.clone_from(&self.current_state);
        for v in self.current_state.values_mut() {
            *v = false;
        }
    }

    pub fn from_input_map(
        mappings: &[(ActionType, Input)],
        stick_tolerance: f32,
        stick_snap_tolerance: f32,
    ) -> Self {
        let mut input_map = HashMap::<ActionType, Vec<Input>>::new();
        for (key, value) in mappings.iter() {
            if let Some(entry) = input_map.get_mut(key) {
                entry.push(*value);
            } else {
                input_map.insert(*key, vec![*value]);
            }
        }

        Self {
            input_map,
            current_state: HashMap::<ActionType, bool>::new(),
            previous_state: HashMap::<ActionType, bool>::new(),
            stick_tolerance,
            stick_snap_tolerance,
            gamepad: GamepadDevice::default(),
        }
    }

    pub fn pressed(&self, action: ActionType) -> bool {
        self.current_state.get(&action).copied().unwrap_or(false)
    }

    pub fn was_pressed(&self, action: ActionType) -> bool {
        self.previous_state.get(&action).copied().unwrap_or(false)
    }

    pub fn just_pressed(&self, action: ActionType) -> bool {
        self.pressed(action) && !self.was_pressed(action)
    }

    pub fn just_released(&self, action: ActionType) -> bool {
        !self.pressed(action) && self.was_pressed(action)
    }

    pub fn direction(&self) -> Option<UiNavDirection> {
        Self::compute_direction(&self.current_state)
    }

    pub fn previous_direction(&self) -> Option<UiNavDirection> {
        Self::compute_direction(&self.previous_state)
    }

    pub fn is_direction_released(&self) -> bool {
        self.previous_direction().is_some() && self.direction().is_none()
    }

    pub fn compute_direction(state: &HashMap<ActionType, bool>) -> Option<UiNavDirection> {
        let left = state.get(&ActionType::Left).copied().unwrap_or(false);
        let right = state.get(&ActionType::Right).copied().unwrap_or(false);
        let up = state.get(&ActionType::Up).copied().unwrap_or(false);
        let down = state.get(&ActionType::Down).copied().unwrap_or(false);

        if down && left {
            Some(UiNavDirection::DownLeft)
        } else if down && right {
            Some(UiNavDirection::DownRight)
        } else if up && left {
            Some(UiNavDirection::UpLeft)
        } else if up && right {
            Some(UiNavDirection::UpRight)
        } else if down {
            Some(UiNavDirection::Down)
        } else if up {
            Some(UiNavDirection::Up)
        } else if left {
            Some(UiNavDirection::Left)
        } else if right {
            Some(UiNavDirection::Right)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_press_type(&self, action: ActionType) -> Option<PressType> {
        if self.just_pressed(action) {
            Some(PressType::Press)
        } else if self.just_released(action) {
            Some(PressType::Release)
        } else {
            None
        }
    }
}

// fn get_gamepad_axes(
//     gamepad: &Gamepad,
//     stick: GamepadStick,
//     stick_tolerance: f32,
//     stick_snap_tolerance: f32,
// ) -> Vec2 {
//     let axes = match stick {
//         GamepadStick::Left => gamepad.left_stick(),
//         GamepadStick::Right => gamepad.right_stick(),
//     };

//     let abs_x = axes.x.abs();
//     let abs_y = axes.y.abs();
//     let mut result = Vec2::new(
//         if abs_x > stick_tolerance { axes.x } else { 0. },
//         if abs_y > stick_tolerance { axes.y } else { 0. },
//     );

//     // Clear small values when moving diagonlly. For example, the user may think they are pressing to the left, but
//     // there could be a small up/down value that makes the navigation feel wrong.
//     if abs_x > stick_tolerance
//         && abs_y > stick_tolerance
//         && (abs_x - abs_y).abs() < stick_snap_tolerance
//     {
//         if abs_x > abs_y {
//             result.y = 0.;
//         } else {
//             result.x = 0.;
//         }
//     }

//     result
// }

// pub fn update_input_manager(
//     input: &mut UiNavInputManager,
//     gamepads: &Query<(Entity, &Gamepad)>,
//     // gamepad_buttons: &ButtonInput<GamepadButton>,
//     // gamepad_axis: &Axis<GamepadAxis>,
// ) {
//     // update the previous state, and clear current state
//     input.previous_state.clone_from(&input.current_state);
//     input
//         .previous_direction
//         .clone_from(&input.current_direction);
//     for v in input.current_state.values_mut() {
//         *v = false;
//     }

//     // update the current state
//     for action in input.input_map.iter() {
//         match action {
//             InputMapping::GamepadButton {
//                 gamepad,
//                 button,
//                 action,
//             } => {
//                 let is_pressed = gamepads
//                     .iter()
//                     .filter(|(e, _)| gamepad.is_none() || Some(*e) == *gamepad)
//                     .any(|(_, g)| g.pressed(*button));

//                 if is_pressed {
//                     input.current_state.insert(*action, is_pressed);
//                 }
//             }
//             InputMapping::GamepadAxes { gamepad, stick } => {
//                 let axes = gamepads
//                     .iter()
//                     .filter(|(e, _)| gamepad.is_none() || Some(*e) == *gamepad)
//                     .map(|(_, g)| {
//                         get_gamepad_axes(
//                             g,
//                             *stick,
//                             input.stick_tolerance,
//                             input.stick_snap_tolerance,
//                         )
//                     })
//                     .fold(Vec2::ZERO, |acc, e| {
//                         let acc_dist = acc.length();
//                         let e_dist = e.length();
//                         if e_dist > acc_dist {
//                             e
//                         } else {
//                             acc
//                         }
//                     });

//                 if axes.x > input.stick_tolerance {
//                     input.current_state.insert(ActionType::Right, true);
//                 } else if axes.x < -input.stick_tolerance {
//                     input.current_state.insert(ActionType::Left, true);
//                 }
//                 if axes.y > input.stick_tolerance {
//                     input.current_state.insert(ActionType::Up, true);
//                 } else if axes.y < -input.stick_tolerance {
//                     input.current_state.insert(ActionType::Down, true);
//                 }
//             }
//             _ => (),
//         }
//     }

//     // Set current direction
//     let left = input.pressed(ActionType::Left);
//     let right = input.pressed(ActionType::Right);
//     let up = input.pressed(ActionType::Up);
//     let down = input.pressed(ActionType::Down);
//     input.current_direction = if down && left {
//         Some(UiNavDirection::DownLeft)
//     } else if down && right {
//         Some(UiNavDirection::DownRight)
//     } else if up && left {
//         Some(UiNavDirection::UpLeft)
//     } else if up && right {
//         Some(UiNavDirection::UpRight)
//     } else if down {
//         Some(UiNavDirection::Down)
//     } else if up {
//         Some(UiNavDirection::Up)
//     } else if left {
//         Some(UiNavDirection::Left)
//     } else if right {
//         Some(UiNavDirection::Right)
//     } else {
//         None
//     };
// }
