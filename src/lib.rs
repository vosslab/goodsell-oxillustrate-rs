//! Owned molecular data and settings for the synchronous Illustrate renderer.
#![forbid(unsafe_code)]

mod assembly;
mod assembly_metadata;
mod atom_table;
mod cif_input;
mod cif_tokens;
mod error;
mod input;
mod molecular;
mod pdb_input;
mod rendered_image;
mod scene;
mod settings;
mod xml_input;

pub use error::Error;
pub use input::load_structure;
pub use molecular::{
    Assembly, AssemblyGenerator, Atom, AtomKind, Namespace, ResidueId, Structure, Transform,
};
pub use rendered_image::RenderedImage;
pub use scene::{Scene, Sphere};
pub use settings::{
    AtomRule, AtomSelector, ChainSelector, FogSettings, Framing, MAX_PIXELS, OutlineSettings,
    ResidueRange, SceneSettings, ShadowSettings, Thresholds,
};
