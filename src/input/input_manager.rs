use bevy::{platform::collections::HashMap, prelude::*};

use crate::prelude::UiNavDirection;

use super::{
    action_map::{Action, ActionRepeat},
    events::ActionEvents,
    input_map::Input,
    GamepadDevice, InputReader, DEFAULT_INPUT_MAP,
};

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

#[derive(Debug, Resource)]
pub struct InputManager {
    pub(crate) input_map: HashMap<ActionType, Vec<Input>>,
    pub(crate) actions: HashMap<ActionType, Action>,
    pub(crate) repeat_delay: f32,
    pub(crate) repeat_interval: f32,
    pub(crate) gamepad: GamepadDevice,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::from_input_map(DEFAULT_INPUT_MAP)
    }
}

impl InputManager {
    pub const DEFAULT_REPEAT_DELAY: f32 = 0.75;
    pub const DEFAULT_REPEAT_INTERVAL: f32 = 0.1;

    /// Sets which gamepad device to use.
    pub fn set_gamepad(&mut self, gamepad: GamepadDevice) {
        self.gamepad = gamepad;
    }

    pub(crate) fn update(&mut self, time: &Time<Virtual>, input_reader: &InputReader) {
        // update the state of each action from the input reader
        for (action, inputs) in &mut self.input_map {
            let is_pressed = inputs.iter().any(|input| input_reader.value(*input));
            let action = self.actions.entry(*action).or_insert(Action::new(
                if action.to_direction().is_some() {
                    Some(ActionRepeat::default())
                } else {
                    None
                },
            ));
            action.update(time, is_pressed, self.repeat_delay, self.repeat_interval);
        }
    }

    pub fn from_input_map(mappings: &[(ActionType, Input)]) -> Self {
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
            gamepad: GamepadDevice::default(),
            actions: HashMap::<ActionType, Action>::new(),
            repeat_interval: Self::DEFAULT_REPEAT_INTERVAL,
            repeat_delay: Self::DEFAULT_REPEAT_DELAY,
        }
    }

    pub fn get_events(&self, action: ActionType) -> ActionEvents {
        self.actions
            .get(&action)
            .map(|action| action.events())
            .unwrap_or(ActionEvents::empty())
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
}
