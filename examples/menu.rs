use bevy::{app::AppExit, prelude::*};
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin, ExampleUtilsPlugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            handle_click_events
                .after(UiNavSet)
                .run_if(on_event::<UiNavClickEvent>()),
        )
        .run();
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    Option1,
    Option2,
    Save,
    Quit,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

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
            // spawn a menu
            spawn_menu(true, false, p, (), |p| {
                spawn_button(p, "Option 1", true, false, ButtonAction::Option1);
                spawn_button(p, "Option 2", false, false, ButtonAction::Option2);
                spawn_button(p, "Disabled", false, true, ButtonAction::Option2);
            });

            // spawn a second menu
            spawn_menu(false, false, p, (), |p| {
                spawn_button(p, "Save", true, false, ButtonAction::Save);
                spawn_button(p, "Quit", false, false, ButtonAction::Quit);
            });
        });
}

fn handle_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.0) {
            println!("ClickEvent: {:?}", button_action);
            match *button_action {
                ButtonAction::Quit => app_exit_writer.send(AppExit),
                ButtonAction::Save => (),
                _ => (),
            };
        }
    }
}
