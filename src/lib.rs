//! Native, synchronous molecular illustration from local structure coordinates.
//!
//! Load a structure, prepare its visible spheres, and save the rendered RGB image:
//!
//! ```no_run
//! use oxillustrate::{SceneSettings, load_structure, prepare_scene, render};
//!
//! fn main() -> Result<(), oxillustrate::Error> {
//!     let structure = load_structure("structure.cif")?;
//!     let scene = prepare_scene(&structure, &SceneSettings::default())?;
//!     render(&scene)?.save_png("structure.png")?;
//!     Ok(())
//! }
//! ```
//!
//! Input supports PDB, PDBx/mmCIF, and PDBML XML, optionally gzip compressed.
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
pub use render::render;
pub use rendered_image::RenderedImage;
pub use scene::{Scene, Sphere};
pub use scene_preparation::prepare_scene;
pub use settings::{
    AtomRule, AtomSelector, ChainSelector, FogSettings, Framing, MAX_PIXELS, OutlineSettings,
    ResidueRange, SceneSettings, ShadowSettings, Thresholds,
};
