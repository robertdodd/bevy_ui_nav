use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

use example_utils::*;

mod example_utils;

fn main() {
    let mut app = App::new();

    // add plugins
    app.add_plugins((DefaultPlugins, BevyUiNavPlugin, ExampleUtilsPlugin));
    // initialize state and scoped entities
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
        handle_click_events
            .after(UiNavSet)
            .run_if(on_event::<UiNavClickEvent>),
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

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    root_full_screen_centered(&mut commands, (), |p| {
        spawn_menu(true, false, p, ()).with_children(|p| {
            menu_button(p, "Show Modal", true, false, false, ButtonAction::ShowModal);
            // add a spacer so we can test that clicking the underlying buttons doesn't work
            p.spawn(Node {
                height: Px(150.),
                ..default()
            });
            menu_button(p, "Quit", false, false, false, ButtonAction::Quit);
        });
    });
}

fn spawn_modal(mut commands: Commands) {
    // Spawn a semi-transparent full-screen overlay layout for the modal
    commands
        .spawn((
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
            // FocusPolicy::Block,
            // NOTE: You may need to add a `ZIndex` component if you have multiple root nodes and find they overlap
        ))
        .with_children(|p| {
            // spawn the modal body
            p.spawn((
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
            ))
            .with_children(|p| {
                // Spawn the modal title
                p.spawn(Text::new("Modal Title"));
                // Spawn the modal footer row containing two buttons
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Percent(100.),
                    ..default()
                })
                .with_children(|p| {
                    menu_button(p, "Cancel", false, false, false, ButtonAction::HideModal);
                    menu_button(p, "Save", true, false, false, ButtonAction::HideModal);
                });
            });
        });
}

fn handle_click_events(
    mut events: EventReader<UiNavClickEvent>,
    query: Query<&ButtonAction, With<Focusable>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    for event in events.read() {
        if let Ok(button_action) = query.get(event.0) {
            println!("ClickEvent: {:?}", button_action);
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
