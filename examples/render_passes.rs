//! Development evidence probe using the actual private render modules.
pub use oxillustrate::*;
#[path = "../src/outlines.rs"]
mod outlines;
#[path = "../src/raster.rs"]
mod raster;
#[path = "../src/shading.rs"]
mod shading;

use std::io::Write;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 4 {
        return Err("usage: render_passes INPUT SETTINGS OUTPUT_DIRECTORY".into());
    }
    let settings: SceneSettings = serde_json::from_str(&std::fs::read_to_string(&arguments[2])?)?;
    let output = Path::new(&arguments[3]);
    std::fs::create_dir_all(output)?;
    let start = Instant::now();
    let structure = load_structure(Path::new(&arguments[1]))?;
    let loaded = start.elapsed().as_secs_f64();
    let scene = prepare_scene(&structure, &settings)?;
    let prepared = start.elapsed().as_secs_f64();
    let render_start = Instant::now();
    let rendered = render(&scene)?;
    let render_seconds = render_start.elapsed().as_secs_f64();
    let raster_start = Instant::now();
    let buffer = raster::rasterize(&scene)?;
    let raster_seconds = raster_start.elapsed().as_secs_f64();
    let mut owners =
        std::io::BufWriter::new(std::fs::File::create(output.join("sphere_source.tsv"))?);
    for sphere in &scene.spheres {
        writeln!(
            owners,
            "{} {} {}",
            sphere.source_atom_index, sphere.chain_id, sphere.assembly_instance
        )?;
    }
    let mut depth = std::io::BufWriter::new(std::fs::File::create(output.join("depth_atom.tsv"))?);
    writeln!(depth, "{} {}", scene.width, scene.height)?;
    let mut mask = Vec::new();
    let mut shaded = Vec::new();
    let mut outline = Vec::new();
    let mut final_pixels = Vec::new();
    let shading = shading::Shading::new(&scene, &buffer);
    let pass_start = Instant::now();
    for row in 0..scene.height as usize {
        for col in 0..scene.width as usize {
            let index = row * scene.width as usize + col;
            let owner = buffer.atom_index[index].map_or(0, |id| id + 1);
            writeln!(depth, "{:.12} {owner}", buffer.depth[index])?;
            mask.extend([if owner == 0 { 255 } else { 0 }; 3]);
            let opacity = outlines::opacity(&scene, &buffer, row, col);
            outline.extend([(opacity * 255.0).trunc().clamp(0.0, 255.0) as u8; 3]);
            for channel in shading.color(row, col) {
                shaded.push((channel * 255.0).trunc().clamp(0.0, 255.0) as u8);
                final_pixels.push(
                    ((1.0 - opacity) * channel * 255.0)
                        .trunc()
                        .clamp(0.0, 255.0) as u8,
                );
            }
        }
    }
    let passes = pass_start.elapsed().as_secs_f64();
    if final_pixels != rendered.pixels {
        return Err("diagnostic pass composition differs from production render".into());
    }
    for (name, pixels) in [
        ("mask", mask),
        ("shaded", shaded),
        ("outline", outline),
        ("image", final_pixels),
    ] {
        RenderedImage {
            width: scene.width,
            height: scene.height,
            pixels,
        }
        .save_png(output.join(format!("{name}.png")))?;
    }
    println!(
        "{}",
        serde_json::json!({"source_atoms": structure.atoms.len(),
        "spheres": scene.spheres.len(), "instances": scene.spheres.iter().map(|s| s.assembly_instance).collect::<std::collections::BTreeSet<_>>().len(),
        "load_seconds": loaded, "prepare_seconds": prepared-loaded,
        "raster_seconds": raster_seconds, "render_seconds": render_seconds,
        "diagnostic_pass_seconds": passes,
        "raw_min": buffer.raw_min, "raw_max": buffer.raw_max,
        "scale": scene.pixels_per_angstrom, "diagnostics": scene.diagnostics})
    );
    Ok(())
}
