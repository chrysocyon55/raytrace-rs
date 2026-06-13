// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec3;

use crate::camera::Camera;
use crate::material::{Lambertian, Metal};
use crate::sphere::Sphere;
use crate::vec3::Vec3;

fn main() {
    let camera = Camera::new();

    let matte_red = Lambertian::new(Vec3([0.95, 0.15, 0.05]), 0.8);
    let matte_green = Lambertian::new(Vec3([0.1, 0.9, 0.25]), 0.75);

    let steel = Metal::new(Vec3([0.3, 0.3, 0.3]), 0.6);
    let gold = Metal::new(Vec3([0.925, 0.875, 0.3]), 0.2);

    let world = vec![
        Sphere::new(Vec3([0.0, -201.5, -3.0]), 200.0, &matte_green),
        Sphere::new(Vec3([0.0, -0.025, -3.2]), 1.5, &matte_red),
        Sphere::new(Vec3([-3.0, -0.05, -3.0]), 1.5, &steel),
        Sphere::new(Vec3([3.0, -0.05, -3.0]), 1.5, &gold),
    ];

    camera.render(&world.as_slice(), "./output.png");
}
