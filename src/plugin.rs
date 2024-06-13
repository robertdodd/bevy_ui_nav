use bevy::{input::mouse::MouseButtonInput, prelude::*, ui::RelativeCursorPosition};

use crate::{
    components::*, default_input_map::DEFAULT_INPUT_MAP, events::*, focus_node::*, input::*,
    resources::*, types::*, utils::*,
};

pub struct BevyUiNavPlugin;

impl Plugin for BevyUiNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InternalSetFocusEvent>()
            .add_event::<InternalFocusMoveEvent>()
            .add_event::<InternalActionButtonEvent>()
            .add_event::<UiNavClickEvent>()
            .add_event::<UiNavLockEvent>()
            .add_event::<UiNavCancelEvent>()
            .add_event::<NavRequest>()
            .add_event::<InternalRefreshEvent>()
            .add_event::<UiNavFocusChangedEvent>()
            .init_resource::<UiNavState>()
            .init_resource::<UiNavSettings>()
            .insert_resource(UiNavInputManager::from_input_map(
                DEFAULT_INPUT_MAP,
                0.1,
                0.9,
            ))
            .add_systems(
                Update,
                (
                    (
                        (
                            handle_current_menu_removed.run_if(any_component_removed::<NavMenu>()),
                            setup_new_menus,
                            setup_new_focusables,
                        )
                            .chain(),
                        handle_input,
                        tick_pressed_timer,
                        (
                            handle_focusable_changed,
                            handle_internal_refresh_events
                                .run_if(on_event::<InternalRefreshEvent>()),
                        )
                            .chain(),
                    )
                        .before(UiNavSet),
                    (
                        handle_input,
                        handle_interactions.run_if(
                            on_event::<CursorMoved>().or_else(on_event::<MouseButtonInput>()),
                        ),
                        handle_nav_requests,
                        handle_lock_events.run_if(on_event::<UiNavLockEvent>()),
                        handle_internal_focus_move_events
                            .run_if(on_event::<InternalFocusMoveEvent>()),
                        handle_internal_set_focus_events
                            .run_if(on_event::<InternalSetFocusEvent>()),
                        handle_internal_action_button_events
                            .run_if(on_event::<InternalActionButtonEvent>()),
                    )
                        .chain()
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
    mut refresh_writer: EventWriter<InternalRefreshEvent>,
) {
    if query.is_empty() {
        return;
    }

    // The new target menu and whether it is prioritized. We need to track and handle this outside the iterator
    // because multiple menus can be spawned simultaneously.
    let mut new_focus: Option<(Entity, bool)> = None;
    for (entity, menu) in query.iter() {
        if let (true, false) = (
            menu.is_priority,
            new_focus.map_or(false, |(_, priority)| priority),
        ) {
            // This menu has priority, and `new_focus` references a non-priority menu.
            new_focus = Some((entity, true));
        } else if new_focus.is_none() && !menu.is_locked {
            // This menu is not prioritized, but is the only one we spawned this frame, so set focus to it.
            new_focus = Some((entity, false));
        }
    }

    // set focus to this menu if there is no current menu
    let has_current_menu = nav_state.menu.map_or(false, |e| menu_query.contains(e));
    if let (Some((target, _)), false) = (new_focus, has_current_menu) {
        if !has_current_menu {
            nav_state.menu = Some(target);
        }
    }

    refresh_writer.send(InternalRefreshEvent);
}

/// System that initializes newly added focusables.
#[allow(clippy::type_complexity)]
fn setup_new_focusables(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Focusable,
            Has<Interaction>,
            Has<RelativeCursorPosition>,
        ),
        Added<Focusable>,
    >,
    parent_query: Query<&Parent>,
    menu_query: Query<(), With<NavMenu>>,
    nav_state: Res<UiNavState>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    if query.is_empty() {
        return;
    }

    // define the first focusable that needs to be given focus
    let mut new_focus = None;

    for (entity, mut focusable, has_interaction, has_relative_cursor_position) in query.iter_mut() {
        // ensure the entity has a `RelativeCursorPosition` component
        let mut cmds = commands.entity(entity);
        if !has_interaction {
            cmds.insert(Interaction::default());
        }
        if !has_relative_cursor_position {
            cmds.insert(RelativeCursorPosition::default());
        }

        if focusable.menu.is_none() {
            let mut current = entity;
            let mut menu_parent = None;

            while let Ok(parent) = parent_query.get(current) {
                if menu_query.contains(parent.get()) {
                    menu_parent = Some(parent.get());
                    break;
                } else {
                    current = parent.get();
                }
            }

            if let Some(menu_parent) = menu_parent {
                focusable.menu = Some(menu_parent);

                // set initial focus on this entity if it is prioritized, and in the current menu
                if focusable.is_priority
                    && !focusable.is_disabled
                    && Some(menu_parent) == nav_state.menu
                    && new_focus.is_none()
                    && !focusable.is_mouse_only
                {
                    new_focus = Some(entity);
                }
            } else {
                warn!("A `Focusable` was added without a root `Menu` entity in it's heirarchy. This `Focusable` will not function.");
            }
        }
    }

    if let Some(new_focus) = new_focus {
        nav_request_writer.send(NavRequest::SetFocus {
            entity: new_focus,
            interaction_type: UiNavInteractionType::Auto,
        });
    }
}

