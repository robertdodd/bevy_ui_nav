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
                focusable_colors,
            )
                .after(UiNavSet),
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

fn ui_root() -> impl Bundle + use<> {
    (
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Px(20.),
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
            border: UiRect::all(Px(4.)),
            padding: UiRect::all(Px(10.)),
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
        children![
            (
                menu(true),
                children![
                    Text::new(
                        "You cannot leave this menu without manually emitting a `SetFocus` event."
                    ),
                    (
                        button("Option 1"),
                        ButtonAction::Option1,
                        Focusable::prioritized()
                    ),
                    (
                        button("Option 2"),
                        ButtonAction::Option2,
                        Focusable::default()
                    ),
                    (
                        button("Disabled"),
                        ButtonAction::Option2,
                        Focusable::default().disabled()
                    ),
                ],
            ),
            (
                menu(false),
                children![
                    Text::new("You cannot enter this menu."),
                    (button("Save"), ButtonAction::Save, Focusable::prioritized()),
                    (button("Quit"), ButtonAction::Quit, Focusable::default()),
                ],
            ),
        ],
    ));
}

fn handle_click_events(
    mut events: EventReader<FocusablePressed>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.entity) {
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
