use bevy::reflect::Reflect;

/// Type describing whether an interaction can from a user or internally.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
pub enum UiNavInteractionType {
    /// The interaction was sent automatically from this plugin, most likely focusing on a newly added focusable.
    Auto,
    /// The interaction was sent manually via an event
    Manual,
    /// The interaction was from the mouse
    Mouse,
    /// The interaction was from a button press (Gamepad or keyboard)
    Button,
}

/// Type used to describe the state of a button, i.e. whether it is pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
pub enum PressType {
    Release,
    Press,
}

/// Type which describes the focus state
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
pub enum FocusState {
    #[default]
    None, // not focused
    Focus,      // focused
    FocusPress, // active and pressed
    Disabled,   // disabled
}

impl FocusState {
    pub fn active(&self) -> bool {
        matches!(*self, FocusState::Focus | FocusState::FocusPress)
    }
}

/// Type describing a navigation direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
pub enum UiNavDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}
