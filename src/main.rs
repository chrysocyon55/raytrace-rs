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
        Sphere::new(Vec3([5.0, 2.5, -50.0]), 20.0),
        Sphere::new(Vec3([-3.0, -1.0, -7.0]), 2.0),
    ];

    camera.render(&world.as_slice(), "./output.png");
}
