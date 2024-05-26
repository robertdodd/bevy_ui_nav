use bevy::{prelude::*, utils::HashMap};

use crate::prelude::{PressType, UiNavDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    Up,
    Down,
    Left,
    Right,
    Action,
    Cancel,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadStick {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMapping {
    Key {
        keycode: KeyCode,
        action: ActionType,
    },
    GamepadButton {
        gamepad: Option<Gamepad>,
        button: GamepadButtonType,
        action: ActionType,
    },
    GamepadAxes {
        gamepad: Option<Gamepad>,
        stick: GamepadStick,
    },
}

#[derive(Debug, Resource)]
pub struct UiNavInputManager {
    input_map: Vec<InputMapping>,
    current_state: HashMap<ActionType, bool>,
    previous_state: HashMap<ActionType, bool>,
    current_direction: Option<UiNavDirection>,
    stick_tolerance: f32,
    stick_snap_tolerance: f32,
}

impl UiNavInputManager {
    pub fn from_input_map(
        input_map: &[InputMapping],
        stick_tolerance: f32,
        stick_snap_tolerance: f32,
    ) -> Self {
        Self {
            input_map: input_map.to_vec(),
            current_state: HashMap::<ActionType, bool>::new(),
            previous_state: HashMap::<ActionType, bool>::new(),
            current_direction: None,
            stick_tolerance,
            stick_snap_tolerance,
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
        self.current_direction
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

fn get_gamepad_axes(
    gamepad: Gamepad,
    gamepad_axis: &Axis<GamepadAxis>,
    stick: GamepadStick,
    stick_tolerance: f32,
    stick_snap_tolerance: f32,
) -> Vec2 {
    let x_axis_type = match stick {
        GamepadStick::Left => GamepadAxisType::LeftStickX,
        GamepadStick::Right => GamepadAxisType::RightStickX,
    };
    let x = gamepad_axis
        .get(GamepadAxis::new(gamepad, x_axis_type))
        .unwrap();

    let y_axis_type = match stick {
        GamepadStick::Left => GamepadAxisType::LeftStickY,
        GamepadStick::Right => GamepadAxisType::RightStickY,
    };
    let y = gamepad_axis
        .get(GamepadAxis::new(gamepad, y_axis_type))
        .unwrap();

    let abs_x = x.abs();
    let abs_y = y.abs();
    let mut result = Vec2::new(
        if abs_x > stick_tolerance { x } else { 0. },
        if abs_y > stick_tolerance { y } else { 0. },
    );

    // Clear small values when moving diagonlly. For example, the user may think they are pressing to the left, but
    // there could be a small up/down value that makes the navigation feel wrong.
    if abs_x > stick_tolerance
        && abs_y > stick_tolerance
        && (abs_x - abs_y).abs() < stick_snap_tolerance
    {
        if abs_x > abs_y {
            result.y = 0.;
        } else {
            result.x = 0.;
        }
    }

    result
}

pub fn update_input_manager(
    input: &mut UiNavInputManager,
    keys: &ButtonInput<KeyCode>,
    gamepads: &Gamepads,
    gamepad_buttons: &ButtonInput<GamepadButton>,
    gamepad_axis: &Axis<GamepadAxis>,
) {
    // update the previous state, and clear current state
    input.previous_state = input.current_state.clone();
    for v in input.current_state.values_mut() {
        *v = false;
    }

    // update the current state
    for action in input.input_map.iter() {
        match action {
            InputMapping::Key { keycode, action } => {
                if keys.pressed(*keycode) {
                    input.current_state.insert(*action, true);
                }
            }
            InputMapping::GamepadButton {
                gamepad,
                button,
                action,
            } => {
                let is_pressed = if let Some(gamepad) = gamepad {
                    let gamepad_button = GamepadButton::new(*gamepad, *button);
                    gamepad_buttons.pressed(gamepad_button)
                } else {
                    gamepads.iter().any(|gamepad| {
                        let gamepad_button = GamepadButton::new(gamepad, *button);
                        gamepad_buttons.pressed(gamepad_button)
                    })
                };
                if is_pressed {
                    input.current_state.insert(*action, is_pressed);
                }
            }
            InputMapping::GamepadAxes { gamepad, stick } => {
                let axes = if let Some(gamepad) = gamepad {
                    get_gamepad_axes(
                        *gamepad,
                        gamepad_axis,
                        *stick,
                        input.stick_tolerance,
                        input.stick_snap_tolerance,
                    )
                } else {
                    gamepads
                        .iter()
                        .map(|gamepad| {
                            get_gamepad_axes(
                                gamepad,
                                gamepad_axis,
                                *stick,
                                input.stick_tolerance,
                                input.stick_snap_tolerance,
                            )
                        })
                        .fold(Vec2::ZERO, |acc, e| {
                            let acc_dist = acc.length();
                            let e_dist = e.length();
                            if e_dist > acc_dist {
                                e
                            } else {
                                acc
                            }
                        })
                };

                if axes.x > input.stick_tolerance {
                    input.current_state.insert(ActionType::Right, true);
                } else if axes.x < -input.stick_tolerance {
                    input.current_state.insert(ActionType::Left, true);
                }
                if axes.y > input.stick_tolerance {
                    input.current_state.insert(ActionType::Up, true);
                } else if axes.y < -input.stick_tolerance {
                    input.current_state.insert(ActionType::Down, true);
                }
            }
        }
    }

    // Set current direction
    let left = input.pressed(ActionType::Left);
    let right = input.pressed(ActionType::Right);
    let up = input.pressed(ActionType::Up);
    let down = input.pressed(ActionType::Down);
    input.current_direction = if down && left {
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
    };
}
