use bevy::{
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, focusable_colors)
        .run();
}

const N_COLUMNS: u16 = 4;

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
            border: UiRect::all(Px(4.)),
            padding: UiRect::all(Px(20.)),
            width: Px(200.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        children![Text::new(text)],
    )
}

fn grid(n_columns: u16) -> impl Bundle + use<> {
    Node {
        width: Val::Percent(100.),
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::auto(n_columns),
        grid_template_rows: RepeatedGridTrack::min_content(1),
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn(ui_root()).with_children(|p| {
        p.spawn((
            NavMenu::default(),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Px(10.),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(grid(N_COLUMNS)).with_children(|p| {
                for i in 0..(N_COLUMNS * N_COLUMNS) {
                    let title = format!("Button {}", i + 1);
                    p.spawn((
                        button(&title),
                        // prioritize the first focusable:
                        Focusable::default().with_priority(i == 0),
                    ));
                }
            });
        });
    });
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
