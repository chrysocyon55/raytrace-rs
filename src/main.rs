// Adapted from "Ray Tracing in One Weekend", at https://raytracing.github.io/.

mod ray;
mod vec3;

use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// Returns the pixel color associated with a given ray.
fn ray_color(ray: &Ray) -> Rgb<u8> {
    // TODO: actual rendering logic.
    // For now, just return a nice shade of blue :)
    [10, 56, 192].into()
}

fn main() {
    // Image size settings.
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as u32;
    assert!(image_height > 0);

    let mut img = RgbImage::new(image_width, image_height);

    // Viewport scaling.
    let viewport_height = 2.0;
    let viewport_width = viewport_height * image_width as f64 / image_height as f64;

    // Camera settings.
    let focal_length = 1.0;
    let camera_center = Vec3::new();

    // Compute vectors along the axes of the viewport.
    let viewport_u = Vec3::from_components(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::from_components(-viewport_height, 0.0, 0.0);
    // Compute the step size for each viewport dimension.
    let step_u = viewport_u / image_width as f64;
    let step_v = viewport_v / image_height as f64;
    // Compute the origin (top-left) of the viewport and the center of the
    // first pixel.
    let viewport_origin = camera_center
        - Vec3::from_components(0.0, 0.0, focal_length)
        - viewport_u / 2.0
        - viewport_v / 2.0;
    let first_pixel = viewport_origin + 0.5 * (step_u + step_v);

    for row in 0..image_height {
        for col in 0..image_width {
            let pixel_center = first_pixel + (step_u * row as f64) + (step_v * col as f64);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(pixel_center, ray_direction);
            img.put_pixel(col, row, ray_color(&ray));
        }
    }

    img.save("./output.png").unwrap();
}
