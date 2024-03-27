mod components;
mod default_input_map;
mod event_reader;
mod events;
mod focus_node;
mod input;
mod plugin;
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
