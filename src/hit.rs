//! Interface for objects that interact with light rays.

use crate::ray::Ray;
use crate::vec3::Vec3;

/// A description of a ray's collision with an object.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HitInfo {
    /// The point at which the ray collided with the object.
    pub hit_point: Vec3,
    /// A normal vector to the object at the point of collision.
    pub normal: Vec3,
    /// The time when the collision occurred.
    pub time: f64,
}

/// Objects that can be hit by light rays.
pub trait Hit {
    /// Determines whether a ray collides with this object in the given time
    /// interval.
    ///
    /// If a collision occurs, returns `Some` containing information about
    /// where and when the collision occurs. Otherwise, if the ray does not
    /// collide with the object, returns `None`.
    fn hit(&self, ray: &Ray, time_interval: (f64, f64)) -> Option<HitInfo>;
}
