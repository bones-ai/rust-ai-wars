use std::time::Duration;

use bevy::{math::vec3, prelude::*, time::common_conditions::on_timer};
use bevy_egui::{
    egui::{
        self,
        epaint::CircleShape,
        plot::{Line, Plot, PlotPoints},
        pos2, Color32, Shape, Stroke,
    },
    EguiContexts, EguiPlugin, EguiSettings,
};

use crate::{
    bullet::Bullet,
    camera::FollowCamera,
    cell::{
        energy::EnergyMap,
        focus::{FocusedCell, FocusedCellNet, FocusedCellStats, UnFocusCellEvent},
        Cell,
    },
    food::{Food, FoodTree},
    settings::{DynamicSettings, SimSettings},
    trackers::{BirthTs, InstantTracker},
    *,
};

pub struct GuiPlugin;

#[derive(PartialEq, Eq)]
enum Panel {
    Stats,
    Graphs,
    Network,
    Settings,
}
#[derive(Resource)]
struct SelectedPanel(Panel);

#[derive(Resource)]
pub struct SimStats {
    pub max_score: f32,
    pub max_age: f32,
    pub best_cell_pos: Vec2,
    pub oldest_cell_pos: Vec2,
    pub sim_start_ts: InstantTracker,
}

#[derive(Resource, Default)]
struct GraphPoints {
    score: Vec<f32>,
    age: Vec<f32>,
    num_cells: Vec<f32>,
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(SimStats::new())
            .insert_resource(SelectedPanel(Panel::Stats))
            .insert_resource(GraphPoints::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update_stats)
            .add_systems(Update, handle_mouse_btn_click)
            .add_systems(
                Update,
                update_graph_points.run_if(on_timer(Duration::from_secs_f32(1.0))),
            )
            .add_systems(Update, update_side_panel);
    }
}

fn setup(mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 2.0;
}

