//! Windowless native capture into a fixed-size GPU texture.
//!
//! The loop intentionally does not acquire or read back the render target. Its clock measures
//! simulation, uploads, encoding/submission, and fixed-cadence waiting, not display presentation.

use super::*;
use std::time::Duration;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FRAME_PERIOD: Duration = Duration::from_nanos(16_666_667);

struct OffscreenGpu {
    _instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    grid_texture: wgpu::Texture,
    target: wgpu::Texture,
    egui: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
}

pub(super) fn run(policy: PolicyChoice, seconds: u64, world_size: u32) -> Result<(), String> {
    println!(
        "OFFSCREEN_CAPTURE_START policy={} seconds={seconds} target={WIDTH}x{HEIGHT} cadence_hz=60 presentation=none",
        policy.name()
    );
    let mut demo = Demo::new_with_size(world_size, world_size)
        .map_err(|error| format!("simulation: {error}"))?;
    demo.set_policy(policy);
    demo.start_fixture()
        .map_err(|error| format!("initial fixture: {error}"))?;

    let mut gpu = create_gpu(demo.dimensions())?;
    let mut camera = Camera::default();
    camera.fit(demo.dimensions(), (WIDTH as f32, HEIGHT as f32));
    let (chunks_x, chunks_y) = (world_size / CHUNK_SIZE, world_size / CHUNK_SIZE);
    let mut uploads = UploadScheduler::new(chunks_x, chunks_y);
    let mut seeded = false;
    let mut dirty_chunks = Vec::with_capacity(MAX_CHUNKS_PER_FRAME);
    let mut feel = Feel::new(chunks_x, chunks_y);
    let mut history = FrameHistory::default();
    let mut deferred_overlay = true;
    let mut last_tick: Option<Instant> = None;
    let mut next_frame = Instant::now();
    let mut scripted_ignite_target = None;
    let mut last_viewport = None;
    let mut viewport_submit_age = 0_u32;
    let mut capture_started = None;
    let mut run = PolicySmoke {
        capture_intervals_ns: Vec::with_capacity(CAPTURE_INTERVAL_CAPACITY),
        ..PolicySmoke::default()
    };
    let mut frame_number = 0_u32;
    let clock_origin = Instant::now();
    let mut last_submit_cpu_ms = 0.0_f32;
    let deadline = Duration::from_secs(seconds);

    loop {
        let now = Instant::now();
        if now < next_frame {
            std::thread::sleep(next_frame - now);
        }
        let frame_at = Instant::now();
        if let Some(previous) = last_tick {
            let interval = elapsed_ns(previous, frame_at);
            if capture_started.is_some() {
                if run.capture_intervals_ns.len() < CAPTURE_INTERVAL_CAPACITY {
                    run.capture_intervals_ns.push(interval);
                } else {
                    run.capture_interval_drops = run.capture_interval_drops.saturating_add(1);
                }
            }
            history.record_interval(interval);
        }
        last_tick = Some(frame_at);
        frame_number = frame_number.saturating_add(1);

        let before = demo.metrics().slice;
        let sim_started = Instant::now();
        demo.tick();
        let sim_cpu_ms = sim_started.elapsed().as_secs_f32() * 1000.0;

        uploads.set_clock(clock_origin.elapsed().as_millis() as u64);
        if !seeded {
            uploads.mark_all();
            seeded = true;
        }
        dirty_chunks.clear();
        demo.world_mut()
            .drain_dirty_chunks(MAX_CHUNKS_PER_FRAME, &mut dirty_chunks);
        for &(x, y) in &dirty_chunks {
            let _ = uploads.mark_dirty(ChunkCoord { x, y });
        }
        let upload_started = Instant::now();
        let last_plan = uploads.plan(bytes_per_chunk(1), UploadBudget::default());
        let mut exempt = [(0_u32, 0_u32); 64];
        let exempt_len = feel.exempt_cells(&mut exempt);
        for chunk in &last_plan.chunks[..last_plan.count] {
            super::write_chunk(
                &gpu.queue,
                &gpu.grid_texture,
                demo.world(),
                *chunk,
                deferred_overlay,
                &exempt[..exempt_len],
            );
        }
        let mut uploaded_coords = [(0_u32, 0_u32); MAX_CHUNKS_PER_FRAME];
        for (index, chunk) in last_plan.chunks[..last_plan.count].iter().enumerate() {
            uploaded_coords[index] = (chunk.x, chunk.y);
        }
        let mut uploaded_count = last_plan.count;
        let mut targets = [(0_u32, 0_u32); 16];
        let target_len = demo.player_targets(&mut targets);
        for &(x, y) in &targets[..target_len] {
            let coord = (x / CHUNK_SIZE, y / CHUNK_SIZE);
            if last_plan.chunks[..last_plan.count]
                .iter()
                .any(|chunk| (chunk.x, chunk.y) == coord)
            {
                continue;
            }
            super::write_chunk(
                &gpu.queue,
                &gpu.grid_texture,
                demo.world(),
                ChunkCoord {
                    x: coord.0,
                    y: coord.1,
                },
                deferred_overlay,
                &exempt[..exempt_len],
            );
            if uploaded_count < uploaded_coords.len() {
                uploaded_coords[uploaded_count] = coord;
                uploaded_count += 1;
            }
        }
        feel.note_uploads(&uploaded_coords[..uploaded_count]);
        let upload_cpu_ms = upload_started.elapsed().as_secs_f32() * 1000.0;

        update_focus_regions(
            &mut demo,
            &mut feel,
            &camera,
            &mut last_viewport,
            &mut viewport_submit_age,
        );

        let sim_metrics = demo.metrics();
        let viewport = (WIDTH as f32, HEIGHT as f32);
        let uniform = frame_uniform(&camera, demo.dimensions());
        gpu.queue.write_buffer(&gpu.uniform_buf, 0, &uniform);
        history.record_pending(sim_metrics.slice.pending_cells);
        let mut intervals = [0_u64; FRAME_SAMPLES];
        let interval_count = history.copy_intervals_ns(&mut intervals);
        let mut pending = [0_usize; FRAME_SAMPLES];
        let pending_count = history.copy_pending(&mut pending);
        let overlay = OverlayInput {
            viewport,
            world: demo.dimensions(),
            summary: history.summary(),
            intervals_ns: &intervals[..interval_count],
            pending_samples: &pending[..pending_count],
            sim_cpu_ms,
            upload_cpu_ms,
            submit_cpu_ms: last_submit_cpu_ms,
            backlog: last_plan.backlog,
            oldest_dirty_ticks: last_plan.oldest_dirty_ticks,
            stale: last_plan.stale,
            uploaded_chunks: last_plan.count,
            payload_bytes: last_plan.payload_bytes,
            surface_reconfigures: 0,
            destroy_presses: 1,
            sim: sim_metrics,
            material: demo.selected_material(),
            credits: demo.credits(),
            credit_draft: demo.credits(),
            deferred_overlay,
            camera_latency: feel.camera_summary(),
            paint_latency: feel.paint_summary(),
            ignite_latency: feel.ignite_summary(),
            detonate_latency: feel.detonate_summary(),
            pending_actions: feel.pending_count(),
            focus_linked: focus_linked(),
            focus_enabled: policy.focus_enabled(),
            offscreen_capture: true,
            story: "Offscreen timing run: no display presentation is measured.",
        };
        let mut actions = OverlayActions::default();
        let mut acks = [None; 64];
        for (index, ack) in feel.acks().take(acks.len()).enumerate() {
            acks[index] = Some(*ack);
        }
        let focus_marks = [None; 8];
        let mut viewports = egui::RawInput::default().viewports;
        viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(2.0);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
            )),
            viewports,
            ..Default::default()
        };
        let mut output = gpu.egui.run_ui(input, |ctx| {
            draw_player_marks(
                ctx,
                &camera,
                acks.iter().flatten().copied(),
                focus_marks.into_iter().flatten(),
                false,
            );
            show_overlay(ctx, &overlay, &mut actions);
        });
        deferred_overlay = actions.deferred_overlay.unwrap_or(deferred_overlay);
        let pixels_per_point = 2.0;
        let paint_jobs = gpu.egui.tessellate(output.shapes, pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [WIDTH, HEIGHT],
            pixels_per_point,
        };
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                gpu.egui_renderer
                    .update_texture(&gpu.device, &gpu.queue, *id, delta);
            }
        }
        let submit_started = Instant::now();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("offscreen-frame"),
            });
        let user_cmds = gpu.egui_renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen,
        );
        let view = gpu
            .target
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen-grid"),
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
                label: Some("offscreen-overlay"),
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
        let submit_cpu_ms = submit_started.elapsed().as_secs_f32() * 1000.0;
        last_submit_cpu_ms = submit_cpu_ms;
        for id in &output.textures_delta.free {
            gpu.egui_renderer.free_texture(id);
        }
        output.textures_delta.clear();
        feel.finish_feedback(clock_origin.elapsed().as_nanos() as u64);
        feel.observe_actions(clock_origin.elapsed().as_nanos() as u64, |x, y| {
            demo.world()
                .cell(x, y)
                .map(|cell| (cell.material as u8, cell.burning))
        });

        if capture_started.is_none()
            && sim_metrics
                .fixture_progress
                .is_some_and(|progress| progress.complete)
            && frame_number > 10
            && last_plan.count > 0
        {
            demo.set_destroy_held(true);
            feel.clear_latencies();
            history = FrameHistory::default();
            last_tick = None;
            capture_started = Some(Instant::now());
            println!(
                "OFFSCREEN_CAPTURE_STARTED policy={} target={}x{} fixture=mixed-overload-v1",
                policy.name(),
                WIDTH,
                HEIGHT
            );
        } else if let Some(started) = capture_started {
            let run_frames = &mut run;
            run_frames.frames = run_frames.frames.saturating_add(1);
            run_frames.last_pending = sim_metrics.slice.pending_cells;
            run_frames.max_pending = run_frames
                .max_pending
                .max(sim_metrics.slice.pending_cells)
                .max(before.pending_cells);
            run_frames.max_ready = run_frames
                .max_ready
                .max(sim_metrics.slice.ready_len)
                .max(before.ready_len);
            run_frames.max_upload_backlog = run_frames.max_upload_backlog.max(last_plan.backlog);
            if sim_cpu_ms >= run_frames.max_sim_cpu_ms {
                run_frames.max_sim_cpu_ms = sim_cpu_ms;
                run_frames.max_sim_slice = sim_metrics.slice;
            }
            run_frames.total_evaluations += sim_metrics.slice.evaluations as u64;
            run_frames.total_blasts += sim_metrics.slice.blasts as u64;
            run_frames.total_recoveries += sim_metrics.slice.recoveries as u64;
            run_frames.total_commands += sim_metrics.slice.commands as u64;
            run_frames.total_selections += sim_metrics.slice.selections as u64;
            run_frames.max_upload_cpu_ms = run_frames.max_upload_cpu_ms.max(upload_cpu_ms);
            run_frames.max_submit_cpu_ms = run_frames.max_submit_cpu_ms.max(submit_cpu_ms);
            let max_interval = run_frames.capture_intervals_ns.last().copied().unwrap_or(0);
            if max_interval > 33_333_333 {
                run_frames.slow_frame_count = run_frames.slow_frame_count.saturating_add(1);
                if max_interval > run_frames.slow_frame_max_ns {
                    run_frames.slow_frame_max_ns = max_interval;
                    run_frames.slow_frame_sim_cpu_ms = sim_cpu_ms;
                    run_frames.slow_frame_upload_cpu_ms = upload_cpu_ms;
                    run_frames.slow_frame_submit_cpu_ms = submit_cpu_ms;
                    run_frames.slow_frame_slice = sim_metrics.slice;
                }
            }
            scripted_action(
                &mut demo,
                &mut feel,
                &mut camera,
                &mut scripted_ignite_target,
                run_frames.frames,
                clock_origin.elapsed().as_nanos() as u64,
            );
            if started.elapsed() >= deadline {
                demo.set_destroy_held(false);
                println!(
                    "OFFSCREEN_CAPTURE_RESULT policy={} frames={} target={}x{} cadence_hz=60 feel_basis=scripted_action_to_cpu_detected_effect_and_upload_queue_write display_presentation=not_measured vsync=not_measured compositor=not_measured scanout=not_measured",
                    policy.name(),
                    run_frames.frames,
                    WIDTH,
                    HEIGHT
                );
                let label = format!("offscreen-{}", policy.name().replace(' ', "-"));
                print_feel(&label, &feel);
                print_capture(&label, run_frames);
                println!("SMOKE_RESULT ok mode=offscreen");
                return Ok(());
            }
        }
        next_frame += FRAME_PERIOD;
        let now = Instant::now();
        if next_frame <= now {
            let missed = now.duration_since(next_frame).as_nanos() / FRAME_PERIOD.as_nanos() + 1;
            next_frame +=
                FRAME_PERIOD * u32::try_from(missed.min(u128::from(u32::MAX))).unwrap_or(u32::MAX);
        }
    }
}

