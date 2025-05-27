use bevy::prelude::*;
use bevy_ui_nav::prelude::*;

use super::*;

use super::{Pressable, Pressables};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            on_pressable_pressed.run_if(on_event::<OnPressed>),
            on_focusable_pressed.run_if(on_event::<FocusablePressed>),
        )
            .after(UiNavSet),
    );
}

#[derive(Component, Debug, Default)]
pub struct FocusableNav(pub Option<Entity>);

/// Handle click events on header nav buttons.
fn on_pressable_pressed(
    mut events: EventReader<OnPressed>,
    mut nav_query: Query<&mut FocusableNav>,
    pressable_query: Query<&PressableOf>,
) {
    for event in events.read() {
        if let Ok(pressable_of) = pressable_query.get(event.0) {
            if let Ok(mut nav) = nav_query.get_mut(pressable_of.0) {
                nav.0 = Some(event.0);
            }
        }
    }
}

/// Handle focusable press events on focusable navs.
fn on_focusable_pressed(
    mut commands: Commands,
    mut events: EventReader<FocusablePressed>,
    mut query: Query<(&Pressables, &mut FocusableNav)>,
    mut pressable_query: Query<&mut PressablePressed, With<Pressable>>,
    mut press_writer: EventWriter<OnPressed>,
) {
    for event in events.read() {
        if let Ok((pressables, mut nav)) = query.get_mut(event.entity) {
            // find the index of the current pressable
            let index = pressables
                .iter()
                .enumerate()
                .find(|(_, entity)| Some(*entity) == nav.0)
                .map(|(index, _)| index);

            // define the next index
            let next_index = match (event.action, index) {
                (_, None) => Some(0),
                (PressableAction::Left, Some(index)) => Some(if index == 0 {
                    pressables.len() - 1
                } else {
                    index - 1
                }),
                (PressableAction::Right, Some(index)) => Some(if index == pressables.len() - 1 {
                    0
                } else {
                    index + 1
                }),
                _ => None,
            };

            // update the current index
            if let Some(entity) = next_index.and_then(|index| pressables.get(index)) {
                nav.0 = Some(*entity);

                // send a click event to the pressable
                if let Ok(mut pressed) = pressable_query.get_mut(*entity) {
                    pressed.0 = true;
                    commands.entity(*entity).insert(PressAnimation::default());
                }
                commands.trigger_targets(OnPress, *entity);
                press_writer.write(OnPressed(*entity));
            }
        }
    }
}
