use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

use oxillustrate::{Error, SceneSettings};

#[derive(Parser)]
#[command(
    name = "illustrate",
    version,
    about = "Native molecular illustration",
    arg_required_else_help = true
)]
struct Arguments {
    /// Local PDB, mmCIF, or PDBML structure, optionally gzip compressed.
    input: PathBuf,
    /// Destination PNG file.
    #[arg(short, long)]
    output: PathBuf,
    /// JSON scene settings; explicit command-line flags take precedence.
    #[arg(long)]
    scene: Option<PathBuf>,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(long)]
    model: Option<i32>,
    #[arg(long)]
    assembly: Option<String>,
}

pub fn run() -> Result<(), Error> {
    let arguments = Arguments::parse();
    protect_input(&arguments.output, &arguments.input)?;
    if let Some(path) = &arguments.scene {
        protect_input(&arguments.output, path)?;
    }
    let mut settings = match &arguments.scene {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            serde_json::from_str::<SceneSettings>(&text)
                .map_err(|error| Error::Settings(format!("{}: {error}", path.display())))?
        }
        None => SceneSettings::default(),
    };
    if let Some(width) = arguments.width {
        settings.width = width;
    }
    if let Some(height) = arguments.height {
        settings.height = height;
    }
    if let Some(model) = arguments.model {
        settings.model = Some(model);
    }
    if let Some(assembly) = arguments.assembly {
        settings.assembly = Some(assembly);
    }
    settings.validate()?;
    let structure = oxillustrate::load_structure(&arguments.input)?;
    let scene = oxillustrate::prepare_scene(&structure, &settings)?;
    for diagnostic in &scene.diagnostics {
        eprintln!("illustrate: {diagnostic}");
    }
    oxillustrate::render(&scene)?.save_png(&arguments.output)
}

fn protect_input(output: &Path, input: &Path) -> Result<(), Error> {
    let canonical_input = fs::canonicalize(input).map_err(|source| Error::Io {
        path: input.to_owned(),
        source,
    })?;
    match fs::canonicalize(output) {
        Ok(canonical_output) if canonical_output == canonical_input => {
            Err(Error::Settings(format!(
                "output {} would overwrite input {}; choose a different PNG path",
                output.display(),
                input.display()
            )))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::Io {
            path: output.to_owned(),
            source,
        }),
    }
}
