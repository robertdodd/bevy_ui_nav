use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
    ui::FocusPolicy,
};
use bevy_ui_nav::prelude::*;

fn main() {
    let mut app = App::new();

    // add plugins
    app.add_plugins((DefaultPlugins, BevyUiNavPlugin));
    // initialize the state we use for showing the modal
    app.init_state::<ScreenState>();
    // enable scoped entities. This allows the modal to be automatically de-spawned when we leave `ScreenState::Modal`
    app.enable_state_scoped_entities::<ScreenState>();

    // spawn the root menu
    app.add_systems(Startup, startup);
    // Spawn the modal when we enter `ScreenState::Modal`
    app.add_systems(OnEnter(ScreenState::Modal), spawn_modal);
    // Add click handler system
    app.add_systems(
        Update,
        (
            handle_click_events
                .after(UiNavSet)
                .run_if(on_event::<FocusablePressed>),
            focusable_colors,
            handle_interactions,
        ),
    );

    app.run();
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum ScreenState {
    #[default]
    Main,
    Modal,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
enum ButtonAction {
    ShowModal,
    HideModal,
    Quit,
}

fn button(text: &str) -> impl Bundle + use<> {
    (
        Name::new("Button"),
        Focusable::default(),
        Button,
        Node {
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

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Node {
            width: Percent(100.),
            height: Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                // Spacer so we can see the buttons while modal is visible
                row_gap: Px(50.),
                ..default()
            },
            NavMenu::default(),
            children![
                (button("Show Modal 1"), ButtonAction::ShowModal),
                (button("Show Modal 2"), ButtonAction::ShowModal),
                (button("Quit"), ButtonAction::Quit),
            ]
        )],
    ));
}

fn spawn_modal(mut commands: Commands) {
    // Spawn a semi-transparent full-screen overlay layout for the modal
    commands.spawn((
        Name::new("Modal Layout"),
        Node {
            width: Percent(100.),
            height: Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::ZINC_800.with_alpha(0.25).into()),
        // IMPORTANT: The following are important for modal-like behavior:
        // - `prioritized` will ause the menu to take focus when spawned.
        // - `locked` will prevent focus from leaving the menu unless an explicit `NavRequest::SetFocus` event is
        //    sent.
        NavMenu::default().prioritized().locked(),
        // StateScoped allows bevy to automatically de-spawn this entity when we leave `ScreenState::Modal`.
        StateScoped(ScreenState::Modal),
        // NOTE: Adding `FocusPolicy::Block` is a good practice, as it will prevent `Interactions` being triggered
        //  through the modal overlay. However, we omit it so we can test that the plugin does not allow focus on
        //  it's own. You should always add `FocusPolicy::Block` if you are handling `Interactions` manually.
        FocusPolicy::Block,
        // NOTE: You may need to add a `ZIndex` component if you have multiple root nodes and find they overlap
        children![(
            Name::new("Modal"),
            Node {
                min_width: Px(250.),
                flex_direction: FlexDirection::Column,
                row_gap: Px(10.),
                padding: UiRect::all(Px(20.)),
                border: UiRect::all(Px(1.)),
                ..default()
            },
            BackgroundColor(tailwind::ZINC_800.into()),
            BorderColor(Color::WHITE),
            children![
                Text::new("Modal Title"),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Percent(100.),
                        ..default()
                    },
                    children![
                        (button("Cancel"), ButtonAction::HideModal),
                        (button("Save"), ButtonAction::HideModal),
                    ]
                )
            ]
        )],
    ));
}

fn handle_click_events(
    mut events: EventReader<FocusablePressed>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.entity) {
            println!("ClickEvent: {:?}, {:?}", button_action, event.entity);
            match *button_action {
                ButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                ButtonAction::ShowModal => {
                    next_state.set(ScreenState::Modal);
                }
                ButtonAction::HideModal => {
                    next_state.set(ScreenState::Main);
                }
            };
        }
    }
}

fn handle_interactions(
    query: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Pressed {
            println!("ClickEvent: {:?}", button);
            match *button {
                ButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                ButtonAction::ShowModal => {
                    next_state.set(ScreenState::Modal);
                }
                ButtonAction::HideModal => {
                    next_state.set(ScreenState::Main);
                }
            };
        }
    }
}

fn focusable_colors(mut query: Query<(&Focusable, &mut BorderColor), Changed<Focusable>>) {
    for (focusable, mut border_color) in query.iter_mut() {
        border_color.0 = match focusable.state() {
            FocusState::None => Color::WHITE,
            FocusState::Focused => tailwind::YELLOW_300.into(),
            FocusState::Disabled => Color::WHITE.with_alpha(0.25),
        };
    }
}
