use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rapier2d::prelude::*;
use kd_tree::KdTree;
use rand::Rng;

use crate::{settings::DynamicSettings, *};

pub struct FoodPlugin;

#[derive(Component)]
pub struct Food;

#[derive(Resource)]
pub struct FoodTree(pub Option<KdTree<[f32; 2]>>);

#[derive(Bundle)]
struct FoodBundle {
    sprite_bundle: SpriteBundle,
    food: Food,
    rigid_body: RigidBody,
    collider: Collider,
    damping: Damping,
    collision_groups: CollisionGroups,
}

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FoodTree(None))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                spawn_food.run_if(on_timer(Duration::from_secs_f32(
                    FOOD_REFRESH_INTERVAL_SECS,
                ))),
            )
            .add_systems(
                Update,
                reload_food_kd_tree.run_if(on_timer(Duration::from_secs_f32(
                    FOOD_TREE_REFRESH_RATE_SECS,
                ))),
            );
    }
}

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    food_query: Query<With<Food>>,
    settings: Res<DynamicSettings>,
) {
    spawn_food(commands, asset_server, settings, food_query);
}

fn spawn_food(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<DynamicSettings>,
    food_query: Query<With<Food>>,
) {
    let mut rng = rand::thread_rng();
    let num_food = food_query.iter().len();
    let food_diff = 500;
    if num_food > (settings.num_food - food_diff) {
        return;
    }
    let num_instances = if num_food == 0 {
        settings.num_food
    } else {
        food_diff
    };
    // Food on the sides is a little bit sparse
    let range_factor = if rng.gen_range(0..100) <= 45 {
        2.0
    } else {
        2.5
    };

    for _ in 0..num_instances {
        let x = rng.gen_range(-(W as f32) / range_factor..W as f32 / range_factor);
        let y = rng.gen_range(-(H as f32) / range_factor..H as f32 / range_factor);
        commands.spawn(FoodBundle::new(x, y, &asset_server));
    }
}

fn reload_food_kd_tree(food_query: Query<&Transform, With<Food>>, mut food_tree: ResMut<FoodTree>) {
    let mut pts = Vec::new();
    for t in food_query.iter() {
        pts.push([t.translation.x, t.translation.y])
    }

    food_tree.0 = Some(KdTree::build_by_ordered_float(pts));
}

impl FoodBundle {
    fn new(x: f32, y: f32, asset_server: &AssetServer) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(2.0)),
                texture: asset_server.load(FOOD_SPRITE),
                ..default()
            },
            food: Food,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(4.0),
            damping: Damping {
                angular_damping: 2.0,
                linear_damping: 2.0,
            },
            collision_groups: CollisionGroups {
                memberships: Group::from_bits_truncate(GRP_FOOD),
                filters: Group::from_bits_truncate(MASK_FOOD),
            },
        }
    }
}
