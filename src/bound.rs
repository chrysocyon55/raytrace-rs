//! Bounding boxes for optimizing hit detection in larger scenes.

use crate::hit::Interval;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy, Default)]
/// An axis-aligned bounding box.
pub struct BoundingBox {
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

    /// Returns the interval of this bounding box along the x axis.
    pub const fn x(&self) -> Interval {
        self.x
    }

    /// Returns the interval of this bounding box along the y axis.
    pub const fn y(&self) -> Interval {
        self.y
    }

    /// Returns the interval of this bounding box along the z axis.
    pub const fn z(&self) -> Interval {
        self.z
    }

    /// Constructs a new bounding box with the given points as its opposite
    /// corners.
    pub fn from_corners(p1: &Vec3, p2: &Vec3) -> Self {
        let minmax = |x1, x2| if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let x_interval = minmax(p1.x(), p2.x()).into();
        let y_interval = minmax(p1.y(), p2.y()).into();
        let z_interval = minmax(p1.z(), p2.z()).into();
        Self {
            x: x_interval,
            y: y_interval,
            z: z_interval,
        }
    }

    /// Constructs a new bounding box that encloses both of the provided
    /// bounding boxes.
    pub fn enclosing(b1: &Self, b2: &Self) -> Self {
        let x_interval = Interval::enclosing(&b1.x, &b2.x);
        let y_interval = Interval::enclosing(&b1.y, &b2.y);
        let z_interval = Interval::enclosing(&b1.z, &b2.z);
        Self {
            x: x_interval,
            y: y_interval,
            z: z_interval,
        }
    }

    /// Determines whether the given ray intersects this bounding box within
    /// the given time interval.
    ///
    /// If the ray intersects this bounding box in the given time interval,
    /// returns `Some` containing the time interval the ray is within the box.
    /// Otherwise, returns `None`.
    pub fn intersected_by(&self, ray: &Ray, ray_time: Interval) -> Option<Interval> {
        // We want to find the intersection of the ray with the six planes
        // defined by this bounding box's intervals.
        // A ray starting at point p and travelling in the direction d will
        // be at the following point r(t):
        //  r(t) = p + dt
        // This applies to each plane and axis, so for the x-axis and the min
        // plane x_0:
        //  x_0 = p_x + d_x * t_x0
        // Solving for t:
        //  t_x0 = (x_0 - p_x) / d_x
        // Similarly, for the max plane x_1:
        //  t_x1 = (x_1 - p_x) / d_x
        // The time interval [t_x0, t_x1] will be the range of times that the
        // ray intersects this box's x-interval.
        // A ray will intersect the box if and only if the time intervals of
        // all three axes have nonzero overlap.
        let mut intersect_time = ray_time;
        let box_axes = [&self.x, &self.y, &self.z].into_iter();
        let ray_origin = ray.origin.0.iter();
        let ray_dir = ray.direction.0.iter();
        for (box_axis, (ray_origin, ray_dir)) in box_axes.zip(ray_origin.zip(ray_dir)) {
            let t0 = (box_axis.start - ray_origin) / ray_dir;
            let t1 = (box_axis.end - ray_origin) / ray_dir;
            // If the ray direction is negative, then t0 may be greater than
            // t1, so we need to reverse the times.
            let (start, end) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
            // Additionally, if the ray is orthogonally aligned, then `ray_dir`
            // will be 0, causing the times to be +/- infinity (or NaN, if the
            // ray's origin is also on the surface of the box). However, these
            // comparisons should handle those cases gracefully.
            if start > intersect_time.start {
                intersect_time.start = start;
            }
            if end < intersect_time.end {
                intersect_time.end = end;
            }
            // If the intersection time interval is empty, bail early.
            if intersect_time.start >= intersect_time.end {
                return None;
            }
        }
        Some(intersect_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounding_box_constructors() {
        let bbox = BoundingBox::from_corners(&Vec3([1.0, -2.0, 3.0]), &Vec3([-1.0, 2.0, -3.0]));
        assert_eq!(bbox.x.start, -1.0);
        assert_eq!(bbox.x.end, 1.0);
        assert_eq!(bbox.y.start, -2.0);
        assert_eq!(bbox.y.end, 2.0);
        assert_eq!(bbox.z.start, -3.0);
        assert_eq!(bbox.z.end, 3.0);

        let bbox1 = BoundingBox::new((0.0, 1.0).into(), (0.0, 2.0).into(), (0.0, 3.0).into());
        let bbox2 = BoundingBox::new((-4.0, 0.0).into(), (-5.0, 0.0).into(), (-6.0, 0.0).into());
        let bbox = BoundingBox::enclosing(&bbox1, &bbox2);
        assert_eq!(bbox.x.start, -4.0);
        assert_eq!(bbox.x.end, 1.0);
        assert_eq!(bbox.y.start, -5.0);
        assert_eq!(bbox.y.end, 2.0);
        assert_eq!(bbox.z.start, -6.0);
        assert_eq!(bbox.z.end, 3.0);
    }

    #[test]
    fn bounding_box_collisions() {
        let bbox = BoundingBox::new((0.0, 2.0).into(), (-1.0, 1.0).into(), (-1.0, 1.0).into());
        let ray = Ray {
            origin: (-1.0, 0.0, 0.0).into(),
            direction: (1.0, 0.0, 0.0).into(),
        };
        let Some(collide) = bbox.intersected_by(&ray, Interval::universal()) else {
            panic!("collision should occur successfully");
        };
        assert_eq!(collide.start, 1.0);
        assert_eq!(collide.end, 3.0);
    }
}
