use std::time::{Duration, Instant};

use bevy::prelude::*;

use crate::{
    trackers::{BirthTs, FitnessScores, InstantTracker, NumCellsSpawned},
    *,
};

use super::{cell::Cell, energy::EnergyMap};

pub struct CellFocusPlugin;

#[derive(Component)]
pub struct FocusedCell;

#[derive(Resource, Default)]
pub struct FocusedCellNet(pub Vec<Vec<f64>>);

#[derive(Event)]
pub struct UnFocusCellEvent(pub u32);

#[derive(Resource)]
pub struct FocusedCellStats {
    pub id: u32,
    pub score: f32,
    pub age: f32,
    pub pos: Vec2,
    pub last_updated: InstantTracker,
    pub num_cells_spawned: u32,
    pub fitness_score: f32,
}

impl Plugin for CellFocusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FocusedCellStats::new())
            .insert_resource(FocusedCellNet::default())
            .add_event::<UnFocusCellEvent>()
            .add_systems(Update, update_focused_cell_stats)
            .add_systems(Update, update_focused_cell);
    }
}

fn update_focused_cell_stats(
    mut stats: ResMut<FocusedCellStats>,
    energy_map: Res<EnergyMap>,
    cells_query: Query<
        (
            &Cell,
            &BirthTs,
            &Transform,
            &NumCellsSpawned,
            &FitnessScores,
        ),
        With<FocusedCell>,
    >,
) {
    if let Some((c, birth_ts, transform, num_cells_spawned, fitness_score)) =
        cells_query.iter().next()
    {
        let id = c.0;
        let score = match energy_map.0.get(&c.0) {
            Some((v, _)) => *v,
            None => 0.0,
        };
        let age = birth_ts.0.elapsed();
        let pos = transform.translation.truncate();
        stats.id = id;
        stats.score = score;
        stats.age = age;
        stats.pos = pos;
        stats.last_updated.set_instant_now();
        stats.num_cells_spawned = num_cells_spawned.0;
        stats.fitness_score = fitness_score.get_fitness();
    }
}

fn update_focused_cell(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut focused_cell_query: Query<(&mut Handle<Image>, &Cell, Entity), With<FocusedCell>>,
    mut reader: EventReader<UnFocusCellEvent>,
) {
    for (mut image_handle, _, _) in focused_cell_query.iter_mut() {
        *image_handle = asset_server.load(FOCUSED_CELL_SPRITE);
    }
    for e in reader.iter() {
        for (mut img_handle, cell, entity) in focused_cell_query.iter_mut() {
            if e.0 == cell.0 {
                commands.entity(entity).remove::<FocusedCell>();
                *img_handle = asset_server.load(CELL_SPRITE);
            }
        }
    }
}

impl FocusedCellStats {
    fn new() -> Self {
        Self {
            age: 0.0,
            id: 0,
            pos: Vec2::ZERO,
            score: 0.0,
            last_updated: InstantTracker(
                Instant::now()
                    .checked_sub(Duration::from_secs_f32(10.0))
                    .unwrap(),
            ),
            num_cells_spawned: 0,
            fitness_score: 1.0,
        }
    }

    pub fn is_cell_focused(&self) -> bool {
        !self.last_updated.elapsed_past(1.0)
    }
}
