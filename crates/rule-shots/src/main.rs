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
const SCALE: u32 = 5;
const PALETTE: [[u8; 3]; 6] = [
    [23, 33, 43],
    [120, 131, 141],
    [141, 78, 49],
    [229, 184, 79],
    [240, 88, 50],
    [69, 169, 197],
];

fn world() -> Result<World, Box<dyn Error>> {
    let world = World::new_with_policy(
        W,
        H,
        Credits::new(20_000),
        Capacity::new(2048),
        Capacity::new(256),
        SchedulerPolicy::Traditional,
    )
    .map_err(|error| std::io::Error::other(format!("create sim world: {error:?}")))?;
    Ok(world)
}

fn paint(world: &mut World, x: u32, y: u32, material: Material) {
    let cell = world.cell_id(x, y).expect("small fixed scene coordinate");
    world.submit(Command::Paint { cell, material });
}

fn settle_setup(world: &mut World) {
    world.step();
}

fn advance(world: &mut World, slices: u32) {
    for _ in 0..slices {
        world.step();
    }
}

fn cell_color(world: &World, x: u32, y: u32) -> [u8; 3] {
    let cell = world.cell(x, y).expect("small fixed scene coordinate");
    let mut rgb = PALETTE[cell.material as usize];
    if world.cell_pending(x, y).unwrap_or(false) {
        rgb = [rgb[0].saturating_add(40), rgb[1].saturating_add(24), rgb[2]];
    }
    if cell.burning > 0 {
        rgb = [
            255,
            72_u8.saturating_add(cell.burning.saturating_mul(8)),
            20,
        ];
    }
    if cell.state > 0 {
        rgb = [255, 54_u8.saturating_add(cell.state.saturating_mul(8)), 20];
    }
    let id = world.cell_id(x, y).expect("small fixed scene coordinate");
    if world.cell_executed_last(id) {
        for channel in &mut rgb {
            *channel = channel.saturating_add(90);
        }
    }
    rgb
}

struct Snapshot(Vec<[u8; 3]>);

fn snapshot(world: &World) -> Snapshot {
    Snapshot(
        (0..H)
            .flat_map(|y| (0..W).map(move |x| cell_color(world, x, y)))
            .collect(),
    )
}

fn save_pair(path: &Path, before: &Snapshot, after: &World) -> Result<(), Box<dyn Error>> {
    let width = W * SCALE * 2 + 3;
    let height = H * SCALE;
    let mut bytes = vec![0_u8; (width * height * 3) as usize];
    for frame_index in 0..2 {
        let x_offset = frame_index as u32 * (W * SCALE + 3);
        for y in 0..H {
            for x in 0..W {
                let rgb = if frame_index == 0 {
                    before.0[(y * W + x) as usize]
                } else {
                    cell_color(after, x, y)
                };
                for sy in 0..SCALE {
                    for sx in 0..SCALE {
                        let px = x_offset + x * SCALE + sx;
                        let py = y * SCALE + sy;
                        let index = ((py * width + px) * 3) as usize;
                        bytes[index..index + 3].copy_from_slice(&rgb);
                    }
                }
            }
        }
    }
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&bytes)?;
    Ok(())
}

fn capture(name: &str, before: &Snapshot, after: &World, dir: &Path) -> Result<(), Box<dyn Error>> {
    save_pair(&dir.join(format!("{name}.png")), before, after)
}

fn generate(dir: PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(&dir)?;
    let mut sand = world()?;
    for x in 0..W {
        paint(&mut sand, x, 22, Material::Stone);
    }
    for y in 4..10 {
        paint(&mut sand, 16, y, Material::Sand);
    }
    settle_setup(&mut sand);
    let sand_before = snapshot(&sand);
    advance(&mut sand, 12);
    capture("sand-gravity", &sand_before, &sand, &dir)?;

    let mut water = world()?;
    for x in 0..W {
        paint(&mut water, x, 22, Material::Stone);
    }
    for y in 4..10 {
        paint(&mut water, 16, y, Material::Water);
    }
    settle_setup(&mut water);
    let water_before = snapshot(&water);
    advance(&mut water, 12);
    capture("water-spread", &water_before, &water, &dir)?;

    let mut fire = world()?;
    for x in 10..22 {
        paint(&mut fire, x, 14, Material::Wood);
    }
    settle_setup(&mut fire);
    let _ = fire.ignite(fire.cell_id(16, 14).expect("fixed scene coordinate"));
    let fire_before = snapshot(&fire);
    advance(&mut fire, 5);
    capture("wood-fire", &fire_before, &fire, &dir)?;

    let mut explosive = world()?;
    for x in 10..22 {
        paint(&mut explosive, x, 14, Material::Explosive);
    }
    settle_setup(&mut explosive);
    let _ = explosive.ignite(explosive.cell_id(10, 14).expect("fixed scene coordinate"));
    let explosive_before = snapshot(&explosive);
    advance(&mut explosive, 1);
    capture("explosive-ignition", &explosive_before, &explosive, &dir)?;

    let mut blast = world()?;
    for x in 8..24 {
        paint(&mut blast, x, 14, Material::Explosive);
    }
    settle_setup(&mut blast);
    blast
        .trigger_blast(blast.cell_id(8, 14).expect("fixed scene coordinate"), 8)
        .map_err(|error| std::io::Error::other(format!("trigger blast: {error:?}")))?;
    let blast_before = snapshot(&blast);
    advance(&mut blast, 4);
    capture("blast-attenuation", &blast_before, &blast, &dir)?;

    let mut waking = world()?;
    for x in 8..24 {
        paint(&mut waking, x, 18, Material::Stone);
    }
    for x in 10..22 {
        paint(&mut waking, x, 10, Material::Sand);
    }
    settle_setup(&mut waking);
    advance(&mut waking, 8);
    let waking_before = snapshot(&waking);
    let _ =
        waking.mark_cell_for_evaluation(waking.cell_id(16, 18).expect("fixed scene coordinate"));
    advance(&mut waking, 1);
    capture("dormancy-waking", &waking_before, &waking, &dir)?;

    let mut reset = world()?;
    for x in 0..W {
        paint(&mut reset, x, 0, Material::Wood);
    }
    for y in 6..18 {
        for x in 8..24 {
            paint(&mut reset, x, y, Material::Wood);
        }
    }
    settle_setup(&mut reset);
    let reset_before = snapshot(&reset);
    reset
        .set_budget(Credits::new(25))
        .map_err(|error| std::io::Error::other(format!("set reset budget: {error:?}")))?;
    reset
        .reset()
        .map_err(|error| std::io::Error::other(format!("begin reset: {error:?}")))?;
    advance(&mut reset, 4);
    capture("reset-preparation", &reset_before, &reset, &dir)?;

    let mut fixture = world()?;
    for y in 6..18 {
        for x in 8..24 {
            paint(&mut fixture, x, y, Material::Wood);
        }
    }
    settle_setup(&mut fixture);
    let fixture_before = snapshot(&fixture);
    fixture
        .set_budget(Credits::new(1024))
        .map_err(|error| std::io::Error::other(format!("set fixture budget: {error:?}")))?;
    fixture
        .start_fixture(ScenarioDescriptor::get(FixtureId::ExplosiveLattice))
        .map_err(|error| std::io::Error::other(format!("start fixture: {error:?}")))?;
    advance(&mut fixture, 6);
    capture("fixture-preparation", &fixture_before, &fixture, &dir)?;

    println!(
        "Generated 8 deterministic before/after rule illustrations in {}",
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
