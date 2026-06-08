//! Interface for objects that interact with light rays.

use crate::ray::Ray;
use crate::vec3::Vec3;

/// The face direction of a hittable object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Front,
    Back,
}

/// A description of a ray's collision with an object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitInfo {
    /// The point at which the ray collided with the object.
    pub hit_point: Vec3,
    /// A normal vector to the object at the point of collision.
    /// This always points outwards from the side of the surface hit by the
    /// ray, which may not be the front/outside face of the surface.
    pub normal: Vec3,
    /// The time when the collision occurred.
    pub time: f64,
    /// Whether the ray hit the front face of the object.
    pub face: Face,
}

/// Determines whether the ray is hitting the front or back face of an object,
/// and produces the unit normal vector for that face.
///
/// Assumes that `outward_normal` is a unit vector that points outwards from
/// the object's front face.
pub fn face_normal(ray: &Ray, outward_normal: Vec3) -> (Face, Vec3) {
    let is_front_face = ray.direction.dot(&outward_normal) < 0.0;
    if is_front_face {
        (Face::Front, outward_normal)
    } else {
        (Face::Back, -outward_normal)
    }
}

/// Objects that can interact with ("hit") light rays.
pub trait Hit {
    /// Determines whether a ray collides with this object in the given time
    /// interval.
    ///
    /// If a collision occurs, returns `Some` containing information about
    /// where and when the collision occurs. Otherwise, if the ray does not
    /// collide with the object, returns `None`.
    fn hit(&self, ray: &Ray, time_interval: (f64, f64)) -> Option<HitInfo>;
}

// A slice of objects that are `Hit` is also `Hit`.
impl<T> Hit for &[T]
where
    T: Hit,
{
    fn hit(&self, ray: &Ray, time_interval: (f64, f64)) -> Option<HitInfo> {
        // Find the nearest valid collision with any of the objects in this
        // slice:
        let (start_time, mut curr_end_time) = time_interval;
        let mut soonest_hit = None;
        for obj in self.iter() {
            if let Some(info) = obj.hit(ray, (start_time, curr_end_time)) {
                soonest_hit = Some(info);
                // Only consider collisions that happen sooner than this one.
                curr_end_time = info.time;
            }
        }
        soonest_hit
    }
}
