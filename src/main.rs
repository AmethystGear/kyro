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
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::MeshBuilder,
        types,
        types::{Mesh, MeshData},
        visibility::BoundingSphere,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Error,
};
use rand::prelude::*;
use std::collections::HashSet;

use amethyst_nphysics::NPhysicsBackend;
use amethyst_physics::{prelude::*, PhysicsBundle};
use renderer::rendy::mesh::Indices;

mod character_systems;
mod chunk_system;
mod components;
mod marching_cubes;
mod matrix_3d;
mod terrain;
mod visual_utils;

use terrain::Terrain;

#[derive(Default)]
struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Add light
        add_light_entity(
            data.world,
            Srgb::new(1.0, 1.0, 1.0),
            Vector3::new(0.0, -1.0, 0.0),
            1.0,
        );

        // Create terrain
        let terrain = Terrain::new(
            random(),
            15,
            1.0,
            vec![0.3, 0.65, 0.05],
            vec![0.05, 0.1, 10.0],
        );
        data.world.insert(terrain);
        data.world.register::<components::Chunk>();

        // Create the character + camera.
        create_character_entity(data.world);
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
            character_systems::CameraMotionSystem::new(),
            "camera_motion_system",
            &["input_system"],
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            PhysicsBundle::<f32, NPhysicsBackend>::new()
                .with_frames_per_seconds(60)
                .with_max_sub_steps(8) // Safety
                .with_pre_physics(
                    character_systems::CharacterMotionControllerSystem::new(),
                    String::from("character_motion_controller"),
                    vec![],
                ).with_pre_physics(
                    chunk_system::ChunkSystem { chunk_posns: HashSet::new() },
                    String::from("chunk system"),
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
                .with_plugin(RenderShaded3D::default()),
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

/// Creates three entities:
/// 1. The character (With RigidBody).
/// 2. The camera boom handle attached to the character.
/// 3. The camera attached to the camera bool handle.
fn create_character_entity(world: &mut World) {
    let character = {
        let shape = {
            let desc = ShapeDesc::Capsule {
                half_height: 0.75,
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
        transf.set_translation(Vector3::new(10.0, 30.0, 10.0));

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
