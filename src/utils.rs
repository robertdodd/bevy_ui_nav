use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};

/// Utility that traverses the `ChildOf` heirarchy until it finds a parent entity matching a query.
pub(crate) fn find_parent<D, F>(
    entity: Entity,
    child_of_query: &Query<&ChildOf>,
    parent_query: &Query<D, F>,
) -> Option<Entity>
where
    D: QueryData,
    F: QueryFilter,
{
    let mut current = entity;
    let mut parent = None;
    while let Ok(child_of) = child_of_query.get(current) {
        if parent_query.contains(child_of.0) {
            parent = Some(child_of.0);
            break;
        } else {
            current = child_of.0;
        }
    }
    parent
}
