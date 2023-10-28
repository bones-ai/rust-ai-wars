use std::time::Instant;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{cell::energy::EnergyMap, settings::DynamicSettings, trackers::BirthTs, *};

pub struct BulletPlugin;

#[derive(Component)]
pub struct Bullet(pub u32);

#[derive(Bundle)]
pub struct BulletBundle {
    sprite_bundle: SpriteBundle,
    bullet: Bullet,
    locked_axis: LockedAxes,
    birth_ts: BirthTs,
    rigid_body: RigidBody,
    collider: Collider,
    velocity: Velocity,
    collision_groups: CollisionGroups,
    active_events: ActiveEvents,
}

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, bullet_cleanup);
    }
}

fn setup() {}

fn bullet_cleanup(
    mut commands: Commands,
    mut energy_map: ResMut<EnergyMap>,
    settings: Res<DynamicSettings>,
    bullet_query: Query<(Entity, &BirthTs, &Bullet), With<Bullet>>,
) {
    for (entity, birth_ts, b) in bullet_query.iter() {
        if birth_ts.0.elapsed_within(BULLET_LIFESPAN) {
            continue;
        }
        commands.entity(entity).despawn();

        match energy_map.0.get_mut(&b.0) {
            Some((v, i)) => {
                *v -= settings.bullet_miss_penalty;
                *i = Instant::now();
            }
            None => {}
        }
    }
}

impl BulletBundle {
    pub fn new(x: f32, y: f32, cell_id: u32, direction: Vec2, asset_server: &AssetServer) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load(BULLET_SPRITE),
                ..default()
            },
            bullet: Bullet(cell_id),
            locked_axis: LockedAxes::ROTATION_LOCKED,
            birth_ts: BirthTs::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(4.0),
            velocity: Velocity::linear(direction),
            collision_groups: CollisionGroups {
                memberships: Group::from_bits_truncate(GRP_BULLET),
                filters: Group::from_bits_truncate(MASK_BULLET),
            },
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}