/// System that handles bevy `Interaction` changes.
///
/// Interaction changes are only respected if the mouse was moved AND the new interaction state is
/// `Interaction::Hovered`.
fn handle_interactions(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut query: Query<(
        Entity,
        &Interaction,
        &mut Focusable,
        &RelativeCursorPosition,
    )>,
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

    // check if blocked by a press on another entity
    let is_blocked = query
        .iter()
        .any(|(__, _, focusable, _)| focusable.is_pressed());

    let is_current_menu_locked = nav_state
        .menu
        .and_then(|e| menu_query.get(e).ok())
        .map_or(false, |nav_menu| nav_menu.is_locked);

    for (entity, interaction, mut focusable, relative_cursor_position) in query.iter_mut() {
        if focusable.is_disabled {
            continue;
        }

        let is_menu_locked = focusable
            .menu
            .and_then(|menu_entity| menu_query.get(menu_entity).ok())
            .map_or(false, |nav_menu| nav_menu.is_locked);

        let is_in_current_menu = focusable.menu == nav_state.menu;
        if !(is_in_current_menu || !(is_current_menu_locked && is_menu_locked)) {
            continue;
        }

        let is_mouse_over = relative_cursor_position.mouse_over();
        // focus on this entity
        let (is_pressed, is_hovered) = match *interaction {
            Interaction::Pressed => (true, is_mouse_over),
            Interaction::Hovered => (false, true),
            Interaction::None => (false, false),
        };
        if focusable.is_mouse_only && !is_hovered && !is_pressed {
            focusable.is_focused = false;
            focusable.is_pressed_interaction = false;
            focusable.is_pressed_interaction_from_active = false;
            focusable.is_hovered_interaction = false;
        } else if is_pressed != focusable.is_pressed_interaction {
            // send click events
            if focusable.is_pressed_interaction
                && focusable.is_pressed_interaction_from_active
                && !is_pressed
                && is_mouse_over
            {
                click_writer.send(UiNavClickEvent(entity));
            }
            // udpate focusable
            focusable.is_pressed_interaction = is_pressed;
            focusable.is_pressed_interaction_from_active = !is_blocked;
        }
        if is_hovered != focusable.is_hovered_interaction {
            focusable.is_hovered_interaction = is_hovered;
        }

        // Set focus on the entity
        if (*interaction == Interaction::Hovered
            || (*interaction == Interaction::Pressed && is_mouse_over))
            && !focusable.active()
            && !is_blocked
        {
            nav_request_writer.send(NavRequest::SetFocus {
                entity,
                interaction_type: UiNavInteractionType::Mouse,
            });
        }
    }
}

