use bevy::prelude::*;

use crate::{
    components::*, events::*, input::*, queries::Queries, resolve::resolve_2d, resources::*,
    types::*, utils::find_parent,
};

pub struct BevyUiNavPlugin;

impl Plugin for BevyUiNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FocusablePressed>()
            .add_event::<UiNavCancelEvent>()
            .add_event::<NavRequest>()
            .add_event::<UiNavFocusChangedEvent>()
            .init_resource::<UiNavState>()
            .init_resource::<UiNavSettings>()
            .init_resource::<InputManager>()
            .add_systems(PreUpdate, (setup_new_menus, setup_new_focusables))
            .add_systems(
                Update,
                (
                    (
                        handle_menu_removed.run_if(any_component_removed::<NavMenu>),
                        update_input,
                        handle_focusable_changed,
                    )
                        .before(UiNavSet),
                    handle_nav_requests
                        .run_if(on_event::<NavRequest>)
                        .in_set(UiNavSet),
                ),
            );
    }
}

/// System that initializes newly added menus
fn setup_new_menus(
    query: Query<(Entity, &NavMenu), Added<NavMenu>>,
    menu_query: Query<(), With<NavMenu>>,
    nav_state: Res<UiNavState>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    if query.is_empty() {
        return;
    }

    let has_current_menu = nav_state.menu.is_some_and(|e| menu_query.contains(e));

    // The new target menu and whether it is prioritized. We need to track and handle this outside the iterator
    // because multiple menus can be spawned simultaneously.
    let mut new_focus: Option<(Entity, bool)> = None;
    for (entity, menu) in query.iter() {
        if let (true, false) = (
            menu.is_priority,
            new_focus.is_some_and(|(_, priority)| priority),
        ) {
            // This menu has priority, and `new_focus` references a non-priority menu.
            new_focus = Some((entity, true));
        } else if new_focus.is_none() && !has_current_menu && !menu.is_locked {
            // This menu is not prioritized, but is the only one we spawned this frame, so set focus to it.
            new_focus = Some((entity, false));
        }
    }

    // set focus to this menu if there is no current menu
    if let Some((target, _)) = new_focus {
        // nav_state.menu = Some(target);
        nav_request_writer.write(NavRequest::SetFocus(target));
    }
}

/// System that initializes new `Pressable` entities by adding their relationship to their root `Focusable` entity.
fn setup_new_focusables(
    mut commands: Commands,
    mut query: Query<(Entity, &Focusable), (Added<Focusable>, Without<FocusableOf>)>,
    child_of_query: Query<&ChildOf>,
    menu_query: Query<(), With<NavMenu>>,
    nav_state: Res<UiNavState>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    if query.is_empty() {
        return;
    }

    // define the first focusable that needs to be given focus
    let mut new_focus = None;

    for (entity, focusable) in query.iter_mut() {
        // insert the relationship if we found the root `NavMenu`, otherwise log a warning
        if let Some(parent) = find_parent(entity, &child_of_query, &menu_query) {
            commands.entity(entity).insert(FocusableOf(parent));

            // set initial focus on this entity if it is prioritized, and in the current menu
            if focusable.is_priority
                && !focusable.is_disabled
                && Some(parent) == nav_state.menu
                && new_focus.is_none()
            {
                new_focus = Some(entity);
            }
        } else {
            warn!("A `Focusable` was added outside of a root `NavMenu`");
        }
    }

    // focus on the new focusable
    if let Some(new_focus) = new_focus {
        nav_request_writer.write(NavRequest::SetFocus(new_focus));
    }
}

/// System that clears the current menu when it is removed.
fn handle_menu_removed(
    mut removed: RemovedComponents<NavMenu>,
    mut nav_state: ResMut<UiNavState>,
    mut nav_request_writer: EventWriter<NavRequest>,
    menu_query: Query<(Entity, &NavMenu), With<NavMenu>>,
) {
    for entity in removed.read() {
        if Some(entity) == nav_state.menu {
            nav_state.menu = None;

            // focus on new menu
            if let Some(new_menu) = menu_query
                .iter()
                .filter(|(e, ..)| *e != entity)
                .reduce(|acc, e| {
                    if e.1.is_priority && !acc.1.is_priority {
                        e
                    } else {
                        acc
                    }
                })
                .map(|(e, _)| e)
            {
                nav_request_writer.write(NavRequest::SetFocus(new_menu));
            }
        }
    }

    nav_request_writer.write(NavRequest::Refresh);
}

