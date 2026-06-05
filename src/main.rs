// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod ray;
mod vec3;

use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// Determines whether the given ray hits a sphere with a given center and
/// radius.
fn hits_sphere(center: Vec3, radius: f64, ray: &Ray) -> bool {
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
    let qc = center - ray.origin;
    let a = ray.direction.dot(&ray.direction);
    let b = -2.0 * ray.direction.dot(&qc);
    let c = qc.dot(&qc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
    // TODO: This implementation allows for negative values of t, so rays can
    // "hit" a sphere placed behind the camera. This isn't correct, but it
    // works for an initial implementation.
}

/// Returns the pixel color associated with a given ray.
fn ray_color(ray: &Ray) -> Rgb<u8> {
    // Test for object collisions:
    if hits_sphere(Vec3([0.0, 0.0, -1.0]), 0.5, ray) {
        return Rgb([240, 21, 56]);
    }

    // If the ray doesn't hit any objects, draw a gradient background.
    // Lerp from white to blue as the y component of the ray increases:
    let unit_dir = ray.direction.normalized();
    const WHITE: Vec3 = Vec3([1.0, 1.0, 1.0]);
    const BLUE: Vec3 = Vec3([0.5, 0.7, 1.0]);
    let blend = 0.5 * unit_dir.y() + 1.0;
    let color = (1.0 - blend) * WHITE + blend * BLUE;
    // Convert color scale from 0.0-1.0 to 0-255.
    Rgb(color.0.map(|component| (255.99 * component).round() as u8))
}

fn main() {
    // Image size settings.
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
    assert!(IMAGE_HEIGHT > 0, "image height must be at least 1 px");

    // Viewport scaling.
    const VIEWPORT_HEIGHT: f64 = 2.0;
    const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64;

    // Camera settings.
    const FOCAL_LENGTH: f64 = 1.0;
    const CAMERA_CENTER: Vec3 = Vec3::new();


    // Spacial coordinates: +X is right, +Y is up, +Z points out of the screen.
    // Viewport coordinates: +U is right, +V is down.

    // Create vectors for the axes of the viewport in terms of spacial
    // coordinates.
    const VIEWPORT_U: Vec3 = Vec3([VIEWPORT_WIDTH, 0.0, 0.0]);
    const VIEWPORT_V: Vec3 = Vec3([0.0, -VIEWPORT_HEIGHT, 0.0]);
    // Compute the per-pixel step size for each viewport axis.
    let step_u = VIEWPORT_U / IMAGE_WIDTH as f64;
    let step_v = VIEWPORT_V / IMAGE_HEIGHT as f64;
    // Compute the origin (top-left) of the viewport and the center of the
    // first pixel.
    const VIEWPORT_DEPTH: Vec3 = Vec3([0.0, 0.0, -FOCAL_LENGTH]);
    let viewport_origin = CAMERA_CENTER + VIEWPORT_DEPTH - VIEWPORT_U / 2.0 - VIEWPORT_V / 2.0;
    let first_pixel = viewport_origin + 0.5 * (step_u + step_v);

    let mut img = RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    for row in 0..IMAGE_HEIGHT {
        for col in 0..IMAGE_WIDTH {
            let pixel_center = first_pixel + (step_u * col as f64) + (step_v * row as f64);
            let ray_direction = pixel_center - CAMERA_CENTER;
            let ray = Ray::new(pixel_center, ray_direction);
            img.put_pixel(col, row, ray_color(&ray));
        }
    }

    img.save("./output.png").unwrap();
}
