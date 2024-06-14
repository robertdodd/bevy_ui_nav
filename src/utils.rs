use bevy::{math::bounding::Aabb2d, prelude::*};

/// Utility that performs linear interpolation between `a` and `b` by the value of `d`.
pub fn f32_lerp(a: f32, b: f32, d: f32) -> f32 {
    a * (1.0 - d) + (b * d)
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
    fn f32_lerp_works() {
        assert_relative_eq!(f32_lerp(0.0, 1.0, 0.5), 0.5);
        assert_relative_eq!(f32_lerp(1.0, 0.0, 0.5), 0.5);
        assert_relative_eq!(f32_lerp(-2.0, -1.0, 0.5), -1.5);
    }

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
