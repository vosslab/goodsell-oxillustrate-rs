//! PDBx identities stay in owned rows; pdbtbx validates a transient coordinate projection.
use std::io::{BufReader, Cursor};

use crate::atom_table;
use crate::cif_tokens;
use crate::error::Error;
use crate::molecular::Structure;

pub(crate) fn parse_cif(text: &str) -> Result<Structure, Error> {
    let categories = cif_tokens::categories(text)?;
    let mut structure = atom_table::from_categories(&categories, Vec::new())?;
    validate_coordinates(&mut structure)?;
    Ok(structure)
}

pub(crate) fn validate_coordinates(structure: &mut Structure) -> Result<(), Error> {
    // pdbtbx requires numeric IDs and label identifiers, and collapses author/label identities.
    let mut projection = String::from(
        "data_validation\nloop_\n_atom_site.group_PDB\n_atom_site.id\n_atom_site.type_symbol\n_atom_site.label_atom_id\n_atom_site.label_comp_id\n_atom_site.label_asym_id\n_atom_site.label_seq_id\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.occupancy\n",
    );
    for (index, atom) in structure.atoms.iter().enumerate() {
        projection.push_str(&format!(
            "ATOM {} {} C UNK A {} {} {} {} {}\n",
            index + 1,
            atom.element,
            index + 1,
            atom.position[0],
            atom.position[1],
            atom.position[2],
            atom.occupancy
        ));
    }
    let result = pdbtbx::ReadOptions::new()
        .set_format(pdbtbx::Format::Mmcif)
        .set_level(pdbtbx::StrictnessLevel::Loose)
        .read_raw(BufReader::new(Cursor::new(projection)));
    match result {
        Ok((parsed, warnings)) => {
            structure.diagnostics.extend(
                warnings
                    .iter()
                    .map(|w| format!("pdbtbx coordinate validation: {w}")),
            );
            if parsed.total_atom_count() != structure.atoms.len() {
                return Err(Error::Parse {
                    format: "PDBx",
                    message: format!(
                        "pdbtbx accepted {} of {} atoms; check atom elements and coordinates",
                        parsed.total_atom_count(),
                        structure.atoms.len()
                    ),
                });
            }
            for (expected, parsed) in structure.atoms.iter().zip(parsed.atoms()) {
                let (x, y, z) = parsed.pos();
                if expected.position != [x, y, z] {
                    return Err(Error::Parse {
                        format: "PDBx",
                        message: format!(
                            "coordinate validation disagreed for source atom {}",
                            expected.source_id
                        ),
                    });
                }
            }
            Ok(())
        }
        Err(errors) => Err(Error::Parse {
            format: "PDBx",
            message: format!(
                "coordinate validation failed: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PDB: &str =
        "ATOM      1  CA  ALA A   1       1.000   2.000   3.000  0.50 20.00           C  \nEND\n";
    const CIF: &str = "data_test\nloop_\n_atom_site.group_PDB\n_atom_site.id\n_atom_site.type_symbol\n_atom_site.auth_atom_id\n_atom_site.auth_comp_id\n_atom_site.auth_asym_id\n_atom_site.auth_seq_id\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.occupancy\nATOM 1 C CA ALA A 1 1 2 3 0.5\n";

    #[test]
    fn equivalent_author_only_pdb_and_cif_atoms() {
        let pdb = crate::pdb_input::parse_pdb(PDB).unwrap();
        let cif = parse_cif(CIF).unwrap();
        assert_eq!(pdb.atoms, cif.atoms);
        assert_eq!(cif.atoms[0].label_name, None);
    }

    #[test]
    fn dual_identities_and_arbitrary_source_ids_survive() {
        let cif = "data_test\nloop_\n_atom_site.group_PDB\n_atom_site.id\n_atom_site.type_symbol\n_atom_site.auth_atom_id\n_atom_site.label_atom_id\n_atom_site.auth_comp_id\n_atom_site.label_comp_id\n_atom_site.auth_asym_id\n_atom_site.label_asym_id\n_atom_site.auth_seq_id\n_atom_site.label_seq_id\n_atom_site.pdbx_PDB_ins_code\n_atom_site.label_alt_id\n_atom_site.pdbx_PDB_model_num\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.occupancy\nHETATM 'atom long id' C AUTH_CA LABEL_CA MSE MET author_chain_very_long label_chain_very_long '42alpha' 7 B A 2 1 2 3 0.75\n";
        let structure = parse_cif(cif).unwrap();
        let atom = &structure.atoms[0];
        assert_eq!(atom.source_id, "atom long id");
        assert_eq!(atom.author_name.as_deref(), Some("AUTH_CA"));
        assert_eq!(atom.label_name.as_deref(), Some("LABEL_CA"));
        assert_eq!(
            atom.residue.author_chain.as_deref(),
            Some("author_chain_very_long")
        );
        assert_eq!(
            atom.residue.label_chain.as_deref(),
            Some("label_chain_very_long")
        );
        assert_eq!(atom.residue.author_sequence.as_deref(), Some("42alpha"));
        assert_eq!(atom.residue.author_sequence_number, None);
        assert_eq!(atom.residue.label_sequence, Some(7));
        assert_eq!(atom.residue.author_name.as_deref(), Some("MSE"));
        assert_eq!(atom.residue.label_name.as_deref(), Some("MET"));
        assert_eq!(atom.residue.insertion_code.as_deref(), Some("B"));
        assert_eq!(atom.alt_loc.as_deref(), Some("A"));
        assert_eq!(atom.kind, crate::molecular::AtomKind::Polymer);
        assert_eq!(atom.model, 2);
        assert_eq!(atom.occupancy, 0.75);
    }

    #[test]
    fn bad_coordinates_fail_with_original_atom_context_and_missing_elements_warn() {
        let error = parse_cif(&CIF.replace("1 1 2 3 0.5", "1 NaN 2 3 0.5"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("row 1 (id 1)"), "{error}");
        assert!(error.contains("Cartn_x must be finite"), "{error}");
        let inferred = parse_cif(&CIF.replace("ATOM 1 C CA", "ATOM 1 ? CA")).unwrap();
        assert_eq!(inferred.atoms[0].element, "C");
        assert!(
            inferred
                .diagnostics
                .iter()
                .any(|d| d.contains("missing element; inferred C"))
        );
        assert!(
            parse_cif("data_small\n_cell.length_a 10\n")
                .unwrap_err()
                .to_string()
                .contains("small-molecule CIF")
        );
    }

    #[test]
    fn cif_assembly_expression_and_long_chain_survive() {
        let cif = format!(
            "{CIF}\n_pdbx_struct_assembly.id 1\n_pdbx_struct_assembly_gen.assembly_id 1\n_pdbx_struct_assembly_gen.oper_expression '(1-3)(4,5)'\n_pdbx_struct_assembly_gen.asym_id_list long_label_chain,second_chain\n_pdbx_struct_oper_list.id 1\n_pdbx_struct_oper_list.matrix[1][1] 1\n_pdbx_struct_oper_list.matrix[1][2] 0\n_pdbx_struct_oper_list.matrix[1][3] 0\n_pdbx_struct_oper_list.matrix[2][1] 0\n_pdbx_struct_oper_list.matrix[2][2] 1\n_pdbx_struct_oper_list.matrix[2][3] 0\n_pdbx_struct_oper_list.matrix[3][1] 0\n_pdbx_struct_oper_list.matrix[3][2] 0\n_pdbx_struct_oper_list.matrix[3][3] 1\n_pdbx_struct_oper_list.vector[1] 10\n_pdbx_struct_oper_list.vector[2] 0\n_pdbx_struct_oper_list.vector[3] 0\n"
        );
        let structure = parse_cif(&cif).unwrap();
        assert_eq!(structure.operations["1"].translation, [10.0, 0.0, 0.0]);
        let generator = &structure.assemblies[0].generators[0];
        assert_eq!(generator.operation_expression, "(1-3)(4,5)");
        assert_eq!(generator.chains, ["long_label_chain", "second_chain"]);
        assert_eq!(
            generator.chain_namespace,
            crate::molecular::Namespace::Label
        );
    }
}
