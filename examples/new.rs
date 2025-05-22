use std::time::Duration;

use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .init_state::<Screen>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_click_events.run_if(on_event::<UiNavClickEvent>),
                update_screen_labels.run_if(state_changed::<Screen>),
                toggle_click_events.run_if(on_event::<PressableClick>),
                handle_pressable_click_events.run_if(on_event::<PressableClick>),
                pressable_system,
                focusable_system,
                debug_nav_requests,
                update_focusable_animations,
            )
                .after(UiNavSet),
        )
        .run();
}

const BUTTON_BORDER: f32 = 8.;
const BUTTON_BG_NORMAL: Srgba = tailwind::RED_500;
const BUTTON_BG_HOVERED: Srgba = tailwind::RED_700;
const BUTTON_BG_PRESSED: Srgba = tailwind::RED_950;
const BUTTON_BORDER_COLOR_NORMAL: Srgba = tailwind::RED_950;
const BUTTON_BORDER_COLOR_HOVERED: Srgba = tailwind::RED_950;
const BUTTON_BORDER_COLOR_PRESSED: Srgba = Srgba::NONE;
const BUTTON_BORDER_NORMAL: UiRect = UiRect::bottom(Val::Px(BUTTON_BORDER));
const BUTTON_BORDER_HOVERED: UiRect = UiRect::bottom(Val::Px(BUTTON_BORDER));
const BUTTON_BORDER_PRESSED: UiRect = UiRect::top(Val::Px(BUTTON_BORDER));

const FOCUSABLE_OUTLINE_START: f32 = 10.;
const FOCUSABLE_OUTLINE_END: f32 = 2.;

#[derive(Component)]
pub struct ScreenLabel;

#[derive(States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Graphics,
    Sound,
    Controls,
}

#[derive(Component)]
struct MainMenu;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum MenuButton {
    Graphics,
    Sound,
    Controls,
    Exit,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ToggleButton {
    Prev,
    Next,
}

#[derive(Component, Debug)]
struct ToggleValue(i32);

#[derive(Component, Debug)]
struct FocusableAnimation(Timer);

impl Default for FocusableAnimation {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs_f32(0.25), TimerMode::Once))
    }
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
            ..default()
        },
        BackgroundColor(tailwind::SLATE_800.into()),
    )
}

fn panel() -> impl Bundle + use<> {
    (
        Name::new("Panel"),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Px(10.),
            padding: UiRect::all(Px(20.)),
            ..default()
        },
        BackgroundColor(tailwind::SLATE_600.into()),
        BorderRadius::all(Px(20.)),
    )
}

fn panel_body() -> impl Bundle + use<> {
    (
        Name::new("Panel Body"),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Px(10.),
            padding: UiRect::all(Px(20.)),
            ..default()
        },
        BackgroundColor(tailwind::SLATE_700.into()),
        BorderRadius::all(Px(20.)),
    )
}

fn panel_header() -> impl Bundle + use<> {
    (
        Name::new("Panel header"),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Px(10.),
            ..default()
        },
        Focusable::default().with_action(FocusableAction::PressXY),
    )
}

fn button_focusable() -> impl Bundle + use<> {
    (
        Name::new("Button - Focusable"),
        Node::default(),
        Focusable::default(),
        BorderRadius::MAX,
    )
}

fn toggle_control() -> impl Bundle + use<> {
    (
        Name::new("Toggle Control - Focusable"),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Px(10.),
            width: Percent(100.),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        Focusable::default().with_action(FocusableAction::PressXY),
    )
}

fn toggle_button(text: &str, pressable: Pressable) -> impl Bundle + use<> {
    (
        Name::new("Button - Pressable"),
        Button,
        pressable,
        Node {
            padding: UiRect::all(Px(10.)),
            min_width: Px(60.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::RED_500.into()),
        BorderColor(tailwind::RED_950.into()),
        BorderRadius::MAX,
        children![Text::new(text)],
    )
}

fn button(text: &str) -> impl Bundle + use<> {
    (
        Name::new("Button - Pressable"),
        Button,
        Pressable::new_press(),
        Node {
            padding: UiRect::all(Px(20.)),
            min_width: Px(200.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::RED_500.into()),
        BorderColor(tailwind::RED_950.into()),
        BorderRadius::MAX,
        children![Text::new(text)],
    )
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        ui_root(),
        NavMenu::default().prioritized(),
        children![(
            panel(),
            children![
                (
                    panel_header(),
                    children![
                        (button("Graphics"), MenuButton::Graphics),
                        (button("Sound"), MenuButton::Sound),
                        (button("Controls"), MenuButton::Controls),
                    ]
                ),
                (
                    panel_body(),
                    children![
                        (Text::default(), ScreenLabel),
                        (
                            toggle_control(),
                            children![
                                (
                                    toggle_button("<", Pressable::new_left()),
                                    ToggleButton::Prev
                                ),
                                (Name::new("Toggle Value"), Text::new("0"), ToggleValue(0)),
                                (
                                    toggle_button(">", Pressable::new_right()),
                                    ToggleButton::Next
                                )
                            ]
                        ),
                        (
                            Node::default(),
                            children![(
                                button_focusable(),
                                children![(button("Exit"), MenuButton::Exit)]
                            )]
                        )
                    ]
                )
            ]
        )],
    ));
}

