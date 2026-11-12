//! Streaming PDBML adapter; only coordinate and assembly categories are retained.
//! Official default or prefixed namespaces and wholly unqualified PDBML are accepted.
use std::collections::BTreeMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;

use crate::atom_table::{self, Categories};
use crate::{Error, Structure};

fn xml_error(message: impl ToString) -> Error {
    Error::Parse {
        format: "PDBML",
        message: message.to_string(),
    }
}

fn official_namespace(namespace: &str) -> bool {
    [
        "http://pdbml.pdb.org/schema/pdbx-",
        "https://pdbml.pdb.org/schema/pdbx-",
    ]
    .iter()
    .any(|prefix| {
        namespace
            .strip_prefix(prefix)
            .is_some_and(|tail| !tail.is_empty() && !tail.contains(['/', '?', '#']))
    })
}

fn accepted_namespace(namespace: ResolveResult<'_>, unqualified: bool) -> bool {
    match namespace {
        ResolveResult::Bound(namespace) => !unqualified && official_namespace(namespace.as_ref()),
        ResolveResult::Unbound => unqualified,
        ResolveResult::Unknown(_) => false,
    }
}

fn needed_category(name: &str) -> bool {
    matches!(
        name,
        "atom_site"
            | "pdbx_struct_assembly"
            | "pdbx_struct_assembly_gen"
            | "pdbx_struct_oper_list"
            | "entity_poly"
            | "entity"
            | "struct_asym"
    )
}

fn field_name(category: Option<&str>, name: &str) -> String {
    if category == Some("pdbx_struct_oper_list") {
        // PDBML removes brackets from the mmCIF matrix/vector field names.
        if let Some(indices) = name.strip_prefix("matrix") {
            let bytes = indices.as_bytes();
            if bytes.len() == 2 && bytes.iter().all(|n| (b'1'..=b'3').contains(n)) {
                return format!("matrix[{}][{}]", bytes[0] as char, bytes[1] as char);
            }
        }
        if let Some(index) = name.strip_prefix("vector")
            && matches!(index, "1" | "2" | "3")
        {
            return format!("vector[{index}]");
        }
    }
    name.to_owned()
}

fn attributes(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
) -> Result<(BTreeMap<String, String>, bool), Error> {
    let mut values = BTreeMap::new();
    let mut nil = false;
    for attribute in element.attributes() {
        let attribute = attribute.map_err(xml_error)?;
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(xml_error)?
            .into_owned();
        match namespace {
            ResolveResult::Unbound => {
                values.insert(local.as_ref().to_owned(), value);
            }
            ResolveResult::Bound(namespace)
                if namespace.as_ref() == "http://www.w3.org/2001/XMLSchema-instance"
                    && local.as_ref() == "nil" =>
            {
                nil = matches!(value.as_str(), "true" | "1");
            }
            _ => {}
        }
    }
    Ok((values, nil))
}

