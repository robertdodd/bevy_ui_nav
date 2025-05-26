use bevy::{math::bounding::Aabb2d, prelude::*};

use crate::prelude::*;

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
    )>,
) -> Option<Entity> {
    let focusables: Vec<(Entity, FocusNode)> = siblings
        .iter()
        .filter_map(|e| {
            if let Ok((focusable, _, node, transform, visibility)) = query.get(*e) {
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

/// Type used internally to describe the distance and direction to a potential navigation target.
#[derive(Debug, Clone)]
pub(crate) struct FocusTarget {
    pub entity: Entity,
    pub position: Vec2,
    pub distance: FocusNodeDistance,
    pub is_prefer: bool,
    pub is_in_direction: bool,
    pub is_in_axis: bool,
    pub overlap: f32,
}

/// Type used internally to define a focus node's position and size.
#[derive(Debug, Clone)]
pub(crate) struct FocusNode {
    pub size: Vec2,
    pub position: Vec2,
    pub is_priority: bool,
}

impl FocusNode {
    pub fn get_aabb(&self) -> Aabb2d {
        Aabb2d::new(self.position, self.size / 2.)
    }

    pub fn distance_to(&self, other: &Self) -> FocusNodeDistance {
        let aabb_self = self.get_aabb();
        let aabb_other = other.get_aabb();

        let overlap = overlap_between_aabbs(&aabb_self, &aabb_other);

        let is_overlap_x = overlap.y > 0.;
        let is_overlap_y = overlap.x > 0.;

        let distance = distance_between_aabbs(&aabb_self, &aabb_other);

        FocusNodeDistance {
            is_left: aabb_other.min.x < aabb_self.min.x,
            is_right: aabb_other.max.x > aabb_self.max.x,
            is_up: aabb_other.min.y < aabb_self.min.y,
            is_down: aabb_other.max.y > aabb_self.max.y,
            is_overlap_x,
            is_overlap_y,
            total: distance,
            overlap_x: overlap.x,
            overlap_y: overlap.x,
        }
    }
}

/// Type used internally to describe the direction and distance between two nodes, and whether they overlap along any
/// axes.
#[derive(Debug, Clone)]
pub(crate) struct FocusNodeDistance {
    pub is_left: bool,
    pub is_right: bool,
    pub is_up: bool,
    pub is_down: bool,
    pub is_overlap_x: bool,
    pub is_overlap_y: bool,
    pub total: f32,
    pub overlap_x: f32,
    pub overlap_y: f32,
}

impl FocusNodeDistance {
    pub fn is_in_direction(&self, direction: UiNavDirection) -> bool {
        match direction {
            UiNavDirection::Up => self.is_up,
            UiNavDirection::Down => self.is_down,
            UiNavDirection::Left => self.is_left,
            UiNavDirection::Right => self.is_right,
            UiNavDirection::UpLeft => self.is_up && self.is_left,
            UiNavDirection::UpRight => self.is_up && self.is_right,
            UiNavDirection::DownLeft => self.is_down && self.is_left,
            UiNavDirection::DownRight => self.is_down && self.is_right,
        }
    }

    pub fn is_along_axis(&self, direction: UiNavDirection) -> bool {
        match direction {
            UiNavDirection::Up | UiNavDirection::Down => self.is_up || self.is_down,
            UiNavDirection::Left | UiNavDirection::Right => self.is_left || self.is_right,
            UiNavDirection::UpLeft | UiNavDirection::DownRight => {
                self.is_up && self.is_left || self.is_down && self.is_right
            }
            UiNavDirection::UpRight | UiNavDirection::DownLeft => {
                self.is_up && self.is_right || self.is_down && self.is_left
            }
        }
    }
}
pub fn f32_equal(a: f32, b: f32) -> bool {
    (b - a).abs() < 0.0001
}

/// Calculate the distance between two AABBs.
/// Source: ChatGPT
pub fn distance_between_aabbs(aabb1: &Aabb2d, aabb2: &Aabb2d) -> f32 {
    // Helper function to calculate the distance between two intervals
    fn interval_distance(min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
        if max1 < min2 {
            min2 - max1
        } else if max2 < min1 {
            min1 - max2
        } else {
            0.0
        }
    }

    // Calculate the distance between the AABBs in each dimension
    let dx = interval_distance(aabb1.min.x, aabb1.max.x, aabb2.min.x, aabb2.max.x);
    let dy = interval_distance(aabb1.min.y, aabb1.max.y, aabb2.min.y, aabb2.max.y);

    // Calculate the Euclidean distance
    (dx * dx + dy * dy).sqrt()
}

/// Calculate the distance between two AABBs.
pub fn overlap_between_aabbs(aabb1: &Aabb2d, aabb2: &Aabb2d) -> Vec2 {
    let overlap_x = compute_overlap(aabb1.min.x, aabb1.max.x, aabb2.min.x, aabb2.max.x);
    let overlap_y = compute_overlap(aabb1.min.y, aabb1.max.y, aabb2.min.y, aabb2.max.y);
    Vec2::new(overlap_x, overlap_y)
}

fn compute_overlap(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> f32 {
    let min = min_a.min(min_b);
    let max = max_a.max(max_b);
    let size_a = max_a - min_a;
    let size_b = max_b - min_b;
    (size_a + size_b) - (max - min)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn f32_equal_works() {
        assert!(f32_equal(0.0, 0.0));
        assert!(f32_equal(0.00000001, 0.00000002));
    }

    #[test]
    fn distance_between_aabbs_works() {
        //           *---------*
        //           |  aabb1  |
        //           *---------*
        //           | <-- distance = spacer
        // *---------*
        // |  aabb2  |
        // *---------*
        let half_size = Vec2::new(50., 10.);
        let aabb1 = Aabb2d::new(Vec2::ZERO, half_size);
        let aabb2 = Aabb2d::new(half_size * 2. + Vec2::new(0., 10.), half_size);
        let distance = distance_between_aabbs(&aabb1, &aabb2);
        assert_relative_eq!(distance, 10.);
    }

    #[test]
    fn overlap_between_aabbs_works() {
        // spacer
        // |
        // v  *---------*
        //    |  aabb1  |
        //    *---------*
        //    [      ] <---- overlap.x
        // *---------*
        // |  aabb2  |
        // *---------*
        let half_size = Vec2::new(50., 10.);
        let spacer = 10.;
        let aabb1 = Aabb2d::new(Vec2::ZERO, half_size);
        let aabb2 = Aabb2d::new(Vec2::new(spacer, half_size.y * 2. + spacer), half_size);
        let overlap = overlap_between_aabbs(&aabb1, &aabb2);
        assert_relative_eq!(overlap.x, half_size.x * 2. - spacer);
        assert_relative_eq!(overlap.y, -spacer);
    }
}
