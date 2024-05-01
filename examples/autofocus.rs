use rand_distr::num_traits::Pow;
use rpt::lens::{Lens, SingleLens};
use rpt::*;
use std::sync::Arc;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    for i in 0..4 {
        let scale = 0.5f64.pow(i as f64);
        scene.add(Object::new(
            sphere()
                .scale(&glm::vec3(scale, scale, scale))
                .translate(&glm::vec3(
                    if i % 2 == 0 { 1. } else { -1. },
                    -1. + scale,
                    -15.0 + 5. * i as f64,
                )),
        ));
    }

    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
    );

    scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
    scene.add(Light::Point(
        glm::vec3(100.0, 100.0, 100.0),
        glm::vec3(0.0, 5.0, 1.0),
    ));

    for i in 0..4 {
        let lens = SingleLens {
            aperture: Aperture {
                scale: 0.2,
                shape: ApertureShape::Circle,
            },
            ..Default::default()
        };
        let lens_system = lens.lens_system(10.);
        let mut camera = PhysicalCamera {
            eye: glm::vec3(0., -0.5, 15.),
            sensor_width: 0.8,
            sensor_height: 0.6,
            lens,
            lens_system,
            ..Default::default()
        };

        camera.focus(30. - 5. * i as f64);

        Renderer::new(&scene, Arc::new(camera))
            .num_samples(128)
            .width(800)
            .height(600)
            .render()
            .save(format!("output_{i}.png"))?;
    }

    Ok(())
}
