use std::collections::{BTreeMap, BTreeSet};

use crate::assembly;
use crate::{
    Atom, AtomKind, AtomSelector, Error, Namespace, Scene, SceneSettings, Sphere, Structure,
};

type ResidueKey<'a> = (
    i32,
    &'a Option<String>,
    &'a Option<String>,
    &'a Option<String>,
    Option<i32>,
    Option<i32>,
    &'a Option<String>,
    &'a str,
);

fn residue_key(atom: &Atom) -> ResidueKey<'_> {
    let residue = &atom.residue;
    (
        atom.model,
        &residue.author_chain,
        &residue.label_chain,
        &residue.author_sequence,
        residue.author_sequence_number,
        residue.label_sequence,
        &residue.insertion_code,
        &residue.name,
    )
}

/// Choose one alternate per structural residue by mean occupancy, retaining shared blank atoms.
/// Equal means select the lexically smaller identifier, independently of source record order.
fn conformers(structure: &Structure, model: i32) -> Result<Vec<bool>, Error> {
    let mut groups: BTreeMap<ResidueKey<'_>, BTreeMap<&str, (f64, usize)>> = BTreeMap::new();
    for atom in structure.atoms.iter().filter(|atom| atom.model == model) {
        if !atom.occupancy.is_finite() || atom.occupancy < 0.0 {
            return Err(Error::Selection(
                "atom occupancy must be finite and nonnegative".into(),
            ));
        }
        if let Some(alt) = atom.alt_loc.as_deref().filter(|alt| !alt.trim().is_empty()) {
            let entry = groups
                .entry(residue_key(atom))
                .or_default()
                .entry(alt)
                .or_default();
            // Incremental mean avoids overflow when valid occupancies are unusually large.
            entry.1 += 1;
            entry.0 += (atom.occupancy - entry.0) / entry.1 as f64;
        }
    }
    let choices: BTreeMap<_, _> = groups
        .into_iter()
        .map(|(key, alternatives)| {
            let choice = alternatives
                .into_iter()
                .max_by(|(a, (mean_a, _)), (b, (mean_b, _))| {
                    mean_a.total_cmp(mean_b).then_with(|| b.cmp(a))
                })
                .map(|(id, _)| id);
            (key, choice)
        })
        .collect();
    let mut keep = Vec::new();
    keep.try_reserve_exact(structure.atoms.len())
        .map_err(allocation_error)?;
    for atom in &structure.atoms {
        let alt = atom.alt_loc.as_deref().filter(|alt| !alt.trim().is_empty());
        keep.push(
            atom.model == model
                && (alt.is_none() || choices.get(&residue_key(atom)).copied().flatten() == alt),
        );
    }
    Ok(keep)
}

fn matches(selector: &AtomSelector, atom: &Atom) -> bool {
    let residue = &atom.residue;
    selector
        .element
        .as_ref()
        .is_none_or(|element| atom.element.eq_ignore_ascii_case(element))
        && selector.chain.as_ref().is_none_or(|chain| {
            let id = match chain.namespace {
                Namespace::Author => &residue.author_chain,
                Namespace::Label => &residue.label_chain,
            };
            id.as_ref() == Some(&chain.id)
        })
        && selector
            .residue_name
            .as_ref()
            .is_none_or(|name| &residue.name == name)
        && selector
            .atom_name
            .as_ref()
            .is_none_or(|name| &atom.name == name)
        && selector.kind.is_none_or(|kind| atom.kind == kind)
        && selector.residue_range.as_ref().is_none_or(|range| {
            let number = match range.namespace {
                Namespace::Author => residue.author_sequence_number,
                Namespace::Label => residue.label_sequence,
            };
            number.is_some_and(|number| (range.start..=range.end).contains(&number))
        })
}

// Related warm colors preserve chain identity without a rainbow of unrelated hues.
const PALETTE: [[f64; 3]; 6] = [
    [1.0, 0.5, 0.5],
    [1.0, 0.7, 0.5],
    [0.85, 0.5, 0.65],
    [0.85, 0.65, 0.45],
    [0.75, 0.5, 0.5],
    [0.9, 0.6, 0.6],
];

