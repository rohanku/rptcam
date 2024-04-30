//! Ferris the crab - https://rustacean.net/
//!
//! Original model credit (.PLY) goes to Raptori at https://www.thingiverse.com/thing:3414267

use std::fs::File;
use std::sync::Arc;

use rpt::lens::{AchromaticDoublet, AchromaticDoubletParams, Lens, SingleLens};
use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    let crab_scale = glm::vec3(2.0, 2.4, 2.0);
    scene.add(
        Object::new(
            load_obj(File::open("examples/rustacean.obj")?)?
                .translate(&glm::vec3(0.0, 0.134649, 0.0))
                .scale(&crab_scale),
        )
        .material(Material::specular(hex_color(0xF84C00), 0.2)),
    );

    for i in 0..=10 {
        scene.add(
            Object::new(sphere().scale(&glm::vec3(5., 5., 5.)).translate(&glm::vec3(
                (i as f64 - 5.) * 12.,
                5.0,
                -25.,
            )))
            .material(Material::diffuse(hex_color(0xFFAA77))),
        );
    }
    for i in 0..=20 {
        scene.add(
            Object::new(
                sphere()
                    .scale(&glm::vec3(0.1, 0.1, 0.1))
                    .translate(&glm::vec3((i as f64 - 10.) * 3., 4.0, -10.)),
            )
            .material(Material::light(hex_color(0xFF7B00), 10.0)),
        );
    }
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 0.0))
            .material(Material::diffuse(hex_color(0xAAAA77))),
    );

    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(0.0, 20.0, 3.0)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), 160.0)),
    ));

    let eye = glm::vec3(-2., 1.1, 15.);
    let center = glm::vec3(0.0, 0.9, 0.0);

    let distance_steps = 5;
    let min_distance = 11.;
    let max_distance = 22.;
    let distance_step_size = (max_distance - min_distance) / (distance_steps - 1) as f64;

    let aperture_steps = 3;
    let min_aperture = 0.01;
    let max_aperture = 0.2;
    let aperture_step_size = (max_aperture - min_aperture) / (aperture_steps - 1) as f64;

    for (shape_i, shape) in [
        ApertureShape::Circle,
        ApertureShape::Poly(Polygon::get_heart(0.05, 0.05)),
    ]
    .iter()
    .enumerate()
    {
        for aperture_i in (0..aperture_steps).rev() {
            let aperture = aperture_step_size * aperture_i as f64 + min_aperture;
            for dist_i in 0..distance_steps {
                let dist = distance_step_size * dist_i as f64 + min_distance;
                let filename = format!(
                    "rustacean_achromatic_shape{}_aperture{}_dist{}.png",
                    shape_i, aperture_i, dist_i
                );
                println!("Rendering {filename}");
                // let lens = SingleLens {
                //     aperture: Aperture {
                //         scale: aperture,
                //         shape: shape.clone(),
                //     },
                //     ..Default::default()
                // };
                let lens = AchromaticDoublet::new(AchromaticDoubletParams {
                    aperture: Aperture {
                        scale: aperture,
                        shape: shape.clone(),
                    },
                    ..Default::default()
                });
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
                camera.look_at(eye, center, glm::vec3(0.0, 1.0, 0.0));
                camera.focus(dist);

                Renderer::new(&scene, Arc::new(camera))
                    .width(800)
                    .height(600)
                    .max_bounces(1)
                    .num_samples(128)
                    .render()
                    .save(filename)?;
            }
        }
    }

    Ok(())
}
