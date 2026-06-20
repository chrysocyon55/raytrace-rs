//! Cameras responsible for casting rays and rendering images.

use std::io::{self, Write};
use std::path::Path;

use image::{Rgb, RgbImage};
use rand::RngExt;

use crate::hit::Hit;
use crate::ray::Ray;
use crate::vec3::{ColorVec3, Vec3};

/// Camera properties used to construct a camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParams {
    /// Width of the output image in pixels.
    pub image_width: u32,
    /// The aspect ratio of the viewport and output image.
    pub aspect_ratio: f64,
    /// The camera's position in world units.
    pub position: Vec3,
    /// The coordinate the camera is pointed at.
    pub view_target: Vec3,
    /// Vertical field of view, in degrees.
    pub vertical_fov: f64,
    /// The maximum angular variation of rays due to defocus blur, in degrees.
    pub defocus_angle: f64,
    /// The distance the camera is focusing on, where no blur wil occur.
    pub focus_dist: f64,
    /// The number of samples to perform per pixel.
    pub samples: usize,
    /// Maximum recursion depth for bounce light.
    pub max_depth: usize,
}

/// Default camera parameters.
static DEFAULT_PARAMS: CameraParams = CameraParams {
    image_width: 1920,
    aspect_ratio: 16.0 / 9.0,
    position: Vec3([0.0, 0.0, 0.0]),
    view_target: Vec3([0.0, 0.0, -1.0]),
    vertical_fov: 90.0,
    defocus_angle: 0.0,
    focus_dist: 1.0,
    samples: 30,
    max_depth: 15,
};

impl Default for CameraParams {
    fn default() -> Self {
        DEFAULT_PARAMS
    }
}

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
    /// The maximum defocus blur basis vectors.
    defocus_basis: Option<(Vec3, Vec3)>,
    /// The number of samples to perform per pixel.
    samples: usize,
    /// Maximum recursion depth for bounce light.
    max_depth: usize,
}

