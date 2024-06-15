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
                handle_scroll,
                (
                    handle_click_events.run_if(on_event::<UiNavClickEvent>()),
                    handle_cancel_events.run_if(on_event::<UiNavCancelEvent>()),
                )
                    .after(UiNavSet),
            ),
        )
        .run();
}

#[derive(Component)]
struct MenuScroll;

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
    commands.spawn(Camera2dBundle::default());

    root_full_screen_centered(&mut commands, |p| {
        spawn_menu(true, false, p, MainMenu, |p| {
            menu_button(p, "Option 1", true, false, false, ButtonAction::Option1);
            menu_button(p, "Disabled", false, true, false, ButtonAction::Option2);
            menu_button(p, "Option 2", false, false, false, ButtonAction::Option2);
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    width: Val::Px(500.),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            })
            .with_children(|p| {
                menu_button(p, "Cancel", false, false, false, ButtonAction::Quit);
                menu_button(p, "Save", false, false, false, ButtonAction::Save);
            });
        });
    });
}

fn handle_scroll(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Style, With<MenuScroll>>) {
    let direction = if keys.just_pressed(KeyCode::KeyW) {
        Some(-1.)
    } else if keys.just_pressed(KeyCode::KeyS) {
        Some(1.)
    } else {
        None
    };
    if let Some(direction) = direction {
        let mut style = query.single_mut();
        let current = if let Val::Px(v) = style.top { v } else { 0. };
        style.top = Val::Px(current + direction * 10.);
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
            ButtonAction::Quit => {
                app_exit_writer.send(AppExit);
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
