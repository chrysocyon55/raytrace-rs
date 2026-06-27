# raytrace-rs

A multithreaded CPU raytracer written in Rust, adapted from the [*Ray Tracing
in One Weekend*](https://raytracing.github.io/) series of textbooks.

Scenes are currently hard-coded as individual functions within `src/main.rs`,
with `main()` calling the desired scene to render.

## Example renders

Sphere field:
![Render of three large spheres in a field, surrounded by many smaller,
randomly-placed spheres.](/images/ball-field.png)

Diffuse lighting:
![Render of a sphere in a dark environment, with a harsh red box light
reflecting off of it.](/images/diffuse-light.png)

["Cornell box"](https://bowers.cornell.edu/cornell-box) test scene with
transluscent materials:
![Render of a Cornell box, featuring two transluscent cuboids inside a red and
green room with an overhead light.](/images/cornell-smoke.png)
