use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use bevy::{math::vec2, prelude::*, time::common_conditions::on_timer};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    bullet::BulletBundle,
    food::FoodTree,
    gui::SimStats,
    nn::Net,
    settings::SimSettings,
    trackers::{
        BirthPlace, BirthTs, FitnessScores, LastBulletFired, LastUpdated, NumCellsSpawned,
        OneSecondTimer, PeriodicUpdateInterval,
    },
    *,
};

use super::{
    bundle::CellBundle,
    energy::{CellEnergyPlugin, EnergyMap},
    focus::{CellFocusPlugin, FocusedCellNet, FocusedCellStats},
    user::{UserCellPlugin, UserControlledCell},
};

pub struct CellPlugin;

#[derive(Component)]
pub struct Cell(pub u32);
#[derive(Resource)]
pub struct CellId(pub u32);
#[derive(Component)]
pub struct Brain(pub Net);

pub struct CellAction {
    pub thrust: bool,
    pub spin_left: bool,
    pub spin_right: bool,
    pub shoot: bool,
}

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CellEnergyPlugin)
            .add_plugins(CellFocusPlugin)
            .add_plugins(UserCellPlugin)
            .insert_resource(CellId(0))
            .add_systems(Startup, setup)
            .add_systems(Update, update_cells_system)
            .add_systems(Update, update_cell_sprite)
            .add_systems(
                Update,
                kill_bad_cells.run_if(on_timer(Duration::from_secs_f32(0.5))),
            )
            .add_systems(
                Update,
                cell_replication_system.run_if(on_timer(Duration::from_secs_f32(0.5))),
            )
            .add_systems(
                Update,
                spawn_cells.run_if(on_timer(Duration::from_secs_f32(5.0))),
            );
    }
}

fn setup(
    commands: Commands,
    cell_id: ResMut<CellId>,
    asset_server: Res<AssetServer>,
    cell_query: Query<(With<Cell>, Without<UserControlledCell>)>,
) {
    spawn_cells(commands, cell_id, asset_server, cell_query);
}

