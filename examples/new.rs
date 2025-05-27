use std::time::Duration;

use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

mod pressables;
use pressables::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin, pressables::plugin))
        .init_state::<Screen>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                setup_new_focusables,
                handle_click_events.run_if(on_event::<OnPressed>),
                on_toggle_click.run_if(on_event::<OnPressed>),
                update_screen_labels.run_if(state_changed::<Screen>),
                handle_focusable_click_events,
                pressable_system,
                focusable_system,
                debug_nav_requests,
                update_focusable_animations,
            )
                .after(UiNavSet),
        )
        .run();
}

const BUTTON_SHADOW_OFFSET: f32 = 8.;
const BUTTON_SHADOW_SPREAD: f32 = -8.;
const BUTTON_SHADOW_BLUR: f32 = 1.;

const BUTTON_BORDER_RADIUS: f32 = 8.;

const BUTTON_BG_NORMAL: Srgba = tailwind::RED_500;
const BUTTON_BG_HOVERED: Srgba = tailwind::RED_700;
const BUTTON_BG_PRESSED: Srgba = tailwind::RED_900;
// const BUTTON_BORDER_COLOR_NORMAL: Srgba = tailwind::RED_950;
// const BUTTON_BORDER_COLOR_HOVERED: Srgba = tailwind::RED_950;
// const BUTTON_BORDER_COLOR_PRESSED: Srgba = Srgba::NONE;

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
            row_gap: Px(20.),
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
            align_items: AlignItems::Center,
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
            padding: UiRect::all(Px(4.)),
            ..default()
        },
        Focusable::default().with_action(FocusableAction::PressXY),
        BorderRadius::all(Px(10.)),
        FocusableNav::default(),
    )
}

fn button_focusable() -> impl Bundle + use<> {
    (
        Name::new("Button - Focusable"),
        Node::default(),
        Focusable::default(),
        BorderRadius::all(Px(8.)),
    )
}

fn toggle_control() -> impl Bundle + use<> {
    (
        Name::new("Toggle Control - Focusable"),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Px(10.),
            padding: UiRect::all(Px(4.)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        Focusable::default().with_action(FocusableAction::PressXY),
        BorderRadius::all(Px(10.)),
    )
}

fn toggle_label(text: &str) -> impl Bundle + use<> {
    (
        Name::new("Toggle - Label"),
        Node {
            padding: UiRect::all(Px(10.)),
            min_width: Px(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::px(0., 0., 0., BUTTON_SHADOW_OFFSET),
            ..default()
        },
        BackgroundColor(BUTTON_BG_NORMAL.into()),
        BorderRadius::all(Px(BUTTON_BORDER_RADIUS)),
        BoxShadow::new(
            Color::BLACK.with_alpha(0.8),
            Px(0.),
            Px(BUTTON_SHADOW_OFFSET),
            Val::Px(BUTTON_SHADOW_SPREAD),
            Val::Px(BUTTON_SHADOW_BLUR),
        ),
        children![(Text::new(text), ToggleValue(0))],
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
            margin: UiRect::px(0., 0., 0., BUTTON_SHADOW_OFFSET),
            ..default()
        },
        BackgroundColor(BUTTON_BG_NORMAL.into()),
        BorderRadius::all(Px(BUTTON_BORDER_RADIUS)),
        BoxShadow::new(
            Color::BLACK.with_alpha(0.8),
            Px(0.),
            Px(BUTTON_SHADOW_OFFSET),
            Val::Px(BUTTON_SHADOW_SPREAD),
            Val::Px(BUTTON_SHADOW_BLUR),
        ),
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
            margin: UiRect::px(0., 0., 0., BUTTON_SHADOW_OFFSET),
            ..default()
        },
        BackgroundColor(BUTTON_BG_NORMAL.into()),
        BorderRadius::all(Px(BUTTON_BORDER_RADIUS)),
        children![Text::new(text)],
        BoxShadow::new(
            Color::BLACK.with_alpha(0.8),
            Px(0.),
            Px(BUTTON_SHADOW_OFFSET),
            Val::Px(BUTTON_SHADOW_SPREAD),
            Val::Px(BUTTON_SHADOW_BLUR),
        ),
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
                                    ToggleButton::Prev,
                                ),
                                toggle_label("0"),
                                (
                                    toggle_button(">", Pressable::new_right()),
                                    ToggleButton::Next,
                                )
                            ]
                        ),
                    ]
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Px(10.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![
                        (button("Save"), Focusable::default(), MenuButton::Graphics),
                        (button("Cancel"), Focusable::default(), MenuButton::Graphics),
                    ]
                )
            ]
        )],
    ));
}

