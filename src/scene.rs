use crate::{FogSettings, OutlineSettings, ShadowSettings};

#[derive(Clone, Debug, PartialEq)]
/// One visible atom instance, independent of its source identity.
pub struct Sphere {
    /// Pixel coordinates: row down, column right, z toward viewer; includes viewport offset.
    pub center: [f64; 3],
    /// Radius in pixels.
    pub radius: f64,
    /// Surface RGB color, with components in the range 0 to 1.
    pub color: [f64; 3],
    /// Index into the source `Structure::atoms` vector.
    pub source_atom_index: usize,
    /// Dense identity of this assembly copy within the prepared scene.
    pub assembly_instance: usize,
    /// Dense residue identity used by the residue-outline pass.
    pub residue_id: usize,
    /// Dense chain identity used by the subunit-outline pass.
    pub chain_id: usize,
    /// Numeric author residue number, falling back to label numbering.
    pub residue_number: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
/// Prepared orthographic geometry and appearance for synchronous rendering.
pub struct Scene {
    /// Raster width in pixels.
    pub width: u32,
    /// Raster height in pixels.
    pub height: u32,
    /// Visible atom instances as projected spheres.
    pub spheres: Vec<Sphere>,
    /// Opaque background RGB components, or None for a transparent background.
    pub background: Option<[f64; 3]>,
    /// Depth-dependent color mixing settings.
    pub fog: FogSettings,
    /// Screen-space shadow settings.
    pub shadows: ShadowSettings,
    /// Outline pass settings.
    pub outlines: OutlineSettings,
    /// Projection scale in pixels per Angstrom.
    pub pixels_per_angstrom: f64,
    /// Non-fatal scene-preparation diagnostics.
    pub diagnostics: Vec<String>,
}