/// System that handles `FocusEvent` events.
///
/// `FocusEvent` events are used to set focus to a specific element.
///
/// If multiple events are received, only the first one is performed, and the others are ignored.
fn handle_internal_set_focus_events(
    mut events: EventReader<InternalSetFocusEvent>,
    mut focusable_query: Query<(Entity, &mut Focusable)>,
    menu_query: Query<(), With<NavMenu>>,
    mut nav_state: ResMut<UiNavState>,
    mut nav_event_writer: EventWriter<UiNavFocusChangedEvent>,
) {
    // Exit if focus if locked
    // IMPORTANT: We must consume the event reader
    if nav_state.locked {
        events.clear();
        return;
    }

    let mut new_focused = None;
    for event in events.read() {
        if menu_query.contains(event.entity) {
            // The focus event was for a menu, so find the first focusable in that menu, preferring prioritized ones
            new_focused = focusable_query
                .iter()
                .filter(|(_, focusable)| focusable.menu == Some(event.entity))
                .reduce(|acc, e| {
                    if !acc.1.is_priority && e.1.is_priority {
                        e
                    } else {
                        acc
                    }
                })
                .map(|(entity, _)| (entity, event.interaction_type, false));
        } else if let Ok((_, focusable)) = focusable_query.get(event.entity) {
            if !focusable.is_disabled {
                new_focused = Some((
                    event.entity,
                    event.interaction_type,
                    focusable.is_mouse_only,
                ));
            }
        } else {
            error!("FocusEvent query failed");
        }
    }

    // remove focus from other entities
    if let Some((new_focused, interaction_type, is_mouse_only)) = new_focused {
        if is_mouse_only {
            if let Ok((_, mut focusable)) = focusable_query.get_mut(new_focused) {
                focusable.is_focused = true;
            }
        } else {
            for (entity, mut focusable) in focusable_query.iter_mut() {
                // Ignore "mouse_only" focusables
                if focusable.is_mouse_only {
                    continue;
                }

                // define whether this focusable should be focused and update the focus state if it changed
                let new_is_focused = if entity == new_focused && focusable.menu.is_some() {
                    nav_state.menu = focusable.menu;
                    true
                } else {
                    false
                };
                if new_is_focused != focusable.is_focused {
                    focusable.is_focused = new_is_focused;

                    // send an event notifying about this focus change
                    if new_is_focused && !focusable.is_mouse_only {
                        nav_event_writer.send(UiNavFocusChangedEvent {
                            entity,
                            interaction_type,
                        });
                    }
                }
            }
        }
    }
}

/// System that clears the current menu when it is removed.
fn handle_current_menu_removed(
    mut removed: RemovedComponents<NavMenu>,
    mut nav_state: ResMut<UiNavState>,
    mut refresh_writer: EventWriter<InternalRefreshEvent>,
) {
    for entity in removed.read() {
        if Some(entity) == nav_state.menu {
            nav_state.menu = None;
            nav_state.clear_direction();
        }
    }

    refresh_writer.send(InternalRefreshEvent);
}

