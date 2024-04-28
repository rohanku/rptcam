//! This is an example that demonstrates bokeh.

use std::sync::Arc;
use std::time::Instant;

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

    let camera = ThinLensCamera::look_at(
        glm::vec3(0.7166, -9.2992, 2.8803),
        glm::vec3(0.8673, 0.2095, 0.9557),
        glm::vec3(0.0, 0.0, 1.0),
        0.6911,
    )
    .focus(
        glm::vec3(0.1, -2.0, 0.6),
        Some(Aperture {
            scale: 0.50,
            focal_distance: 0.,
            shape: ApertureShape::Poly(Polygon::get_heart(0.05, 0.05)),
        }),
    );

    let mut time = Instant::now();
    Renderer::new(&scene, Arc::new(camera))
        .width(800)
        .height(600)
        .max_bounces(6)
        .num_samples(400)
        .iterative_render(100, |iteration, buffer| {
            let millis = time.elapsed().as_millis();
            println!(
                "Finished iteration {}, took {} ms, variance: {}",
                iteration,
                millis,
                buffer.variance()
            );
            buffer
                .image()
                .save(format!("output_{:03}.png", iteration - 1))
                .expect("Failed to save image");
            time = Instant::now();
        });

    Ok(())
}
