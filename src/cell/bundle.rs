use bevy::math::vec2;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::trackers::*;
use crate::{nn::Net, *};

use super::{Brain, Cell};

#[derive(Bundle)]
pub struct CellBundle {
    sprite_bundle: SpriteBundle,
    cell: Cell,
    birth_place: BirthPlace,
    birth_ts: BirthTs,
    last_bullet_fired: LastBulletFired,
    periodic_update_interval: PeriodicUpdateInterval,
    last_updated: LastUpdated,
    rigid_body: RigidBody,
    collider: Collider,
    damping: Damping,
    brain: Brain,
    num_cells_spawned: NumCellsSpawned,
    fitness_score: FitnessScores,
    external_force: ExternalForce,
    collision_groups: CollisionGroups,
}

impl CellBundle {
    pub fn new(
        x: f32,
        y: f32,
        cell_id: u32,
        net: Net,
        sprite_path: &str,
        asset_server: &AssetServer,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let rot = rng.gen_range(0.0..6.0);
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(x, y, 1.0)
                    .with_rotation(Quat::from_rotation_z(rot))
                    .with_scale(Vec3::splat(1.5)),
                texture: asset_server.load(sprite_path),
                ..default()
            },
            cell: Cell(cell_id),
            birth_place: BirthPlace(vec2(x, y)),
            birth_ts: BirthTs::default(),
            last_bullet_fired: LastBulletFired::default(),
            periodic_update_interval: PeriodicUpdateInterval(rng.gen_range(0.0..=1.0)),
            last_updated: LastUpdated::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(7.0),
            damping: Damping {
                angular_damping: 2.0,
                linear_damping: 2.0,
            },
            brain: Brain(net),
            num_cells_spawned: NumCellsSpawned(0),
            fitness_score: FitnessScores::new(),
            external_force: ExternalForce {
                force: Vec2::ZERO,
                torque: 0.0,
            },
            collision_groups: CollisionGroups {
                memberships: Group::from_bits_truncate(GRP_CELLS),
                filters: Group::from_bits_truncate(MASK_CELLS),
            },
        }
    }
}
