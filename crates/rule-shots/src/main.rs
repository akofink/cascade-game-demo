use cascade_sim::{
    Capacity, Command, Credits, Material, SchedulerPolicy, World,
    fixtures::{FixtureId, ScenarioDescriptor},
};
use std::{
    error::Error,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

const W: u32 = 32;
const H: u32 = 24;
const SCALE: u32 = 7;
const FRAMES: usize = 4;
const HEADER: u32 = 22;
const FOOTER: u32 = 20;
const GAP: u32 = 8;
const PALETTE: [[u8; 3]; 6] = [
    [23, 33, 43],
    [120, 131, 141],
    [141, 78, 49],
    [229, 184, 79],
    [240, 88, 50],
    [69, 169, 197],
];
const AMBER: [u8; 3] = [255, 205, 75];
const PALE: [u8; 3] = [230, 238, 243];

fn world() -> Result<World, Box<dyn Error>> {
    Ok(World::new_with_policy(
        W,
        H,
        Credits::new(20_000),
        Capacity::new(2048),
        Capacity::new(256),
        SchedulerPolicy::Traditional,
    )
    .map_err(|error| std::io::Error::other(format!("create sim world: {error:?}")))?)
}

fn paint(world: &mut World, x: u32, y: u32, material: Material) {
    let cell = world.cell_id(x, y).expect("small fixed scene coordinate");
    world.submit(Command::Paint { cell, material });
}

fn step(world: &mut World) {
    world.step();
}

fn advance(world: &mut World, slices: u32) {
    for _ in 0..slices {
        step(world);
    }
}

fn cell_color(world: &World, x: u32, y: u32) -> [u8; 3] {
    let cell = world.cell(x, y).expect("small fixed scene coordinate");
    let mut rgb = PALETTE[cell.material as usize];
    if world.cell_pending(x, y).unwrap_or(false) {
        rgb = blend(rgb, AMBER, 105);
    }
    if cell.burning > 0 {
        rgb = [
            255,
            92_u8.saturating_add(cell.burning.saturating_mul(8)),
            28,
        ];
    } else if cell.state > 0 {
        rgb = [255, 54_u8.saturating_add(cell.state.saturating_mul(8)), 20];
    }
    let id = world.cell_id(x, y).expect("small fixed scene coordinate");
    if world.cell_executed_last(id) {
        rgb = blend(rgb, PALE, 145);
    }
    rgb
}

fn blend(base: [u8; 3], overlay: [u8; 3], alpha: u16) -> [u8; 3] {
    std::array::from_fn(|channel| {
        ((u16::from(base[channel]) * (255 - alpha) + u16::from(overlay[channel]) * alpha) / 255)
            as u8
    })
}

#[derive(Clone)]
struct Frame {
    slice: u64,
    cells: Vec<[u8; 3]>,
}

fn snapshot(world: &World) -> Frame {
    Frame {
        slice: world.slice_index(),
        cells: (0..H)
            .flat_map(|y| (0..W).map(move |x| cell_color(world, x, y)))
            .collect(),
    }
}

fn pixel(bytes: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 3]) {
    let index = ((y * width + x) * 3) as usize;
    bytes[index..index + 3].copy_from_slice(&color);
}

fn rect(bytes: &mut [u8], width: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 3]) {
    for py in y..y + h {
        for px in x..x + w {
            pixel(bytes, width, px, py, color);
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '|' => [4, 4, 4, 4, 4, 4, 4],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        _ => [0; 7],
    }
}

fn text(bytes: &mut [u8], width: u32, x: u32, y: u32, value: &str, color: [u8; 3]) {
    const ZOOM: u32 = 2;
    for (index, character) in value.chars().enumerate() {
        let left = x + index as u32 * 12;
        for (row, bits) in glyph(character).into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    rect(
                        bytes,
                        width,
                        left + column * ZOOM,
                        y + row as u32 * ZOOM,
                        ZOOM,
                        ZOOM,
                        color,
                    );
                }
            }
        }
    }
}

