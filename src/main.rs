// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod camera;
mod hit;
mod ray;
mod sphere;
mod vec3;

use crate::camera::Camera;
use crate::sphere::Sphere;
use crate::vec3::Vec3;

fn main() {
    let camera = Camera::new();

    let world = vec![
        Sphere::new(Vec3([0.0, 0.0, -1.0]), 0.5),
        Sphere::new(Vec3([0.0, -100.5, -1.0]), 100.0),
    ];

    camera.render(&world.as_slice(), "./output.png");
}
