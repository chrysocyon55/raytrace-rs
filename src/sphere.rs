//! The sphere collider.

use std::fmt::Debug;

use crate::bound::BoundingBox;
use crate::hit::{self, Hit, HitInfo, Interval};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A spherical collider.
pub struct Sphere<'mat> {
    center: Vec3,
    radius: f64,
    material: &'mat dyn Material,
    bounds: BoundingBox,
}

impl<'mat> Sphere<'mat> {
    /// Constructs a new sphere at a given center with a given radius, whose
    /// surface is a given material.
    ///
    /// Panics if `radius` is less than or equal to `0.0`.
    pub fn new(center: Vec3, radius: f64, material: &'mat dyn Material) -> Self {
        assert!(radius > 0.0);
        let center_to_bbox_corner = Vec3([radius; 3]);
        let bounds = BoundingBox::from_corners(
            &(center - center_to_bbox_corner),
            &(center + center_to_bbox_corner),
        );
        Self {
            center,
            radius,
            material,
            bounds,
        }
    }
}

impl Debug for Sphere<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sphere")
            .field("center", &self.center)
            .field("radius", &self.radius)
            .finish_non_exhaustive()
    }
}

impl<'mat> Hit for Sphere<'mat> {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
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
        let a = ray.direction.square_length();
        let h = ray.direction.dot(&qc);
        let c = qc.square_length() - (self.radius * self.radius);
        let discriminant = (h * h) - (a * c);
        if discriminant < 0.0 {
            // A negative discriminant means there are no real solutions for
            // t, so the ray does not intersect the sphere.
            return None;
        }
        let discr_root = discriminant.sqrt();
        // Compute the intersection times and check whether they are within
        // the provided interval, starting with the nearer point:
        let time_near = (h - discr_root) / a;
        let time = if ray_time.contains_exclusive(time_near) {
            time_near
        } else {
            let time_far = (h + discr_root) / a;
            if ray_time.contains_exclusive(time_far) {
                time_far
            } else {
                // Neither collision time is within the interval.
                return None;
            }
        };
        let hit_point = ray.at_time(time);
        let out_normal = (hit_point - self.center) / self.radius; // unit normal
        let (face, normal) = hit::face_normal(ray, out_normal);

        Some(HitInfo {
            hit_point,
            normal,
            time,
            face,
            material: self.material,
        })
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }
}
