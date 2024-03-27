/// Utility that performs linear interpolation between `a` and `b` by the value of `d`.
pub fn f32_lerp(a: f32, b: f32, d: f32) -> f32 {
    a * (1.0 - d) + (b * d)
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
}
