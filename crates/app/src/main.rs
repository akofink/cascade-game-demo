//! Window loop, wgpu renderer, and native smoke check.
//!
//! GPU objects for the cell texture and pipeline are created once. The swapchain
//! is reconfigured only when the physical size changes to a non-zero value.

use std::sync::Arc;
use std::time::Instant;

use cascade_app::{
    BRUSH_CELLS_PER_FRAME, CHUNK_CELLS, CHUNK_SIZE, Camera, ChunkCoord, DEMO_HEIGHT, DEMO_WIDTH,
    Demo, FrameHistory, MAX_CHUNKS_PER_FRAME, OverlayActions, OverlayInput, PALETTE, PolicyChoice,
    SurfaceChange, UploadBudget, UploadPlan, UploadScheduler, ViewCommand, apply_command,
    bytes_per_chunk, frame_uniform, show_overlay, surface_change, zoom_factor,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const SMOKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(180);
const FRAME_SAMPLES: usize = 240;

struct Gpu {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    configured_size: (u32, u32),
    surface_reconfigures: u32,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    grid_texture: wgpu::Texture,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    egui_inited: bool,
}

#[derive(Default)]
struct PolicySmoke {
    frames: u32,
    first_pending: usize,
    last_pending: usize,
    max_pending: usize,
    max_ready: usize,
    max_upload_backlog: usize,
    max_sim_cpu_ms: f32,
    max_frame_interval_ns: u64,
    p99_frame_interval_ns: u64,
}

struct Smoke {
    started: Instant,
    ready_printed: bool,
    settled_printed: bool,
    presents: u32,
    saw_stale: bool,
    uploaded: u32,
    pan_ok: bool,
    zoom_ok: bool,
    readback_ok: bool,
    readback_note: String,
    resize_targets_seen: u8,
    resize_from: (u32, u32),
    resize_mid: (u32, u32),
    zero_frames: u32,
    zero_start_reconfigs: u32,
    zero_ok: bool,
    hold_frames: u32,
    result_printed: bool,
    bounded: PolicySmoke,
    traditional: PolicySmoke,
    traditional_started: bool,
    bounded_armed: bool,
    traditional_armed: bool,
    failed: Option<String>,
}

struct App {
    gpu: Option<Gpu>,
    camera: Camera,
    demo: Demo,
    deferred_overlay: bool,
    credit_draft: u32,
    uploads: UploadScheduler,
    dirty_chunks: Vec<(u32, u32)>,
    history: FrameHistory,
    last_present: Option<Instant>,
    clock_origin: Instant,
    sim_cpu_ms: f32,
    pre_step_pending: usize,
    pre_step_ready: usize,
    upload_cpu_ms: f32,
    submit_cpu_ms: f32,
    last_plan: UploadPlan,
    destroy_presses: u64,
    destroy_started: bool,
    occluded: bool,
    seeded: bool,
    dragging: bool,
    painting: bool,
    shift_down: bool,
    brush_commands_this_frame: usize,
    last_cursor: Option<(f32, f32)>,
    outdated_handled: bool,
    surface_rebuilds: u32,
    skip_timeout: u32,
    skip_occluded: u32,
    skip_outdated: u32,
    smoke: Option<Smoke>,
    exit_code: i32,
}

fn main() {
    let (smoke, world_size) = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let event_loop = match build_event_loop() {
        Ok(loop_) => loop_,
        Err(error) => {
            eprintln!("event loop: {error}");
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(smoke, world_size);
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("event loop stopped: {error}");
        std::process::exit(1);
    }
    if app.exit_code != 0 {
        std::process::exit(app.exit_code);
    }
}

fn build_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Regular);
    }
    builder.build()
}

