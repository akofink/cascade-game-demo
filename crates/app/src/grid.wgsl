// Full-screen triangle. Fragment @builtin(position) is framebuffer pixels, origin top-left, y down.
// `Frame` matches `frame_uniform`: origin at 0, world size at 8, cells_per_pixel at 16, palette at 32.

struct Frame {
    origin: vec2<f32>,
    world_size: vec2<f32>,
    cells_per_pixel: f32,
    _pad0: f32,
    _pad1: vec2<f32>,
    palette: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var grid_tex: texture_2d<u32>;
@group(0) @binding(1) var<uniform> frame: Frame;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let xy = positions[vertex_index];
    return vec4<f32>(xy, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let world = frame.origin + position.xy * frame.cells_per_pixel;
    if (world.x < 0.0 || world.y < 0.0 || world.x >= frame.world_size.x || world.y >= frame.world_size.y) {
        return vec4<f32>(0.04, 0.045, 0.06, 1.0);
    }
    let texel = textureLoad(grid_tex, vec2<i32>(floor(world)), 0).r;
    return frame.palette[min(texel, 7u)];
}
