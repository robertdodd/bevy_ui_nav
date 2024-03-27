use bevy::{app::AppExit, prelude::*, window::PresentMode};
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            BevyUiNavPlugin,
            ExampleUtilsPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_loading,
                (
                    handle_click_events.run_if(on_event::<UiNavClickEvent>()),
                    handle_cancel_events.run_if(on_event::<UiNavCancelEvent>()),
                    handle_nav_events.run_if(on_event::<UiNavFocusChangedEvent>()),
                )
                    .after(UiNavSet),
            ),
        )
        .run();
}

#[derive(Resource)]
struct LoadingTimer(pub Timer);

#[derive(Component)]
struct MainMenu;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    Option1,
    Option2,
    Quit,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(LoadingTimer(Timer::from_seconds(2., TimerMode::Once)));

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_menu(true, false, p, MainMenu, |p| {
                spawn_button(p, "Option 1", true, true, ButtonAction::Option1);
                spawn_button(p, "Option 2", false, true, ButtonAction::Option2);
                spawn_button(p, "Quit", false, true, ButtonAction::Quit);
            });
        });
}

fn handle_loading(
    time: Res<Time>,
    mut loading_timer: ResMut<LoadingTimer>,
    mut query: Query<&mut Focusable>,
) {
    loading_timer.0.tick(time.delta());

    if loading_timer.0.just_finished() {
        for mut focusable in query.iter_mut() {
            focusable.enable();
        }
    }
}

fn handle_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    // This is equivalent to:
    // ```
    // for event in events.read() {
    //     if let Ok(button_action) = query.get(event.0) {
    //         ...
    //     }
    // }
    // ```
    for button_action in events.nav_iter().in_query(&query) {
        println!("ClickEvent: {:?}", button_action);
        match *button_action {
            ButtonAction::Quit => app_exit_writer.send(AppExit),
            ButtonAction::Option1 => (),
            ButtonAction::Option2 => (),
        };
    }
}

fn handle_cancel_events(mut events: EventReader<UiNavCancelEvent>, query: Query<&MainMenu>) {
    for event in events.read() {
        if query.contains(event.0) {
            println!("CancelEvent: {:?}", event);
        }
    }
}

fn handle_nav_events(mut events: EventReader<UiNavFocusChangedEvent>) {
    for event in events.read() {
        println!("UiNavEvent: {:?}", event);
    }
}
