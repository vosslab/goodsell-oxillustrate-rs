use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Naming namespace used by structural selectors.
pub enum Namespace {
    #[default]
    /// Author-assigned identifiers from the source record.
    Author,
    /// Label identifiers from the source record.
    Label,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Chemical record category, with water identified independently.
pub enum AtomKind {
    /// Atom belonging to a polymer entity.
    Polymer,
    /// Non-water heterogen atom.
    Hetero,
    /// Water molecule atom.
    Water,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Source residue identity; missing namespaces remain absent.
pub struct ResidueId {
    /// Author chain identifier, when supplied.
    pub author_chain: Option<String>,
    /// Label chain identifier, when supplied.
    pub label_chain: Option<String>,
    /// Raw author sequence text, retained even when it is not numeric.
    pub author_sequence: Option<String>,
    /// Parsed numeric author sequence, when available.
    pub author_sequence_number: Option<i32>,
    /// Numeric label sequence, when supplied.
    pub label_sequence: Option<i32>,
    /// Insertion code associated with the author sequence.
    pub insertion_code: Option<String>,
    /// Effective residue name, preferring the author name when present.
    pub name: String,
    /// Author residue name, when supplied.
    pub author_name: Option<String>,
    /// Label residue name, when supplied.
    pub label_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
/// One source atom, before assembly replication or visibility filtering.
pub struct Atom {
    /// Stable identity from the source atom record; not a dense render index.
    pub source_id: String,
    /// Source Cartesian coordinates in Angstroms.
    pub position: [f64; 3],
    /// Chemical element symbol.
    pub element: String,
    /// Effective atom name, preferring author naming when available.
    pub name: String,
    /// Author atom name, when supplied.
    pub author_name: Option<String>,
    /// Label atom name, when supplied.
    pub label_name: Option<String>,
    /// Full source residue identity, including optional namespace identifiers.
    pub residue: ResidueId,
    /// Deposited model number.
    pub model: i32,
    /// Source occupancy value.
    pub occupancy: f64,
    /// Alternate-location identifier; absent for shared atoms.
    pub alt_loc: Option<String>,
    /// Polymer, heterogen, or water classification.
    pub kind: AtomKind,
}

/// Row-major rotation acting on a column vector: p' = rotation * p + translation.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Row-major rotation matrix applied to column vectors.
    pub rotation: [[f64; 3]; 3],
    /// Translation vector in Angstroms.
    pub translation: [f64; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Chains targeted by an unmodified source operation expression.
pub struct AssemblyGenerator {
    /// Chain identifiers targeted by this generator.
    pub chains: Vec<String>,
    /// Namespace used to interpret `chains`.
    pub chain_namespace: Namespace,
    /// Original operation expression from the source metadata.
    pub operation_expression: String,
}

#[derive(Clone, Debug, PartialEq)]
/// A named biological assembly definition.
pub struct Assembly {
    /// Source assembly identifier.
    pub id: String,
    /// Generators that define the assembly's chain and operation combinations.
    pub generators: Vec<AssemblyGenerator>,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Owned source atoms, assembly definitions, operations, and parser diagnostics.
pub struct Structure {
    /// Parsed source atoms, before model and visibility selection.
    pub atoms: Vec<Atom>,
    /// Deposited biological assembly definitions.
    pub assemblies: Vec<Assembly>,
    /// Named transforms referenced by assembly operation expressions.
    pub operations: BTreeMap<String, Transform>,
    /// Non-fatal parser diagnostics retained for callers.
    pub diagnostics: Vec<String>,
}