fn create_gpu(world: (u32, u32)) -> Result<OffscreenGpu, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .map_err(|error| format!("request offscreen adapter: {error}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cascade-offscreen"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request offscreen device: {error}"))?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("grid"),
        source: wgpu::ShaderSource::Wgsl(include_str!("grid.wgsl").into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("offscreen-grid"),
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
                format,
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
        layout: &layout,
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
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen-target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let egui = egui::Context::default();
    let egui_renderer =
        egui_wgpu::Renderer::new(&device, format, egui_wgpu::RendererOptions::default());
    Ok(OffscreenGpu {
        _instance: instance,
        device,
        queue,
        pipeline,
        bind_group,
        uniform_buf,
        grid_texture,
        target,
        egui,
        egui_renderer,
    })
}

fn note_player_mark(feel: &mut Feel, mark: PlayerMark, now_ns: u64) {
    feel.admit(Ack {
        kind: mark.kind,
        x: mark.x,
        y: mark.y,
        state: if mark.admitted {
            AckState::Pending
        } else if mark.kind == ActionKind::Ignite {
            AckState::NoEffect
        } else {
            AckState::Rejected
        },
        admitted_ns: now_ns,
        seen_stamp: feel.stamp_of(mark.x / CHUNK_SIZE, mark.y / CHUNK_SIZE),
        baseline_material: mark.material,
        baseline_burning: mark.burning,
        paint_material: mark.paint_material,
        visible_frames: 0,
    });
}

fn scripted_action(
    demo: &mut Demo,
    feel: &mut Feel,
    camera: &mut Camera,
    ignite_target: &mut Option<(u32, u32)>,
    step: u32,
    now_ns: u64,
) {
    let (width, height) = demo.dimensions();
    let action_index = step / 3;
    let mark = match step % 3 {
        0 if ignite_target.is_none() => {
            let x = width / 8 + action_index % 64;
            let y = height / 8 + action_index / 64;
            let mark = demo.paint_material_at(x, y, cascade_sim::Material::Wood);
            if mark.is_some_and(|mark| mark.admitted) {
                *ignite_target = Some((x, y));
            }
            mark
        }
        0 => None,
        1 => ignite_target.take().and_then(|(x, y)| {
            let cell = demo.world().cell_id(x, y)?;
            let result = demo
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
        _ => demo.detonate_at(
            width / 2 + ((action_index % 32) + 1) * 4,
            height / 2 + ((action_index / 32) + 1) * 4,
        ),
    };
    if let Some(mark) = mark {
        note_player_mark(feel, mark, now_ns);
    }
    apply_command(
        camera,
        ViewCommand::PanPixels { dx: 1.0, dy: 0.0 },
        (WIDTH as f32, HEIGHT as f32),
        (width, height),
    );
    feel.note_feedback(now_ns);
}

fn update_focus_regions(
    demo: &mut Demo,
    feel: &mut Feel,
    camera: &Camera,
    last_viewport: &mut Option<FocusRect>,
    submit_age: &mut u32,
) {
    let (width, height) = demo.dimensions();
    let chunks_x = width / CHUNK_SIZE;
    let chunks_y = height / CHUNK_SIZE;
    let (x0, y0) = camera.world_at_pixel(0.0, 0.0);
    let (x1, y1) = camera.world_at_pixel(WIDTH as f32, HEIGHT as f32);
    let min_chunk_x = chunk_index(x0, chunks_x);
    let min_chunk_y = chunk_index(y0, chunks_y);
    let viewport = FocusRect {
        min_chunk_x,
        min_chunk_y,
        max_chunk_x: (chunk_index(x1, chunks_x) + 1).min(chunks_x),
        max_chunk_y: (chunk_index(y1, chunks_y) + 1).min(chunks_y),
        kind: FocusKind::Viewport,
    };
    if focus_linked() && demo.policy().focus_enabled() {
        if *last_viewport != Some(viewport) || *submit_age >= 4 {
            submit_viewport(demo.world_mut(), viewport);
            *last_viewport = Some(viewport);
            *submit_age = 0;
        } else {
            *submit_age = submit_age.saturating_add(1);
        }
        let mut regions = [viewport; 8];
        let mut len = 0;
        for region in demo.world().active_focus_regions() {
            if len == regions.len() {
                break;
            }
            regions[len] = FocusRect {
                min_chunk_x: region.min_chunk_x,
                min_chunk_y: region.min_chunk_y,
                max_chunk_x: region.max_chunk_x,
                max_chunk_y: region.max_chunk_y,
                kind: FocusKind::Viewport,
            };
            len += 1;
        }
        let focus_regions = if len == 0 {
            std::slice::from_ref(&viewport)
        } else {
            &regions[..len]
        };
        feel.set_focus(focus_regions);
    } else {
        if focus_linked() {
            feel.set_focus(&[]);
        } else {
            feel.set_focus(std::slice::from_ref(&viewport));
        }
    }
}

fn chunk_index(world: f32, chunks: u32) -> u32 {
    if world <= 0.0 {
        0
    } else {
        ((world as u32) / CHUNK_SIZE).min(chunks.saturating_sub(1))
    }
}
