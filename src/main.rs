// Adapted from the "Ray Tracing in One Weekend" series, at
// https://raytracing.github.io/.

mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec3;

use crate::camera::{Camera, CameraParams};
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

    let matte_red = Lambertian::new(Vec3([0.95, 0.15, 0.05]), 0.8);
    let matte_green = Lambertian::new(Vec3([0.1, 0.9, 0.25]), 0.75);

    // let steel = Metal::new(Vec3([0.3, 0.3, 0.3]), 0.6);
    let gold = Metal::new(Vec3([0.925, 0.875, 0.3]), 0.2);

    let glass = Dielectric::new(0.75);

    let world = &[
        Sphere::new(Vec3([0.0, -100.5, -1.0]), 100.0, &matte_green),
        Sphere::new(Vec3([0.0, 0.0, -0.25]), 0.5, &matte_red),
        Sphere::new(Vec3([-1.0, -0.05, 0.0]), 0.5, &glass),
        Sphere::new(Vec3([1.0, -0.05, 0.0]), 0.5, &gold),
    ];

    camera.render(world.as_slice(), "./output.png");
}
