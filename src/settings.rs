use serde::{Deserialize, Serialize};

use crate::{AtomKind, Error, Namespace};

/// Bounds raster scratch storage; additional image-buffer allocations remain fallible.
pub const MAX_PIXELS: usize = 16_777_216;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// JSON-deserializable scene configuration; call validate before preparing a scene.
pub struct SceneSettings {
    /// Render canvas width in pixels, before content trimming.
    pub width: u32,
    /// Render canvas height in pixels, before content trimming.
    pub height: u32,
    /// Print resolution in pixels per inch; does not resample the raster.
    pub dpi: u32,
    /// Deposited model number; absent selects the first available model.
    pub model: Option<i32>,
    /// Biological assembly ID; absent uses deposited coordinates.
    pub assembly: Option<String>,
    /// Source-compatible x/y/z cards in degrees, applied Z then Y then X.
    pub orientation: [f64; 3],
    /// Translation in source coordinates, in Angstroms.
    pub translation: [f64; 3],
    /// Centering and orthographic scale controls.
    pub framing: Framing,
    /// Background RGB components in the range 0 to 1; None (JSON null) is transparent.
    pub background: Option<[f64; 3]>,
    /// Ordered appearance rules; the first matching rule wins.
    pub atom_rules: Vec<AtomRule>,
    /// Molecular color weights and color used for depth fog.
    pub fog: FogSettings,
    /// Screen-space shadow parameters.
    pub shadows: ShadowSettings,
    /// Independent contour, subunit, and residue outline controls.
    pub outlines: OutlineSettings,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            dpi: 300,
            model: None,
            assembly: None,
            orientation: [0.0; 3],
            translation: [0.0; 3],
            framing: Framing::default(),
            background: None,
            atom_rules: Vec::new(),
            fog: FogSettings::default(),
            shadows: ShadowSettings::default(),
            outlines: OutlineSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// Orthographic camera framing after source-compatible rotation.
pub struct Framing {
    /// Explicit rotated-coordinate center replaces automatic centering and z burial.
    pub center: Option<[f64; 3]>,
    /// Pixels per Angstrom; None fits visible sphere bounds into the image.
    pub scale: Option<f64>,
    /// Padding in pixels on each side of the automatically fitted image.
    pub margin: f64,
}

impl Default for Framing {
    fn default() -> Self {
        Self {
            center: None,
            scale: None,
            margin: 30.0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// First-match appearance overrides; omitted properties keep native defaults.
pub struct AtomRule {
    /// Conditions used to select atoms for this override.
    pub selector: AtomSelector,
    /// Replacement RGB color in the inclusive range 0 to 1.
    pub color: Option<[f64; 3]>,
    /// Replacement sphere radius in Angstroms; must be positive.
    pub radius: Option<f64>,
    /// Whether to hide a match; false restores atoms hidden by native defaults.
    pub omit: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// All supplied conditions must match one atom.
pub struct AtomSelector {
    /// Case-insensitive element symbol.
    pub element: Option<String>,
    /// Exact chain ID and namespace.
    pub chain: Option<ChainSelector>,
    /// Effective residue name.
    pub residue_name: Option<String>,
    /// Inclusive numeric residue interval.
    pub residue_range: Option<ResidueRange>,
    /// Effective atom name.
    pub atom_name: Option<String>,
    /// Source atom category.
    pub kind: Option<AtomKind>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact chain identifier in an explicitly chosen namespace.
pub struct ChainSelector {
    /// Namespace used to interpret the chain identifier; defaults to author.
    #[serde(default)]
    pub namespace: Namespace,
    /// Exact chain identifier; no trimming or wildcard matching is applied.
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Inclusive numeric residue bounds in the chosen namespace.
pub struct ResidueRange {
    /// Namespace used for numeric residue matching; defaults to author.
    #[serde(default)]
    pub namespace: Namespace,
    /// Inclusive lower numeric residue bound.
    pub start: i32,
    /// Inclusive upper numeric residue bound.
    pub end: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// Depth cueing endpoints expressed as molecular color weights.
pub struct FogSettings {
    /// Fog RGB color in the inclusive range 0 to 1.
    pub color: [f64; 3],
    /// Molecular color weights: one means no fog.
    pub front: f64,
    /// Molecular color weight at the far endpoint; one means no fog.
    pub back: f64,
}

impl Default for FogSettings {
    fn default() -> Self {
        Self {
            color: [1.0; 3],
            front: 1.0,
            back: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// Fixed-neighborhood conical screen-space shadow parameters.
pub struct ShadowSettings {
    /// Whether to apply the shadow pass.
    pub enabled: bool,
    /// Brightness decrement per shadow contribution.
    pub decrement: f64,
    /// Distance-dependent shadow attenuation slope.
    pub slope: f64,
    /// Depth threshold in pixels, matching the source arithmetic.
    pub threshold: f64,
    /// Minimum resulting brightness in the inclusive range 0 to 1.
    pub brightness_floor: f64,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            decrement: 0.0023,
            slope: 2.0,
            threshold: 1.0,
            brightness_floor: 0.2,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// A linear opacity ramp whose high endpoint must exceed low.
pub struct Thresholds {
    /// Lower edge threshold, where the opacity ramp begins.
    pub low: f64,
    /// Upper edge threshold, where the opacity ramp reaches one.
    pub high: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
/// Independent contour, subunit, and residue outline passes.
pub struct OutlineSettings {
    /// Enable the silhouette contour pass.
    pub contour_enabled: bool,
    /// Enable boundaries between assembly subunits.
    pub subunit_enabled: bool,
    /// Enable boundaries between residues.
    pub residue_enabled: bool,
    /// Opacity thresholds for the contour pass.
    pub contour: Thresholds,
    /// Opacity thresholds for the subunit pass.
    pub subunit: Thresholds,
    /// Opacity thresholds for the residue pass.
    pub residue: Thresholds,
    /// Neighborhood kernel width, from one through four.
    pub kernel: u8,
    /// Contour depth endpoints in Angstroms.
    pub depth_min: f64,
    /// Maximum depth endpoint in Angstroms; must exceed `depth_min`.
    pub depth_max: f64,
    /// Residue-number difference at which the residue pass draws an edge.
    pub residue_difference: f64,
}

impl Default for OutlineSettings {
    fn default() -> Self {
        Self {
            contour_enabled: true,
            subunit_enabled: true,
            residue_enabled: true,
            contour: Thresholds {
                low: 3.0,
                high: 10.0,
            },
            subunit: Thresholds {
                low: 3.0,
                high: 10.0,
            },
            residue: Thresholds {
                low: 3.0,
                high: 8.0,
            },
            kernel: 4,
            depth_min: 0.0,
            depth_max: 5.0,
            residue_difference: 6000.0,
        }
    }
}

impl SceneSettings {
    /// Return the checked image size within the documented scratch budget.
    ///
    /// # Errors
    /// Returns `Error::Allocation` for zero dimensions, multiplication overflow, or an
    /// image exceeding [`MAX_PIXELS`].
    pub fn pixel_count(&self) -> Result<usize, Error> {
        let count = (self.width as usize).checked_mul(self.height as usize);
        match count {
            Some(count) if self.width > 0 && self.height > 0 && count <= MAX_PIXELS => Ok(count),
            _ => Err(Error::Allocation {
                width: self.width,
                height: self.height,
            }),
        }
    }

    /// Reject invalid numeric settings before geometry or image allocation.
    ///
    /// # Errors
    /// Returns `Error::Allocation` for invalid dimensions and `Error::Settings` for
    /// non-finite or out-of-range values, invalid thresholds, or malformed rules.
    pub fn validate(&self) -> Result<(), Error> {
        self.pixel_count()?;
        pixels_per_meter(self.dpi)?;
        if self.model.is_some_and(|model| model <= 0) {
            return invalid("model must be positive");
        }
        if self
            .assembly
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return invalid("assembly ID must not be empty");
        }
        finite("orientation", &self.orientation)?;
        finite("translation", &self.translation)?;
        if let Some(center) = self.framing.center {
            finite("framing center", &center)?;
        }
        if self
            .framing
            .scale
            .is_some_and(|scale| !scale.is_finite() || scale <= 0.0)
        {
            return invalid("framing scale must be finite and positive");
        }
        nonnegative("framing margin", self.framing.margin)?;
        if self.framing.scale.is_none()
            && self.framing.margin * 2.0 >= f64::from(self.width.min(self.height))
        {
            return invalid("framing margin must leave room for the molecule");
        }
        if let Some(background) = self.background {
            color("background", &background)?;
        }
        color("fog color", &self.fog.color)?;
        unit("fog front", self.fog.front)?;
        unit("fog back", self.fog.back)?;
        unit("shadow decrement", self.shadows.decrement)?;
        unit("shadow brightness floor", self.shadows.brightness_floor)?;
        nonnegative("shadow slope", self.shadows.slope)?;
        nonnegative("shadow threshold", self.shadows.threshold)?;
        for (name, pair) in [
            ("contour", &self.outlines.contour),
            ("subunit", &self.outlines.subunit),
            ("residue", &self.outlines.residue),
        ] {
            nonnegative(name, pair.low)?;
            if !pair.high.is_finite() || pair.high <= pair.low {
                return invalid(&format!("{name} high threshold must exceed low threshold"));
            }
        }
        if !(1..=4).contains(&self.outlines.kernel) {
            return invalid("outline kernel must be 1, 2, 3, or 4");
        }
        nonnegative("outline depth minimum", self.outlines.depth_min)?;
        if !self.outlines.depth_max.is_finite()
            || self.outlines.depth_max <= self.outlines.depth_min
        {
            return invalid("outline depth maximum must exceed minimum");
        }
        nonnegative("residue difference", self.outlines.residue_difference)?;
        for rule in &self.atom_rules {
            if let Some(rgb) = rule.color {
                color("rule color", &rgb)?;
            }
            if rule
                .radius
                .is_some_and(|radius| !radius.is_finite() || radius <= 0.0)
            {
                return invalid("rule radius must be finite and positive; use omit to hide atoms");
            }
            if rule
                .selector
                .residue_range
                .as_ref()
                .is_some_and(|range| range.start > range.end)
            {
                return invalid("residue range start must not exceed end");
            }
        }
        Ok(())
    }
}

pub(crate) fn pixels_per_meter(dpi: u32) -> Result<u32, Error> {
    let density = (f64::from(dpi) / 0.0254).round();
    if dpi == 0 || density > f64::from(u32::MAX) {
        return invalid("dpi must be positive and fit PNG physical-resolution metadata");
    }
    Ok(density as u32)
}

fn invalid<T>(message: &str) -> Result<T, Error> {
    Err(Error::Settings(message.to_owned()))
}

fn finite(name: &str, values: &[f64]) -> Result<(), Error> {
    if values.iter().any(|value| !value.is_finite()) {
        return invalid(&format!("{name} must be finite"));
    }
    Ok(())
}

fn unit(name: &str, value: f64) -> Result<(), Error> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return invalid(&format!("{name} must be between zero and one"));
    }
    Ok(())
}

fn color(name: &str, values: &[f64; 3]) -> Result<(), Error> {
    for value in values {
        unit(name, *value)?;
    }
    Ok(())
}

fn nonnegative(name: &str, value: f64) -> Result<(), Error> {
    if !value.is_finite() || value < 0.0 {
        return invalid(&format!("{name} must be finite and nonnegative"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_partial_json_are_valid() {
        let settings: SceneSettings = serde_json::from_str(r#"{"width":640,"fog":{"back":0.4},"atom_rules":[{"selector":{"chain":{"id":"long_chain"}},"omit":false}]}"#).unwrap();
        settings.validate().unwrap();
        assert_eq!(settings.pixel_count().unwrap(), 640 * 800);
        assert_eq!(
            settings.atom_rules[0]
                .selector
                .chain
                .as_ref()
                .unwrap()
                .namespace,
            Namespace::Author
        );
        assert_eq!(settings.fog.front, 1.0);
        assert_eq!(settings.background, None);
        let opaque: SceneSettings = serde_json::from_str(r#"{"background":[1,1,1]}"#).unwrap();
        assert_eq!(opaque.background, Some([1.0; 3]));
        let transparent: SceneSettings = serde_json::from_str(r#"{"background":null}"#).unwrap();
        assert_eq!(transparent.background, None);
    }

    #[test]
    fn invalid_values_and_unknown_fields_are_rejected() {
        assert!(serde_json::from_str::<SceneSettings>(r#"{"widht":10}"#).is_err());
        let mut settings = SceneSettings::default();
        settings.orientation[0] = f64::NAN;
        assert!(matches!(settings.validate(), Err(Error::Settings(_))));
        settings.orientation[0] = 0.0;
        settings.outlines.contour.high = settings.outlines.contour.low;
        assert!(settings.validate().is_err());
        settings.outlines = OutlineSettings::default();
        settings.width = u32::MAX;
        assert!(matches!(settings.validate(), Err(Error::Allocation { .. })));
    }

    #[test]
    fn explicit_scale_allows_small_frames_but_rules_require_valid_appearance() {
        let mut settings: SceneSettings = serde_json::from_str(
            r#"{"width":20,"height":20,"framing":{"scale":2},"atom_rules":[{"radius":0}]}"#,
        )
        .unwrap();
        assert!(settings.validate().is_err());
        settings.atom_rules[0].radius = Some(1.6);
        settings.validate().unwrap();
        settings.atom_rules[0].color = Some([1.1, 0.0, 0.0]);
        assert!(settings.validate().is_err());
    }
}
