use color_eyre::Result;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Constants for cycle times
// These are definitely inaccurate, here for tweaking
pub const CYCLES_PER_FRAME: usize = GB_CYCLE_SPEED / 60;
pub const FRAME_TIME: Duration = Duration::from_nanos(16_670_000);
pub const GB_CYCLE_SPEED: usize = 4_194_304;

// GPU Output settings
pub const WINDOW_HEIGHT: u32 = 144;
pub const WINDOW_WIDTH: u32 = 160;
pub const MAP_WIDTH: u32 = 256;

pub fn setup_logger() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false).without_time())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()?;
    Ok(())
}
