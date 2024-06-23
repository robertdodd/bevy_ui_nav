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
    Ok,
    Quit,
    Panic,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    root_full_screen_centered(&mut commands, |p| {
        spawn_menu(true, false, p, (), |p| {
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
                    app_exit_writer.send(AppExit);
                }
                ButtonAction::Ok => (),
                ButtonAction::Panic => panic!("Button should not have been clicked!"),
            };
        }
    }
}
