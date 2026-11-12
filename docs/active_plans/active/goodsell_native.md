# Goodsell-style molecular illustration in Rust

## Context and objectives

Build a native Rust library and CLI that reproduce Illustrate's distinctive molecular appearance: space-filling atoms, restrained colors, depth fog, soft conical shadows, and contour, subunit, and residue outlines.

The repository currently contains scaffolding, local papers and web documentation, and Goodsell's Apache-licensed Fortran implementation. There is no existing Rust application to preserve.

The first release accepts PDB, PDBx/mmCIF, and coordinate-bearing PDBML XML. Reuse existing readers wherever practical: `pdbtbx` documents PDB and mmCIF support; XML requires a separate adapter. Preserve modern identifiers rather than converting inputs through legacy PDB. Sources: [pdbtbx documentation](https://docs.rs/pdbtbx/latest/pdbtbx/), [wwPDB coordinate definitions](https://mmcif.wwpdb.org/docs/tutorials/content/atomic-description.html).

## Design philosophy and boundaries

- Preserve the Goodsell look through direct translation of the relevant rendering mathematics. Replace fixed arrays, fixed-column selection, and implicit global state with owned data and explicit parameters.
- Deliver one native package under `oxillustrate/`, containing a library and an `illustrate` binary. Use safe, synchronous Rust; introduce no GPU, async runtime, plugin system, or rendering framework.
- Support macOS ARM64 and Linux x86-64. Use Rust edition 2024, repository dependency conventions, and a committed lockfile.
- Exclude legacy command parsing, GUI, browser delivery, ribbons, molecular surfaces, animation, and additional illustration styles. Clear module boundaries leave room for later additions without implementing them now.
- Keep `LOCAL_ONLY/` and `OTHER_REPOS/` as reference material. Retain required Goodsell copyright and Apache notices in translated portions and distribution notices.

### Architecture and public contract

The library owns three operations: load a structure, prepare a scene, and render an image. Expose owned `Structure`, `SceneSettings`, and `RenderedImage` values plus a typed error enum. Rendering borrows immutable structure/settings and owns its scratch buffers; no dependency-specific types escape the library.

Separate input adapters, normalized molecular data, scene preparation, sphere rasterization, shading/outlines, and image output into cohesive modules. Use dynamic vectors and checked image-size arithmetic.

Provide:

```text
illustrate structure.cif --output illustration.png
illustrate structure.xml --scene scene.json --output illustration.png
```

JSON scene settings cover model/assembly selection, orientation, framing, background, ordered atom styling rules, fog, shadows, and outlines. CLI flags cover input, output, scene file, image dimensions, and model/assembly selection; explicit flags override scene values. Keep rendering parameters in the scene file rather than creating dozens of CLI switches.

Use `clap`, `serde`/`serde_json`, `pdbtbx`, `quick-xml`, and PNG encoding from an existing image crate. Add gzip decoding for compressed structure files. Disable unnecessary dependency features.

## Milestones

Each milestone has one implementation owner and a separate reviewing agent. The manager dispatches independent tasks only after their shared contracts are settled, records evidence, and resolves failures without waiting for human responses.

| Milestone | Deliverable and implementation | Dependency and completion evidence |
| --- | --- | --- |
| 1. Reference specification | Read the complete local papers, web files, Fortran, and sample scene. Document equations, coordinate conventions, operation order, kernel boundaries, defaults, and attribution requirements. Distinguish Illustrate behavior from Protein Imager features. | None. Independent reviewer checks the specification against source, including conical shadows and all three outline types. |
| 2. Reference oracle | Compile Fortran and render supplied hemoglobin plus small generated sphere scenes. Preserve commands, compiler version, settings, images, and reference provenance. Run the oracle in an isolated output directory. | M1. Successful repeatable renders; provision `gfortran` locally or in a Linux build container if absent. No end-user Fortran dependency. |
| 3. Native crate and contracts | Create the library/CLI package, domain errors, settings serialization, and owned molecular types. Separate structural identifiers from dense internal indices. | M1. Native build and one API/CLI success and failure path. |
| 4. PDB and mmCIF adapters | Use `pdbtbx`; retain coordinates, element, atom/residue names, model, occupancy, alternate locations, insertion codes, and chain identifiers. Inspect its actual metadata coverage and add focused category extraction where required information is discarded. | M3. Equivalent PDB/mmCIF fixtures normalize to equivalent renderable structures; long chain identifiers remain intact. |
| 5. PDBML and compression | Implement namespace-aware streaming XML extraction using `quick-xml`, including atom coordinates and required identity fields. Decode gzip before parsing. Reject XML without coordinates with an actionable error. | M3; may run alongside M4. Equivalent XML fixture matches normalized atoms; malformed and coordinate-free inputs fail predictably. |
| 6. Models and assemblies | Preserve assembly definitions and operations from mmCIF/PDBML and PDB BIOMT records. Expand requested assemblies with correct operator composition and unique instance identities. Default to the deposited coordinates and first model. | M4-M5. Known transform fixtures verify positions, composition, chain targeting, and no duplicate identity operation. |
| 7. Styling and scene preparation | Implement ordered first-match rules for element, chain, residue name/range, atom name, and polymer/hetero/water identity. Rules set color, radius, or omission. Apply assembly, orientation, centering, framing, and projection explicitly. | M6. Hemoglobin settings reproduce source atom selection and colors; bounds include sphere radii. |
| 8. Sphere rasterization | Implement orthographic sphere depth rasterization with color, depth, atom, residue, and subunit identity buffers. Match source clipping and visibility conventions. | M7. Analytic sphere silhouettes, overlap, clipping, and nearest-surface tests pass; raw depth images agree with the oracle. |
| 9. Goodsell shading | Translate the source conical shadow calculation, fog, and compositing order. Keep calculations deterministic and avoid replacing the shadow model with generic ambient occlusion. | M8. Isolated pass comparisons identify and correct differences against Fortran. |
| 10. Goodsell outlines | Translate contour kernels, subunit boundaries, and residue-distance outlines. Use dense identity indices while preserving residue sequence information separately. Prevent accidental boundaries caused by modern identifier encoding. | M8; may run alongside M9. Dedicated scenes exercise each outline independently and together. |
| 11. Finished rendering and CLI | Integrate passes, PNG output, scene loading, diagnostics, and reproducible examples. Write output only after successful rendering; use temporary output plus rename to avoid partial images. | M9-M10. End-to-end PDB, mmCIF, XML, and gzip commands create equivalent images. |
| 12. Fidelity and scale | Generate contact sheets, difference images, timings, and peak-memory evidence. Compare hemoglobin, a protein-nucleic acid complex, and a large assembly. Add conservative tile-based atom binning for rasterization; parallelize independent output rows only where measurements justify it. | M11. Automated geometry/pass tests and independent visual assessment pass; documented measurements include hardware, input, resolution, and commands. |
| 13. Release closure | Complete usage/API documentation, example scenes, fixture provenance, notices, changelog, human guidance, design decisions, and native CI. Archive the completed plan with its evidence report. | M12. Clean checkout builds and renders the documented example on both supported targets. |

**Parallel-plan ready: yes.** Input adapters can proceed independently after M3; shading and outlines can proceed independently after M8. The manager owns integration and contract changes.

## Required defaults

- First model; highest-occupancy alternate conformer per residue, with deterministic identifier tie-breaking and shared blank-location atoms retained.
- Hide hydrogen/deuterium and waters by default; retain other hetero atoms.
- Use source-derived radii and a restrained chain palette, with an explicit fallback radius for unknown elements and a diagnostic.
- Use white background, orthographic projection, automatic framing, and source-derived rendering defaults.
- Accept arbitrary-length chain identifiers and preserve author and label identifiers when available. Explicit selectors distinguish those namespaces.
- Reject non-finite coordinates, invalid rendering parameters, empty visible scenes, unknown requested models/assemblies, and impossible allocations. Report parser warnings without silently dropping scientifically meaningful data.
- Support macromolecular PDBx/mmCIF; small-molecule crystallographic CIF is outside this release.

## Verification and recovery

Run `cargo fmt --check`, `cargo check`, `cargo test`, and `cargo clippy -- -D warnings`, plus native target CI.

Retain focused tests for format equivalence, identity preservation, assembly transforms, alternate-location selection, projection, depth ordering, rendering passes, and CLI errors. Use small synthetic fixtures for mathematical expectations and representative structures for end-to-end evidence.

For visual fidelity, compare matched scenes against Fortran using silhouettes, depth discontinuities, shadow distribution, outline placement, and final contact sheets. Require an independent image-assessment agent to identify material departures from the reference appearance. Pixel identity is not required. Any departure must be traced to a pass and corrected before closure; deliberate fixes to legacy artifacts require documented evidence.

Performance results are measurements, not an invented speed promise. Remove repeated full atom scans using tile bins; investigate any measured regression after optimization. Preserve deterministic output and bound scratch storage by image size and scene size.

If a reader loses required metadata, complete the focused adapter in this plan. If oracle generation fails, repair its build environment before fidelity-dependent milestones proceed. If visual assessment fails, reopen the responsible rendering milestone. No milestone requires human approval, credentials, publication, or a remote service.

## Documentation and completion

Expected edits are confined to the new native package, relevant fixtures/CI, and repository documentation. Save the implementation plan under `docs/active_plans/active/` when execution begins; maintain milestone status and evidence there.

Completion means a clean native installation renders all three required formats into PNG, preserves modern molecular identities, reproduces the Goodsell appearance under independent assessment, and ships documented scenes and reproducible verification commands.

No implementation or Cargo checks have been performed during planning. GPU acceleration and interactive interfaces remain outside this release; visual fidelity is the principal implementation risk.

## User amendment: 2026-11-11

The user specifies a new implementation of the documented mathematics and behavior under the
existing [LICENSE.AGPL-3.0](../../../LICENSE.AGPL-3.0). This amendment supersedes the original
direct-translation, translated-source notice, and legacy-retention wording above. Preserve the
original plan text as history. The legacy code is barely usable and serves only as a temporary
implementation and goal reference. Scientific attribution and upstream provenance remain in
[NOTICE](../../../NOTICE) and [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md). Reference inspection
has occurred; this amendment makes no clean-room claim.

Keep the legacy reference through M12 fidelity comparisons. After final comparison evidence,
delete the old source and compiled copies, including `.f` files and executables in
`output_oracle/`. Retain comparison images, source hashes, commands, and provenance. Developer
oracle regeneration may fetch a temporary upstream snapshot, but the finished native product
has no end-user dependency on legacy source, executables, or Fortran.

