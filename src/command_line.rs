use clap::{Parser, ValueEnum};
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
    #[arg(required_unless_present = "pdb", conflicts_with = "pdb")]
    input: Option<PathBuf>,
    /// Download an RCSB entry by PDB ID, for example 1lmb.
    #[arg(long, value_name = "ID", required_unless_present = "input")]
    pdb: Option<String>,
    /// Destination .png or .webp file.
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
    /// Override scene background; transparent is the default without a scene color.
    #[arg(long, value_enum)]
    background: Option<Background>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Background {
    Transparent,
    White,
    Black,
}

pub fn run() -> Result<(), Error> {
    let arguments = Arguments::parse();
    if let Some(input) = &arguments.input {
        protect_input(&arguments.output, input)?;
    }
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
    if let Some(background) = arguments.background {
        settings.background = match background {
            Background::Transparent => None,
            Background::White => Some([1.0; 3]),
            Background::Black => Some([0.0; 3]),
        };
    }
    settings.validate()?;
    let structure = match (&arguments.input, &arguments.pdb) {
        (Some(input), None) => oxillustrate::load_structure(input)?,
        (None, Some(id)) => oxillustrate::load_pdb(id)?,
        _ => return Err(Error::Selection("provide a local input or --pdb ID".into())),
    };
    let scene = oxillustrate::prepare_scene(&structure, &settings)?;
    for diagnostic in &scene.diagnostics {
        eprintln!("illustrate: {diagnostic}");
    }
    oxillustrate::render(&scene)?.save(&arguments.output)
}

fn protect_input(output: &Path, input: &Path) -> Result<(), Error> {
    let canonical_input = fs::canonicalize(input).map_err(|source| Error::Io {
        path: input.to_owned(),
        source,
    })?;
    match fs::canonicalize(output) {
        Ok(canonical_output) if canonical_output == canonical_input => {
            Err(Error::Settings(format!(
                "output {} would overwrite input {}; choose a different image path",
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
