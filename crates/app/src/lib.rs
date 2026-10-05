//! Native presentation and bounded-control adapter for Cascade.
//!
//! The app owns a headless `cascade-sim` world and uploads its dirty chunks under a separate cap.

mod camera;
mod demo;
mod grid;
mod metrics;
mod overlay;
mod surface;
mod upload;

pub use camera::{Camera, UNIFORM_BYTES, ViewCommand, apply_command, frame_uniform, zoom_factor};
pub use demo::{
    BRUSH_CELLS_PER_FRAME, DEFAULT_CREDITS, DEMO_HEIGHT, DEMO_WIDTH, Demo, DemoMetrics,
    MAX_CREDITS, MIN_CREDITS, MaterialChoice,
};
pub use grid::{
    CHUNK_CELLS, CHUNK_SIZE, ChunkCoord, GridError, MAX_DIRTY_PER_TICK, MaterialGrid, PALETTE,
    PLACEHOLDER_SIZE, PlaceholderSource, SMOKE_SAMPLE_CELL,
};
pub use metrics::{FrameHistory, FrameSummary, TARGET_FRAME_NS};
pub use overlay::{OverlayActions, OverlayInput, show_overlay};
pub use surface::{SurfaceChange, surface_change};
pub use upload::{
    MAX_CHUNKS_PER_FRAME, MAX_COPIES_PER_FRAME, MAX_PAYLOAD_BYTES_PER_FRAME, UploadBudget,
    UploadPlan, UploadScheduler, bytes_per_chunk,
};
