//! Three-dimensional marching rays.

use super::vec3::Vec3;

/// A three-dimensional, real-valued ray.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Constructs a new ray that starts at the given origin and travels in a
    /// given direction.
    pub const fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Computes the location of this ray at the given time.
    pub fn at_time(&self, time: f64) -> Vec3 {
        self.origin + time * self.direction
    }
}
