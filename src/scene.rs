use crate::{FogSettings, OutlineSettings, ShadowSettings};

#[derive(Clone, Debug, PartialEq)]
/// One visible atom instance, independent of its source identity.
pub struct Sphere {
    /// Pixel coordinates: row down, column right, z toward viewer; includes viewport offset.
    pub center: [f64; 3],
    pub radius: f64,
    pub color: [f64; 3],
    pub source_atom_index: usize,
    pub assembly_instance: usize,
    pub residue_id: usize,
    pub chain_id: usize,
    pub residue_number: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
/// Prepared orthographic geometry and appearance for synchronous rendering.
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub spheres: Vec<Sphere>,
    pub background: [f64; 3],
    pub fog: FogSettings,
    pub shadows: ShadowSettings,
    pub outlines: OutlineSettings,
    pub pixels_per_angstrom: f64,
    pub diagnostics: Vec<String>,
}
