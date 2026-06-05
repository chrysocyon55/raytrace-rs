// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod ray;
mod vec3;

use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// Returns the pixel color associated with a given ray.
fn ray_color(ray: &Ray) -> Rgb<u8> {
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

    // Spacial coordinates: +X is right, +Y is up, +Z is into the screen.
    // Viewport coordinates: +U is right, +V is down.

    // Viewport scaling.
    const VIEWPORT_HEIGHT: f64 = 2.0;
    const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64;

    // Camera settings.
    const FOCAL_LENGTH: f64 = 1.0;
    const CAMERA_CENTER: Vec3 = Vec3::new();

    // Create vectors for the axes of the viewport in terms of spacial
    // coordinates.
    const VIEWPORT_U: Vec3 = Vec3::from_components(VIEWPORT_WIDTH, 0.0, 0.0);
    const VIEWPORT_V: Vec3 = Vec3::from_components(0.0, -VIEWPORT_HEIGHT, 0.0);
    // Compute the per-pixel step size for each viewport axis.
    let step_u = VIEWPORT_U / IMAGE_WIDTH as f64;
    let step_v = VIEWPORT_V / IMAGE_HEIGHT as f64;
    // Compute the origin (top-left) of the viewport and the center of the
    // first pixel.
    let viewport_origin = CAMERA_CENTER
        - Vec3::from_components(0.0, 0.0, FOCAL_LENGTH)
        - VIEWPORT_U / 2.0
        - VIEWPORT_V / 2.0;
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
