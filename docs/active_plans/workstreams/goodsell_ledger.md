# Goodsell native implementation ledger

This ledger tracks the approved work in [goodsell_native.md](../../archive/goodsell_native.md). It is separate from the plan so that the plan remains the source of product intent.

## Status

| ID | Owner | Status | Prerequisites | Evidence |
| --- | --- | --- | --- | --- |
| M1 | reference_spec | accepted | None | Full paper/source analysis; specification and quality reviews accepted. [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md). |
| M2 | oracle | accepted | M1 | Current launcher: 38 render records (19 scenes per variant), each rendered twice; original 28-record snapshot remains preserved. The earlier 14 corrected bounds/FPE checks are historical and were not rerun on the added five scenes. Fresh SPEC/QUALITY reviews accepted; final integration ACCEPT confirms evidence correspondence. [goodsell_oracle.md](../reports/goodsell_oracle.md). |
| M3 | native_contracts | accepted | M1 | Root-crate contracts passed fresh SPEC and QUALITY review; D11 removed a redundant parser projection. |
| M4 | pdb_cif_adapters | accepted | M3 | PDB/mmCIF adapters passed fresh SPEC and QUALITY review; D11 fixes decimal roundtrip rejection; public RCSB 2HHB render passes. |
| M5 | xml_compression | accepted | M3 | PDBML and gzip input passed fresh SPEC and QUALITY review. |
| M6 | assembly_engine | accepted | M4, M5 | Model/assembly handling and PDB BIOMT passed fresh SPEC and QUALITY review; assembly metadata belongs to M4. |
| M7 | scene_preparation | accepted | M6 | Scene preparation passed fresh SPEC and QUALITY review; included within the 42-unit-test total. |
| M8 | raster | accepted | M7 | Rasterization passed fresh SPEC and QUALITY review; matched-depth comparison evidence is in M12. |
| M9 | shading | accepted | M8 | Shading passed fresh SPEC and QUALITY review. |
| M10 | outlines | accepted | M8 | Outlines passed fresh SPEC and QUALITY review. |
| M11 | native_cli | accepted | M9, M10 | CLI/rendering passed fresh SPEC and QUALITY review; three integration tests and 922 x 750 hemoglobin output are recorded. |
| M12 | render_evidence | accepted | M11 | All 19 masks match; 13 small scenes match atom ownership, with at most 1 byte RGB differences; hemoglobin D10 and one precision-tie pixel are attributed. Independent visual assessment passes. Benchmarks and D12 refinement are recorded; fresh SPEC/QUALITY reviews and final integration ACCEPT confirm current module/probe hashes. |
| M13 | release_docs / native_validation | accepted | M12 | Public docs, local native platform evidence, cleanup, and final integration ACCEPT complete closure. macOS arm64 and emulated Linux x86_64 checks pass; physical x86_64 remains unobserved. GitHub workflows were removed following the user's 2026-11-12 correction. [goodsell_integration_review.md](../reports/goodsell_integration_review.md), [goodsell_cleanup.md](../reports/goodsell_cleanup.md). |

## Decisions

| ID | Decision | Rationale | Affected milestones | Evidence |
| --- | --- | --- | --- | --- |
| P1 | Treat `goodsell_native.md` as the source of user intent. | The user supplied a complete approved plan. | M1-M13 | This ledger and plan artifact. |
| P2 | This plan transcription uses inherited context; later bounded work uses smaller `gpt-6.1-sol` or `gpt-6-luna` delegates. | Preserve the complete approved plan accurately, then use the smallest capable delegate. | M1-M13 | Dispatch records. |
| P3 | Use root `Cargo.toml`, `Cargo.lock`, and `src/`; run Cargo from the repository root. | The user's 2026-11-11 preference removes the extra single-crate directory layer. | M3-M13 | Dated layout amendment in [goodsell_native.md](../../archive/goodsell_native.md); [goodsell_contract.md](../decisions/goodsell_contract.md). |
| D1 | Correct undefined source state by initializing the background halo and `type(0)`, resetting per-pixel and kernel 3/4 accumulators, and using `pfogh` for occupied pixels and `1` for background pixels when occupied depth spread is zero. | Undefined or invalid Fortran behavior must be deterministic in native rendering. | M1, M3, M8-M10 | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); source contract and final native integration accepted. |
| D2 | Use the shared Rust contract in [goodsell_contract.md](../decisions/goodsell_contract.md). | Native milestones must implement one consistent contract. | M3-M13 | [goodsell_contract.md](../decisions/goodsell_contract.md); implemented root crate. |
| D3 | Native automatic-depth centering uses physical maximum radius in Angstroms plus one; preserve the original double-scaled equation as legacy documentation. Keep rotate, center, translate, scale order; match camera centers for raw-depth comparisons. | Correct the automatic-depth offset without changing operation order or raw-depth comparison conditions. | M1, M3, M7-M8, M12 | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); source contract and final native integration accepted. |
| D12 | Keep one-pass sphere rasterization; defer persistent tile binning and threading. | Raster is 2.9-7.3% of measured public render time across the tested 800 x 800 workloads; added indexing/concurrency lacks measured benefit. | M8, M12 | [goodsell_fidelity.md](../reports/goodsell_fidelity.md); dated amendment in [goodsell_native.md](../../archive/goodsell_native.md). |