fn radius(element: &str) -> (f64, bool) {
    match element.to_ascii_uppercase().as_str() {
        "C" => (1.6, true),
        "S" | "FE" => (1.8, true),
        "H" | "D" | "N" | "O" | "P" | "F" | "CL" | "BR" | "I" | "NA" | "MG" | "K" | "CA" | "MN"
        | "CO" | "NI" | "CU" | "ZN" | "SE" => (1.5, true),
        _ => (
            1.5,
            concat!(
                "HE LI BE B NE AL SI AR SC TI V CR GA GE AS KR RB SR Y ZR NB MO TC RU RH PD AG CD ",
                "IN SN SB TE XE CS BA LA CE PR ND PM SM EU GD TB DY HO ER TM YB LU HF TA W RE OS ",
                "IR PT AU HG TL PB BI PO AT RN FR RA AC TH PA U NP PU AM CM BK CF ES FM MD NO LR ",
                "RF DB SG BH HS MT DS RG CN NH FL MC LV TS OG"
            )
            .split_ascii_whitespace()
            .any(|known| known.eq_ignore_ascii_case(element)),
        ),
    }
}

fn allocation_error(error: std::collections::TryReserveError) -> Error {
    Error::Selection(format!("cannot allocate prepared geometry: {error}"))
}

fn finite(point: [f64; 3]) -> Result<[f64; 3], Error> {
    if point.iter().all(|value| value.is_finite()) {
        Ok(point)
    } else {
        Err(Error::Selection(
            "atom coordinates and transformed coordinates must be finite".into(),
        ))
    }
}

/// Source-card rotations act Z then Y then X. Negative-angle row-vector source matrices
/// are equivalent to these positive-angle column-vector rotations.
fn rotate(mut point: [f64; 3], orientation: [f64; 3]) -> [f64; 3] {
    for axis in [2, 1, 0] {
        let (sin, cos) = orientation[axis].to_radians().sin_cos();
        let [x, y, z] = point;
        point = match axis {
            2 => [cos * x - sin * y, sin * x + cos * y, z],
            1 => [cos * x + sin * z, y, -sin * x + cos * z],
            _ => [x, cos * y - sin * z, sin * y + cos * z],
        };
    }
    point
}

