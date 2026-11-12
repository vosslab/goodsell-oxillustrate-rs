use std::collections::BTreeMap;

use crate::{Error, Namespace, Structure, Transform};

/// Limits expression expansion and temporary atom references before allocation.
const MAX_TRANSFORMS: usize = 1_000_000;
const MAX_ATOM_REFERENCES: usize = 20_000_000;

#[derive(Debug)]
pub(crate) struct Instance {
    pub id: usize,
    pub transform: Transform,
    pub atom_indices: Vec<usize>,
}

/// Default to the first deposited model in source order, not the lowest model ID.
pub(crate) fn select_model(structure: &Structure, requested: Option<i32>) -> Result<i32, Error> {
    let model = requested
        .or_else(|| structure.atoms.first().map(|atom| atom.model))
        .ok_or(Error::EmptyScene)?;
    if !structure.atoms.iter().any(|atom| atom.model == model) {
        return invalid(format!("unknown model {model}"));
    }
    Ok(model)
}

/// Expand explicit biological operations, capped at 1M transforms and 20M atom references.
pub(crate) fn instances(
    structure: &Structure,
    selected_model: i32,
    assembly: Option<&str>,
) -> Result<Vec<Instance>, Error> {
    select_model(structure, Some(selected_model))?;
    let mut model_atoms = Vec::new();
    for (index, atom) in structure.atoms.iter().enumerate() {
        if atom.model == selected_model {
            model_atoms.try_reserve(1).map_err(allocation_error)?;
            model_atoms.push(index);
        }
    }
    let Some(id) = assembly else {
        return Ok(vec![Instance {
            id: 0,
            transform: Transform::default(),
            atom_indices: model_atoms,
        }]);
    };
    let definition = structure
        .assemblies
        .iter()
        .find(|definition| definition.id == id)
        .ok_or_else(|| Error::Selection(format!("unknown assembly {id}")))?;
    let mut result: Vec<Instance> = Vec::new();
    let mut by_transform = BTreeMap::new();
    let mut reference_count = 0usize;
    for generator in &definition.generators {
        let mut atom_indices = Vec::new();
        for &index in &model_atoms {
            let residue = &structure.atoms[index].residue;
            let chain = match generator.chain_namespace {
                Namespace::Author => &residue.author_chain,
                Namespace::Label => &residue.label_chain,
            };
            if chain
                .as_ref()
                .is_some_and(|chain| generator.chains.contains(chain))
            {
                if atom_indices.len() == MAX_ATOM_REFERENCES {
                    return invalid("assembly exceeds 20M atom reference limit".into());
                }
                atom_indices.try_reserve(1).map_err(allocation_error)?;
                atom_indices.push(index);
            }
        }
        let transforms = expand(&generator.operation_expression, &structure.operations)?;
        let added = atom_indices.len().checked_mul(transforms.len());
        reference_count = added
            .and_then(|added| reference_count.checked_add(added))
            .filter(|&count| count <= MAX_ATOM_REFERENCES)
            .ok_or_else(|| Error::Selection("assembly exceeds 20M atom reference limit".into()))?;
        for transform in transforms {
            if atom_indices.is_empty() {
                continue;
            }
            let key = transform_key(&transform);
            let index = match by_transform.get(&key) {
                Some(&index) => index,
                None => {
                    if result.len() == MAX_TRANSFORMS {
                        return invalid("assembly exceeds 1M transform limit".into());
                    }
                    result.try_reserve(1).map_err(allocation_error)?;
                    let index = result.len();
                    result.push(Instance {
                        id: index,
                        transform,
                        atom_indices: Vec::new(),
                    });
                    by_transform.insert(key, index);
                    index
                }
            };
            let indices = &mut result[index].atom_indices;
            indices
                .try_reserve(atom_indices.len())
                .map_err(allocation_error)?;
            indices.extend_from_slice(&atom_indices);
        }
    }
    for instance in &mut result {
        instance.atom_indices.sort_unstable();
        instance.atom_indices.dedup();
    }
    Ok(result)
}

fn transform_key(transform: &Transform) -> [u64; 12] {
    let mut key = [0; 12];
    for (slot, value) in key.iter_mut().zip(
        transform
            .rotation
            .iter()
            .flatten()
            .chain(transform.translation.iter()),
    ) {
        *slot = if *value == 0.0 { 0 } else { value.to_bits() };
    }
    key
}

