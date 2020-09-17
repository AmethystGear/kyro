mod low_poly_shader;
mod perlin;
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        math::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Parent,
    },
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        self,
        camera::Camera,
        light,
        palette::{LinSrgba, Srgb},
        plugins::RenderToWindow,
        rendy::mesh::{MeshBuilder, Normal, Position, TexCoord},
        types,
        types::{Mesh, MeshData},
        visibility::BoundingSphere,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Error,
};
use low_poly_shader::MyRenderFlat3D;
use perlin::generate_perlin_noise;
use rand::prelude::*;

use amethyst_nphysics::NPhysicsBackend;
use amethyst_physics::{prelude::*, PhysicsBundle};
use renderer::rendy::mesh::Indices;

mod components;
mod systems;
mod visual_utils;

#[derive(Default)]
struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Add light
        add_light_entity(
            data.world,
            Srgb::new(1.0, 1.0, 1.0),
            Vector3::new(-0.0, -1.0, -0.0),
            1.0,
        );
        /*
        add_light_entity(
            data.world,
            Srgb::new(0.0, 0.0, 1.0),
            Vector3::new(0.2, 1.0, 0.2),
            2.0,
        );*/

        // Create floor
        create_floor(data.world, 2000.0f32, 256, random());

        // Create the character + camera.
        create_character_entity(data.world);

        // Create Box
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let display_config_path = app_root.join("config").join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(assets_dir.join("input_bindings.ron"))
                .unwrap(),
        )?
        .with(
            systems::CameraMotionSystem::new(),
            "camera_motion_system",
            &["input_system"],
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            PhysicsBundle::<f32, NPhysicsBackend>::new()
                .with_frames_per_seconds(60)
                .with_max_sub_steps(8) // Safety
                .with_pre_physics(
                    systems::CharacterMotionControllerSystem::new(),
                    String::from("character_motion_controller"),
                    vec![],
                ),
        )?
        .with_bundle(
            RenderingBundle::<types::DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .unwrap()
                        .with_clear([0.7188, 0.2578, 0.0586, 1.0]),
                )
                .with_plugin(MyRenderFlat3D::default()),
        )?;
    let mut game = Application::build(assets_dir, Example::default())?.build(game_data)?;
    game.run();
    Ok(())
}

fn add_light_entity(world: &mut World, color: Srgb, direction: Vector3<f32>, intensity: f32) {
    let light: light::Light = light::DirectionalLight {
        color,
        direction: direction.normalize(),
        intensity,
    }
    .into();

    world.create_entity().with(light).build();
}

fn create_floor(world: &mut World, map_size: f32, num_divisions: u16, seed: i64) {
    let rb = {
        let mut rb_desc = RigidBodyDesc::default();
        rb_desc.mode = BodyMode::Static;

        let physics_world = world.fetch::<PhysicsWorld<f32>>();
        physics_world.rigid_body_server().create(&rb_desc)
    };

    let mut indicies = vec![];
    let mut posns = vec![];
    let mut norms = vec![];
    let mut coords = vec![];
    let noise = generate_perlin_noise(num_divisions, num_divisions, 7, seed);
    let mut i = 0;
    for y in 0..num_divisions {
        for x in 0..num_divisions {
            let x_flt = (x as f32) / (num_divisions as f32) * map_size;
            let z_flt = (y as f32) / (num_divisions as f32) * map_size;
            let mut raw = noise[i] as f32 * 300.0;
            if raw < 100.0 {
                raw = 100.0;
            }
            let posn = Position {
                0: [x_flt, raw, z_flt],
            };
            i += 1;
            posns.push(posn);
            let norm = Normal {
                0: [0.0f32, 0.0f32, 0.0f32],
            };
            norms.push(norm);
            let coord = TexCoord {
                0: [0.0f32, 0.0f32],
            };
            coords.push(coord);

            if x != num_divisions - 1 && y != num_divisions - 1 {
                let curr = x + y * num_divisions;
                // first tri
                indicies.push(curr + 1 + num_divisions);
                indicies.push(curr + 1);
                indicies.push(curr);
                // second tri
                indicies.push(curr + num_divisions);
                indicies.push(curr + 1 + num_divisions);
                indicies.push(curr);
            }
        }
    }

    let mut indicies_collision = Vec::new();
    for i in 0..indicies.len() / 3 {
        indicies_collision.push(Point3::from_slice(&[
            indicies[i * 3] as usize,
            indicies[i * 3 + 1] as usize,
            indicies[i * 3 + 2] as usize,
        ]));
    }
    let mut points_collision = Vec::new();
    for p in &posns {
        points_collision.push(Point3::from_slice(&[p.0[0], p.0[1], p.0[2]]))
    }
    let mesh = world.exec(|loader: AssetLoaderSystemData<Mesh>| {
        loader.load_from_data(
            MeshData(
                MeshBuilder::new()
                    .with_vertices(posns)
                    .with_vertices(norms)
                    .with_vertices(coords)
                    .with_indices(Indices::U16(indicies.into())),
            ),
            (),
        )
    });

    let shape = {
        let desc = ShapeDesc::TriMesh {
            indices: indicies_collision,
            points: points_collision,
        };
        let physics_world = world.fetch::<PhysicsWorld<f32>>();
        physics_world.shape_server().create(&desc)
    };

    let mat = visual_utils::create_material(
        world,
        LinSrgba::new(0.7188, 0.1578, 0.0, 1.0),
        0.0, // Metallic
        1.0, // Roughness
    );

    world
        .create_entity()
        .with(mesh)
        .with(mat)
        .with(BoundingSphere::new(
            Point3::new(map_size / 2.0, 0.0, map_size / 2.0),
            map_size,
        ))
        .with(Transform::default())
        .with(shape)
        .with(rb)
        .build();
}

/// Creates three entities:
/// 1. The character (With RigidBody).
/// 2. The camera boom handle attached to the character.
/// 3. The camera attached to the camera bool handle.
fn create_character_entity(world: &mut World) {
    let character = {
        let shape = {
            let desc = ShapeDesc::Capsule {
                half_height: 1.0,
                radius: 0.5,
            };
            let physics_world = world.fetch::<PhysicsWorld<f32>>();
            physics_world.shape_server().create(&desc)
        };

        let rb = {
            let mut rb_desc = RigidBodyDesc::default();
            rb_desc.lock_rotation_x = true;
            rb_desc.lock_rotation_y = true;
            rb_desc.lock_rotation_z = true;
            rb_desc.contacts_to_report = 3;
            rb_desc.friction = 0.0;
            rb_desc.bounciness = 0.0;

            let physics_world = world.fetch::<PhysicsWorld<f32>>();
            physics_world.rigid_body_server().create(&rb_desc)
        };

        let mut transf = Transform::default();
        transf.set_translation(Vector3::new(500.0, 500.0, 500.0));

        world
            .create_entity()
            .with(transf)
            .with(shape)
            .with(rb)
            .with(components::CharacterBody)
            .build()
    };

    let camera_boom_handle = {
        let mut transf = Transform::default();
        transf.set_translation_y(0.0);

        world
            .create_entity()
            .with(transf)
            .with(components::CameraBoomHandle)
            .with(Parent { entity: character })
            .build()
    };

    let _camera = {
        let mut camera_transform = Transform::default();
        camera_transform.set_translation_xyz(0.0, 0.0, 0.0);

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        world
            .create_entity()
            .with(camera_transform)
            .with(Camera::standard_3d(width, height))
            .with(Parent {
                entity: camera_boom_handle,
            })
            .build()
    };
}
