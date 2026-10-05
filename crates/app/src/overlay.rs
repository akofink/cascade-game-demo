//! Bounded debug controls and instrumentation overlay.

use cascade_sim::fixtures::FixtureId;

use crate::{
    DemoMetrics, MaterialChoice,
    metrics::{FrameSummary, TARGET_FRAME_NS},
};

pub struct OverlayInput<'a> {
    pub world: (u32, u32),
    pub summary: FrameSummary,
    pub intervals_ns: &'a [u64],
    pub upload_cpu_ms: f32,
    pub submit_cpu_ms: f32,
    pub backlog: usize,
    pub oldest_dirty_ticks: Option<u64>,
    pub stale: bool,
    pub uploaded_chunks: usize,
    pub payload_bytes: usize,
    pub surface_reconfigures: u32,
    pub destroy_presses: u64,
    pub sim: DemoMetrics,
    pub material: MaterialChoice,
    pub credits: u32,
    pub deferred_overlay: bool,
}

#[derive(Default)]
pub struct OverlayActions {
    pub toggle_pause: bool,
    pub single_step: bool,
    pub reset: bool,
    pub load_fixture: bool,
    pub cancel_fixture: bool,
    pub fixture: Option<FixtureId>,
    pub material: Option<MaterialChoice>,
    pub credits: Option<u32>,
    pub deferred_overlay: Option<bool>,
    pub destroy_pressed: bool,
    pub destroy_held: bool,
}

pub fn show_overlay(ctx: &egui::Context, input: &OverlayInput<'_>, actions: &mut OverlayActions) {
    egui::Window::new("Cascade")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .default_width(360.0)
        .show(ctx, |ui| {
            let preparing = input.sim.reset_in_progress
                || input
                    .sim
                    .fixture_progress
                    .is_some_and(|progress| !progress.complete && !progress.cancelled);
            ui.label("Bounded-work cellular simulation");
            ui.label(format!("world: {} x {}", input.world.0, input.world.1));
            ui.label("drag to pan; scroll to zoom; right-click ignite; shift-right-click detonate");
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .button(if input.sim.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    actions.toggle_pause = true;
                }
                if ui.button("Single step").clicked() {
                    actions.single_step = true;
                }
                if ui.button("Reset").clicked() {
                    actions.reset = true;
                }
            });
            egui::ComboBox::from_label("Material")
                .selected_text(input.material.name())
                .show_ui(ui, |ui| {
                    for material in MaterialChoice::ALL {
                        if ui
                            .selectable_label(input.material == material, material.name())
                            .clicked()
                        {
                            actions.material = Some(material);
                        }
                    }
                });
            egui::ComboBox::from_label("Fixture")
                .selected_text(fixture_name(input.sim.selected_fixture))
                .show_ui(ui, |ui| {
                    for fixture in fixtures() {
                        if ui
                            .selectable_label(
                                input.sim.selected_fixture == fixture,
                                fixture_name(fixture),
                            )
                            .clicked()
                        {
                            actions.fixture = Some(fixture);
                        }
                    }
                });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!preparing, egui::Button::new("Prepare selected fixture"))
                    .clicked()
                {
                    actions.load_fixture = true;
                }
                if input
                    .sim
                    .fixture_progress
                    .is_some_and(|progress| !progress.complete && !progress.cancelled)
                    && ui.button("Cancel preparation").clicked()
                {
                    actions.cancel_fixture = true;
                }
            });
            ui.label(format!("scheduler policy: {}", input.sim.policy));
            let mut credits = input.credits;
            if ui
                .add(
                    egui::Slider::new(&mut credits, crate::MIN_CREDITS..=crate::MAX_CREDITS)
                        .text("credits / slice"),
                )
                .changed()
            {
                actions.credits = Some(credits);
            }
            let mut show_deferred = input.deferred_overlay;
            if ui
                .checkbox(&mut show_deferred, "Show deferred work")
                .changed()
            {
                actions.deferred_overlay = Some(show_deferred);
            }
            ui.separator();
            draw_intervals(ui, input.intervals_ns);
            if input.summary.samples == 0 {
                ui.label("frame intervals: waiting");
            } else {
                ui.label(format!(
                    "frame p50/p95/p99/max ms (last {}): {:.2} / {:.2} / {:.2} / {:.2}",
                    input.summary.samples,
                    ns_to_ms(input.summary.p50_ns),
                    ns_to_ms(input.summary.p95_ns),
                    ns_to_ms(input.summary.p99_ns),
                    ns_to_ms(input.summary.max_ns),
                ));
            }
            ui.label(format!(
                "missed 60 Hz frames: {}",
                input.summary.missed_target
            ));
            ui.label(format!(
                "upload {:.3} ms, render submission {:.3} ms",
                input.upload_cpu_ms, input.submit_cpu_ms
            ));
            ui.label(format!(
                "upload backlog {} chunks, {} bytes uploaded, {} this frame",
                input.backlog, input.payload_bytes, input.uploaded_chunks
            ));
            ui.label(format!(
                "texture stale: {}; dirty age: {:?}",
                input.stale, input.oldest_dirty_ticks
            ));
            ui.label(format!(
                "surface reconfigures: {}",
                input.surface_reconfigures
            ));
            let m = input.sim.slice;
            ui.separator();
            ui.label(format!(
                "policy: {:?}; slice {}; credits allowed/used: {}/{}",
                input.sim.policy, m.slice, m.allowed, m.charged
            ));
            ui.label(format!(
                "quanta: eval {} blast {} commands {} recovery {}",
                m.evaluations, m.blasts, m.commands, m.recoveries
            ));
            ui.label(format!(
                "ready ring: {}; pending cells: {}; oldest pending age: {} slices",
                m.ready_len, m.pending_cells, m.oldest_pending_age
            ));
            ui.label(format!(
                "commands rejected/coalesced: {}/{}",
                m.rejected_commands, m.coalesced_commands
            ));
            ui.label(format!(
                "fixture: {}; prep: {:?}",
                fixture_name(input.sim.selected_fixture),
                input
                    .sim
                    .fixture_progress
                    .map(|p| (p.prepared_cells, p.total_cells, p.complete))
            ));
            ui.label(format!(
                "resources: {} MiB",
                input.sim.resource_bytes / (1024 * 1024)
            ));
            let button = ui.add(egui::Button::new(
                egui::RichText::new(if input.sim.destroy_held {
                    "RELEASE DESTROY PERFORMANCE"
                } else {
                    "DESTROY PERFORMANCE (hold)"
                })
                .strong()
                .color(egui::Color32::from_rgb(180, 32, 32)),
            ));
            actions.destroy_pressed = button.clicked();
            actions.destroy_held = button.is_pointer_button_down_on();
            ui.label(format!(
                "disturbance descriptors admitted: {}; presses: {}",
                input.sim.disturbance_emitted, input.destroy_presses
            ));
        });
}

