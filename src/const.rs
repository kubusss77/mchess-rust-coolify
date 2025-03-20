pub const PV_MOVE: f64 = 20000.0;
pub const MVV_LVA_VALUE: f64 = 10000.0;
pub const PROMOTION_VALUE: f64 = 9000.0;
pub const KILLER_MOVE_VALUE: f64 = 5000.0;
pub const CAPTURE_VALUE: f64 = 1000.0;
pub const CHECK_VALUE: f64 = 500.0;
pub const CASTLING_VALUE: f64 = 300.0;

pub const PAWN_DEVELOPMENT_BONUS: f64 = 500.0;
pub const PAWN_ISOLATION_PENALTY: f64 = 0.2;
pub const MOBILITY_VALUE: f64 = 0.05;
pub const NO_SAFETY_PENALTY: f64 = 0.8;
pub const LOW_SAFETY_PENALTY: f64 = 0.5; 

pub const MOVE_PREALLOC: usize = 30;
pub const MAX_PLIES: u8 = 50;