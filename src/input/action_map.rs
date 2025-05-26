use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use super::events::ActionEvents;

#[derive(Debug, Copy, Clone)]
pub struct ActionRepeat {
    elapsed_secs: f32,
    trigger_count: u32,
}

impl Default for ActionRepeat {
    fn default() -> Self {
        Self {
            elapsed_secs: 0.,
            trigger_count: 0,
        }
    }
}

/// Data associated with an [`InputAction`] marker.
///
/// Stored inside [`ActionMap`].
///
/// This struct could also be created manually to track state for an action
/// with externally sourced data (e.g., network). Use [`Self::update`] to apply
/// the data followed by [`Self::trigger_events`].
#[derive(Debug, Clone)]
pub struct Action {
    state: ActionState,
    events: ActionEvents,
    value: bool,
    elapsed_secs: f32,
    fired_secs: f32,
    repeat: Option<ActionRepeat>,
}

impl Action {
    /// Creates a new instance associated with action `A`.
    ///
    /// [`Self::trigger_events`] will trigger events for `A`.
    #[must_use]
    pub fn new(repeat: Option<ActionRepeat>) -> Self {
        Self {
            state: Default::default(),
            events: ActionEvents::empty(),
            value: false,
            elapsed_secs: 0.0,
            fired_secs: 0.0,
            repeat,
        }
    }

    /// Updates internal state.
    pub fn update(
        &mut self,
        time: &Time<Virtual>,
        value: bool,
        repeat_delay: f32,
        repeat_interval: f32,
    ) {
        match self.state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
                if let Some(repeat) = &mut self.repeat {
                    repeat.elapsed_secs = 0.;
                    repeat.trigger_count = 0;
                }
            }
            ActionState::Ongoing => {
                self.elapsed_secs += time.delta_secs();
                self.fired_secs = 0.0;
                if let Some(repeat) = &mut self.repeat {
                    repeat.elapsed_secs += time.delta_secs();
                }
            }
            ActionState::Fired => {
                self.elapsed_secs += time.delta_secs();
                self.fired_secs += time.delta_secs();
                if let Some(repeat) = &mut self.repeat {
                    repeat.elapsed_secs += time.delta_secs();
                }
            }
        }

        // define the new state
        let mut state = if !value {
            ActionState::None
        } else if self.state == ActionState::None {
            ActionState::Fired
        } else {
            ActionState::Ongoing
        };

        // apply repeat to new state
        if let (Some(repeat), ActionState::Ongoing, true) = (&mut self.repeat, self.state, value) {
            if repeat.trigger_count == 0 && repeat.elapsed_secs > repeat_delay {
                repeat.trigger_count += 1;
                repeat.elapsed_secs = 0.;
                state = ActionState::Fired;
            // if repeat.elapsed_secs >= self.interval * trigger_count as f32 {
            } else if repeat.trigger_count > 0
                && repeat.elapsed_secs >= repeat_interval * repeat.trigger_count as f32
            {
                repeat.trigger_count += 1;
                state = ActionState::Fired;
            }
        }

        // if value {
        //     println!("UPDATE");
        //     println!("self.state = {:?}", self.state);
        //     println!("state = {:?}", state);
        // }

        self.events = ActionEvents::new(self.state, state);
        self.state = state;
        self.value = value;
    }

    /// Returns the current state.
    pub fn events(&self) -> ActionEvents {
        self.events
    }
}

/// State for [`Action`].
///
/// States are ordered by their significance.
///
/// See also [`ActionEvents`] and [`ActionBinding`]().
#[derive(
    Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect,
)]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, [`Hold`](crate::input_condition::hold::Hold) condition
    /// requires its state to be maintained over several frames.
    Ongoing,
    /// The condition has been met.
    Fired,
}
