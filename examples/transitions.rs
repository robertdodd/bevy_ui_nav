use bevy::{app::AppExit, prelude::*};
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin, ExampleUtilsPlugin))
        .init_state::<AppState>()
        .enable_state_scoped_entities::<AppState>()
        .init_state::<PlayState>()
        .enable_state_scoped_entities::<PlayState>()
        .add_systems(Startup, startup)
        .add_systems(OnEnter(AppState::Menu), on_enter_menu)
        .add_systems(OnEnter(AppState::Play), on_enter_play)
        .add_systems(OnEnter(AppState::Menu), setup_main_menu)
        .add_systems(OnEnter(PlayState::Pause), setup_pause_menu)
        .add_systems(
            Update,
            handle_click_events
                .run_if(on_event::<UiNavClickEvent>())
                .after(UiNavSet),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Play,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PlayState {
    #[default]
    None,
    Pause,
}

#[derive(Component, PartialEq, Eq, Clone, Debug)]
enum ButtonAction {
    Play,
    Menu,
    Debug(String),
    Quit,
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn on_enter_menu(mut next_play_state: ResMut<NextState<PlayState>>) {
    next_play_state.set(PlayState::None);
}

fn on_enter_play(mut next_play_state: ResMut<NextState<PlayState>>) {
    next_play_state.set(PlayState::Pause);
}

fn setup_main_menu(mut commands: Commands) {
    root_full_screen_centered(&mut commands, StateScoped(AppState::Menu), |p| {
        spawn_menu(true, false, p, (), |p| {
            menu_title(p, "Main Menu");
            menu_button(p, "Play", true, false, false, ButtonAction::Play);
            menu_button(
                p,
                "Settings",
                false,
                false,
                false,
                ButtonAction::Debug("Settings".to_string()),
            );
            menu_button(p, "Quit", false, false, false, ButtonAction::Quit);
        });
    });
}

fn setup_pause_menu(mut commands: Commands) {
    root_full_screen_centered(&mut commands, StateScoped(PlayState::Pause), |p| {
        spawn_menu(true, false, p, (), |p| {
            menu_title(p, "Pause");
            menu_button(
                p,
                "Debug",
                true,
                false,
                false,
                ButtonAction::Debug("Pause menu debug".to_string()),
            );
            menu_button(p, "Exit to Menu", false, false, false, ButtonAction::Menu);
        });
    });
}

fn handle_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for button_action in events.nav_iter().in_query(&query) {
        println!("ClickEvent: {:?}", button_action);
        match button_action {
            ButtonAction::Menu => {
                next_app_state.set(AppState::Menu);
            }
            ButtonAction::Play => {
                next_app_state.set(AppState::Play);
            }
            ButtonAction::Debug(debug_text) => println!("clicked: {debug_text}"),
            ButtonAction::Quit => {
                app_exit_writer.send(AppExit::Success);
            }
        };
    }
}
