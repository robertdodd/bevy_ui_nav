use bevy::{
    app::AppExit,
    color::palettes::css,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin, ExampleUtilsPlugin))
        .init_resource::<GameData>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                text_control_style,
                debug_cancel_events.run_if(on_event::<UiNavCancelEvent>()),
                (handle_button_click_events, handle_text_control_click_events)
                    .run_if(on_event::<UiNavClickEvent>()),
                listen_received_character_events.run_if(on_event::<KeyboardInput>()),
                update_text_on_change,
                update_title_label.run_if(resource_changed::<GameData>),
            )
                .after(UiNavSet),
        )
        .run();
}

const TEXT_CONTROL_BG_DEFAULT: Srgba = css::DARK_GRAY;
const TEXT_CONTROL_BG_ACTIVE: Srgba = css::WHITE;

const TEXT_CONTROL_BORDER_DEFAULT: Srgba = css::WHITE;
const TEXT_CONTROL_BORDER_ACTIVE: Srgba = css::RED;
const TEXT_CONTROL_BORDER_HOVER: Srgba = css::YELLOW;

#[derive(Resource, Debug, Default)]
struct GameData {
    name: String,
}

#[derive(Component)]
struct TitleLabel;

#[derive(Component, Default, Debug)]
struct TextControl(String);

#[derive(Component, Default, Debug, PartialEq, Eq)]
enum TextControlStatus {
    #[default]
    InActive,
    Active,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    Reset,
    Quit,
}

/// Utility that spawns a text control.
fn spawn_text_control(
    parent: &mut ChildBuilder,
    text: impl Into<String>,
    focus: bool,
    extras: impl Bundle,
) -> Entity {
    parent
        .spawn((
            if focus {
                Focusable::prioritized()
            } else {
                Focusable::default()
            },
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Px(200.),
                    height: Val::Px(50.),
                    margin: UiRect::bottom(Val::Px(10.)),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                background_color: TEXT_CONTROL_BG_DEFAULT.into(),
                border_color: TEXT_CONTROL_BORDER_DEFAULT.into(),
                ..default()
            },
            Interaction::default(),
            TextControl::default(),
            TextControlStatus::InActive,
            extras,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    color: Color::BLACK,
                    font_size: 20.,
                    ..default()
                },
            ));
        })
        .id()
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, (), |p| {
            // title
            p.spawn((
                TextBundle::from_sections(["Name: ".into(), "".into()]),
                TitleLabel,
            ));

            // text control
            spawn_text_control(p, "", true, ());

            // Save and cancel buttons
            menu_button(p, "Reset", false, false, false, ButtonAction::Reset);
            menu_button(p, "Quit", false, false, false, ButtonAction::Quit);
        });
    });
}

/// System that updates the style of text controls when their focus state changes
#[allow(clippy::type_complexity)]
fn text_control_style(
    mut query: Query<
        (
            &Focusable,
            &mut BackgroundColor,
            &mut BorderColor,
            &TextControlStatus,
        ),
        Or<(Changed<Focusable>, Changed<TextControlStatus>)>,
    >,
) {
    for (focusable, mut bg, mut border, status) in query.iter_mut() {
        // Update background color
        *bg = if *status == TextControlStatus::Active {
            TEXT_CONTROL_BG_ACTIVE
        } else {
            TEXT_CONTROL_BG_DEFAULT
        }
        .into();

        // Update border color
        *border = if *status == TextControlStatus::Active {
            TEXT_CONTROL_BORDER_ACTIVE
        } else if focusable.is_hovered() || focusable.state().active() {
            TEXT_CONTROL_BORDER_HOVER
        } else {
            TEXT_CONTROL_BORDER_DEFAULT
        }
        .into();
    }
}

fn handle_button_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&ButtonAction, (With<Focusable>, With<Button>)>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut game_data: ResMut<GameData>,
    mut text_control_query: Query<&mut TextControl>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.0) {
            println!("ClickEvent: {:?}", button_action);
            match *button_action {
                ButtonAction::Quit => {
                    app_exit_writer.send(AppExit::Success);
                }
                ButtonAction::Reset => {
                    game_data.name = "".to_string();
                    for mut text_control in text_control_query.iter_mut() {
                        text_control.0 = "".to_string();
                    }
                }
            };
        }
    }
}

/// System that handles click events on a text control
fn handle_text_control_click_events(
    mut events: EventReader<UiNavClickEvent>,
    mut query: Query<&mut TextControlStatus>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for event in events.read() {
        if let Ok(mut status) = query.get_mut(event.0) {
            match *status {
                TextControlStatus::InActive => {
                    *status = TextControlStatus::Active;
                    nav_request_writer.send(NavRequest::Lock);
                }
                TextControlStatus::Active => {
                    *status = TextControlStatus::InActive;
                    nav_request_writer.send(NavRequest::Unlock);
                }
            }
        }
    }
}

/// System that updates the label value when `GameData::name` changes
fn update_title_label(game_data: Res<GameData>, mut query: Query<&mut Text, With<TitleLabel>>) {
    for mut text in query.iter_mut() {
        text.sections[1].value.clone_from(&game_data.name);
    }
}

/// System that listens for characer key presses in the text control
fn listen_received_character_events(
    mut events: EventReader<KeyboardInput>,
    mut query: Query<(&mut TextControl, &mut TextControlStatus, &Children)>,
    mut text_query: Query<&mut Text>,
    mut nav_request_writer: EventWriter<NavRequest>,
    mut game_data: ResMut<GameData>,
) {
    for event in events.read() {
        if event.state == ButtonState::Pressed {
            for (mut text_control, mut status, children) in query.iter_mut() {
                if *status != TextControlStatus::Active {
                    continue;
                }

                // track whether we handled the key press
                let is_changed = match &event.logical_key {
                    Key::Character(char) => {
                        text_control.0.push_str(char);
                        true
                    }
                    Key::Backspace => {
                        text_control.0.pop();
                        true
                    }
                    Key::Enter => {
                        *status = TextControlStatus::InActive;
                        nav_request_writer.send(NavRequest::Unlock);
                        game_data.name.clone_from(&text_control.0);
                        true
                    }
                    Key::Space => {
                        text_control.0.push(' ');
                        true
                    }
                    Key::Escape => {
                        *status = TextControlStatus::InActive;
                        nav_request_writer.send(NavRequest::Unlock);
                        text_control.0.clone_from(&game_data.name);
                        true
                    }
                    _ => false,
                };

                // Update the text content instantly
                if is_changed {
                    for &child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.sections[0].value.clone_from(&text_control.0);
                        }
                    }
                }
            }
        }
    }
}

/// System that updates the content of a text control when it is changed.
/// NOTE: This is not important in this example, as we update the value instantly when we receive a character keypress.
/// However, this system would allow the text to update if you manually changed `TextControl::value`.
fn update_text_on_change(
    query: Query<(&TextControl, &Children), Changed<TextControl>>,
    mut text_query: Query<&mut Text>,
) {
    for (text_control, children) in query.iter() {
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.sections[0].value.clone_from(&text_control.0);
            }
        }
    }
}

/// System that prints [`UiNavCancelEvent`] events to console.
fn debug_cancel_events(mut events: EventReader<UiNavCancelEvent>) {
    for event in events.read() {
        println!("{event:?}");
    }
}
