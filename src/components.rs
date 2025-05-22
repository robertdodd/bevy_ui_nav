use core::slice;

use bevy::prelude::*;

use crate::types::*;

/// Defines what inputs on a `Focusable` a pressable responds to.
#[derive(Default, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum PressableAction {
    #[default]
    Press,
    Left,
    Right,
    Up,
    Down,
}

impl PressableAction {
    pub fn matches_direction(&self, direction: UiNavDirection) -> bool {
        matches!(
            (*self, direction),
            (PressableAction::Left, UiNavDirection::Left)
                | (PressableAction::Right, UiNavDirection::Right)
                | (PressableAction::Up, UiNavDirection::Up)
                | (PressableAction::Down, UiNavDirection::Down)
        )
    }
}

/// Component marking a pressable button inside a focusable.
#[derive(Component, Default, Debug, Clone)]
#[require(Node, Interaction)]
pub struct Pressable {
    pub action: PressableAction,
    pub _is_pressed_interaction: bool,
    pub _is_pressed_focusable: bool,
    pub _is_hover_interaction: bool,
    pub _is_hover_focusable: bool,
}

impl Pressable {
    pub fn is_pressed(&self) -> bool {
        self._is_pressed_interaction || self._is_pressed_focusable
    }

    pub fn is_hovered(&self) -> bool {
        self._is_hover_interaction || self._is_hover_focusable
    }

    pub fn state(&self) -> PressableState {
        if self.is_pressed() {
            PressableState::Pressed
        } else if self.is_hovered() {
            PressableState::Hovered
        } else {
            PressableState::None
        }
    }

    pub fn new_press() -> Self {
        Self {
            action: PressableAction::Press,
            ..default()
        }
    }
    pub fn new_left() -> Self {
        Self {
            action: PressableAction::Left,
            ..default()
        }
    }
    pub fn new_right() -> Self {
        Self {
            action: PressableAction::Right,
            ..default()
        }
    }
}

/// Component marking a pressable button inside a focusable.
#[derive(Default, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum PressableState {
    #[default]
    None,
    Hovered,
    Pressed,
}

/// Component added to `Pressable` entities with a reference to their parent `Focusable`.
#[derive(Component, Debug)]
#[relationship(relationship_target = Pressables)]
pub struct PressableOf(pub Entity);

/// Component added to `Focusable` entities containing their child `Pressables`s.
#[derive(Component, Debug)]
#[relationship_target(relationship = PressableOf)]
pub struct Pressables(Vec<Entity>);

impl<'a> IntoIterator for &'a Pressables {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Component added to `Focusable` entities with a reference to their parent `NavMenu`.
#[derive(Component, Debug)]
#[relationship(relationship_target = Focusables)]
pub struct FocusableOf(pub Entity);

/// Component added to `NavMenu` entities containing their child `Focusable`s.
#[derive(Component, Debug)]
#[relationship_target(relationship = FocusableOf)]
pub struct Focusables(Vec<Entity>);

impl<'a> IntoIterator for &'a Focusables {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Component defining a menu that contains `Focusable` entities.
#[derive(Component, Debug, Clone, Reflect)]
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

impl Default for NavMenu {
    fn default() -> Self {
        Self {
            is_priority: false,
            is_wrap: true,
            is_locked: false,
        }
    }
}

impl NavMenu {
    pub fn new(is_priority: bool, is_wrap: bool) -> Self {
        Self {
            is_priority,
            is_wrap,
            ..default()
        }
    }

    /// Sets the `is_locked` value to `true` and returns the `NavMenu`.
    /// This will lock navigation to and from the menu except via explicity sending `NavRequest::SetFocus` events.
    pub fn locked(mut self) -> Self {
        self.is_locked = true;
        self
    }

    /// Sets the `is_wrap` value and returns the `NavMenu`.
    pub fn with_wrap(mut self, is_wrap: bool) -> Self {
        self.is_wrap = is_wrap;
        self
    }

    /// Sets the `is_priority` value to `true` and returns the `NavMenu`.
    /// This will cause the menu to take focus as soon as it is spawned.
    pub fn prioritized(mut self) -> Self {
        self.is_priority = true;
        self
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default, Debug, PartialEq, Hash)]
pub enum FocusableAction {
    #[default]
    Press,
    PressXY,
}

impl FocusableAction {
    pub fn matches_direction(&self, direction: UiNavDirection) -> bool {
        matches!(
            (*self, direction),
            (FocusableAction::PressXY, UiNavDirection::Left)
                | (FocusableAction::PressXY, UiNavDirection::Right)
        )
    }

    pub fn matches_action(&self, action: PressableAction) -> bool {
        matches!(
            (*self, action),
            (FocusableAction::PressXY, PressableAction::Left)
                | (FocusableAction::PressXY, PressableAction::Right)
                | (FocusableAction::Press, PressableAction::Press)
        )
    }
}

/// Component which marks a node as focusable.
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component, Default, Debug)]
#[require(Node)]
pub struct Focusable {
    /// What actions the focusable responds to
    pub action: FocusableAction,
    /// Whether pressed via key press
    pub(crate) is_pressed_key: bool,
    /// When true, focus will be given to this entity when it is spawned
    pub(crate) is_priority: bool,
    /// Whether hovered by interaction
    pub(crate) is_hovered_interaction: bool,
    /// Whether the button is disabled, which blocks focus and click events
    pub(crate) is_disabled: bool,
    /// Whether the button is focused
    pub(crate) is_focused: bool,
}

impl Focusable {
    /// Creates a new `Focusable` with `is_priority` set to true.
    pub fn prioritized() -> Self {
        Self {
            is_priority: true,
            ..default()
        }
    }

    /// Sets the `action` value and returns the `Focusable`
    pub fn with_action(mut self, action: FocusableAction) -> Self {
        self.action = action;
        self
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
        self.active() && self.is_pressed_key
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
