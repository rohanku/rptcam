use glm::vec3;
use rpt::lens::{FisheyeLens, SingleLens};
use rpt::*;
use std::sync::Arc;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    scene.add(Object::new(sphere().translate(&glm::vec3(0., 0., -4.0))));

    scene.add(Object::new(
        sphere()
            .scale(&glm::vec3(0.2, 0.2, 0.2))
            .translate(&glm::vec3(0., -0.8, 2.0)),
    ));
    scene.add(
        Object::new(
            cube()
                .rotate_y(glm::pi::<f64>() / 6.0)
                .scale(&glm::vec3(0.5, 0.3, 0.4))
                .translate(&glm::vec3(0.4, -0.8, 3.0)),
        )
        .material(Material::light(hex_color(0xffffff), 0.5)),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(1.5, -0.5, 1.0)),
        )
        .material(Material::light(hex_color(0xffffff), 2.0)),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(-1.5, -0.5, 1.0)),
        )
        .material(Material::light(hex_color(0xffffff), 0.5)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
    );

    scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
    scene.add(Light::Point(
        glm::vec3(100.0, 100.0, 100.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    let mut cam = PhysicalCamera::<FisheyeLens>::default();
    cam.focus(10.);

    Renderer::new(&scene, Arc::new(cam))
        .num_samples(256)
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}