fn handle_click_events(
    mut events: EventReader<OnPressed>,
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

fn on_toggle_click(
    mut events: EventReader<OnPressed>,
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
            &PressablePressed,
            &Interaction,
            &mut Node,
            &mut BackgroundColor,
            &mut BoxShadow,
        ),
        (
            With<Pressable>,
            Or<(Changed<Interaction>, Changed<PressablePressed>)>,
        ),
    >,
) {
    for (pressed, interaction, mut node, mut bg, mut box_shadow) in &mut interaction_query {
        let is_pressed = pressed.0;
        let pressable_state = match (is_pressed, *interaction) {
            (true, _) | (_, Interaction::Pressed) => PressableState::Pressed,
            (false, Interaction::Hovered) => PressableState::Hovered,
            (false, Interaction::None) => PressableState::None,
        };
        match pressable_state {
            PressableState::Pressed => {
                node.margin = UiRect::px(0., 0., BUTTON_SHADOW_OFFSET, 0.);
                *bg = BUTTON_BG_PRESSED.into();
                for style in &mut box_shadow.0 {
                    style.color = Color::NONE;
                }
                *box_shadow = BoxShadow::new(
                    // fmt: line break
                    Color::NONE,
                    Px(0.),
                    Px(0.),
                    Val::Percent(0.),
                    Val::Px(BUTTON_SHADOW_BLUR),
                );
            }
            PressableState::Hovered => {
                node.margin = UiRect::px(0., 0., 0., BUTTON_SHADOW_OFFSET);
                *bg = BUTTON_BG_HOVERED.into();
                for style in &mut box_shadow.0 {
                    style.color = Color::NONE;
                }
                *box_shadow = BoxShadow::new(
                    Color::BLACK.with_alpha(0.8),
                    Px(0.),
                    Px(BUTTON_SHADOW_OFFSET),
                    Val::Px(BUTTON_SHADOW_SPREAD),
                    Val::Px(BUTTON_SHADOW_BLUR),
                );
            }
            PressableState::None => {
                node.margin = UiRect::px(0., 0., 0., BUTTON_SHADOW_OFFSET);
                *bg = BUTTON_BG_NORMAL.into();
                for style in &mut box_shadow.0 {
                    style.color = Color::NONE;
                }
                *box_shadow = BoxShadow::new(
                    Color::BLACK.with_alpha(0.8),
                    Px(0.),
                    Px(BUTTON_SHADOW_OFFSET),
                    Val::Px(BUTTON_SHADOW_SPREAD),
                    Val::Px(BUTTON_SHADOW_BLUR),
                );
            }
        }
    }
}

fn focusable_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Focusable, &mut Outline, Has<FocusableAnimation>),
        Changed<Focusable>,
    >,
) {
    for (entity, focusable, mut outline, has_animation) in query.iter_mut() {
        let is_focused = matches!(focusable.state(), FocusState::Focused);
        // insert an outline animation if focused and we dont have one, or whenever pressed
        if is_focused {
            if focusable.state() == FocusState::Focused {
                outline.color = Color::WHITE;
                commands
                    .entity(entity)
                    .insert(FocusableAnimation::default());
            }
        } else {
            outline.color = Color::NONE;
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
        let width = FOCUSABLE_OUTLINE_START.lerp(FOCUSABLE_OUTLINE_END, animation.0.fraction());
        outline.width = Px(width);
        if animation.0.just_finished() {
            commands.entity(entity).remove::<FocusableAnimation>();
        }
    }
}

fn handle_focusable_click_events(mut commands: Commands, mut events: EventReader<PressEvent>) {
    for event in events.read() {
        commands
            .entity(event.entity)
            .insert(FocusableAnimation::default());
    }
}

fn setup_new_focusables(
    mut commands: Commands,
    query: Query<(Entity, &Focusable), (Added<Focusable>, Without<Outline>)>,
) {
    for (entity, focusable) in query.iter() {
        commands.entity(entity).insert(Outline::new(
            Px(FOCUSABLE_OUTLINE_END),
            Val::ZERO,
            if focusable.state() == FocusState::Focused {
                Color::WHITE
            } else {
                Color::NONE
            },
        ));
    }
}
