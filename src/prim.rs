//! Geometric primitives.

use std::fmt::{self, Debug};

use itertools::Itertools;

use crate::bound::{BoundingBox, Interval};
use crate::camera;
use crate::collections::ObjList;
use crate::hit::{self, Hit, HitInfo};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A spherical collider.
pub struct Sphere<'mat> {
    center: Vec3,
    radius: f64,
    material: &'mat (dyn Material + Sync),
    bound: BoundingBox,
}

impl<'mat> Sphere<'mat> {
    /// Constructs a new sphere at a given center with a given radius, whose
    /// surface is a given material.
    ///
    /// Panics if `radius` is less than or equal to `0.0`.
    pub fn new(center: Vec3, radius: f64, material: &'mat (dyn Material + Sync)) -> Self {
        assert!(radius > 0.0);
        let center_to_bbox_corner = Vec3([radius; 3]);
        let bound = BoundingBox::from_corners(
            &(center - center_to_bbox_corner),
            &(center + center_to_bbox_corner),
        );
        Self {
            center,
            radius,
            material,
            bound,
        }
    }
}

impl Debug for Sphere<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sphere")
            .field("center", &self.center)
            .field("radius", &self.radius)
            .field("bound", &self.bound)
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
        let unit_normal = (hit_point - self.center) / self.radius;
        let (face, normal) = hit::face_normal(ray, unit_normal);

        Some(HitInfo {
            hit_point,
            normal,
            time,
            face,
            material: self.material,
        })
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bound
    }
}

/// A parallelogram ("quadrilateral") collider.
pub struct Quad<'mat> {
    /// The starting vertex of the quad, as a point.
    origin: Vec3,
    /// The first extend of the quad, from `position` to the first adjacent
    /// vertex.
    u: Vec3,
    /// The second extent of the quad, from `position` to the second adjacent
    /// vertex.
    v: Vec3,
    /// The unit normal of the quad.
    unit_normal: Vec3,
    /// Precomputed term to assist with collision detection.
    w: Vec3,
    /// The surface material of this quad.
    material: &'mat (dyn Material + Sync),
    /// The bounding box around this quad.
    bound: BoundingBox,
}

impl<'mat> Quad<'mat> {
    /// Constructs a new quad with the given origin and span, and a given
    /// material.
    ///
    /// `origin` is the location of one vertex in world coordinates.
    /// `span` is the offset from `origin` to its adjacent vertices. The front
    /// face of the quad will be the face where the span vectors are in
    /// clockwise order.
    pub fn new(origin: Vec3, span: (Vec3, Vec3), material: &'mat (dyn Material + Sync)) -> Self {
        let (u, v) = span;
        let n = u.cross(&v);
        let w = n / n.dot(&n);
        let unit_normal = n.normalized();
        let bound_diag1 = BoundingBox::from_corners(&origin, &(origin + u + v));
        let bound_diag2 = BoundingBox::from_corners(&(origin + u), &(origin + v));
        let mut bound = BoundingBox::enclosing(&bound_diag1, &bound_diag2);
        // Ensure that the bounding box has a non-zero volume in the case that
        // this quad is axis-aligned.
        bound.pad_to_minimums();

        Self {
            origin,
            u,
            v,
            unit_normal,
            w,
            material,
            bound,
        }
    }

    /// Constructs a new axis-aligned cuboid with the given opposite corners.
    pub fn new_cuboid(
        corners: (&Vec3, &Vec3),
        material: &'mat (dyn Material + Sync),
    ) -> ObjList<'mat> {
        let minmax = |x1: f64, x2: f64| if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_x, max_x) = minmax(corners.0.x(), corners.1.x());
        let (min_y, max_y) = minmax(corners.0.y(), corners.1.y());
        let (min_z, max_z) = minmax(corners.0.z(), corners.1.z());
        let min_corner = Vec3([min_x, min_y, min_z]);
        let max_corner = Vec3([max_x, max_y, max_z]);
        let dx = Vec3::new(max_x - min_x, 0, 0);
        let dy = Vec3::new(0, max_y - min_y, 0);
        let dz = Vec3::new(0, 0, max_z - min_z);
        let sides: [Box<dyn Hit + Sync>; _] = [
            Box::new(Quad::new(min_corner, (dx, dy), material)),
            Box::new(Quad::new(min_corner, (dy, dz), material)),
            Box::new(Quad::new(min_corner, (dz, dx), material)),
            Box::new(Quad::new(max_corner, (-dx, -dy), material)),
            Box::new(Quad::new(max_corner, (-dy, -dz), material)),
            Box::new(Quad::new(max_corner, (-dz, -dx), material)),
        ];
        sides.into_iter().collect()
    }
}

