use ava::{
    bullet::BulletPlugin, camera::FollowCameraPlugin, cell::CellPlugin, food::FoodPlugin,
    gui::GuiPlugin, physics::PhysicsPlugin, settings::SettingsPlugin, trackers::TrackersPlugin, *,
};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

#[derive(Component)]
struct UICameraFollower;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // mode: bevy::window::WindowMode::Fullscreen,
                        resolution: (WW as f32, WH as f32).into(),
                        title: "Ava".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, bevy::window::close_on_esc)
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
        .add_plugins(GuiPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(TrackersPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(FollowCameraPlugin)
        .add_plugins(BulletPlugin)
        .add_plugins(FoodPlugin)
        .add_plugins(CellPlugin)
        .run();
}
