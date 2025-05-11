#![allow(dead_code)]

use bevy::{color::palettes::css, ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_ui_nav::prelude::*;

pub struct ExampleUtilsPlugin;

impl Plugin for ExampleUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_style.after(UiNavSet));
    }
}

pub const BUTTON_BG_DEFAULT: Srgba = css::DARK_GRAY;
pub const BUTTON_BG_ACTIVE: Srgba = css::GRAY;
pub const BUTTON_BG_PRESS: Srgba = css::BLACK;
pub const BUTTON_BG_DISABLED: Srgba = css::BEIGE;

pub const FONT_SIZE_SM: f32 = 20.;
pub const FONT_SIZE_LG: f32 = 40.;

#[derive(Component)]
pub struct StyledButton;

pub enum FontSize {
    Small,
    Large,
}

impl From<FontSize> for f32 {
    fn from(value: FontSize) -> Self {
        match value {
            FontSize::Small => FONT_SIZE_SM,
            FontSize::Large => FONT_SIZE_LG,
        }
    }
}

/// Utility that spawns a root node covering the entire screen, with content aligned to the center.
pub fn root_full_screen_centered(
    commands: &mut Commands,
    extras: impl Bundle,
    children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            extras,
        ))
        .with_children(children);
}

/// Utility that spawns a nav menu.
pub fn spawn_menu<'w>(
    active: bool,
    locked: bool,
    parent: &'w mut RelatedSpawnerCommands<ChildOf>,
    extras: impl Bundle,
) -> EntityCommands<'w> {
    parent.spawn((
        NavMenu {
            is_priority: active,
            is_wrap: true,
            is_locked: locked,
        },
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(Val::Px(10.)),
            padding: UiRect {
                left: Val::Px(10.),
                right: Val::Px(10.),
                top: Val::Px(10.),
                bottom: Val::Auto,
            },
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BorderColor(Color::WHITE),
        extras,
    ))
}

/// Utility that spawns a button for the grid.
///
/// Its the same as spawn_button, except the button has no margin.
pub fn text_widget(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font_size: FontSize,
    text: impl Into<String>,
) {
    parent.spawn((
        Text::new(text),
        TextColor(Color::WHITE),
        TextFont::from_font_size(font_size.into()),
    ));
}

/// Utility that spawns a button.
pub fn menu_button(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    focus: bool,
    disabled: bool,
    mouse_only: bool,
    extras: impl Bundle,
) -> Entity {
    menu_buttoni(
        spawner,
        text,
        (
            Focusable::default()
                .with_priority(focus)
                .with_disabled(disabled)
                .with_mouse_only(mouse_only),
            extras,
        ),
        |_| {},
    )
}

/// Utility that spawns a button.
pub fn menu_buttoni(
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    extras: impl Bundle,
    class: impl FnOnce(&mut Node),
) -> Entity {
    let mut node = Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: Val::Px(200.),
        height: Val::Px(50.),
        margin: UiRect::bottom(Val::Px(10.)),
        border: UiRect::all(Val::Px(1.)),
        ..default()
    };
    class(&mut node);

    spawner
        .spawn((
            StyledButton,
            node,
            BackgroundColor(BUTTON_BG_DEFAULT.into()),
            BorderColor(BUTTON_BG_DEFAULT.into()),
            Button,
            extras,
        ))
        .with_children(|p| {
            text_widget(p, FontSize::Small, text);
        })
        .id()
}

/// Utility that spawns a title.
pub fn menu_title(parent: &mut RelatedSpawnerCommands<ChildOf>, text: impl Into<String>) {
    text_widget(parent, FontSize::Large, text);
}

/// System that updates button colors when they change
#[allow(clippy::type_complexity)]
fn button_style(
    mut query: Query<
        (&Focusable, &mut BackgroundColor, &mut BorderColor),
        (Changed<Focusable>, With<StyledButton>),
    >,
) {
    for (focusable, mut bg, mut border) in query.iter_mut() {
        // background color
        *bg = match focusable.state() {
            FocusState::Focus => BUTTON_BG_ACTIVE,
            FocusState::FocusPress => BUTTON_BG_PRESS,
            FocusState::Disabled => BUTTON_BG_DISABLED,
            _ => BUTTON_BG_DEFAULT,
        }
        .into();

        // border color
        *border = if focusable.is_hovered() {
            Color::WHITE.into()
        } else {
            BUTTON_BG_DEFAULT.into()
        };
    }
}
