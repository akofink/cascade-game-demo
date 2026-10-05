//! Pan/zoom camera. World origin is the top-left cell under the view.

use crate::grid::PALETTE;

/// `Frame` uniform size. Layout is the WGSL uniform ABI, not Rust field order.
pub const UNIFORM_BYTES: usize = 160;

const MIN_CELLS_PER_PIXEL: f32 = 1.0 / 64.0;
const MAX_CELLS_PER_PIXEL: f32 = 32.0;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub origin_x: f32,
    pub origin_y: f32,
    pub cells_per_pixel: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            origin_x: 0.0,
            origin_y: 0.0,
            cells_per_pixel: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ViewCommand {
    /// Grab-the-map pan. Positive `dx` moves content right.
    PanPixels { dx: f32, dy: f32 },
    /// `factor` > 1 zooms in, keeping the world point under the cursor fixed.
    ZoomAt {
        cursor_x: f32,
        cursor_y: f32,
        factor: f32,
    },
}

impl Camera {
    pub fn fit(&mut self, world: (u32, u32), viewport: (f32, f32)) {
        if viewport.0 <= 1.0 || viewport.1 <= 1.0 || world.0 == 0 || world.1 == 0 {
            return;
        }
        let cpp_x = world.0 as f32 / viewport.0;
        let cpp_y = world.1 as f32 / viewport.1;
        self.cells_per_pixel =
            (cpp_x.max(cpp_y) * 1.05).clamp(MIN_CELLS_PER_PIXEL, MAX_CELLS_PER_PIXEL);
        let view_w = viewport.0 * self.cells_per_pixel;
        let view_h = viewport.1 * self.cells_per_pixel;
        self.origin_x = (world.0 as f32 - view_w) * 0.5;
        self.origin_y = (world.1 as f32 - view_h) * 0.5;
        self.clamp_origin(viewport, (world.0 as f32, world.1 as f32));
    }

    pub fn world_at_pixel(&self, px: f32, py: f32) -> (f32, f32) {
        (
            self.origin_x + px * self.cells_per_pixel,
            self.origin_y + py * self.cells_per_pixel,
        )
    }

    fn clamp_zoom(&mut self) {
        self.cells_per_pixel = self
            .cells_per_pixel
            .clamp(MIN_CELLS_PER_PIXEL, MAX_CELLS_PER_PIXEL);
    }

    fn clamp_origin(&mut self, viewport: (f32, f32), world: (f32, f32)) {
        let view_w = viewport.0 * self.cells_per_pixel;
        let view_h = viewport.1 * self.cells_per_pixel;
        self.origin_x = clamp_axis(self.origin_x, view_w, world.0);
        self.origin_y = clamp_axis(self.origin_y, view_h, world.1);
    }
}

fn clamp_axis(origin: f32, view: f32, world: f32) -> f32 {
    let min = -view * 0.25;
    let max = world - view * 0.75;
    if max < min {
        (world - view) * 0.5
    } else {
        origin.clamp(min, max)
    }
}

pub fn zoom_factor(lines: f32) -> f32 {
    1.1_f32.powf(lines)
}

pub fn apply_command(
    camera: &mut Camera,
    command: ViewCommand,
    viewport: (f32, f32),
    world: (u32, u32),
) {
    let world_f = (world.0 as f32, world.1 as f32);
    match command {
        ViewCommand::PanPixels { dx, dy } => {
            camera.origin_x -= dx * camera.cells_per_pixel;
            camera.origin_y -= dy * camera.cells_per_pixel;
        }
        ViewCommand::ZoomAt {
            cursor_x,
            cursor_y,
            factor,
        } => {
            if !factor.is_finite() || factor <= 0.0 {
                return;
            }
            let (wx, wy) = camera.world_at_pixel(cursor_x, cursor_y);
            camera.cells_per_pixel =
                (camera.cells_per_pixel / factor).clamp(MIN_CELLS_PER_PIXEL, MAX_CELLS_PER_PIXEL);
            camera.origin_x = wx - cursor_x * camera.cells_per_pixel;
            camera.origin_y = wy - cursor_y * camera.cells_per_pixel;
        }
    }
    camera.clamp_zoom();
    camera.clamp_origin(viewport, world_f);
}

