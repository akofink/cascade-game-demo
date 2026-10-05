//! Material byte grid and the placeholder source that will later be swapped for `cascade-sim`.

use std::fmt;

pub const CHUNK_SIZE: u32 = 32;
pub const CHUNK_CELLS: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;
pub const MAX_DIRTY_PER_TICK: usize = 16;
pub const PLACEHOLDER_SIZE: u32 = 1024;
pub const MAX_WORLD_AXIS: u32 = 4096;

/// Cell the native smoke sample reads. Seeded as sand and outside the first brush step.
pub const SMOKE_SAMPLE_CELL: (u32, u32) = (220, 220);

/// Palette order matches `cascade-sim` material ids 0..=5. Values are sRGB bytes.
pub const PALETTE: [[u8; 4]; 8] = [
    [20, 24, 32, 255],
    [128, 128, 136, 255],
    [112, 74, 40, 255],
    [196, 168, 96, 255],
    [196, 64, 32, 255],
    [48, 112, 196, 255],
    [220, 180, 48, 255],
    [255, 236, 120, 255],
];

pub const MATERIAL_SAND: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ChunkCoord {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridError {
    pub reason: &'static str,
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason)
    }
}

#[derive(Clone, Debug)]
pub struct MaterialGrid {
    width: u32,
    height: u32,
    materials: Vec<u8>,
}

