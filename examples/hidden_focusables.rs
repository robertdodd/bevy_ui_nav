use bevy::{app::AppExit, ecs::relationship::RelatedSpawnerCommands, prelude::*};
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
                handle_keys,
                handle_click_events
                    .after(UiNavSet)
                    .run_if(on_event::<UiNavClickEvent>),
            ),
        )
        .run();
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    Ok,
    Quit,
    Panic,
}

#[derive(Component, Debug, Clone)]
struct ButtonWrapper;

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, ()).with_children(|p| {
            text_widget(p, FontSize::Small, "Press <TAB> to reveal focusables");
            button_wrapper_widget(p).with_children(|p| {
                menu_buttoni(
                    p,
                    "Hidden via Display",
                    (Focusable::default(), ButtonAction::Panic),
                    |node: &mut Node| {
                        node.display = Display::None;
                    },
                );
                menu_buttoni(
                    p,
                    "Hidden via Visibility",
                    (
                        Focusable::default(),
                        ButtonAction::Panic,
                        Visibility::Hidden,
                    ),
                    |_| {},
                );
                menu_buttoni(
                    p,
                    "Hidden via 0 size",
                    (Focusable::default(), ButtonAction::Panic),
                    |node: &mut Node| {
                        node.max_width = Val::ZERO;
                        node.max_height = Val::ZERO;
                        node.padding = UiRect::ZERO;
                        node.border = UiRect::ZERO;
                        node.overflow = Overflow::clip();
                    },
                );
                p.spawn(Node {
                    display: Display::None,
                    ..default()
                })
                .with_children(|p| {
                    menu_button(
                        p,
                        "Hidden via parent display",
                        false,
                        false,
                        false,
                        ButtonAction::Panic,
                    );
                });
                p.spawn((Node::default(), Visibility::Hidden))
                    .with_children(|p| {
                        menu_button(
                            p,
                            "Hidden via parent visibility",
                            false,
                            false,
                            false,
                            ButtonAction::Panic,
                        );
                    });
                menu_button(p, "Button 1", true, false, false, ButtonAction::Ok);
                menu_button(p, "Button 2", false, false, false, ButtonAction::Ok);
                menu_button(p, "Quit", false, true, false, ButtonAction::Quit);
            });
        });
    });
}

/// Utility that spawns a nav menu.
pub fn button_wrapper_widget<'w>(
    parent: &'w mut RelatedSpawnerCommands<ChildOf>,
) -> EntityCommands<'w> {
    let cmds = parent.spawn((
        ButtonWrapper,
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            display: Display::None,
            ..default()
        },
    ));
    cmds
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
                ButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                ButtonAction::Ok => (),
                ButtonAction::Panic => panic!("Button should not have been clicked!"),
            };
        }
    }
}

fn handle_keys(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Node, With<ButtonWrapper>>) {
    if keys.just_pressed(KeyCode::Tab) {
        if let Ok(mut node) = query.single_mut() {
            node.display = match node.display {
                Display::None => Display::Flex,
                _ => Display::Flex,
            };
        }
    }
}
