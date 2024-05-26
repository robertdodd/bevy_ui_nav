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

/// Event emitted to lock/unlock the nav request systems. Event handling will be blocked while locked.
///
/// This event should be emitted by the user when they wish to lock/un-lock navigation.
#[derive(Event, PartialEq, Eq, Debug)]
pub enum UiNavLockEvent {
    Lock,
    Unlock,
}

/// Event emitted when a focusable is clicked.
///
/// This event is sent by this plugin and should be handled by the user.
#[derive(Event)]
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
}

/// Event used internally to move focus in a specific direction.
///
/// This event is sent internally in response to a `NavRequest::Movement` event.
#[derive(Event)]
pub(crate) struct InternalFocusMoveEvent(pub UiNavDirection);

/// Event used internally to handle action buttons being pressed/released.
///
/// This event is sent internally in response to a `NavRequest::ActionPress` or `NavRequest::ActionRelease` event.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalActionButtonEvent(pub PressType);

/// Event used internally to refresh focusables in the current menu.
///
/// This event should be emitted when a new focusable is added, a focusable is enabled/disabled, a new menu is added
/// or any other action that may result in no current focusable.
#[derive(Event)]
pub(crate) struct InternalRefreshEvent;

/// Event used to set focus to a specific focusable entity.
///
/// This event is used internally, but can also be used if a user wishes to set focus to a specific `Focusable`.
#[derive(Event)]
pub(crate) struct InternalSetFocusEvent {
    pub entity: Entity,
    pub interaction_type: UiNavInteractionType,
}

impl InternalSetFocusEvent {
    pub fn new_button(entity: Entity) -> Self {
        Self {
            entity,
            interaction_type: UiNavInteractionType::Button,
        }
    }
}