impl MaterialGrid {
    pub fn new(width: u32, height: u32) -> Result<Self, GridError> {
        if width == 0 || height == 0 || width > MAX_WORLD_AXIS || height > MAX_WORLD_AXIS {
            return Err(GridError {
                reason: "world size must be in 1..=4096",
            });
        }
        if !width.is_multiple_of(CHUNK_SIZE) || !height.is_multiple_of(CHUNK_SIZE) {
            return Err(GridError {
                reason: "world size must be a multiple of 32",
            });
        }
        let cells = (width as usize)
            .checked_mul(height as usize)
            .ok_or(GridError {
                reason: "world size overflows",
            })?;
        Ok(Self {
            width,
            height,
            materials: vec![0; cells],
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn chunk_extent(&self) -> (u32, u32) {
        (self.width / CHUNK_SIZE, self.height / CHUNK_SIZE)
    }

    pub fn materials(&self) -> &[u8] {
        &self.materials
    }

    pub fn material_at(&self, x: u32, y: u32) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y as usize) * (self.width as usize) + x as usize;
        Some(self.materials[index])
    }

    pub fn set(&mut self, x: u32, y: u32, material: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = (y as usize) * (self.width as usize) + x as usize;
        self.materials[index] = material;
        true
    }

    /// Tightly packed R8 rows. This is the `write_texture` payload, not a 256-aligned copy.
    pub fn copy_chunk(&self, chunk: ChunkCoord, out: &mut [u8]) -> bool {
        if out.len() < CHUNK_CELLS {
            return false;
        }
        let (chunks_x, chunks_y) = self.chunk_extent();
        if chunk.x >= chunks_x || chunk.y >= chunks_y {
            return false;
        }
        let x0 = chunk.x * CHUNK_SIZE;
        let y0 = chunk.y * CHUNK_SIZE;
        for row in 0..CHUNK_SIZE {
            let src = ((y0 + row) * self.width + x0) as usize;
            let dst = (row * CHUNK_SIZE) as usize;
            out[dst..dst + CHUNK_SIZE as usize]
                .copy_from_slice(&self.materials[src..src + CHUNK_SIZE as usize]);
        }
        true
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DirtyBatch {
    pub chunks: [ChunkCoord; MAX_DIRTY_PER_TICK],
    pub count: usize,
}

impl Default for DirtyBatch {
    fn default() -> Self {
        Self {
            chunks: [ChunkCoord::default(); MAX_DIRTY_PER_TICK],
            count: 0,
        }
    }
}

/// Cheap animated pattern. Each tick dirties at most [`MAX_DIRTY_PER_TICK`] chunks.
#[derive(Clone, Debug)]
pub struct PlaceholderSource {
    grid: MaterialGrid,
    frame: u64,
}

impl PlaceholderSource {
    pub fn new() -> Result<Self, GridError> {
        let mut grid = MaterialGrid::new(PLACEHOLDER_SIZE, PLACEHOLDER_SIZE)?;
        paint_border(&mut grid, 4, 1);
        paint_rect(&mut grid, 200, 200, 128, 128, MATERIAL_SAND);
        paint_rect(&mut grid, 500, 400, 64, 64, 5);
        paint_rect(&mut grid, 700, 180, 48, 96, 4);
        Ok(Self { grid, frame: 0 })
    }

    pub fn grid(&self) -> &MaterialGrid {
        &self.grid
    }

    pub fn tick(&mut self) -> DirtyBatch {
        let (chunks_x, chunks_y) = self.grid.chunk_extent();
        let travel = chunks_x.saturating_sub(2).max(1);
        let x = (self.frame as u32) % travel;
        let y = ((self.frame as u32) / travel) % chunks_y.saturating_sub(1).max(1);
        let material = 3 + (self.frame % 3) as u8;
        let mut batch = DirtyBatch::default();
        for dy in 0..2 {
            for dx in 0..2 {
                let chunk = ChunkCoord {
                    x: x + dx,
                    y: y + dy,
                };
                paint_chunk(&mut self.grid, chunk, material);
                push_unique(&mut batch, chunk);
            }
        }
        self.frame += 1;
        batch
    }
}

fn paint_border(grid: &mut MaterialGrid, thickness: u32, material: u8) {
    let (width, height) = grid.size();
    for y in 0..height {
        for x in 0..width {
            if x < thickness || y < thickness || x + thickness >= width || y + thickness >= height {
                let _ = grid.set(x, y, material);
            }
        }
    }
}

fn paint_rect(grid: &mut MaterialGrid, x: u32, y: u32, w: u32, h: u32, material: u8) {
    for row in y..y.saturating_add(h) {
        for col in x..x.saturating_add(w) {
            let _ = grid.set(col, row, material);
        }
    }
}

fn paint_chunk(grid: &mut MaterialGrid, chunk: ChunkCoord, material: u8) {
    let x0 = chunk.x * CHUNK_SIZE;
    let y0 = chunk.y * CHUNK_SIZE;
    paint_rect(grid, x0, y0, CHUNK_SIZE, CHUNK_SIZE, material);
}

fn push_unique(batch: &mut DirtyBatch, chunk: ChunkCoord) {
    if batch.count == MAX_DIRTY_PER_TICK {
        return;
    }
    if batch.chunks[..batch.count].contains(&chunk) {
        return;
    }
    batch.chunks[batch.count] = chunk;
    batch.count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unaligned_and_oversized_worlds() {
        assert!(MaterialGrid::new(31, 32).is_err());
        assert!(MaterialGrid::new(4097, 32).is_err());
        assert!(MaterialGrid::new(0, 32).is_err());
    }

    #[test]
    fn chunk_copy_is_tightly_packed() {
        let mut grid = MaterialGrid::new(64, 32).unwrap();
        assert!(grid.set(35, 2, 7));
        let mut bytes = [0_u8; CHUNK_CELLS];
        assert!(grid.copy_chunk(ChunkCoord { x: 1, y: 0 }, &mut bytes));
        assert_eq!(bytes[2 * 32 + 3], 7);
        assert!(!grid.copy_chunk(ChunkCoord { x: 2, y: 0 }, &mut bytes));
    }

    #[test]
    fn placeholder_tick_is_bounded_and_sample_cell_is_sand() {
        let mut source = PlaceholderSource::new().unwrap();
        let (sx, sy) = SMOKE_SAMPLE_CELL;
        assert_eq!(source.grid().material_at(sx, sy), Some(MATERIAL_SAND));
        let (chunks_x, chunks_y) = source.grid().chunk_extent();
        for _ in 0..8 {
            let batch = source.tick();
            assert!(batch.count <= MAX_DIRTY_PER_TICK);
            assert!(batch.count <= 4);
            for chunk in &batch.chunks[..batch.count] {
                assert!(chunk.x < chunks_x);
                assert!(chunk.y < chunks_y);
            }
        }
        assert_eq!(
            source.grid().materials().len(),
            (PLACEHOLDER_SIZE as usize).pow(2)
        );
    }
}
