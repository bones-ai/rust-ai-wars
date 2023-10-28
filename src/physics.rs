use std::time::Instant;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{bullet::Bullet, cell::energy::EnergyMap, food::Food, settings::DynamicSettings, *};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            // .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Update, handle_collision_events);
    }
}

fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::ZERO;
}

fn handle_collision_events(
    mut commands: Commands,
    mut energy_map: ResMut<EnergyMap>,
    settings: Res<DynamicSettings>,
    food_query: Query<With<Food>>,
    bullet_query: Query<&Bullet, With<Bullet>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok(b) = bullet_query.get(*e1) {
                    if let Ok(_) = food_query.get(*e2) {
                        commands.entity(*e2).despawn();
                        commands.entity(*e1).despawn();
                        match energy_map.0.get_key_value_mut(&(b.0)) {
                            Some((_, (v, i))) => {
                                *v = MAX_ENERGY.min(*v + settings.energy_per_food);
                                *i = Instant::now();
                            }
                            None => {
                                energy_map
                                    .0
                                    .insert(b.0, (settings.energy_per_food, Instant::now()));
                            }
                        }
                    }
                }
                if let Ok(b) = bullet_query.get(*e2) {
                    if let Ok(_) = food_query.get(*e1) {
                        commands.entity(*e1).despawn();
                        commands.entity(*e2).despawn();
                        match energy_map.0.get_key_value_mut(&(b.0)) {
                            Some((_, (v, i))) => {
                                *v = MAX_ENERGY.min(*v + settings.energy_per_food);
                                *i = Instant::now();
                            }
                            None => {
                                energy_map
                                    .0
                                    .insert(b.0, (settings.energy_per_food, Instant::now()));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
