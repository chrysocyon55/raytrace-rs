//! Bounding volume hierarchy (BVH) for efficient ray collisions in large
//! scenes.

use std::cmp::Ordering;

use rand::RngExt;

use crate::bound::BoundingBox;
use crate::hit::{Hit, HitInfo, Interval};
use crate::ray::Ray;

/// A hierarchical scene that efficiently computes ray intersections.
pub enum SceneTree {
    Empty,
    Leaf(Box<dyn Hit>),
    Tree {
        left: Box<SceneTree>,
        right: Box<SceneTree>,
        bound: BoundingBox,
    },
}

impl SceneTree {
    /// Constructs a new scene tree over a list of hittable objects.
    pub fn new(mut objects: Vec<Box<dyn Hit>>) -> Self {
        if objects.is_empty() {
            return Self::Empty;
        } else if objects.len() == 1 {
            return Self::Leaf(objects.into_iter().next().unwrap());
        }
        // Pick a random axis to divide the scene along.
        let get_axis =
            [BoundingBox::x, BoundingBox::y, BoundingBox::z][rand::rng().random_range(0..2)];
        // Sort the objects by the coordinates of their bounding boxes along
        // that axis (first by the intervals' starts, then tiebreaking by their
        // ends).
        let cmp_bounds = |obj1: &Box<dyn Hit>, obj2: &Box<dyn Hit>| -> Ordering {
            let axis1 = get_axis(obj1.bounds());
            let axis2 = get_axis(obj2.bounds());
            match axis1.start.total_cmp(&axis2.start) {
                Ordering::Equal => axis1.end.total_cmp(&axis2.end),
                ord => ord,
            }
        };
        objects.sort_unstable_by(cmp_bounds);
        // Split the list in half and construct the child nodes recursively.
        let right_objects = objects.split_off(objects.len() / 2);
        let left_objects = objects;
        let left = Box::new(Self::new(left_objects));
        let right = Box::new(Self::new(right_objects));
        // Compute the overall bounding box for this tree.
        let bound = BoundingBox::enclosing(left.bounds(), right.bounds());
        Self::Tree { left, right, bound }
    }
}

impl Hit for SceneTree {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        match self {
            Self::Empty => None,
            Self::Leaf(obj) => obj.hit(ray, ray_time),
            Self::Tree { left, right, bound } => todo!("binary search impl"),
        }
    }

    fn bounds(&self) -> &BoundingBox {
        static EMPTY_BBOX: BoundingBox =
            BoundingBox::new(Interval::empty(), Interval::empty(), Interval::empty());
        match self {
            Self::Empty => &EMPTY_BBOX,
            Self::Leaf(obj) => obj.bounds(),
            Self::Tree {
                left: _,
                right: _,
                bound,
            } => bound,
        }
    }
}
