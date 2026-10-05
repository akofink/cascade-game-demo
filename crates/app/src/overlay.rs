//! Compact player-facing controls. Raw counters stay behind a closed details section.

use cascade_sim::fixtures::FixtureId;

use crate::{
    Ack, AckState, Camera, DemoMetrics, FocusKind, FocusRect, LatencySummary, MaterialChoice,
    PolicyChoice,
    metrics::{FrameSummary, TARGET_FRAME_NS},
};

pub struct OverlayInput<'a> {
    pub viewport: (f32, f32),
    pub world: (u32, u32),
    pub summary: FrameSummary,
    pub intervals_ns: &'a [u64],
    pub pending_samples: &'a [usize],
    pub sim_cpu_ms: f32,
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
    pub credit_draft: u32,
    pub deferred_overlay: bool,
    pub camera_latency: LatencySummary,
    pub paint_latency: LatencySummary,
    pub ignite_latency: LatencySummary,
    pub detonate_latency: LatencySummary,
    pub pending_actions: usize,
    pub focus_linked: bool,
    pub focus_enabled: bool,
    pub story: &'static str,
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
    pub credit_draft: Option<u32>,
    pub apply_credits: bool,
    pub policy: Option<PolicyChoice>,
    pub deferred_overlay: Option<bool>,
    pub destroy_clicked: bool,
    pub destroy_pointer: bool,
}

pub fn show_overlay(ctx: &egui::Context, input: &OverlayInput<'_>, actions: &mut OverlayActions) {
    let max_height = (input.viewport.1 - 24.0).clamp(280.0, 900.0);
    egui::Window::new("Cascade")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .default_width(320.0)
        .max_height(max_height)
        .vscroll(true)
        .show(ctx, |ui| {
            let preparing = input.sim.reset_in_progress
                || input
                    .sim
                    .fixture_progress
                    .is_some_and(|progress| !progress.complete && !progress.cancelled);
            let policy_color = match input.sim.policy {
                "traditional" => egui::Color32::from_rgb(250, 142, 64),
                "bounded focus" => egui::Color32::from_rgb(96, 196, 255),
                _ => egui::Color32::from_rgb(92, 210, 143),
            };
            ui.label(
                egui::RichText::new(input.sim.policy.to_uppercase())
                    .strong()
                    .size(18.0)
                    .color(policy_color),
            );
            ui.label(
                egui::RichText::new(input.story)
                    .size(15.0)
                    .color(egui::Color32::from_rgb(28, 32, 40)),
            );
            let pending_line = if input.pending_actions == 0 {
                "No player action is waiting.".to_string()
            } else {
                format!(
                    "{} player action{} still pending.",
                    input.pending_actions,
                    if input.pending_actions == 1 { "" } else { "s" }
                )
            };
            ui.label(pending_line);
            let focus_line = if !input.focus_linked {
                "Focus scheduling is not linked yet. The cyan box is the viewport the app will submit."
            } else if input.focus_enabled {
                "Focus is on: your neighborhood is serviced while the rest of the world defers."
            } else {
                "Focus is off. This run is bounded FIFO."
            };
            ui.label(focus_line);
            ui.label("Left-drag paints. Right-drag pans. Right-click ignites. Shift-right-click detonates.");
            ui.separator();
            let destroy_label = if input.sim.destroy_held {
                "STOP DESTROY PERFORMANCE"
            } else {
                "DESTROY PERFORMANCE"
            };
            let button = ui.add(
                egui::Button::new(
                    egui::RichText::new(destroy_label)
                        .strong()
                        .size(16.0)
                        .color(egui::Color32::from_rgb(255, 236, 232)),
                )
                .fill(egui::Color32::from_rgb(148, 32, 28))
                .min_size(egui::vec2(ui.available_width(), 36.0)),
            );
            actions.destroy_clicked = button.clicked();
            actions.destroy_pointer = button.is_pointer_button_down_on();
            ui.label(format!(
                "disturbances admitted: {}",
                input.sim.disturbance_emitted
            ));
            ui.separator();
            ui.label(format_latency("camera / UI", input.camera_latency));
            ui.label(format_latency("paint visible", input.paint_latency));
            ui.label(format_latency("ignite visible", input.ignite_latency));
            ui.label(format_latency("detonate visible", input.detonate_latency));
            if input.summary.samples == 0 {
                ui.label("frame intervals: waiting");
            } else {
                ui.label(format!(
                    "frame p50/p95/p99 ms: {:.2} / {:.2} / {:.2}",
                    ns_to_ms(input.summary.p50_ns),
                    ns_to_ms(input.summary.p95_ns),
                    ns_to_ms(input.summary.p99_ns),
                ));
            }
            let graph_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Frame ms");
                    draw_intervals(ui, input.intervals_ns, graph_width);
                });
                ui.vertical(|ui| {
                    ui.label("Pending");
                    draw_pending(ui, input.pending_samples, graph_width);
                });
            });
            ui.label(format!(
                "world {} x {} · pending cells {}",
                input.world.0, input.world.1, input.sim.slice.pending_cells
            ));
            ui.separator();
            ui.label("Same trigger");
            ui.horizontal_wrapped(|ui| {
                for policy in PolicyChoice::ALL {
                    let enabled = !preparing && (policy != PolicyChoice::BoundedFocus || input.focus_linked);
                    let label = if policy == PolicyChoice::BoundedFocus && !input.focus_linked {
                        "Focus (waiting)"
                    } else {
                        policy.short_label()
                    };
                    if ui
                        .add_enabled(enabled, egui::Button::new(label))
                        .clicked()
                    {
                        actions.policy = Some(policy);
                    }
                }
            });
            egui::CollapsingHeader::new("Paint, fixture, details")
                .default_open(false)
                .show(ui, |ui| {
                    details(ui, input, actions, preparing);
                });
        });
}

