//! Native, synchronous molecular illustration from local coordinates or an RCSB entry.
//!
//! Load a structure, prepare its visible spheres, and save its transparent image:
//!
//! ```no_run
//! use oxillustrate::{SceneSettings, load_structure, prepare_scene, render};
//!
//! fn main() -> Result<(), oxillustrate::Error> {
//!     let structure = load_structure("structure.cif")?;
//!     let scene = prepare_scene(&structure, &SceneSettings::default())?;
//!     render(&scene)?.save("structure.webp")?;
//!     Ok(())
//! }
//! ```
//!
//! Input supports PDB, PDBx/mmCIF, and PDBML XML, optionally gzip compressed.
//! [`load_pdb`] downloads mmCIF coordinates from RCSB by accession without saving an input file.
//! PNG and lossless WebP preserve alpha; an explicit background color selects opaque output.
//! Images default to 300 dpi with three pixels of padding after content trimming.
//! [`SceneSettings`] supplies validated defaults and JSON-deserializable appearance controls.
//! [`Scene`] owns its prepared spheres; rendering requires no further file reads.
#![forbid(unsafe_code)]

mod assembly;
mod assembly_metadata;
mod atom_table;
mod cif_input;
mod cif_tokens;
mod error;
mod input;
mod molecular;
mod outlines;
mod pdb_download;
mod pdb_input;
mod raster;
mod render;
mod rendered_image;
mod scene;
mod scene_preparation;
mod settings;
mod shading;
mod xml_input;

pub use error::Error;
pub use input::load_structure;
pub use molecular::{
    Assembly, AssemblyGenerator, Atom, AtomKind, Namespace, ResidueId, Structure, Transform,
};
pub use pdb_download::load_pdb;
pub use render::{render, render_with_bounds};
pub use rendered_image::RenderedImage;
pub use scene::{Scene, Sphere};
pub use scene_preparation::prepare_scene;
pub use settings::{
    AtomRule, AtomSelector, ChainSelector, FogSettings, Framing, MAX_PIXELS, OutlineSettings,
    ResidueRange, SceneSettings, ShadowSettings, Thresholds,
};
