use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Naming namespace used by structural selectors.
pub enum Namespace {
    #[default]
    Author,
    Label,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Chemical record category, with water identified independently.
pub enum AtomKind {
    Polymer,
    Hetero,
    Water,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Source residue identity; missing namespaces remain absent.
pub struct ResidueId {
    pub author_chain: Option<String>,
    pub label_chain: Option<String>,
    pub author_sequence: Option<String>,
    pub author_sequence_number: Option<i32>,
    pub label_sequence: Option<i32>,
    pub insertion_code: Option<String>,
    pub name: String,
    pub author_name: Option<String>,
    pub label_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
/// One source atom, before assembly replication or visibility filtering.
pub struct Atom {
    pub source_id: String,
    pub position: [f64; 3],
    pub element: String,
    pub name: String,
    pub author_name: Option<String>,
    pub label_name: Option<String>,
    pub residue: ResidueId,
    pub model: i32,
    pub occupancy: f64,
    pub alt_loc: Option<String>,
    pub kind: AtomKind,
}

/// Row-major rotation acting on a column vector: p' = rotation * p + translation.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub rotation: [[f64; 3]; 3],
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
    pub chains: Vec<String>,
    pub chain_namespace: Namespace,
    pub operation_expression: String,
}

#[derive(Clone, Debug, PartialEq)]
/// A named biological assembly definition.
pub struct Assembly {
    pub id: String,
    pub generators: Vec<AssemblyGenerator>,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Owned source atoms, assembly definitions, operations, and parser diagnostics.
pub struct Structure {
    pub atoms: Vec<Atom>,
    pub assemblies: Vec<Assembly>,
    pub operations: BTreeMap<String, Transform>,
    pub diagnostics: Vec<String>,
}
