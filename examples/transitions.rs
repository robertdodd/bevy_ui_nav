use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .init_state::<AppState>()
        .enable_state_scoped_entities::<AppState>()
        .add_systems(Startup, startup)
        .add_systems(OnEnter(AppState::Menu), spawn_main_menu)
        .add_systems(OnEnter(AppState::Play), spawn_play_menu)
        .add_systems(
            Update,
            (
                handle_click_events.run_if(on_event::<PressEvent>),
                focusable_colors,
            )
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

#[derive(Component, PartialEq, Eq, Clone, Debug)]
enum ButtonAction {
    Play,
    Menu,
    Debug(String),
    Quit,
}

fn ui_root() -> impl Bundle + use<> {
    (
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::SLATE_800.into()),
    )
}

fn menu(focused: bool) -> impl Bundle + use<> {
    (
        Name::new("Menu"),
        NavMenu::default().with_priority(focused),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Px(10.),
            width: Px(400.),
            padding: UiRect::all(Px(20.)),
            border: UiRect::all(Px(2.)),
            ..default()
        },
        BackgroundColor(tailwind::SLATE_700.into()),
        BorderRadius::all(Px(20.)),
    )
}

fn button(text: &str) -> impl Bundle + use<> {
    (
        Name::new("Button"),
        Button,
        Node {
            width: Percent(100.),
            border: UiRect::all(Px(4.)),
            padding: UiRect::all(Px(20.)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        children![Text::new(text)],
    )
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        ui_root(),
        StateScoped(AppState::Menu),
        children![(
            menu(true),
            children![
                Text::new("Main Menu"),
                (button("Play"), Focusable::prioritized(), ButtonAction::Play),
                (
                    button("Settings"),
                    Focusable::default(),
                    ButtonAction::Debug("Settings".to_string())
                ),
                (button("Quit"), Focusable::default(), ButtonAction::Quit),
            ]
        )],
    ));
}

fn spawn_play_menu(mut commands: Commands) {
    commands.spawn((
        ui_root(),
        StateScoped(AppState::Play),
        children![(
            menu(true),
            children![
                Text::new("Play Menu"),
                (
                    button("Debug"),
                    Focusable::prioritized(),
                    ButtonAction::Debug("Pause debug".to_string())
                ),
                (button("Exit"), Focusable::default(), ButtonAction::Menu),
            ]
        )],
    ));
}

fn handle_click_events(
    mut events: EventReader<PressEvent>,
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
                app_exit_writer.write(AppExit::Success);
            }
        };
    }
}

fn focusable_colors(mut query: Query<(&Focusable, &mut BorderColor), Changed<Focusable>>) {
    for (focusable, mut border_color) in query.iter_mut() {
        border_color.0 = match focusable.state() {
            FocusState::None => Color::WHITE,
            FocusState::Focused => tailwind::YELLOW_300.into(),
            FocusState::Disabled => Color::WHITE.with_alpha(0.25),
        };
    }
}