impl<'mat> Debug for Quad<'mat> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Quad")
            .field("origin", &self.origin)
            .field("u", &self.u)
            .field("v", &self.v)
            .field("unit_normal", &self.unit_normal)
            .field("w", &self.w)
            .field("bound", &self.bound)
            .finish_non_exhaustive()
    }
}

impl<'mat> Hit for Quad<'mat> {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        // A point is on an infinite plane iff it is perpendicular to the plane's
        // normal (relative to another point on the plane), which happens only when
        // its dot product with the normal vector is 0.
        // We know that the quad's origin (Q) is always a point on the plane, so for
        // some point P and any normal vector n:
        //  n . (P - Q) = 0
        //  (n . P) - (n . Q) = 0
        //  n . P = n . Q
        // For some ray with a starting point r_0 and a direction vector d:
        //  P(t) = r_0 + dt
        // So:
        //  n . (r_0 + dt) = n . Q
        //  (n . r_0) + (n . dt) = n . Q
        //  n . dt = (n . Q) - (n . r_0)
        //  n . dt = n . (Q - r_0)
        //  (n . d)t = n . (Q - r_0)
        //  t = (n . (Q - r_0)) / (n . d)
        // This gives the intersection time of the ray with the infinite plane
        // (except when the rays is parallel to the plane, where (n . d) is zero and
        // we can just record a miss).
        // We can replace n with the unit normal here, since scaling n changes the
        // numerator and denominator by the same amount, leaving t unchanged.
        let denominator = self.unit_normal.dot(&ray.direction);
        const PARALLEL_THRESHOLD: f64 = 1e-8;
        if denominator.abs() < PARALLEL_THRESHOLD {
            // The ray is parallel to the plane, so it cannot intersect the quad.
            return None;
        }
        let time = self.unit_normal.dot(&(self.origin - ray.origin)) / denominator;
        if !ray_time.contains_inclusive(time) {
            return None;
        }
        let hit_point = ray.at_time(time);
        // However, the quad is only a section of the infinite plane.
        // Its vectors u and v span the plane; any point P intersecting the plane can
        // be expressed as a linear combination of u and v relative to the plane's
        // origin Q:
        //  P = Q + au + bv
        //  P - Q = au + bv
        // Let p := P - Q be the point on the plane relative to the plane's origin:
        //  p = au + bv
        // The points on the quad are those whose coefficients of that linear
        // combination are on the interval [0, 1].
        // To solve for a (or b), we can cross both sides by v (or u) to cancel the
        // other unknown, since any vector crossed with itself is the zero vector.
        //  v x p = v x (au + bv)
        //  v x p = a(v x u) + b(v x v)
        //  v x p = a(v x u)
        //  v x p = -a(u x v)
        // Let n be the normal vector n := u x v. Then:
        //  v x p = -an
        //  p x v = an
        // Similarly,
        //  u x p = b(u x v) = bn
        // We can't divide by vector quantities, so we dot both sides by n to convert
        // them to scalar quantities. (n is chosen because it will be convienent to
        // compute later.)
        //  n . (p x v) = n . (an)
        //  n . (p x v) = a(n . n)
        //  a = (n . (p x v)) / (n . n)
        //  a = (n / (n . n)) . (p x v)
        // Similarly:
        //  b = (n . (u x p)) / (n . n)
        //  b = (n / (n . n)) . (p x v)
        // We can factor out the common term w := (n / (n . n)) to reduce the number
        // of computations:
        //  a = w . (p x v)
        //  b = w . (u x p)
        let plane_hit_point = hit_point - self.origin;
        let a = self.w.dot(&plane_hit_point.cross(&self.v));
        if !(0.0..=1.0).contains(&a) {
            return None;
        }
        let b = self.w.dot(&self.u.cross(&plane_hit_point));
        if !(0.0..=1.0).contains(&b) {
            return None;
        }
        let (face, normal) = hit::face_normal(ray, self.unit_normal);

        Some(HitInfo {
            hit_point,
            normal,
            time,
            face,
            material: self.material,
        })
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bound
    }
}

/// A wrapper around a collider that translates it by the given amount.
pub struct Translate<H> {
    obj: H,
    offset: Vec3,
    bounds: BoundingBox,
}

impl<H: Hit> Translate<H> {
    /// Constructs a translated instance of a given object, offset by the given
    /// vector.
    pub fn new(obj: H, offset: Vec3) -> Self {
        let bounds = obj.bounds() + &offset;
        Self {
            obj,
            offset,
            bounds,
        }
    }
}

impl<H: Hit> Hit for Translate<H> {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        // Offset the ray in the opposite direction to the instance's offset
        // (convert from world coordinates to instance coordinates):
        let offset_ray = Ray {
            origin: ray.origin - self.offset,
            direction: ray.direction,
        };
        // Check if the translated ray hits the inner object:
        let mut hit_info = self.obj.hit(&offset_ray, ray_time)?;
        // If it does, then translate the hit point back to compensate for the ray's
        // offset (convert from instance coordinates to world coordinates):
        hit_info.hit_point += self.offset;
        // Translation does not affect the direction of the object's normals, so the
        // normal vector is returned unchanged.
        Some(hit_info)
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }
}