fn update_side_panel(
    mut contexts: EguiContexts,
    mut panel: ResMut<SelectedPanel>,
    energy_map: Res<EnergyMap>,
    stats: Res<SimStats>,
    graph_points: Res<GraphPoints>,
    focused_cell_stats: Res<FocusedCellStats>,
    mut settings: ResMut<SimSettings>,
    mut dynamic_settings: ResMut<DynamicSettings>,
    food_tree: Res<FoodTree>,
    best_brain: Res<FocusedCellNet>,
    cells_query: Query<(&Cell, &Transform), With<Cell>>,
    food_query: Query<With<Food>>,
    bullet_query: Query<With<Bullet>>,
) {
    if !settings.show_side_panel {
        return;
    }

    let ctx = contexts.ctx_mut();
    let tree_size = match &food_tree.0 {
        Some(v) => v.len(),
        None => 0,
    };

    egui::SidePanel::left("left-side-panel")
        .min_width(200.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut panel.0, Panel::Stats, Panel::Stats.get_label());
                ui.selectable_value(&mut panel.0, Panel::Graphs, Panel::Graphs.get_label());
                ui.selectable_value(&mut panel.0, Panel::Network, Panel::Network.get_label());
                ui.selectable_value(&mut panel.0, Panel::Settings, Panel::Settings.get_label());
            });
            ui.separator();

            match panel.0 {
                Panel::Stats => {
                    egui::CollapsingHeader::new("Stats")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(format!("Cells: {:?}", cells_query.iter().len()));
                            ui.label(format!("Food: {:?}", food_query.iter().len()));
                            ui.label(format!("Bullets: {:?}", bullet_query.iter().len()));
                            ui.label(format!("Max Fitness: {:?}", stats.max_score));
                            ui.label(format!("Max Lifespan: {:.2}", stats.max_age));
                        });
                    egui::CollapsingHeader::new("Cell")
                        .default_open(true)
                        .show(ui, |ui| match !focused_cell_stats.is_cell_focused() {
                            true => {
                                ui.label("No cell selected");
                            }
                            false => {
                                ui.label(format!("Id: {:?}", focused_cell_stats.id));
                                ui.label(format!("Score: {:?}", focused_cell_stats.score));
                                ui.label(format!("Age: {:.1}", focused_cell_stats.age));
                                ui.label(format!(
                                    "Pos: ({:.1}, {:.1})",
                                    focused_cell_stats.pos.x, focused_cell_stats.pos.y
                                ));
                                ui.label(format!(
                                    "Spawned: {:?}",
                                    focused_cell_stats.num_cells_spawned
                                ));
                                ui.label(format!(
                                    "Fitness: {:?}",
                                    focused_cell_stats.fitness_score
                                ));
                            }
                        });
                    egui::CollapsingHeader::new("Debug")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(format!("Energy Map: {:?}", energy_map.0.len()));
                            ui.label(format!("Food Tree: {:?}", tree_size));
                            ui.label(format!(
                                "Runtime: {:.1} m",
                                stats.sim_start_ts.elapsed() / 60.0
                            ));
                        });
                }
                Panel::Graphs => {
                    let line1 = Line::new(
                        (0..graph_points.score.len())
                            .map(|i| [i as f64, graph_points.score[i] as f64])
                            .collect::<PlotPoints>(),
                    );
                    let line2 = Line::new(
                        (0..graph_points.age.len())
                            .map(|i| [i as f64, graph_points.age[i] as f64])
                            .collect::<PlotPoints>(),
                    );
                    let line3 = Line::new(
                        (0..graph_points.num_cells.len())
                            .map(|i| [i as f64, graph_points.num_cells[i] as f64])
                            .collect::<PlotPoints>(),
                    );
                    let aspect = 1.8;
                    egui::CollapsingHeader::new("Score")
                        .default_open(true)
                        .show(ui, |ui| {
                            Plot::new("score")
                                .view_aspect(aspect)
                                .show(ui, |plot_ui| plot_ui.line(line1));
                        });
                    egui::CollapsingHeader::new("Max Lifespan")
                        .default_open(true)
                        .show(ui, |ui| {
                            Plot::new("lifespan")
                                .view_aspect(aspect)
                                .show(ui, |plot_ui| plot_ui.line(line2));
                        });
                    egui::CollapsingHeader::new("Cells Count")
                        .default_open(true)
                        .show(ui, |ui| {
                            Plot::new("count")
                                .view_aspect(aspect)
                                .show(ui, |plot_ui| plot_ui.line(line3));
                        });
                }
                Panel::Network => {
                    let shapes = get_nn_shapes(&best_brain);
                    if shapes.is_empty() {
                        ui.label("Select a cell first");
                    }
                    shapes.iter().for_each(|s| {
                        ui.painter().add(s.clone());
                    });
                }
                Panel::Settings => {
                    egui::CollapsingHeader::new("Camera")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.checkbox(&mut settings.follow_best, "Follow Best");
                            ui.checkbox(&mut settings.follow_oldest, "Follow Oldest");
                            ui.checkbox(&mut settings.follow_player, "Follow Player");
                            ui.checkbox(&mut settings.follow_focused_cell, "Follow Focused Cell");
                        });
                    egui::CollapsingHeader::new("Others")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("Bullet miss penalty");
                            ui.add(
                                egui::DragValue::new(&mut dynamic_settings.bullet_miss_penalty)
                                    .speed(1.0)
                                    .clamp_range(0.0..=300.0),
                            );
                            ui.label("Energy per food");
                            ui.add(
                                egui::DragValue::new(&mut dynamic_settings.energy_per_food)
                                    .speed(1.0)
                                    .clamp_range(0.0..=1000.0),
                            );
                            ui.label("Energy decay rate");
                            ui.add(
                                egui::DragValue::new(&mut dynamic_settings.energy_decay_rate)
                                    .speed(1.0)
                                    .clamp_range(0.0..=1000.0),
                            );
                            ui.label("Num food");
                            ui.add(egui::DragValue::new(&mut dynamic_settings.num_food).speed(1.0));
                        });
                }
            }
        });
}

fn update_graph_points(
    stats: Res<SimStats>,
    mut graph_points: ResMut<GraphPoints>,
    cells_query: Query<With<Cell>>,
) {
    graph_points.add_age(stats.max_age);
    graph_points.add_score(stats.max_score);
    graph_points.add_num_cells(cells_query.iter().count() as f32);
}

