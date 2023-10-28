use std::time::{Duration, Instant};

use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashMap};

use crate::{settings::DynamicSettings, trackers::FitnessScores, *};

use super::{user::UserControlledCell, Cell};

pub struct CellEnergyPlugin;

#[derive(Resource)]
pub struct EnergyMap(pub HashMap<u32, (f32, Instant)>);

impl Plugin for CellEnergyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnergyMap(HashMap::new())).add_systems(
            Update,
            update_cell_energy.run_if(on_timer(Duration::from_secs_f32(
                ENERGY_UPDATE_INTERVAL_SECS,
            ))),
        );
    }
}

fn update_cell_energy(
    mut energy_map: ResMut<EnergyMap>,
    settings: Res<DynamicSettings>,
    cell_query: Query<(&Cell, &FitnessScores), (With<Cell>, Without<UserControlledCell>)>,
) {
    for (cell, fitness) in cell_query.iter() {
        match energy_map.0.get_mut(&cell.0) {
            Some((v, i)) => {
                *v -= settings.energy_decay_rate * fitness.get_fitness();
                *i = Instant::now();
            }
            None => {
                energy_map.0.insert(cell.0, (BASE_ENERGY, Instant::now()));
            }
        }
    }

    energy_map
        .0
        .retain(|_, (_, i)| i.elapsed().as_secs_f32() < 10.0);
}
