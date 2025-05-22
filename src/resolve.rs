use bevy::prelude::*;

use crate::{
    focus_node::{FocusNode, FocusTarget},
    prelude::*,
    utils::f32_equal,
};

pub(crate) fn resolve_2d(
    focused: Option<Entity>,
    mut direction: UiNavDirection,
    cycles: bool,
    siblings: &[Entity],
    query: &Query<(
        &Focusable,
        &FocusableOf,
        &ComputedNode,
        &GlobalTransform,
        &InheritedVisibility,
        &Pressables,
    )>,
) -> Option<Entity> {
    let focusables: Vec<(Entity, FocusNode)> = siblings
        .iter()
        .filter_map(|e| {
            if let Ok((focusable, _, node, transform, visibility, _)) = query.get(*e) {
                if !focusable.is_disabled && is_node_visible(visibility.get(), node.size()) {
                    Some((
                        *e,
                        FocusNode {
                            size: node.size(),
                            position: transform.compute_transform().translation.truncate(),
                            is_priority: focusable.is_priority,
                        },
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // get a `FocusNode` for the current focusable
    let current = focusables
        .iter()
        .find(|(e, _)| Some(*e) == focused)
        .map(|(_, f)| f.clone());

    // if current entity not found, then return the first prioritized focusable if we can
    if current.is_none() {
        if let Some((e, _)) = focusables.iter().find(|(_, f)| f.is_priority) {
            return Some(*e);
        }
    }

    // if no current or prioritized focuable was found, change `direction` to `DownRight` and create a fake focus node
    // in the top left.
    let current = current.unwrap_or_else(|| {
        direction = UiNavDirection::DownRight;
        FocusNode {
            size: Vec2::splat(100.),
            position: Vec2::splat(-100.),
            is_priority: false,
        }
    });

    // find the nearest, and furthest nodes in the direction of travel
    let (nearest, furthest) = focusables
        .iter()
        // ignore the current focusable and any focusables outside the current menu
        .filter(|(entity, _)| Some(*entity) != focused)
        // map to a `FocusTarget` type
        .map(|(entity, focus_node)| {
            let distance = current.distance_to(focus_node);
            FocusTarget {
                entity: *entity,
                position: focus_node.position,
                is_in_direction: distance.is_in_direction(direction),
                is_in_axis: distance.is_along_axis(direction),
                // Only prefer movement along direct axes. It doesn't matter when moving diagonally.
                is_prefer: match direction {
                    UiNavDirection::Up | UiNavDirection::Down => distance.is_overlap_y,
                    UiNavDirection::Left | UiNavDirection::Right => distance.is_overlap_x,
                    _ => false,
                },
                overlap: match direction {
                    UiNavDirection::Up | UiNavDirection::Down => distance.overlap_x,
                    UiNavDirection::Left | UiNavDirection::Right => distance.overlap_y,
                    _ => 0.,
                },
                distance,
            }
        })
        // Remove any nodes that do not lie along the axis of the movement event. If wrapping is enabled,
        // allow any nodes along the axis. Otherwise, only allow nodes in the direction of the movement event.
        .filter(|focus_target| {
            if cycles {
                focus_target.is_in_axis
            } else {
                focus_target.is_in_direction
            }
        })
        .fold(
            (None, None),
            #[allow(clippy::type_complexity)]
            |(acc_nearest, acc_furthest), e| -> (Option<FocusTarget>, Option<FocusTarget>) {
                let e_is_in_direction = e.is_in_direction;

                // Fold the nearest focus node in the direction of the movement event
                let nearest = if let Some(acc_nearest) = acc_nearest {
                    // Prefer `e` if it lies in the correct direction and is closer than `acc_nearest`
                    if e_is_in_direction
                        && ((acc_nearest.is_prefer == e.is_prefer
                            && (e.overlap > 0. && acc_nearest.overlap <= 0.
                                || e.distance.total < acc_nearest.distance.total))
                            || (!acc_nearest.is_prefer && e.is_prefer))
                    {
                        Some(e.clone())
                    } else {
                        Some(acc_nearest)
                    }
                } else if e_is_in_direction {
                    // set the initial nearest node
                    Some(e.clone())
                } else {
                    None
                };

                // Fold the furthest focus node
                let furthest = if !cycles {
                    // skip if wrapping is disabled
                    None
                } else if let Some(acc_furthest) = acc_furthest {
                    // Prefer `e` if it is further than `acc_furthest` and does not lie in the dirction of the
                    // movement event.
                    if !e_is_in_direction
                        && ((acc_furthest.is_prefer == e.is_prefer
                            && (e.overlap > 0. && acc_furthest.overlap <= 0.
                                || e.distance.total > acc_furthest.distance.total
                                || (f32_equal(e.distance.total, acc_furthest.distance.total)
                                    && f32_equal(e.overlap, acc_furthest.overlap)
                                    && e.position.x < acc_furthest.position.x)))
                            || (!acc_furthest.is_prefer && e.is_prefer))
                    {
                        Some(e.clone())
                    } else {
                        Some(acc_furthest)
                    }
                } else if !e_is_in_direction {
                    // set the initial furthest node if it does not lie in the direction of the movement event
                    Some(e.clone())
                } else {
                    None
                };

                (nearest, furthest)
            },
        );

    // return the neareset if found, or the furthest if cycles, otherwise None
    if let Some(nearest) = nearest {
        Some(nearest.entity)
    } else if cycles {
        furthest.map(|target| target.entity)
    } else {
        None
    }
}

fn is_node_visible(is_visible: bool, size: Vec2) -> bool {
    is_visible && size.x > f32::EPSILON && size.y > f32::EPSILON
}
