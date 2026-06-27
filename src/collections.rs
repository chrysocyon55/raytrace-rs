//! Collections of objects that comprise renderable scenes.

use std::cmp::Ordering;

use crate::bound::{BoundingBox, Interval};
use crate::hit::{Hit, HitInfo};
use crate::ray::Ray;

/// A list of hittable objects and their collective bounding box.
///
/// Ray collision detection is performed by checking every object in the list,
/// which may be slow for large scenes. Consider using [`SceneTree`] instead
/// for scenes with many objects.
#[derive(Default)]
pub struct ObjList<'a> {
    objects: Vec<Box<dyn Hit + Sync + 'a>>,
    bounds: BoundingBox,
}

impl<'a> ObjList<'a> {
    /// Constructs a new empty scene.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new object into this scene, updating the scene's bounding box
    /// accordingly.
    pub fn push(&mut self, object: Box<dyn Hit + Sync + 'a>) {
        self.bounds = BoundingBox::enclosing(&self.bounds, object.bounds());
        self.objects.push(object);
    }
}

impl<'a> From<Vec<Box<dyn Hit + Sync + 'a>>> for ObjList<'a> {
    fn from(value: Vec<Box<dyn Hit + Sync + 'a>>) -> Self {
        let bounds = value.iter().fold(BoundingBox::empty(), |bbox, obj| {
            BoundingBox::enclosing(&bbox, obj.bounds())
        });
        Self {
            objects: value,
            bounds,
        }
    }
}

impl<'a> FromIterator<Box<dyn Hit + Sync + 'a>> for ObjList<'a> {
    fn from_iter<I: IntoIterator<Item = Box<dyn Hit + Sync + 'a>>>(iter: I) -> Self {
        let mut scene = Self::new();
        for obj in iter.into_iter() {
            scene.push(obj);
        }
        scene
    }
}

impl Hit for ObjList<'_> {
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

/// A hierarchical collection of hittable objects that efficiently computes
/// ray intersections.
pub enum ObjTree<'a> {
    Empty,
    Leaf(Box<dyn Hit + Sync + 'a>),
    Tree {
        left: Box<ObjTree<'a>>,
        right: Box<ObjTree<'a>>,
        bound: BoundingBox,
    },
}

impl<'a> ObjTree<'a> {
    /// Constructs a new scene tree over a list of hittable objects.
    pub fn new(mut objects: Vec<Box<dyn Hit + Sync + 'a>>) -> Self {
        if objects.is_empty() {
            return Self::Empty;
        } else if objects.len() == 1 {
            return Self::Leaf(objects.swap_remove(0));
        }
        // Compute the overall bounding box for this tree.
        let bound = objects.iter().fold(BoundingBox::empty(), |acc, x| {
            BoundingBox::enclosing(&acc, x.bounds())
        });
        // Divide the scene along the longest axis of the current bounding box.
        let get_longest_axis = if bound.x().size() >= bound.y().size() {
            if bound.x().size() >= bound.z().size() {
                BoundingBox::x
            } else {
                BoundingBox::z
            }
        } else if bound.y().size() >= bound.z().size() {
            BoundingBox::y
        } else {
            BoundingBox::z
        };
        // Sort the objects by the coordinates of their bounding boxes along
        // that axis (first by the intervals' starts, then tiebreaking by their
        // ends).
        objects.sort_unstable_by(|obj1, obj2| {
            let axis1 = get_longest_axis(obj1.bounds());
            let axis2 = get_longest_axis(obj2.bounds());
            match axis1.start.total_cmp(&axis2.start) {
                Ordering::Equal => axis1.end.total_cmp(&axis2.end),
                ord => ord,
            }
        });
        // Split the list in half and construct the child nodes recursively.
        let right_objects = objects.split_off(objects.len() / 2);
        let left_objects = objects;
        let left = Box::new(Self::new(left_objects));
        let right = Box::new(Self::new(right_objects));
        Self::Tree { left, right, bound }
    }
}

impl<'a> Hit for ObjTree<'a> {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        match self {
            // A tree containing no objects can never be hit.
            Self::Empty => None,
            // A tree containing a single object just checks that object.
            Self::Leaf(obj) => obj.hit(ray, ray_time),
            // A tree containing subtrees is checked via a recursive binary
            // search.
            Self::Tree { left, right, bound } => {
                // Compute if and when the ray intersects the current tree's
                // bounding box, bailing early if no intersection occurs.
                let intersect_time = bound.intersected_by(ray, *ray_time)?;
                // Check intersections with both of the subtrees. There are
                // several cases:
                // - If neither subtree is intersected, return None.
                // - If only one subtree is intersected, return its info.
                // - If both subtrees are intersected, return the info with the
                //      earliest hit.
                let left_info = left.hit(ray, &intersect_time);
                let right_info = right.hit(ray, &intersect_time);
                match (left_info, right_info) {
                    (None, None) => None,
                    (Some(hit), None) | (None, Some(hit)) => Some(hit),
                    (Some(left_hit), Some(right_hit)) => {
                        if left_hit.time <= right_hit.time {
                            Some(left_hit)
                        } else {
                            Some(right_hit)
                        }
                    }
                }
            }
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
