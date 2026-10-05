//! App calls into simulator player focus.

use cascade_sim::{Command, World};

use crate::feel::FocusRect;

/// The milestone 6 simulator API is present: focus enablement, viewport commands, and region reads.
pub const LINKED: bool = true;

pub fn apply_policy(world: &mut World, focus_enabled: bool) {
    world.set_focus_enabled(focus_enabled);
}

/// Submit the camera viewport as a short-lived focus command. Callers should not do this every frame.
pub fn submit_viewport(world: &mut World, rect: FocusRect) {
    if rect.min_chunk_x >= rect.max_chunk_x || rect.min_chunk_y >= rect.max_chunk_y {
        return;
    }
    let _ = world.submit(Command::FocusViewport {
        min_chunk_x: rect.min_chunk_x,
        min_chunk_y: rect.min_chunk_y,
        max_chunk_x: rect.max_chunk_x,
        max_chunk_y: rect.max_chunk_y,
        lifetime_slices: 8,
    });
}

pub fn linked() -> bool {
    LINKED
}
