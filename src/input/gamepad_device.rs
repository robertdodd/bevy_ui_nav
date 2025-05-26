use bevy::prelude::*;

/// Associated gamepad.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Reflect)]
pub enum GamepadDevice {
    /// Matches input from any gamepad.
    ///
    /// For an axis, the [`ActionValue`] will be calculated as the sum of inputs from all gamepads.
    /// For a button, the [`ActionValue`] will be `true` if any gamepad has this button pressed.
    ///
    /// [`ActionValue`]: crate::action_value::ActionValue
    #[default]
    Any,
    /// Matches input from specific gamepad.
    Single(Entity),
}

impl From<Entity> for GamepadDevice {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}
