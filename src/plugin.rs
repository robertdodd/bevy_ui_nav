use bevy::{
    ecs::query::{QueryData, QueryFilter},
    input::mouse::MouseButtonInput,
    prelude::*,
};

use crate::{components::*, events::*, input::*, resolve::resolve_2d, resources::*, types::*};

pub struct BevyUiNavPlugin;

impl Plugin for BevyUiNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiNavClickEvent>()
            .add_event::<UiNavCancelEvent>()
            .add_event::<NavRequest>()
            .add_event::<UiNavFocusChangedEvent>()
            .add_event::<PressableClick>()
            .init_resource::<UiNavState>()
            .init_resource::<UiNavSettings>()
            .init_resource::<UiNavInputManager>()
            .add_systems(
                PreUpdate,
                (setup_new_menus, setup_new_focusables, setup_new_pressables).chain(),
            )
            .add_systems(
                Update,
                (
                    (
                        handle_current_menu_removed.run_if(any_component_removed::<NavMenu>),
                        (
                            handle_interactions
                                .run_if(on_event::<CursorMoved>.or(on_event::<MouseButtonInput>)),
                            update_input,
                            // handle_gamepad_input,
                            // handle_keyboard_input_events.run_if(on_event::<KeyboardInput>),
                            // handle_keyboard_input_presses,
                        )
                            .chain(),
                        tick_pressed_timer,
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
fn tick_pressed_timer(time: Res<Time>, mut nav_state: ResMut<UiNavState>) {
    if nav_state.direction.is_some() {
        nav_state.nav_timer.tick(time.delta());
        nav_state.hold_timer.tick(time.delta());
    }
}

/// System that initializes newly added menus
fn setup_new_menus(
    query: Query<(Entity, &NavMenu), Added<NavMenu>>,
    menu_query: Query<(), With<NavMenu>>,
    mut nav_state: ResMut<UiNavState>,
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
        nav_state.menu = Some(target);
    }

    nav_request_writer.write(NavRequest::Refresh);
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
        nav_request_writer.write(NavRequest::SetFocus {
            entity: new_focus,
            interaction_type: UiNavInteractionType::Auto,
        });
    }
}

/// System that handles bevy `Interaction` changes.
///
/// When an interaction changes the navigation state of `Focusables` is removed, but the current focusable is
/// remembered in case the user switches back to ui navigatin mode.
fn handle_interactions(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut query: Query<
        (&mut Pressable, &PressableOf, &Interaction),
        (Changed<Interaction>, With<Pressable>),
    >,
    focusable_query: Query<(&Focusable, &FocusableOf)>,
    menu_query: Query<&NavMenu>,
    mut nav_request_writer: EventWriter<NavRequest>,
    mut click_writer: EventWriter<UiNavClickEvent>,
    nav_state: Res<UiNavState>,
) {
    // only handle interaction changes when the mouse is moved
    if cursor_moved_events.read().count() == 0 && mouse_button_input_events.read().count() == 0 {
        return;
    }

    // Exit if focus if locked
    // IMPORTANT: Do this AFTER we consume the event readers
    if nav_state.locked {
        return;
    }

    let is_current_menu_locked = nav_state
        .menu
        .and_then(|e| menu_query.get(e).ok())
        .is_some_and(|nav_menu| nav_menu.is_locked);

    for (mut pressable, pressable_of, interaction) in query.iter_mut() {
        // update the pressable state
        match *interaction {
            Interaction::Pressed => {
                pressable._is_pressed_interaction = true;
                pressable._is_hover_interaction = true
            }
            Interaction::Hovered => {
                pressable._is_hover_interaction = true;
                pressable._is_pressed_interaction = false;
            }
            Interaction::None => {
                pressable._is_pressed_interaction = false;
                pressable._is_hover_interaction = false;
            }
        }

        // ignore all except receiving focus
        if *interaction == Interaction::None {
            continue;
        }

        if let Ok((focusable, focusable_of)) = focusable_query.get(pressable_of.0) {
            // ignore if the focusable is disabled or already active
            if focusable.is_disabled || focusable.active() {
                continue;
            }

            // ignore if the focusable is outside the current menu and either:
            // - in a menu that is locked
            // - the current menu is locked
            let is_menu_locked = menu_query
                .get(focusable_of.0)
                .map_or(true, |menu| menu.is_locked);
            let is_in_current_menu = Some(focusable_of.0) == nav_state.menu;
            if !is_in_current_menu && (is_current_menu_locked || is_menu_locked) {
                continue;
            }

            // Set focus on the focusable
            nav_request_writer.write(NavRequest::SetFocus {
                entity: pressable_of.0,
                interaction_type: UiNavInteractionType::Mouse,
            });

            // TODO: do this when handling the NavRequest::SetFocus for a mouse interaction
            // // clear focus from nav state
            // nav_state.focusable = None;
            // nav_state.last_focusable = Some(entity);

            if *interaction == Interaction::Pressed {
                click_writer.write(UiNavClickEvent(pressable_of.0));
            }
        }
    }
}

/// System that clears the current menu when it is removed.
fn handle_current_menu_removed(
    mut removed: RemovedComponents<NavMenu>,
    mut nav_state: ResMut<UiNavState>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for entity in removed.read() {
        if Some(entity) == nav_state.menu {
            nav_state.menu = None;
            nav_state.clear_direction();
        }
    }

    nav_request_writer.write(NavRequest::Refresh);
}

// /// System that listens for keyboard or gamepad input and emits the appropriate navigation events.
// #[allow(clippy::too_many_arguments)]
// fn handle_gamepad_input(
//     gamepads: Query<(Entity, &Gamepad)>,
//     mut nav_request_writer: EventWriter<NavRequest>,
//     mut nav_state: ResMut<UiNavState>,
//     settings: Res<UiNavSettings>,
//     mut input_manager: ResMut<UiNavInputManager>,
// ) {
//     update_input_manager(&mut input_manager, &gamepads);

//     if nav_state.menu.is_some() && !nav_state.locked {
//         // send movement event
//         if let Some(direction) = input_manager.direction() {
//             if nav_state.direction.is_none() {
//                 // send movement key on first pressed
//                 nav_request_writer.write(NavRequest::Movement(direction));
//             } else {
//                 // send movement key on timer tick while held
//                 let movement_speed = f32_lerp(
//                     settings.movement_speed_slow,
//                     settings.movement_speed_fast,
//                     (nav_state.hold_timer.elapsed_secs() / settings.movement_acceleration_time)
//                         .min(1.),
//                 );
//                 if nav_state.nav_timer.elapsed_secs() > movement_speed {
//                     nav_request_writer.write(NavRequest::Movement(direction));
//                     nav_state.nav_timer.reset();
//                 }
//             }
//             nav_state.direction = Some(direction);
//         } else if nav_state.direction.is_some() {
//             nav_state.clear_direction();
//         }

//         // send action press event
//         if input_manager.just_pressed(ActionType::Action) {
//             nav_request_writer.write(NavRequest::ActionPress);
//         } else if input_manager.just_released(ActionType::Action) {
//             nav_request_writer.write(NavRequest::ActionRelease);
//         }
//     } else if nav_state.direction.is_some() {
//         // clear direction keys when the menu is locked, or we don't have a current menu
//         nav_state.clear_direction();
//     }

//     if input_manager.is_direction_released() {
//         nav_request_writer.write(NavRequest::MovementReleased);
//     }

//     // send cancel event
//     // NOTE: This runs even when locked, in case the user wishes to lsiten for cancel events in order to unlock
//     // navigation.
//     if input_manager.just_pressed(ActionType::Cancel) {
//         nav_request_writer.write(NavRequest::Cancel);
//     }
// }

/// Util that returns the focusable and menu `Entity` for the `entity` of a `NavRequest::SetFocus` event, and checks
/// that focus can be set to the target entity.
/// `entity` can be a `Focusable` or `NavMenu`.
#[allow(clippy::too_many_arguments)]
fn try_set_focus(
    entity: Entity,
    query: &Query<(
        &Focusable,
        &FocusableOf,
        &ComputedNode,
        &GlobalTransform,
        &InheritedVisibility,
        &Pressables,
    )>,
    menu_query: &Query<(Entity, &NavMenu, &Focusables)>,
) -> Option<Entity> {
    if let Ok((focusable, ..)) = query.get(entity) {
        // if the target entity is a focusable, check that it is not disabled.
        if focusable.is_disabled {
            None
        } else {
            Some(entity)
        }
    } else if let Ok((_, _, focusables)) = menu_query.get(entity) {
        // find the first non-disabled focusable in the nav menu to focus on.
        // try find a prioritized focusable first, otherwise use the first one.
        let first_focusable = focusables
            .iter()
            .filter_map(|e| {
                query
                    .get(e)
                    .map(|(focusable, ..)| (e, focusable.is_priority, focusable.is_disabled))
                    .ok()
            })
            .filter(|(_, _, is_disabled)| !is_disabled)
            .reduce(|acc, e| if e.1 { e } else { acc })
            .map(|(e, ..)| e);

        // if we found a focusable in the menu, focus on it
        if let Some(focusable_entity) = first_focusable {
            Some(focusable_entity)
        } else {
            warn!("Tried to set focus to nav menu, but it contained no active focusables.");
            None
        }
    } else {
        warn!("Tried to set focus to invalid entity.");
        None
    }
}

/// Set focus on an entity and remove focus from the old one.
fn set_focus_to(
    from: Option<Entity>,
    to: Entity,
    query: &mut Query<(
        &mut Focusable,
        &FocusableOf,
        &ComputedNode,
        &GlobalTransform,
        &InheritedVisibility,
        &Pressables,
    )>,
    pressable_query: &mut Query<&mut Pressable>,
    nav_state: &mut UiNavState,
    clear_focus: bool,
) {
    // remove focus from the old focusable
    if let Some(from) = from {
        if let Ok((mut focusable, _, _, _, _, pressables)) = query.get_mut(from) {
            focusable.is_focused = false;
            for pressable in pressables {
                if let Ok(mut pressable) = pressable_query.get_mut(*pressable) {
                    pressable._is_hover_focusable = false;
                    pressable._is_pressed_focusable = false;
                }
            }
        }
    }
    nav_state.menu = None;

    // set focus to the new focusable
    if let Ok((mut focusable, focusable_of, ..)) = query.get_mut(to) {
        if !clear_focus {
            focusable.is_focused = true;
        }

        // TODO: don't set focus if focus was given using the mouse
        nav_state.menu = Some(focusable_of.0);
        nav_state.last_focusable = Some(to);
    }
}

/// Tries to pass movement to the `Pressable` children of a `Focusable`. Returns `true` if the direction request was
/// consumed by a pressable.
fn try_consume_nav_request(
    entity: Entity,
    action: PressableAction,
    query: &mut Query<(
        &mut Focusable,
        &FocusableOf,
        &ComputedNode,
        &GlobalTransform,
        &InheritedVisibility,
        &Pressables,
    )>,
    pressable_query: &mut Query<&mut Pressable>,
    pressable_click_writer: &mut EventWriter<PressableClick>,
) -> bool {
    if let Ok((mut focusable, _, _, _, _, pressables)) = query.get_mut(entity) {
        // exit early if the focusable does not consume the direction
        if !focusable.action.matches_action(action) {
            return false;
        }

        // mark the focusable as pressed
        focusable.is_pressed_key = true;

        // iterate over all pressables
        for child in pressables {
            if let Ok(mut pressable) = pressable_query.get_mut(*child) {
                if pressable.action == action {
                    pressable._is_pressed_focusable = true;
                    pressable_click_writer.write(PressableClick(*child));
                } else if pressable._is_pressed_focusable {
                    pressable._is_pressed_focusable = false;
                }
            }
        }

        return true;
    }
    false
}

/// System that handles internal `NavRequest` events.
#[allow(clippy::too_many_arguments)]
fn handle_nav_requests(
    mut events: EventReader<NavRequest>,
    mut query: Query<(
        &mut Focusable,
        &FocusableOf,
        &ComputedNode,
        &GlobalTransform,
        &InheritedVisibility,
        &Pressables,
    )>,
    mut pressable_query: Query<&mut Pressable>,
    menu_query: Query<(Entity, &NavMenu, &Focusables)>,
    mut nav_state: ResMut<UiNavState>,
    mut pressable_click_writer: EventWriter<PressableClick>,
) {
    // Find the current focused entity
    let focused = nav_state.menu.and_then(|e| {
        if let Ok((_, _, focusables)) = menu_query.get(e) {
            focusables
                .iter()
                .find(|e| query.get(*e).is_ok_and(|(f, ..)| f.is_focused))
        } else {
            None
        }
    });

    for event in events.read() {
        match event {
            NavRequest::SetFocus {
                entity,
                interaction_type,
            } => {
                if let Some(focusable_entity) =
                    try_set_focus(*entity, &query.as_readonly(), &menu_query)
                {
                    set_focus_to(
                        focused,
                        focusable_entity,
                        &mut query,
                        &mut pressable_query,
                        &mut nav_state,
                        *interaction_type == UiNavInteractionType::Mouse,
                    );
                }
            }
            NavRequest::Movement(direction) => {
                // Check that the current focusable does not consume these events
                let action = match direction {
                    UiNavDirection::Left => Some(PressableAction::Left),
                    UiNavDirection::Right => Some(PressableAction::Right),
                    _ => None,
                };
                let is_consumed = if let (Some(focused), Some(action)) = (focused, action) {
                    try_consume_nav_request(
                        focused,
                        action,
                        &mut query,
                        &mut pressable_query,
                        &mut pressable_click_writer,
                    )
                } else {
                    false
                };

                if is_consumed {
                    continue;
                }

                if let Some(menu_entity) = nav_state.menu {
                    if let Ok((_, menu, focusables)) = menu_query.get(menu_entity) {
                        let siblings: Vec<Entity> = focusables.iter().collect();
                        let result = resolve_2d(
                            focused,
                            *direction,
                            menu.is_wrap,
                            &siblings,
                            &query.as_readonly(),
                        );
                        if let Some(entity) = result {
                            set_focus_to(
                                focused,
                                entity,
                                &mut query,
                                &mut pressable_query,
                                &mut nav_state,
                                false,
                            );
                        }
                    } else {
                        warn!("Current menu not found");
                    }
                } else {
                    warn!("No current menu");
                }
            }
            NavRequest::MovementReleased => {
                if let Some(focused) = focused {
                    if let Ok((mut focusable, _, _, _, _, pressables)) = query.get_mut(focused) {
                        if focusable.action == FocusableAction::PressXY && focusable.is_pressed_key
                        {
                            focusable.is_pressed_key = false;
                            for pressable in pressables.iter() {
                                if let Ok(mut pressable) = pressable_query.get_mut(pressable) {
                                    pressable._is_pressed_focusable = false;
                                }
                            }
                        }
                    }
                }
            }
            NavRequest::ActionPress => {
                if let Some(focused) = focused {
                    try_consume_nav_request(
                        focused,
                        PressableAction::Press,
                        &mut query,
                        &mut pressable_query,
                        &mut pressable_click_writer,
                    );
                }
            }
            NavRequest::ActionRelease => {
                if let Some(focused) = focused {
                    if let Ok((mut focusable, _, _, _, _, pressables)) = query.get_mut(focused) {
                        if focusable.action == FocusableAction::Press && focusable.is_pressed_key {
                            focusable.is_pressed_key = false;
                            for pressable in pressables.iter() {
                                if let Ok(mut pressable) = pressable_query.get_mut(pressable) {
                                    pressable._is_pressed_focusable = false;
                                }
                            }
                        }
                    }
                }
            }
            NavRequest::Cancel => {
                // is_cancel = true;
            }
            NavRequest::Lock => {
                // spatial_map.lock();
            }
            NavRequest::Unlock => {
                // spatial_map.unlock();
            }
            NavRequest::Refresh => (),
        }
    }

    // for event in spatial_map.events() {
    //     match event {
    //         UiSpatialMapEvent::Press(entity) => {
    //             if let Ok((_, mut focusable, _, _, _)) = query.get_mut(*entity) {
    //                 focusable.is_pressed_key = true;
    //             }
    //         }
    //         UiSpatialMapEvent::Click(entity) => {
    //             click_writer.write(UiNavClickEvent(*entity));
    //         }
    //         UiSpatialMapEvent::Release(entity) => {
    //             if let Ok((_, mut focusable, _, _, _)) = query.get_mut(*entity) {
    //                 focusable.is_pressed_key = false;
    //             }
    //         }
    //     }
    // }

    // // Focus on new menu
    // if let Some(new_menu) = spatial_map.get_new_menu() {
    //     nav_state.menu = new_menu;
    // }

    // // Focus on new focusable
    // if let Some((new_focusable, interaction_type)) = spatial_map.get_new_focusable() {
    //     for (entity, mut focusable, _, _, _, _) in query.iter_mut() {
    //         focusable.is_focused = Some(entity) == new_focusable;
    //         if focusable.is_focused {
    //             focus_change_writer.write(UiNavFocusChangedEvent {
    //                 entity,
    //                 interaction_type,
    //             });
    //         }
    //     }
    // }

    // // Handle new locked state
    // if let Some(locked) = spatial_map.get_new_locked() {
    //     nav_state.locked = locked
    // }

    // // Handle cancel events
    // if let (false, true, Some(menu)) = (nav_state.locked, is_cancel, spatial_map.menu()) {
    //     cancel_writer.write(UiNavCancelEvent(menu));
    // }
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

// /// This system prints out all keyboard events as they come in
// fn handle_keyboard_input_events(
//     mut events: EventReader<KeyboardInput>,
//     mut nav_request_writer: EventWriter<NavRequest>,
//     input_manager: Res<UiNavInputManager>,
// ) {
//     if events.is_empty() {
//         return;
//     }
//     for event in events.read() {
//         for action in input_manager.input_map.iter() {
//             if let InputMapping::Key { keycode, action } = action {
//                 if *keycode == event.key_code {
//                     let nav_request = match (action, event.state) {
//                         (ActionType::Up, ButtonState::Pressed) => {
//                             Some(NavRequest::Movement(UiNavDirection::Up))
//                         }
//                         (ActionType::Down, ButtonState::Pressed) => {
//                             Some(NavRequest::Movement(UiNavDirection::Down))
//                         }
//                         (ActionType::Left, ButtonState::Pressed) => {
//                             Some(NavRequest::Movement(UiNavDirection::Left))
//                         }
//                         (ActionType::Right, ButtonState::Pressed) => {
//                             Some(NavRequest::Movement(UiNavDirection::Right))
//                         }
//                         // NOTE: we do not handle `ActionType::Action` or `ActionType::Cancel` presses here, as we
//                         //  only want to handle those when just pressed or just released. They are handled in
//                         //  `handle_keyboard_input_presses`.
//                         _ => None,
//                     };
//                     if let Some(nav_request) = nav_request {
//                         nav_request_writer.write(nav_request);
//                     }
//                 }
//             }
//         }
//     }
// }

// /// This system prints out all keyboard events as they come in
// fn handle_keyboard_input_presses(
//     keys: Res<ButtonInput<KeyCode>>,
//     mut nav_request_writer: EventWriter<NavRequest>,
//     input_manager: Res<UiNavInputManager>,
// ) {
//     for action in input_manager.input_map.iter() {
//         if let InputMapping::Key { keycode, action } = action {
//             let nav_request = match action {
//                 ActionType::Action => {
//                     if keys.just_pressed(*keycode) {
//                         Some(NavRequest::ActionPress)
//                     } else if keys.just_released(*keycode) {
//                         Some(NavRequest::ActionRelease)
//                     } else {
//                         None
//                     }
//                 }
//                 ActionType::Cancel => {
//                     if keys.just_pressed(*keycode) {
//                         Some(NavRequest::Cancel)
//                     } else {
//                         None
//                     }
//                 }
//                 _ => None,
//             };
//             if let Some(nav_request) = nav_request {
//                 nav_request_writer.write(nav_request);
//             }
//         }
//     }
// }

/// System that initializes new `Pressable` entities by adding their relationship to their root `Focusable` entity.
fn setup_new_pressables(
    mut commands: Commands,
    mut query: Query<Entity, (Added<Pressable>, Without<PressableOf>)>,
    child_of_query: Query<&ChildOf>,
    focusable_query: Query<(), With<Focusable>>,
) {
    for entity in query.iter_mut() {
        // insert the relationship if we found the root `Focusable`, otherwise log a warning
        if let Some(parent) = find_parent(entity, &child_of_query, &focusable_query) {
            commands.entity(entity).insert(PressableOf(parent));
        } else {
            warn!("A `Pressable` was added without a root `Focusable` entity.");
        }
    }
}

/// Utility that traverses the entity heirarchy until it finds a parent entity matching a query.
fn find_parent<D, F>(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    parent_query: &Query<D, F>,
) -> Option<Entity>
where
    D: QueryData,
    F: QueryFilter,
{
    // traverse the `ChildOf` heirarchy until we find a parent `Focusable` entity
    let mut current = entity;
    let mut parent = None;
    while let Ok(child_of) = child_of_query.get(current) {
        if parent_query.contains(child_of.0) {
            parent = Some(child_of.0);
            break;
        } else {
            current = child_of.0;
        }
    }
    parent
}

fn update_input(
    mut input: ResMut<UiNavInputManager>,
    mut input_reader: InputReader,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    // Set the gamepad
    input_reader.set_gamepad(input.gamepad);

    // update the input manager
    input.update(&input_reader);

    // TODO: Send events
    if input.just_pressed(ActionType::Action) {
        nav_request_writer.write(NavRequest::ActionPress);
    } else if input.just_released(ActionType::Action) {
        nav_request_writer.write(NavRequest::ActionRelease);
    }

    if input.just_pressed(ActionType::Up) {
        nav_request_writer.write(NavRequest::Movement(UiNavDirection::Up));
    }
    if input.just_pressed(ActionType::Down) {
        nav_request_writer.write(NavRequest::Movement(UiNavDirection::Down));
    }
    if input.just_pressed(ActionType::Left) {
        nav_request_writer.write(NavRequest::Movement(UiNavDirection::Left));
    }
    if input.just_pressed(ActionType::Right) {
        nav_request_writer.write(NavRequest::Movement(UiNavDirection::Right));
    }

    if input.is_direction_released() {
        nav_request_writer.write(NavRequest::MovementReleased);
    }
}
