//! Cameras responsible for casting rays and rendering images.

use std::path::Path;

use crate::hit::Hit;
use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// Camera properties used to construct a camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParams {
    /// The width of the output image in pixels.
    pub image_width: u32,
    /// The aspect ratio of the viewport and output image.
    pub aspect_ratio: f64,
    /// The camera's position in world units.
    pub position: Vec3,
    /// The height of the viewport in world units.
    pub viewport_height: f64,
    /// The distance from the camera to the viewport in world units.
    pub focal_length: f64,
}

/// Default camera parameters.
static DEFAULT_PARAMS: CameraParams = CameraParams {
    image_width: 800,
    aspect_ratio: 16.0 / 9.0,
    position: Vec3::new(),
    viewport_height: 2.0,
    focal_length: 1.0,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// The width of the output image in pixels.
    image_width: u32,
    /// The height of the output image in pixels
    image_height: u32,
    /// The camera's position in world units.
    position: Vec3,
    /// The center of the top-left viewport pixel in world units.
    first_pixel: Vec3,
    /// The horizontal displacement between viewport pixels.
    step_u: Vec3,
    /// The vertical displacement between viewport pixels.
    step_v: Vec3,
}

impl Camera {
    /// Construct a new camera with the default parameters.
    pub fn new() -> Self {
        Self::with_parameters(&DEFAULT_PARAMS)
    }

    /// Construct a new camera with the given parameters.
    pub fn with_parameters(params: &CameraParams) -> Self {
        let &CameraParams {
            image_width,
            aspect_ratio,
            position,
            viewport_height,
            focal_length,
        } = params;

        let image_height = (image_width as f64 / aspect_ratio) as u32;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        // Spacial coordinates: +X is right, +Y is up, +Z points out of the screen.
        // Viewport coordinates: +U is right, +V is down.

        // Create vectors for the axes of the viewport in terms of spacial
        // coordinates.
        let viewport_u: Vec3 = Vec3([viewport_width, 0.0, 0.0]);
        let viewport_v: Vec3 = Vec3([0.0, -viewport_height, 0.0]);
        // Compute the per-pixel step size for each viewport axis.
        let step_u = viewport_u / image_width as f64;
        let step_v = viewport_v / image_height as f64;
        // Compute the origin (top-left) of the viewport and the center of the
        // first pixel.
        let viewport_depth = Vec3([0.0, 0.0, -focal_length]);
        let viewport_origin = position + viewport_depth - (0.5 * viewport_u) - (0.5 * viewport_v);
        let first_pixel = viewport_origin + 0.5 * (step_u + step_v);

        Self {
            image_width,
            image_height,
            position,
            first_pixel,
            step_u,
            step_v,
        }
    }

    /// Render the given objects using this camera, saving the resulting image
    /// at the given path.
    pub fn render(&self, world: &impl Hit, path: impl AsRef<Path>) {
        let mut img = RgbImage::new(self.image_width, self.image_height);

        for row in 0..self.image_height {
            for col in 0..self.image_width {
                // Construct a ray travelling from the camera through the center
                // of the current pixel:
                let pixel_center =
                    self.first_pixel + (self.step_u * col as f64) + (self.step_v * row as f64);
                let ray_direction = pixel_center - self.position;
                let ray = Ray::new(pixel_center, ray_direction);
                // Compute the ray's color and write it to the image buffer:
                let px_color = ray_color(&ray, world);
                img.put_pixel(col, row, px_color);
            }
        }

        img.save(path).unwrap();
    }
}

/// Returns the pixel color associated with a given ray.
fn ray_color(ray: &Ray, world: &impl Hit) -> Rgb<u8> {
    // Check for collisions with objects in the scene:
    if let Some(info) = world.hit(ray, &(0.0, f64::INFINITY).into()) {
        // Scale the unit normal and convert it into a color:
        let normal_scaled = 0.5 * (info.normal + Vec3([1.0, 1.0, 1.0]));
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

/// Converts a vector representing a color into an RGB value.
///
/// Assumes that the provided vector's components are each in the interval
/// [0.0, 1.0]. If not, the resulting color's components will still be clamped
/// to within [0, 255], but it may result in odd visuals.
fn to_color(v: Vec3) -> Rgb<u8> {
    Rgb(v.0.map(|component| (component * 255.0) as u8))
}