fn save_strip(path: &Path, frames: &[Frame]) -> Result<(), Box<dyn Error>> {
    assert_eq!(frames.len(), FRAMES);
    let panel_width = W * SCALE;
    let panel_height = HEADER + H * SCALE;
    let width = FRAMES as u32 * panel_width + (FRAMES as u32 - 1) * GAP;
    let height = panel_height + FOOTER;
    let mut bytes = vec![0_u8; (width * height * 3) as usize];
    rect(&mut bytes, width, 0, 0, width, height, PALETTE[0]);
    for (frame_index, frame) in frames.iter().enumerate() {
        let left = frame_index as u32 * (panel_width + GAP);
        text(
            &mut bytes,
            width,
            left + 8,
            5,
            &format!("STEP {}", frame.slice),
            PALE,
        );
        for y in 0..H {
            for x in 0..W {
                let color = frame.cells[(y * W + x) as usize];
                rect(
                    &mut bytes,
                    width,
                    left + x * SCALE,
                    HEADER + y * SCALE,
                    SCALE,
                    SCALE,
                    color,
                );
            }
        }
    }
    text(
        &mut bytes,
        width,
        8,
        panel_height + 4,
        "AMBER=PENDING | ORANGE=HEAT/BLAST | PALE=EXECUTED",
        PALE,
    );
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&bytes)?;
    Ok(())
}

fn capture(name: &str, frames: &[Frame], dir: &Path) -> Result<(), Box<dyn Error>> {
    save_strip(&dir.join(format!("{name}.png")), frames)
}

