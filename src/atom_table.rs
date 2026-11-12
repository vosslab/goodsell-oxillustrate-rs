//! Lossless normalization shared by PDBx/mmCIF and PDBML adapters.
use std::collections::BTreeMap;

use crate::error::Error;
use crate::molecular::{Atom, AtomKind, ResidueId, Structure};

pub(crate) type Categories = BTreeMap<String, Vec<BTreeMap<String, String>>>;

pub(crate) fn value<'a>(row: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    row.get(key)
        .map(String::as_str)
        .filter(|v| !matches!(*v, "" | "." | "?"))
}

pub(crate) fn from_categories(
    categories: &Categories,
    diagnostics: Vec<String>,
) -> Result<Structure, Error> {
    let mut structure = Structure {
        diagnostics,
        ..Structure::default()
    };
    let rows = categories.get("atom_site").filter(|r| !r.is_empty()).ok_or_else(|| Error::Parse {
        format: "PDBx", message: "no atom_site coordinates; provide macromolecular PDBx/mmCIF or coordinate-bearing PDBML, not small-molecule CIF".into(),
    })?;
    for (index, row) in rows.iter().enumerate() {
        let context = format!(
            "atom_site row {} (id {})",
            index + 1,
            value(row, "id").unwrap_or("missing")
        );
        let invalid = |message: String| Error::Parse {
            format: "PDBx",
            message: format!("{context}: {message}"),
        };
        let number = |key: &str, default: Option<f64>| -> Result<f64, Error> {
            let n = match value(row, key) {
                Some(text) => text
                    .parse::<f64>()
                    .map_err(|_| invalid(format!("{key} must be a number, got {text:?}")))?,
                None => default.ok_or_else(|| invalid(format!("missing required {key}")))?,
            };
            if !n.is_finite() {
                return Err(invalid(format!("{key} must be finite")));
            }
            Ok(n)
        };
        let text = |key: &str| value(row, key).map(str::to_owned);
        let author_name = text("auth_atom_id");
        let label_name = text("label_atom_id");
        let name = author_name
            .clone()
            .or_else(|| label_name.clone())
            .ok_or_else(|| invalid("missing auth_atom_id and label_atom_id".into()))?;
        let author_residue = text("auth_comp_id");
        let label_residue = text("label_comp_id");
        let residue_name = author_residue
            .clone()
            .or_else(|| label_residue.clone())
            .ok_or_else(|| invalid("missing auth_comp_id and label_comp_id".into()))?;
        let occupancy = number("occupancy", Some(1.0))?;
        if occupancy < 0.0 {
            return Err(invalid("occupancy must be nonnegative".into()));
        }
        let model = value(row, "pdbx_PDB_model_num")
            .unwrap_or("1")
            .parse::<i32>()
            .map_err(|_| invalid("pdbx_PDB_model_num must be an integer".into()))?;
        if model < 1 {
            return Err(invalid("pdbx_PDB_model_num must be positive".into()));
        }
        if let Some(group) = value(row, "group_PDB") {
            if !matches!(group, "ATOM" | "HETATM") {
                return Err(invalid(format!(
                    "group_PDB must be ATOM or HETATM, got {group:?}"
                )));
            }
        }
        let label_sequence = value(row, "label_seq_id")
            .map(|v| {
                v.parse::<i32>()
                    .map_err(|_| invalid(format!("label_seq_id must be an integer, got {v:?}")))
            })
            .transpose()?;
        let author_sequence = text("auth_seq_id");
        let mut element = text("type_symbol").unwrap_or_default().to_ascii_uppercase();
        if element.is_empty() {
            // Atom names usually encode the element first; ambiguous two-letter names use carbon/nitrogen/etc.
            element = name
                .chars()
                .find(char::is_ascii_alphabetic)
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "C".into());
            structure.diagnostics.push(format!(
                "{context}: missing element; inferred {element} from atom name {name:?}"
            ));
        }
        if element.len() > 2 || !element.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(invalid(format!(
                "type_symbol must be an element symbol, got {element:?}"
            )));
        }
        let entity_id = value(row, "label_entity_id");
        let polymer = label_sequence.is_some()
            || categories.get("entity_poly").is_some_and(|rows| {
                rows.iter()
                    .any(|r| value(r, "entity_id") == entity_id && entity_id.is_some())
            })
            || categories.get("entity").is_some_and(|rows| {
                rows.iter().any(|r| {
                    value(r, "id") == entity_id
                        && entity_id.is_some()
                        && value(r, "type") == Some("polymer")
                })
            });
        let kind = if matches!(residue_name.as_str(), "HOH" | "DOD" | "WAT") {
            AtomKind::Water
        } else if value(row, "group_PDB") == Some("ATOM") || polymer {
            AtomKind::Polymer
        } else {
            AtomKind::Hetero
        };
        if value(row, "group_PDB").is_none() && !polymer && kind != AtomKind::Water {
            structure.diagnostics.push(format!(
                "{context}: missing group_PDB and polymer metadata; classified as hetero"
            ));
        }
        structure.atoms.push(Atom {
            source_id: text("id").unwrap_or_else(|| (index + 1).to_string()),
            position: [
                number("Cartn_x", None)?,
                number("Cartn_y", None)?,
                number("Cartn_z", None)?,
            ],
            element,
            name,
            author_name,
            label_name,
            residue: ResidueId {
                author_chain: text("auth_asym_id"),
                label_chain: text("label_asym_id"),
                author_sequence_number: author_sequence.as_deref().and_then(|v| v.parse().ok()),
                author_sequence,
                label_sequence,
                insertion_code: text("pdbx_PDB_ins_code"),
                name: residue_name,
                author_name: author_residue,
                label_name: label_residue,
            },
            model,
            occupancy,
            alt_loc: text("label_alt_id").or_else(|| text("auth_alt_id")),
            kind,
        });
    }
    crate::assembly_metadata::apply_categories(categories, &mut structure)?;
    Ok(structure)
}
