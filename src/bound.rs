//! Bounding boxes for optimizing hit detection in larger scenes.

use crate::hit::Interval;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy, Default)]
/// An axis-aligned bounding box.
struct BoundingBox {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl BoundingBox {
    /// Constructs an empty bounding box.
    pub fn empty() -> Self {
        Default::default() 
    }

    /// Constructs a new bounding box which encloses the given intervals
    /// along its axes.
    pub const fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    /// Constructs a new bounding box with the given points as its opposite
    /// corners.
    pub fn from_corners(p1: &Vec3, p2: &Vec3) -> Self {
        todo!()
    }

    /// Constructs a new bounding box that encloses both of the provided
    /// bounding boxes.
    pub fn enclosing(b1: &Self, b2: &Self) -> Self {
        todo!()
    }
   
    /// Determines whether the given ray intersects this bounding box within
    /// the given time interval.
    pub fn intersected_by(&self, ray: &Ray, ray_time: &Interval) -> bool {
        todo!()
    }
}
