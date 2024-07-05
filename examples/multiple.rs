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

#[derive(Component)]
struct MenuParent(Entity);

#[derive(Component, PartialEq, Eq, Clone, Debug)]
enum ButtonAction {
    Menu(Entity),
    Debug(String),
    Quit,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let mut settings_menu = None;
    let mut graphics_menu = None;
    let mut sound_menu = None;

    // Spawn a button that can only be triggered by clicking it
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                margin: UiRect::vertical(Val::Px(20.)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            menu_button(p, "Quit", true, false, true, ButtonAction::Quit);
        });

    commands
        .spawn((
            MenuScroll,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            // Main Menu
            settings_menu = Some(spawn_menu(true, false, p, MainMenu, |p| {
                menu_title(p, "Settings");
            }));

            // Graphics Menu
            graphics_menu = Some(spawn_menu(false, false, p, MainMenu, |p| {
                menu_title(p, "Graphics");
            }));

            // Sound Menu
            sound_menu = Some(spawn_menu(false, false, p, MainMenu, |p| {
                menu_title(p, "Sound");
            }));
        });

    if let (Some(settings_menu), Some(graphics_menu), Some(sound_menu)) =
        (settings_menu, graphics_menu, sound_menu)
    {
        // Add buttons to main settings menu
        commands.entity(settings_menu).with_children(|p| {
            menu_button(
                p,
                "Graphics",
                true,
                false,
                false,
                ButtonAction::Menu(graphics_menu),
            );
            menu_button(
                p,
                "Sound",
                false,
                false,
                false,
                ButtonAction::Menu(sound_menu),
            );
            menu_button(p, "Quit", false, false, false, ButtonAction::Quit);
        });

        // Add buttons to graphics settings menu
        commands
            .entity(graphics_menu)
            .insert(MenuParent(settings_menu))
            .with_children(|p| {
                menu_button(
                    p,
                    "Option 1",
                    true,
                    false,
                    false,
                    ButtonAction::Debug("Graphics Option 1".to_string()),
                );
                menu_button(
                    p,
                    "Option 2",
                    false,
                    false,
                    false,
                    ButtonAction::Debug("Graphics Option 2".to_string()),
                );
                menu_button(
                    p,
                    "Cancel",
                    false,
                    false,
                    false,
                    ButtonAction::Menu(settings_menu),
                );
            });

        // Add buttons to sound settings menu
        commands
            .entity(sound_menu)
            .insert(MenuParent(settings_menu))
            .with_children(|p| {
                menu_button(
                    p,
                    "Option 1",
                    true,
                    false,
                    false,
                    ButtonAction::Debug("Sound Option 1".to_string()),
                );
                menu_button(
                    p,
                    "Option 2",
                    false,
                    false,
                    false,
                    ButtonAction::Debug("Sound Option 2".to_string()),
                );
                menu_button(
                    p,
                    "Cancel",
                    false,
                    false,
                    false,
                    ButtonAction::Menu(settings_menu),
                );
            });
    }
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
    mut nav_request_writer: EventWriter<NavRequest>,
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
        match button_action {
            ButtonAction::Menu(menu) => {
                nav_request_writer.send(NavRequest::SetFocus {
                    entity: *menu,
                    interaction_type: UiNavInteractionType::Manual,
                });
            }
            ButtonAction::Debug(debug_text) => println!("clicked: {debug_text}"),
            ButtonAction::Quit => {
                app_exit_writer.send(AppExit::Success);
            }
        };
    }
}

fn handle_cancel_events(
    mut events: EventReader<UiNavCancelEvent>,
    query: Query<&MenuParent>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for event in events.read() {
        if let Ok(menu_parent) = query.get(event.0) {
            println!("CancelEvent: {:?}", event);
            nav_request_writer.send(NavRequest::SetFocus {
                entity: menu_parent.0,
                interaction_type: UiNavInteractionType::Manual,
            });
        }
    }
}