fn fixtures() -> [FixtureId; 8] {
    [
        FixtureId::QuietWorld,
        FixtureId::ExplosiveLattice,
        FixtureId::SandRelease,
        FixtureId::ReservoirBreach,
        FixtureId::BurningForest,
        FixtureId::DirtyWorldSweep,
        FixtureId::TinyCapacity,
        FixtureId::MixedOverload,
    ]
}

fn fixture_name(id: FixtureId) -> &'static str {
    ScenarioDescriptorName::name(id)
}

struct ScenarioDescriptorName;
impl ScenarioDescriptorName {
    fn name(id: FixtureId) -> &'static str {
        match id {
            FixtureId::QuietWorld => "quiet-world",
            FixtureId::ExplosiveLattice => "explosive-lattice",
            FixtureId::SandRelease => "sand-release",
            FixtureId::ReservoirBreach => "reservoir-breach",
            FixtureId::BurningForest => "burning-forest",
            FixtureId::DirtyWorldSweep => "dirty-world-sweep",
            FixtureId::TinyCapacity => "tiny-capacity",
            FixtureId::MixedOverload => "mixed-overload",
        }
    }
}

fn draw_intervals(ui: &mut egui::Ui, intervals_ns: &[u64]) {
    let (response, painter) = ui.allocate_painter(egui::vec2(320.0, 72.0), egui::Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(16, 18, 24));
    if intervals_ns.is_empty() {
        return;
    }
    let target_ms = ns_to_ms(TARGET_FRAME_NS);
    let mut peak = target_ms;
    for interval in intervals_ns {
        peak = peak.max(ns_to_ms(*interval));
    }
    let y_of = |ms: f32| rect.bottom() - (ms / peak).clamp(0.0, 1.0) * rect.height();
    let target_y = y_of(target_ms);
    painter.hline(
        rect.x_range(),
        target_y,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 64, 64)),
    );
    let n = intervals_ns.len();
    let step = rect.width() / (n.saturating_sub(1).max(1) as f32);
    for index in 1..n {
        painter.line_segment(
            [
                egui::pos2(
                    rect.left() + (index - 1) as f32 * step,
                    y_of(ns_to_ms(intervals_ns[index - 1])),
                ),
                egui::pos2(
                    rect.left() + index as f32 * step,
                    y_of(ns_to_ms(intervals_ns[index])),
                ),
            ],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(120, 200, 160)),
        );
    }
}

fn ns_to_ms(ns: u64) -> f32 {
    ns as f32 / 1_000_000.0
}
