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
                handle_keys,
                handle_click_events
                    .after(UiNavSet)
                    .run_if(on_event::<UiNavClickEvent>()),
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
    commands.spawn(Camera2dBundle::default());

    root_full_screen_centered(&mut commands, |p| {
        spawn_menu(true, false, p, (), |p| {
            text_widget(p, FontSize::Small, "Press <TAB> to reveal focusables");
            button_wrapper_widget(p, |p| {
                menu_buttoni(
                    p,
                    "Hidden via Display",
                    (Focusable::default(), ButtonAction::Panic),
                    |b: &mut ButtonBundle| {
                        b.style.display = Display::None;
                    },
                );
                menu_buttoni(
                    p,
                    "Hidden via Visibility",
                    (Focusable::default(), ButtonAction::Panic),
                    |b: &mut ButtonBundle| {
                        b.visibility = Visibility::Hidden;
                    },
                );
                menu_buttoni(
                    p,
                    "Hidden via 0 size",
                    (Focusable::default(), ButtonAction::Panic),
                    |b: &mut ButtonBundle| {
                        b.style.max_width = Val::ZERO;
                        b.style.max_height = Val::ZERO;
                        b.style.padding = UiRect::ZERO;
                        b.style.border = UiRect::ZERO;
                        b.style.overflow = Overflow::clip();
                    },
                );
                menu_button(p, "Button 1", true, false, false, ButtonAction::Ok);
                menu_button(p, "Button 2", false, false, false, ButtonAction::Ok);
                menu_button(p, "Quit", false, true, false, ButtonAction::Quit);
            });
        });
    });
}

/// Utility that spawns a nav menu.
pub fn button_wrapper_widget(parent: &mut ChildBuilder, children: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn((
            ButtonWrapper,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(children);
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
                    app_exit_writer.send(AppExit::Success);
                }
                ButtonAction::Ok => (),
                ButtonAction::Panic => panic!("Button should not have been clicked!"),
            };
        }
    }
}

fn handle_keys(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Style, With<ButtonWrapper>>) {
    if keys.just_pressed(KeyCode::Tab) {
        if let Ok(mut style) = query.get_single_mut() {
            style.display = match style.display {
                Display::None => Display::Flex,
                _ => Display::Flex,
            };
        }
    }
}