pub(crate) fn parse_xml(text: &str) -> Result<Structure, Error> {
    let mut reader = NsReader::from_str(text);
    reader.config_mut().check_comments = true;
    let mut categories = Categories::new();
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut unqualified = false;
    let mut category: Option<String> = None;
    let mut row: Option<BTreeMap<String, String>> = None;
    let mut field: Option<(String, String, bool)> = None;
    loop {
        let event = reader.read_event().map_err(|error| {
            xml_error(format!(
                "malformed XML at byte {}: {error}",
                reader.error_position()
            ))
        })?;
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let empty = matches!(event, Event::Empty(_));
                let (namespace, local) = reader.resolver().resolve_element(element.name());
                if depth == 0 {
                    if root_seen || local.as_ref() != "datablock" {
                        return Err(xml_error("expected one PDBML datablock root"));
                    }
                    unqualified = matches!(namespace, ResolveResult::Unbound);
                    root_seen = true;
                }
                let accepted = accepted_namespace(namespace, unqualified);
                if depth == 0 && !accepted {
                    return Err(xml_error("datablock must use an official PDBML namespace"));
                }
                let name = local.as_ref();
                let (values, nil) = attributes(&reader, element)?;
                if depth == 1 {
                    category = if accepted {
                        name.strip_suffix("Category")
                            .filter(|name| needed_category(name))
                            .map(str::to_owned)
                    } else {
                        None
                    };
                } else if depth == 2 && accepted && category.as_deref() == Some(name) {
                    row = Some(values);
                } else if depth == 3 && row.is_some() {
                    if !accepted {
                        return Err(xml_error(format!(
                            "field {name} uses a foreign or undeclared namespace"
                        )));
                    }
                    field = Some((field_name(category.as_deref(), name), String::new(), nil));
                } else if depth >= 4 && field.is_some() {
                    return Err(xml_error(
                        "PDBML fields must contain text, not nested elements",
                    ));
                }
                depth += 1;
                if !empty {
                    continue;
                }
                finish_element(
                    &mut depth,
                    &mut category,
                    &mut row,
                    &mut field,
                    &mut categories,
                );
            }
            Event::End(_) => {
                if depth == 0 {
                    return Err(xml_error("unexpected closing element"));
                }
                finish_element(
                    &mut depth,
                    &mut category,
                    &mut row,
                    &mut field,
                    &mut categories,
                );
            }
            Event::Text(value) => {
                let value = value.xml_content(quick_xml::XmlVersion::Implicit1_0);
                if let Some((_, content, _)) = &mut field {
                    content.push_str(&value);
                } else if depth == 0 && !value.trim().is_empty() {
                    return Err(xml_error("text outside the datablock root"));
                }
            }
            Event::CData(value) => {
                if let Some((_, content, _)) = &mut field {
                    content.push_str(&value.xml_content(quick_xml::XmlVersion::Implicit1_0));
                } else if depth == 0 {
                    return Err(xml_error("CDATA outside the datablock root"));
                }
            }
            Event::GeneralRef(value) => {
                let escaped = format!(
                    "&{};",
                    value.xml_content(quick_xml::XmlVersion::Implicit1_0)
                );
                let decoded = quick_xml::escape::unescape(&escaped).map_err(xml_error)?;
                if let Some((_, content, _)) = &mut field {
                    content.push_str(&decoded);
                } else if depth == 0 {
                    return Err(xml_error("entity reference outside the datablock root"));
                }
            }
            Event::DocType(_) => {
                return Err(xml_error(
                    "DTD declarations are not supported; external entities are disabled",
                ));
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if depth != 0 || !root_seen {
        return Err(xml_error(
            "truncated XML: expected a complete PDBML datablock",
        ));
    }
    atom_table::from_categories(&categories, Vec::new()).map_err(|error| match error {
        Error::Parse { message, .. } => xml_error(message),
        other => other,
    })
}

fn finish_element(
    depth: &mut usize,
    category: &mut Option<String>,
    row: &mut Option<BTreeMap<String, String>>,
    field: &mut Option<(String, String, bool)>,
    categories: &mut Categories,
) {
    if *depth == 4 {
        if let (Some((name, value, nil)), Some(row)) = (field.take(), row.as_mut())
            && !nil
        {
            row.insert(name, value.trim().to_owned());
        }
    } else if *depth == 3 {
        if let (Some(category), Some(row)) = (category.as_ref(), row.take()) {
            categories.entry(category.clone()).or_default().push(row);
        }
    } else if *depth == 2 {
        *category = None;
    }
    *depth -= 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn fixture() -> String {
        r#"<p:datablock xmlns:p="http://pdbml.pdb.org/schema/pdbx-v50.xsd"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
          <p:atom_siteCategory><p:atom_site id="7">
            <p:group_PDB>ATOM</p:group_PDB><p:type_symbol>C</p:type_symbol>
            <p:label_atom_id>CA</p:label_atom_id><p:auth_atom_id>C&amp;A</p:auth_atom_id>
            <p:label_comp_id>GLY</p:label_comp_id><p:auth_comp_id>GLY</p:auth_comp_id>
            <p:label_asym_id>LABEL_LONG</p:label_asym_id><p:auth_asym_id>AUTHOR_LONG</p:auth_asym_id>
            <p:label_seq_id>4</p:label_seq_id><p:auth_seq_id>12</p:auth_seq_id>
            <p:pdbx_PDB_ins_code>B</p:pdbx_PDB_ins_code><p:label_alt_id>A</p:label_alt_id>
            <p:Cartn_x><![CDATA[1.25]]></p:Cartn_x><p:Cartn_y>-2</p:Cartn_y><p:Cartn_z>3</p:Cartn_z>
            <p:occupancy>0.75</p:occupancy><p:pdbx_PDB_model_num>2</p:pdbx_PDB_model_num>
          </p:atom_site></p:atom_siteCategory>
        </p:datablock>"#.to_owned()
    }

    #[test]
    fn namespaces_and_text_preserve_normalized_atom_identity() {
        let structure = parse_xml(&fixture()).unwrap();
        let atom = &structure.atoms[0];
        assert_eq!(atom.source_id, "7");
        assert_eq!(atom.position, [1.25, -2.0, 3.0]);
        assert_eq!(atom.author_name.as_deref(), Some("C&A"));
        assert_eq!(atom.label_name.as_deref(), Some("CA"));
        assert_eq!(atom.residue.author_chain.as_deref(), Some("AUTHOR_LONG"));
        assert_eq!(atom.residue.label_chain.as_deref(), Some("LABEL_LONG"));
        assert_eq!(atom.residue.author_sequence_number, Some(12));
        assert_eq!(atom.residue.label_sequence, Some(4));
        assert_eq!(atom.residue.insertion_code.as_deref(), Some("B"));
        assert_eq!(atom.model, 2);
        assert_eq!(atom.occupancy, 0.75);
        assert_eq!(atom.alt_loc.as_deref(), Some("A"));
        let default = fixture()
            .replace("xmlns:p=", "xmlns=")
            .replace("<p:", "<")
            .replace("</p:", "</");
        assert_eq!(parse_xml(&default).unwrap(), structure);
        let unqualified = default.replace("xmlns=\"http://pdbml.pdb.org/schema/pdbx-v50.xsd\"", "");
        assert_eq!(parse_xml(&unqualified).unwrap(), structure);
    }

    #[test]
    fn xml_and_cif_share_normalization() {
        let cif = "data_example\nloop_\n_atom_site.id\n_atom_site.group_PDB\n_atom_site.type_symbol\n_atom_site.label_atom_id\n_atom_site.auth_atom_id\n_atom_site.label_comp_id\n_atom_site.auth_comp_id\n_atom_site.label_asym_id\n_atom_site.auth_asym_id\n_atom_site.label_seq_id\n_atom_site.auth_seq_id\n_atom_site.pdbx_PDB_ins_code\n_atom_site.label_alt_id\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.occupancy\n_atom_site.pdbx_PDB_model_num\n7 ATOM C CA 'C&A' GLY GLY LABEL_LONG AUTHOR_LONG 4 12 B A 1.25 -2 3 0.75 2\n";
        assert_eq!(
            parse_xml(&fixture()).unwrap().atoms,
            crate::cif_input::parse_cif(cif).unwrap().atoms
        );
    }

    #[test]
    fn pdbml_assembly_attribute_ids_and_matrix_fields_are_normalized() {
        let metadata = r#"
          <p:pdbx_struct_assemblyCategory><p:pdbx_struct_assembly id="bio1"/></p:pdbx_struct_assemblyCategory>
          <p:pdbx_struct_assembly_genCategory><p:pdbx_struct_assembly_gen assembly_id="bio1">
            <p:oper_expression>op1</p:oper_expression><p:asym_id_list>LABEL_LONG</p:asym_id_list>
          </p:pdbx_struct_assembly_gen></p:pdbx_struct_assembly_genCategory>
          <p:pdbx_struct_oper_listCategory><p:pdbx_struct_oper_list id="op1">
            <p:matrix11>1</p:matrix11><p:matrix12>0</p:matrix12><p:matrix13>0</p:matrix13>
            <p:matrix21>0</p:matrix21><p:matrix22>1</p:matrix22><p:matrix23>0</p:matrix23>
            <p:matrix31>0</p:matrix31><p:matrix32>0</p:matrix32><p:matrix33>1</p:matrix33>
            <p:vector1>10</p:vector1><p:vector2>0</p:vector2><p:vector3>-5</p:vector3>
          </p:pdbx_struct_oper_list></p:pdbx_struct_oper_listCategory>"#;
        let xml = fixture().replace("</p:datablock>", &format!("{metadata}</p:datablock>"));
        let structure = parse_xml(&xml).unwrap();
        assert_eq!(structure.operations["op1"].translation, [10.0, 0.0, -5.0]);
        assert_eq!(
            structure.operations["op1"].rotation,
            crate::Transform::default().rotation
        );
        assert_eq!(structure.assemblies[0].id, "bio1");
        assert_eq!(structure.assemblies[0].generators[0].chains, ["LABEL_LONG"]);
        assert_eq!(
            structure.assemblies[0].generators[0].operation_expression,
            "op1"
        );
    }

    #[test]
    fn rejects_malformed_empty_unsafe_foreign_and_nonfinite_input() {
        for text in [
            "<datablock>",
            "<datablock><atom_siteCategory></datablock>",
            "<datablock/>",
            "<!DOCTYPE datablock [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><datablock/>",
            "<datablock xmlns='https://foreign.example/pdbx-v50.xsd'/>",
        ] {
            assert!(parse_xml(text).is_err(), "accepted {text}");
        }
        let foreign = fixture()
            .replace("<p:Cartn_x>", "<f:Cartn_x xmlns:f='urn:foreign'>")
            .replace("</p:Cartn_x>", "</f:Cartn_x>");
        assert!(parse_xml(&foreign).is_err());
        let nil = fixture().replace("<p:Cartn_x>", "<p:Cartn_x xsi:nil='true'>");
        assert!(parse_xml(&nil).is_err());
        let nonfinite = fixture().replace("1.25", "NaN");
        assert!(
            parse_xml(&nonfinite)
                .unwrap_err()
                .to_string()
                .contains("Cartn_x")
        );
    }
}
