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
            pixels: vec![0; 3]
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
            pixels: vec![0; 3]
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
