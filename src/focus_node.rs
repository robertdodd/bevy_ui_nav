use bevy::prelude::*;

use crate::prelude::UiNavDirection;

/// Type used internally to describe the distance and direction to a potential navigation target.
#[derive(Debug, Clone)]
pub(crate) struct FocusTarget {
    pub entity: Entity,
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
}

impl FocusNode {
    pub fn get_points(&self) -> [Vec2; 4] {
        let half_size = self.size / 2.;
        [
            // top left
            self.position - half_size,
            // bottom right
            self.position + half_size,
            // bottom left
            self.position + Vec2::new(-half_size.x, half_size.y),
            // top right
            self.position + Vec2::new(half_size.x, -half_size.y),
        ]
    }

    pub fn distance_to(&self, other: &Self) -> FocusNodeDistance {
        let self_points = self.get_points();
        let other_points = other.get_points();

        let min_self = self_points
            .iter()
            .fold(self_points[0], |acc, &e| acc.min(e));
        let max_self = self_points
            .iter()
            .fold(self_points[0], |acc, &e| acc.max(e));
        let min_other = other_points
            .iter()
            .fold(other_points[0], |acc, &e| acc.min(e));
        let max_other = other_points
            .iter()
            .fold(other_points[0], |acc, &e| acc.max(e));

        let is_left = min_other.x < min_self.x;
        let is_right = max_other.x > max_self.x;
        let is_up = min_other.y < min_self.y;
        let is_down = max_other.y > max_self.y;

        let overlap_x = compute_overlap(min_self.x, max_self.x, min_other.x, max_other.x);
        let overlap_y = compute_overlap(min_self.y, max_self.y, min_other.y, max_other.y);

        let is_overlap_x = overlap_y > 0.;
        let is_overlap_y = overlap_x > 0.;

        let distance = self_points
            .iter()
            .map(|p| {
                other_points
                    .iter()
                    .map(|po| po.distance(*p))
                    .reduce(|acc, e| if e < acc { e } else { acc })
            })
            .reduce(|acc, e| if e < acc { e } else { acc })
            .flatten()
            .unwrap();

        FocusNodeDistance {
            is_left,
            is_right,
            is_up,
            is_down,
            is_overlap_x,
            is_overlap_y,
            total: distance,
            overlap_x,
            overlap_y,
        }
    }
}

fn compute_overlap(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> f32 {
    let min = min_a.min(min_b);
    let max = max_a.max(max_b);
    let size_a = max_a - min_a;
    let size_b = max_b - min_b;
    (size_a + size_b) - (max - min)
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
