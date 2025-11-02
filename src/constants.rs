
use std::time::Duration;

// Constants for cycle times
// These are definitely inaccurate, here for tweaking
pub const CYCLES_PER_FRAME: usize = GB_CYCLE_SPEED / 60;
pub const FRAME_TIME: Duration = Duration::from_nanos(16_670_000);
pub const GB_CYCLE_SPEED: usize = 4_194_304;

// GPU Output settings
pub const WINDOW_HEIGHT: u32 = 144;
pub const WINDOW_WIDTH: u32 = 160;
pub const MAP_WIDTH: u32 = 256;
