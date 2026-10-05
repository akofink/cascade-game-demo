//! Debug overlay. The interval graph is the raw ring, not a smoothed FPS value.

use crate::metrics::{FrameSummary, TARGET_FRAME_NS};

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
}

pub fn show_overlay(ctx: &egui::Context, input: &OverlayInput<'_>, destroy_clicked: &mut bool) {
    egui::Window::new("Cascade")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.label("source: placeholder grid (sim not connected)");
            ui.label(format!("world: {} x {}", input.world.0, input.world.1));
            ui.label("drag to pan, scroll to zoom");
            ui.separator();
            draw_intervals(ui, input.intervals_ns);
            if input.summary.samples == 0 {
                ui.label("frame intervals: waiting");
            } else {
                ui.label(format!(
                    "p50/p95/p99/max ms (last {}): {:.2} / {:.2} / {:.2} / {:.2}",
                    input.summary.samples,
                    ns_to_ms(input.summary.p50_ns),
                    ns_to_ms(input.summary.p95_ns),
                    ns_to_ms(input.summary.p99_ns),
                    ns_to_ms(input.summary.max_ns),
                ));
            }
            ui.label(format!(
                "missed 60 Hz (session, > {:.2} ms): {}",
                ns_to_ms(TARGET_FRAME_NS),
                input.summary.missed_target
            ));
            ui.label(format!(
                "cpu upload {:.3} ms, render submission {:.3} ms",
                input.upload_cpu_ms, input.submit_cpu_ms
            ));
            ui.label(format!(
                "upload backlog: {} chunks, payload last frame {} bytes, uploaded {}",
                input.backlog, input.payload_bytes, input.uploaded_chunks
            ));
            match input.oldest_dirty_ticks {
                Some(age) => ui.label(format!("oldest dirty age: {age} ms")),
                None => ui.label("oldest dirty age: n/a"),
            };
            ui.label(format!(
                "texture stale: {}",
                if input.stale { "yes" } else { "no" }
            ));
            ui.label(format!(
                "surface reconfigures: {}",
                input.surface_reconfigures
            ));
            ui.separator();
            let button = ui.add(egui::Button::new(
                egui::RichText::new("DESTROY PERFORMANCE")
                    .strong()
                    .color(egui::Color32::from_rgb(180, 32, 32)),
            ));
            if button.clicked() {
                *destroy_clicked = true;
            }
            ui.label(format!(
                "stub: not connected (presses: {})",
                input.destroy_presses
            ));
        });
}

fn draw_intervals(ui: &mut egui::Ui, intervals_ns: &[u64]) {
    let (response, painter) = ui.allocate_painter(egui::vec2(280.0, 72.0), egui::Sense::hover());
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
    let y_of = |ms: f32| {
        let t = (ms / peak).clamp(0.0, 1.0);
        rect.bottom() - t * rect.height()
    };
    let target_y = y_of(target_ms);
    painter.hline(
        rect.x_range(),
        target_y,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 64, 64)),
    );
    let n = intervals_ns.len();
    let step = rect.width() / (n.saturating_sub(1).max(1) as f32);
    let color = egui::Color32::from_rgb(120, 200, 160);
    for index in 1..n {
        let x0 = rect.left() + (index - 1) as f32 * step;
        let x1 = rect.left() + index as f32 * step;
        painter.line_segment(
            [
                egui::pos2(x0, y_of(ns_to_ms(intervals_ns[index - 1]))),
                egui::pos2(x1, y_of(ns_to_ms(intervals_ns[index]))),
            ],
            egui::Stroke::new(1.5, color),
        );
    }
}

fn ns_to_ms(ns: u64) -> f32 {
    ns as f32 / 1_000_000.0
}
