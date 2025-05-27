use std::time::Duration;

use bevy::prelude::{Val::*, *};
use bevy_ui_nav::prelude::*;

use super::Pressable;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ShowFocusables>();
    app.add_systems(
        Update,
        (
            setup_new_focusables,
            handle_focusable_click_events,
            focusable_system,
            update_focusable_animations,
            on_show_focusables_changed.run_if(resource_exists_and_changed::<ShowFocusables>),
            show_focus_state.run_if(on_event::<UiNavFocusChangedEvent>),
            hide_focus_state.run_if(on_event::<CursorMoved>),
        )
            .after(UiNavSet),
    );
}

const FOCUSABLE_OUTLINE_START: f32 = 10.;
const FOCUSABLE_OUTLINE_END: f32 = 2.;
const FOCUSABLE_ANIMATION_SECONDS: f32 = 0.25;

/// Resource used to control whether focusable outlines are visible.
///
/// Allows focusables to be hidden when using the mouse and visible when receiving keyboard or gamepad navigation
/// input.
#[derive(Resource, Debug, Default)]
struct ShowFocusables(pub bool);

/// Component holding the timer for focusable outline animations.
///
/// The animation is removed when the timer finishes.
#[derive(Component, Debug)]
struct FocusableAnimation(Timer);

impl Default for FocusableAnimation {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(FOCUSABLE_ANIMATION_SECONDS),
            TimerMode::Once,
        ))
    }
}

/// System that starts or stops the focusable animation when a focusable's state changes.
fn focusable_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Focusable, &mut Outline, Has<FocusableAnimation>),
        Changed<Focusable>,
    >,
) {
    for (entity, focusable, mut outline, has_animation) in query.iter_mut() {
        // insert an outline animation if focused and we dont have one, or whenever pressed
        let is_focused = matches!(focusable.state(), FocusState::Focused);
        if is_focused {
            if focusable.state() == FocusState::Focused {
                outline.color = Color::WHITE;
                commands
                    .entity(entity)
                    .insert(FocusableAnimation::default());
            }
        } else {
            outline.color = Color::NONE;
            if has_animation {
                commands.entity(entity).remove::<FocusableAnimation>();
            }
        }
    }
}

/// System that updates focusable outline animations.
fn update_focusable_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FocusableAnimation, &mut Outline), With<Focusable>>,
) {
    for (entity, mut animation, mut outline) in query.iter_mut() {
        animation.0.tick(time.delta());
        let width = FOCUSABLE_OUTLINE_START.lerp(FOCUSABLE_OUTLINE_END, animation.0.fraction());
        outline.width = Px(width);
        if animation.0.just_finished() {
            commands.entity(entity).remove::<FocusableAnimation>();
        }
    }
}

/// System that starts the focusable outline animation when a focusable is pressed
fn handle_focusable_click_events(mut commands: Commands, mut events: EventReader<PressEvent>) {
    for event in events.read() {
        commands
            .entity(event.entity)
            .insert(FocusableAnimation::default());
    }
}

/// System that adds an `Outline` component to newly added focusables.
fn setup_new_focusables(
    mut commands: Commands,
    query: Query<(Entity, &Focusable), (Added<Focusable>, Without<Outline>)>,
    show_focusables: Res<ShowFocusables>,
) {
    for (entity, focusable) in query.iter() {
        commands.entity(entity).insert(Outline::new(
            Px(FOCUSABLE_OUTLINE_END),
            Val::ZERO,
            if show_focusables.0 && focusable.state() == FocusState::Focused {
                Color::WHITE
            } else {
                Color::NONE
            },
        ));
    }
}

/// System that shows or hides focusable outlines when the `ShowFocusables` resource changes.
fn on_show_focusables_changed(
    show_focusables: Res<ShowFocusables>,
    mut query: Query<(&Focusable, &mut Outline)>,
) {
    for (focusable, mut outline) in query.iter_mut() {
        outline.color = if show_focusables.0 && focusable.state() == FocusState::Focused {
            Color::WHITE
        } else {
            Color::NONE
        };
    }
}

/// System that shows focusable outlines when focus change events are emitted.
fn show_focus_state(
    mut events: EventReader<UiNavFocusChangedEvent>,
    mut show_focusables: ResMut<ShowFocusables>,
) {
    for event in events.read() {
        if event.interaction_type == UiNavInteractionType::Button {
            show_focusables.0 = true;
        }
    }
}

/// System that hides focusable outlines when any mouse events effect a pressable.
fn hide_focus_state(
    mut events: EventReader<CursorMoved>,
    query: Query<(), (Changed<Interaction>, With<Pressable>)>,
    mut show_focusables: ResMut<ShowFocusables>,
) {
    events.clear();
    if !query.is_empty() {
        show_focusables.0 = false;
    }
}
