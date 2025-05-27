use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_click_events.run_if(on_event::<FocusablePressed>),
                handle_cancel_events.run_if(on_event::<UiNavCancelEvent>),
                focusable_colors,
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

fn ui_root() -> impl Bundle + use<> {
    (
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::SLATE_800.into()),
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

    commands.spawn((
        ui_root(),
        children![(
            NavMenu::default(),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Px(10.),
                ..default()
            },
            children![
                Text::new("Hit <ESC> to enable Option 2."),
                (
                    button("Option 1"),
                    ButtonAction::Option1,
                    Focusable::prioritized()
                ),
                (
                    button("Disabled"),
                    ButtonAction::Option2,
                    Focusable::default().disabled()
                ),
                (
                    button("Option 2"),
                    ButtonAction::Option2,
                    Focusable::default()
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        width: Val::Px(500.),
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: Px(10.),
                        ..default()
                    },
                    children![
                        (button("Save"), ButtonAction::Save, Focusable::default()),
                        (button("Quit"), ButtonAction::Quit, Focusable::default()),
                    ]
                )
            ],
        )],
    ));
}

fn handle_click_events(
    mut events: EventReader<FocusablePressed>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
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

fn focusable_colors(mut query: Query<(&Focusable, &mut BorderColor), Changed<Focusable>>) {
    for (focusable, mut border_color) in query.iter_mut() {
        border_color.0 = match focusable.state() {
            FocusState::None => Color::WHITE,
            FocusState::Focused => tailwind::YELLOW_300.into(),
            FocusState::Disabled => Color::WHITE.with_alpha(0.25),
        };
    }
}
