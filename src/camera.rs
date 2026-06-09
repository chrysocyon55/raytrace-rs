//! Cameras responsible for casting rays and rendering images.

use std::path::Path;

use crate::hit::Hit;
use crate::ray::Ray;
use crate::vec3::Vec3;

use image::{Rgb, RgbImage};

/// A `Vec3` used to store real-valued color data.
pub type ColorVec3 = Vec3;

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
    /// The minimum number of samples to perform per pixel.
    /// Samples are taken as an evenly-spaced grid of N by N points, so the
    /// actual number of samples per pixel will be the smallest square number
    /// that is greater or equal to this minimum.
    pub min_samples: usize,
}

/// Default camera parameters.
static DEFAULT_PARAMS: CameraParams = CameraParams {
    image_width: 800,
    aspect_ratio: 16.0 / 9.0,
    position: Vec3::new(),
    viewport_height: 2.0,
    focal_length: 1.0,
    min_samples: 16,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// The width of the output image in pixels.
    image_width: u32,
    /// The height of the output image in pixels
    image_height: u32,
    /// The camera's position in world units.
    position: Vec3,
    /// The top-left corner of the viewport, in world units.
    viewport_origin: Vec3,
    /// The horizontal displacement between viewport pixels, in world units.
    step_u: Vec3,
    /// The vertical displacement between viewport pixels, in world units.
    step_v: Vec3,
    /// The side length of the square sample grid, in samples.
    samples_per_dir: usize,
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
            min_samples,
        } = params;
        assert!(aspect_ratio > 0.0, "aspect ratio must be positive");
        assert!(viewport_height > 0.0, "viewport height must be positive");
        assert!(focal_length > 0.0, "focal length must be positive");
        assert!(min_samples > 0, "must have at least one sample per pixel");

        let image_height = (image_width as f64 / aspect_ratio) as u32;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        // Spacial coordinates: +X is right, +Y is up, +Z points out of the screen.
        // Viewport coordinates: +U is right, +V is down.

        // Create vectors for the axes of the viewport in terms of world
        // coordinates.
        let viewport_u: Vec3 = Vec3([viewport_width, 0.0, 0.0]);
        let viewport_v: Vec3 = Vec3([0.0, -viewport_height, 0.0]);
        // Compute the per-pixel step size for each viewport axis.
        let step_u = viewport_u / image_width as f64;
        let step_v = viewport_v / image_height as f64;
        // Compute the origin (top-left) of the viewport.
        let viewport_depth = Vec3([0.0, 0.0, -focal_length]);
        let viewport_origin = position + viewport_depth - (0.5 * viewport_u) - (0.5 * viewport_v);

        let samples_per_dir = (min_samples + 1).isqrt();

        Self {
            image_width,
            image_height,
            position,
            viewport_origin,
            step_u,
            step_v,
            samples_per_dir,
        }
    }

    /// Render the given objects using this camera, saving the resulting image
    /// at the given path.
    pub fn render(&self, world: &impl Hit, path: impl AsRef<Path>) {
        let sample_step_u = self.step_u / (self.samples_per_dir as f64);
        let sample_step_v = self.step_v / (self.samples_per_dir as f64);
        let samples_per_px = self.samples_per_dir * self.samples_per_dir;

        // Computes the average color of a pixel at the given index by
        // sampling with multiple rays and averaging their resulting colors.
        let pixel_color = |px_row, px_col| -> Rgb<u8> {
            let mut total_color = ColorVec3::new();
            let px_offset_u = px_col as f64 * self.step_u;
            let px_offset_v = px_row as f64 * self.step_v;
            let px_topleft = self.viewport_origin + px_offset_u + px_offset_v;
            // Sample within each pixel multiple times, in a square grid:
            for samp_row in 0..self.samples_per_dir {
                let samp_offset_v = (samp_row as f64 + 0.5) * sample_step_v;
                for samp_col in 0..self.samples_per_dir {
                    let samp_offset_u = (samp_col as f64 + 0.5) * sample_step_u;
                    let samp_position = px_topleft + samp_offset_v + samp_offset_u;
                    let ray_direction = samp_position - self.position;
                    let ray = Ray::new(self.position, ray_direction);
                    total_color += ray_color(&ray, world);
                }
            }
            let avg_color = total_color / samples_per_px as f64;
            to_rgb(avg_color)
        };

        let mut img = RgbImage::new(self.image_width, self.image_height);
        // Iterate over each pixel in the image and compute its color:
        for row in 0..self.image_height {
            for col in 0..self.image_width {
                let color = pixel_color(row, col);
                img.put_pixel(col, row, color);
            }
        }
        img.save(path).unwrap();
    }
}

/// Returns the pixel color associated with a given ray as a color vector.
fn ray_color(ray: &Ray, world: &impl Hit) -> ColorVec3 {
    // Check for collisions with objects in the scene:
    if let Some(info) = world.hit(ray, &(0.0, f64::INFINITY).into()) {
        // Scale the unit normal's components to the range 0.0-1.0].
        return 0.5 * (info.normal + Vec3([1.0, 1.0, 1.0]));
    }

    // If the ray doesn't hit any objects, draw a gradient background.
    // Lerp from white to blue as the y component of the ray increases:
    let unit_dir = ray.direction.normalized();
    const WHITE: Vec3 = Vec3([1.0, 1.0, 1.0]);
    const BLUE: Vec3 = Vec3([0.5, 0.7, 1.0]);
    let blend = 0.5 * (unit_dir.y() + 1.0);
    (1.0 - blend) * WHITE + blend * BLUE
}

/// Converts a real color vector into an RGB value.
///
/// The vector's components are intended to be within the interval [0.0, 1.0],
/// and will be clamped otherwise.
fn to_rgb(v: ColorVec3) -> Rgb<u8> {
    Rgb(v.0.map(|component| (component * 255.0) as u8))
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