## Decision propagation

| Task | Constraint | Restatement | Application and evidence | Next owner |
| --- | --- | --- | --- | --- |
| M1-M13 | P1: the approved plan controls scope and acceptance. | The approved plan controls the bounded milestone assignments. | Root manager dispatch records and this ledger. | Manager. |
| M1-M13 | P2: use a bounded smaller delegate after transcription. | Bounded work uses the manager-assigned smaller capable delegates. | Dispatch records, including M13 documentation assignment. | Manager. |
| M3-M13 | P3: use the single-crate root layout. | All source owners use root `src/`; package and binary names remain unchanged. | Root `Cargo.toml`, `Cargo.lock`, and `src/`; Cargo verification from repository root. | All native source owners. |
| M3, M8-M10 | D1: initialize the background halo and `type(0)`; reset per-pixel and kernel 3/4 accumulators; apply the exact flat-depth fog weights. | Contract and implementation must state each initialization/reset and use `pfogh` for occupied pixels and `1` for background pixels at zero occupied depth spread. | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); implemented in integrated raster, shading, and outline passes. | Native integration reviewer |
| M3 | D2: use the shared Rust contract. | Match the released contract decision record. | [goodsell_contract.md](../decisions/goodsell_contract.md); implemented by root crate. | `native_contracts` |
| M7-M8, M12 | D3: use Angstrom `rmax` plus one for native automatic-depth centering, with legacy equation retained. | Preserve rotate, center, translate, scale order; raw-depth oracle comparisons use an explicitly matched camera center; full-scene projections may frame radii-inclusive bounds. | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); source contract and final native integration accepted. | Native integration reviewer |
| M8, M12 | D12: retain one-pass rasterization; defer persistent tiles and threading. | Keep the measured single traversal and reconsider only when a measured workload shows a raster bottleneck. | [goodsell_fidelity.md](../reports/goodsell_fidelity.md); raster is 2.9-7.3% of measured public render time. | Manager |

## Review route

Each milestone requires a fresh specification reviewer, then a different fresh quality reviewer. Corrections and re-reviews use fresh agents. A fresh integration review accepts the completed composition before M13 closes the plan.

## Final acceptance: 2026-11-11

All M1-M13 milestones are complete and accepted. M1 and M3-M11 passed independent specification
and quality review; M2 and M12/M13 verification-tool work passed fresh specification and quality
review before the final independent [goodsell_integration_review.md](../reports/goodsell_integration_review.md)
returned ACCEPT. [goodsell_visual_assessment.md](../reports/goodsell_visual_assessment.md) records
independent visual PASS; [goodsell_fidelity.md](../reports/goodsell_fidelity.md) records all 19 exact
masks and six identical format/gzip outputs. Final review verified all five production module
hashes and the diagnostic probe hash against the retained manifest.

Fresh integration checks passed 837 pytest tests, 42 Rust unit tests, 3 CLI integration tests,
1 doctest, formatting, all-target check/Clippy, and missing-docs rustdoc. The post-cleanup
320 x 240 CIF render matches the clean-source macOS and emulated Linux SHA-256.
[goodsell_cleanup.md](../reports/goodsell_cleanup.md) records completed legacy-code removal;
[goodsell_oracle_lifecycle.md](../reports/goodsell_oracle_lifecycle.md) records the retained evidence
and temporary regeneration lifecycle. The completed plan is archived in
[goodsell_native.md](../../archive/goodsell_native.md). D12 remains an explicit measured deferral;
physical x86_64 and remote CI execution remain unobserved.
