use bevy::{
    ecs::query::{QueryData, QueryFilter, ReadOnlyQueryData},
    prelude::*,
};

use crate::events::*;

/// Trait for events that referencce an Entity.
pub trait UiNavEvent {
    fn entity(&self) -> Entity;
}

impl UiNavEvent for UiNavClickEvent {
    fn entity(&self) -> Entity {
        self.0
    }
}

impl UiNavEvent for UiNavCancelEvent {
    fn entity(&self) -> Entity {
        self.0
    }
}

/// Extend [`EventReader<impl UiNavEvent>`] with methods to simplify working with UI navigation events related to entities.
pub trait UiNavEventReaderExt<'w, 's, T: Event + UiNavEvent> {
    /// Create a [`UiNavEventReader`] from this event reader.
    fn nav_iter(&mut self) -> UiNavEventReader<'w, 's, '_, T>;
}
impl<'w, 's, T: Event + UiNavEvent> UiNavEventReaderExt<'w, 's, T> for EventReader<'w, 's, T> {
    fn nav_iter(&mut self) -> UiNavEventReader<'w, 's, '_, T> {
        UiNavEventReader { event_reader: self }
    }
}

/// A wrapper for `EventReader<impl UiNavEvent>` to simplify dealing with navigation events related to entities.
pub struct UiNavEventReader<'w, 's, 'a, T: Event + UiNavEvent> {
    event_reader: &'a mut EventReader<'w, 's, T>,
}
impl<'w, 's, 'a, T: Event + UiNavEvent> UiNavEventReader<'w, 's, 'a, T> {
    /// Iterate over query items pointed to by the events.
    pub fn in_query<'b, 'c: 'b, Q: ReadOnlyQueryData, F: QueryFilter>(
        &'b mut self,
        query: &'c Query<Q, F>,
    ) -> impl Iterator<Item = Q::Item<'c>> + 'b {
        query.iter_many(self.event_reader.read().map(|event| event.entity()))
    }

    /// Run `for_each` with result of `query` for each event entity.
    ///
    /// Unlike [`Self::in_query`] this works with mutable queries.
    pub fn in_query_foreach_mut<Q: QueryData, F: QueryFilter>(
        &mut self,
        query: &mut Query<Q, F>,
        mut for_each: impl FnMut(Q::Item<'_>),
    ) {
        let mut iter = query.iter_many_mut(self.event_reader.read().map(|event| event.entity()));
        while let Some(item) = iter.fetch_next() {
            for_each(item)
        }
    }
}
