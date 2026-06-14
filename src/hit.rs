//! Interface for objects that interact with light rays.

use std::fmt::{self, Debug};

use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// The face direction of a hittable object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Front,
    Back,
}

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
    pub face: Face,
    /// The material of the object hit by the ray.
    pub material: &'mat dyn Material,
}

impl Debug for HitInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HitInfo")
            .field("hit_point", &self.hit_point)
            .field("normal", &self.normal)
            .field("time", &self.time)
            .field("face", &self.face)
            .finish_non_exhaustive()
    }
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

/// An interval of time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub start: f64,
    pub end: f64,
}

impl Interval {
    /// Constructs a new empty interval.
    pub const fn empty() -> Self {
        Self {
            start: f64::INFINITY,
            end: f64::NEG_INFINITY,
        }
    }

    /// Constructs a new interval that contains all times.
    pub const fn universal() -> Self {
        Self {
            start: f64::NEG_INFINITY,
            end: f64::INFINITY,
        }
    }

    /// Returns the size of this interval.
    pub const fn size(&self) -> f64 {
        0.0_f64.min(self.end - self.start)
    }

    // Determines whether the given time is part of this interval. including
    // the bounds.
    pub const fn contains_inclusive(&self, t: f64) -> bool {
        self.start <= t && t <= self.end
    }

    // Determines whether the given time is part of this interval, excluding
    // the bounds.
    pub const fn contains_exclusive(&self, t: f64) -> bool {
        self.start < t && t < self.end
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<(f64, f64)> for Interval {
    fn from(value: (f64, f64)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<Interval> for (f64, f64) {
    fn from(value: Interval) -> Self {
        (value.start, value.end)
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
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>>;
}

// A slice of objects that are `Hit` is also `Hit`.
impl<T: Hit> Hit for &[T] {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        // Find the nearest valid collision with any of the objects in this
        // slice:
        let start = ray_time.start;
        let mut curr_end = ray_time.end;
        let mut soonest_hit = None;
        for obj in self.iter() {
            let curr_interval = Interval::from((start, curr_end));
            if let Some(info) = obj.hit(ray, &curr_interval) {
                // Only consider collisions that happen sooner than this one.
                curr_end = info.time;
                soonest_hit = Some(info);
            }
        }
        soonest_hit
    }
}