fn details(
    ui: &mut egui::Ui,
    input: &OverlayInput<'_>,
    actions: &mut OverlayActions,
    preparing: bool,
) {
    ui.horizontal(|ui| {
        if ui
            .button(if input.sim.paused { "Resume" } else { "Pause" })
            .clicked()
        {
            actions.toggle_pause = true;
        }
        if ui.button("Step").clicked() {
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
                    .selectable_label(input.sim.selected_fixture == fixture, fixture_name(fixture))
                    .clicked()
                {
                    actions.fixture = Some(fixture);
                }
            }
        });
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
    let mut credits = input.credit_draft;
    if ui
        .add_enabled(
            !preparing,
            egui::Slider::new(&mut credits, crate::MIN_CREDITS..=crate::max_credits())
                .text("credits"),
        )
        .changed()
    {
        actions.credit_draft = Some(credits);
    }
    if ui
        .add_enabled(
            !preparing && credits != input.credits,
            egui::Button::new("Apply credits and restart"),
        )
        .clicked()
    {
        actions.apply_credits = true;
    }
    let mut show_deferred = input.deferred_overlay;
    if ui
        .checkbox(&mut show_deferred, "Show deferred work")
        .changed()
    {
        actions.deferred_overlay = Some(show_deferred);
    }
    let m = input.sim.slice;
    ui.label(format!(
        "slice {} · credits {}/{}",
        m.slice, m.allowed, m.charged
    ));
    ui.label(format!(
        "quanta eval {} blast {} commands {} recovery {}",
        m.evaluations, m.blasts, m.commands, m.recoveries
    ));
    ui.label(format!(
        "ready {}/{} · oldest pending {} slices",
        m.ready_len, input.sim.ready_capacity, m.oldest_pending_age
    ));
    ui.label(format!(
        "commands rejected/coalesced {}/{}",
        m.rejected_commands, m.coalesced_commands
    ));
    ui.label(format!(
        "CPU sim {:.2} ms · upload {:.2} ms · submit {:.2} ms",
        input.sim_cpu_ms, input.upload_cpu_ms, input.submit_cpu_ms
    ));
    ui.label(format!(
        "upload backlog {} · this frame {} chunks · stale {}",
        input.backlog, input.uploaded_chunks, input.stale
    ));
    ui.label(format!(
        "dirty age {:?} · surface reconfigures {} · presses {}",
        input.oldest_dirty_ticks, input.surface_reconfigures, input.destroy_presses
    ));
    ui.label(format!(
        "resources {} MiB",
        input.sim.resource_bytes / (1024 * 1024)
    ));
    let _ = input.payload_bytes;
}

