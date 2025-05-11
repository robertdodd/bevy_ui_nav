use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
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
    commands.spawn(Camera2d);

    let n_columns = 4;

    // Spawn multiple buttons in a grid, with no spacing between them, to check that navigation works correctly.
    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, ()).with_children(|p| {
            button_grid(p, n_columns).with_children(|p| {
                for i in 0..(n_columns * n_columns) {
                    let title = format!("Button {}", i + 1);
                    spawn_grid_button(p, title.clone(), Name::new(title));
                }
            });
        });
    });
}

fn button_grid<'w>(
    spawner: &'w mut RelatedSpawnerCommands<ChildOf>,
    n_columns: u16,
) -> EntityCommands<'w> {
    let cmds = spawner.spawn(Node {
        width: Val::Percent(100.),
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::auto(n_columns),
        grid_template_rows: RepeatedGridTrack::min_content(1),
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    });
    cmds
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn spawn_grid_button(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    extras: impl Bundle,
) {
    spawner
        .spawn((
            StyledButton,
            Focusable::default(),
            Button,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(200.),
                height: Val::Px(50.),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor(BUTTON_BG_DEFAULT.into()),
            BorderColor(BUTTON_BG_DEFAULT.into()),
            extras,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(text),
                TextColor(Color::WHITE),
                TextFont::from_font_size(20.),
            ));
        });
}