/// Select, style, replicate, and project visible atoms into source-compatible sphere geometry.
/// Atom rules are first-match overrides. Traversal priority is rule index, source atom, instance.
/// Centers retain the source's continuous 1-based convention and viewport H/2,W/2 offset;
/// the rasterizer truncates center-plus-offset before subtracting one for a buffer index.
///
/// # Errors
/// Returns `Error::Allocation` when image dimensions exceed supported limits,
/// `Error::Settings` for invalid settings, `Error::Selection` when the requested model or
/// assembly is unavailable, coordinates are invalid, or geometry storage cannot be reserved,
/// and `Error::EmptyScene` when filtering leaves no visible atoms.
pub fn prepare_scene(structure: &Structure, settings: &SceneSettings) -> Result<Scene, Error> {
    settings.validate()?;
    for atom in &structure.atoms {
        finite(atom.position)?;
    }
    let model = assembly::select_model(structure, settings.model)?;
    let keep = conformers(structure, model)?;
    let instances = assembly::instances(structure, model, settings.assembly.as_deref())?;
    let mut spheres = Vec::new();
    let mut priorities = Vec::new();
    let mut chains = BTreeMap::new();
    let mut palette_chains = BTreeMap::new();
    let mut residues = BTreeMap::new();
    let mut unknown_elements = BTreeSet::new();
    let mut missing_numbers = false;
    for instance in instances {
        for index in instance.atom_indices {
            if !keep[index] {
                continue;
            }
            let atom = &structure.atoms[index];
            let matching = settings
                .atom_rules
                .iter()
                .enumerate()
                .find(|(_, rule)| matches(&rule.selector, atom));
            let default_omit = atom.kind == AtomKind::Water
                || atom.element.eq_ignore_ascii_case("H")
                || atom.element.eq_ignore_ascii_case("D");
            if matching
                .and_then(|(_, rule)| rule.omit)
                .unwrap_or(default_omit)
            {
                continue;
            }
            let residue = &atom.residue;
            let next_chain = chains.len();
            let chain_id = *chains
                .entry((instance.id, &residue.author_chain, &residue.label_chain))
                .or_insert(next_chain);
            let next_residue = residues.len();
            let residue_id = *residues
                .entry((instance.id, residue_key(atom)))
                .or_insert(next_residue);
            let (base_radius, known) = radius(&atom.element);
            let radius = matching
                .and_then(|(_, rule)| rule.radius)
                .unwrap_or(base_radius);
            if !known && matching.is_none_or(|(_, rule)| rule.radius.is_none()) {
                unknown_elements.insert(atom.element.clone());
            }
            let next_palette = palette_chains.len();
            let palette_index = *palette_chains
                .entry((&residue.author_chain, &residue.label_chain))
                .or_insert(next_palette);
            let mut color = PALETTE[palette_index % PALETTE.len()];
            if atom.element.eq_ignore_ascii_case("C") {
                for component in &mut color {
                    *component += (1.0 - *component) * 0.2;
                }
            }
            let color = matching.and_then(|(_, rule)| rule.color).unwrap_or(color);
            let transformed = std::array::from_fn(|row| {
                instance.transform.translation[row]
                    + (0..3)
                        .map(|column| {
                            instance.transform.rotation[row][column] * atom.position[column]
                        })
                        .sum::<f64>()
            });
            let center = finite(rotate(finite(transformed)?, settings.orientation))?;
            let residue_number = residue.author_sequence_number.or(residue.label_sequence);
            missing_numbers |= residue_number.is_none();
            spheres.try_reserve(1).map_err(allocation_error)?;
            priorities.try_reserve(1).map_err(allocation_error)?;
            spheres.push(Sphere {
                center,
                radius,
                color,
                source_atom_index: index,
                assembly_instance: instance.id,
                residue_id,
                chain_id,
                residue_number,
            });
            priorities.push(matching.map_or(usize::MAX, |(priority, _)| priority));
        }
    }
    if spheres.is_empty() {
        return Err(Error::EmptyScene);
    }
    let mut minimum = [f64::INFINITY; 2];
    let mut maximum = [f64::NEG_INFINITY; 2];
    let mut zmax = f64::NEG_INFINITY;
    let mut rmax: f64 = 0.0;
    for sphere in &spheres {
        for axis in 0..2 {
            minimum[axis] = minimum[axis].min(sphere.center[axis] - sphere.radius);
            maximum[axis] = maximum[axis].max(sphere.center[axis] + sphere.radius);
        }
        zmax = zmax.max(sphere.center[2]);
        rmax = rmax.max(sphere.radius);
    }
    let center = finite(settings.framing.center.unwrap_or([
        minimum[0] / 2.0 + maximum[0] / 2.0,
        minimum[1] / 2.0 + maximum[1] / 2.0,
        zmax + rmax + 1.0,
    ]))?;
    let mut extent = [0.0_f64; 2];
    for sphere in &mut spheres {
        sphere.center = finite(std::array::from_fn(|axis| {
            sphere.center[axis] - center[axis] + settings.translation[axis]
        }))?;
        for (axis, value) in extent.iter_mut().enumerate() {
            *value = value.max(sphere.center[axis].abs() + sphere.radius);
        }
    }
    let scale = settings.framing.scale.unwrap_or_else(|| {
        ((f64::from(settings.height) / 2.0 - settings.framing.margin) / extent[0])
            .min((f64::from(settings.width) / 2.0 - settings.framing.margin) / extent[1])
    });
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::Settings(
            "fitted scale must be finite and positive".into(),
        ));
    }
    for sphere in &mut spheres {
        sphere.center = finite([
            sphere.center[0] * scale + f64::from(settings.height) / 2.0,
            sphere.center[1] * scale + f64::from(settings.width) / 2.0,
            sphere.center[2] * scale,
        ])?;
        sphere.radius *= scale;
        if !sphere.radius.is_finite() || sphere.radius <= 0.0 {
            return Err(Error::Settings(
                "projected radius must be finite and positive".into(),
            ));
        }
    }
    // Source priority is retained without making source identities depend on rule order.
    let mut ordered = Vec::new();
    ordered
        .try_reserve_exact(spheres.len())
        .map_err(allocation_error)?;
    ordered.extend(priorities.into_iter().zip(spheres));
    ordered.sort_unstable_by_key(|(priority, sphere)| {
        (
            *priority,
            sphere.source_atom_index,
            sphere.assembly_instance,
        )
    });
    let mut spheres = Vec::new();
    spheres
        .try_reserve_exact(ordered.len())
        .map_err(allocation_error)?;
    spheres.extend(ordered.into_iter().map(|(_, sphere)| sphere));
    let mut diagnostics = structure.diagnostics.clone();
    for element in unknown_elements {
        diagnostics.push(format!(
            "unknown element {element:?}: using radius 1.5 Angstrom"
        ));
    }
    if missing_numbers {
        diagnostics.push("some residues have no numeric author or label sequence; residue-distance outlines skip those boundaries".into());
    }
    Ok(Scene {
        width: settings.width,
        height: settings.height,
        spheres,
        background: settings.background,
        fog: settings.fog.clone(),
        shadows: settings.shadows.clone(),
        outlines: settings.outlines.clone(),
        pixels_per_angstrom: scale,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Assembly, AssemblyGenerator, AtomRule, ResidueId, Transform};

    fn atom(position: [f64; 3], element: &str) -> Atom {
        Atom {
            source_id: "source".into(),
            position,
            element: element.into(),
            name: "CA".into(),
            author_name: None,
            label_name: None,
            residue: ResidueId {
                author_chain: Some("alpha-long".into()),
                label_chain: Some("L1".into()),
                author_sequence: Some("10".into()),
                author_sequence_number: Some(10),
                label_sequence: Some(1),
                insertion_code: None,
                name: "ALA".into(),
                author_name: None,
                label_name: None,
            },
            model: 1,
            occupancy: 1.0,
            alt_loc: None,
            kind: AtomKind::Polymer,
        }
    }

    fn structure(atoms: Vec<Atom>) -> Structure {
        Structure {
            atoms,
            ..Structure::default()
        }
    }

    #[test]
    fn alternatives_use_mean_occupancy_and_lexical_ties_and_keep_shared_atoms() {
        let mut atoms = vec![atom([0.0; 3], "C"); 6];
        for (i, (alt, occupancy)) in [
            (None, 1.0),
            (Some("B"), 0.8),
            (Some("B"), 0.2),
            (Some("A"), 0.6),
            (Some("C"), 0.6),
            (None, 1.0),
        ]
        .into_iter()
        .enumerate()
        {
            atoms[i].alt_loc = alt.map(str::to_owned);
            atoms[i].occupancy = occupancy;
        }
        atoms[5].model = 2;
        assert_eq!(
            conformers(&structure(atoms), 1).unwrap(),
            [true, false, false, true, false, false]
        );
    }

    #[test]
    fn defaults_rules_and_namespace_conditions_preserve_identity() {
        let mut atoms = vec![
            atom([0.0; 3], "H"),
            atom([0.0; 3], "O"),
            atom([0.0; 3], "XX"),
            atom([0.0; 3], "C"),
        ];
        atoms[1].kind = AtomKind::Water;
        atoms[2].kind = AtomKind::Hetero;
        atoms[3].residue.author_sequence = Some("10A".into());
        atoms[3].residue.author_sequence_number = None;
        atoms[3].residue.label_sequence = None;
        atoms[3].residue.insertion_code = Some("A".into());
        let source = structure(atoms);
        let default_scene = prepare_scene(&source, &SceneSettings::default()).unwrap();
        assert_eq!(default_scene.spheres.len(), 2);
        assert_eq!(default_scene.spheres[0].residue_number, Some(10));
        assert_eq!(default_scene.spheres[1].residue_number, None);
        assert_ne!(
            default_scene.spheres[0].residue_id,
            default_scene.spheres[1].residue_id
        );
        assert_eq!(default_scene.diagnostics.len(), 2);
        let settings: SceneSettings = serde_json::from_str(r#"{"atom_rules":[{"selector":{"element":"H","chain":{"namespace":"label","id":"L1"},"residue_range":{"start":10,"end":10},"residue_name":"ALA","atom_name":"CA","kind":"polymer"},"omit":false,"color":[0,0,1],"radius":2},{"selector":{"element":"H"},"omit":true}]}"#).unwrap();
        let scene = prepare_scene(&source, &settings).unwrap();
        assert_eq!(scene.spheres.len(), 3);
        assert_eq!(scene.spheres[0].source_atom_index, 0);
        assert_eq!(scene.spheres[0].color, [0.0, 0.0, 1.0]);
        assert!((scene.spheres[0].radius / scene.pixels_per_angstrom - 2.0).abs() < 1e-12);
    }

    #[test]
    fn sphere_extents_fit_rectangular_viewport_and_burial_uses_physical_radius() {
        let source = structure(vec![
            atom([-10.0, 0.0, 3.0], "C"),
            atom([10.0, 4.0, 1.0], "S"),
        ]);
        let settings = SceneSettings {
            width: 160,
            height: 100,
            ..SceneSettings::default()
        };
        let scene = prepare_scene(&source, &settings).unwrap();
        for sphere in &scene.spheres {
            assert!(sphere.center[0] - sphere.radius >= settings.framing.margin - 1e-10);
            assert!(
                sphere.center[0] + sphere.radius
                    <= f64::from(settings.height) - settings.framing.margin + 1e-10
            );
            assert!(sphere.center[1] - sphere.radius >= settings.framing.margin - 1e-10);
            assert!(
                sphere.center[1] + sphere.radius
                    <= f64::from(settings.width) - settings.framing.margin + 1e-10
            );
        }
        assert!((scene.spheres[0].center[2] / scene.pixels_per_angstrom + 2.8).abs() < 1e-12);
        assert!((scene.spheres[0].radius / scene.pixels_per_angstrom - 1.6).abs() < 1e-12);
    }

    #[test]
    fn model_assembly_rotation_center_translation_scale_order_is_explicit() {
        let mut second_model = atom([1000.0; 3], "C");
        second_model.model = 2;
        let mut source = structure(vec![atom([1.0, 0.0, 0.0], "C"), second_model]);
        source.operations.insert(
            "copy".into(),
            Transform {
                translation: [0.0, 2.0, 0.0],
                ..Transform::default()
            },
        );
        source.assemblies.push(Assembly {
            id: "bio".into(),
            generators: vec![AssemblyGenerator {
                chains: vec!["L1".into()],
                chain_namespace: Namespace::Label,
                operation_expression: "copy".into(),
            }],
        });
        let mut settings = SceneSettings {
            width: 100,
            height: 80,
            assembly: Some("bio".into()),
            orientation: [90.0, 0.0, 90.0],
            translation: [1.0, 2.0, -1.0],
            ..SceneSettings::default()
        };
        settings.framing.center = Some([1.0, 1.0, 3.0]);
        settings.framing.scale = Some(2.0);
        let scene = prepare_scene(&source, &settings).unwrap();
        let center = scene.spheres[0].center;
        for (actual, expected) in center.into_iter().zip([36.0, 52.0, -6.0]) {
            assert!((actual - expected).abs() < 1e-10);
        }
        settings.model = Some(3);
        assert!(prepare_scene(&source, &settings).is_err());
    }

    #[test]
    fn invalid_geometry_empty_selection_and_rule_order_are_checked() {
        let mut source = structure(vec![atom([0.0; 3], "H")]);
        assert!(matches!(
            prepare_scene(&source, &SceneSettings::default()),
            Err(Error::EmptyScene)
        ));
        source.atoms[0].element = "C".into();
        source.atoms[0].position[0] = f64::NAN;
        assert!(prepare_scene(&source, &SceneSettings::default()).is_err());
        source.atoms[0].position = [0.0; 3];
        source.atoms.push(atom([0.0; 3], "S"));
        let settings = SceneSettings {
            atom_rules: vec![AtomRule {
                selector: AtomSelector {
                    element: Some("S".into()),
                    ..AtomSelector::default()
                },
                color: Some([0.0, 0.0, 1.0]),
                ..AtomRule::default()
            }],
            ..SceneSettings::default()
        };
        let scene = prepare_scene(&source, &settings).unwrap();
        assert_eq!(
            scene
                .spheres
                .iter()
                .map(|s| s.source_atom_index)
                .collect::<Vec<_>>(),
            [1, 0]
        );
    }
}
