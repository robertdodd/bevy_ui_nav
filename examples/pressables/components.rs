use core::slice;
use std::time::Duration;

use bevy::prelude::*;
use bevy_ui_nav::prelude::*;

/// Event triggered on a UI entity when the [`Interaction`] component on the same entity changes to
/// [`Interaction::Pressed`]. Observe this event to detect e.g. button presses.
#[derive(Event, Debug)]
pub struct OnPressed(pub Entity);

/// Event triggered on a UI entity when the [`Interaction`] component on the same entity changes to
/// [`Interaction::Pressed`]. Observe this event to detect e.g. button presses.
#[derive(Event, Debug)]
pub struct OnPress;

/// Component that tracks whether a pressable is pressed.
#[derive(Component, Default, Debug, Clone)]
pub struct PressablePressed(pub bool);

/// Component added to a pressable when it's pressed animation is in progress. The button is un-pressed and this
/// component is removed when complete.
#[derive(Component)]
pub struct PressAnimation(pub Timer);

impl Default for PressAnimation {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs_f32(0.05), TimerMode::Once))
    }
}

/// Component marking a pressable button inside a focusable.
#[derive(Component, Default, Debug, Clone)]
#[require(Node, Interaction, PressablePressed)]
pub struct Pressable(pub PressableAction);

impl Pressable {
    pub fn new_press() -> Self {
        Self(PressableAction::Press)
    }
    pub fn new_left() -> Self {
        Self(PressableAction::Left)
    }
    pub fn new_right() -> Self {
        Self(PressableAction::Right)
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

impl Pressables {
    pub fn get(&self, index: usize) -> Option<&Entity> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> IntoIterator for &'a Pressables {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
