//! Window loop, wgpu renderer, and native smoke check.
//!
//! GPU objects for the cell texture and pipeline are created once. The swapchain
//! is reconfigured only when the physical size changes to a non-zero value.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use image::ImageEncoder;

use cascade_sim::fixtures::FixtureId;

use cascade_app::{
    Ack, AckState, ActionKind, BRUSH_CELLS_PER_FRAME, CHUNK_CELLS, CHUNK_SIZE, Camera, ChunkCoord,
    DEMO_HEIGHT, DEMO_WIDTH, Demo, Feel, FocusKind, FocusRect, FrameHistory, MAX_CHUNKS_PER_FRAME,
    OverlayActions, OverlayInput, PALETTE, PlayerMark, PolicyChoice, SurfaceChange, UploadBudget,
    UploadPlan, UploadScheduler, ViewCommand, apply_command, apply_focus_policy, brush_cells,
    brush_radius_cells, bytes_per_chunk, clamped_zoom_lines, draw_player_marks, focus_linked,
    frame_uniform, presented_byte, show_overlay, submit_viewport, surface_change, zoom_factor,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

mod offscreen;

const SMOKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
const FRAME_SAMPLES: usize = 240;
const SMOKE_POLICY_FRAMES: u32 = 120;
const CAPTURE_INTERVAL_CAPACITY: usize = 16_384;
const MAX_PRIORITY_UPLOADS: usize = 16;
const UPLOAD_BYTES_PER_ROW: usize = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
const STAGING_BYTES_PER_CHUNK: usize =
    (CHUNK_SIZE as usize - 1) * UPLOAD_BYTES_PER_ROW + CHUNK_SIZE as usize;
const STAGING_PADDING_BYTES_PER_CHUNK: usize = STAGING_BYTES_PER_CHUNK - CHUNK_CELLS;
const UPLOAD_STAGING_CAPACITY_BYTES: usize = MAX_CHUNKS_PER_FRAME * STAGING_BYTES_PER_CHUNK;

#[derive(Clone, Copy)]
struct CaptureConfig {
    policy: PolicyChoice,
    fixture: FixtureId,
    seconds: u64,
}

struct CliArgs {
    smoke: bool,
    world_size: u32,
    screenshot_path: Option<PathBuf>,
    capture: Option<CaptureConfig>,
    offscreen_capture: bool,
}

struct ScreenshotReadback {
    buffer: wgpu::Buffer,
    path: PathBuf,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    format: wgpu::TextureFormat,
}

struct Gpu {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    configured_size: (u32, u32),
    initial_size: (u32, u32),
    surface_reconfigures: u32,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    grid_texture: wgpu::Texture,
    upload_buffer: wgpu::Buffer,
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
    max_sim_slice: cascade_sim::SliceMetrics,
    total_evaluations: u64,
    total_blasts: u64,
    total_recoveries: u64,
    total_commands: u64,
    total_selections: u64,
    total_focus_evaluations: u64,
    total_background_evaluations: u64,
    total_focus_blasts: u64,
    total_background_blasts: u64,
    max_upload_cpu_ms: f32,
    max_upload_chunks: usize,
    max_upload_payload_bytes: usize,
    max_upload_row_padding_bytes: usize,
    max_upload_staging_bytes: usize,
    upload_cpu_peak_chunks: usize,
    upload_cpu_peak_payload_bytes: usize,
    upload_cpu_peak_row_padding_bytes: usize,
    upload_cpu_peak_staging_bytes: usize,
    max_submit_cpu_ms: f32,
    slow_frame_count: u32,
    slow_frame_max_ns: u64,
    slow_frame_sim_cpu_ms: f32,
    slow_frame_upload_cpu_ms: f32,
    slow_frame_submit_cpu_ms: f32,
    slow_frame_slice: cascade_sim::SliceMetrics,
    max_frame_interval_ns: u64,
    p99_frame_interval_ns: u64,
    capture_intervals_ns: Vec<u64>,
    capture_interval_drops: u64,
    capture_sim_cpu_ns: Vec<u64>,
    capture_upload_cpu_ns: Vec<u64>,
    capture_submit_cpu_ns: Vec<u64>,
    capture_frame_work_cpu_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_focus_evaluation_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_background_evaluation_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_focus_blast_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_background_blast_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_recovery_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_command_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_selection_ns: Vec<u64>,
    #[cfg(feature = "quantum-diagnostics")]
    capture_scheduler_overhead_ns: Vec<u64>,
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
    focus: PolicySmoke,
    traditional_started: bool,
    focus_started: bool,
    bounded_armed: bool,
    traditional_armed: bool,
    focus_armed: bool,
    failed: Option<String>,
    capture: Option<CaptureConfig>,
    capture_started: Option<Instant>,
}

struct App {
    gpu: Option<Gpu>,
    camera: Camera,
    demo: Demo,
    deferred_overlay: bool,
    credit_draft: u32,
    uploads: UploadScheduler,
    dirty_chunks: Vec<(u32, u32)>,
    upload_bytes: Vec<u8>,
    upload_copy_coords: [(u32, u32); MAX_CHUNKS_PER_FRAME],
    upload_copy_count: usize,
    history: FrameHistory,
    last_present: Option<Instant>,
    clock_origin: Instant,
    sim_cpu_ms: f32,
    pre_step_pending: usize,
    pre_step_ready: usize,
    upload_cpu_ms: f32,
    submit_cpu_ms: f32,
    last_frame_interval_ns: u64,
    last_plan: UploadPlan,
    last_upload_chunks: usize,
    destroy_presses: u64,
    destroy_press_at: Option<Instant>,
    deferred_user_set: bool,
    occluded: bool,
    seeded: bool,
    painting: bool,
    panning: bool,
    pan_origin: Option<(f32, f32)>,
    shift_down: bool,
    key_pan: (bool, bool, bool, bool),
    brush_commands_this_frame: usize,
    last_cursor: Option<(f32, f32)>,
    last_paint_cell: Option<(i32, i32)>,
    scripted_ignite_target: Option<(u32, u32)>,
    feel: Feel,
    last_viewport: Option<FocusRect>,
    viewport_submit_age: u32,
    outdated_handled: bool,
    surface_rebuilds: u32,
    skip_timeout: u32,
    skip_occluded: u32,
    skip_outdated: u32,
    smoke: Option<Smoke>,
    screenshot_path: Option<PathBuf>,
    screenshot_resize_requested: bool,
    screenshot_written: bool,
    exit_code: i32,
}

fn main() {
    let args = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    if args.offscreen_capture {
        let capture = args
            .capture
            .expect("offscreen capture requires a capture policy");
        if let Err(error) = offscreen::run(
            capture.policy,
            capture.fixture,
            capture.seconds,
            args.world_size,
        ) {
            eprintln!("offscreen capture: {error}");
            std::process::exit(1);
        }
        return;
    }
    let event_loop = match build_event_loop() {
        Ok(loop_) => loop_,
        Err(error) => {
            eprintln!("event loop: {error}");
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(
        args.smoke,
        args.world_size,
        args.screenshot_path,
        args.capture,
    );
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

fn parse_args() -> Result<CliArgs, String> {
    let mut smoke = false;
    let mut offscreen_capture = false;
    let mut world_size = DEMO_WIDTH;
    let mut screenshot_path = None;
    let mut capture_policy = None;
    let mut capture_fixture = FixtureId::MixedOverload;
    let mut capture_seconds = 60_u64;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--smoke" => smoke = true,
            "--offscreen-capture" => offscreen_capture = true,
            "--screenshot" => {
                let value = args.get(index + 1).ok_or("missing path for --screenshot")?;
                screenshot_path = Some(PathBuf::from(value));
                index += 1;
            }
            "--capture-policy" => {
                let value = args
                    .get(index + 1)
                    .ok_or("missing value for --capture-policy")?;
                capture_policy =
                    Some(match value.as_str() {
                        "bounded-fifo" => PolicyChoice::BoundedFifo,
                        "bounded-focus" => PolicyChoice::BoundedFocus,
                        "traditional" => PolicyChoice::Traditional,
                        _ => return Err(
                            "capture policy must be bounded-fifo, bounded-focus, or traditional"
                                .into(),
                        ),
                    });
                index += 1;
            }
            "--fixture" => {
                let value = args.get(index + 1).ok_or("missing value for --fixture")?;
                capture_fixture = match value.as_str() {
                    "quiet-world" => FixtureId::QuietWorld,
                    "explosive-lattice" => FixtureId::ExplosiveLattice,
                    "sand-release" => FixtureId::SandRelease,
                    "reservoir-breach" => FixtureId::ReservoirBreach,
                    "burning-forest" => FixtureId::BurningForest,
                    "dirty-world-sweep" => FixtureId::DirtyWorldSweep,
                    "tiny-capacity" => FixtureId::TinyCapacity,
                    "mixed-overload" => FixtureId::MixedOverload,
                    _ => return Err("unknown fixture; use a section-12 fixture name".into()),
                };
                index += 1;
            }
            "--capture-seconds" => {
                let value = args
                    .get(index + 1)
                    .ok_or("missing value for --capture-seconds")?;
                capture_seconds = value
                    .parse()
                    .map_err(|_| "capture seconds must be an integer")?;
                if !(1..=60).contains(&capture_seconds) {
                    return Err("capture seconds must be in 1..=60".into());
                }
                index += 1;
            }
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
                    "cascade-app [--smoke] [--offscreen-capture --capture-policy POLICY --fixture FIXTURE --capture-seconds N] [--world-size N] [--screenshot PATH] [--capture-policy POLICY --fixture FIXTURE --capture-seconds N]\n\nDefault world: {DEMO_WIDTH}x{DEMO_HEIGHT}; supported sizes are multiples of 32 through {}. --smoke runs the native three-policy check. --capture-policy runs one windowed 4096x4096 fixture/policy for the requested wall duration; --fixture defaults to mixed-overload. --offscreen-capture runs the same policy into a fixed 1920x1080 GPU texture without a window or surface. --screenshot writes a PNG from GPU readback after the measured smoke frames.",
                    cascade_app::MAX_WORLD_AXIS
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    if offscreen_capture && capture_policy.is_none() {
        return Err("--offscreen-capture requires --capture-policy".to_string());
    }
    if offscreen_capture && screenshot_path.is_some() {
        return Err("--offscreen-capture does not perform readback or screenshots".to_string());
    }
    if screenshot_path.is_some() && !smoke {
        return Err("--screenshot requires --smoke".to_string());
    }
    if capture_policy.is_some() && screenshot_path.is_some() {
        return Err("--screenshot cannot be combined with --capture-policy".to_string());
    }
    let capture = capture_policy.map(|policy| CaptureConfig {
        policy,
        fixture: capture_fixture,
        seconds: capture_seconds,
    });
    Ok(CliArgs {
        smoke: smoke || capture.is_some(),
        world_size,
        screenshot_path,
        capture,
        offscreen_capture,
    })
}

impl App {
    fn new(
        smoke: bool,
        world_size: u32,
        screenshot_path: Option<PathBuf>,
        capture: Option<CaptureConfig>,
    ) -> Self {
        let mut demo = match Demo::new_with_size(world_size, world_size) {
            Ok(demo) => demo,
            Err(error) => {
                eprintln!("simulation: {error}");
                std::process::exit(1);
            }
        };
        if let Some(capture) = capture {
            demo.set_policy(capture.policy);
            demo.select_fixture(capture.fixture);
        } else if !smoke && focus_linked() {
            demo.set_policy(PolicyChoice::BoundedFocus);
        }
        if let Err(error) = demo.start_fixture() {
            eprintln!("initial fixture: {error}");
            std::process::exit(1);
        }
        let mut smoke_state = smoke.then(|| Smoke {
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
            focus: PolicySmoke::default(),
            traditional_started: false,
            focus_started: false,
            bounded_armed: false,
            traditional_armed: false,
            focus_armed: false,
            failed: None,
            capture,
            capture_started: None,
        });
        if let (Some(config), Some(smoke)) = (capture, smoke_state.as_mut()) {
            let run = match config.policy {
                PolicyChoice::BoundedFifo => &mut smoke.bounded,
                PolicyChoice::BoundedFocus => &mut smoke.focus,
                PolicyChoice::Traditional => &mut smoke.traditional,
            };
            run.capture_intervals_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
            run.capture_sim_cpu_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
            #[cfg(feature = "quantum-diagnostics")]
            {
                run.capture_focus_evaluation_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_background_evaluation_ns =
                    Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_focus_blast_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_background_blast_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_recovery_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_command_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_selection_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
                run.capture_scheduler_overhead_ns = Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY);
            }
        }
        let (width, height) = demo.dimensions();
        let chunks_x = width / CHUNK_SIZE;
        let chunks_y = height / CHUNK_SIZE;
        Self {
            gpu: None,
            camera: Camera::default(),
            demo,
            deferred_overlay: false,
            credit_draft: cascade_app::default_credits(),
            uploads: UploadScheduler::new(chunks_x, chunks_y),
            dirty_chunks: Vec::with_capacity(MAX_CHUNKS_PER_FRAME),
            upload_bytes: vec![0; UPLOAD_STAGING_CAPACITY_BYTES],
            upload_copy_coords: [(0, 0); MAX_CHUNKS_PER_FRAME],
            upload_copy_count: 0,
            history: FrameHistory::default(),
            last_present: None,
            clock_origin: Instant::now(),
            sim_cpu_ms: 0.0,
            pre_step_pending: 0,
            pre_step_ready: 0,
            upload_cpu_ms: 0.0,
            submit_cpu_ms: 0.0,
            last_frame_interval_ns: 0,
            last_plan: UploadPlan::default(),
            last_upload_chunks: 0,
            destroy_presses: 0,
            destroy_press_at: None,
            deferred_user_set: false,
            occluded: false,
            seeded: false,
            painting: false,
            panning: false,
            pan_origin: None,
            shift_down: false,
            key_pan: (false, false, false, false),
            brush_commands_this_frame: 0,
            last_cursor: None,
            last_paint_cell: None,
            scripted_ignite_target: None,
            feel: Feel::new(chunks_x, chunks_y),
            last_viewport: None,
            viewport_submit_age: 0,
            outdated_handled: false,
            surface_rebuilds: 0,
            skip_timeout: 0,
            skip_occluded: 0,
            skip_outdated: 0,
            smoke: smoke_state,
            screenshot_path,
            screenshot_resize_requested: false,
            screenshot_written: false,
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
        if let Some(policy) = actions.policy {
            self.restart_policy(policy);
        }
        if let Some(show) = actions.deferred_overlay
            && self.deferred_overlay != show
        {
            self.deferred_user_set = true;
            self.deferred_overlay = show;
            self.uploads.mark_all();
            self.feel.note_feedback(self.now_ns());
        }
        if actions.destroy_pointer {
            if self.destroy_press_at.is_none() {
                self.destroy_press_at = Some(Instant::now());
            }
            self.demo.set_destroy_held(true);
            self.prefer_deferred_story();
            self.feel.note_feedback(self.now_ns());
        } else {
            self.demo.set_destroy_held(false);
            if actions.destroy_clicked {
                let short = self.destroy_press_at.is_none_or(|started| {
                    started.elapsed() < std::time::Duration::from_millis(280)
                });
                if short {
                    self.demo.toggle_destroy_latch();
                    self.prefer_deferred_story();
                }
                self.destroy_presses = self.destroy_presses.saturating_add(1);
                self.destroy_press_at = None;
                self.feel.note_feedback(self.now_ns());
            }
        }
    }

    fn restart_policy(&mut self, policy: PolicyChoice) {
        if policy == PolicyChoice::BoundedFocus && !focus_linked() {
            return;
        }
        let previous = self.demo.policy();
        self.demo.set_destroy_held(false);
        self.demo.set_policy(policy);
        if let Err(error) = self.demo.start_fixture() {
            self.demo.set_policy(previous);
            apply_focus_policy(self.demo.world_mut(), previous.focus_enabled());
            eprintln!("restart fixture under {}: {error}", policy.name());
            return;
        }
        self.history = FrameHistory::default();
        self.last_present = None;
        if !self.demo.destroy_latched() {
            self.demo.toggle_destroy_latch();
        }
        self.prefer_deferred_story();
    }

    fn prefer_deferred_story(&mut self) {
        if !self.deferred_user_set && !self.deferred_overlay {
            self.deferred_overlay = true;
            self.uploads.mark_all();
        }
    }

    fn now_ns(&self) -> u64 {
        self.clock_origin.elapsed().as_nanos() as u64
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
        let plan = self.uploads.plan(bytes_per_chunk(1), upload_budget());
        self.last_upload_chunks = 0;
        self.upload_copy_count = 0;
        if let Some(gpu) = self.gpu.as_ref() {
            let mut exempt = [(0_u32, 0_u32); 64];
            let exempt_len = self.feel.exempt_cells(&mut exempt);
            let mut uploaded = [(0_u32, 0_u32); MAX_CHUNKS_PER_FRAME];
            let mut uploaded_len = 0;
            for chunk in &plan.chunks[..plan.count] {
                uploaded[uploaded_len] = (chunk.x, chunk.y);
                let start = uploaded_len * STAGING_BYTES_PER_CHUNK;
                write_chunk(
                    self.demo.world(),
                    &mut self.upload_bytes[start..start + STAGING_BYTES_PER_CHUNK],
                    *chunk,
                    self.deferred_overlay,
                    &exempt[..exempt_len],
                );
                uploaded_len += 1;
            }
            let mut targets = [(0_u32, 0_u32); MAX_PRIORITY_UPLOADS];
            let target_len = self.demo.player_targets(&mut targets);
            for &(x, y) in &targets[..target_len.min(MAX_PRIORITY_UPLOADS)] {
                if uploaded_len == MAX_CHUNKS_PER_FRAME {
                    break;
                }
                let chunk = (x / CHUNK_SIZE, y / CHUNK_SIZE);
                if uploaded[..uploaded_len].contains(&chunk) {
                    continue;
                }
                uploaded[uploaded_len] = chunk;
                let start = uploaded_len * STAGING_BYTES_PER_CHUNK;
                write_chunk(
                    self.demo.world(),
                    &mut self.upload_bytes[start..start + STAGING_BYTES_PER_CHUNK],
                    ChunkCoord {
                        x: chunk.0,
                        y: chunk.1,
                    },
                    self.deferred_overlay,
                    &exempt[..exempt_len],
                );
                uploaded_len += 1;
            }
            if uploaded_len > 0 {
                let bytes = uploaded_len * STAGING_BYTES_PER_CHUNK;
                gpu.queue
                    .write_buffer(&gpu.upload_buffer, 0, &self.upload_bytes[..bytes]);
            }
            self.upload_copy_coords[..uploaded_len].copy_from_slice(&uploaded[..uploaded_len]);
            self.upload_copy_count = uploaded_len;
            self.feel.note_uploads(&uploaded[..uploaded_len]);
            self.last_upload_chunks = uploaded_len;
        }
        self.upload_cpu_ms = started.elapsed().as_secs_f32() * 1000.0;
        if let Some(smoke) = self.smoke.as_mut() {
            smoke.uploaded += self.last_upload_chunks as u32;
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
        self.apply_held_keys();
        let before_step = self.demo.metrics().slice;
        self.pre_step_pending = before_step.pending_cells;
        self.pre_step_ready = before_step.ready_len;
        let sim_started = Instant::now();
        self.demo.tick();
        self.sim_cpu_ms = sim_started.elapsed().as_secs_f32() * 1000.0;
        self.upload_dirty();
        self.update_focus_regions();
        let viewport = self.viewport().unwrap_or((1.0, 1.0));
        let story = self.story_line();
        let surface_reconfigures = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.surface_reconfigures)
            .unwrap_or(0);
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
            viewport,
            world,
            summary,
            intervals_ns: &intervals[..interval_count],
            pending_samples: &pending_samples[..pending_count],
            sim_cpu_ms: self.sim_cpu_ms,
            hot_path_allocations: self.demo.metrics().hot_path_allocation_calls,
            upload_cpu_ms: self.upload_cpu_ms,
            submit_cpu_ms: self.submit_cpu_ms,
            backlog: self.last_plan.backlog,
            oldest_dirty_ticks: self.last_plan.oldest_dirty_ticks,
            stale: self.last_plan.stale,
            uploaded_chunks: self.last_plan.count,
            payload_bytes: self.last_plan.payload_bytes,
            surface_reconfigures,
            destroy_presses: self.destroy_presses,
            sim: self.demo.metrics(),
            material: self.demo.selected_material(),
            credits: self.demo.credits(),
            credit_draft: self.credit_draft,
            deferred_overlay: self.deferred_overlay,
            camera_latency: self.feel.camera_summary(),
            paint_latency: self.feel.paint_summary(),
            ignite_latency: self.feel.ignite_summary(),
            detonate_latency: self.feel.detonate_summary(),
            pending_actions: self.feel.pending_count(),
            focus_linked: focus_linked(),
            focus_enabled: self.demo.policy().focus_enabled(),
            offscreen_capture: false,
            story,
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
        let mut mark_acks = [None; 64];
        for (ack_len, ack) in self.feel.acks().enumerate() {
            if ack_len == mark_acks.len() {
                break;
            }
            mark_acks[ack_len] = Some(*ack);
        }
        let mut mark_focus = [None; 8];
        for (focus_len, rect) in self.feel.focus_rects().enumerate() {
            if focus_len == mark_focus.len() {
                break;
            }
            mark_focus[focus_len] = Some(rect);
        }
        let show_focus = self.demo.policy().focus_enabled() || !focus_linked();
        let mut full_output = egui_ctx.run_ui(raw_input, |_| {
            draw_player_marks(
                &egui_ctx,
                &self.camera,
                mark_acks.iter().flatten().copied(),
                mark_focus.iter().copied().flatten(),
                show_focus,
            );
            show_overlay(&egui_ctx, &overlay, &mut actions);
        });
        self.apply_overlay_actions(actions);
        let screenshot_path = if self.screenshot_path.is_some()
            && !self.screenshot_written
            && self
                .smoke
                .as_ref()
                .is_some_and(|smoke| smoke.settled_printed && smoke.hold_frames >= 5)
            && self
                .gpu
                .as_ref()
                .is_some_and(|gpu| gpu.configured_size == gpu.initial_size)
        {
            self.screenshot_path.clone()
        } else {
            None
        };
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
        for (index, &(x, y)) in self.upload_copy_coords[..self.upload_copy_count]
            .iter()
            .enumerate()
        {
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: &gpu.upload_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: (index * STAGING_BYTES_PER_CHUNK) as u64,
                        bytes_per_row: Some(UPLOAD_BYTES_PER_ROW as u32),
                        rows_per_image: Some(CHUNK_SIZE),
                    },
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &gpu.grid_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: x * CHUNK_SIZE,
                        y: y * CHUNK_SIZE,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: CHUNK_SIZE,
                    height: CHUNK_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }
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
        let screenshot = screenshot_path.map(|path| {
            let width = gpu.config.width;
            let height = gpu.config.height;
            let unpadded_bytes_per_row = width * 4;
            let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("smoke-screenshot-readback"),
                size: u64::from(padded_bytes_per_row) * u64::from(height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &frame.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            ScreenshotReadback {
                buffer,
                path,
                width,
                height,
                padded_bytes_per_row,
                format: gpu.config.format,
            }
        });
        gpu.queue.submit(
            user_cmds
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        self.submit_cpu_ms = submit_started.elapsed().as_secs_f32() * 1000.0;
        gpu.window.pre_present_notify();
        gpu.queue.present(frame);
        if let Some(readback) = screenshot {
            match save_screenshot(&gpu.device, readback) {
                Ok(bytes) => {
                    self.screenshot_written = true;
                    if let Some(path) = &self.screenshot_path {
                        println!("SMOKE_SCREENSHOT {} bytes={bytes}", path.display());
                    }
                }
                Err(error) => {
                    self.fail(event_loop, error);
                    return;
                }
            }
        }
        for id in &full_output.textures_delta.free {
            gpu.egui_renderer.free_texture(id);
        }
        // egui debug-asserts that a dropped delta was applied. Clearing records that.
        full_output.textures_delta.clear();

        let now = Instant::now();
        if let Some(previous) = self.last_present {
            let interval = elapsed_ns(previous, now);
            self.last_frame_interval_ns = interval;
            self.history.record_interval(interval);
            if let Some(smoke) = self.smoke.as_mut()
                && let (Some(config), Some(_started)) = (smoke.capture, smoke.capture_started)
            {
                let run = match config.policy {
                    PolicyChoice::BoundedFifo => &mut smoke.bounded,
                    PolicyChoice::BoundedFocus => &mut smoke.focus,
                    PolicyChoice::Traditional => &mut smoke.traditional,
                };
                if run.capture_intervals_ns.len() < CAPTURE_INTERVAL_CAPACITY {
                    run.capture_intervals_ns.push(interval);
                } else {
                    run.capture_interval_drops = run.capture_interval_drops.saturating_add(1);
                }
            }
        }
        self.last_present = Some(now);
        self.after_present(event_loop);
        self.request_frame();
    }

    fn after_present(&mut self, event_loop: &ActiveEventLoop) {
        self.finish_presented_frame();
        let Some(mut smoke) = self.smoke.take() else {
            return;
        };
        smoke.presents += 1;
        let sim_metrics = self.demo.metrics();
        let policy_label = match sim_metrics.policy {
            "traditional" => "traditional",
            "bounded focus" => "bounded-focus",
            _ => "bounded-fifo",
        };
        if let Some(capture) = smoke.capture {
            self.capture_frame(event_loop, &mut smoke, capture, policy_label, sim_metrics);
            self.smoke = Some(smoke);
            return;
        }
        let run = match policy_label {
            "traditional" => &mut smoke.traditional,
            "bounded-focus" => &mut smoke.focus,
            _ => &mut smoke.bounded,
        };
        let mut script_this_frame = false;
        let mut script_step = 0;
        if (1..SMOKE_POLICY_FRAMES).contains(&run.frames) {
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
            script_this_frame = run.frames < SMOKE_POLICY_FRAMES;
            script_step = run.frames;
            if run.frames == SMOKE_POLICY_FRAMES {
                if !has_minimum_action_samples(&self.feel) {
                    smoke.failed = Some(format!(
                        "{policy_label} action stream sample counts below 30 per action type"
                    ));
                }
                print_feel(policy_label, &self.feel);
                let last_window = policy_label == "bounded-focus"
                    || (policy_label == "traditional" && !focus_linked());
                if !last_window {
                    self.feel.clear_latencies();
                }
                self.demo.set_destroy_held(false);
                if last_window && !self.demo.destroy_latched() {
                    self.demo.toggle_destroy_latch();
                }
            }
        }
        if script_this_frame {
            self.scripted_player_action(script_step);
        }
        let fixture_complete = sim_metrics
            .fixture_progress
            .is_some_and(|progress| progress.complete);
        if fixture_complete && !smoke.bounded_armed {
            smoke.bounded_armed = true;
            self.feel.clear_latencies();
            smoke.bounded.first_pending = sim_metrics.slice.pending_cells;
            smoke.bounded.last_pending = sim_metrics.slice.pending_cells;
            self.history = FrameHistory::default();
            self.last_present = None;
            self.demo.set_destroy_held(true);
            smoke.bounded.frames = 1;
        }
        if smoke.bounded_armed
            && smoke.bounded.frames >= SMOKE_POLICY_FRAMES
            && !smoke.traditional_started
        {
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
            self.feel.clear_latencies();
            let pending = self.demo.metrics().slice.pending_cells;
            smoke.traditional.first_pending = pending;
            smoke.traditional.last_pending = pending;
            self.history = FrameHistory::default();
            self.last_present = None;
            self.demo.set_destroy_held(true);
            smoke.traditional.frames = 1;
        }
        if smoke.traditional.frames >= SMOKE_POLICY_FRAMES && focus_linked() && !smoke.focus_started
        {
            self.demo.set_destroy_held(false);
            self.demo.set_policy(PolicyChoice::BoundedFocus);
            if let Err(error) = self.demo.start_fixture() {
                smoke.failed = Some(format!("start focus smoke fixture: {error}"));
            } else {
                smoke.focus_started = true;
                self.history = FrameHistory::default();
                self.last_present = None;
            }
        }
        if smoke.focus_started
            && self
                .demo
                .metrics()
                .fixture_progress
                .is_some_and(|progress| progress.complete)
            && !smoke.focus_armed
        {
            smoke.focus_armed = true;
            self.feel.clear_latencies();
            let pending = self.demo.metrics().slice.pending_cells;
            smoke.focus.first_pending = pending;
            smoke.focus.last_pending = pending;
            self.history = FrameHistory::default();
            self.last_present = None;
            self.demo.set_destroy_held(true);
            smoke.focus.frames = 1;
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
        if smoke.settled_printed
            && self.screenshot_path.is_some()
            && !self.screenshot_resize_requested
        {
            self.screenshot_resize_requested = true;
            if let Some(gpu) = self.gpu.as_ref() {
                let _ = gpu
                    .window
                    .request_inner_size(PhysicalSize::new(gpu.initial_size.0, gpu.initial_size.1));
            }
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
            && smoke.bounded.frames >= SMOKE_POLICY_FRAMES
            && smoke.traditional.frames >= SMOKE_POLICY_FRAMES
            && (!focus_linked() || smoke.focus.frames >= SMOKE_POLICY_FRAMES)
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
            && (self.screenshot_path.is_none() || self.screenshot_written)
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

    fn capture_frame(
        &mut self,
        event_loop: &ActiveEventLoop,
        smoke: &mut Smoke,
        capture: CaptureConfig,
        policy_label: &str,
        sim_metrics: cascade_app::DemoMetrics,
    ) {
        if self.occluded {
            self.fail(
                event_loop,
                "sustained capture aborted: native window is occluded".into(),
            );
            return;
        }
        if smoke.result_printed {
            return;
        }
        if smoke.capture_started.is_none() {
            let fixture_complete = sim_metrics
                .fixture_progress
                .is_some_and(|progress| progress.complete);
            let render_size = self
                .gpu
                .as_ref()
                .map(|gpu| gpu.configured_size)
                .unwrap_or((0, 0));
            if fixture_complete && smoke.presents >= 10 && smoke.uploaded > 0 {
                if render_size != (1920, 1080) {
                    smoke.failed = Some(format!(
                        "sustained capture requires a 1920x1080 surface, got {}x{}",
                        render_size.0, render_size.1
                    ));
                    self.exit_code = 1;
                    event_loop.exit();
                    return;
                }
                let mixed_overload = capture.fixture == FixtureId::MixedOverload;
                self.demo.set_destroy_held(mixed_overload);
                self.feel.clear_latencies();
                self.history = FrameHistory::default();
                self.last_present = None;
                let run = match capture.policy {
                    PolicyChoice::BoundedFifo => &mut smoke.bounded,
                    PolicyChoice::BoundedFocus => &mut smoke.focus,
                    PolicyChoice::Traditional => &mut smoke.traditional,
                };
                run.first_pending = sim_metrics.slice.pending_cells;
                run.last_pending = sim_metrics.slice.pending_cells;
                smoke.capture_started = Some(Instant::now());
                println!(
                    "SMOKE_CAPTURE_STARTED policy={policy_label} fixture={} seconds={} render={}x{} load_gate=external",
                    cascade_sim::fixtures::ScenarioDescriptor::get(capture.fixture).name(),
                    capture.seconds,
                    render_size.0,
                    render_size.1
                );
            }
            return;
        }

        let run = match capture.policy {
            PolicyChoice::BoundedFifo => &mut smoke.bounded,
            PolicyChoice::BoundedFocus => &mut smoke.focus,
            PolicyChoice::Traditional => &mut smoke.traditional,
        };
        run.frames = run.frames.saturating_add(1);
        run.capture_sim_cpu_ns
            .push((self.sim_cpu_ms.max(0.0) * 1_000_000.0) as u64);
        #[cfg(feature = "quantum-diagnostics")]
        {
            run.capture_focus_evaluation_ns
                .push(sim_metrics.slice.focus_evaluation_ns);
            run.capture_background_evaluation_ns
                .push(sim_metrics.slice.background_evaluation_ns);
            run.capture_focus_blast_ns
                .push(sim_metrics.slice.focus_blast_ns);
            run.capture_background_blast_ns
                .push(sim_metrics.slice.background_blast_ns);
            run.capture_recovery_ns.push(sim_metrics.slice.recovery_ns);
            run.capture_command_ns.push(sim_metrics.slice.command_ns);
            run.capture_selection_ns
                .push(sim_metrics.slice.selection_ns);
            run.capture_scheduler_overhead_ns.push(
                ((self.sim_cpu_ms.max(0.0) * 1_000_000.0) as u64).saturating_sub(
                    sim_metrics.slice.evaluation_ns
                        + sim_metrics.slice.blast_ns
                        + sim_metrics.slice.recovery_ns
                        + sim_metrics.slice.command_ns
                        + sim_metrics.slice.selection_ns,
                ),
            );
        }
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
        if self.sim_cpu_ms > run.max_sim_cpu_ms {
            run.max_sim_cpu_ms = self.sim_cpu_ms;
            run.max_sim_slice = sim_metrics.slice;
        }
        run.total_evaluations += sim_metrics.slice.evaluations as u64;
        run.total_blasts += sim_metrics.slice.blasts as u64;
        run.total_recoveries += sim_metrics.slice.recoveries as u64;
        run.total_commands += sim_metrics.slice.commands as u64;
        run.total_selections += sim_metrics.slice.selections as u64;
        run.total_focus_evaluations += sim_metrics.slice.focus_evaluations as u64;
        run.total_background_evaluations += sim_metrics.slice.background_evaluations as u64;
        run.total_focus_blasts += sim_metrics.slice.focus_blasts as u64;
        run.total_background_blasts += sim_metrics.slice.background_blasts as u64;
        run.max_upload_cpu_ms = run.max_upload_cpu_ms.max(self.upload_cpu_ms);
        run.max_upload_chunks = run.max_upload_chunks.max(self.last_upload_chunks);
        let upload_payload_bytes = self.last_upload_chunks * bytes_per_chunk(1);
        run.max_upload_payload_bytes = run.max_upload_payload_bytes.max(upload_payload_bytes);
        let row_padding_bytes = self.last_upload_chunks * STAGING_PADDING_BYTES_PER_CHUNK;
        let staging_bytes = self.last_upload_chunks * STAGING_BYTES_PER_CHUNK;
        run.max_upload_row_padding_bytes = run.max_upload_row_padding_bytes.max(row_padding_bytes);
        run.max_upload_staging_bytes = run.max_upload_staging_bytes.max(staging_bytes);
        if self.upload_cpu_ms >= run.max_upload_cpu_ms {
            run.upload_cpu_peak_chunks = self.last_upload_chunks;
            run.upload_cpu_peak_payload_bytes = upload_payload_bytes;
            run.upload_cpu_peak_row_padding_bytes = row_padding_bytes;
            run.upload_cpu_peak_staging_bytes = staging_bytes;
        }
        run.max_submit_cpu_ms = run.max_submit_cpu_ms.max(self.submit_cpu_ms);
        if self.last_frame_interval_ns > 33_333_333 {
            run.slow_frame_count = run.slow_frame_count.saturating_add(1);
            if self.last_frame_interval_ns > run.slow_frame_max_ns {
                run.slow_frame_max_ns = self.last_frame_interval_ns;
                run.slow_frame_sim_cpu_ms = self.sim_cpu_ms;
                run.slow_frame_upload_cpu_ms = self.upload_cpu_ms;
                run.slow_frame_submit_cpu_ms = self.submit_cpu_ms;
                run.slow_frame_slice = sim_metrics.slice;
            }
        }
        if capture.fixture == FixtureId::MixedOverload {
            self.scripted_player_action(run.frames);
        }
        if smoke.capture_started.is_some_and(|started| {
            started.elapsed() >= std::time::Duration::from_secs(capture.seconds)
        }) {
            if capture.fixture == FixtureId::MixedOverload
                && !has_minimum_action_samples(&self.feel)
            {
                smoke.failed = Some(format!(
                    "{policy_label} action stream sample counts below 30 per action type"
                ));
                self.exit_code = 1;
                event_loop.exit();
                return;
            }
            self.demo.set_destroy_held(false);
            print_feel(policy_label, &self.feel);
            print_capture(policy_label, run);
            println!("SMOKE_RESULT ok");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            smoke.result_printed = true;
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

    fn apply_held_keys(&mut self) {
        let Some(viewport) = self.viewport() else {
            return;
        };
        let (left, right, up, down) = self.key_pan;
        let dx = (f32::from(u8::from(right)) - f32::from(u8::from(left))) * 12.0;
        let dy = (f32::from(u8::from(down)) - f32::from(u8::from(up))) * 12.0;
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.pan_pixels(dx, dy, viewport, self.demo.dimensions());
    }

    fn pan_pixels(&mut self, dx: f32, dy: f32, viewport: (f32, f32), world: (u32, u32)) {
        apply_command(
            &mut self.camera,
            ViewCommand::PanPixels { dx, dy },
            viewport,
            world,
        );
        self.feel.note_feedback(self.now_ns());
    }

    fn story_line(&self) -> &'static str {
        if self.demo.metrics().reset_in_progress
            || self
                .demo
                .metrics()
                .fixture_progress
                .is_some_and(|progress| !progress.complete && !progress.cancelled)
        {
            "Preparing the world. Your clicks are acknowledged and wait."
        } else if self.demo.destroy_active() {
            "Destroy is on. Paint: the ring is yours, gold is the world still catching up."
        } else {
            "Click DESTROY PERFORMANCE, then paint. Your mark stays ahead of the world."
        }
    }

    fn update_focus_regions(&mut self) {
        let Some(viewport) = self.viewport() else {
            return;
        };
        let (width, height) = self.demo.dimensions();
        let chunks_x = width / CHUNK_SIZE;
        let chunks_y = height / CHUNK_SIZE;
        if chunks_x == 0 || chunks_y == 0 {
            return;
        }
        let (x0, y0) = self.camera.world_at_pixel(0.0, 0.0);
        let (x1, y1) = self.camera.world_at_pixel(viewport.0, viewport.1);
        let min_chunk_x = chunk_index(x0, chunks_x);
        let min_chunk_y = chunk_index(y0, chunks_y);
        let max_chunk_x = chunk_index(x1, chunks_x).saturating_add(1).min(chunks_x);
        let max_chunk_y = chunk_index(y1, chunks_y).saturating_add(1).min(chunks_y);
        let mut rects = [FocusRect {
            min_chunk_x,
            min_chunk_y,
            max_chunk_x,
            max_chunk_y,
            kind: FocusKind::Viewport,
        }; 8];
        let mut len = 1;
        for ack in self.feel.acks() {
            if len == rects.len() || ack.state != AckState::Pending {
                continue;
            }
            let cx = ack.x / CHUNK_SIZE;
            let cy = ack.y / CHUNK_SIZE;
            rects[len] = FocusRect {
                min_chunk_x: cx.saturating_sub(1),
                min_chunk_y: cy.saturating_sub(1),
                max_chunk_x: (cx + 2).min(chunks_x),
                max_chunk_y: (cy + 2).min(chunks_y),
                kind: FocusKind::Action,
            };
            len += 1;
        }
        if focus_linked() && self.demo.policy().focus_enabled() {
            let mut sim_rects = [rects[0]; 8];
            let mut sim_len = 0;
            for region in self.demo.world().active_focus_regions() {
                if sim_len == sim_rects.len() {
                    break;
                }
                sim_rects[sim_len] = FocusRect {
                    min_chunk_x: region.min_chunk_x,
                    min_chunk_y: region.min_chunk_y,
                    max_chunk_x: region.max_chunk_x,
                    max_chunk_y: region.max_chunk_y,
                    kind: FocusKind::Viewport,
                };
                sim_len += 1;
            }
            if sim_len == 0 {
                self.feel.set_focus(&rects[..1]);
            } else {
                self.feel.set_focus(&sim_rects[..sim_len]);
            }
            let viewport = rects[0];
            let changed = self.last_viewport != Some(viewport);
            if changed || self.viewport_submit_age >= 4 {
                submit_viewport(self.demo.world_mut(), viewport);
                self.last_viewport = Some(viewport);
                self.viewport_submit_age = 0;
            } else {
                self.viewport_submit_age = self.viewport_submit_age.saturating_add(1);
            }
        } else {
            self.feel
                .set_focus(if focus_linked() { &[] } else { &rects[..len] });
        }
    }

    fn finish_presented_frame(&mut self) {
        let present_ns = self.now_ns();
        self.feel.finish_feedback(present_ns);
        let mut checks = [(0_u32, 0_u32, 0_u8, 0_u8); 64];
        let mut count = 0;
        for ack in self.feel.acks() {
            if count == checks.len() {
                break;
            }
            if let Some(cell) = self.demo.world().cell(ack.x, ack.y) {
                checks[count] = (ack.x, ack.y, cell.material as u8, cell.burning);
                count += 1;
            }
        }
        self.feel.observe_actions(present_ns, |x, y| {
            checks[..count]
                .iter()
                .find(|cell| cell.0 == x && cell.1 == y)
                .map(|cell| (cell.2, cell.3))
        });
    }

    fn note_mark(&mut self, mark: PlayerMark) {
        let state = if mark.admitted {
            AckState::Pending
        } else if mark.kind == ActionKind::Ignite {
            AckState::NoEffect
        } else {
            AckState::Rejected
        };
        self.feel.admit(Ack {
            kind: mark.kind,
            x: mark.x,
            y: mark.y,
            state,
            admitted_ns: self.now_ns(),
            seen_stamp: self.feel.stamp_of(mark.x / CHUNK_SIZE, mark.y / CHUNK_SIZE),
            baseline_material: mark.material,
            baseline_burning: mark.burning,
            paint_material: mark.paint_material,
            visible_frames: 0,
        });
    }

    fn stamp_brush(&mut self, cursor: Option<(f32, f32)>) {
        let Some((px, py)) = cursor.or(self.last_cursor) else {
            return;
        };
        let (wx, wy) = self.camera.world_at_pixel(px, py);
        if wx < 0.0 || wy < 0.0 {
            return;
        }
        let cell = (wx.floor() as i32, wy.floor() as i32);
        let from = self.last_paint_cell.unwrap_or(cell);
        let mut cells = [(0_u32, 0_u32); BRUSH_CELLS_PER_FRAME];
        let remaining = BRUSH_CELLS_PER_FRAME.saturating_sub(self.brush_commands_this_frame);
        let count = brush_cells(
            from,
            cell,
            brush_radius_cells(self.camera.cells_per_pixel),
            remaining,
            &mut cells,
        );
        self.brush_commands_this_frame += count;
        let mut marked = false;
        for &(x, y) in &cells[..count] {
            if let Some(mark) = self.demo.paint_at(x, y)
                && mark.admitted
                && !marked
            {
                self.note_mark(mark);
                marked = true;
            }
        }
        if count > 0
            && !marked
            && let Some(mark) = self
                .demo
                .paint_at(cell.0.max(0) as u32, cell.1.max(0) as u32)
        {
            self.note_mark(mark);
        }
        self.last_paint_cell = Some(cell);
    }

    fn scripted_player_action(&mut self, step: u32) {
        let (width, height) = self.demo.dimensions();
        let action_index = step / 3;
        let mark = match step % 3 {
            0 if self.scripted_ignite_target.is_none() => {
                let x = width / 8 + action_index % 64;
                let y = height / 8 + action_index / 64;
                let mark = self
                    .demo
                    .paint_material_at(x, y, cascade_sim::Material::Wood);
                if mark.is_some_and(|mark| mark.admitted) {
                    self.scripted_ignite_target = Some((x, y));
                }
                mark
            }
            0 => None,
            1 => self.scripted_ignite_target.take().and_then(|(x, y)| {
                let cell = self.demo.world().cell_id(x, y)?;
                let result = self
                    .demo
                    .world_mut()
                    .submit(cascade_sim::Command::Ignite { cell });
                let admitted = matches!(
                    result,
                    cascade_sim::SubmitResult::Accepted | cascade_sim::SubmitResult::Coalesced
                );
                Some(PlayerMark {
                    kind: ActionKind::Ignite,
                    x,
                    y,
                    admitted,
                    rejected: !admitted,
                    material: cascade_sim::Material::Wood as u8,
                    burning: 0,
                    paint_material: cascade_sim::Material::Wood as u8,
                })
            }),
            _ => {
                let x = width / 2 + ((action_index % 32) + 1) * 4;
                let y = height / 2 + ((action_index / 32) + 1) * 4;
                self.demo.detonate_at(x, y)
            }
        };
        if let Some(mark) = mark {
            self.note_mark(mark);
        }
        if let Some(viewport) = self.viewport() {
            self.pan_pixels(1.0, 0.0, viewport, (width, height));
        }
    }

    fn pointer_action(&mut self, detonate: bool) {
        let Some((px, py)) = self.last_cursor else {
            return;
        };
        let (wx, wy) = self.camera.world_at_pixel(px, py);
        if wx < 0.0 || wy < 0.0 {
            return;
        }
        let x = wx.floor() as u32;
        let y = wy.floor() as u32;
        let mark = if detonate {
            self.demo.detonate_at(x, y)
        } else {
            self.demo.ignite_at(x, y)
        };
        if let Some(mark) = mark {
            self.note_mark(mark);
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
                    self.painting = pressed;
                    if pressed {
                        self.stamp_brush(None);
                    } else {
                        self.last_paint_cell = None;
                    }
                } else if *button == MouseButton::Right {
                    if pressed {
                        self.panning = false;
                        self.pan_origin = self.last_cursor;
                    } else {
                        if !self.panning {
                            self.pointer_action(self.shift_down);
                        }
                        self.panning = false;
                        self.pan_origin = None;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let cursor = (position.x as f32, position.y as f32);
                if let Some(origin) = self.pan_origin {
                    let distance = (cursor.0 - origin.0).hypot(cursor.1 - origin.1);
                    if distance > 4.0 {
                        self.panning = true;
                    }
                }
                if self.panning
                    && let Some(previous) = self.last_cursor
                {
                    self.pan_pixels(
                        cursor.0 - previous.0,
                        cursor.1 - previous.1,
                        viewport,
                        world,
                    );
                }
                if self.painting {
                    self.stamp_brush(Some(cursor));
                }
                self.last_cursor = Some(cursor);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(offset) => offset.y as f32 / 80.0,
                };
                if clamped_zoom_lines(lines) != 0.0 {
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
                    self.feel.note_feedback(self.now_ns());
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if event.logical_key == Key::Named(NamedKey::Shift) {
                    self.shift_down = pressed;
                    self.request_frame();
                    return;
                }
                if let Some(index) = pan_key_index(&event.logical_key) {
                    match index {
                        0 => self.key_pan.0 = pressed,
                        1 => self.key_pan.1 = pressed,
                        2 => self.key_pan.2 = pressed,
                        3 => self.key_pan.3 = pressed,
                        _ => {}
                    }
                    self.request_frame();
                    return;
                }
                if let Key::Character(text) = &event.logical_key
                    && text.eq_ignore_ascii_case("f")
                {
                    self.demo.set_destroy_key_held(pressed);
                    if pressed {
                        self.prefer_deferred_story();
                    }
                    self.feel.note_feedback(self.now_ns());
                    self.request_frame();
                    return;
                }
                if !pressed || event.repeat {
                    return;
                }
                match &event.logical_key {
                    Key::Named(NamedKey::Space) => {
                        self.demo.toggle_paused();
                        self.feel.note_feedback(self.now_ns());
                    }
                    Key::Character(text) if text == "." => self.demo.single_step(),
                    Key::Character(text) if text.eq_ignore_ascii_case("r") => {
                        if let Err(error) = self.demo.reset() {
                            eprintln!("{error}");
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::Focused(false) => {
                self.painting = false;
                self.panning = false;
                self.pan_origin = None;
                self.key_pan = (false, false, false, false);
                self.demo.set_destroy_key_held(false);
            }
            _ => {}
        }
        self.request_frame();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        match create_gpu(
            event_loop,
            self.demo.dimensions(),
            self.screenshot_path.is_some(),
            self.smoke
                .as_ref()
                .is_some_and(|smoke| smoke.capture.is_some()),
        ) {
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.occluded && self.smoke.is_none() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
            self.request_frame();
        }
    }
}

fn create_gpu(
    event_loop: &ActiveEventLoop,
    world: (u32, u32),
    capture_surface: bool,
    sustained_capture: bool,
) -> Result<Gpu, String> {
    let logical_size = if sustained_capture {
        winit::dpi::LogicalSize::new(960.0, 540.0)
    } else {
        winit::dpi::LogicalSize::new(1280.0, 720.0)
    };
    let attributes = Window::default_attributes()
        .with_title("Cascade")
        .with_inner_size(logical_size);
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
    // One frame in flight keeps input-to-present on the next refresh instead of queuing another.
    config.desired_maximum_frame_latency = 1;
    if capture_surface
        && (!capabilities.usages.contains(wgpu::TextureUsages::COPY_SRC)
            || !matches!(
                config.format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
            ))
    {
        return Err("surface does not support RGBA8 screenshot readback".to_string());
    }
    config.usage = if capture_surface {
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT
    };
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
    let upload_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunk-upload-staging"),
        size: UPLOAD_STAGING_CAPACITY_BYTES as u64,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
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
        initial_size: (config.width, config.height),
        surface_reconfigures: 1,
        config,
        pipeline,
        bind_group,
        uniform_buf,
        grid_texture,
        upload_buffer,
        egui_state,
        egui_renderer,
        egui_inited: false,
    })
}

fn upload_budget() -> UploadBudget {
    UploadBudget {
        max_chunks: MAX_CHUNKS_PER_FRAME - MAX_PRIORITY_UPLOADS,
        max_copies: MAX_CHUNKS_PER_FRAME - MAX_PRIORITY_UPLOADS,
        ..UploadBudget::default()
    }
}

fn exempt_mask(chunk: ChunkCoord, exempt: &[(u32, u32)]) -> [bool; CHUNK_CELLS] {
    let mut mask = [false; CHUNK_CELLS];
    let chunk_x = chunk.x * CHUNK_SIZE;
    let chunk_y = chunk.y * CHUNK_SIZE;
    for &(cell_x, cell_y) in exempt {
        if cell_x / CHUNK_SIZE == chunk.x && cell_y / CHUNK_SIZE == chunk.y {
            let local_x = cell_x - chunk_x;
            let local_y = cell_y - chunk_y;
            mask[(local_y * CHUNK_SIZE + local_x) as usize] = true;
        }
    }
    mask
}

fn write_chunk(
    world: &cascade_sim::World,
    bytes: &mut [u8],
    chunk: ChunkCoord,
    deferred_overlay: bool,
    exempt: &[(u32, u32)],
) {
    debug_assert!(bytes.len() >= STAGING_BYTES_PER_CHUNK);
    let exempt_mask = exempt_mask(chunk, exempt);
    let (width, height) = world.dimensions();
    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let cell_x = chunk.x * CHUNK_SIZE + x;
            let cell_y = chunk.y * CHUNK_SIZE + y;
            if cell_x < width && cell_y < height {
                let cell = world.cell(cell_x, cell_y);
                let pending = world.cell_pending(cell_x, cell_y) == Some(true);
                let exempt_cell = exempt_mask[(y * CHUNK_SIZE + x) as usize];
                bytes[y as usize * UPLOAD_BYTES_PER_ROW + x as usize] = cell
                    .map(|cell| {
                        presented_byte(
                            cell.material as u8,
                            cell.burning,
                            pending,
                            deferred_overlay,
                            exempt_cell,
                        )
                    })
                    .unwrap_or(0);
            }
        }
    }
}

fn pan_key_index(key: &winit::keyboard::Key) -> Option<usize> {
    match key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => Some(0),
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => Some(1),
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => Some(2),
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => Some(3),
        winit::keyboard::Key::Character(text) if text.eq_ignore_ascii_case("a") => Some(0),
        winit::keyboard::Key::Character(text) if text.eq_ignore_ascii_case("d") => Some(1),
        winit::keyboard::Key::Character(text) if text.eq_ignore_ascii_case("w") => Some(2),
        winit::keyboard::Key::Character(text) if text.eq_ignore_ascii_case("s") => Some(3),
        _ => None,
    }
}

fn chunk_index(world: f32, chunks: u32) -> u32 {
    if world <= 0.0 {
        0
    } else {
        ((world as u32) / CHUNK_SIZE).min(chunks.saturating_sub(1))
    }
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

fn save_screenshot(device: &wgpu::Device, screenshot: ScreenshotReadback) -> Result<u64, String> {
    let slice = screenshot.buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| format!("screenshot poll: {error}"))?;
    receiver
        .recv()
        .map_err(|error| format!("screenshot map channel: {error}"))?
        .map_err(|error| format!("screenshot map: {error}"))?;

    let mapped = slice
        .get_mapped_range()
        .map_err(|error| format!("screenshot mapped view: {error}"))?;
    let row_bytes = screenshot.width as usize * 4;
    let padded_row_bytes = screenshot.padded_bytes_per_row as usize;
    let mut rgba = Vec::with_capacity(row_bytes * screenshot.height as usize);
    for row in mapped
        .chunks_exact(padded_row_bytes)
        .take(screenshot.height as usize)
    {
        for pixel in row[..row_bytes].as_chunks::<4>().0 {
            match screenshot.format {
                wgpu::TextureFormat::Bgra8Unorm => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
                wgpu::TextureFormat::Rgba8Unorm => rgba.extend_from_slice(pixel),
                _ => {
                    return Err(format!(
                        "unsupported screenshot format: {:?}",
                        screenshot.format
                    ));
                }
            }
        }
    }
    drop(mapped);
    screenshot.buffer.unmap();

    if let Some(parent) = screenshot
        .path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create screenshot directory: {error}"))?;
    }
    let file = std::fs::File::create(&screenshot.path)
        .map_err(|error| format!("create screenshot: {error}"))?;
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        file,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive,
    );
    encoder
        .write_image(
            &rgba,
            screenshot.width,
            screenshot.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|error| format!("encode screenshot PNG: {error}"))?;
    std::fs::metadata(&screenshot.path)
        .map(|metadata| metadata.len())
        .map_err(|error| format!("stat screenshot: {error}"))
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

fn has_minimum_action_samples(feel: &Feel) -> bool {
    feel.paint_summary().samples >= 30
        && feel.ignite_summary().samples >= 30
        && feel.detonate_summary().samples >= 30
}

fn print_feel(label: &str, feel: &Feel) {
    let camera = feel.camera_summary();
    let paint = feel.paint_summary();
    let ignite = feel.ignite_summary();
    let detonate = feel.detonate_summary();
    println!(
        "SMOKE_FEEL policy={label} focus={} camera_n={} camera_p50_ms={:.2} camera_p95_ms={:.2} paint_n={} paint_p50_ms={:.2} paint_p95_ms={:.2} ignite_n={} ignite_p50_ms={:.2} ignite_p95_ms={:.2} detonate_n={} detonate_p50_ms={:.2} detonate_p95_ms={:.2}",
        if focus_linked() { "linked" } else { "unlinked" },
        camera.samples,
        camera.p50_ns as f32 / 1_000_000.0,
        camera.p95_ns as f32 / 1_000_000.0,
        paint.samples,
        paint.p50_ns as f32 / 1_000_000.0,
        paint.p95_ns as f32 / 1_000_000.0,
        ignite.samples,
        ignite.p50_ns as f32 / 1_000_000.0,
        ignite.p95_ns as f32 / 1_000_000.0,
        detonate.samples,
        detonate.p50_ns as f32 / 1_000_000.0,
        detonate.p95_ns as f32 / 1_000_000.0,
    );
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

#[cfg(feature = "quantum-diagnostics")]
fn timing_p99_ms(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (99 * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1] as f64 / 1_000_000.0
}

#[cfg(feature = "quantum-diagnostics")]
fn print_quantum_timing(run: &PolicySmoke, label: &str) {
    let estimated_mean_ns = |values: &[u64], count: u64| {
        if count == 0 {
            0.0
        } else {
            values.iter().map(|&value| value as u128).sum::<u128>() as f64 / count as f64
        }
    };
    println!(
        "{label} focus_eval_p99_ms={:.3} background_eval_p99_ms={:.3} focus_blast_p99_ms={:.3} background_blast_p99_ms={:.3} recovery_p99_ms={:.3} command_p99_ms={:.3} selection_p99_ms={:.3} selection_mean_ns_per_probe={:.1} scheduler_overhead_p99_ms={:.3} estimated_mean_ns_per_quantum=focus-eval:{:.1},background-eval:{:.1},focus-blast:{:.1},background-blast:{:.1},recovery:{:.1},command:{:.1} total_focus_eval={} total_background_eval={} total_focus_blast={} total_background_blast={}",
        timing_p99_ms(&run.capture_focus_evaluation_ns),
        timing_p99_ms(&run.capture_background_evaluation_ns),
        timing_p99_ms(&run.capture_focus_blast_ns),
        timing_p99_ms(&run.capture_background_blast_ns),
        timing_p99_ms(&run.capture_recovery_ns),
        timing_p99_ms(&run.capture_command_ns),
        timing_p99_ms(&run.capture_selection_ns),
        estimated_mean_ns(&run.capture_selection_ns, run.total_selections),
        timing_p99_ms(&run.capture_scheduler_overhead_ns),
        estimated_mean_ns(
            &run.capture_focus_evaluation_ns,
            run.total_focus_evaluations,
        ),
        estimated_mean_ns(
            &run.capture_background_evaluation_ns,
            run.total_background_evaluations,
        ),
        estimated_mean_ns(&run.capture_focus_blast_ns, run.total_focus_blasts),
        estimated_mean_ns(
            &run.capture_background_blast_ns,
            run.total_background_blasts,
        ),
        estimated_mean_ns(&run.capture_recovery_ns, run.total_recoveries),
        estimated_mean_ns(&run.capture_command_ns, run.total_commands),
        run.total_focus_evaluations,
        run.total_background_evaluations,
        run.total_focus_blasts,
        run.total_background_blasts,
    );
}

fn print_capture(label: &str, run: &PolicySmoke) {
    let mut intervals = run.capture_intervals_ns.clone();
    intervals.sort_unstable();
    let percentile = |percent: usize| {
        if intervals.is_empty() {
            0
        } else {
            let rank = (percent * intervals.len()).div_ceil(100).max(1);
            intervals[rank - 1]
        }
    };
    let over_33_3_ms = intervals.iter().filter(|&&ns| ns > 33_333_333).count();
    let p99_ms = |samples: &[u64]| {
        if samples.is_empty() {
            0.0
        } else {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (99 * sorted.len()).div_ceil(100).max(1);
            sorted[rank - 1] as f64 / 1_000_000.0
        }
    };
    let sim_p99_ms = p99_ms(&run.capture_sim_cpu_ns);
    println!(
        "SMOKE_CAPTURE policy={label} frames={} p50_ms={:.3} p95_ms={:.3} p99_ms={:.3} max_ms={:.3} over_33_3_ms={} interval_drops={} sim_p99_ms={:.3} sim_samples={} max_sim_cpu_ms={:.3} max_sim_credits={}/{} max_sim_counts=eval:{},blast:{},recovery:{},commands:{},selection:{} totals=eval:{},blast:{},recovery:{},commands:{},selection:{} max_upload_cpu_ms={:.3} max_upload_chunks={} max_upload_payload_bytes={} max_upload_staging_bytes={} max_upload_row_padding_bytes={} upload_cpu_peak_frame=chunks:{},payload_bytes:{},staging_bytes:{},row_padding_bytes:{} max_submit_cpu_ms={:.3} slow_frames={} worst_slow_frame_ms={:.3} slow_frame_cpu_ms=sim:{:.3},upload:{:.3},submit:{:.3} slow_frame_counts=eval:{},blast:{},recovery:{},commands:{},selection:{} max_pending={} max_upload_backlog={}",
        intervals.len(),
        percentile(50) as f64 / 1_000_000.0,
        percentile(95) as f64 / 1_000_000.0,
        percentile(99) as f64 / 1_000_000.0,
        intervals.last().copied().unwrap_or(0) as f64 / 1_000_000.0,
        over_33_3_ms,
        run.capture_interval_drops,
        sim_p99_ms,
        run.capture_sim_cpu_ns.len(),
        run.max_sim_cpu_ms,
        run.max_sim_slice.charged,
        run.max_sim_slice.allowed,
        run.max_sim_slice.evaluations,
        run.max_sim_slice.blasts,
        run.max_sim_slice.recoveries,
        run.max_sim_slice.commands,
        run.max_sim_slice.selections,
        run.total_evaluations,
        run.total_blasts,
        run.total_recoveries,
        run.total_commands,
        run.total_selections,
        run.max_upload_cpu_ms,
        run.max_upload_chunks,
        run.max_upload_payload_bytes,
        run.max_upload_staging_bytes,
        run.max_upload_row_padding_bytes,
        run.upload_cpu_peak_chunks,
        run.upload_cpu_peak_payload_bytes,
        run.upload_cpu_peak_staging_bytes,
        run.upload_cpu_peak_row_padding_bytes,
        run.max_submit_cpu_ms,
        run.slow_frame_count,
        run.slow_frame_max_ns as f64 / 1_000_000.0,
        run.slow_frame_sim_cpu_ms,
        run.slow_frame_upload_cpu_ms,
        run.slow_frame_submit_cpu_ms,
        run.slow_frame_slice.evaluations,
        run.slow_frame_slice.blasts,
        run.slow_frame_slice.recoveries,
        run.slow_frame_slice.commands,
        run.slow_frame_slice.selections,
        run.max_pending,
        run.max_upload_backlog,
    );
    #[cfg(feature = "quantum-diagnostics")]
    print_quantum_timing(run, "SMOKE_QUANTUM_TIMING");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_uploads_share_the_frame_copy_cap() {
        let budget = upload_budget();
        assert_eq!(
            budget.max_chunks,
            MAX_CHUNKS_PER_FRAME - MAX_PRIORITY_UPLOADS
        );
        assert_eq!(
            budget.max_copies,
            MAX_CHUNKS_PER_FRAME - MAX_PRIORITY_UPLOADS
        );
        assert!(budget.max_copies + MAX_PRIORITY_UPLOADS <= MAX_CHUNKS_PER_FRAME);
        assert_eq!(
            UPLOAD_BYTES_PER_ROW,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize
        );
        assert_eq!(
            STAGING_BYTES_PER_CHUNK,
            (CHUNK_SIZE as usize - 1) * 256 + CHUNK_SIZE as usize
        );
        assert_eq!(
            STAGING_PADDING_BYTES_PER_CHUNK,
            STAGING_BYTES_PER_CHUNK - CHUNK_CELLS
        );
        assert_eq!(
            STAGING_BYTES_PER_CHUNK % wgpu::COPY_BUFFER_ALIGNMENT as usize,
            0
        );
        assert_eq!(
            UPLOAD_STAGING_CAPACITY_BYTES,
            MAX_CHUNKS_PER_FRAME * STAGING_BYTES_PER_CHUNK
        );
    }

    #[test]
    fn exempt_mask_is_chunk_local_and_indexed_by_cell() {
        let mask = exempt_mask(ChunkCoord { x: 0, y: 0 }, &[(31, 31), (32, 0), (0, 1)]);
        assert!(mask[31 * CHUNK_SIZE as usize + 31]);
        assert!(mask[CHUNK_SIZE as usize]);
        assert!(!mask[0]);
        assert_eq!(mask.iter().filter(|&&marked| marked).count(), 2);
    }
}