fn generate(dir: PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(&dir)?;

    let mut sand = world()?;
    for x in 0..W {
        paint(&mut sand, x, 22, Material::Stone);
    }
    for x in 12..21 {
        paint(&mut sand, x, 15, Material::Stone);
    }
    for (x, y) in [
        (13, 14),
        (14, 14),
        (15, 14),
        (16, 14),
        (17, 14),
        (18, 14),
        (14, 13),
        (15, 13),
        (16, 13),
        (17, 13),
        (15, 12),
        (16, 12),
    ] {
        paint(&mut sand, x, y, Material::Sand);
    }
    step(&mut sand);
    let mut frames = vec![snapshot(&sand)];
    advance(&mut sand, 4);
    frames.push(snapshot(&sand));
    paint(&mut sand, 16, 15, Material::Air);
    step(&mut sand);
    frames.push(snapshot(&sand));
    advance(&mut sand, 7);
    frames.push(snapshot(&sand));
    capture("sand-gravity", &frames, &dir)?;

    let mut water = world()?;
    for x in 0..W {
        paint(&mut water, x, 21, Material::Stone);
    }
    for y in 8..21 {
        paint(&mut water, 8, y, Material::Stone);
        paint(&mut water, 23, y, Material::Stone);
    }
    for x in 9..23 {
        if x != 16 {
            paint(&mut water, x, 18, Material::Stone);
        }
    }
    for y in 10..18 {
        for x in 12..20 {
            paint(&mut water, x, y, Material::Water);
        }
    }
    step(&mut water);
    let mut frames = vec![snapshot(&water)];
    for count in [3, 5, 8] {
        advance(&mut water, count);
        frames.push(snapshot(&water));
    }
    capture("water-spread", &frames, &dir)?;

    let mut fire = world()?;
    for y in 10..16 {
        for x in 10..22 {
            paint(&mut fire, x, y, Material::Wood);
        }
    }
    step(&mut fire);
    let mut frames = vec![snapshot(&fire)];
    let _ = fire.ignite(fire.cell_id(16, 13).expect("fixed scene coordinate"));
    for count in [2, 3, 5] {
        advance(&mut fire, count);
        frames.push(snapshot(&fire));
    }
    capture("wood-fire", &frames, &dir)?;

    let mut explosive = world()?;
    for x in 7..25 {
        paint(&mut explosive, x, 14, Material::Explosive);
    }
    for y in 9..20 {
        paint(&mut explosive, 7, y, Material::Stone);
        paint(&mut explosive, 24, y, Material::Stone);
    }
    for x in 18..23 {
        paint(&mut explosive, x, 13, Material::Wood);
        paint(&mut explosive, x, 15, Material::Wood);
    }
    step(&mut explosive);
    let mut frames = vec![snapshot(&explosive)];
    let _ = explosive.ignite(explosive.cell_id(8, 14).expect("fixed scene coordinate"));
    for count in [1, 2, 3] {
        advance(&mut explosive, count);
        frames.push(snapshot(&explosive));
    }
    capture("explosive-ignition", &frames, &dir)?;

    let mut blast = world()?;
    for y in 8..21 {
        for x in 5..27 {
            if x == 5 || x == 26 || y == 8 || y == 20 {
                paint(&mut blast, x, y, Material::Stone);
            }
        }
    }
    for x in 8..24 {
        paint(&mut blast, x, 14, Material::Explosive);
    }
    for x in 20..24 {
        paint(&mut blast, x, 13, Material::Wood);
        paint(&mut blast, x, 15, Material::Wood);
    }
    step(&mut blast);
    let mut frames = vec![snapshot(&blast)];
    blast
        .trigger_blast(blast.cell_id(8, 14).expect("fixed scene coordinate"), 8)
        .map_err(|error| std::io::Error::other(format!("trigger blast: {error:?}")))?;
    for count in [1, 2, 3] {
        advance(&mut blast, count);
        frames.push(snapshot(&blast));
    }
    capture("blast-attenuation", &frames, &dir)?;

    let mut waking = world()?;
    for x in 8..24 {
        paint(&mut waking, x, 18, Material::Stone);
    }
    for x in 10..22 {
        paint(&mut waking, x, 10, Material::Sand);
    }
    step(&mut waking);
    advance(&mut waking, 8);
    let mut frames = vec![snapshot(&waking)];
    let _ =
        waking.mark_cell_for_evaluation(waking.cell_id(16, 18).expect("fixed scene coordinate"));
    for _ in 0..3 {
        step(&mut waking);
        frames.push(snapshot(&waking));
    }
    capture("dormancy-waking", &frames, &dir)?;

    let mut reset = world()?;
    for y in 0..16 {
        for x in 0..W {
            paint(&mut reset, x, y, Material::Wood);
        }
    }
    step(&mut reset);
    let mut frames = vec![snapshot(&reset)];
    reset
        .set_budget(Credits::new(100))
        .map_err(|error| std::io::Error::other(format!("set reset budget: {error:?}")))?;
    reset
        .reset()
        .map_err(|error| std::io::Error::other(format!("begin reset: {error:?}")))?;
    for _ in 0..3 {
        advance(&mut reset, 4);
        frames.push(snapshot(&reset));
    }
    capture("reset-preparation", &frames, &dir)?;

    let mut fixture = world()?;
    for y in 0..16 {
        for x in 0..W {
            paint(&mut fixture, x, y, Material::Wood);
        }
    }
    step(&mut fixture);
    let mut frames = vec![snapshot(&fixture)];
    fixture
        .set_budget(Credits::new(160))
        .map_err(|error| std::io::Error::other(format!("set fixture budget: {error:?}")))?;
    fixture
        .start_fixture(ScenarioDescriptor::get(FixtureId::ExplosiveLattice))
        .map_err(|error| std::io::Error::other(format!("start fixture: {error:?}")))?;
    for slices in [19, 20, 20] {
        advance(&mut fixture, slices);
        frames.push(snapshot(&fixture));
    }
    capture("fixture-preparation", &frames, &dir)?;

    println!(
        "Generated 8 deterministic four-frame, 32 by 24 rule strips in {}",
        dir.display()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/rules-images"));
    generate(dir)
}