fn parse_args() -> Result<(bool, u32), String> {
    let mut smoke = false;
    let mut world_size = DEMO_WIDTH;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--smoke" => smoke = true,
            "--world-size" => {
                let value = args
                    .get(index + 1)
                    .ok_or("missing value for --world-size")?;
                world_size = value.parse().map_err(|_| "world size must be an integer")?;
                if !(32..=cascade_app::MAX_WORLD_AXIS).contains(&world_size)
                    || !world_size.is_multiple_of(32)
                {
                    return Err(format!(
                        "world size must be a multiple of 32 in 32..={}",
                        cascade_app::MAX_WORLD_AXIS
                    ));
                }
                index += 1;
            }
            "--help" | "-h" => {
                println!(
                    "cascade-app [--smoke] [--world-size N]\n\nDefault world: {DEMO_WIDTH}x{DEMO_HEIGHT}; supported sizes are multiples of 32 through {}. Drag to pan. Scroll to zoom. --smoke prepares the mixed fixture, checks a sampled pixel, exercises pan/zoom/resize, and runs paired bounded/traditional overloads before exit.",
                    cascade_app::MAX_WORLD_AXIS
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    Ok((smoke, world_size))
}

impl App {
    fn new(smoke: bool, world_size: u32) -> Self {
        let mut demo = match Demo::new_with_size(world_size, world_size) {
            Ok(demo) => demo,
            Err(error) => {
                eprintln!("simulation: {error}");
                std::process::exit(1);
            }
        };
        if let Err(error) = demo.start_fixture() {
            eprintln!("initial fixture: {error}");
            std::process::exit(1);
        }
        let (width, height) = demo.dimensions();
        let chunks_x = width / CHUNK_SIZE;
        let chunks_y = height / CHUNK_SIZE;
        Self {
            gpu: None,
            camera: Camera::default(),
            demo,
            deferred_overlay: false,
            credit_draft: cascade_app::DEFAULT_CREDITS,
            uploads: UploadScheduler::new(chunks_x, chunks_y),
            dirty_chunks: Vec::with_capacity(MAX_CHUNKS_PER_FRAME),
            history: FrameHistory::default(),
            last_present: None,
            clock_origin: Instant::now(),
            sim_cpu_ms: 0.0,
            pre_step_pending: 0,
            pre_step_ready: 0,
            upload_cpu_ms: 0.0,
            submit_cpu_ms: 0.0,
            last_plan: UploadPlan::default(),
            destroy_presses: 0,
            destroy_started: false,
            occluded: false,
            seeded: false,
            dragging: false,
            painting: false,
            shift_down: false,
            brush_commands_this_frame: 0,
            last_cursor: None,
            outdated_handled: false,
            surface_rebuilds: 0,
            skip_timeout: 0,
            skip_occluded: 0,
            skip_outdated: 0,
            smoke: smoke.then(|| Smoke {
                started: Instant::now(),
                ready_printed: false,
                settled_printed: false,
                presents: 0,
                saw_stale: false,
                uploaded: 0,
                pan_ok: false,
                zoom_ok: false,
                readback_ok: false,
                readback_note: String::new(),
                resize_targets_seen: 0,
                resize_from: (0, 0),
                resize_mid: (0, 0),
                zero_frames: 0,
                zero_start_reconfigs: 0,
                zero_ok: false,
                hold_frames: 0,
                result_printed: false,
                bounded: PolicySmoke::default(),
                traditional: PolicySmoke::default(),
                traditional_started: false,
                bounded_armed: false,
                traditional_armed: false,
                failed: None,
            }),
            exit_code: 0,
        }
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.clock_origin.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn viewport(&self) -> Option<(f32, f32)> {
        self.gpu
            .as_ref()
            .map(|gpu| (gpu.config.width as f32, gpu.config.height as f32))
    }

    fn apply_overlay_actions(&mut self, actions: OverlayActions) {
        let mut restart_disturbance = false;
        if actions.toggle_pause {
            self.demo.toggle_paused();
        }
        if actions.single_step {
            self.demo.single_step();
        }
        if actions.reset
            && let Err(error) = self.demo.reset()
        {
            eprintln!("{error}");
        }
        if let Some(fixture) = actions.fixture {
            self.demo.select_fixture(fixture);
        }
        if actions.cancel_fixture {
            self.demo.cancel_fixture();
        }
        if actions.load_fixture
            && let Err(error) = self.demo.start_fixture()
        {
            eprintln!("{error}");
        }
        if let Some(material) = actions.material {
            self.demo.set_selected_material(material);
        }
        if let Some(credits) = actions.credit_draft {
            self.credit_draft = credits;
        }
        if actions.apply_credits
            && self.demo.set_credits(self.credit_draft)
            && let Err(error) = self.demo.start_fixture()
        {
            eprintln!("restart fixture with adjusted credits: {error}");
        }
        if actions.toggle_policy {
            let previous = self.demo.policy();
            let next = match previous {
                PolicyChoice::Bounded => PolicyChoice::Traditional,
                PolicyChoice::Traditional => PolicyChoice::Bounded,
            };
            self.demo.set_destroy_held(false);
            self.destroy_started = false;
            self.demo.set_policy(next);
            if let Err(error) = self.demo.start_fixture() {
                self.demo.set_policy(previous);
                eprintln!("restart fixture under other policy: {error}");
            } else {
                self.history = FrameHistory::default();
                self.last_present = None;
                self.demo.set_destroy_held(true);
                self.destroy_started = true;
                restart_disturbance = true;
            }
        }
        if let Some(show) = actions.deferred_overlay
            && self.deferred_overlay != show
        {
            self.deferred_overlay = show;
            self.uploads.mark_all();
        }
        if !restart_disturbance {
            self.demo.set_destroy_held(actions.destroy_held);
        }
        if actions.destroy_held && !self.destroy_started {
            let mixed = cascade_sim::fixtures::FixtureId::MixedOverload;
            let already_preparing_mixed =
                self.demo
                    .metrics()
                    .fixture_progress
                    .is_some_and(|progress| {
                        progress
                            .descriptor
                            .is_some_and(|descriptor| descriptor.id == mixed)
                            && !progress.complete
                            && !progress.cancelled
                    });
            if already_preparing_mixed {
                self.destroy_started = true;
            } else {
                self.demo.cancel_fixture();
                self.demo.select_fixture(mixed);
                if self.demo.start_fixture().is_ok() {
                    self.destroy_started = true;
                }
            }
        } else if !actions.destroy_held && !restart_disturbance {
            self.destroy_started = false;
        }
        if actions.destroy_pressed {
            self.destroy_presses = self.destroy_presses.saturating_add(1);
        }
    }

    fn apply_size(&mut self, requested: (u32, u32)) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match surface_change(gpu.configured_size, requested) {
            SurfaceChange::Unchanged | SurfaceChange::SkipEmpty => {}
            SurfaceChange::Reconfigure { width, height } => {
                gpu.config.width = width;
                gpu.config.height = height;
                gpu.surface.configure(&gpu.device, &gpu.config);
                gpu.configured_size = (width, height);
                gpu.surface_reconfigures += 1;
            }
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, message: String) {
        eprintln!("{message}");
        if let Some(smoke) = self.smoke.as_mut() {
            smoke.failed = Some(message);
            print_smoke(
                smoke,
                self.gpu
                    .as_ref()
                    .map(|gpu| gpu.surface_reconfigures)
                    .unwrap_or(0),
            );
        }
        self.exit_code = 1;
        event_loop.exit();
    }

    fn upload_dirty(&mut self) {
        self.brush_commands_this_frame = 0;
        self.uploads.set_clock(self.now_ms());
        if !self.seeded {
            self.uploads.mark_all();
            self.seeded = true;
        }
        self.dirty_chunks.clear();
        self.demo
            .world_mut()
            .drain_dirty_chunks(MAX_CHUNKS_PER_FRAME, &mut self.dirty_chunks);
        for &(x, y) in &self.dirty_chunks {
            let _ = self.uploads.mark_dirty(ChunkCoord { x, y });
        }
        let started = Instant::now();
        let plan = self
            .uploads
            .plan(bytes_per_chunk(1), UploadBudget::default());
        if let Some(gpu) = self.gpu.as_ref() {
            for chunk in &plan.chunks[..plan.count] {
                write_chunk(
                    &gpu.queue,
                    &gpu.grid_texture,
                    self.demo.world(),
                    *chunk,
                    self.deferred_overlay,
                );
            }
        }
        self.upload_cpu_ms = started.elapsed().as_secs_f32() * 1000.0;
        if let Some(smoke) = self.smoke.as_mut() {
            smoke.uploaded += plan.count as u32;
            smoke.saw_stale |= plan.stale;
        }
        self.last_plan = plan;
    }

    fn acquire_frame(&mut self, event_loop: &ActiveEventLoop) -> Option<wgpu::SurfaceTexture> {
        let gpu = self.gpu.as_mut()?;
        if gpu.configured_size.0 == 0 || gpu.configured_size.1 == 0 {
            return None;
        }
        match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => {
                self.outdated_handled = false;
                Some(frame)
            }
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.skip_timeout += 1;
                self.request_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                self.skip_occluded += 1;
                if (self.skip_occluded == 1 || self.skip_occluded.is_multiple_of(30))
                    && let Some(gpu) = self.gpu.as_ref()
                {
                    gpu.window.set_visible(true);
                    gpu.window.focus_window();
                }
                std::thread::sleep(std::time::Duration::from_millis(8));
                self.request_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                if let Some(gpu) = self.gpu.as_mut() {
                    let size = gpu.window.inner_size();
                    if size.width > 0 && size.height > 0 {
                        gpu.config.width = size.width;
                        gpu.config.height = size.height;
                        gpu.surface.configure(&gpu.device, &gpu.config);
                        gpu.configured_size = (size.width, size.height);
                        gpu.surface_reconfigures += 1;
                    }
                }
                self.skip_outdated += 1;
                self.request_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface_rebuilds += 1;
                if self.surface_rebuilds > 3 {
                    self.fail(event_loop, "surface lost repeatedly".to_string());
                    return None;
                }
                let new_surface = self
                    .gpu
                    .as_ref()
                    .and_then(|gpu| gpu.instance.create_surface(gpu.window.clone()).ok());
                if let Some(surface) = new_surface {
                    if let Some(gpu) = self.gpu.as_mut() {
                        gpu.surface = surface;
                        let size = gpu.configured_size;
                        gpu.configured_size = (0, 0);
                        self.apply_size(size);
                    }
                } else {
                    self.fail(event_loop, "recreate surface failed".to_string());
                    return None;
                }
                self.request_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                self.fail(event_loop, "surface validation error".to_string());
                None
            }
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if self.exit_code != 0 {
            return;
        }
        if self
            .smoke
            .as_ref()
            .is_some_and(|smoke| smoke.started.elapsed() > SMOKE_TIMEOUT)
        {
            self.fail(
                event_loop,
                format!(
                    "smoke timed out (timeout={} occluded={} outdated={})",
                    self.skip_timeout, self.skip_occluded, self.skip_outdated
                ),
            );
            return;
        }
        let Some(frame) = self.acquire_frame(event_loop) else {
            return;
        };
        let before_step = self.demo.metrics().slice;
        self.pre_step_pending = before_step.pending_cells;
        self.pre_step_ready = before_step.ready_len;
        let sim_started = Instant::now();
        self.demo.tick();
        self.sim_cpu_ms = sim_started.elapsed().as_secs_f32() * 1000.0;
        self.upload_dirty();
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };

        let world = self.demo.dimensions();
        let uniform = frame_uniform(&self.camera, world);
        let mut intervals = [0_u64; FRAME_SAMPLES];
        let interval_count = self.history.copy_intervals_ns(&mut intervals);
        let current_pending = self.demo.metrics().slice.pending_cells;
        self.history.record_pending(current_pending);
        let mut pending_samples = [0_usize; FRAME_SAMPLES];
        let pending_count = self.history.copy_pending(&mut pending_samples);
        let summary = self.history.summary();
        let overlay = OverlayInput {
            world,
            summary,
            intervals_ns: &intervals[..interval_count],
            pending_samples: &pending_samples[..pending_count],
            sim_cpu_ms: self.sim_cpu_ms,
            upload_cpu_ms: self.upload_cpu_ms,
            submit_cpu_ms: self.submit_cpu_ms,
            backlog: self.last_plan.backlog,
            oldest_dirty_ticks: self.last_plan.oldest_dirty_ticks,
            stale: self.last_plan.stale,
            uploaded_chunks: self.last_plan.count,
            payload_bytes: self.last_plan.payload_bytes,
            surface_reconfigures: gpu.surface_reconfigures,
            destroy_presses: self.destroy_presses,
            sim: self.demo.metrics(),
            material: self.demo.selected_material(),
            credits: self.demo.credits(),
            credit_draft: self.credit_draft,
            deferred_overlay: self.deferred_overlay,
        };

        let egui_ctx = gpu.egui_state.egui_ctx().clone();
        egui_winit::update_viewport_info(
            gpu.egui_state
                .egui_input_mut()
                .viewports
                .entry(egui::ViewportId::ROOT)
                .or_default(),
            &egui_ctx,
            &gpu.window,
            !gpu.egui_inited,
        );
        gpu.egui_inited = true;
        let raw_input = gpu.egui_state.take_egui_input(&gpu.window);
        let mut actions = OverlayActions::default();
        let mut full_output = egui_ctx.run_ui(raw_input, |_| {
            show_overlay(&egui_ctx, &overlay, &mut actions);
        });
        self.apply_overlay_actions(actions);
        let gpu = self.gpu.as_mut().expect("gpu still present");
        gpu.egui_state
            .handle_platform_output(&gpu.window, full_output.platform_output);
        let pixels_per_point = gpu.window.scale_factor() as f32 * egui_ctx.zoom_factor();
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.config.width, gpu.config.height],
            pixels_per_point,
        };
        for (id, deltas) in &full_output.textures_delta.set {
            for delta in deltas {
                gpu.egui_renderer
                    .update_texture(&gpu.device, &gpu.queue, *id, delta);
            }
        }
        gpu.queue.write_buffer(&gpu.uniform_buf, 0, &uniform);

        let submit_started = Instant::now();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let user_cmds = gpu.egui_renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen,
        );
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("grid"),
                color_attachments: &[Some(color_attachment(
                    &view,
                    wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.04,
                        g: 0.045,
                        b: 0.06,
                        a: 1.0,
                    }),
                ))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&gpu.pipeline);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay"),
                color_attachments: &[Some(color_attachment(&view, wgpu::LoadOp::Load))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            gpu.egui_renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen);
        }
        gpu.queue.submit(
            user_cmds
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        self.submit_cpu_ms = submit_started.elapsed().as_secs_f32() * 1000.0;
        gpu.window.pre_present_notify();
        gpu.queue.present(frame);
        for id in &full_output.textures_delta.free {
            gpu.egui_renderer.free_texture(id);
        }
        // egui debug-asserts that a dropped delta was applied. Clearing records that.
        full_output.textures_delta.clear();

        let now = Instant::now();
        if let Some(previous) = self.last_present {
            self.history.record_interval(elapsed_ns(previous, now));
        }
        self.last_present = Some(now);
        self.after_present(event_loop);
        self.request_frame();
    }

    fn after_present(&mut self, event_loop: &ActiveEventLoop) {
        let Some(mut smoke) = self.smoke.take() else {
            return;
        };
        smoke.presents += 1;
        let sim_metrics = self.demo.metrics();
        let is_traditional = sim_metrics.policy == "traditional";
        let run = if is_traditional {
            &mut smoke.traditional
        } else {
            &mut smoke.bounded
        };
        if (1..120).contains(&run.frames) {
            run.last_pending = sim_metrics.slice.pending_cells;
            run.max_pending = run
                .max_pending
                .max(sim_metrics.slice.pending_cells)
                .max(self.pre_step_pending);
            run.max_ready = run
                .max_ready
                .max(sim_metrics.slice.ready_len)
                .max(self.pre_step_ready);
            run.max_upload_backlog = run.max_upload_backlog.max(self.last_plan.backlog);
            run.max_sim_cpu_ms = run.max_sim_cpu_ms.max(self.sim_cpu_ms);
            let summary = self.history.summary();
            run.max_frame_interval_ns = run.max_frame_interval_ns.max(summary.max_ns);
            run.p99_frame_interval_ns = summary.p99_ns;
            run.frames += 1;
            if run.frames == 120 {
                self.demo.set_destroy_held(false);
            }
        }
        let fixture_complete = sim_metrics
            .fixture_progress
            .is_some_and(|progress| progress.complete);
        if fixture_complete && !smoke.bounded_armed {
            smoke.bounded_armed = true;
            smoke.bounded.first_pending = sim_metrics.slice.pending_cells;
            smoke.bounded.last_pending = sim_metrics.slice.pending_cells;
            self.history = FrameHistory::default();
            self.last_present = None;
            self.demo.set_destroy_held(true);
            smoke.bounded.frames = 1;
        }
        if smoke.bounded_armed && smoke.bounded.frames >= 120 && !smoke.traditional_started {
            self.demo.set_destroy_held(false);
            self.demo.set_policy(cascade_app::PolicyChoice::Traditional);
            if let Err(error) = self.demo.start_fixture() {
                smoke.failed = Some(format!("start traditional smoke fixture: {error}"));
            } else {
                smoke.traditional_started = true;
                self.history = FrameHistory::default();
                self.last_present = None;
            }
        }
        if smoke.traditional_started
            && self
                .demo
                .metrics()
                .fixture_progress
                .is_some_and(|progress| progress.complete)
            && !smoke.traditional_armed
        {
            smoke.traditional_armed = true;
            let pending = self.demo.metrics().slice.pending_cells;
            smoke.traditional.first_pending = pending;
            smoke.traditional.last_pending = pending;
            self.history = FrameHistory::default();
            self.last_present = None;
            self.demo.set_destroy_held(true);
            smoke.traditional.frames = 1;
        }
        if smoke.presents >= 1
            && !smoke.readback_ok
            && smoke.readback_note.is_empty()
            && self
                .demo
                .metrics()
                .fixture_progress
                .is_some_and(|progress| progress.complete)
        {
            match self.sample_grid() {
                Ok(note) => {
                    smoke.readback_ok = true;
                    smoke.readback_note = note;
                }
                Err(error) => {
                    smoke.readback_note = error.clone();
                    smoke.failed = Some(error);
                }
            }
            // The smoke-only synchronous GPU readback must not contaminate frame intervals.
            self.last_present = None;
        }
        if !smoke.ready_printed {
            smoke.ready_printed = true;
            println!("SMOKE_READY");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        let configured = self.gpu.as_ref().map(|gpu| gpu.configured_size);
        let reconfigs = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.surface_reconfigures)
            .unwrap_or(0);
        if !smoke.pan_ok {
            let before = self.camera.cells_per_pixel;
            let origin = self.camera.origin_x;
            let viewport = self.viewport().unwrap_or((1.0, 1.0));
            let world = self.demo.dimensions();
            apply_command(
                &mut self.camera,
                ViewCommand::PanPixels {
                    dx: 80.0,
                    dy: -30.0,
                },
                viewport,
                world,
            );
            apply_command(
                &mut self.camera,
                ViewCommand::ZoomAt {
                    cursor_x: viewport.0 * 0.5,
                    cursor_y: viewport.1 * 0.5,
                    factor: zoom_factor(1.0),
                },
                viewport,
                world,
            );
            smoke.pan_ok = (self.camera.origin_x - origin).abs() > f32::EPSILON;
            smoke.zoom_ok = self.camera.cells_per_pixel < before;
            smoke.resize_from = configured.unwrap_or((0, 0));
            if let Some(gpu) = self.gpu.as_ref() {
                let _ = gpu.window.request_inner_size(PhysicalSize::new(960, 640));
            }
        } else if smoke.resize_targets_seen == 0
            && configured.is_some_and(|size| size != smoke.resize_from && size.0 > 0 && size.1 > 0)
        {
            smoke.resize_targets_seen = 1;
            smoke.resize_mid = configured.unwrap_or((0, 0));
            if let Some(gpu) = self.gpu.as_ref() {
                let _ = gpu.window.request_inner_size(PhysicalSize::new(800, 600));
            }
        } else if smoke.resize_targets_seen == 1
            && configured.is_some_and(|size| size != smoke.resize_mid && size.0 > 0 && size.1 > 0)
        {
            smoke.resize_targets_seen = 2;
            smoke.zero_start_reconfigs = reconfigs;
        }
        if smoke.resize_targets_seen == 2 && !smoke.zero_ok {
            self.apply_size((0, 0));
            let after = self
                .gpu
                .as_ref()
                .map(|gpu| gpu.surface_reconfigures)
                .unwrap_or(0);
            smoke.zero_frames += 1;
            if after != reconfigs {
                smoke.failed = Some("zero-size resize reconfigured the surface".to_string());
            }
            if smoke.zero_frames >= 10 && smoke.failed.is_none() {
                smoke.zero_ok = after == smoke.zero_start_reconfigs;
            }
        }
        let ready = smoke.pan_ok
            && smoke.zoom_ok
            && smoke.readback_ok
            && smoke.saw_stale
            && smoke.uploaded > 0
            && smoke.resize_targets_seen == 2
            && smoke.zero_ok
            && smoke.bounded.frames >= 120
            && smoke.traditional.frames >= 120
            && smoke.presents >= 20;
        if ready && !smoke.settled_printed {
            smoke.settled_printed = true;
            println!("SMOKE_SETTLED");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        if smoke.settled_printed {
            smoke.hold_frames += 1;
        }
        let failed = smoke.failed.clone();
        let finish = smoke.settled_printed
            && smoke.hold_frames >= 45
            && failed.is_none()
            && !smoke.result_printed;
        if finish {
            smoke.result_printed = true;
            print_smoke(&smoke, reconfigs);
            println!("SMOKE_RESULT ok");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        self.smoke = Some(smoke);
        if let Some(message) = failed {
            self.fail(event_loop, message);
            return;
        }
        if finish {
            self.exit_code = 0;
            event_loop.exit();
        }
    }

    fn sample_grid(&mut self) -> Result<String, String> {
        let gpu = self.gpu.as_ref().ok_or("gpu missing for sample")?;
        let (width, height) = self.demo.dimensions();
        let sample_cell = (width / 4, height / 4);
        let sample_camera = Camera {
            origin_x: sample_cell.0 as f32,
            origin_y: sample_cell.1 as f32,
            cells_per_pixel: 1.0,
        };
        let uniform = frame_uniform(&sample_camera, self.demo.dimensions());
        gpu.queue.write_buffer(&gpu.uniform_buf, 0, &uniform);
        let format = gpu.config.format;
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("smoke-sample"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let sample_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bytes_per_row = 256_u32;
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("smoke-readback"),
            size: u64::from(bytes_per_row) * 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("smoke-sample"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("smoke-sample"),
                color_attachments: &[Some(color_attachment(
                    &sample_view,
                    wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                ))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&gpu.pipeline);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(8),
                },
            },
            wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| format!("sample poll: {error}"))?;
        receiver
            .recv()
            .map_err(|error| format!("sample map channel: {error}"))?
            .map_err(|error| format!("sample map: {error}"))?;
        let data = slice
            .get_mapped_range()
            .map_err(|error| format!("sample view: {error}"))?;
        let offset = 2 * bytes_per_row as usize + 8;
        let pixel = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        drop(data);
        readback.unmap();
        let expected = stored_pixel(format, PALETTE[3]);
        let close = pixel
            .iter()
            .zip(expected)
            .all(|(got, want)| got.abs_diff(want) <= 1);
        if !close {
            return Err(format!(
                "sample pixel {pixel:?} != sand {expected:?} ({format:?}) at {sample_cell:?}"
            ));
        }
        Ok(format!("sample {pixel:?} {format:?}"))
    }

    fn request_frame(&self) {
        let animate = self.smoke.is_some() || !self.occluded;
        if animate && let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
    }

    fn on_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let consumed = self
            .gpu
            .as_mut()
            .is_some_and(|gpu| gpu.egui_state.on_window_event(&gpu.window, &event).consumed);
        if !consumed {
            self.apply_view_event(&event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.apply_size((size.width, size.height)),
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if !occluded {
                    self.request_frame();
                }
            }
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }

    fn apply_view_event(&mut self, event: &WindowEvent) {
        let Some(viewport) = self.viewport() else {
            return;
        };
        let world = self.demo.dimensions();
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == ElementState::Pressed;
                if *button == MouseButton::Left {
                    self.painting = pressed && self.shift_down;
                    self.dragging = pressed && !self.shift_down;
                } else if *button == MouseButton::Right
                    && pressed
                    && let Some((px, py)) = self.last_cursor
                {
                    let (wx, wy) = self.camera.world_at_pixel(px, py);
                    if wx >= 0.0 && wy >= 0.0 {
                        if self.shift_down {
                            self.demo.detonate_at(wx.floor() as u32, wy.floor() as u32);
                        } else {
                            self.demo.ignite_at(wx.floor() as u32, wy.floor() as u32);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let cursor = (position.x as f32, position.y as f32);
                if self.dragging
                    && let Some(previous) = self.last_cursor
                {
                    apply_command(
                        &mut self.camera,
                        ViewCommand::PanPixels {
                            dx: cursor.0 - previous.0,
                            dy: cursor.1 - previous.1,
                        },
                        viewport,
                        world,
                    );
                }
                if self.painting && self.brush_commands_this_frame < BRUSH_CELLS_PER_FRAME {
                    let (wx, wy) = self.camera.world_at_pixel(cursor.0, cursor.1);
                    if wx >= 0.0 && wy >= 0.0 {
                        let remaining = BRUSH_CELLS_PER_FRAME - self.brush_commands_this_frame;
                        self.brush_commands_this_frame += self.demo.submit_brush_disk(
                            wx.floor() as u32,
                            wy.floor() as u32,
                            2,
                            false,
                            remaining,
                        );
                    }
                }
                self.last_cursor = Some(cursor);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(offset) => offset.y as f32 / 40.0,
                };
                let cursor = self
                    .last_cursor
                    .unwrap_or((viewport.0 * 0.5, viewport.1 * 0.5));
                apply_command(
                    &mut self.camera,
                    ViewCommand::ZoomAt {
                        cursor_x: cursor.0,
                        cursor_y: cursor.1,
                        factor: zoom_factor(lines),
                    },
                    viewport,
                    world,
                );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Shift) {
                    self.shift_down = event.state == ElementState::Pressed;
                    return;
                }
                if event.state != ElementState::Pressed {
                    return;
                }
                match &event.logical_key {
                    Key::Named(NamedKey::Space) => self.demo.toggle_paused(),
                    Key::Character(text) if text == "." => self.demo.single_step(),
                    Key::Character(text) if text.eq_ignore_ascii_case("r") => {
                        if let Err(error) = self.demo.reset() {
                            eprintln!("{error}");
                        }
                    }
                    _ => {}
                }
                let pan = match &event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => Some((-32.0, 0.0)),
                    Key::Named(NamedKey::ArrowRight) => Some((32.0, 0.0)),
                    Key::Named(NamedKey::ArrowUp) => Some((0.0, -32.0)),
                    Key::Named(NamedKey::ArrowDown) => Some((0.0, 32.0)),
                    Key::Character(text) if text.eq_ignore_ascii_case("a") => Some((-32.0, 0.0)),
                    Key::Character(text) if text.eq_ignore_ascii_case("d") => Some((32.0, 0.0)),
                    Key::Character(text) if text.eq_ignore_ascii_case("w") => Some((0.0, -32.0)),
                    Key::Character(text) if text.eq_ignore_ascii_case("s") => Some((0.0, 32.0)),
                    _ => None,
                };
                if let Some((dx, dy)) = pan {
                    apply_command(
                        &mut self.camera,
                        ViewCommand::PanPixels { dx, dy },
                        viewport,
                        world,
                    );
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        match create_gpu(event_loop, self.demo.dimensions()) {
            Ok(gpu) => {
                let viewport = (gpu.config.width as f32, gpu.config.height as f32);
                self.camera.fit(self.demo.dimensions(), viewport);
                gpu.window.request_redraw();
                self.gpu = Some(gpu);
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.on_window_event(event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_frame();
    }
}

fn create_gpu(event_loop: &ActiveEventLoop, world: (u32, u32)) -> Result<Gpu, String> {
    let attributes = Window::default_attributes()
        .with_title("Cascade")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
    let window = Arc::new(
        event_loop
            .create_window(attributes)
            .map_err(|error| format!("create window: {error}"))?,
    );
    let display_handle = event_loop.owned_display_handle();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
        Box::new(display_handle),
    ));
    let surface = instance
        .create_surface(window.clone())
        .map_err(|error| format!("create surface: {error}"))?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .map_err(|error| format!("request adapter: {error}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cascade"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request device: {error}"))?;

    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);
    let capabilities = surface.get_capabilities(&adapter);
    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .ok_or("surface has no default configuration")?;
    config.format = preferred_format(&capabilities.formats).unwrap_or(config.format);
    if capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
    {
        config.present_mode = wgpu::PresentMode::Fifo;
    }
    config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("grid"),
        source: wgpu::ShaderSource::Wgsl(include_str!("grid.wgsl").into()),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("grid"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("grid"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("grid"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let grid_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("materials"),
        size: wgpu::Extent3d {
            width: world.0,
            height: world.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let grid_view = grid_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frame"),
        size: cascade_app::UNIFORM_BYTES as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("grid"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&grid_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: uniform_buf.as_entire_binding(),
            },
        ],
    });

    let egui_ctx = egui::Context::default();
    let max_texture_side =
        usize::try_from(device.limits().max_texture_dimension_2d).unwrap_or(2048);
    let egui_state = egui_winit::State::new(
        egui_ctx,
        egui::ViewportId::ROOT,
        window.as_ref(),
        Some(window.scale_factor() as f32),
        window.theme(),
        Some(max_texture_side),
    );
    let egui_renderer = egui_wgpu::Renderer::new(
        &device,
        config.format,
        egui_wgpu::RendererOptions::default(),
    );
    window.set_visible(true);
    window.focus_window();

    Ok(Gpu {
        window,
        instance,
        surface,
        device,
        queue,
        configured_size: (config.width, config.height),
        surface_reconfigures: 1,
        config,
        pipeline,
        bind_group,
        uniform_buf,
        grid_texture,
        egui_state,
        egui_renderer,
        egui_inited: false,
    })
}

fn write_chunk(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    world: &cascade_sim::World,
    chunk: ChunkCoord,
    deferred_overlay: bool,
) {
    let mut bytes = [0_u8; CHUNK_CELLS];
    let (width, height) = world.dimensions();
    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let cell_x = chunk.x * CHUNK_SIZE + x;
            let cell_y = chunk.y * CHUNK_SIZE + y;
            if cell_x < width && cell_y < height {
                bytes[(y * CHUNK_SIZE + x) as usize] =
                    if deferred_overlay && world.cell_pending(cell_x, cell_y) == Some(true) {
                        6
                    } else {
                        world
                            .cell(cell_x, cell_y)
                            .map(|cell| cell.material as u8)
                            .unwrap_or(0)
                    };
            }
        }
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: chunk.x * CHUNK_SIZE,
                y: chunk.y * CHUNK_SIZE,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(CHUNK_SIZE),
            rows_per_image: Some(CHUNK_SIZE),
        },
        wgpu::Extent3d {
            width: CHUNK_SIZE,
            height: CHUNK_SIZE,
            depth_or_array_layers: 1,
        },
    );
}

fn color_attachment<'a>(
    view: &'a wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
) -> wgpu::RenderPassColorAttachment<'a> {
    wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
        },
    }
}

