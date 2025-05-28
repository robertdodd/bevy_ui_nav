use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{prelude::*, types::UiNavDirection};

/// Input state for actions.
///
/// Actions can read input values and optionally consume them without affecting Bevy input resources.
#[derive(SystemParam)]
pub(crate) struct Queries<'w, 's> {
    pub focusables: Query<
        'w,
        's,
        (
            &'static mut Focusable,
            &'static FocusableOf,
            &'static ComputedNode,
            &'static GlobalTransform,
            &'static InheritedVisibility,
        ),
    >,
    pub menus: Query<
        'w,
        's,
        (
            Entity,
            &'static NavMenu,
            &'static Focusables,
            &'static mut LastFocusable,
        ),
    >,
    pub nav_state: ResMut<'w, UiNavState>,
    pub lock_state: ResMut<'w, LockState>,
}

impl Queries<'_, '_> {
    pub fn get_current_focusable(&self) -> Option<Entity> {
        // Find the current focused entity
        self.nav_state.menu.and_then(|e| {
            if let Ok((_, _, focusables, _)) = self.menus.get(e) {
                focusables
                    .iter()
                    .find(|e| self.focusables.get(*e).is_ok_and(|(f, ..)| f.is_focused))
            } else {
                None
            }
        })
    }

    /// Set focus to a new focusable or menu.
    pub fn set_focus(&mut self, from: Option<Entity>, to: Entity) -> Option<Entity> {
        // set focus to the new focusable
        let new_focusable =
            if let Ok((mut focusable, focusable_of, ..)) = self.focusables.get_mut(to) {
                // check the focusable is not disabled
                if focusable.is_disabled {
                    warn!("tried to set focus to a disabled focusable");
                    return None;
                }
                focusable.is_focused = true;

                self.nav_state.menu = Some(focusable_of.0);

                // track the last focusable in the menu
                if let Ok((.., mut last_focusable)) = self.menus.get_mut(focusable_of.0) {
                    last_focusable.0 = Some(to);
                }

                Some(to)
            } else if let Ok((_, _, focusables, last_focusable)) = self.menus.get(to) {
                // find the first non-disabled focusable in the nav menu to focus on.
                // try find a prioritized focusable first, otherwise use the first one.
                let first_focusable = if let Some(last_focusable) =
                    last_focusable.0.filter(|e| self.focusables.contains(*e))
                {
                    Some(last_focusable)
                } else {
                    focusables
                        .iter()
                        .filter_map(|e| {
                            self.focusables
                                .get(e)
                                .map(|(focusable, ..)| {
                                    (e, focusable.is_priority, focusable.is_disabled)
                                })
                                .ok()
                        })
                        .filter(|(_, _, is_disabled)| !is_disabled)
                        .reduce(|acc, e| if e.1 { e } else { acc })
                        .map(|(e, ..)| e)
                };

                // focus on the menu
                self.nav_state.menu = Some(to);

                // if we found a focusable in the menu, focus on it
                if let Some(focusable_entity) = first_focusable {
                    // track the last focusable in the menu
                    if let Ok((.., mut last_focusable)) = self.menus.get_mut(to) {
                        last_focusable.0 = Some(focusable_entity);
                    }

                    // mark focusable as focused
                    if let Ok((mut focusable, ..)) = self.focusables.get_mut(focusable_entity) {
                        focusable.is_focused = true;
                    }
                    Some(focusable_entity)
                } else {
                    warn!("Tried to set focus to nav menu, but it contained no active focusables.");
                    None
                }
            } else {
                warn!("Tried to set focus to invalid entity.");
                None
            };

        // remove focus from the old focusable
        if new_focusable != from {
            if let Some(from) = from {
                if let Ok((mut focusable, _, _, _, _)) = self.focusables.get_mut(from) {
                    focusable.is_focused = false;
                }
            }
        }

        new_focusable
    }

    /// Check whether a focusable captures a specific movement direction.
    pub fn get_focusable_captures_movement(
        &self,
        focused: Entity,
        direction: UiNavDirection,
    ) -> bool {
        // Check that the current focusable does not consume these events
        let action = match direction {
            UiNavDirection::Left => Some(PressableAction::Left),
            UiNavDirection::Right => Some(PressableAction::Right),
            _ => None,
        };
        if let Some(action) = action {
            self.focusables
                .get(focused)
                .is_ok_and(|(f, ..)| f.action.matches_action(action))
        } else {
            false
        }
    }
}
