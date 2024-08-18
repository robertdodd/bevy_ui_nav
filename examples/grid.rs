use bevy::prelude::*;
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin, ExampleUtilsPlugin))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let n_columns = 4;

    // Spawn multiple buttons in a grid, with no spacing between them, to check that navigation works correctly.
    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, (), |p| {
            button_grid(p, n_columns, |p| {
                for i in 0..(n_columns * n_columns) {
                    let title = format!("Button {}", i + 1);
                    spawn_grid_button(p, title.clone(), Name::new(title));
                }
            });
        });
    });
}

fn button_grid(
    parent: &mut ChildBuilder,
    n_columns: u16,
    children: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::auto(n_columns),
                grid_template_rows: RepeatedGridTrack::min_content(1),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(children);
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn spawn_grid_button(
    parent: &mut ChildBuilder,
    text: impl Into<String>,
    extras: impl Bundle,
) -> Entity {
    parent
        .spawn((
            StyledButton,
            Focusable::default(),
            ButtonBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Px(200.),
                    height: Val::Px(50.),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                background_color: BUTTON_BG_DEFAULT.into(),
                border_color: BUTTON_BG_DEFAULT.into(),
                ..default()
            },
            extras,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    color: Color::WHITE,
                    font_size: 20.,
                    ..default()
                },
            ));
        })
        .id()
}
