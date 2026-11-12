//! Deposited assembly definitions; applying operators belongs to the assembly engine.
use std::collections::BTreeMap;

use crate::atom_table::{Categories, value};
use crate::error::Error;
use crate::molecular::{Assembly, AssemblyGenerator, Namespace, Structure, Transform};

fn invalid(format: &'static str, message: impl Into<String>) -> Error {
    Error::Parse {
        format,
        message: message.into(),
    }
}

pub(crate) fn apply_categories(
    categories: &Categories,
    structure: &mut Structure,
) -> Result<(), Error> {
    if let Some(rows) = categories.get("pdbx_struct_oper_list") {
        for row in rows {
            let id = value(row, "id")
                .ok_or_else(|| invalid("PDBx", "assembly operation is missing id"))?;
            let mut transform = Transform::default();
            for axis in 0..3 {
                for column in 0..4 {
                    let key = if column == 3 {
                        format!("vector[{}]", axis + 1)
                    } else {
                        format!("matrix[{}][{}]", axis + 1, column + 1)
                    };
                    let number = value(row, &key)
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| {
                            invalid(
                                "PDBx",
                                format!("assembly operation {id}: {key} must be a finite number"),
                            )
                        })?;
                    if column == 3 {
                        transform.translation[axis] = number;
                    } else {
                        transform.rotation[axis][column] = number;
                    }
                }
            }
            if structure
                .operations
                .insert(id.to_owned(), transform)
                .is_some()
            {
                return Err(invalid(
                    "PDBx",
                    format!("duplicate assembly operation {id}"),
                ));
            }
        }
    }
    if let Some(rows) = categories.get("pdbx_struct_assembly") {
        for row in rows {
            let id = value(row, "id").ok_or_else(|| invalid("PDBx", "assembly is missing id"))?;
            if !structure.assemblies.iter().any(|a| a.id == id) {
                structure.assemblies.push(Assembly {
                    id: id.to_owned(),
                    generators: Vec::new(),
                });
            }
        }
    }
    if let Some(rows) = categories.get("pdbx_struct_assembly_gen") {
        for row in rows {
            let required = |key| {
                value(row, key)
                    .ok_or_else(|| invalid("PDBx", format!("assembly generator is missing {key}")))
            };
            let id = required("assembly_id")?;
            let expression = required("oper_expression")?;
            let chains: Vec<_> = required("asym_id_list")?
                .split(',')
                .map(str::trim)
                .map(str::to_owned)
                .collect();
            if chains.iter().any(String::is_empty) {
                return Err(invalid(
                    "PDBx",
                    format!("assembly {id}: empty chain in asym_id_list"),
                ));
            }
            let index = match structure.assemblies.iter().position(|a| a.id == id) {
                Some(index) => index,
                None => {
                    structure.assemblies.push(Assembly {
                        id: id.to_owned(),
                        generators: Vec::new(),
                    });
                    structure.assemblies.len() - 1
                }
            };
            structure.assemblies[index]
                .generators
                .push(AssemblyGenerator {
                    chains,
                    chain_namespace: Namespace::Label,
                    operation_expression: expression.to_owned(),
                });
        }
    }
    Ok(())
}

