//! App calls into simulator player focus.
//!
//! The milestone 6 sim API is not on `main` yet. This module is the only place that should
//! gain those calls, so the rest of the app can already record viewport and action regions.

use cascade_sim::World;

use crate::feel::FocusRect;

/// True once `cascade-sim` exports focus enablement and viewport commands.
pub const LINKED: bool = false;

pub fn apply_policy(world: &mut World, focus_enabled: bool) {
    let _ = (world, focus_enabled);
    // When LINKED: `world.set_focus_enabled(focus_enabled)`.
}

pub fn submit_viewport(world: &mut World, rect: FocusRect) {
    let _ = (world, rect);
    // When LINKED: submit `Command::FocusViewport` with a short slice lifetime.
    // Submit on viewport change or before expiry, not every presented frame.
}

pub fn linked() -> bool {
    LINKED
}