/// System that handles `FocusMoveEvent` events.
///
/// `FocusMoveEvent` events are used to move focus in a specific direction.
fn handle_internal_focus_move_events(
    mut events: EventReader<InternalFocusMoveEvent>,
    query: Query<(Entity, &Focusable, &Node, &GlobalTransform)>,
    mut set_focus_writer: EventWriter<InternalSetFocusEvent>,
    nav_state: Res<UiNavState>,
    menu_query: Query<&NavMenu>,
) {
    // Exit if focus if locked
    // IMPORTANT: We must consume the event reader
    if nav_state.locked {
        events.clear();
        return;
    }

    // get current menu, exit if there isn't one
    // TODO: Do not exit if no current menu. Loop over all focusables and clear their state if not in active menu.
    let current_menu = nav_state.menu.and_then(|e| menu_query.get(e).ok());
    if current_menu.is_none() {
        return;
    }
    let current_menu_entity = nav_state.menu.unwrap();
    let current_menu = current_menu.unwrap();

    // Find the currently focused element
    let current = query
        .iter()
        .find(|(_, focusable, _, _)| {
            focusable.active()
                && focusable.menu == Some(current_menu_entity)
                && !focusable.is_disabled
                && !focusable.is_mouse_only
        })
        .map(|(_, focusable, node, global_transform)| {
            let focus_node = FocusNode {
                size: node.size(),
                position: global_transform.compute_transform().translation.truncate(),
            };
            let is_blocked = focusable.is_pressed();
            (focus_node, is_blocked)
        });

    if let Some((current, is_blocked)) = current {
        if is_blocked {
            return;
        }
        for event in events.read() {
            let (nearest, furthest) = query
                .iter()
                // skip active
                .filter(|(_, focusable, _, _)| {
                    !focusable.active()
                        && !focusable.is_disabled
                        && !focusable.is_mouse_only
                        && focusable.menu == Some(current_menu_entity)
                })
                // convert to focus node
                .map(|(entity, _, node, global_transform)| {
                    let size = FocusNode {
                        size: node.size(),
                        position: global_transform.compute_transform().translation.truncate(),
                    };
                    let distance = current.distance_to(&size);
                    FocusTarget {
                        entity,
                        is_in_direction: distance.is_in_direction(event.0),
                        is_in_axis: distance.is_along_axis(event.0),
                        // Only prefer movement along direct axes. It doesn't matter when moving diagonally.
                        is_prefer: match event.0 {
                            UiNavDirection::Up | UiNavDirection::Down => distance.is_overlap_y,
                            UiNavDirection::Left | UiNavDirection::Right => distance.is_overlap_x,
                            _ => false,
                        },
                        distance,
                    }
                })
                // Remove any nodes that do not lie along the axis of the movement event. If wrapping is enabled,
                // allow any nodes along the axis. Otherwise, only allow nodes in the direction of the movement event.
                .filter(|focus_node| {
                    if current_menu.is_wrap {
                        focus_node.is_in_axis
                    } else {
                        focus_node.is_in_direction
                    }
                })
                .fold(
                    (None, None),
                    #[allow(clippy::type_complexity)]
                    |(acc_nearest, acc_furthest),
                     e|
                     -> (Option<FocusTarget>, Option<FocusTarget>) {
                        let e_is_in_direction = e.is_in_direction;

                        // Fold the nearest focus node in the direction of the movement event
                        let nearest = if let Some(acc_nearest) = acc_nearest {
                            // Prefer `e` if it lies in the correct direction and is closer than `acc_nearest`
                            if e_is_in_direction
                                && ((acc_nearest.is_prefer == e.is_prefer
                                    && e.distance.total < acc_nearest.distance.total)
                                    || (!acc_nearest.is_prefer && e.is_prefer))
                            {
                                Some(e.clone())
                            } else {
                                Some(acc_nearest)
                            }
                        } else if e_is_in_direction {
                            // set the initial nearest node
                            Some(e.clone())
                        } else {
                            None
                        };

                        // Fold the furthest focus node
                        let furthest = if !current_menu.is_wrap {
                            // skip if wrapping is disabled
                            None
                        } else if let Some(acc_furthest) = acc_furthest {
                            // Prefer `e` if it is further than `acc_furthest` and does not lie in the dirction of the
                            // movement event.
                            if !e_is_in_direction
                                && ((acc_furthest.is_prefer == e.is_prefer
                                    && e.distance.total > acc_furthest.distance.total)
                                    || (!acc_furthest.is_prefer && e.is_prefer))
                            {
                                Some(e.clone())
                            } else {
                                Some(acc_furthest)
                            }
                        } else if !e_is_in_direction {
                            // set the initial furthest node if it does not lie in the direction of the movement event
                            Some(e.clone())
                        } else {
                            None
                        };

                        (nearest, furthest)
                    },
                );

            if let Some(nearest) = nearest {
                set_focus_writer.send(InternalSetFocusEvent::new_button(nearest.entity));
            } else if let (Some(furthest), true) = (furthest, current_menu.is_wrap) {
                // No nearest, wrapping around
                set_focus_writer.send(InternalSetFocusEvent::new_button(furthest.entity));
            }
        }
    } else {
        error!("no current focusable");
    }
}

