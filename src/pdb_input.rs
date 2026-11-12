//! Legacy PDB fixed-column metadata, without manufacturing label identities.
use std::collections::BTreeMap;
use std::io::{BufReader, Cursor};

use crate::atom_table::{self, Categories};
use crate::error::Error;
use crate::molecular::Structure;

pub(crate) fn parse_pdb(text: &str) -> Result<Structure, Error> {
    let mut categories = Categories::new();
    let mut rows = Vec::new();
    let mut model = "1".to_owned();
    for (index, line) in text.lines().enumerate() {
        let record = line.get(..6).unwrap_or(line).trim();
        if record == "MODEL" {
            model = line.get(10..14).unwrap_or("").trim().to_owned();
            if model.parse::<i32>().is_err() {
                return Err(Error::Parse {
                    format: "PDB",
                    message: format!("line {}: MODEL requires an integer serial", index + 1),
                });
            }
        }
        if !matches!(record, "ATOM" | "HETATM") {
            continue;
        }
        if !line.is_ascii() || line.len() < 54 {
            return Err(Error::Parse {
                format: "PDB",
                message: format!(
                    "line {}: atom record needs ASCII fixed columns through Z coordinate (column 54)",
                    index + 1
                ),
            });
        }
        let mut row = BTreeMap::new();
        for (key, start, end) in [
            ("id", 6, 11),
            ("auth_atom_id", 12, 16),
            ("label_alt_id", 16, 17),
            ("auth_comp_id", 17, 20),
            ("auth_asym_id", 21, 22),
            ("auth_seq_id", 22, 26),
            ("pdbx_PDB_ins_code", 26, 27),
            ("Cartn_x", 30, 38),
            ("Cartn_y", 38, 46),
            ("Cartn_z", 46, 54),
            ("occupancy", 54, 60),
            ("type_symbol", 76, 78),
        ] {
            let value = line.get(start..end).unwrap_or("").trim();
            if !value.is_empty() || key == "auth_asym_id" {
                row.insert(key.to_owned(), value.to_owned());
            }
        }
        // PDB atom-name alignment distinguishes calcium ("CA  ") from alpha carbon (" CA ").
        if !row.contains_key("type_symbol") {
            let raw = &line[12..16];
            let letters: String = raw
                .chars()
                .filter(char::is_ascii_alphabetic)
                .take(
                    if raw.starts_with(' ') || raw.starts_with(|c: char| c.is_ascii_digit()) {
                        1
                    } else {
                        2
                    },
                )
                .collect();
            if !letters.is_empty() {
                row.insert("type_symbol".into(), letters);
                row.insert("inferred_element".into(), "true".into());
            }
        }
        row.insert("group_PDB".into(), record.to_owned());
        row.insert("pdbx_PDB_model_num".into(), model.clone());
        rows.push(row);
    }
    let mut diagnostics = Vec::new();
    for row in &rows {
        if row.contains_key("inferred_element") {
            diagnostics.push(format!(
                "PDB atom {}: missing element; inferred {} from aligned atom name",
                row.get("id").map(String::as_str).unwrap_or("missing"),
                row["type_symbol"]
            ));
        }
    }
    categories.insert("atom_site".into(), rows);
    let mut structure = atom_table::from_categories(&categories, diagnostics)?;
    for (atom, row) in structure.atoms.iter_mut().zip(&categories["atom_site"]) {
        // A blank PDB chain is an author identifier, including REMARK 350 NULL targets.
        atom.residue.author_chain = row.get("auth_asym_id").cloned();
    }
    let result = pdbtbx::ReadOptions::new()
        .set_format(pdbtbx::Format::Pdb)
        .set_level(pdbtbx::StrictnessLevel::Loose)
        .read_raw(BufReader::new(Cursor::new(text)));
    match result {
        Ok((parsed, warnings)) => {
            structure
                .diagnostics
                .extend(warnings.iter().map(|w| format!("pdbtbx PDB reader: {w}")));
            if parsed.total_atom_count() != structure.atoms.len() {
                return Err(Error::Parse {
                    format: "PDB",
                    message: format!(
                        "pdbtbx accepted {} of {} coordinate records; check atom names, serials, and elements",
                        parsed.total_atom_count(),
                        structure.atoms.len()
                    ),
                });
            }
        }
        Err(errors) => {
            return Err(Error::Parse {
                format: "PDB",
                message: errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            });
        }
    }
    crate::assembly_metadata::apply_pdb(text, &mut structure)?;
    Ok(structure)
}