fn format_latency(label: &str, summary: LatencySummary) -> String {
    if summary.samples == 0 {
        format!("{label}: no samples")
    } else {
        format!(
            "{label} p50/p95 ms (n={}): {:.2} / {:.2}",
            summary.samples,
            ns_to_ms(summary.p50_ns),
            ns_to_ms(summary.p95_ns)
        )
    }
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

fn draw_intervals(ui: &mut egui::Ui, intervals_ns: &[u64], width: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 52.0), egui::Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(16, 18, 24));
    if intervals_ns.is_empty() {
        return;
    }
    let target_ms = ns_to_ms(TARGET_FRAME_NS);
    let mut peak = target_ms;
    let step_n = intervals_ns.len().div_ceil(60).max(1);
    let mut samples = [0.0_f32; 60];
    let mut count = 0;
    for (index, interval) in intervals_ns.iter().enumerate() {
        if index % step_n != 0 && index + 1 != intervals_ns.len() {
            continue;
        }
        if count == samples.len() {
            break;
        }
        let ms = ns_to_ms(*interval);
        samples[count] = ms;
        peak = peak.max(ms);
        count += 1;
    }
    let y_of = |ms: f32| rect.bottom() - (ms / peak).clamp(0.0, 1.0) * rect.height();
    painter.hline(
        rect.x_range(),
        y_of(target_ms),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 64, 64)),
    );
    let step = rect.width() / (count.saturating_sub(1).max(1) as f32);
    for index in 1..count {
        painter.line_segment(
            [
                egui::pos2(
                    rect.left() + (index - 1) as f32 * step,
                    y_of(samples[index - 1]),
                ),
                egui::pos2(rect.left() + index as f32 * step, y_of(samples[index])),
            ],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(120, 200, 160)),
        );
    }
}

fn draw_pending(ui: &mut egui::Ui, pending: &[usize], width: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 52.0), egui::Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(16, 18, 24));
    if pending.is_empty() {
        return;
    }
    let step_n = pending.len().div_ceil(60).max(1);
    let mut samples = [0_usize; 60];
    let mut count = 0;
    for (index, value) in pending.iter().enumerate() {
        if index % step_n != 0 && index + 1 != pending.len() {
            continue;
        }
        if count == samples.len() {
            break;
        }
        samples[count] = *value;
        count += 1;
    }
    let peak = samples[..count].iter().copied().max().unwrap_or(1).max(1) as f32;
    let y_of = |value: usize| rect.bottom() - (value as f32 / peak).clamp(0.0, 1.0) * rect.height();
    let step = rect.width() / (count.saturating_sub(1).max(1) as f32);
    for index in 1..count {
        painter.line_segment(
            [
                egui::pos2(
                    rect.left() + (index - 1) as f32 * step,
                    y_of(samples[index - 1]),
                ),
                egui::pos2(rect.left() + index as f32 * step, y_of(samples[index])),
            ],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(240, 180, 72)),
        );
    }
}

fn ns_to_ms(ns: u64) -> f32 {
    ns as f32 / 1_000_000.0
}

pub fn draw_player_marks(
    ctx: &egui::Context,
    camera: &Camera,
    acks: impl Iterator<Item = Ack>,
    focus: impl Iterator<Item = FocusRect>,
    show_focus: bool,
) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("player-marks"),
    ));
    if show_focus {
        for rect in focus {
            let (x0, y0) = screen_at(camera, rect.min_chunk_x * 32, rect.min_chunk_y * 32);
            let (x1, y1) = screen_at(camera, rect.max_chunk_x * 32, rect.max_chunk_y * 32);
            let color = if rect.kind == FocusKind::Viewport {
                egui::Color32::from_rgba_unmultiplied(96, 196, 255, 180)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 236, 120, 160)
            };
            painter.rect_stroke(
                egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1)),
                0.0,
                egui::Stroke::new(1.5, color),
                egui::StrokeKind::Outside,
            );
        }
    }
    for ack in acks {
        let (x, y) = screen_at(camera, ack.x, ack.y);
        let (color, radius) = match ack.state {
            AckState::Pending => (egui::Color32::from_rgb(96, 220, 255), 11.0),
            AckState::Visible => (egui::Color32::from_rgb(186, 255, 160), 8.0),
            AckState::Rejected | AckState::NoEffect => (egui::Color32::from_rgb(255, 96, 96), 8.0),
        };
        painter.circle_stroke(egui::pos2(x, y), radius, egui::Stroke::new(2.0, color));
    }
}

fn screen_at(camera: &Camera, x: u32, y: u32) -> (f32, f32) {
    (
        (x as f32 + 0.5 - camera.origin_x) / camera.cells_per_pixel,
        (y as f32 + 0.5 - camera.origin_y) / camera.cells_per_pixel,
    )
}