fn handle_mouse_btn_click(
    mut commands: Commands,
    windows: Query<&Window>,
    mut writer: EventWriter<UnFocusCellEvent>,
    mouse: Res<Input<MouseButton>>,
    cam_query: Query<(&Camera, &GlobalTransform), With<FollowCamera>>,
    cells_query: Query<(&Transform, Entity), (With<Cell>, Without<FocusedCell>)>,
    focused_cells_query: Query<(&Cell, &Transform), With<FocusedCell>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = cam_query.single();

    let world_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor));
    if world_pos.is_none() {
        return;
    }
    let world_pos = world_pos.unwrap();
    let (x, y) = (world_pos.x, world_pos.y);

    for (t, e) in cells_query.iter() {
        let dist = t.translation.distance_squared(vec3(x, y, 0.0));
        if dist <= 100.0 {
            // Remove all cells that are focused
            for (c, _) in focused_cells_query.iter() {
                writer.send(UnFocusCellEvent(c.0));
            }
            // When marking a cell as focused,
            // We simply add the component to the entity so that the cell system can update their sprite
            commands.entity(e).insert(FocusedCell);
            return;
        }
    }
    for (c, t) in focused_cells_query.iter() {
        let dist = t.translation.distance_squared(vec3(x, y, 0.0));
        if dist <= 100.0 {
            // When un-focusing a cell, we create an event with index
            // In the cell system, we get all the focused cells, then loop over all focused cells
            // Then unfocus this cell by id (ie remove the focus component)
            // If the focus component is removed here, then we'll have to iterate over all cells to find the cell by id
            writer.send(UnFocusCellEvent(c.0));
            return;
        }
    }
}

fn update_stats(
    mut stats: ResMut<SimStats>,
    energy_map: Res<EnergyMap>,
    cells_query: Query<(&Cell, &BirthTs, &Transform), With<Cell>>,
) {
    let mut max_score = 0.0;
    let mut max_age = 0.0;
    let mut best_cell_pos = Vec3::ZERO;
    let mut oldest_cell_pos = Vec3::ZERO;

    for (c, birth_ts, transform) in cells_query.iter() {
        let score = match energy_map.0.get(&c.0) {
            Some((v, _)) => *v,
            None => 0.0,
        };
        if score > max_score {
            max_score = score;
            best_cell_pos = transform.translation;
        }

        let age = birth_ts.0.elapsed();
        if age > max_age {
            max_age = age;
            oldest_cell_pos = transform.translation;
        }
    }

    stats.max_score = max_score;
    stats.max_age = max_age;
    stats.best_cell_pos = best_cell_pos.truncate();
    stats.oldest_cell_pos = oldest_cell_pos.truncate();
}

