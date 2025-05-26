mod components;
mod event_reader;
mod events;
mod input;
mod plugin;
mod queries;
mod resolve;
mod resources;
mod types;
mod utils;

pub mod prelude {
    pub use crate::{
        components::*, event_reader::*, events::*, input::*, plugin::*, resources::*, types::*,
    };
}

#[cfg(test)]
#[macro_use]
extern crate approx;
