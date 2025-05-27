use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .init_resource::<GameData>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                text_control_style,
                debug_cancel_events.run_if(on_event::<UiNavCancelEvent>),
                (handle_button_click_events, handle_text_control_click_events)
                    .run_if(on_event::<FocusablePressed>),
                update_text_on_change,
                update_title_label.run_if(resource_changed::<GameData>),
                focusable_colors,
            )
                .after(UiNavSet),
        )
        // listen for keyboard input BEFORE `UiNavSet` so there is no overlap in handling key presses.
        // for example: <Enter> locks navigation, but also detected by input system which unlocks it immediately.
        .add_systems(
            Update,
            listen_received_character_events
                .run_if(on_event::<KeyboardInput>)
                .before(UiNavSet),
        )
        .run();
}

const TEXT_CONTROL_FONT_COLOR: Color = Color::WHITE;

const TEXT_CONTROL_BG_DEFAULT: Srgba = tailwind::ZINC_700;
const TEXT_CONTROL_BG_ACTIVE: Srgba = tailwind::ZINC_800;

const TEXT_CONTROL_BORDER_DEFAULT: Srgba = Srgba::new(1., 1., 1., 0.25);
const TEXT_CONTROL_BORDER_ACTIVE: Srgba = tailwind::YELLOW_300;
const TEXT_CONTROL_BORDER_HOVER: Srgba = tailwind::YELLOW_300;

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

fn ui_root() -> impl Bundle + use<> {
    (
        Name::new("UI Root"),
        Node {
            width: Percent(100.),
            height: Percent(100.),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
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

/// Utility that spawns a text control.
fn text_input() -> impl Bundle + use<> {
    (
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Percent(100.),
            height: Px(50.),
            margin: UiRect::bottom(Px(10.)),
            border: UiRect::all(Px(4.)),
            ..default()
        },
        BackgroundColor(TEXT_CONTROL_BG_DEFAULT.into()),
        BorderColor(TEXT_CONTROL_BORDER_DEFAULT.into()),
        Interaction::default(),
        TextControl::default(),
        TextControlStatus::InActive,
        children![(
            Text::default(),
            TextColor(TEXT_CONTROL_FONT_COLOR),
            TextFont::from_font_size(20.),
        )],
    )
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        ui_root(),
        children![(
            menu(true),
            children![
                (
                    Text::new("Name: "),
                    children![(TitleLabel, TextSpan::default())],
                ),
                (text_input(), Focusable::prioritized()),
                (button("Reset"), Focusable::default(), ButtonAction::Reset),
                (button("Quit"), Focusable::default(), ButtonAction::Quit),
            ]
        )],
    ));
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
        *bg = if *status == TextControlStatus::Active {
            TEXT_CONTROL_BG_ACTIVE
        } else {
            TEXT_CONTROL_BG_DEFAULT
        }
        .into();
        *border = match (status, focusable.state()) {
            (TextControlStatus::Active, _) => TEXT_CONTROL_BORDER_ACTIVE,
            (TextControlStatus::InActive, FocusState::Focused) => TEXT_CONTROL_BORDER_HOVER,
            _ => TEXT_CONTROL_BORDER_DEFAULT,
        }
        .into();
    }
}

fn handle_button_click_events(
    mut events: EventReader<FocusablePressed>,
    query: Query<&ButtonAction, (With<Focusable>, With<Button>)>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut game_data: ResMut<GameData>,
    mut text_control_query: Query<&mut TextControl>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.entity) {
            println!("ClickEvent: {:?}", button_action);
            match *button_action {
                ButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
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
    mut events: EventReader<FocusablePressed>,
    mut query: Query<&mut TextControlStatus>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for event in events.read() {
        if let Ok(mut status) = query.get_mut(event.entity) {
            match *status {
                TextControlStatus::InActive => {
                    *status = TextControlStatus::Active;
                    nav_request_writer.write(NavRequest::Lock);
                }
                TextControlStatus::Active => {
                    *status = TextControlStatus::InActive;
                    nav_request_writer.write(NavRequest::Unlock);
                }
            }
        }
    }
}

/// System that updates the label value when `GameData::name` changes
fn update_title_label(game_data: Res<GameData>, mut query: Query<&mut TextSpan, With<TitleLabel>>) {
    for mut text in query.iter_mut() {
        text.0.clone_from(&game_data.name);
    }
}

/// System that listens for characer key presses in the text control
fn listen_received_character_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<KeyboardInput>,
    mut query: Query<(&mut TextControl, &mut TextControlStatus, &Children)>,
    mut text_query: Query<&mut Text>,
    mut nav_request_writer: EventWriter<NavRequest>,
    mut game_data: ResMut<GameData>,
    button_query: Query<(Entity, &ButtonAction)>,
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
                        if !keys.just_pressed(KeyCode::Enter) {
                            false
                        } else {
                            *status = TextControlStatus::InActive;
                            game_data.name.clone_from(&text_control.0);
                            // unlock navigation
                            nav_request_writer.write(NavRequest::Unlock);
                            // set focus on the submit button
                            if let Some(submit_button) = button_query
                                .iter()
                                .find(|(_, action)| matches!(action, ButtonAction::Reset))
                                .map(|(e, _)| e)
                            {
                                nav_request_writer.write(NavRequest::SetFocus(submit_button));
                            }
                            true
                        }
                    }
                    Key::Space => {
                        text_control.0.push(' ');
                        true
                    }
                    Key::Escape => {
                        *status = TextControlStatus::InActive;
                        nav_request_writer.write(NavRequest::Unlock);
                        text_control.0.clone_from(&game_data.name);
                        true
                    }
                    _ => false,
                };

                // Update the text content instantly
                if is_changed {
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.0.clone_from(&text_control.0);
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
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0.clone_from(&text_control.0);
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

fn focusable_colors(
    mut query: Query<(&Focusable, &mut BorderColor), (Changed<Focusable>, With<Button>)>,
) {
    for (focusable, mut border_color) in query.iter_mut() {
        border_color.0 = match focusable.state() {
            FocusState::None => Color::WHITE,
            FocusState::Focused => tailwind::YELLOW_300.into(),
            FocusState::Disabled => Color::WHITE.with_alpha(0.25),
        };
    }
}
