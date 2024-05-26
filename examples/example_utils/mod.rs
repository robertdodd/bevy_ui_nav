#![allow(dead_code)]

use bevy::prelude::*;
use bevy_ui_nav::prelude::*;

pub struct ExampleUtilsPlugin;

impl Plugin for ExampleUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_style.after(UiNavSet));
    }
}

const BUTTON_BG_DEFAULT: Color = Color::DARK_GRAY;
const BUTTON_BG_ACTIVE: Color = Color::GRAY;
const BUTTON_BG_PRESS: Color = Color::BLACK;
const BUTTON_BG_DISABLED: Color = Color::BEIGE;

#[derive(Component)]
struct StyledButton;

/// Utility that spawns a nav menu.
pub fn spawn_menu(
    active: bool,
    locked: bool,
    parent: &mut ChildBuilder,
    extras: impl Bundle,
    children: impl FnOnce(&mut ChildBuilder),
) -> Entity {
    parent
        .spawn((
            NavMenu {
                is_priority: active,
                is_wrap: true,
                is_locked: locked,
            },
            NodeBundle {
                style: Style {
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
                border_color: Color::WHITE.into(),
                ..default()
            },
            extras,
        ))
        .with_children(children)
        .id()
}

/// Utility that spawns a button.
pub fn spawn_button(
    parent: &mut ChildBuilder,
    text: impl Into<String>,
    focus: bool,
    disabled: bool,
    mouse_only: bool,
    extras: impl Bundle,
) -> Entity {
    parent
        .spawn((
            StyledButton,
            Focusable::default()
                .with_priority(focus)
                .with_disabled(disabled)
                .with_mouse_only(mouse_only),
            ButtonBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Px(200.),
                    height: Val::Px(50.),
                    margin: UiRect::bottom(Val::Px(10.)),
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

/// Utility that spawns a title.
pub fn spawn_title(
    text: impl Into<String>,
    extras: impl Bundle,
    parent: &mut ChildBuilder,
) -> Entity {
    parent
        .spawn((
            TextBundle::from_section(
                text,
                TextStyle {
                    color: Color::WHITE,
                    font_size: 40.,
                    ..default()
                },
            ),
            extras,
        ))
        .id()
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
            Color::WHITE
        } else {
            BUTTON_BG_DEFAULT
        }
        .into();
    }
}
