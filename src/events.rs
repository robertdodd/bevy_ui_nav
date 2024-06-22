use bevy::prelude::*;

use crate::types::*;

/// Event sent when a new focusable is focused.
///
/// A user can use these events to react to UI Navigation events, for example to play a sound when focus changes.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiNavFocusChangedEvent {
    pub entity: Entity,
    pub interaction_type: UiNavInteractionType,
}

/// Event emitted when the "Cancel" key is pressed. The entity is the menu.
///
/// This event is emitted by this plugin and should be handled by the user if they wish to handle cancel events in a
/// menu.
#[derive(Event, Debug)]
pub struct UiNavCancelEvent(pub Entity);

/// Event emitted when a focusable is clicked.
///
/// This event is sent by this plugin and should be handled by the user.
#[derive(Event, Debug)]
pub struct UiNavClickEvent(pub Entity);

/// Event used internally to trigger a UI navigation request.
///
/// These events are emitted in response to keyboard or gamepad button input.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavRequest {
    /// Set focus on a `Focusable`
    SetFocus {
        entity: Entity,
        interaction_type: UiNavInteractionType,
    },
    /// Move focus in a specific direction
    Movement(UiNavDirection),
    /// Press the action key
    ActionPress,
    /// Release the action key
    ActionRelease,
    /// Cancel key pressed for first time
    Cancel,
    /// Refresh focus state if menus have changed
    Refresh,
    /// Lock the nav request systems. Event handling will be blocked while locked. No effect if already locked.
    Lock,
    /// Unlock the nav request systems and enable event handling again. No effect if already unlocked.
    Unlock,
}
