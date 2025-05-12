use bevy::prelude::*;

use crate::types::*;

/// Component defining a menu that contains `Focusable` entities.
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component, Default, Debug)]
pub struct NavMenu {
    /// Whether this menu should be made active when it is spawned. Changing this value after it is spawned will not
    /// have any effect.
    pub is_priority: bool,
    /// Whether navigation should warp around to the other side of the menu.
    pub is_wrap: bool,
    /// Whether navigation to/from this menu is locked.
    pub is_locked: bool,
}

impl NavMenu {
    pub fn new(is_priority: bool, is_wrap: bool) -> Self {
        Self {
            is_priority,
            is_wrap,
            ..default()
        }
    }

    /// Sets the `is_locked` value and returns the `NavMenu`.
    pub fn with_locked(mut self, locked: bool) -> Self {
        self.is_locked = locked;
        self
    }
}

/// Component which marks a node as focusable.
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component, Default, Debug)]
pub struct Focusable {
    /// The parent `NavMenu` entity this focusable belongs to
    pub(crate) menu: Option<Entity>,
    /// When true, focus will be given to this entity when it is spawned
    pub(crate) is_priority: bool,
    /// Whether pressed via interaction
    pub(crate) is_pressed_interaction: bool,
    /// Whether the interaction press occurred while the focusable was active
    pub(crate) is_pressed_interaction_from_active: bool,
    /// Whether pressed via key press
    pub(crate) is_pressed_key: bool,
    /// Whether hovered by interaction
    pub(crate) is_hovered_interaction: bool,
    /// Whether the button is disabled, which blocks focus and click events
    pub(crate) is_disabled: bool,
    /// Whether the button is focused
    pub(crate) is_focused: bool,
    /// Whether the button can only be pressed via the mouse. If `true`, focusing on this button will not remove focus
    /// from other buttons.
    pub is_mouse_only: bool,
    /// Whether the button is visible
    pub is_visible: bool,
}

impl Focusable {
    /// Creates a new `Focusable` with `is_priority` set to true.
    pub fn prioritized() -> Self {
        Self {
            is_priority: true,
            ..default()
        }
    }

    /// Sets the `disabled` value to true and returns the `Focusable`
    pub fn disabled(mut self) -> Self {
        self.is_disabled = true;
        self
    }

    /// Sets the `disabled` value and returns the `Focusable`.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    /// Sets the `is_mouse_only` value and returns the `Focusable`.
    pub fn with_mouse_only(mut self, mouse_only: bool) -> Self {
        self.is_mouse_only = mouse_only;
        self
    }

    /// Sets the `is_priority` value and returns the `Focusable`.
    pub fn with_priority(mut self, prioritized: bool) -> Self {
        self.is_priority = prioritized;
        self
    }

    /// Computes the state of this focusable.
    pub fn state(&self) -> FocusState {
        if self.is_disabled {
            FocusState::Disabled
        } else if self.is_pressed() {
            FocusState::FocusPress
        } else if self.is_focused {
            FocusState::Focus
        } else {
            FocusState::None
        }
    }

    pub fn active(&self) -> bool {
        !self.is_disabled && self.is_focused
    }

    /// Returns whether a focusable is pressed.
    pub fn is_pressed(&self) -> bool {
        self.active()
            && (self.is_pressed_key
                || (self.is_pressed_interaction && self.is_pressed_interaction_from_active))
    }

    /// Returns whether a focusable is hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered_interaction && !self.is_disabled
    }

    /// Enables a `Focusable`.
    pub fn enable(&mut self) {
        self.is_disabled = false;
    }

    /// Disables a `Focusable`.
    pub fn disable(&mut self) {
        self.is_disabled = true;
    }
}