/// Pack the WGSL `Frame` uniform. Offsets are part of the shader contract.
pub fn frame_uniform(camera: &Camera, world: (u32, u32)) -> [u8; UNIFORM_BYTES] {
    let mut dst = [0_u8; UNIFORM_BYTES];
    write_f32(&mut dst, 0, camera.origin_x);
    write_f32(&mut dst, 4, camera.origin_y);
    write_f32(&mut dst, 8, world.0 as f32);
    write_f32(&mut dst, 12, world.1 as f32);
    write_f32(&mut dst, 16, camera.cells_per_pixel);
    for (index, color) in PALETTE.iter().enumerate() {
        let offset = 32 + index * 16;
        for (channel, component) in color.iter().enumerate() {
            write_f32(
                &mut dst,
                offset + channel * 4,
                f32::from(*component) / 255.0,
            );
        }
    }
    dst
}

fn write_f32(dst: &mut [u8], offset: usize, value: f32) {
    dst[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_cursor_world_point() {
        let mut camera = Camera {
            origin_x: 10.0,
            origin_y: 20.0,
            cells_per_pixel: 2.0,
        };
        let cursor = (30.0, 40.0);
        let before = camera.world_at_pixel(cursor.0, cursor.1);
        apply_command(
            &mut camera,
            ViewCommand::ZoomAt {
                cursor_x: cursor.0,
                cursor_y: cursor.1,
                factor: zoom_factor(1.0),
            },
            (200.0, 100.0),
            (1024, 1024),
        );
        let after = camera.world_at_pixel(cursor.0, cursor.1);
        assert!(camera.cells_per_pixel < 2.0);
        assert!((before.0 - after.0).abs() < 1e-3);
        assert!((before.1 - after.1).abs() < 1e-3);
    }

    #[test]
    fn grab_pan_moves_origin_opposite_the_drag() {
        let mut camera = Camera::default();
        apply_command(
            &mut camera,
            ViewCommand::PanPixels { dx: 10.0, dy: -4.0 },
            (400.0, 300.0),
            (1024, 1024),
        );
        assert!((camera.origin_x + 10.0).abs() < 1e-4);
        assert!((camera.origin_y - 4.0).abs() < 1e-4);
    }

    #[test]
    fn zoom_clamps_and_ignores_non_positive_factors() {
        let mut camera = Camera {
            cells_per_pixel: 1.0,
            ..Camera::default()
        };
        apply_command(
            &mut camera,
            ViewCommand::ZoomAt {
                cursor_x: 0.0,
                cursor_y: 0.0,
                factor: 10_000.0,
            },
            (800.0, 600.0),
            (1024, 1024),
        );
        assert!((camera.cells_per_pixel - MIN_CELLS_PER_PIXEL).abs() < 1e-6);
        let clamped = camera.cells_per_pixel;
        apply_command(
            &mut camera,
            ViewCommand::ZoomAt {
                cursor_x: 1.0,
                cursor_y: 1.0,
                factor: 0.0,
            },
            (800.0, 600.0),
            (1024, 1024),
        );
        assert_eq!(camera.cells_per_pixel, clamped);
    }

    #[test]
    fn uniform_layout_matches_the_shader_contract() {
        let camera = Camera {
            origin_x: 3.0,
            origin_y: 4.0,
            cells_per_pixel: 0.5,
        };
        let bytes = frame_uniform(&camera, (1024, 512));
        assert_eq!(bytes.len(), UNIFORM_BYTES);
        assert_eq!(f32::from_le_bytes(bytes[16..20].try_into().unwrap()), 0.5);
        assert_eq!(f32::from_le_bytes(bytes[8..12].try_into().unwrap()), 1024.0);
        let sand = 32 + 3 * 16;
        let red = f32::from_le_bytes(bytes[sand..sand + 4].try_into().unwrap());
        assert!((red - PALETTE[3][0] as f32 / 255.0).abs() < 1e-6);
    }
}
