use std::f32::consts::PI;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};
use bevy_rapier2d::prelude::*;

use crate::{
    food::FoodTree,
    nn::Net,
    trackers::{LastBulletFired, LastUpdated, OneSecondTimer, PeriodicUpdateInterval},
    *,
};

use super::{
    bundle::CellBundle,
    cell::{perform_cell_action, CellAction},
};

pub struct UserCellPlugin;

#[derive(Component)]
pub struct UserControlledCell;

impl Plugin for UserCellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugLinesPlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update_user_controlled_cell);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    if !IS_USER_ENABLED {
        return;
    }

    let net = Net::new(NET_ARCH.to_vec());
    commands.spawn((
        CellBundle::new(0.0, 0.0, 0, net, USER_CELL_SPRITE, &asset_server),
        UserControlledCell,
    ));
}

fn update_user_controlled_cell(
    mut commands: Commands,
    second_timer: Res<OneSecondTimer>,
    food_tree: Res<FoodTree>,
    keyboard_input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut lines: ResMut<DebugLines>,
    mut shapes: ResMut<DebugShapes>,
    mut user_query: Query<
        (
            &mut Transform,
            &mut ExternalForce,
            &mut LastUpdated,
            &mut LastBulletFired,
            &PeriodicUpdateInterval,
        ),
        With<UserControlledCell>,
    >,
) {
    if user_query.is_empty() {
        return;
    }

    let (
        mut transform,
        mut external_force,
        mut last_updated,
        mut last_bullet_fired,
        periodic_update_interval,
    ) = user_query.get_single_mut().unwrap();
    let w_key = keyboard_input.pressed(KeyCode::W);
    let a_key = keyboard_input.pressed(KeyCode::A);
    let s_key = keyboard_input.pressed(KeyCode::S);
    let d_key = keyboard_input.pressed(KeyCode::D);
    let space_key = keyboard_input.pressed(KeyCode::Space);

    lines.line_colored(Vec3::splat(0.0), transform.translation, 0.0, Color::RED);
    shapes
        .circle()
        .position(transform.translation)
        .radius(VISION_RADIUS)
        .color(Color::RED);

    if last_updated.0.elapsed_within(UPDATE_INTERVAL) {
        return;
    }
    if second_timer.0.elapsed_within(periodic_update_interval.0) {
        return;
    }

    last_updated.0.set_instant_now();
    let mut target_x = 0.0;
    let mut target_y = 0.0;
    let key = (transform.translation.x, transform.translation.y);
    if let Some(t) = &food_tree.0 {
        match t.nearest(&[key.0 as f32, key.1 as f32]) {
            Some(v) => {
                if v.squared_distance <= VISION_RADIUS * VISION_RADIUS {
                    let [x, y] = v.item;
                    target_x = *x;
                    target_y = *y;
                }
            }
            None => {}
        }
    }
    lines.line_colored(
        transform.translation,
        vec3(target_x as f32, target_y as f32, 0.0),
        0.0,
        Color::BLUE,
    );

    let spin_left = a_key;
    let spin_right = d_key;
    let thrust = w_key;
    let shoot = s_key || space_key;

    let action = CellAction {
        thrust,
        spin_left,
        spin_right,
        shoot,
    };
    perform_cell_action(
        action,
        0,
        &mut last_bullet_fired,
        &mut external_force,
        &mut commands,
        &mut transform,
        &asset_server,
    );

    // This is for debug prints
    let nn_inp_dist = transform
        .translation
        .truncate()
        .distance(vec2(target_x as f32, target_y as f32))
        / VISION_RADIUS;
    let nn_inp_dist = if nn_inp_dist > 1.0 { 1.0 } else { nn_inp_dist };

    let nn_inp_angle = angle_between(
        transform.translation.x,
        transform.translation.y,
        target_x,
        target_y,
    );
    let nn_inp_angle = nn_inp_angle / 360.0;
    let nn_inp_angle = if nn_inp_angle < 0.0 {
        nn_inp_angle + 360.0
    } else {
        nn_inp_angle
    };

    let nn_cell_angle = (transform.rotation.to_euler(EulerRot::XYZ).2 + PI / 2.0).to_degrees();
    let nn_cell_angle = if nn_cell_angle < 0.0 {
        nn_cell_angle + 360.0
    } else {
        nn_cell_angle
    };
    let nn_cell_angle = nn_cell_angle / 360.0;
    let angle_diff = (nn_inp_angle - nn_cell_angle).abs();

    println!(
        "{:?}",
        (nn_inp_dist, angle_diff, nn_inp_angle, nn_cell_angle)
    );
}

fn angle_between(a: f32, b: f32, x: f32, y: f32) -> f32 {
    let angle_radians = (y - b).atan2(x - a);
    let mut angle_degrees = angle_radians.to_degrees();
    if angle_degrees < 0.0 {
        angle_degrees += 360.0;
    }

    angle_degrees
}
