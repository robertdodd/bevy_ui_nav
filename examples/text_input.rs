use bevy::{app::AppExit, input::keyboard::KeyboardInput, prelude::*};
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
                handle_button_click_events.run_if(on_event::<UiNavClickEvent>()),
                handle_text_control_click_events.run_if(on_event::<UiNavClickEvent>()),
                (
                    handle_keyboard_input_events.run_if(on_event::<KeyboardInput>()),
                    listen_received_character_events,
                    handle_text_control_active_input,
                )
                    .chain(),
                update_text_on_change,
                update_title_label.run_if(resource_changed::<GameData>),
            )
                .after(UiNavSet),
        )
        .run();
}

const TEXT_CONTROL_BG_DEFAULT: Color = Color::DARK_GRAY;
const TEXT_CONTROL_BG_ACTIVE: Color = Color::WHITE;

const TEXT_CONTROL_BORDER_DEFAULT: Color = Color::WHITE;
const TEXT_CONTROL_BORDER_ACTIVE: Color = Color::RED;
const TEXT_CONTROL_BORDER_HOVER: Color = Color::YELLOW;

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

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_menu(true, false, p, (), |p| {
                // title
                p.spawn((
                    TextBundle::from_sections(["Name: ".into(), "".into()]),
                    TitleLabel,
                ));

                // text control
                spawn_text_control(p, "", true, ());

                // Save and cancel buttons
                spawn_button(p, "Reset", false, false, ButtonAction::Reset);
                spawn_button(p, "Quit", false, false, ButtonAction::Quit);
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
                    app_exit_writer.send(AppExit);
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

/// System that handles key presses while a text control has focus
fn handle_text_control_active_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut TextControlStatus, &mut TextControl)>,
    mut lock_writer: EventWriter<UiNavLockEvent>,
    mut game_data: ResMut<GameData>,
) {
    for (mut status, mut text_control) in query.iter_mut() {
        if *status == TextControlStatus::InActive {
            continue;
        }

        if keys.just_pressed(KeyCode::Enter) {
            *status = TextControlStatus::InActive;
            lock_writer.send(UiNavLockEvent::Unlock);
            game_data.name = text_control.0.clone();
        } else if keys.just_pressed(KeyCode::Escape) {
            *status = TextControlStatus::InActive;
            lock_writer.send(UiNavLockEvent::Unlock);
            text_control.0 = game_data.name.clone();
        }
    }
}

/// System that handles key presses while a text control has focus
fn handle_keyboard_input_events(
    mut events: EventReader<KeyboardInput>,
    mut query: Query<(&TextControlStatus, &mut TextControl, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for event in events.read() {
        for (status, mut text_control, children) in query.iter_mut() {
            if *status == TextControlStatus::InActive {
                continue;
            }

            if let KeyCode::Backspace = event.key_code {
                text_control.0.pop();
                text_control.set_changed();
                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.sections[0].value = text_control.0.clone();
                    }
                }
            }
        }
    }
}

/// System that handles click events on a text control
fn handle_text_control_click_events(
    mut events: EventReader<UiNavClickEvent>,
    mut query: Query<&mut TextControlStatus>,
    mut lock_writer: EventWriter<UiNavLockEvent>,
) {
    for event in events.read() {
        if let Ok(mut status) = query.get_mut(event.0) {
            match *status {
                TextControlStatus::InActive => {
                    *status = TextControlStatus::Active;
                    lock_writer.send(UiNavLockEvent::Lock);
                }
                TextControlStatus::Active => {
                    *status = TextControlStatus::InActive;
                    lock_writer.send(UiNavLockEvent::Unlock);
                }
            }
        }
    }
}

/// System that updates the label value when `GameData::name` changes
fn update_title_label(game_data: Res<GameData>, mut query: Query<&mut Text, With<TitleLabel>>) {
    for mut text in query.iter_mut() {
        text.sections[1].value = game_data.name.clone();
    }
}

/// System that listens for characer key presses in the text control
fn listen_received_character_events(
    mut events: EventReader<ReceivedCharacter>,
    mut query: Query<(&mut TextControl, &TextControlStatus, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for event in events.read().filter(|event| event.char != "\r") {
        for (mut text_control, status, children) in query.iter_mut() {
            if *status != TextControlStatus::Active {
                continue;
            }

            text_control.0.push_str(&event.char);

            // Update the text content instantly
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.sections[0].value = text_control.0.clone();
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
                text.sections[0].value = text_control.0.clone();
            }
        }
    }
}