fn validate_transform(transform: &Transform) -> Result<(), Error> {
    if transform
        .rotation
        .iter()
        .flatten()
        .chain(transform.translation.iter())
        .any(|v| !v.is_finite())
    {
        return invalid("assembly transform must contain finite values".into());
    }
    Ok(())
}

/// Matrix product A*B: B acts first on the coordinate column vector.
fn compose(a: &Transform, b: &Transform) -> Result<Transform, Error> {
    let mut result = Transform::default();
    for row in 0..3 {
        for column in 0..3 {
            result.rotation[row][column] = (0..3)
                .map(|k| a.rotation[row][k] * b.rotation[k][column])
                .sum();
        }
        result.translation[row] = a.translation[row]
            + (0..3)
                .map(|k| a.rotation[row][k] * b.translation[k])
                .sum::<f64>();
    }
    validate_transform(&result)?;
    Ok(result)
}

fn expand(
    expression: &str,
    operations: &BTreeMap<String, Transform>,
) -> Result<Vec<Transform>, Error> {
    let expression = expression.trim();
    let mut groups = Vec::new();
    if expression.starts_with('(') {
        let mut rest = expression;
        while !rest.is_empty() {
            let body = rest
                .strip_prefix('(')
                .ok_or_else(|| Error::Selection("expected parenthesized operation group".into()))?;
            let close = body
                .find(')')
                .ok_or_else(|| Error::Selection("unclosed operation group".into()))?;
            groups.push(&body[..close]);
            rest = body[close + 1..].trim_start();
        }
    } else {
        groups.push(expression);
    }
    let mut expanded = vec![Transform::default()];
    for group in groups {
        let group_operations = operation_group(group, operations)?;
        let count = expanded
            .len()
            .checked_mul(group_operations.len())
            .filter(|&count| count <= MAX_TRANSFORMS)
            .ok_or_else(|| {
                Error::Selection("operation expression exceeds 1M transform limit".into())
            })?;
        let mut product = Vec::new();
        product.try_reserve_exact(count).map_err(allocation_error)?;
        for prefix in &expanded {
            for operation in &group_operations {
                product.push(compose(prefix, operation)?);
            }
        }
        expanded = product;
    }
    Ok(expanded)
}

fn operation_group<'a>(
    group: &str,
    operations: &'a BTreeMap<String, Transform>,
) -> Result<Vec<&'a Transform>, Error> {
    let mut result = Vec::new();
    for token in group.split(',').map(str::trim) {
        if token.is_empty() || token.contains(['(', ')']) || token.chars().any(char::is_whitespace)
        {
            return invalid(format!("invalid operation identifier {token:?}"));
        }
        // Exact IDs take precedence so nonnumeric identifiers retain their source spelling.
        if let Some(operation) = operations.get(token) {
            append_operation(&mut result, operation)?;
        } else if let Some((start, end)) = token.split_once('-') {
            let start: u32 = start
                .parse()
                .map_err(|_| Error::Selection(format!("unknown operation {token}")))?;
            let end: u32 = end
                .parse()
                .map_err(|_| Error::Selection(format!("unknown operation {token}")))?;
            if start > end || u64::from(end) - u64::from(start) + 1 > MAX_TRANSFORMS as u64 {
                return invalid(format!("invalid or oversized operation range {token}"));
            }
            for id in start..=end {
                let operation = operations
                    .get(&id.to_string())
                    .ok_or_else(|| Error::Selection(format!("unknown operation {id}")))?;
                append_operation(&mut result, operation)?;
            }
        } else {
            return invalid(format!("unknown operation {token}"));
        }
    }
    Ok(result)
}

fn append_operation<'a>(
    operations: &mut Vec<&'a Transform>,
    operation: &'a Transform,
) -> Result<(), Error> {
    if operations.len() == MAX_TRANSFORMS {
        return invalid("operation group exceeds 1M transform limit".into());
    }
    validate_transform(operation)?;
    operations.try_reserve(1).map_err(allocation_error)?;
    operations.push(operation);
    Ok(())
}

fn allocation_error(error: std::collections::TryReserveError) -> Error {
    Error::Selection(format!("cannot allocate assembly expansion: {error}"))
}