fn stored_pixel(format: wgpu::TextureFormat, rgba: [u8; 4]) -> [u8; 4] {
    match format {
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
            [rgba[2], rgba[1], rgba[0], rgba[3]]
        }
        _ => rgba,
    }
}

fn preferred_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ]
    .into_iter()
    .find(|format| formats.contains(format))
}

fn elapsed_ns(previous: Instant, now: Instant) -> u64 {
    u64::try_from(now.saturating_duration_since(previous).as_nanos()).unwrap_or(u64::MAX)
}

fn print_smoke(smoke: &Smoke, surface_reconfigures: u32) {
    println!(
        "smoke presents={} pan={} zoom={} readback={} ({}) stale={} uploaded={} resizes_seen={} resize_from={:?} resize_mid={:?} zero_ok={} reconfigures={} bounded_frames={} bounded_pending_first_last={}->{} bounded_max_pending={} bounded_max_ready={} bounded_upload_backlog={} bounded_max_sim_cpu_ms={:.2} bounded_p99_ms={:.2} bounded_max_ms={:.2} traditional_frames={} traditional_pending_first_last={}->{} traditional_max_pending={} traditional_max_ready={} traditional_upload_backlog={} traditional_max_sim_cpu_ms={:.2} traditional_p99_ms={:.2} traditional_max_ms={:.2} hold={}",
        smoke.presents,
        smoke.pan_ok,
        smoke.zoom_ok,
        smoke.readback_ok,
        smoke.readback_note,
        smoke.saw_stale,
        smoke.uploaded,
        smoke.resize_targets_seen,
        smoke.resize_from,
        smoke.resize_mid,
        smoke.zero_ok,
        surface_reconfigures,
        smoke.bounded.frames,
        smoke.bounded.first_pending,
        smoke.bounded.last_pending,
        smoke.bounded.max_pending,
        smoke.bounded.max_ready,
        smoke.bounded.max_upload_backlog,
        smoke.bounded.max_sim_cpu_ms,
        smoke.bounded.p99_frame_interval_ns as f32 / 1_000_000.0,
        smoke.bounded.max_frame_interval_ns as f32 / 1_000_000.0,
        smoke.traditional.frames,
        smoke.traditional.first_pending,
        smoke.traditional.last_pending,
        smoke.traditional.max_pending,
        smoke.traditional.max_ready,
        smoke.traditional.max_upload_backlog,
        smoke.traditional.max_sim_cpu_ms,
        smoke.traditional.p99_frame_interval_ns as f32 / 1_000_000.0,
        smoke.traditional.max_frame_interval_ns as f32 / 1_000_000.0,
        smoke.hold_frames,
    );
}
