use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    ui::RelativeCursorPosition,
};

use crate::{
    components::*,
    default_input_map::DEFAULT_INPUT_MAP,
    events::*,
    input::*,
    resources::*,
    spatial_map::{UiSpatialMap, UiSpatialMapEvent},
    types::*,
    utils::*,
};

pub struct BevyUiNavPlugin;

impl Plugin for BevyUiNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiNavClickEvent>()
            .add_event::<UiNavCancelEvent>()
            .add_event::<NavRequest>()
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
                        (
                            handle_interactions.run_if(
                                on_event::<CursorMoved>().or_else(on_event::<MouseButtonInput>()),
                            ),
                            handle_gamepad_input,
                            handle_keyboard_input,
                        )
                            .chain(),
                        tick_pressed_timer,
                        handle_focusable_changed,
                    )
                        .before(UiNavSet),
                    handle_nav_requests
                        .run_if(on_event::<NavRequest>())
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

    nav_request_writer.send(NavRequest::Refresh);
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
            cmds.try_insert(Interaction::default());
        }
        if !has_relative_cursor_position {
            cmds.try_insert(RelativeCursorPosition::default());
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
            // update focusable
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

    nav_request_writer.send(NavRequest::Refresh);
}

/// System that listens for keyboard or gamepad input and emits the appropriate navigation events.
#[allow(clippy::too_many_arguments)]
fn handle_gamepad_input(
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
    menu_query: Query<(Entity, &NavMenu)>,
    mut cancel_writer: EventWriter<UiNavCancelEvent>,
    mut nav_state: ResMut<UiNavState>,
    mut query: Query<(Entity, &mut Focusable, &Node, &GlobalTransform)>,
    mut click_writer: EventWriter<UiNavClickEvent>,
    mut focus_change_writer: EventWriter<UiNavFocusChangedEvent>,
) {
    let mut spatial_map = UiSpatialMap::new(&menu_query, &query.to_readonly(), &nav_state);

    let mut is_cancel: bool = false;
    for event in events.read() {
        match event {
            NavRequest::SetFocus {
                entity,
                interaction_type,
            } => {
                spatial_map.set_focus(*entity, *interaction_type);
            }
            NavRequest::Movement(direction) => {
                spatial_map.apply_movement(*direction);
            }
            NavRequest::ActionPress => {
                spatial_map.press();
            }
            NavRequest::ActionRelease => {
                spatial_map.release();
            }
            NavRequest::Cancel => {
                is_cancel = true;
            }
            NavRequest::Lock => {
                spatial_map.lock();
            }
            NavRequest::Unlock => {
                spatial_map.unlock();
            }
            NavRequest::Refresh => (),
        }
    }

    for event in spatial_map.events() {
        match event {
            UiSpatialMapEvent::Press(entity) => {
                if let Ok((_, mut focusable, _, _)) = query.get_mut(*entity) {
                    focusable.is_pressed_key = true;
                }
            }
            UiSpatialMapEvent::Release(entity) => {
                if let Ok((_, mut focusable, _, _)) = query.get_mut(*entity) {
                    focusable.is_pressed_key = false;
                }
            }
            UiSpatialMapEvent::Click(entity) => {
                click_writer.send(UiNavClickEvent(*entity));
            }
        }
    }

    // Focus on new menu
    if let Some(new_menu) = spatial_map.get_new_menu() {
        nav_state.menu = new_menu;
    }

    // Focus on new focusable
    if let Some((new_focusable, interaction_type)) = spatial_map.get_new_focusable() {
        for (entity, mut focusable, _, _) in query.iter_mut() {
            focusable.is_focused = Some(entity) == new_focusable;
            if focusable.is_focused {
                focus_change_writer.send(UiNavFocusChangedEvent {
                    entity,
                    interaction_type,
                });
            }
        }
    }

    // Focus on new mouse-only focusable
    if let Some(new_focusable) = spatial_map.get_new_mouse_only_focusable() {
        for (entity, mut focusable, _, _) in query.iter_mut() {
            if focusable.is_mouse_only {
                focusable.is_focused = Some(entity) == new_focusable;
                if focusable.is_focused {
                    focus_change_writer.send(UiNavFocusChangedEvent {
                        entity,
                        interaction_type: UiNavInteractionType::Mouse,
                    });
                }
            }
        }
    }

    // Handle new locked state
    if let Some(locked) = spatial_map.get_new_locked() {
        nav_state.locked = locked
    }

    // Handle cancel events
    if let (true, Some(menu)) = (is_cancel, spatial_map.menu()) {
        cancel_writer.send(UiNavCancelEvent(menu));
    }
}

/// System that refreshes the UI navigation state whenever a focusable changes.
fn handle_focusable_changed(
    query: Query<(), Changed<Focusable>>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    if !query.is_empty() {
        nav_request_writer.send(NavRequest::Refresh);
    }
}

/// This system prints out all keyboard events as they come in
fn handle_keyboard_input(
    mut events: EventReader<KeyboardInput>,
    mut nav_request_writer: EventWriter<NavRequest>,
    input_manager: Res<UiNavInputManager>,
) {
    if events.is_empty() {
        return;
    }
    for event in events.read() {
        for action in input_manager.input_map.iter() {
            if let InputMapping::Key { keycode, action } = action {
                if *keycode == event.key_code {
                    let nav_request = match (action, event.state) {
                        (ActionType::Up, ButtonState::Pressed) => {
                            Some(NavRequest::Movement(UiNavDirection::Up))
                        }
                        (ActionType::Down, ButtonState::Pressed) => {
                            Some(NavRequest::Movement(UiNavDirection::Down))
                        }
                        (ActionType::Left, ButtonState::Pressed) => {
                            Some(NavRequest::Movement(UiNavDirection::Left))
                        }
                        (ActionType::Right, ButtonState::Pressed) => {
                            Some(NavRequest::Movement(UiNavDirection::Right))
                        }
                        (ActionType::Action, ButtonState::Pressed) => Some(NavRequest::ActionPress),
                        (ActionType::Action, ButtonState::Released) => {
                            Some(NavRequest::ActionRelease)
                        }
                        (ActionType::Cancel, ButtonState::Pressed) => Some(NavRequest::Cancel),
                        _ => None,
                    };
                    if let Some(nav_request) = nav_request {
                        nav_request_writer.send(nav_request);
                    }
                }
            }
        }
    }
}
