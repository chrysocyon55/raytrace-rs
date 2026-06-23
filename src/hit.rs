//! Interface for objects that interact with light rays.

use std::fmt::{self, Debug};

use crate::bound::{BoundingBox, Interval};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A description of a ray's collision with an object.
#[derive(Clone)]
pub struct HitInfo<'mat> {
    /// The point at which the ray collided with the object.
    pub hit_point: Vec3,
    /// A normal vector to the object at the point of collision.
    /// This always points outwards from the side of the surface hit by the
    /// ray, which may not be the front/outside face of the surface.
    pub normal: Vec3,
    /// The time when the collision occurred.
    pub time: f64,
    /// Whether the ray hit the front face of the object.
    pub is_front_face: bool,
    /// The material of the object hit by the ray.
    pub material: &'mat dyn Material,
}

impl Debug for HitInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HitInfo")
            .field("hit_point", &self.hit_point)
            .field("normal", &self.normal)
            .field("time", &self.time)
            .field("is_front_face", &self.is_front_face)
            .finish_non_exhaustive()
    }
}

/// Determines whether the ray is hitting the front face of an object with the
/// given normal vector.
pub fn hitting_front_face(ray: &Ray, outward_normal: &Vec3) -> bool {
    ray.direction.dot(outward_normal) < 0.0
}

/// Objects that can interact with ("hit") light rays.
pub trait Hit {
    /// Determines whether a ray collides with this object in the given time
    /// interval.
    ///
    /// If a collision occurs, returns `Some` containing information about
    /// where and when the collision occurs. Otherwise, if the ray does not
    /// collide with the object, returns `None`.
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>>;

    /// Return a bounding box enclosing this object.
    fn bounds(&self) -> &BoundingBox;
}
