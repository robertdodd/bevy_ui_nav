use bevy::prelude::*;

/// System set in which the UI navigation systems run.
///
/// Systems that handle navigation events should be scheduled after this set: `my_system.after(UiNavSet)`
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct UiNavSet;

/// Resource holding the global menu navigation state.
#[derive(Resource, Default, Debug)]
pub struct UiNavState {
    /// Whether navigation state is locked
    pub(crate) locked: bool,
    /// The current active `Menu`
    pub(crate) menu: Option<Entity>,
}

impl UiNavState {
    pub fn locked(&self) -> bool {
        self.locked
    }
}

/// Resource containing settings for how the UI Navigation plugin behaves.
#[derive(Resource, Debug)]
pub struct UiNavSettings {
    /// Number of seconds before emitting a navigation event when a direction button is held. This is the slowest
    /// speed.
    pub movement_speed_slow: f32,
    /// Number of seconds before emitting a navigation event when a direction button is held. This is the the fastest
    /// speed.
    pub movement_speed_fast: f32,
    /// Number of seconds a direciton button must be held to reach maximum navigation speed. The navigation speed is
    /// `movement_speed_slow` when first pressed, and is `movement_speed_fast` when we have held it for the value of
    /// `movement_acceleration_time`.
    pub movement_acceleration_time: f32,
}

impl Default for UiNavSettings {
    fn default() -> Self {
        Self {
            movement_speed_slow: 0.5,
            movement_speed_fast: 0.1,
            movement_acceleration_time: 1.0,
        }
    }
}
