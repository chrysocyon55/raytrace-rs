//! Surface materials for rendered objects.

use rand::{self, RngExt};

use crate::hit::HitInfo;
use crate::ray::Ray;
use crate::vec3::{ColorVec3, Vec3};

/// A trait for surface materials that scatter, reflect, or absorb light.
pub trait Material {
    /// Scatter the given light ray off of this surface according to its
    /// properties.
    ///
    /// If the ray is completely absorbed by the surface, returns `None`.
    /// Otherwise, returns `Some` containing the attenuated color and the
    /// direction of the scattered ray.
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> Option<(ColorVec3, Ray)>;
}

/// A Lambertian diffuse material that either absorbs or randomly scatters
/// light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lambertian {
    /// The color of the surface.
    albedo: ColorVec3,
    /// The probability that light rays are scattered in a random direction.
    /// Rays that aren't scattered will be absorbed by the surface.
    scatter_chance: f64,
}

impl Lambertian {
    /// Constructs a new Lambertian material with a given albedo and scatter
    /// chance.
    ///
    /// Panics if `scatter_chance` is not in the interval [0.0, 1.0].
    pub fn new(albedo: ColorVec3, scatter_chance: f64) -> Self {
        assert!((0.0..=1.0).contains(&scatter_chance));
        Self {
            albedo,
            scatter_chance,
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit_info: &HitInfo) -> Option<(ColorVec3, Ray)> {
        if rand::rng().random::<f64>() > self.scatter_chance {
            return None;
        }
        // Lambertian scattering is not uniformly random over the hemisphere,
        // it favors scattering near the surface's normal.
        let mut scatter_dir = hit_info.normal + Vec3::random_unit();
        // Ensure scatter_dir is not too small, which can occur when the
        // random unit vector happens to point opposite to the surface's unit
        // normal.
        if scatter_dir.is_near_zero() {
            // Very short scatter dirs intend to point nearly the same
            // direction as the surface normal, so just assign them the normal
            // vector.
            scatter_dir = hit_info.normal;
        }
        let scattered_ray = Ray::new(hit_info.hit_point, scatter_dir);
        Some((self.albedo, scattered_ray))
    }
}

/// A metal surface that reflects light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metal {
    // The color of the surface.
    albedo: ColorVec3,
}

impl Metal {
    /// Constructs a metal material with a given albedo.
    pub const fn new(albedo: ColorVec3) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> Option<(ColorVec3, Ray)> {
        let reflected = ray.direction.reflected_over(hit_info.normal);
        let reflected_ray = Ray::new(hit_info.hit_point, reflected);
        Some((self.albedo, reflected_ray))
    }
}
