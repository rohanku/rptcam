//! This is an example that demonstrates bokeh.

use std::sync::Arc;
use std::time::Instant;

use rpt::lens::{Lens, SingleLens};
use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    let red = Material::light(hex_color(0xE78999), 10.0);
    let yellow = Material::light(hex_color(0xE7A94D), 2.0);
    let green = Material::light(hex_color(0xB3E7AA), 0.5);
    let blue = Material::light(hex_color(0x7CA3E7), 2.0);
    let grey = Material::light(hex_color(0xAAAAAA), 3.0);
    let light_mtl = Material::light(hex_color(0xFFFFFF), 0.2);

    let spheres = vec![
        (glm::vec3(0.5, 4.0, 1.0), red),
        (glm::vec3(3.15, -0.7, 1.5), yellow),
        (glm::vec3(0.1, -2.0, 0.6), green),
        (glm::vec3(-1.7, -0.2, 1.1), blue),
        (glm::vec3(1.2, 0.4, 0.5), grey),
    ];

    scene.add(
        Object::new(plane(glm::vec3(0.0, 0.0, 1.0), 0.0))
            .material(Material::diffuse(hex_color(0xE7E7E7))),
    );
    for (pos, mtl) in spheres {
        scene.add(
            Object::new(sphere().scale(&glm::vec3(0.1, 0.1, 0.1)).translate(&pos)).material(mtl),
        )
    }

    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(1.2, -1.5, 8.0)),
        )
        .material(light_mtl),
    ));

    let eye = glm::vec3(0.7166, -12.2992, 2.8803);
    let center = glm::vec3(0.8673, 0.2095, 0.9557);
    let dir = (center - eye).normalize();

    let distance_steps = 10;
    let min_distance = f64::abs((glm::vec3(0.1, -2.0, 0.6) - eye).dot(&dir));
    let max_distance = f64::abs((glm::vec3(0.5, 4.0, 1.0) - eye).dot(&dir));
    let distance_step_size = (max_distance - min_distance) / (distance_steps - 1) as f64;

    let aperture_steps = 4;
    let min_aperture = 0.01;
    let max_aperture = 0.2;
    let aperture_step_size = (max_aperture - min_aperture) / (aperture_steps - 1) as f64;

    for (shape_i, shape) in [
        ApertureShape::Circle,
        ApertureShape::Square,
        ApertureShape::Poly(Polygon::get_star(5.0)),
        ApertureShape::Poly(Polygon::get_heart(0.05, 0.05)),
    ]
    .iter()
    .enumerate()
    {
        for aperture_i in 0..aperture_steps {
            let aperture = aperture_step_size * aperture_i as f64 + min_aperture;
            for dist_i in 0..distance_steps + 4 {
                let dist = distance_step_size * (dist_i as f64 - 2.) + min_distance;
                let filename = format!(
                    "bokeh_singlelens_shape{}_aperture{}_dist{}.png",
                    shape_i, aperture_i, dist_i
                );
                println!("Rendering {filename}");
                let lens = SingleLens {
                    aperture: Aperture {
                        scale: aperture,
                        shape: shape.clone(),
                    },
                    ..Default::default()
                };
                let lens_system = lens.lens_system(10.);
                let mut camera = PhysicalCamera {
                    eye: Default::default(),
                    direction: Default::default(),
                    up: Default::default(),
                    sensor_width: 4.,
                    sensor_height: 3.,
                    lens,
                    lens_system,
                };
                camera.look_at(eye, center, glm::vec3(0.0, 0.0, 1.0));
                camera.focus(dist);

                Renderer::new(&scene, Arc::new(camera))
                    .width(800)
                    .height(600)
                    .max_bounces(1)
                    .num_samples(400)
                    .render()
                    .save(filename)?;
            }
        }
    }

    Ok(())
}