fn get_nn_shapes(best_brain: &FocusedCellNet) -> Vec<Shape> {
    if best_brain.0.is_empty() {
        return Vec::new();
    }

    let mut shapes = Vec::new();
    let tot_height = 450.0;

    // Padding
    let padding_top = 30.0;
    // NN viz points
    let points1 = get_nn_viz_points(NUM_INPUT_NODES as usize, tot_height);
    let points2 = get_nn_viz_points(NUM_HIDDEN_NODES as usize, tot_height);
    let points3 = get_nn_viz_points(NUM_OUTPUT_NODES as usize, tot_height);
    // NN output
    let values1 = best_brain.0[0].clone();
    let values2 = best_brain.0[1].clone();
    let values3 = best_brain.0[2].clone();
    // x's
    let x1 = 25.0;
    let x2 = 100.0;
    let x3 = 175.0;

    // colors
    let colors1: Vec<Color32> = values1
        .iter()
        .rev()
        .map(|v| {
            if *v <= 0.7 {
                Color32::GREEN
            } else {
                Color32::RED
            }
        })
        .collect();
    let colors2: Vec<Color32> = values2
        .iter()
        .rev()
        .map(|v| {
            if *v > 0.5 {
                Color32::GREEN
            } else {
                Color32::RED
            }
        })
        .collect();
    let mut colors3 = vec![Color32::RED, Color32::RED, Color32::RED, Color32::RED];
    colors3[0] = if values3[0] >= values3[1] {
        Color32::GREEN
    } else {
        Color32::RED
    };
    colors3[1] = if values3[1] >= values3[0] {
        Color32::GREEN
    } else {
        Color32::RED
    };
    colors3[2] = if values3[2] >= 0.7 {
        Color32::GREEN
    } else {
        Color32::RED
    };
    colors3[3] = if values3[3] >= 0.7 {
        Color32::GREEN
    } else {
        Color32::RED
    };

    // layer 1 -> 2 lines
    for (p1, c1) in points1.iter().zip(colors1.iter()) {
        for (p2, c2) in points2.iter().zip(colors2.iter()) {
            let mut color = Color32::RED;
            if are_colors_equal(*c1, *c2) {
                color = Color32::GREEN;
            }
            shapes.push(egui::Shape::line(
                vec![pos2(x1, *p1 + padding_top), pos2(x2, *p2 + padding_top)],
                Stroke { width: 1.0, color },
            ));
        }
    }

    // layer 2 -> 3 lines
    for (p2, c2) in points2.iter().zip(colors2.iter()) {
        for (p3, c3) in points3.iter().zip(colors3.iter()) {
            let mut color = Color32::RED;
            if are_colors_equal(*c2, *c3) {
                color = Color32::GREEN;
            }
            shapes.push(egui::Shape::line(
                vec![pos2(x2, *p2 + padding_top), pos2(x3, *p3 + padding_top)],
                Stroke { width: 1.0, color },
            ));
        }
    }

    // layer 1
    let padding = padding_top;
    for (p, c) in points1.iter().zip(colors1.iter()) {
        shapes.push(get_nn_node_shape(x1, *p + padding, *c));
    }
    // layer 2
    let padding = padding_top;
    for (p, c) in points2.iter().zip(colors2.iter()) {
        shapes.push(get_nn_node_shape(x2, *p + padding, *c));
    }
    // layer 3
    let padding = padding_top;
    for (p, c) in points3.iter().zip(colors3.iter()) {
        shapes.push(get_nn_node_shape(x3, *p + padding, *c));
    }

    shapes
}

fn get_nn_node_shape(x: f32, y: f32, color: Color32) -> egui::Shape {
    egui::Shape::Circle(CircleShape {
        center: (x, y).into(),
        radius: NN_NODE_SIZE,
        fill: color,
        stroke: Stroke {
            width: 1.0,
            color: Color32::WHITE,
        },
    })
}

fn get_nn_viz_points(n: usize, tot_size: f32) -> Vec<f32> {
    let point_spacing = (tot_size) / (n + 1) as f32;
    let mut points = Vec::new();

    for i in 1..=n {
        let x = i as f32 * point_spacing;
        points.push(x);
    }

    points
}

fn are_colors_equal(first: Color32, second: Color32) -> bool {
    (first.g() == 255 && second.g() == 255) || (first.r() == 255 && second.r() == 255)
}

impl SimStats {
    fn new() -> Self {
        Self {
            best_cell_pos: Vec2::ZERO,
            max_age: 0.0,
            max_score: 0.0,
            oldest_cell_pos: Vec2::ZERO,
            sim_start_ts: InstantTracker::default(),
        }
    }
}

impl Panel {
    fn get_label(&self) -> &str {
        match self {
            Panel::Stats => "Stats",
            Panel::Graphs => "Graphs",
            Panel::Network => "Network",
            Panel::Settings => "Settings",
        }
    }
}

impl GraphPoints {
    pub fn add_score(&mut self, value: f32) {
        self.score.push(value);
        if self.score.len() > MAX_GRAPH_POINTS {
            self.score.remove(0);
        }
    }

    pub fn add_age(&mut self, value: f32) {
        self.age.push(value);
        if self.age.len() > MAX_GRAPH_POINTS {
            self.age.remove(0);
        }
    }

    pub fn add_num_cells(&mut self, value: f32) {
        self.num_cells.push(value);
        if self.num_cells.len() > MAX_GRAPH_POINTS {
            self.num_cells.remove(0);
        }
    }
}
