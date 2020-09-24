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

use amethyst_nphysics::NPhysicsBackend;
use amethyst_physics::{prelude::*, PhysicsBundle};
use renderer::rendy::mesh::Indices;

mod character_systems;
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
            Vector3::new(-0.1, -1.0, -0.1),
            1.0,
        );
        add_light_entity(
            data.world,
            Srgb::new(1.0, 1.0, 1.0),
            Vector3::new(0.1, 1.0, 0.1),
            0.2,
        );

        // Create terrain

        let mut terrain = Terrain::new(
            random(),
            15,
            1.0,
            vec![0.3, 0.65, 0.05],
            vec![0.05, 0.1, 10.0],
        );
        data.world.register::<components::Chunk>();
        let size = 5;
        for z in -size..(size + 1) {
            for y in -size..(size + 1) {
                for x in -size..(size + 1) {
                    create_chunk(data.world, &mut terrain, x, y, z);
                }
            }
        }

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

fn create_chunk(
    world: &mut World,
    terrain: &mut Terrain,
    chunk_x: i16,
    chunk_y: i16,
    chunk_z: i16,
) {
    let rb = {
        let mut rb_desc = RigidBodyDesc::default();
        rb_desc.mode = BodyMode::Static;

        let physics_world = world.fetch::<PhysicsWorld<f32>>();
        physics_world.rigid_body_server().create(&rb_desc)
    };
    let (indicies, posns, norms, coords) =
        terrain.get_chunk(chunk_x, chunk_y, chunk_z).get_mesh_data();
    if indicies.len() == 0 {
        return;
    }
    let mut indicies_collision = Vec::new();
    for i in 0..posns.len() / 3 {
        indicies_collision.push(Point3::from_slice(&[i * 3, i * 3 + 1, i * 3 + 2]));
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

    let mut transform = Transform::default();
    transform.set_translation_xyz(
        chunk_x as f32 * terrain.chunk_size(),
        chunk_y as f32 * terrain.chunk_size(),
        chunk_z as f32 * terrain.chunk_size(),
    );
    world
        .create_entity()
        .with(mesh)
        .with(mat)
        .with(BoundingSphere::new(
            Point3::new(
                terrain.chunk_size() / 2.0,
                terrain.chunk_size() / 2.0,
                terrain.chunk_size() / 2.0,
            ),
            terrain.chunk_size() * 1.5,
        ))
        .with(transform)
        .with(shape)
        .with(rb)
        .with(components::Chunk)
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
