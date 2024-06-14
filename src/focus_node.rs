use bevy::{math::bounding::Aabb2d, prelude::*};

use crate::{
    prelude::UiNavDirection,
    utils::{distance_between_aabbs, overlap_between_aabbs},
};

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
    pub menu: Option<Entity>,
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
