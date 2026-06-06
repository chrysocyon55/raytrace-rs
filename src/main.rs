// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod ray;
mod vec3;

use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// Converts a vector representing a color into an RGB value.
///
/// Assumes that the provided vector's components are each in the interval
/// [0.0, 1.0]. If not, the resulting color's components will still be clamped
/// to within [0, 255], but it may result in odd visuals.
fn to_color(v: Vec3) -> Rgb<u8> {
    Rgb(v.0.map(|component| (component * 255.0) as u8))
}

/// Determines whether the given ray hits a sphere with a given center and
/// radius.
///
/// If the given ray hits the sphere, returns `Some` containing the time
/// at which the ray intersects the sphere. Otherwise, if the ray does not hit
/// the sphere, returns `None`.
fn hits_sphere(center: Vec3, radius: f64, ray: &Ray) -> Option<f64> {
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
    //      t = (-b - sqrt(b^2 - 4ac)) / 2a
    // Let h = d . (C - Q), then b = -2h:
    //      t = (2h - sqrt((-2h)^2 - 4ac)) / 2a
    //      t = (2h - 2*sqrt(h^2 - ac)) / 2a
    //      t = (h - sqrt(h^2 - ac)) / a
    let qc = center - ray.origin;
    let a = ray.direction.dot(&ray.direction);
    let h = ray.direction.dot(&qc);
    let c = qc.dot(&qc) - radius * radius;
    let discriminant = h * h - a * c;
    if discriminant < 0.0 {
        None
    } else {
        // Return the closer intersection time, since that's the side of the
        // sphere facing the camera.
        Some((h - discriminant.sqrt()) / a)
    }
}

/// Returns the pixel color associated with a given ray.
fn ray_color(ray: &Ray) -> Rgb<u8> {
    const SPHERE_CENTER: Vec3 = Vec3([0.0, 0.0, -1.0]);
    const RADIUS: f64 = 0.5;

    // Test for object collisions:
    if let Some(t) = hits_sphere(SPHERE_CENTER, RADIUS, ray) {
        // Compute the unit normal vector at the intersection point:
        let normal = (ray.at_time(t) - SPHERE_CENTER).normalized();
        let normal_scaled = 0.5 * (normal + Vec3([1.0, 1.0, 1.0]));
        let normal_color = to_color(normal_scaled);
        return normal_color;
    }
    // If the ray doesn't hit any objects, draw a gradient background.
    // Lerp from white to blue as the y component of the ray increases:
    let unit_dir = ray.direction.normalized();
    const WHITE: Vec3 = Vec3([1.0, 1.0, 1.0]);
    const BLUE: Vec3 = Vec3([0.5, 0.7, 1.0]);
    let blend = 0.5 * (unit_dir.y() + 1.0);
    let bg_color_vector = (1.0 - blend) * WHITE + blend * BLUE;
    to_color(bg_color_vector)
}

fn main() {
    // Image size settings.
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
    assert!(IMAGE_HEIGHT > 0, "image height must be at least 1 px");

    // Viewport scaling.
    const VIEWPORT_HEIGHT: f64 = 2.0;
    const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * (IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64);

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
    let viewport_origin = CAMERA_CENTER + VIEWPORT_DEPTH - (0.5 * VIEWPORT_U) - (0.5 * VIEWPORT_V);
    let first_pixel = viewport_origin + 0.5 * (step_u + step_v);

    let mut img = RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    for row in 0..IMAGE_HEIGHT {
        for col in 0..IMAGE_WIDTH {
            // Construct a ray travelling from the camera through the center
            // of the current pixel:
            let pixel_center = first_pixel + (step_u * col as f64) + (step_v * row as f64);
            let ray_direction = pixel_center - CAMERA_CENTER;
            let ray = Ray::new(pixel_center, ray_direction);
            // Compute the ray's color and write it to the image buffer:
            let px_color = ray_color(&ray);
            img.put_pixel(col, row, px_color);
        }
    }

    img.save("./output.png").unwrap();
}
