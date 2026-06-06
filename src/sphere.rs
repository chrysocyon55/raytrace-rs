//! The sphere collider.

use crate::hit::{Hit, HitInfo};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A spherical collider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    /// Constructs a new sphere at a given center with a given radius.
    ///
    /// Panics if `radius` is less than or equal to `0.0`.
    pub const fn new(center: Vec3, radius: f64) -> Self {
        assert!(radius > 0.0);
        Self { center, radius }
    }
}

impl Hit for Sphere {
    fn hit(&self, ray: &Ray, time_interval: (f64, f64)) -> Option<HitInfo> {
        // A sphere is a set of vectors P that are all a distance of r away from
        // the center C. We can express this using a dot product:
        //      (C - P) . (C - P) = r^2
        // Each ray is of the form Q + dt for some origin Q, direction d, and time
        // t, which we plug in for P:
        //      (C - (Q + dt)) . (C - (Q + dt)) = r^2
        //      (-dt + (C - Q)) . (-dt + (C - Q)) = r^2
        //      t^2 * (d . d) - 2td . (C - Q) + (C - Q) . (C - Q) = r^2
        //      t^2 * (d . d) - 2td . (C - Q) + (C - Q) . (C - Q) - r^2 = 0
        // This is a quadratic equation in t, with the following coefficients:
        //      a = d . d
        //      b = -2d . (C - Q)
        //      c = (C - Q) . (C - Q) - r^2
        // By computing the discriminant of the quadratic formula (the inside of
        // the square root), we can see whether the equation has any real
        // solutions. If it does, then the ray will intersect with the sphere.
        //      t = (-b +/- sqrt(b^2 - 4ac)) / 2a
        // Let h = d . (C - Q), then b = -2h:
        //      t = (2h +/- sqrt((-2h)^2 - 4ac)) / 2a
        //      t = (2h +/- 2*sqrt(h^2 - ac)) / 2a
        //      t = (h +/- sqrt(h^2 - ac)) / a
        let qc = self.center - ray.origin;
        let a = ray.direction.dot(&ray.direction);
        let h = ray.direction.dot(&qc);
        let c = qc.dot(&qc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            // A negative discriminant means there are no real solutions for
            // t, so the ray does not intersect the sphere.
            return None;
        }
        let discr_root = discriminant.sqrt();
        // Compute the intersection times and check whether they are within
        // the provided interval, starting with the nearer point:
        let (start, end) = time_interval;
        let time_near = (h - discr_root) / a;
        let time = if time_near >= start && time_near <= end {
            time_near
        } else {
            let time_far = (h + discr_root) / a;
            if time_far >= start && time_far <= end {
                time_far
            } else {
                // Neither collision time is within the interval.
                return None;
            }
        };
        let hit_point = ray.at_time(time);
        let normal = (hit_point - self.center) / self.radius; // unit normal
    
        dbg!(&normal);
        Some(HitInfo {
            hit_point,
            normal,
            time,
        })
    }
}
