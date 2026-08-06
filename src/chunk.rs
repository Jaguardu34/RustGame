use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{map_generator::MapGenerator, world_mesh::generate_terrain_mesh};

pub struct Chunk {
    pub pos: IVec2,
    pub entity: Option<Entity>,
    pub water_entity: Option<Entity>,
    pub mesh_handle: Handle<Mesh>,
    pub material_handle: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct ChunkEntity;

impl Chunk {
    pub fn new(
        pos: IVec2,
        height_mult: f64,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        map_gen: &MapGenerator,
    ) -> Self {
        let mesh = generate_terrain_mesh(
            Vec2 {
                x: (pos.x * 16) as f32,
                y: (pos.y * 16) as f32,
            },
            16,
            16,
            height_mult,
            map_gen,
        );

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        });

        Self {
            pos,
            entity: None,
            mesh_handle,
            material_handle,
            water_entity: None,
        }
    }

    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        if self.entity.is_some() {
            return;
        }

        if self.water_entity.is_some() {
            return;
        }

        let mesh = meshes.get(&self.mesh_handle).expect("Mesh non trouvé");
        let collider = Collider::from_bevy_mesh(
            mesh,
            &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
        )
        .expect("Impossible de générer le collider");

        self.entity = Some(
            commands
                .spawn((
                    Mesh3d(self.mesh_handle.clone()),
                    MeshMaterial3d(self.material_handle.clone()),
                    RigidBody::Fixed,
                    Transform::from_xyz((self.pos.x * 16) as f32, 0.0, (self.pos.y * 16) as f32),
                    collider,
                    ChunkEntity,
                    Name::new("ChunkMesh"),
                ))
                .id(),
        );

        // self.water_entity = Some(
        //     commands
        //         .spawn((
        //             Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 16.0).subdivisions(10))),
        //             MeshMaterial3d(materials.add(StandardMaterial {
        //                 base_color: Color::srgba(0.0, 0.3, 0.5, 0.6),
        //                 alpha_mode: AlphaMode::Blend,
        //                 perceptual_roughness: 0.05,
        //                 reflectance: 0.9,
        //                 ..default()
        //             })),
        //             Transform::from_xyz((self.pos.x * 16) as f32, 2.0, (self.pos.y * 16) as f32),
        //             ChunkEntity,
        //             Name::new("WaterMesh"),
        //         ))
        //         .id(),
        // );
    }

    pub fn despawn(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.entity.take() {
            commands.entity(entity).despawn();
        }
        if let Some(water_entity) = self.water_entity.take() {
            commands.entity(water_entity).despawn();
        }
    }
}