pub(crate) fn apply_pdb(text: &str, structure: &mut Structure) -> Result<(), Error> {
    let mut assembly_ids = Vec::<String>::new();
    let mut chains = Vec::<String>::new();
    let mut pending = BTreeMap::<String, ([Option<[f64; 4]>; 3], Vec<String>, Vec<String>)>::new();
    let mut operation_number = 0;
    for (line_index, line) in text.lines().enumerate() {
        let Some(content) = line.strip_prefix("REMARK 350") else {
            continue;
        };
        let content = content.trim();
        if let Some(ids) = content.strip_prefix("BIOMOLECULE:") {
            assembly_ids = ids
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .collect();
            for id in &assembly_ids {
                if !structure.assemblies.iter().any(|a| &a.id == id) {
                    structure.assemblies.push(Assembly {
                        id: id.clone(),
                        generators: Vec::new(),
                    });
                }
            }
            chains.clear();
        } else if let Some(list) = content
            .strip_prefix("APPLY THE FOLLOWING TO CHAINS:")
            .or_else(|| content.strip_prefix("IN ADDITION APPLY THE FOLLOWING TO CHAINS:"))
        {
            chains = pdb_chains(list);
        } else if let Some(list) = content.strip_prefix("AND CHAINS:") {
            chains.extend(pdb_chains(list));
        } else if content.starts_with("BIOMT") {
            let fields: Vec<_> = content.split_whitespace().collect();
            let fail = || {
                invalid(
                    "PDB",
                    format!(
                        "line {}: BIOMT requires row 1-3, operation id, and four finite numbers",
                        line_index + 1
                    ),
                )
            };
            if fields.len() != 6 {
                return Err(fail());
            }
            let axis = fields[0]
                .strip_prefix("BIOMT")
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| (1..=3).contains(v))
                .ok_or_else(fail)?
                - 1;
            if assembly_ids.is_empty() || chains.is_empty() {
                return Err(invalid(
                    "PDB",
                    format!(
                        "line {}: BIOMT has no preceding BIOMOLECULE and chain list",
                        line_index + 1
                    ),
                ));
            }
            let mut numbers = [0.0; 4];
            for (index, field) in fields[2..].iter().enumerate() {
                numbers[index] = field
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or_else(fail)?;
            }
            let entry = pending
                .entry(fields[1].to_owned())
                .or_insert_with(|| ([None; 3], chains.clone(), assembly_ids.clone()));
            if entry.1 != chains || entry.2 != assembly_ids || entry.0[axis].is_some() {
                return Err(invalid(
                    "PDB",
                    format!(
                        "line {}: duplicate or interrupted BIOMT operation {}",
                        line_index + 1,
                        fields[1]
                    ),
                ));
            }
            entry.0[axis] = Some(numbers);
            if entry.0.iter().all(Option::is_some) {
                let (rows, target_chains, target_assemblies) = pending
                    .remove(fields[1])
                    .expect("completed operation exists");
                let mut transform = Transform::default();
                for (axis, row) in rows.into_iter().enumerate() {
                    let row = row.expect("all three BIOMT rows are present");
                    transform.rotation[axis].copy_from_slice(&row[..3]);
                    transform.translation[axis] = row[3];
                }
                operation_number += 1;
                let id = operation_number.to_string();
                structure.operations.insert(id.clone(), transform);
                for assembly in structure
                    .assemblies
                    .iter_mut()
                    .filter(|a| target_assemblies.contains(&a.id))
                {
                    assembly.generators.push(AssemblyGenerator {
                        chains: target_chains.clone(),
                        chain_namespace: Namespace::Author,
                        operation_expression: id.clone(),
                    });
                }
            }
        }
    }
    if !pending.is_empty() {
        return Err(invalid(
            "PDB",
            format!(
                "incomplete BIOMT operation(s): {}",
                pending.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    Ok(())
}

fn pdb_chains(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| {
            if v == "NULL" {
                String::new()
            } else {
                v.to_owned()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pdb_operations_target_their_own_chain_group() {
        let mut structure = Structure::default();
        apply_pdb("REMARK 350 BIOMOLECULE: 1\nREMARK 350 APPLY THE FOLLOWING TO CHAINS: A, B\nREMARK 350   BIOMT1 1 1 0 0 0\nREMARK 350   BIOMT2 1 0 1 0 0\nREMARK 350   BIOMT3 1 0 0 1 0\nREMARK 350 APPLY THE FOLLOWING TO CHAINS: C\nREMARK 350   BIOMT1 1 1 0 0 10\nREMARK 350   BIOMT2 1 0 1 0 0\nREMARK 350   BIOMT3 1 0 0 1 0\n", &mut structure).unwrap();
        assert_eq!(structure.assemblies[0].generators[0].chains, ["A", "B"]);
        assert_eq!(structure.assemblies[0].generators[1].chains, ["C"]);
        assert_eq!(structure.operations["2"].translation, [10.0, 0.0, 0.0]);
        assert!(apply_pdb("REMARK 350 BIOMOLECULE: 1\nREMARK 350 APPLY THE FOLLOWING TO CHAINS: A\nREMARK 350 BIOMT1 1 1 0 0 0", &mut Structure::default()).is_err());
    }
}
