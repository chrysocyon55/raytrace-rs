//! Interface for objects that interact with light rays.

use std::fmt::{self, Debug};

use crate::bound::{BoundingBox, Interval};
use crate::material::Material;
use crate::prim::{RotateY, Translate};
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
    /// Whether the ray hit the front or back face of the object.
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

/// Objects that can interact with ("hit") light rays.
///
/// Objects implementing `Hit` should override `Hit::hit` to determine when a
/// ray collides with the object, and `Hit::bounds` to give an upper bound on
/// the physical size of this object.
/// The provided methods are all instance adapters offered for convenience and
/// should not be overridden.
pub trait Hit {
    /// Determines whether a ray collides with this object in the given time
    /// interval.
    ///
    /// If a collision occurs, returns `Some` containing information about
    /// where and when the collision occurs. Otherwise, if the ray does not
    /// collide with the object, returns `None`.
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>>;

    /// Returns a bounding box enclosing this object.
    fn bounds(&self) -> &BoundingBox;

    /// Translates this object by the given offset.
    fn translate(self, offset: Vec3) -> Translate<Self>
    where
        Self: Sized,
    {
        Translate::new(self, offset)
    }

    /// Rotates this object about the positive y-axis by the given angle, in
    /// degrees.
    fn rotate_y(self, angle: f64) -> RotateY<Self>
    where
        Self: Sized,
    {
        RotateY::new(self, angle)
    }
}