/// System that handles internal `NavRequest` events.
#[allow(clippy::too_many_arguments)]
fn handle_nav_requests(
    mut events: EventReader<NavRequest>,
    mut queries: Queries,
    mut press_writer: EventWriter<FocusablePressed>,
    mut focus_writer: EventWriter<UiNavFocusChangedEvent>,
    mut cancel_writer: EventWriter<UiNavCancelEvent>,
) {
    // Find the current focused entity
    let mut focused = queries.get_current_focusable();

    for event in events.read() {
        match event {
            NavRequest::SetFocus(entity) => {
                if queries.nav_state.locked {
                    continue;
                }
                if let Some(new_focused) = queries.set_focus(focused, *entity) {
                    focused = Some(new_focused);
                    focus_writer.write(UiNavFocusChangedEvent {
                        entity: new_focused,
                        interaction_type: UiNavInteractionType::Auto,
                    });
                }
            }
            NavRequest::Movement(direction) => {
                if queries.nav_state.locked {
                    continue;
                }
                // try capture movement against the current focusable
                let action = match direction {
                    UiNavDirection::Left => Some(PressableAction::Left),
                    UiNavDirection::Right => Some(PressableAction::Right),
                    _ => None,
                };
                if let (Some(focused), Some(action)) = (focused, action) {
                    if queries.get_focusable_captures_movement(focused, *direction) {
                        press_writer.write(FocusablePressed {
                            entity: focused,
                            action,
                        });
                        continue;
                    }
                }

                // Move focus to a new entity
                if let Some(menu_entity) = queries.nav_state.menu {
                    if let Ok((_, menu, focusables, _)) = queries.menus.get(menu_entity) {
                        let siblings: Vec<Entity> = focusables.iter().collect();
                        let result = resolve_2d(
                            focused,
                            *direction,
                            menu.is_wrap,
                            &siblings,
                            &queries.focusables.as_readonly(),
                        );
                        if let Some(entity) = result {
                            let new_focused = queries.set_focus(focused, entity);
                            if let Some(new_focused) = new_focused {
                                focused = Some(new_focused);
                                focus_writer.write(UiNavFocusChangedEvent {
                                    entity: new_focused,
                                    interaction_type: UiNavInteractionType::Button,
                                });
                            }
                        }
                    } else {
                        warn!("Current menu not found");
                    }
                } else {
                    warn!("No current menu");
                }
            }
            NavRequest::ActionPress => {
                if queries.nav_state.locked {
                    continue;
                }
                if let Some(focused) = focused {
                    press_writer.write(FocusablePressed {
                        entity: focused,
                        action: PressableAction::Press,
                    });
                }
            }
            NavRequest::Cancel => {
                if queries.nav_state.locked {
                    continue;
                }
                if let Some(menu) = queries.nav_state.menu {
                    cancel_writer.write(UiNavCancelEvent(menu));
                } else {
                    warn!("NavRequest::Cancel received but no current menu");
                }
            }
            NavRequest::Lock => {
                queries.nav_state.locked = true;
            }
            NavRequest::Unlock => {
                queries.nav_state.locked = false;
            }
            NavRequest::Refresh => (),
        }
    }
}

/// System that refreshes the UI navigation state whenever a focusable changes.
fn handle_focusable_changed(
    query: Query<(), Changed<Focusable>>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    if !query.is_empty() {
        nav_request_writer.write(NavRequest::Refresh);
    }
}

fn update_input(
    time: Res<Time<Virtual>>,
    mut input: ResMut<InputManager>,
    mut reader: InputReader,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    // Set the gamepad
    reader.set_gamepad(input.gamepad);

    // update the input manager
    input.update(&time, &reader);

    for (action_type, action) in input.actions.iter() {
        for event in action.events().iter() {
            match event {
                ActionEvents::FIRED => {
                    if let Some(direction) = action_type.to_direction() {
                        nav_request_writer.write(NavRequest::Movement(direction));
                    } else if matches!(action_type, ActionType::Action) {
                        nav_request_writer.write(NavRequest::ActionPress);
                    } else if matches!(action_type, ActionType::Cancel) {
                        nav_request_writer.write(NavRequest::Cancel);
                    }
                }
                ActionEvents::ONGOING => {
                    // println!("ONGOING");
                }
                ActionEvents::STARTED => {
                    // println!("STARTED");
                }
                // ActionEvents::CANCELED | ActionEvents::COMPLETED => {
                //     if let Some(direction) = action_type.to_direction() {
                //         nav_request_writer.write(NavRequest::MovementReleased(direction));
                //     } else if matches!(action_type, ActionType::Action) {
                //         nav_request_writer.write(NavRequest::ActionRelease);
                //     }
                //     // println!("CANCELED");
                // }
                _ => (),
            }
        }
    }
}