/// System that handles action button events.
fn handle_internal_action_button_events(
    mut events: EventReader<InternalActionButtonEvent>,
    mut query: Query<(Entity, &mut Focusable)>,
    mut click_writer: EventWriter<UiNavClickEvent>,
    nav_state: Res<UiNavState>,
) {
    // Exit if focus if locked
    // IMPORTANT: We must consume the event reader
    if nav_state.locked {
        events.clear();
        return;
    }

    for event in events.read() {
        for (entity, mut focusable) in query.iter_mut() {
            // ignore non-active focusables
            if !focusable.active() || focusable.is_disabled || focusable.is_mouse_only {
                continue;
            }

            // ignore down events if the focusable is already pressed via interaction
            if event.0 == PressType::Press && focusable.is_pressed() {
                continue;
            }

            // update the focusable and emit a click event on button up
            let new_is_pressed_key = match event.0 {
                PressType::Release => false,
                PressType::Press => true,
            };
            if new_is_pressed_key != focusable.is_pressed_key {
                focusable.is_pressed_key = new_is_pressed_key;
                if !new_is_pressed_key {
                    click_writer.send(UiNavClickEvent(entity));
                }
            }
        }
    }
}

/// System that reacts to `FocusLockEvent` events to lock or unlock navigation.
fn handle_lock_events(mut events: EventReader<UiNavLockEvent>, mut nav_state: ResMut<UiNavState>) {
    for event in events.read() {
        match *event {
            UiNavLockEvent::Lock => nav_state.locked = true,
            UiNavLockEvent::Unlock => nav_state.locked = false,
        }
    }
}

/// System that listens for keyboard or gamepad input and emits the appropriate navigation events.
#[allow(clippy::too_many_arguments)]
fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    gamepad_axis: Res<Axis<GamepadAxis>>,
    mut nav_request_writer: EventWriter<NavRequest>,
    mut nav_state: ResMut<UiNavState>,
    settings: Res<UiNavSettings>,
    mut input_manager: ResMut<UiNavInputManager>,
) {
    update_input_manager(
        &mut input_manager,
        &keys,
        &gamepads,
        &gamepad_buttons,
        &gamepad_axis,
    );

    if nav_state.menu.is_some() && !nav_state.locked {
        // send movement event
        if let Some(direction) = input_manager.direction() {
            if nav_state.direction.is_none() {
                // send movement key on first pressed
                nav_request_writer.send(NavRequest::Movement(direction));
            } else {
                // send movement key on timer tick while held
                let movement_speed = f32_lerp(
                    settings.movement_speed_slow,
                    settings.movement_speed_fast,
                    (nav_state.hold_timer.elapsed_secs() / settings.movement_acceleration_time)
                        .min(1.),
                );
                if nav_state.nav_timer.elapsed_secs() > movement_speed {
                    nav_request_writer.send(NavRequest::Movement(direction));
                    nav_state.nav_timer.reset();
                }
            }
            nav_state.direction = Some(direction);
        } else if nav_state.direction.is_some() {
            nav_state.clear_direction();
        }

        // send action press event
        if input_manager.just_pressed(ActionType::Action) {
            nav_request_writer.send(NavRequest::ActionPress);
        } else if input_manager.just_released(ActionType::Action) {
            nav_request_writer.send(NavRequest::ActionRelease);
        }
    } else if nav_state.direction.is_some() {
        // clear direction keys when the menu is locked, or we don't have a current menu
        nav_state.clear_direction();
    }

    // send cancel event
    // NOTE: This runs even when locked, in case the user wishes to lsiten for cancel events in order to unlock
    // navigation.
    if input_manager.just_pressed(ActionType::Cancel) {
        nav_request_writer.send(NavRequest::Cancel);
    }
}

