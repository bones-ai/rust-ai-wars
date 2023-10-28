// Windowing
pub const WW: usize = 900;
pub const WH: usize = 700;
pub const BG_COLOR: (u8, u8, u8) = (195, 232, 208);

// Environment
pub const W: usize = 10000;
pub const H: usize = 10000;

// GUI
pub const MAX_GRAPH_POINTS: usize = 1500;
pub const NN_NODE_SIZE: f32 = 10.0;

// Cell
pub const NUM_CELLS: usize = 4000;
pub const CELL_SPEED: f32 = 1.0;
pub const BASE_ENERGY: f32 = 100.0;
pub const ENERGY_UPDATE_INTERVAL_SECS: f32 = 1.0;
pub const ENERGY_DECAY_RATE: f32 = 5.0;
pub const UPDATE_INTERVAL: f32 = 0.5;
pub const VISION_RADIUS: f32 = 200.0;
pub const MAX_ENERGY: f32 = 4000.0;
pub const IS_USER_ENABLED: bool = false;
pub const CELL_SPRITE: &str = "turret.png";
pub const FOCUSED_CELL_SPRITE: &str = "turret-focused.png";
pub const USER_CELL_SPRITE: &str = "turret-focused.png";

// Bullet
pub const BULLET_LIFESPAN: f32 = 1.0;
pub const BULLET_SPEED: f32 = 200.0;
pub const BULLET_FIRE_RATE: f32 = 1.0;
pub const BULLET_MISS_PENALTY: f32 = 5.0;
pub const NO_BULLET_PENALTY: f32 = 30.0;
pub const BULLET_SPRITE: &str = "brown-ball.png";

// Food
pub const NUM_FOOD: usize = 5000;
pub const ENERGY_PER_FOOD: f32 = 70.0;
pub const FOOD_REFRESH_INTERVAL_SECS: f32 = 0.5;
pub const FOOD_TREE_REFRESH_RATE_SECS: f32 = 1.0;
pub const FOOD_SPRITE: &str = "red-dot.png";

// NN
pub const NUM_INPUT_NODES: usize = 3;
pub const NUM_HIDDEN_NODES: usize = 8;
pub const NUM_OUTPUT_NODES: usize = 4;
pub const NET_ARCH: [usize; 3] = [NUM_INPUT_NODES, NUM_HIDDEN_NODES, NUM_OUTPUT_NODES];
pub const BRAIN_MUTATION_RATE: f32 = 0.1;
pub const BRAIN_MUTATION_VARIATION: f32 = 0.1;

/// Collision groups
/// bit 1 - Cells
/// bit 2 - Food
/// bit 3 - Bullet
pub const GRP_CELLS: u32 = 0b1000;
pub const GRP_FOOD: u32 = 0b0100;
pub const GRP_BULLET: u32 = 0b0010;
pub const MASK_CELLS: u32 = 0b0000;
pub const MASK_FOOD: u32 = 0b0010;
pub const MASK_BULLET: u32 = 0b0100;
