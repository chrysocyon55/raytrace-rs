// Adapted from the "Ray Tracing in One Weekend" series, at
// https://raytracing.github.io/.

mod bound;
mod bvh;
mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec3;

use crate::camera::{Camera, CameraParams};
use crate::hit::{Hit, SceneList};
use crate::material::{Dielectric, Lambertian, Metal};
use crate::sphere::Sphere;
use crate::vec3::Vec3;

fn main() {
    let camera = Camera::with_parameters(&CameraParams {
        position: Vec3([-2.0, 2.0, 1.0]),
        view_target: Vec3([0.0, 0.0, -0.5]),
        vertical_fov: 30.0,
        ..Default::default()
    });

    static MATTE_RED: Lambertian = Lambertian::new(Vec3([0.95, 0.15, 0.05]), 0.8);
    static MATTE_GREEN: Lambertian = Lambertian::new(Vec3([0.1, 0.9, 0.25]), 0.75);

    static GOLD: Metal = Metal::new(Vec3([0.925, 0.875, 0.3]), 0.2);

    static GLASS: Dielectric = Dielectric::new(0.75);

    let world: [Box<dyn Hit>; _] = [
        Box::new(Sphere::new(Vec3([0.0, -100.5, -1.0]), 100.0, &MATTE_GREEN)),
        Box::new(Sphere::new(Vec3([0.0, 0.0, -0.25]), 0.5, &MATTE_RED)),
        Box::new(Sphere::new(Vec3([-1.0, -0.05, 0.0]), 0.5, &GLASS)),
        Box::new(Sphere::new(Vec3([1.0, -0.05, 0.0]), 0.5, &GOLD)),
    ];
    let scene = world.into_iter().collect::<SceneList>();

    camera.render(&scene, "./output.png");
}
