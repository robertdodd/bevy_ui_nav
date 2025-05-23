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
            (
                handle_click_events.run_if(on_event::<UiNavClickEvent>),
                handle_cancel_events.run_if(on_event::<UiNavCancelEvent>),
            )
                .after(UiNavSet),
        )
        .run();
}

#[derive(Component)]
struct MainMenu;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    Option1,
    Option2,
    Save,
    Quit,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, MainMenu).with_children(|p| {
            menu_button(p, "Option 1", true, false, false, ButtonAction::Option1);
            menu_button(p, "Disabled", false, true, false, ButtonAction::Option2);
            menu_button(p, "Option 2", false, false, false, ButtonAction::Option2);
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                width: Val::Px(500.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|p| {
                menu_button(p, "Cancel", false, false, false, ButtonAction::Quit);
                menu_button(p, "Save", false, false, false, ButtonAction::Save);
            });
        });
    });
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
            ButtonAction::Quit => {
                app_exit_writer.write(AppExit::Success);
            }
            ButtonAction::Save => (),
            _ => (),
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
