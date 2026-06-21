// Adapted from the "Ray Tracing in One Weekend" series, at
// https://raytracing.github.io/.

mod bound;
mod camera;
mod hit;
mod material;
mod ray;
mod scene;
mod sphere;
mod vec3;

use rand::{self, RngExt};

use crate::camera::{Camera, CameraParams};
use crate::hit::Hit;
use crate::material::{Dielectric, Lambertian, Material, Metal};
use crate::scene::SceneTree;
use crate::sphere::Sphere;
use crate::vec3::Vec3;

/// Allocate a new static reference on the heap.
fn new_static<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

fn main() {
    const RENDER_FULL_SCENE: bool = true;

    let camera = Camera::with_parameters(&CameraParams {
        position: Vec3([13.0, 2.0, 4.0]),
        view_target: Vec3([0.0, 0.0, 0.0]),
        vertical_fov: 20.0,
        defocus_angle: 0.4,
        focus_dist: 11.0,
        samples: if RENDER_FULL_SCENE { 500 } else { 50 },
        max_depth: if RENDER_FULL_SCENE { 50 } else { 25 },
        ..Default::default()
    });

    let mut world: Vec<Box<dyn Hit + Sync>> = vec![];
    // Add a large sphere to mimic a ground plane.
    static GROUND_MAT: Lambertian = Lambertian::new(Vec3([0.5, 0.5, 0.5]), 0.9);
    let ground = Sphere::new((0.0, -1000.0, 0.0).into(), 1000.0, &GROUND_MAT);
    world.push(Box::new(ground));

    if RENDER_FULL_SCENE {
        // Populate the world with a grid of spheres, randomly adjusting their
        // positions for a less uniform appearance.
        let mut rng = rand::rng();
        for s_row in -11..=11 {
            for s_col in -11..=11 {
                let center = Vec3([
                    s_row as f64 + 0.9 * rng.random::<f64>(),
                    0.2,
                    s_col as f64 + 0.9 * rng.random::<f64>(),
                ]);
                if (center - Vec3([4.0, 0.2, 0.0])).length() < 1.0 {
                    // Skip any spheres that are too close to the origin, to make
                    // room for the larger spheres later.
                    continue;
                }
                // Pick a material at random.
                let material: &'static (dyn Material + Sync) = match rng.random::<f64>() {
                    (0.0..=0.8) => {
                        let albedo = Vec3::random() * Vec3::random();
                        new_static(Lambertian::new(albedo, rng.random_range(0.8..=1.0)))
                    }
                    (0.8..=0.95) => {
                        let albedo = Vec3::random_range(0.5, 1.0);
                        let fuzz = rng.random_range(0.01..=0.5);
                        new_static(Metal::new(albedo, fuzz))
                    }
                    _ => new_static(Dielectric::new(1.5)),
                };
                let sphere = Sphere::new(center, 0.2, material);
                world.push(Box::new(sphere));
            }
        }
    }

    // Add center subject spheres.
    static MATTE_RED: Lambertian = Lambertian::new(Vec3([0.95, 0.15, 0.05]), 0.8);
    static GOLD: Metal = Metal::new(Vec3([0.925, 0.875, 0.3]), 0.2);
    static GLASS: Dielectric = Dielectric::new(1.5);
    let center_spheres: [Box<dyn Hit + Sync>; _] = [
        Box::new(Sphere::new(Vec3([-4.0, 1.0, 0.0]), 1.0, &GOLD)),
        Box::new(Sphere::new(Vec3([0.0, 1.0, 0.0]), 1.0, &MATTE_RED)),
        Box::new(Sphere::new(Vec3([4.0, 1.0, 0.0]), 1.0, &GLASS)),
    ];
    world.extend(center_spheres);

    let scene = SceneTree::new(world);
    camera.render(&scene, "./output.png");
}
