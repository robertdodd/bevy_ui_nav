//! This example demonstrates navigating between multiple `NavMenus` on the screen.

use bevy::{
    app::AppExit,
    color::palettes::tailwind,
    prelude::{Val::*, *},
};
use bevy_ui_nav::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_click_events.run_if(on_event::<FocusablePressed>),
                handle_cancel_events.run_if(on_event::<UiNavCancelEvent>),
                focusable_colors,
            )
                .after(UiNavSet),
        )
        .run();
}

#[derive(Component, PartialEq, Eq, Clone, Debug)]
enum Menu {
    Main,
    Graphics,
    Sound,
}

#[derive(Component, PartialEq, Eq, Clone, Debug)]
enum ButtonAction {
    Menu(Menu),
    Debug(String),
    Quit,
}

fn ui_root() -> impl Bundle + use<> {
    (
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        ui_root(),
        children![
            (
                menu(true),
                Menu::Main,
                children![
                    Text::new("Main Menu"),
                    (
                        Text::new("Navigate between menus. It should return focus to the last focused item in each menu."),
                        TextFont::from_font_size(14.),
                    ),
                    (
                        button("Graphics"),
                        ButtonAction::Menu(Menu::Graphics),
                        Focusable::prioritized()
                    ),
                    (
                        button("Sound"),
                        ButtonAction::Menu(Menu::Sound),
                        Focusable::default()
                    ),
                    (button("Quit"), ButtonAction::Quit, Focusable::default()),
                ]
            ),
            (
                menu(false),
                Menu::Graphics,
                children![
                    Text::new("Graphics Menu"),
                    (
                        button("Option 1"),
                        ButtonAction::Debug("Graphics Option 1".to_string()),
                        Focusable::prioritized()
                    ),
                    (
                        button("Option 2"),
                        ButtonAction::Debug("Graphics Option 2".to_string()),
                        Focusable::default()
                    ),
                    (
                        button("Cancel"),
                        ButtonAction::Menu(Menu::Main),
                        Focusable::default()
                    ),
                ]
            ),
            (
                menu(false),
                Menu::Sound,
                children![
                    Text::new("Sound Menu"),
                    (
                        button("Option 1"),
                        ButtonAction::Debug("Sound Option 1".to_string()),
                        Focusable::prioritized()
                    ),
                    (
                        button("Option 2"),
                        ButtonAction::Debug("Sound Option 2".to_string()),
                        Focusable::default()
                    ),
                    (
                        button("Cancel"),
                        ButtonAction::Menu(Menu::Main),
                        Focusable::default()
                    ),
                ]
            ),
        ],
    ));
}

fn handle_click_events(
    mut events: EventReader<FocusablePressed>,
    query: Query<&ButtonAction, With<Focusable>>,
    menu_query: Query<(Entity, &Menu)>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for button_action in events.nav_iter().in_query(&query) {
        println!("ClickEvent: {:?}", button_action);
        match button_action {
            ButtonAction::Menu(target) => {
                let menu = menu_query
                    .iter()
                    .find(|(_, menu)| **menu == *target)
                    .map(|(e, _)| e);
                if let Some(menu) = menu {
                    nav_request_writer.write(NavRequest::SetFocus(menu));
                }
            }
            ButtonAction::Debug(debug_text) => println!("  clicked on \"{debug_text}\""),
            ButtonAction::Quit => {
                app_exit_writer.write(AppExit::Success);
            }
        };
    }
}

fn handle_cancel_events(
    mut events: EventReader<UiNavCancelEvent>,
    menu_query: Query<(Entity, &Menu)>,
    mut nav_request_writer: EventWriter<NavRequest>,
) {
    for event in events.read() {
        println!("CancelEvent: {:?}", event);

        // find and set focus to the main menu, unless the event was sent from the main menu.
        let main_menu = menu_query
            .iter()
            .filter(|(e, _)| *e != event.0)
            .find(|(_, menu)| **menu == Menu::Main)
            .map(|(e, _)| e);
        if let Some(main_menu) = main_menu {
            nav_request_writer.write(NavRequest::SetFocus(main_menu));
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
