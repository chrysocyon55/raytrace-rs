//! Interface for objects that interact with light rays.

use std::fmt::{self, Debug};

use crate::bound::BoundingBox;
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

/// A real-valued interval.
#[derive(Debug, Clone, Copy)]
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

    /// Constructs a new interval that contains all values.
    pub const fn universal() -> Self {
        Self {
            start: f64::NEG_INFINITY,
            end: f64::INFINITY,
        }
    }

    /// Constructs the smallest interval that encloses both of the provided
    /// intervals.
    pub const fn enclosing(iv1: &Self, iv2: &Self) -> Self {
        let start = iv1.start.min(iv2.start);
        let end = iv1.end.max(iv2.end);
        Self { start, end }
    }

    /// Constructs the union of two intervals.
    pub const fn union(iv1: &Self, iv2: &Self) -> Self {
        let start = iv1.start.max(iv2.start);
        let end = iv1.end.min(iv2.end);
        Self { start, end }
    }

    /// Returns the size of this interval.
    pub const fn size(&self) -> f64 {
        (self.end - self.start).min(0.0)
    }

    /// Determines whether the given value is part of this interval,
    /// including the bounds.
    pub const fn contains_inclusive(&self, x: f64) -> bool {
        self.start <= x && x <= self.end
    }

    /// Determines whether the given value is part of this interval,
    /// excluding the bounds.
    pub const fn contains_exclusive(&self, x: f64) -> bool {
        self.start < x && x < self.end
    }

    /// Produces a new interval by expanding the size of this interval by a
    /// given delta.
    ///
    /// The expansion is performed by moving the upper and lower bounds apart
    /// by half of `delta`, resulting in the interval's size increasing by
    /// `delta` overall.
    pub const fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self {
            start: self.start - padding,
            end: self.end + padding,
        }
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

    /// Return a bounding box enclosing this object.
    fn bounds(&self) -> &BoundingBox;
}

/// A list of hittable objects and their collective bounding box.
#[derive(Default)]
pub struct Scene {
    objects: Vec<Box<dyn Hit>>,
    bounds: BoundingBox,
}

impl Scene {
    /// Constructs a new empty scene.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new object into this scene, updating the scene's bounding box
    /// accordingly.
    pub fn push(&mut self, object: Box<dyn Hit>) {
        self.bounds = BoundingBox::enclosing(&self.bounds, object.bounds());
        self.objects.push(object);
    }
}

impl<H> FromIterator<H> for Scene
where
    H: Hit + 'static,
{
    fn from_iter<T: IntoIterator<Item = H>>(iter: T) -> Self {
        let mut scene = Self::new();
        for obj in iter.into_iter() {
            scene.push(Box::new(obj));
        }
        scene
    }
}

impl Hit for Scene {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        let mut curr_interval = self.bounds().intersected_by(ray, *ray_time)?;
        // Iterate over each hittable object in the scene, returning the hit
        // info for the nearest object hit by the ray.
        let mut soonest_hit = None;
        for obj in &self.objects {
            if let Some(curr_info) = obj.hit(ray, &curr_interval) {
                // Only consider collisions that happen sooner than this one.
                curr_interval.end = curr_info.time;
                soonest_hit = Some(curr_info);
            }
        }
        soonest_hit
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }
}
