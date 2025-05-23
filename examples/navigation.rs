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

const BUTTON_SIZE_SM: f32 = 50.;
const BUTTON_SPACER: f32 = 10.;
const BUTTON_SIZE_LG: f32 = BUTTON_SIZE_SM * 2. + BUTTON_SPACER;
const BUTTON_HEIGHT: f32 = BUTTON_SIZE_SM;
const FORM_SPACER: f32 = 20.;

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn multiple buttons in a grid, with no spacing between them, to check that navigation works correctly.
    root_full_screen_centered(&mut commands, |p| {
        spawn_menu(true, false, p, ()).with_children(|p| {
            form_control_sm(p, "Item 1");
            form_control_lg(p, "Item 2");
            form_control_sm(p, "Item 3");
            form_control_lg(p, "Item 4");
            form_group(p, |p| {
                form_label(p, "Large Button");
                button(p, "Large", Val::Percent(100.));
            });
        });
    });
}

fn root_full_screen_centered(
    commands: &mut Commands,
    children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(children);
}

fn button_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Px(400.),
            ..default()
        })
        .with_children(children);
}

fn form_group(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            margin: UiRect::bottom(Val::Px(FORM_SPACER)),
            ..default()
        })
        .with_children(children);
}

fn form_label(parent: &mut RelatedSpawnerCommands<ChildOf>, text: impl Into<String>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            margin: UiRect::bottom(Val::Px(FORM_SPACER)),
            ..default()
        })
        .with_children(|p| {
            text_widget(p, FontSize::Small, text);
        });
}

fn button_container(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Px(BUTTON_SIZE_LG),
            height: Val::Px(BUTTON_HEIGHT),
            ..default()
        })
        .with_children(children);
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    width: Val,
) -> Entity {
    let text: String = text.into();
    parent
        .spawn((
            StyledButton,
            Focusable::default(),
            Button,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width,
                height: Val::Px(BUTTON_HEIGHT),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor(BUTTON_BG_DEFAULT.into()),
            BorderColor(BUTTON_BG_DEFAULT.into()),
            Name::new(text.clone()),
        ))
        .with_children(|p| {
            text_widget(p, FontSize::Small, text);
        })
        .id()
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn form_control_sm(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label_text: impl Into<String>,
) {
    form_group(parent, |p| {
        form_label(p, label_text);
        button_row(p, |p| {
            button_container(p, |p| {
                button(p, "<<", Val::Px(BUTTON_SIZE_SM));
                button(p, "<", Val::Px(BUTTON_SIZE_SM));
            });
            text_widget(p, FontSize::Small, "value");
            button_container(p, |p| {
                button(p, ">", Val::Px(BUTTON_SIZE_SM));
                button(p, ">>", Val::Px(BUTTON_SIZE_SM));
            });
        });
    });
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn form_control_lg(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label_text: impl Into<String>,
) {
    form_group(parent, |p| {
        form_label(p, label_text);
        button_row(p, |p| {
            button_container(p, |p| {
                button(p, "<", Val::Px(BUTTON_SIZE_LG));
            });
            text_widget(p, FontSize::Small, "value");
            button_container(p, |p| {
                button(p, ">", Val::Px(BUTTON_SIZE_LG));
            });
        });
    });
}