fn handle_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&MenuButton>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    for button in events.nav_iter().in_query(&query) {
        println!("ClickEvent: {:?}", button);
        match *button {
            MenuButton::Graphics => {
                next_screen.set(Screen::Graphics);
            }
            MenuButton::Sound => {
                next_screen.set(Screen::Sound);
            }
            MenuButton::Controls => {
                next_screen.set(Screen::Controls);
            }
            MenuButton::Exit => {
                app_exit_writer.write(AppExit::Success);
            }
        };
    }
}

fn handle_pressable_click_events(
    mut events: EventReader<PressableClick>,
    query: Query<&MenuButton>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.0) {
            println!("ClickEvent: {:?}", button);
            match *button {
                MenuButton::Graphics => {
                    next_screen.set(Screen::Graphics);
                }
                MenuButton::Sound => {
                    next_screen.set(Screen::Sound);
                }
                MenuButton::Controls => {
                    next_screen.set(Screen::Controls);
                }
                MenuButton::Exit => {
                    app_exit_writer.write(AppExit::Success);
                }
            };
        }
    }
}

fn toggle_click_events(
    mut events: EventReader<PressableClick>,
    query: Query<&ToggleButton>,
    mut toggle_query: Query<(&mut ToggleValue, &mut Text)>,
) {
    for event in events.read() {
        if let Ok(button) = query.get(event.0) {
            println!("ClickEvent: {:?}", button);
            for (mut value, mut text) in toggle_query.iter_mut() {
                match *button {
                    ToggleButton::Prev => value.0 -= 1,
                    ToggleButton::Next => value.0 += 1,
                }
                text.0 = format!("{}", value.0);
            }
        }
    }
}

fn update_screen_labels(
    mut query: Query<(Option<&mut Text>, Option<&mut TextSpan>), With<ScreenLabel>>,
    screen_state: Res<State<Screen>>,
) {
    for (mut text, mut text_span) in query.iter_mut() {
        set_text_value(
            text.as_deref_mut(),
            text_span.as_deref_mut(),
            format!("{:?}", screen_state.get()),
        );
    }
}

pub fn set_text_value(
    text: Option<&mut Text>,
    text_span: Option<&mut TextSpan>,
    value: impl Into<String>,
) {
    if let Some(text) = text {
        text.0 = value.into();
    } else if let Some(text_span) = text_span {
        text_span.0 = value.into();
    }
}

/// System that updates button colors
fn pressable_system(
    mut interaction_query: Query<
        (
            &mut Node,
            &Pressable,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Pressable>,
    >,
) {
    for (mut node, pressable, mut bg, mut border_color) in &mut interaction_query {
        match pressable.state() {
            PressableState::Pressed => {
                node.border = BUTTON_BORDER_PRESSED;
                *bg = BUTTON_BG_PRESSED.into();
                border_color.0 = BUTTON_BORDER_COLOR_PRESSED.into();
            }
            PressableState::Hovered => {
                node.border = BUTTON_BORDER_HOVERED;
                *bg = BUTTON_BG_HOVERED.into();
                border_color.0 = BUTTON_BORDER_COLOR_HOVERED.into();
            }
            PressableState::None => {
                node.border = BUTTON_BORDER_NORMAL;
                *bg = BUTTON_BG_NORMAL.into();
                border_color.0 = BUTTON_BORDER_COLOR_NORMAL.into();
            }
        }
    }
}

fn focusable_system(
    mut commands: Commands,
    query: Query<(Entity, &Focusable, Has<Outline>, Has<FocusableAnimation>), Changed<Focusable>>,
) {
    for (entity, focusable, has_outline, has_animation) in query.iter() {
        let is_focused = matches!(
            focusable.state(),
            FocusState::Focus | FocusState::FocusPress
        );
        // insert an outline animation if focused and we dont have one, or whenever pressed
        if is_focused {
            if !has_outline || focusable.state() == FocusState::FocusPress {
                commands.entity(entity).insert((
                    Outline::new(Px(FOCUSABLE_OUTLINE_START), Px(2.), Color::WHITE),
                    FocusableAnimation::default(),
                ));
            }
        } else {
            if has_outline {
                commands.entity(entity).remove::<Outline>();
            }
            if has_animation {
                commands.entity(entity).remove::<FocusableAnimation>();
            }
        }
    }
}

fn debug_nav_requests(mut events: EventReader<NavRequest>) {
    for event in events.read() {
        info!("NavRequest::{:?}", event);
    }
}

fn update_focusable_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FocusableAnimation, &mut Outline), With<Focusable>>,
) {
    for (entity, mut animation, mut outline) in query.iter_mut() {
        animation.0.tick(time.delta());
        outline.width =
            Px(FOCUSABLE_OUTLINE_START.lerp(FOCUSABLE_OUTLINE_END, animation.0.fraction()));
        if animation.0.just_finished() {
            commands.entity(entity).remove::<FocusableAnimation>();
        }
    }
}
