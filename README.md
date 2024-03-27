# Bevy Ui Nav

A [Bevy](https://bevyengine.org/) plugin that enables spatial UI navigation between UI nodes via key-presses and
consolidates click event handling from key presses and mouse button clicks (using bevy `Interaction` internally).

---

## TODO

- add directional press events, e.g. a focusable that increments/decrements when you press left/right, instead of
  navigating
- allow pressing DOWN (hold) then LEFT. This should be treated as 2 key presses, even though directions are pressed the
  entire time. The second keypress should register instantly. This should not happen when using the gamepad stick.

## Features

- No external dependencies, only `bevy`.
- Supports click events on mouse button release
- Automatically handle movement when holding a directional key. Increase speed the longer it's held. Customizable.
- Customize movement speed
- Menu wrapping

## Differences from `bevy-ui-navigation`

The key difference from [bevy-ui-navigation](https://github.com/nicopap/ui-navigation) are:

- No automatic sub-menu navigation. You need to manually send a `NavRequest::SetFocus` event to change focus to a menu.
- No external dependencies (`bevy-ui-navigation` depends on `bevy_mod_picking`). `bevy_ui_nav` uses the following core
  `bevy` types for handling mouse interactions: `Interaction` and `RelativeCursorPosition`
- Automatically handles movement when holding a directional key.

## Usage

Add the plugin to your app:

```rust
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyUiNavPlugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_focus_keypress.before(UiNavSet),
                button_style.after(UiNavSet),
                handle_click_events
                    .after(UiNavSet)
                    .run_if(on_event::<FocusableClickEvent>()),
            ),
        )
        .run();
}
```

Add `Focusable` components when spawning UI nodes:

NOTE: The following components will always be added to new `Focusable` entities:

- `Interaction`
- `RelativeCursorPosition`

```rust
commands
    .spawn((
        // Add focusable here:
        Focusable::default(),
        // Add a custom component for identifying which button was clicked:
        ButtonAction::Quit,
        // Add along with a standard bevy UI node:
        ButtonBundle {
            style: Style {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        },
    ))
    .with_children(|p| {
        p.spawn(TextBundle::from_section(
            "Quit",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        ));
    });
```

Handle click events:

```rust
fn handle_click_events(
    mut events: EventReader<FocusableClickEvent>,
    query: Query<&ButtonAction>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    // NOTE: This is equivalent to the following:
    // for event in events.read() {
    //     if let Ok(button_action) = query.get(event.0) {...}
    // }
    for event in events.nav_iter().activated_in_query(&query) {
        match *button_action {
            ButtonAction::Quit => app_exit_writer.send(AppExit),
            _ => (),
        };
    }
}
```

Handle cancel events:

```rust
fn handle_click_events(
    mut events: EventReader<FocusableClickEvent>,
    query: Query<(), With<MyMenu>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    // NOTE: This is equivalent to the following:
    // for event in events.read() {
    //     if query.contains(event.0) {...}
    // }
    for _ in events.nav_iter().activated_in_query(&query) {
        app_exit_writer.send(AppExit);
    }
}
```

Configure input mapping:

```rust
pub(crate) const DEFAULT_INPUT_MAP: &[InputMapping] = &[
    // Keyboard navigation keys
    InputMapping::Key {
        keycode: KeyCode::Up,
        action: ActionType::Up,
    },
    InputMapping::Key {
        keycode: KeyCode::Down,
        action: ActionType::Down,
    },
    InputMapping::Key {
        keycode: KeyCode::Left,
        action: ActionType::Left,
    },
    InputMapping::Key {
        keycode: KeyCode::Right,
        action: ActionType::Right,
    },
    // Keyboard action/cancel buttons
    InputMapping::Key {
        keycode: KeyCode::Return,
        action: ActionType::Action,
    },
    InputMapping::Key {
        keycode: KeyCode::Escape,
        action: ActionType::Cancel,
    },
    // Gamepad action/cancel buttons
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButtonType::South,
        action: ActionType::Action,
    },
    InputMapping::GamepadButton {
        gamepad: None,
        button: GamepadButtonType::East,
        action: ActionType::Cancel,
    },
    // Gamepad direction stick (left)
    InputMapping::GamepadAxes {
        gamepad: None,
        stick: GamepadStick::Left,
    },
];

app.insert_resource(UiNavInputManager::from_input_map(
    DEFAULT_INPUT_MAP,
    // `stick_tolerance`: Tolerance for gamepad sticks
    0.1,
    // `stick_snap_tolerance`: Tolerance for gamepad sticks snapping to a specified direction
    0.9,
));
```

Update button colors when the `Focusable` changes:

```rust
fn button_style(mut query: Query<(&Focusable, &mut BackgroundColor), Changed<Focusable>>) {
    for (focusable, mut background_color) in query.iter_mut() {
        *background_color = match focusable.computed_state() {
            FocusState::Active | FocusState::Focus => Color::GRAY,
            FocusState::Press => Color::BLACK,
            _ => Color::DARK_GRAY,
        }
        .into();
    }
}
```

Play sounds when navigating between focusables:

```rust
fn handle_focus_change_events(mut events: EventReader<UiNavFocusChangedEvent>) {
    for event in events.read() {
        println!("{event:?}");
        // TODO: Spawn appropriate sound effect
    }
}
```

# Credits

- [bevy-ui-navigation](https://github.com/nicopap/ui-navigation) was the original inspiration, and the source for the
  event reader implementation used in [event_reader.rs](src/event_reader.rs).
