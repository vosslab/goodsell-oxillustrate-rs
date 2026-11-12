use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::{Compression, write::GzEncoder};
use oxillustrate::{Error, RenderedImage, SceneSettings};

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "oxillustrate_render_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

const PDB: &str =
    "ATOM      1  CA  GLY A   1       1.000   2.000   3.000  1.00 10.00           C  \nEND\n";
const CIF: &str = "data_test\nloop_\n_atom_site.id\n_atom_site.type_symbol\n_atom_site.label_atom_id\n_atom_site.label_comp_id\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n1 C CA GLY 1 2 3\n";
const XML: &str = "<datablock><atom_siteCategory><atom_site id='1'><type_symbol>C</type_symbol><label_atom_id>CA</label_atom_id><label_comp_id>GLY</label_comp_id><Cartn_x>1</Cartn_x><Cartn_y>2</Cartn_y><Cartn_z>3</Cartn_z></atom_site></atom_siteCategory></datablock>";

#[test]
fn cli_renders_all_input_formats_and_flags_override_json() {
    let temporary = TemporaryDirectory::new();
    let scene = temporary.0.join("scene.json");
    fs::write(&scene, r#"{"width":0,"height":0,"framing":{"scale":4}}"#).unwrap();
    let mut compressed = GzEncoder::new(Vec::new(), Compression::default());
    compressed.write_all(CIF.as_bytes()).unwrap();
    for (name, bytes) in [
        ("one.pdb", PDB.as_bytes().to_vec()),
        ("one.cif", CIF.as_bytes().to_vec()),
        ("one.xml", XML.as_bytes().to_vec()),
        ("one.cif.gz", compressed.finish().unwrap()),
    ] {
        let input = temporary.0.join(name);
        let output = temporary.0.join(format!("{name}.png"));
        fs::write(&input, bytes).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_illustrate"))
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .arg("--scene")
            .arg(&scene)
            .args(["--width", "32", "--height", "28"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let image = image::open(&output).unwrap().to_rgb8();
        assert_eq!(image.dimensions(), (32, 28));
        assert!(image.pixels().any(|pixel| pixel.0 != [255; 3]));
    }
}

#[test]
fn api_renders_and_rejects_invalid_images_without_replacing_output() {
    let temporary = TemporaryDirectory::new();
    let input = temporary.0.join("one.pdb");
    let output = temporary.0.join("one.png");
    fs::write(&input, PDB).unwrap();
    let structure = oxillustrate::load_structure(&input).unwrap();
    let mut settings = SceneSettings {
        width: 32,
        height: 28,
        ..SceneSettings::default()
    };
    settings.framing.scale = Some(4.0);
    let mut scene = oxillustrate::prepare_scene(&structure, &settings).unwrap();
    oxillustrate::render(&scene)
        .unwrap()
        .save_png(&output)
        .unwrap();
    let original = fs::read(&output).unwrap();
    assert!(
        RenderedImage {
            width: 2,
            height: 2,
            pixels: vec![0; 3],
            alpha: None,
        }
        .save_png(&output)
        .is_err()
    );
    assert_eq!(fs::read(&output).unwrap(), original);
    let blocked = temporary.0.join("directory.png");
    fs::create_dir(&blocked).unwrap();
    assert!(
        RenderedImage {
            width: 1,
            height: 1,
            pixels: vec![0; 3],
            alpha: None,
        }
        .save_png(&blocked)
        .is_err()
    );
    assert!(blocked.is_dir());
    for sphere in &mut scene.spheres {
        sphere.center[2] = 1.0;
    }
    assert!(matches!(
        oxillustrate::render(&scene),
        Err(Error::EmptyScene)
    ));
    assert_eq!(fs::read_dir(&temporary.0).unwrap().count(), 3);
}

#[test]
fn cli_errors_preserve_input_config_and_existing_output() {
    let temporary = TemporaryDirectory::new();
    let input = temporary.0.join("one.pdb");
    let scene = temporary.0.join("scene.json");
    let output = temporary.0.join("existing.png");
    fs::write(&input, PDB).unwrap();
    fs::write(&scene, "{}").unwrap();
    fs::write(&output, "keep existing output").unwrap();
    for target in [&input, &scene] {
        let result = Command::new(env!("CARGO_BIN_EXE_illustrate"))
            .arg(&input)
            .arg("--scene")
            .arg(&scene)
            .arg("--output")
            .arg(target)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("would overwrite input"));
    }
    let result = Command::new(env!("CARGO_BIN_EXE_illustrate"))
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .args(["--model", "999"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
    assert_eq!(fs::read_to_string(&input).unwrap(), PDB);
    assert_eq!(fs::read_to_string(&scene).unwrap(), "{}");
    assert_eq!(fs::read_to_string(&output).unwrap(), "keep existing output");
    assert_eq!(fs::read_dir(&temporary.0).unwrap().count(), 3);
}

#[test]
fn transparent_formats_preserve_alpha_and_composite_without_a_matte() {
    let temporary = TemporaryDirectory::new();
    let input = temporary.0.join("one.pdb");
    fs::write(&input, PDB).unwrap();
    let structure = oxillustrate::load_structure(&input).unwrap();
    let mut settings = SceneSettings {
        width: 64,
        height: 64,
        ..SceneSettings::default()
    };
    settings.framing.scale = Some(10.0);
    let transparent =
        oxillustrate::render(&oxillustrate::prepare_scene(&structure, &settings).unwrap()).unwrap();
    let alpha = transparent.alpha.as_ref().unwrap();
    assert!(alpha.contains(&0) && alpha.contains(&255));
    assert!(alpha.iter().any(|value| (1..255).contains(value)));
    for (rgb, &alpha) in transparent.pixels.as_chunks::<3>().0.iter().zip(alpha) {
        if alpha < 255 {
            assert_eq!(
                *rgb, [0; 3],
                "outline coverage must not contain a background matte"
            );
        }
    }
    for background in [0_u8, 255] {
        settings.background = Some([f64::from(background) / 255.0; 3]);
        let opaque =
            oxillustrate::render(&oxillustrate::prepare_scene(&structure, &settings).unwrap())
                .unwrap();
        assert!(opaque.alpha.is_none());
        for ((rgb, &alpha), expected) in transparent
            .pixels
            .as_chunks::<3>()
            .0
            .iter()
            .zip(alpha)
            .zip(opaque.pixels.as_chunks::<3>().0.iter())
        {
            for channel in 0..3 {
                let composed = (u32::from(rgb[channel]) * u32::from(alpha)
                    + u32::from(background) * (255 - u32::from(alpha)))
                    / 255;
                assert!(composed.abs_diff(u32::from(expected[channel])) <= 1);
            }
        }
    }
    let png = temporary.0.join("transparent.png");
    let webp = temporary.0.join("transparent.WEBP");
    transparent.save(&png).unwrap();
    transparent.save(&webp).unwrap();
    assert_eq!(
        image::guess_format(&fs::read(&png).unwrap()).unwrap(),
        image::ImageFormat::Png
    );
    assert_eq!(
        image::guess_format(&fs::read(&webp).unwrap()).unwrap(),
        image::ImageFormat::WebP
    );
    let decoded = image::open(&png).unwrap().to_rgba8();
    assert_eq!(decoded, image::open(&webp).unwrap().to_rgba8());
    for (pixel, (rgb, &alpha)) in decoded
        .pixels()
        .zip(transparent.pixels.as_chunks::<3>().0.iter().zip(alpha))
    {
        assert_eq!(pixel.0, [rgb[0], rgb[1], rgb[2], alpha]);
    }
    let original = fs::read(&webp).unwrap();
    let invalid = RenderedImage {
        alpha: Some(vec![0; 2]),
        ..transparent
    };
    assert!(invalid.save(&webp).is_err());
    assert_eq!(fs::read(&webp).unwrap(), original);
    let too_wide = RenderedImage {
        width: 16385,
        height: 1,
        pixels: vec![0; 16385 * 3],
        alpha: None,
    };
    assert!(matches!(too_wide.save_webp(&webp), Err(Error::Image(_))));
    assert_eq!(fs::read(&webp).unwrap(), original);
    let unsupported = temporary.0.join("existing.jpg");
    fs::write(&unsupported, &original).unwrap();
    assert!(invalid.save(&unsupported).is_err());
    assert_eq!(fs::read(&unsupported).unwrap(), original);
}

#[test]
fn cli_source_and_background_selection_preserve_inputs_and_outputs() {
    let temporary = TemporaryDirectory::new();
    let input = temporary.0.join("one.pdb");
    let scene = temporary.0.join("scene.json");
    let output = temporary.0.join("image.webp");
    fs::write(&input, PDB).unwrap();
    fs::write(&scene, r#"{"background":[0.4,0.6,0.8]}"#).unwrap();
    for (background, corner) in [
        ("transparent", [0, 0, 0, 0]),
        ("white", [255; 4]),
        ("black", [0, 0, 0, 255]),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_illustrate"))
            .arg(&input)
            .arg("--scene")
            .arg(&scene)
            .arg("--output")
            .arg(&output)
            .args([
                "--background",
                background,
                "--width",
                "64",
                "--height",
                "64",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            image::open(&output).unwrap().to_rgba8().get_pixel(0, 0).0,
            corner
        );
    }
    fs::write(&output, "keep existing output").unwrap();
    for source in [
        vec!["--pdb", "../x"],
        vec![input.to_str().unwrap(), "--pdb", "1lmb"],
        vec![],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_illustrate"))
            .args(source)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!result.stderr.is_empty());
        assert_eq!(fs::read_to_string(&output).unwrap(), "keep existing output");
    }
    assert_eq!(fs::read_to_string(&input).unwrap(), PDB);
}
