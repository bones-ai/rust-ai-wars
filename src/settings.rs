use bevy::prelude::*;

use crate::*;

pub struct SettingsPlugin;

#[derive(Resource)]
pub struct SimSettings {
    pub follow_best: bool,
    pub follow_player: bool,
    pub follow_oldest: bool,
    pub show_side_panel: bool,
    pub follow_focused_cell: bool,
}

#[derive(Resource)]
pub struct DynamicSettings {
    pub bullet_miss_penalty: f32,
    pub energy_per_food: f32,
    pub energy_decay_rate: f32,
    pub num_food: usize,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            follow_best: false,
            follow_player: false,
            follow_oldest: false,
            show_side_panel: false,
            follow_focused_cell: false,
        }
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimSettings::default())
            .insert_resource(DynamicSettings::new())
            .add_systems(Update, handle_keyboard_input);
    }
}

fn handle_keyboard_input(keyboard_input: Res<Input<KeyCode>>, mut settings: ResMut<SimSettings>) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        settings.show_side_panel = !settings.show_side_panel;
    }
    if keyboard_input.just_pressed(KeyCode::B) {
        settings.follow_best = !settings.follow_best;
    }
    if keyboard_input.just_pressed(KeyCode::O) {
        settings.follow_oldest = !settings.follow_oldest;
    }
    if keyboard_input.just_pressed(KeyCode::P) {
        settings.follow_player = !settings.follow_player;
    }
    if keyboard_input.just_pressed(KeyCode::C) {
        settings.follow_focused_cell = !settings.follow_focused_cell;
    }
}

impl DynamicSettings {
    fn new() -> Self {
        Self {
            bullet_miss_penalty: BULLET_MISS_PENALTY,
            energy_per_food: ENERGY_PER_FOOD,
            num_food: NUM_FOOD,
            energy_decay_rate: ENERGY_DECAY_RATE,
        }
    }
}
