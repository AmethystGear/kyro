use crate::Terrain;
use std::collections::HashSet;
use amethyst::{
    assets::{self, AssetLoaderSystemData, AssetStorage, Handle},
    core::{
        math::{Point3, Vector3},
        Transform,
    },
    ecs::prelude::*,
    renderer::{
        mtl,
        palette::LinSrgba,
        rendy::mesh::{Indices, MeshBuilder},
        types::{self, MeshData},
        visibility::BoundingSphere,
        Material, Mesh, Texture,
    },
};

pub struct ChunkSystem {
    pub chunk_posns: HashSet<(isize, isize, isize)>
}
use crate::{components::*, visual_utils};
use amethyst_physics::{
    prelude::{PhysicsHandle, PhysicsRigidBodyTag, PhysicsShapeTag},
    servers::{BodyMode, PhysicsWorld, RigidBodyDesc, ShapeDesc},
};

impl<'s> System<'s> for ChunkSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, PhysicsWorld<f32>>,
        AssetLoaderSystemData<'s, Mesh>,        
        Read<'s, Terrain>,
        WriteStorage<'s, Chunk>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Mesh>>,
        WriteStorage<'s, Handle<Material>>,
        ReadStorage<'s, CharacterBody>,
        WriteStorage<'s, BoundingSphere>,
        WriteStorage<'s, PhysicsHandle<PhysicsShapeTag>>,
        WriteStorage<'s, PhysicsHandle<PhysicsRigidBodyTag>>,
        ReadExpect<'s, assets::Loader>,
        ReadExpect<'s, AssetStorage<Texture>>,
        ReadExpect<'s, AssetStorage<mtl::Material>>,
        ReadExpect<'s, mtl::MaterialDefaults>
    );

    fn run(
        &mut self,
        (
            mut entities,
            physics_world,
            mesh_loader,
            terrain,
            mut chunks,
            mut transforms,
            mut meshes,
            mut materials,
            camera_boom_handles,
            mut bounds,
            mut physics_shape,
            mut physics_rb,
            loader,
            tex,
            mat,
            mat_defaults
        ): Self::SystemData,
    ) {
        let (cam_posn, _) = (&transforms, &camera_boom_handles).join().next().unwrap();
        let cam_posn = cam_posn.translation();
        let chunk_size = (&*terrain).chunk_size();
        for (e, transform, c) in (&*entities, &transforms, &chunks).join() {
            let mut diff: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
            cam_posn.sub_to(transform.translation(), &mut diff);
            let dist: f32 = diff.magnitude();
            if dist > chunk_size * 10.0 {
                println!("deleting chunk");
                entities.delete(e).unwrap();
                let chunk_posn = Vector3::new(
                    (transform.translation().x / chunk_size) as isize,
                    (transform.translation().y / chunk_size) as isize,
                    (transform.translation().z / chunk_size) as isize,
                );
                self.chunk_posns.remove(&(chunk_posn.x, chunk_posn.y, chunk_posn.z));
            }
        }
        
        let posn = cam_posn;
        let base_posn = Vector3::new(
            (posn.x / chunk_size) as isize,
            (posn.y / chunk_size) as isize,
            (posn.z / chunk_size) as isize,
        );

        let range = 3;
        for x in -range..(range + 1) {
            for y in -range..(range + 1) {
                for z in -range..(range + 1) {
                    if !self.chunk_posns.contains(&(base_posn.x + x, base_posn.y + y, base_posn.z + z)) {
                        let chunk_posn = Vector3::new(base_posn.x + x, base_posn.y + y, base_posn.z + z);
                        create_chunk(
                            &mut entities,
                            &*physics_world,
                            &*terrain,
                            &chunk_posn,
                            &mesh_loader,
                            &*loader,
                            &*tex,
                            &*mat,
                            &*mat_defaults,
                            &mut meshes,
                            &mut transforms,
                            &mut chunks,
                            &mut materials,
                            &mut bounds,
                            &mut physics_shape,
                            &mut physics_rb,
                        );
                        self.chunk_posns.insert((base_posn.x + x, base_posn.y + y, base_posn.z + z));
                    }
                }
            }
        }
    }
}

fn create_chunk(
    entities: &mut Entities,
    physics_world: &PhysicsWorld<f32>,
    terrain: &Terrain,
    chunk_posn: &Vector3<isize>,
    mesh_loader: &AssetLoaderSystemData<Mesh>,
    loader: &assets::Loader,
    tex: &AssetStorage<types::Texture>,
    mat: &AssetStorage<mtl::Material>,
    mat_defaults: &mtl::MaterialDefaults,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    transforms: &mut WriteStorage<Transform>,
    chunks: &mut WriteStorage<Chunk>,
    materials: &mut WriteStorage<Handle<Material>>,
    bounds: &mut WriteStorage<BoundingSphere>,
    physics_shape: &mut WriteStorage<PhysicsHandle<PhysicsShapeTag>>,
    physics_rb: &mut WriteStorage<PhysicsHandle<PhysicsRigidBodyTag>>,
) {
    let (indicies, posns, norms, coords) = terrain.get_chunk(chunk_posn).get_mesh_data();
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
    let mesh = mesh_loader.load_from_data(
        MeshData(
            MeshBuilder::new()
                .with_vertices(posns)
                .with_vertices(norms)
                .with_vertices(coords)
                .with_indices(Indices::U16(indicies.into())),
        ),
        (),
    );
    /*
    let shape = {
        let desc = ShapeDesc::TriMesh {
            indices: indicies_collision,
            points: points_collision,
        };
        physics_world.shape_server().create(&desc)
    };
    */
    let mat = visual_utils::create_material(
        loader,
        tex,
        mat,
        mat_defaults,
        LinSrgba::new(0.7188, 0.1578, 0.0, 1.0),
        0.0, // Metallic
        1.0, // Roughness
    );

    let mut transform = Transform::default();
    transform.set_translation_xyz(
        chunk_posn.x as f32 * terrain.chunk_size(),
        chunk_posn.y as f32 * terrain.chunk_size(),
        chunk_posn.z as f32 * terrain.chunk_size(),
    );
    /*
    let rb = {
        let mut rb_desc = RigidBodyDesc::default();
        rb_desc.mode = BodyMode::Static;
        physics_world.rigid_body_server().create(&rb_desc)
    };
    */
    entities
        .build_entity()
        .with(mesh, meshes)
        .with(mat, materials)
        .with(
            BoundingSphere::new(
                Point3::new(
                    terrain.chunk_size() / 2.0,
                    terrain.chunk_size() / 2.0,
                    terrain.chunk_size() / 2.0,
                ),
                terrain.chunk_size(),
            ),
            bounds,
        )
        .with(transform, transforms)
        /*
        .with(shape, physics_shape)
        .with(rb, physics_rb)
        */
        .with(Chunk, chunks)
        .build();
}