fn invalid<T>(message: String) -> Result<T, Error> {
    Err(Error::Selection(message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Assembly, AssemblyGenerator, Atom, AtomKind, ResidueId};

    fn atom(model: i32, author: &str, label: &str) -> Atom {
        Atom {
            source_id: "1".into(),
            position: [1.0, 0.0, 0.0],
            element: "C".into(),
            name: "CA".into(),
            author_name: None,
            label_name: None,
            residue: ResidueId {
                author_chain: Some(author.into()),
                label_chain: Some(label.into()),
                author_sequence: Some("1".into()),
                author_sequence_number: Some(1),
                label_sequence: Some(1),
                insertion_code: None,
                name: "ALA".into(),
                author_name: None,
                label_name: None,
            },
            model,
            occupancy: 1.0,
            alt_loc: None,
            kind: AtomKind::Polymer,
        }
    }

    fn generator(namespace: Namespace, chain: &str, expression: &str) -> AssemblyGenerator {
        AssemblyGenerator {
            chains: vec![chain.into()],
            chain_namespace: namespace,
            operation_expression: expression.into(),
        }
    }

    #[test]
    fn products_apply_rightmost_first_and_expand_numeric_ranges() {
        let rotate = Transform {
            rotation: [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        };
        let translate = Transform {
            translation: [2.0, 0.0, 0.0],
            ..Transform::default()
        };
        let operations = BTreeMap::from([
            ("X0".into(), rotate),
            ("1".into(), Transform::default()),
            ("2".into(), translate),
        ]);
        let product = expand(" (X0) (1-2) ", &operations).unwrap();
        assert_eq!(product.len(), 2);
        assert_eq!(product[1].translation, [0.0, 2.0, 0.0]);
        assert_eq!(product[1].rotation[0], [0.0, -1.0, 0.0]);
        let point = [1.0, 0.0, 0.0];
        let output: [f64; 3] = std::array::from_fn(|row| {
            product[1].translation[row]
                + (0..3)
                    .map(|k| product[1].rotation[row][k] * point[k])
                    .sum::<f64>()
        });
        assert_eq!(output, [0.0, 3.0, 0.0]);
        assert_eq!(expand("(1,2)(1-2)", &operations).unwrap().len(), 4);
        assert_eq!(
            expand("(2)(X0)", &operations).unwrap()[0].translation,
            [2.0, 0.0, 0.0]
        );
    }

    #[test]
    fn model_selection_namespace_targets_and_duplicate_identity() {
        let mut structure = Structure {
            atoms: vec![atom(7, "A", "L1"), atom(7, "L1", "L2"), atom(1, "A", "L1")],
            operations: BTreeMap::from([
                ("1".into(), Transform::default()),
                ("identity".into(), Transform::default()),
                (
                    "copy".into(),
                    Transform {
                        translation: [4.0, 0.0, 0.0],
                        ..Transform::default()
                    },
                ),
            ]),
            assemblies: vec![Assembly {
                id: "bio".into(),
                generators: vec![
                    generator(Namespace::Label, "L1", "1,identity,copy"),
                    generator(Namespace::Author, "L1", "1"),
                ],
            }],
            ..Structure::default()
        };
        assert_eq!(select_model(&structure, None).unwrap(), 7);
        assert_eq!(select_model(&structure, Some(1)).unwrap(), 1);
        let deposited = instances(&structure, 7, None).unwrap();
        assert_eq!(deposited[0].atom_indices, [0, 1]);
        let copies = instances(&structure, 7, Some("bio")).unwrap();
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0].atom_indices, [0, 1]);
        assert_eq!(copies[1].atom_indices, [0]);
        assert_eq!((copies[0].id, copies[1].id), (0, 1));
        assert_eq!(copies[1].transform.translation, [4.0, 0.0, 0.0]);
        assert!(instances(&structure, 2, None).is_err());
        assert!(instances(&structure, 7, Some("unknown")).is_err());
        structure.operations.get_mut("copy").unwrap().translation[0] = f64::NAN;
        assert!(instances(&structure, 7, Some("bio")).is_err());
    }

    #[test]
    fn invalid_expressions_missing_operations_and_expansion_overflow_are_errors() {
        let operations = BTreeMap::from([("1".into(), Transform::default())]);
        for expression in [
            "",
            "()",
            "(1",
            "1)",
            "(1),1",
            "1,,1",
            "2",
            "2-1",
            "1-4294967295",
        ] {
            assert!(expand(expression, &operations).is_err(), "{expression}");
        }
        let group = std::iter::repeat_n("1", 101).collect::<Vec<_>>().join(",");
        let expression = format!("({group})({group})({group})");
        assert!(expand(&expression, &operations).is_err());
        assert!(matches!(
            select_model(&Structure::default(), None),
            Err(Error::EmptyScene)
        ));
    }
}
