use bevy::input_focus::tab_navigation::NavAction;

use super::{input_map::Input, ActionType};

/// Bindings associated with an [`InputAction`] marker.
///
/// Stored inside [`Actions`](crate::actions::Actions).
///
/// Bindings are stored separately from [`ActionMap`] to allow reading other actions' data during evaluation.
///
/// Action bindings evaluation follows these steps:
///
/// 1. Iterate over each [`ActionValue`] from the associated [`Input`]s:
///    1.1. Apply input-level [`InputModifier`]s.
///    1.2. Evaluate input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
/// 2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine based on [`InputAction::ACCUMULATION`].
///    Combined value will be converted into [`InputAction::Output`] using [`ActionValue::convert`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///    Final value will be converted into [`InputAction::Output`] using [`ActionValue::convert`].
pub struct ActionBinding {
    action: ActionType,
    consume_input: bool,
    require_reset: bool,

    modifiers: Vec<Box<dyn InputModifier>>,
    conditions: Vec<Box<dyn InputCondition>>,
    inputs: Vec<Input>,

    /// Consumed inputs during state evaluation.
    consume_buffer: Vec<Input>,
}

impl ActionBinding {
    #[must_use]
    pub(crate) fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::Output::DIM,
            consume_input: A::CONSUME_INPUT,
            accumulation: A::ACCUMULATION,
            require_reset: A::REQUIRE_RESET,
            modifiers: Default::default(),
            conditions: Default::default(),
            inputs: Default::default(),
            consume_buffer: Default::default(),
        }
    }
}
