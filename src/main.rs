// Adapted from the "Ray Tracing in One Weekend" series, at
// https://raytracing.github.io/.

mod bound;
mod camera;
mod collections;
mod hit;
mod material;
mod prim;
mod ray;
mod vec3;

use rand::{self, RngExt};

use crate::camera::{Camera, CameraParams};
use crate::collections::ObjTree;
use crate::hit::Hit;
use crate::material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use crate::prim::{Quad, Sphere};
use crate::vec3::{ColorVec3, Vec3};

/// Allocate a new static reference on the heap.
fn new_static<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

#[allow(unused)]
fn render_ball_field() {
    const RENDER_FULL_SCENE: bool = false;

    let camera = Camera::with_parameters(&CameraParams {
        position: Vec3([13.0, 2.0, 4.0]),
        view_target: Vec3([0.0, 0.0, 0.0]),
        vertical_fov: 20.0,
        defocus_angle: 0.4,
        focus_dist: 11.0,
        samples: if RENDER_FULL_SCENE { 500 } else { 50 },
        max_depth: if RENDER_FULL_SCENE { 100 } else { 25 },
        ..Default::default()
    });

    let mut world: Vec<Box<dyn Hit + Sync>> = vec![];
    // Add a large sphere to mimic a ground plane.
    let ground_mat: Lambertian = Lambertian::new(Vec3([0.5, 0.5, 0.5]), 0.9);
    let ground = Sphere::new((0.0, -1000.0, 0.0).into(), 1000.0, &ground_mat);
    world.push(Box::new(ground));

    if RENDER_FULL_SCENE {
        // Populate the world with a grid of spheres, randomly adjusting their
        // positions for a less uniform appearance.
        let mut rng = rand::rng();
        for s_row in -11..=11 {
            for s_col in -11..=11 {
                let center = Vec3([
                    s_row as f64 + 0.9 * rng.random::<f64>(),
                    0.2,
                    s_col as f64 + 0.9 * rng.random::<f64>(),
                ]);
                if (center - Vec3([4.0, 0.2, 0.0])).length() < 1.0 {
                    // Skip any spheres that are too close to the origin, to make
                    // room for the larger spheres later.
                    continue;
                }
                // Pick a material at random.
                let material: &'static (dyn Material + Sync) = match rng.random::<f64>() {
                    (0.0..=0.8) => {
                        let albedo = Vec3::random() * Vec3::random();
                        new_static(Lambertian::new(albedo, rng.random_range(0.8..=1.0)))
                    }
                    (0.8..=0.95) => {
                        let albedo = Vec3::random_range(0.5, 1.0);
                        let fuzz = rng.random_range(0.01..=0.5);
                        new_static(Metal::new(albedo, fuzz))
                    }
                    _ => new_static(Dielectric::new(1.5)),
                };
                let sphere = Sphere::new(center, 0.2, material);
                world.push(Box::new(sphere));
            }
        }
    }

    // Add center subject spheres.
    let matte_red: Lambertian = Lambertian::new(Vec3([0.95, 0.15, 0.05]), 0.8);
    let gold: Metal = Metal::new(Vec3([0.925, 0.875, 0.3]), 0.2);
    let glass: Dielectric = Dielectric::new(1.5);
    let center_spheres: [Box<dyn Hit + Sync>; _] = [
        Box::new(Sphere::new(Vec3([-4.0, 1.0, 0.0]), 1.0, &gold)),
        Box::new(Sphere::new(Vec3([0.0, 1.0, 0.0]), 1.0, &matte_red)),
        Box::new(Sphere::new(Vec3([4.0, 1.0, 0.0]), 1.0, &glass)),
    ];
    world.extend(center_spheres);

    let scene = ObjTree::new(world);
    camera.render(&scene, "./ball-field.png");
}

#[allow(unused)]
fn render_quads() {
    let camera = Camera::with_parameters(&CameraParams {
        aspect_ratio: 1.0, // square
        image_width: 800,
        position: Vec3([0.0, 0.0, 9.0]),
        view_target: Vec3([0.0, 0.0, 0.0]),
        vertical_fov: 80.0,
        samples: 50,
        max_depth: 20,
        ..Default::default()
    });

    static RED: Lambertian = Lambertian::new(Vec3([1.0, 0.2, 0.2]), 1.0);
    static GREEN: Lambertian = Lambertian::new(Vec3([0.2, 1.0, 0.2]), 1.0);
    static BLUE: Lambertian = Lambertian::new(Vec3([0.1, 0.2, 1.0]), 1.0);
    static ORANGE: Lambertian = Lambertian::new(Vec3([1.0, 0.75, 0.1]), 1.0);
    static CYAN: Lambertian = Lambertian::new(Vec3([0.0, 0.8, 0.9]), 1.0);

    let world = ObjTree::new(vec![
        Box::new(Quad::new(
            Vec3::new(-3, -2, 5),
            (Vec3::new(0, 0, -4), Vec3::new(0, 4, 0)),
            &RED,
        )),
        Box::new(Quad::new(
            Vec3::new(-2, -2, 0),
            (Vec3::new(4, 0, 0), Vec3::new(0, 4, 0)),
            &GREEN,
        )),
        Box::new(Quad::new(
            Vec3::new(3, -2, 1),
            (Vec3::new(0, 0, 4), Vec3::new(0, 4, 0)),
            &BLUE,
        )),
        Box::new(Quad::new(
            Vec3::new(-2, 3, 1),
            (Vec3::new(4, 0, 0), Vec3::new(0, 0, 4)),
            &ORANGE,
        )),
        Box::new(Quad::new(
            Vec3::new(-2, -3, 5),
            (Vec3::new(4, 0, 0), Vec3::new(0, 0, -4)),
            &CYAN,
        )),
    ]);

    camera.render(&world, "./quads.png");
}

#[allow(unused)]
fn render_emission_test() {
    let camera = Camera::with_parameters(&CameraParams {
        position: Vec3::new(26, 3, 6),
        view_target: Vec3::new(0, 2, 0),
        vertical_fov: 20.0,
        samples: 300,
        max_depth: 50,
        background_color: ColorVec3::zero(),
        ..Default::default()
    });

    let matte_grey: Lambertian = Lambertian::new(Vec3([0.6, 0.6, 0.6]), 1.0);
    let ground = Sphere::new(Vec3::new(0, -1000, 0), 1000.0, &matte_grey);
    let subject = Sphere::new(Vec3::new(0, 2, 0), 2.0, &matte_grey);
    // The diffuse light is stronger than (1.0, 1.0, 1.0) so that it lights
    // the scene even after the ray bounding several times.
    let light_mat: DiffuseLight = DiffuseLight::new(Vec3([0.0, 4.0, 4.0]));
    let light = Quad::new(
        Vec3::new(3, 1, -2),
        (Vec3::new(2, 0, 0), Vec3::new(0, 2, 0)),
        &light_mat,
    );

    let world = ObjTree::new(vec![Box::new(ground), Box::new(subject), Box::new(light)]);
    camera.render(&world, "./diffuse-light.png");
}

#[allow(unused)]
fn render_cornell_box() {
    let camera = Camera::with_parameters(&CameraParams {
        image_width: 1920,
        aspect_ratio: 1.0,
        position: Vec3::new(278, 278, -800),
        view_target: Vec3::new(278, 278, 0),
        vertical_fov: 40.0,
        samples: 200,
        max_depth: 50,
        ..Default::default()
    });
    todo!()
}

fn main() {
    // render_ball_field();
    // render_quads();
    render_emission_test();
}
