use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};
use bevy_ui_nav::prelude::*;

mod components;
mod navbar;

pub use {components::*, navbar::*};

pub fn plugin(app: &mut App) {
    app.add_event::<OnPressed>();
    app.add_plugins(navbar::plugin);
    app.add_systems(PreUpdate, setup_new_pressables);
    app.add_systems(
        Update,
        (
            tick_pressed,
            handle_interactions,
            handle_focusable_click_events,
        ),
    );
}

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

/// Utility that traverses the `ChildOf` heirarchy until it finds a parent entity matching a query.
fn find_parent<D, F>(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    parent_query: &Query<D, F>,
) -> Option<Entity>
where
    D: QueryData,
    F: QueryFilter,
{
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

/// System that ticks the press animation on pressables.
fn tick_pressed(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PressAnimation, &mut PressablePressed), With<Pressable>>,
) {
    for (entity, mut animation, mut pressed) in query.iter_mut() {
        animation.0.tick(time.delta());
        if animation.0.just_finished() {
            pressed.0 = false;
            commands.entity(entity).remove::<PressAnimation>();
        }
    }
}

/// System that handles pressable interactions and emits press events for them.
fn handle_interactions(
    mut commands: Commands,
    mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Pressable>)>,
    mut press_writer: EventWriter<OnPressed>,
) {
    for (entity, interaction) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            commands.trigger_targets(OnPress, entity);
            press_writer.write(OnPressed(entity));
        }
    }
}

/// System that handles press events for focusables and transfers their action to an appropriate child pressable.
fn handle_focusable_click_events(
    mut commands: Commands,
    mut events: EventReader<PressEvent>,
    query: Query<(&Focusable, Option<&Pressables>, Has<Pressable>)>,
    mut pressable_query: Query<(&Pressable, &mut PressablePressed)>,
    mut press_writer: EventWriter<OnPressed>,
) {
    for event in events.read() {
        if let Ok((focusable, pressables, has_pressable)) = query.get(event.entity) {
            // ignore if the focusable does not match the action
            if !focusable.action.matches_action(event.action) {
                continue;
            }

            // find the appropriate pressable
            let target = if has_pressable {
                Some(event.entity)
            } else if let Some(pressables) = pressables {
                pressables.iter().find(|e| {
                    pressable_query
                        .get(*e)
                        .is_ok_and(|(pressable, _)| pressable.0 == event.action)
                })
            } else {
                None
            };

            // click on the appropriate pressable
            if let Some(target) = target {
                if let Ok((pressable, mut pressed)) = pressable_query.get_mut(target) {
                    if pressable.0 == event.action {
                        pressed.0 = true;
                        commands.entity(target).insert(PressAnimation::default());
                        commands.trigger_targets(OnPress, target);
                        press_writer.write(OnPressed(target));
                    }
                }
            }
        }
    }
}
