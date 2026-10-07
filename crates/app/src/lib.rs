//! Native presentation and bounded-control adapter for Cascade.
//!
//! The app owns a headless `cascade-sim` world and uploads its dirty chunks under a separate cap.

mod camera;
mod demo;
mod feel;
mod focus_bridge;
mod grid;
mod metrics;
mod overlay;
mod surface;
mod upload;

#[cfg(feature = "allocation-diagnostics")]
#[global_allocator]
static DIAGNOSTIC_ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> =
    &stats_alloc::INSTRUMENTED_SYSTEM;

/// Allocation calls observed by the diagnostic global allocator, when enabled.
#[cfg(feature = "allocation-diagnostics")]
pub fn allocation_count() -> Option<usize> {
    Some(stats_alloc::INSTRUMENTED_SYSTEM.stats().allocations)
}

/// Allocation diagnostics are absent from ordinary builds.
#[cfg(not(feature = "allocation-diagnostics"))]
pub fn allocation_count() -> Option<usize> {
    None
}

pub use camera::{Camera, UNIFORM_BYTES, ViewCommand, apply_command, frame_uniform, zoom_factor};
pub use demo::{
    BRUSH_CELLS_PER_FRAME, DEMO_HEIGHT, DEMO_WIDTH, Demo, DemoMetrics, MIN_CREDITS, MaterialChoice,
    PlayerMark, PolicyChoice, default_credits, max_credits,
};
pub use feel::{
    Ack, AckState, ActionKind, Feel, FocusKind, FocusRect, LatencySummary, brush_cells,
    brush_radius_cells, clamped_zoom_lines, presented_byte,
};
pub use focus_bridge::{
    apply_policy as apply_focus_policy, linked as focus_linked, submit_viewport,
};
pub use grid::{
    CHUNK_CELLS, CHUNK_SIZE, ChunkCoord, GridError, MAX_DIRTY_PER_TICK, MAX_WORLD_AXIS,
    MaterialGrid, PALETTE, PLACEHOLDER_SIZE, PlaceholderSource, SMOKE_SAMPLE_CELL,
};
pub use metrics::{FrameHistory, FrameSummary, TARGET_FRAME_NS};
pub use overlay::{OverlayActions, OverlayInput, draw_player_marks, show_overlay};
pub use surface::{SurfaceChange, surface_change};
pub use upload::{
    MAX_CHUNKS_PER_FRAME, MAX_COPIES_PER_FRAME, MAX_PAYLOAD_BYTES_PER_FRAME, UploadBudget,
    UploadPlan, UploadScheduler, bytes_per_chunk,
};
