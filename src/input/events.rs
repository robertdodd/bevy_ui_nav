use core::fmt::Debug;

use bitflags::bitflags;

use super::action_map::ActionState;

bitflags! {
    /// Bitset with events triggered by updating [`ActionState`] for an action.
    ///
    /// Stored inside [`Action`](crate::action_map::Action).
    ///
    /// On transition, events will be triggered with dedicated types that correspond to bitflags.
    ///
    /// Table of state transitions:
    ///
    /// | Last state                  | New state                | Events                    |
    /// | --------------------------- | ------------------------ | ------------------------- |
    /// | [`ActionState::None`]       | [`ActionState::None`]    | No events                 |
    /// | [`ActionState::None`]       | [`ActionState::Ongoing`] | [`Started`] + [`Ongoing`] |
    /// | [`ActionState::None`]       | [`ActionState::Fired`]   | [`Started`] + [`Fired`]   |
    /// | [`ActionState::Ongoing`]    | [`ActionState::None`]    | [`Canceled`]              |
    /// | [`ActionState::Ongoing`]    | [`ActionState::Ongoing`] | [`Ongoing`]               |
    /// | [`ActionState::Ongoing`]    | [`ActionState::Fired`]   | [`Fired`]                 |
    /// | [`ActionState::Fired`]      | [`ActionState::Fired`]   | [`Fired`]                 |
    /// | [`ActionState::Fired`]      | [`ActionState::Ongoing`] | [`Ongoing`]               |
    /// | [`ActionState::Fired`]      | [`ActionState::None`]    | [`Completed`]             |
    ///
    /// The meaning of each kind depends on the assigned [`InputCondition`](crate::input_condition::InputCondition)s.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ActionEvents: u8 {
        /// Corresponds to [`Started`].
        const STARTED = 0b00000001;
        /// Corresponds to [`Ongoing`].
        const ONGOING = 0b00000010;
        /// Corresponds to [`Fired`].
        const FIRED = 0b00000100;
        /// Corresponds to [`Canceled`].
        const CANCELED = 0b00001000;
        /// Corresponds to [`Completed`].
        const COMPLETED = 0b00010000;
    }
}

impl ActionEvents {
    /// Creates a new instance based on state transition.
    pub fn new(previous: ActionState, current: ActionState) -> ActionEvents {
        match (previous, current) {
            (ActionState::None, ActionState::None) => ActionEvents::empty(),
            (ActionState::None, ActionState::Ongoing) => {
                ActionEvents::STARTED | ActionEvents::ONGOING
            }
            (ActionState::None, ActionState::Fired) => ActionEvents::STARTED | ActionEvents::FIRED,
            (ActionState::Ongoing, ActionState::None) => ActionEvents::CANCELED,
            (ActionState::Ongoing, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Ongoing, ActionState::Fired) => ActionEvents::FIRED,
            (ActionState::Fired, ActionState::None) => ActionEvents::COMPLETED,
            (ActionState::Fired, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Fired, ActionState::Fired) => ActionEvents::FIRED,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bevy::time::{Time, Virtual};

//     use super::*;
//     use crate::input::action_map::Action;

//     #[test]
//     fn none_none() {
//         let events = transition(ActionState::None, ActionState::None);
//         assert!(events.is_empty());
//     }

//     #[test]
//     fn none_ongoing() {
//         let events = transition(ActionState::None, ActionState::Ongoing);
//         assert_eq!(events, ActionEvents::STARTED | ActionEvents::ONGOING);
//     }

//     #[test]
//     fn none_fired() {
//         let events = transition(ActionState::None, ActionState::Fired);
//         assert_eq!(events, ActionEvents::STARTED | ActionEvents::FIRED);
//     }

//     #[test]
//     fn ongoing_none() {
//         let events = transition(ActionState::Ongoing, ActionState::None);
//         assert_eq!(events, ActionEvents::CANCELED);
//     }

//     #[test]
//     fn ongoing_ongoing() {
//         let events = transition(ActionState::Ongoing, ActionState::Ongoing);
//         assert_eq!(events, ActionEvents::ONGOING);
//     }

//     #[test]
//     fn ongoing_fired() {
//         let events = transition(ActionState::Ongoing, ActionState::Fired);
//         assert_eq!(events, ActionEvents::FIRED);
//     }

//     #[test]
//     fn fired_none() {
//         let events = transition(ActionState::Fired, ActionState::None);
//         assert_eq!(events, ActionEvents::COMPLETED);
//     }

//     #[test]
//     fn fired_ongoing() {
//         let events = transition(ActionState::Fired, ActionState::Ongoing);
//         assert_eq!(events, ActionEvents::ONGOING);
//     }

//     #[test]
//     fn fired_fired() {
//         let events = transition(ActionState::Fired, ActionState::Fired);
//         assert_eq!(events, ActionEvents::FIRED);
//     }

//     fn transition(initial_state: ActionState, target_state: ActionState) -> ActionEvents {
//         let time = Time::<Virtual>::default();
//         let mut action = Action::new();
//         action.update(&time, initial_state, true);
//         action.update(&time, target_state, true);
//         action.events()
//     }
// }