const fn degrees_to_radians(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
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
            view_target,
            vertical_fov,
            defocus_angle,
            focus_dist,
            samples,
            max_depth,
        } = params;
        assert!(aspect_ratio > 0.0, "aspect ratio must be positive");
        assert!(vertical_fov > 0.0, "vertical FOV must be positive");
        assert!(defocus_angle >= 0.0, "defocus angle cannot be negative");
        assert!(samples > 0, "must have at least one sample per pixel");

        let image_height = (image_width as f64 / aspect_ratio) as u32;

        // Compute viewport size.
        let half_height_scale = (degrees_to_radians(vertical_fov) / 2.0).tan();
        let viewport_height = 2.0 * half_height_scale * focus_dist;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        // Compute camera basis vectors: +u is right, +v is up, and +w is out
        // of the screen.
        // +w is opposite to the direction the camera is pointing.
        let basis_w = (position - view_target).normalized();
        // +u is right-handed relative to the world's +Y (up) with +w.
        let basis_u = Vec3([0.0, 1.0, 0.0]).cross(&basis_w).normalized();
        // +v is right-handed relative to +w with +u.
        let basis_v = basis_w.cross(&basis_u);

        // Viewport coordinates: +U is right, +V is down.
        // Create vectors for the axes of the viewport in terms of world
        // coordinates:
        let viewport_u: Vec3 = viewport_width * basis_u;
        let viewport_v: Vec3 = -viewport_height * basis_v;
        // Compute the per-pixel step size for each viewport axis.
        let step_u = viewport_u / image_width as f64;
        let step_v = viewport_v / image_height as f64;
        // Compute the origin (top-left) of the viewport.
        let viewport_depth = -focus_dist * basis_w;
        let viewport_origin = position + viewport_depth - (0.5 * viewport_u) - (0.5 * viewport_v);

        // Compute the basis for the defocus disk.
        let defocus_basis = if defocus_angle == 0.0 {
            None
        } else {
            let defocus_radius = focus_dist * (degrees_to_radians(defocus_angle / 2.0)).tan();
            let defocus_u = basis_u * defocus_radius;
            let defocus_v = basis_v * defocus_radius;
            Some((defocus_u, defocus_v))
        };

        Self {
            image_width,
            image_height,
            position,
            viewport_origin,
            step_u,
            step_v,
            defocus_basis,
            samples,
            max_depth,
        }
    }

    /// Render the given objects using this camera, saving the resulting image
    /// at the given path.
    pub fn render<H: Hit + ?Sized>(&self, world: &H, path: impl AsRef<Path>) {
        let mut rng = rand::rng();
        // Computes the average color of a pixel at the given index by
        // randomly sampling with multiple rays and averaging their resulting
        // colors.
        let mut pixel_color = |px_row, px_col| -> Rgb<u8> {
            let mut total_color = ColorVec3::new();
            let px_offset_u = px_col as f64 * self.step_u;
            let px_offset_v = px_row as f64 * self.step_v;
            let px_topleft = self.viewport_origin + px_offset_u + px_offset_v;
            // Sample within each pixel multiple times:
            for _ in 0..self.samples {
                // Compute the starting point from the camera, applying a
                // random defocus blur if needed:
                let ray_origin = match self.defocus_basis {
                    None => self.position,
                    Some((defocus_u, defocus_v)) => {
                        let blur = Vec3::random_in_disk();
                        self.position + blur.0[0] * defocus_u + blur.0[1] * defocus_v
                    }
                };
                // Compute a random point within the pixel:
                let rand_u = rng.random::<f64>() * self.step_u;
                let rand_v = rng.random::<f64>() * self.step_v;
                let samp_position = px_topleft + rand_u + rand_v;
                // Fire a ray from the camera through that point:
                let ray_direction = samp_position - ray_origin;
                let ray = Ray {
                    origin: ray_origin,
                    direction: ray_direction,
                };
                total_color += ray_color(&ray, self.max_depth, world);
            }
            let avg_color = total_color / self.samples as f64;
            let corrected_color = linear_to_gamma(avg_color);
            to_rgb(corrected_color)
        };

        println!("Starting render.");
        let mut img = RgbImage::new(self.image_width, self.image_height);
        // Iterate over each pixel in the image and compute its color:
        for row in 0..self.image_height {
            print!("Rendering scanline {}/{}...\r", row + 1, self.image_height);
            io::stdout().flush().unwrap();
            for col in 0..self.image_width {
                let color = pixel_color(row, col);
                img.put_pixel(col, row, color);
            }
        }
        img.save(&path).unwrap();
        // Long blank space to overwrite the "rendering scanline" text above
        println!("Render complete.                         ");
        println!("Saved output image as {}", &path.as_ref().display());
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the pixel color associated with a given ray as a color vector.
fn ray_color<H: Hit + ?Sized>(ray: &Ray, depth: usize, world: &H) -> ColorVec3 {
    if depth == 0 {
        // If the maximum recursion depth has been reached, the ray "fizzles"
        // and returns pure black.
        const BLACK: Vec3 = Vec3([0.0; _]);
        return BLACK;
    }

    // Check for collisions with objects in the scene:
    // We ignore collisions that happen almost immediately, since they are
    // likely rays incorrectly intersecting with the surfaces they just
    // bounced off of, due to floating point imprecision. This prevents
    // "shadow acne", where such rays would produce isolated dark pixels due
    // to repeated in-place collisions.
    if let Some(collision) = world.hit(ray, &(0.001, f64::INFINITY).into()) {
        // Determine whether the ray is absorbed or reflected:
        if let Some((color, scattered_ray)) = collision.material.scatter(ray, &collision) {
            return color * ray_color(&scattered_ray, depth - 1, world);
        } else {
            // Absorbed rays cause the area to appear perfectly black.
            const BLACK: ColorVec3 = Vec3([0.0; _]);
            return BLACK;
        }
    }

    // If the ray doesn't hit any objects, draw the sky using a gradient.
    // Lerp from white to blue as the y component of the ray increases:
    let unit_dir = ray.direction.normalized();
    const SKY_WHITE: Vec3 = Vec3([1.0, 1.0, 1.0]);
    const SKY_BLUE: Vec3 = Vec3([0.5, 0.7, 1.0]);
    let blend = 0.5 * (unit_dir.y() + 1.0);
    (1.0 - blend) * SKY_WHITE + blend * SKY_BLUE
}

/// Performs gamma correction on a color vector.
///
/// Apparent brightness is not linear: a linear average of black and white
/// appears much darker than a neutral grey. This function compensates for
/// this effect, producing a more natural-looking transition from light to
/// dark.
fn linear_to_gamma(v: ColorVec3) -> ColorVec3 {
    Vec3(v.0.map(|component| {
        if component > 0.0 {
            component.sqrt()
        } else {
            0.0
        }
    }))
}

/// Converts a real color vector into an RGB value.
///
/// The vector's components are intended to be within the interval [0.0, 1.0],
/// and will be clamped otherwise.
fn to_rgb(v: ColorVec3) -> Rgb<u8> {
    Rgb(v.0.map(|component| (component * 255.0) as u8))
}