/// System that handles internal `NavRequest` events.
#[allow(clippy::too_many_arguments)]
fn handle_nav_requests(
    mut events: EventReader<NavRequest>,
    focusable_query: Query<(), With<Focusable>>,
    menu_query: Query<&NavMenu>,
    mut set_focus_writer: EventWriter<InternalSetFocusEvent>,
    mut cancel_writer: EventWriter<UiNavCancelEvent>,
    mut move_writer: EventWriter<InternalFocusMoveEvent>,
    mut action_writer: EventWriter<InternalActionButtonEvent>,
    nav_state: Res<UiNavState>,
) {
    if let Some(current_menu) = nav_state.menu {
        let mut new_current = None;
        let mut move_direction = None;
        let mut action_press = None;

        for event in events.read() {
            match event {
                NavRequest::SetFocus {
                    entity: target,
                    interaction_type,
                } => {
                    if focusable_query.contains(*target) || menu_query.contains(*target) {
                        new_current = Some((*target, *interaction_type));
                    }
                }
                NavRequest::Movement(direction) => {
                    move_direction = Some(direction);
                }
                NavRequest::ActionPress => action_press = Some(PressType::Press),
                NavRequest::ActionRelease => action_press = Some(PressType::Release),
                NavRequest::Cancel => {
                    cancel_writer.send(UiNavCancelEvent(current_menu));
                }
            }
        }

        // Handle events in the appropriate order
        if let Some((entity, interaction_type)) = new_current {
            set_focus_writer.send(InternalSetFocusEvent {
                entity,
                interaction_type,
            });
        } else if let Some(press_type) = action_press {
            action_writer.send(InternalActionButtonEvent(press_type));
        } else if let Some(direction) = move_direction {
            move_writer.send(InternalFocusMoveEvent(*direction));
        }
    }
}

/// System that refreshes the UI navigation state whenever a focusable changes.
fn handle_focusable_changed(
    query: Query<(), Changed<Focusable>>,
    mut refresh_writer: EventWriter<InternalRefreshEvent>,
) {
    if !query.is_empty() {
        refresh_writer.send(InternalRefreshEvent);
    }
}

/// System that handles `RefreshFocusEvent` events
fn handle_internal_refresh_events(
    mut events: EventReader<InternalRefreshEvent>,
    nav_state: Res<UiNavState>,
    menu_query: Query<Entity, With<NavMenu>>,
    focusable_query: Query<(Entity, &Focusable)>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    events.clear();

    // ensure that we have a current menu
    let current_menu = nav_state.menu.and_then(|menu| menu_query.get(menu).ok());

    // Find the current focusable
    let current_focusable = focusable_query.iter().find(|(_, focusable)| {
        focusable.active() && focusable.menu.is_some() && focusable.menu == current_menu
    });

    // If we don't have a focusable, find one in the current menu. Prefer one that has `is_priority` set to true,
    // otherwise, use the first one we find.
    if current_focusable.is_none() {
        let new_focusable = focusable_query
            .iter()
            .filter(|(_, focusable)| {
                focusable.menu == current_menu && !focusable.is_disabled && !focusable.is_mouse_only
            })
            .reduce(|acc, e| {
                if e.1.is_priority && !acc.1.is_priority {
                    e
                } else {
                    acc
                }
            });
        if let Some((entity, _)) = new_focusable {
            nav_request_writer.send(NavRequest::SetFocus {
                entity,
                interaction_type: UiNavInteractionType::Auto,
            });
        }
    }
}
