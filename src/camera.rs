use bevy::{math::vec3, prelude::*};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::{
    cell::{focus::FocusedCellStats, user::UserControlledCell, Cell},
    gui::SimStats,
    settings::SimSettings,
};

pub struct FollowCameraPlugin;

#[derive(Component)]
pub struct FollowCamera;

impl Plugin for FollowCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Update, follow_player_system)
            .add_systems(Update, follow_focused_cell_system)
            .add_systems(Update, follow_best_cells_system);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default())
        .insert(FollowCamera);
}

fn follow_player_system(
    sim_settings: Res<SimSettings>,
    player_query: Query<&Transform, (With<UserControlledCell>, With<Cell>)>,
    mut cam_query: Query<(&Camera, &mut Transform), Without<Cell>>,
) {
    if !sim_settings.follow_player {
        return;
    }
    if player_query.is_empty() {
        return;
    }

    let transform = player_query.single();
    let (_, mut cam_transform) = cam_query.get_single_mut().unwrap();
    cam_transform.translation = cam_transform.translation.lerp(
        vec3(transform.translation.x, transform.translation.y, 0.0),
        0.1,
    );
}

fn follow_focused_cell_system(
    sim_settings: Res<SimSettings>,
    stats: Res<FocusedCellStats>,
    mut cam_query: Query<(&Camera, &mut Transform), Without<Cell>>,
) {
    if sim_settings.follow_player {
        return;
    }
    if !sim_settings.follow_focused_cell || !stats.is_cell_focused() {
        return;
    }

    let padding = if sim_settings.show_side_panel {
        -100.0
    } else {
        0.0
    };
    let (_, mut cam_transform) = cam_query.get_single_mut().unwrap();
    cam_transform.translation = cam_transform
        .translation
        .lerp(vec3(stats.pos.x + padding, stats.pos.y, 0.0), 0.1);
}

fn follow_best_cells_system(
    sim_settings: Res<SimSettings>,
    stats: Res<SimStats>,
    mut cam_query: Query<(&Camera, &mut Transform), Without<Cell>>,
) {
    if sim_settings.follow_player || sim_settings.follow_focused_cell {
        return;
    }

    let padding = if sim_settings.show_side_panel {
        -100.0
    } else {
        0.0
    };
    if sim_settings.follow_best {
        let (_, mut cam_transform) = cam_query.get_single_mut().unwrap();
        cam_transform.translation = cam_transform.translation.lerp(
            vec3(stats.best_cell_pos.x + padding, stats.best_cell_pos.y, 0.0),
            0.1,
        );
        return;
    }

    if sim_settings.follow_oldest {
        let (_, mut cam_transform) = cam_query.get_single_mut().unwrap();
        cam_transform.translation = cam_transform.translation.lerp(
            vec3(
                stats.oldest_cell_pos.x + padding,
                stats.oldest_cell_pos.y,
                0.0,
            ),
            0.1,
        );
    }
}
