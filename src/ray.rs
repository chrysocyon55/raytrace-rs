//! Three-dimensional marching rays.

use super::vec3::Vec3;

/// A three-dimensional, real-valued ray.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Computes the location of this ray at the given time.
    pub fn at_time(&self, time: f64) -> Vec3 {
        self.origin + time * self.direction
    }
}
