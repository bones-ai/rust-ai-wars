use std::time::{Duration, Instant};

use bevy::{prelude::*, time::common_conditions::on_timer};

pub struct TrackersPlugin;

pub struct InstantTracker(pub Instant);

#[derive(Default, Component)]
pub struct LastUpdated(pub InstantTracker);
#[derive(Component)]
pub struct BirthPlace(pub Vec2);
#[derive(Default, Component)]
pub struct BirthTs(pub InstantTracker);
#[derive(Default, Component)]
pub struct LastBulletFired(pub InstantTracker);
#[derive(Component)]
pub struct NumCellsSpawned(pub u32);
#[derive(Component)]
pub struct PeriodicUpdateInterval(pub f32);
#[derive(Component)]
pub struct FitnessScores(Vec<f32>);

#[derive(Default, Resource)]
pub struct OneSecondTimer(pub InstantTracker);

impl Plugin for TrackersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OneSecondTimer::default()).add_systems(
            Update,
            update_second_timer.run_if(on_timer(Duration::from_secs_f32(1.0))),
        );
    }
}

fn update_second_timer(mut one_second_timer: ResMut<OneSecondTimer>) {
    one_second_timer.0.set_instant_now();
}

impl FitnessScores {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: f32) {
        if self.0.len() >= 10 {
            self.0.remove(0);
        }

        self.0.push(value);
    }

    pub fn get_fitness(&self) -> f32 {
        self.0.iter().sum::<f32>() / 10.0
    }
}

impl InstantTracker {
    pub fn elapsed(&self) -> f32 {
        self.get_instant().elapsed().as_secs_f32()
    }

    pub fn elapsed_past(&self, interval: f32) -> bool {
        self.get_instant().elapsed().as_secs_f32() >= interval
    }

    pub fn elapsed_within(&self, interval: f32) -> bool {
        self.get_instant().elapsed().as_secs_f32() < interval
    }

    pub fn get_instant(&self) -> Instant {
        self.0
    }

    pub fn set_instant_now(&mut self) {
        self.0 = Instant::now();
    }
}

impl Default for InstantTracker {
    fn default() -> Self {
        Self(Instant::now())
    }
}
