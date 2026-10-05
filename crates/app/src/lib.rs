//! Presentation shell for Cascade.
//!
//! The simulation crate is not a dependency. A later adapter can replace
//! [`PlaceholderSource`] with bytes and dirty chunks from `cascade-sim`.

mod camera;
mod grid;
mod metrics;
mod overlay;
mod surface;
mod upload;

pub use camera::{Camera, UNIFORM_BYTES, ViewCommand, apply_command, frame_uniform, zoom_factor};
pub use grid::{
    CHUNK_CELLS, CHUNK_SIZE, ChunkCoord, GridError, MAX_DIRTY_PER_TICK, MaterialGrid, PALETTE,
    PLACEHOLDER_SIZE, PlaceholderSource, SMOKE_SAMPLE_CELL,
};
pub use metrics::{FrameHistory, FrameSummary, TARGET_FRAME_NS};
pub use overlay::{OverlayInput, show_overlay};
pub use surface::{SurfaceChange, surface_change};
pub use upload::{
    MAX_CHUNKS_PER_FRAME, MAX_COPIES_PER_FRAME, MAX_PAYLOAD_BYTES_PER_FRAME, UploadBudget,
    UploadPlan, UploadScheduler, bytes_per_chunk,
};