/// A wrapper around a collider that rotates it about the y-axis.
pub struct RotateY<H> {
    obj: H,
    sin_theta: f64,
    cos_theta: f64,
    bounds: BoundingBox,
}

impl<H: Hit> RotateY<H> {
    /// Converts a vector's x and z coordinates from world space to object space.
    const fn to_obj_space(x: f64, z: f64, sin_theta: f64, cos_theta: f64) -> (f64, f64) {
        (
            (x * cos_theta - z * sin_theta),
            (z * cos_theta + x * sin_theta),
        )
    }

    /// Converts a vector's x and z coordinates from object space to world space.
    const fn to_world_space(x: f64, z: f64, sin_theta: f64, cos_theta: f64) -> (f64, f64) {
        (
            (x * cos_theta + z * sin_theta),
            (z * cos_theta - x * sin_theta),
        )
    }

    /// Constructs an instance of the given object, rotated about the positive
    /// y-axis by a given angle (in degrees).
    pub fn new(obj: H, angle: f64) -> Self {
        let radians = camera::degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        // Examine all eight corners of the object's bounding box after it is rotated,
        // and construct a bounding box in the world coordinate system that encloses
        // it.
        let original_bounds = obj.bounds();
        let (x_min, x_max) = original_bounds.x().into();
        let (z_min, z_max) = original_bounds.z().into();
        let extrema = [x_min, z_min].into_iter().cartesian_product([x_max, z_max]);
        let mut x_bounds = Interval::empty();
        let mut z_bounds = Interval::empty();
        for (x, z) in extrema {
            let (rot_x, rot_z) = Self::to_obj_space(x, z, sin_theta, cos_theta);
            x_bounds.start = x_bounds.start.min(rot_x);
            x_bounds.end = x_bounds.end.max(rot_x);
            z_bounds.start = z_bounds.end.min(rot_z);
            z_bounds.end = z_bounds.end.max(rot_z);
        }
        let bounds = BoundingBox::new(x_bounds, original_bounds.y(), z_bounds);
        Self {
            obj,
            sin_theta,
            cos_theta,
            bounds,
        }
    }
}

impl<H: Hit> Hit for RotateY<H> {
    fn hit<'m>(&'m self, ray: &Ray, ray_time: &Interval) -> Option<HitInfo<'m>> {
        let to_obj_space = |mut v: Vec3| -> Vec3 {
            let (x, z) = Self::to_obj_space(v.x(), v.z(), self.sin_theta, self.cos_theta);
            v.0[0] = x;
            v.0[2] = z;
            v
        };
        let to_world_space = |mut v: Vec3| -> Vec3 {
            let (x, z) = Self::to_world_space(v.x(), v.z(), self.sin_theta, self.cos_theta);
            v.0[0] = x;
            v.0[2] = z;
            v
        };
        // Convert the ray from world space to object space.
        let new_origin = to_obj_space(ray.origin);
        let new_direction = to_obj_space(ray.direction);
        let obj_ray = Ray {
            origin: new_origin,
            direction: new_direction,
        };
        // Perform the inner hit check within object space.
        let mut hit_info = self.obj.hit(&obj_ray, ray_time)?;
        // Convert the resulting hit point and normal vector from object space to
        // world space.
        hit_info.hit_point = to_world_space(hit_info.hit_point);
        hit_info.normal = to_world_space(hit_info.normal);
        Some(hit_info)
    }

    fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit::Face;
    use crate::material::Lambertian;

    #[test]
    fn quad_ray_intersections() {
        static GREY: Lambertian = Lambertian::new(Vec3([0.6, 0.6, 0.6]), 1.0);
        // Axis-aligned quad in the z = -2 plane.
        let quad = Quad::new(
            Vec3::new(-1, -1, -2),
            (Vec3::new(2, 0, 0), Vec3::new(0, 2, 0)),
            &GREY,
        );
        assert!(quad.bounds().x().size() > 0.0);
        assert!(quad.bounds().y().size() > 0.0);
        assert!(quad.bounds().z().size() > 0.0);
        // Axis-aligned ray travelling in the -z direction.
        let ray = Ray {
            origin: Vec3::new(0, 0, 5),
            direction: Vec3::new(0, 0, -1),
        };
        let Some(hit_info) = quad.hit(&ray, &Interval::universal()) else {
            panic!("failed to hit quad");
        };
        assert_eq!(hit_info.hit_point, Vec3::new(0, 0, -2));
        assert_eq!(hit_info.time, 7.0);
        assert_eq!(hit_info.face, Face::Front);
        assert_eq!(hit_info.normal, Vec3::new(0, 0, 1));
    }
}