fn kill_bad_cells(
    mut commands: Commands,
    mut energy_map: ResMut<EnergyMap>,
    cell_query: Query<
        (
            &Cell,
            &Transform,
            &BirthPlace,
            &BirthTs,
            &LastBulletFired,
            Entity,
        ),
        (With<Cell>, Without<UserControlledCell>),
    >,
) {
    for (c, transform, birth_place, birth_ts, last_bullet_fired, entity) in cell_query.iter() {
        // Low energy cells
        match energy_map.0.get(&c.0) {
            Some((v, _)) => {
                if *v <= 0.0 {
                    energy_map.0.remove(&c.0);
                    commands.entity(entity).despawn();
                    continue;
                }
            }
            None => {}
        }

        // Unmoving cells
        if birth_ts.0.elapsed_past(10.0) && birth_ts.0.elapsed_within(20.0) {
            if birth_place
                .0
                .distance_squared(transform.translation.truncate())
                < 50.0
            {
                energy_map.0.remove(&c.0);
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Revolving
        if birth_ts.0.elapsed_past(15.0) && birth_ts.0.elapsed_within(18.0) {
            if birth_place
                .0
                .distance_squared(transform.translation.truncate())
                < 25000.0
            {
                energy_map.0.remove(&c.0);
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Traveling in one direction only
        if birth_ts.0.elapsed_past(20.0) && birth_ts.0.elapsed_within(30.0) {
            let x_disp = (transform.translation.x - birth_place.0.x).abs();
            let y_disp = (transform.translation.y - birth_place.0.y).abs();

            if x_disp * 3.0 < y_disp || y_disp * 3.0 < x_disp {
                energy_map.0.remove(&c.0);
                commands.entity(entity).despawn();
                continue;
            }
        }

        // No bullets fired
        if last_bullet_fired.0.elapsed_past(8.0) {
            match energy_map.0.get_mut(&c.0) {
                Some((v, i)) => {
                    *v -= NO_BULLET_PENALTY;
                    *i = Instant::now();
                }
                None => {}
            }
        }
    }
}

fn update_cell_sprite(
    settings: Res<SimSettings>,
    asset_server: Res<AssetServer>,
    stats: Res<SimStats>,
    energy_map: Res<EnergyMap>,
    mut cell_query: Query<(&Cell, &BirthTs, &Transform, &mut Handle<Image>), With<Cell>>,
) {
    if !settings.follow_best && !settings.follow_oldest {
        return;
    }

    for (cell, birth_ts, transform, mut image_handle) in cell_query.iter_mut() {
        if settings.follow_best
            && transform
                .translation
                .truncate()
                .distance_squared(stats.best_cell_pos)
                <= 200.0
        {
            match energy_map.0.get(&cell.0) {
                Some((v, _)) => {
                    if *v == stats.max_score {
                        *image_handle = asset_server.load(FOCUSED_CELL_SPRITE);
                    }
                }
                None => {}
            }
        } else if settings.follow_oldest
            && transform
                .translation
                .truncate()
                .distance_squared(stats.oldest_cell_pos)
                <= 200.0
            && birth_ts.0.elapsed() as i32 == stats.max_age as i32
        {
            *image_handle = asset_server.load(FOCUSED_CELL_SPRITE);
        } else {
            *image_handle = asset_server.load(CELL_SPRITE);
        }
    }
}

fn update_cells_system(
    mut commands: Commands,
    one_second_timer: Res<OneSecondTimer>,
    asset_server: Res<AssetServer>,
    food_tree: Res<FoodTree>,
    focused_cell_stats: Res<FocusedCellStats>,
    mut focused_cell_net: ResMut<FocusedCellNet>,
    mut cell_query: Query<
        (
            &Cell,
            &mut Transform,
            &Brain,
            &mut ExternalForce,
            &mut LastUpdated,
            &mut LastBulletFired,
            &mut FitnessScores,
            &PeriodicUpdateInterval,
        ),
        (With<Cell>, Without<UserControlledCell>),
    >,
) {
    for (
        cell,
        mut transform,
        brain,
        mut external_force,
        mut last_updated,
        mut last_bullet_fired,
        mut fitness_scores,
        periodic_update_interval,
    ) in cell_query.iter_mut()
    {
        if last_updated.0.elapsed_within(UPDATE_INTERVAL) {
            continue;
        }
        if one_second_timer
            .0
            .elapsed_within(periodic_update_interval.0)
        {
            continue;
        }

        last_updated.0.set_instant_now();
        let mut target_x = 0.0;
        let mut target_y = 0.0;
        // Get the closest food
        let key = (transform.translation.x, transform.translation.y);
        if let Some(t) = &food_tree.0 {
            match t.nearest(&[key.0 as f32, key.1 as f32]) {
                Some(v) => {
                    let [x, y] = v.item;
                    target_x = *x;
                    target_y = *y;
                }
                None => {}
            }
        }

        // NN Inputs
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
        let nn_target_angle = if nn_inp_angle < 0.0 {
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
        // let angle_diff = nn_cell_angle / 360.0;

        // Update brain
        let input = [
            nn_inp_dist as f64,
            nn_target_angle as f64,
            nn_cell_angle as f64,
        ];
        let output = &brain.0.predict(&input.to_vec());
        if focused_cell_stats.id == cell.0 {
            focused_cell_net.0 = output.clone();
        }

        let output = &output[NET_ARCH.len() - 1];
        let mut spin_left = false;
        let mut spin_right = false;
        let thrust = output[2] >= 0.7;
        let shoot = output[3] >= 0.7;

        if output[0] > output[1] {
            spin_left = true;
        } else if output[1] > output[0] {
            spin_right = true;
        }

        let fitness = calc_fitness(input, [output[0], output[1], output[2], output[3]]);
        fitness_scores.push(fitness);

        let action = CellAction {
            thrust,
            spin_left,
            spin_right,
            shoot,
        };
        perform_cell_action(
            action,
            cell.0,
            &mut last_bullet_fired,
            &mut external_force,
            &mut commands,
            &mut transform,
            &asset_server,
        );
    }
}

pub fn perform_cell_action(
    action: CellAction,
    cell_id: u32,
    last_bullet_fired: &mut LastBulletFired,
    external_force: &mut ExternalForce,
    commands: &mut Commands,
    transform: &mut Transform,
    asset_server: &AssetServer,
) {
    let spin_strength = 0.5;

    // Apply Cell force
    let angle = (transform.rotation.to_euler(EulerRot::XYZ).2 + PI / 2.0) as f64;
    if action.thrust {
        external_force.force = vec2(angle.cos() as f32, angle.sin() as f32) * CELL_SPEED;
    } else {
        external_force.force = Vec2::ZERO;
    }
    // Apply Spin
    if action.spin_left {
        transform.rotate_z(spin_strength);
    } else if action.spin_right {
        transform.rotate_z(-spin_strength);
    }

    if !action.shoot {
        return;
    }
    if last_bullet_fired.0.elapsed_within(BULLET_FIRE_RATE) {
        return;
    }

    // Bullet spawn
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + PI / 2.0;
    let x = angle.cos();
    let y = angle.sin();
    let direction = vec2(x, y);
    let x = transform.translation.x + (x * 5.0);
    let y = transform.translation.y + (y * 5.0);
    last_bullet_fired.0.set_instant_now();
    commands.spawn(BulletBundle::new(
        x,
        y,
        cell_id,
        direction * BULLET_SPEED,
        &asset_server,
    ));
}

fn cell_replication_system(
    mut commands: Commands,
    mut cell_id: ResMut<CellId>,
    energy_map: Res<EnergyMap>,
    stats: Res<SimStats>,
    asset_server: Res<AssetServer>,
    mut cell_query: Query<(&Cell, &Brain, &mut NumCellsSpawned), With<Cell>>,
) {
    let mut num_cells = cell_query.iter().len();
    for (c, brain, mut num_cells_spawned) in cell_query.iter_mut() {
        let mut rng = rand::thread_rng();
        if num_cells >= NUM_CELLS {
            continue;
        }

        match energy_map.0.get(&c.0) {
            Some((v, _)) => {
                if rng.gen_range(0.0..1.0) >= (v / stats.max_score) {
                    continue;
                }
                // if rng.gen_range(0.0..100.0) >= (birth_ts.0.elapsed() / stats.max_age) * 20.0 {
                //     continue;
                // }
                // if rng.gen_range(0.0..100.0) >= 20.0 {
                //     continue;
                // }

                let x = rng.gen_range(-(W as f32) / 2.0..W as f32 / 2.0);
                let y = rng.gen_range(-(H as f32) / 2.0..H as f32 / 2.0);
                let mut child_net = brain.0.clone();
                child_net.mutate();

                cell_id.0 += 1;
                num_cells += 1;

                num_cells_spawned.0 += 1;
                commands.spawn(CellBundle::new(
                    x,
                    y,
                    cell_id.0,
                    child_net,
                    CELL_SPRITE,
                    &asset_server,
                ));
            }
            None => {}
        }
    }
}

fn spawn_cells(
    mut commands: Commands,
    mut cell_id: ResMut<CellId>,
    asset_server: Res<AssetServer>,
    cell_query: Query<(With<Cell>, Without<UserControlledCell>)>,
) {
    let num_cells = cell_query.iter().len();
    if num_cells > 0 {
        return;
    }

    let mut rng = rand::thread_rng();
    for _ in 0..NUM_CELLS {
        let x = rng.gen_range(-(W as f32) / 2.0..W as f32 / 2.0);
        let y = rng.gen_range(-(H as f32) / 2.0..H as f32 / 2.0);
        let net = Net::new(NET_ARCH.to_vec());

        cell_id.0 += 1;
        commands.spawn(CellBundle::new(
            x,
            y,
            cell_id.0,
            net,
            CELL_SPRITE,
            &asset_server,
        ));
    }
}

fn calc_fitness(inp: [f64; NUM_INPUT_NODES], out: [f64; NUM_OUTPUT_NODES]) -> f32 {
    // Inp
    // 1 - dist between cell and target
    // 2 - angle diff between cell and target
    let dist = inp[0];
    let target_angle = inp[1];
    let cell_angle = inp[2];

    // Out
    // 0 & 1 - spin direction
    // 0 > 1 - spin left
    // 0 < 1 - spin right
    // 2 - thrust when >= 0.7
    // 3 - shoot when >= 0.7
    let spin_l = out[0] > out[1];
    let spin_r = out[1] > out[0];
    let thrust = out[2] >= 0.7;
    let shoot = out[3] >= 0.7;

    let mut score = 0.0;
    let scale = 1.0;

    // Rules
    if dist > 0.3 && thrust {
        score += scale;
    }
    if cell_angle < target_angle && spin_l {
        score += scale;
    } else if cell_angle > target_angle && spin_r {
        score += scale;
    }
    if dist < 0.5 && shoot {
        score += scale;
    }

    (4.0 * scale) - score
}

fn angle_between(a: f32, b: f32, x: f32, y: f32) -> f32 {
    let angle_radians = (y - b).atan2(x - a);
    let mut angle_degrees = angle_radians.to_degrees();
    if angle_degrees < 0.0 {
        angle_degrees += 360.0;
    }

    angle_degrees
}
