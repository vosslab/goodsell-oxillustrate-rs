# Changelog

## 2026-11-11

### Additions and New Features

- Implement the native `oxillustrate` library and `illustrate` CLI: local structure loading,
  owned molecular records, assembly expansion, scene preparation, sphere depth rasterization,
  conical shadows, depth fog, three outline passes, and PNG output.
- Accept PDB, macromolecular PDBx/mmCIF, and PDBML XML, including gzip-compressed inputs.
  Preserve author/label namespaces, insertion codes, alternate locations, and assembly metadata.
- Add JSON scene settings with explicit CLI overrides and source-derived hemoglobin appearance.
- Add the reference mathematics specification and reproducible deterministic Fortran oracle.
- Add README onboarding, installation, usage/API, architecture, and file-structure documentation;
  compile-check the crate-front public API example.
- Embed the native hemoglobin WebP in the README and verify the full public API documentation
  build, including rustdoc's missing-documentation checks.

### Behavior or Interface Changes

- Use first-appearing model selection and highest mean-occupancy residue alternates with lexical
  ties; retain shared blank-location atoms. Default visibility hides hydrogen, deuterium, and water.
- Use a warm chain palette, sphere-bounds orthographic fit, and source-compatible row/column axes.
- Apply D1 initialized background/scratch and flat-depth fog corrections and D3 physical-radius
  automatic depth burial; preserve explicit legacy-camera comparisons as separate evidence.
- D10 uses stable structural subunit identities, joining each hemoglobin heme to its protein chain
  and deliberately removing legacy chain-run outline arcs; isolated comparisons document this.
- Bound decompressed input, image pixels, assembly operations, and expanded atom references;
  reject invalid numeric settings, unsupported selections, and empty visible scenes.

### Fixes and Maintenance

- Refresh the README with a directly reproducible public hemoglobin example, reader-focused
  documentation groups, and current legacy-code and platform verification status; preserve the
  native render and scientific credits.
- Keep the single Rust package at repository root after the user's layout amendment.
- Save PNG through a sibling temporary file and rename; reject CLI output paths resolving to
  input coordinates or scene JSON. Preserve previous output when encoding or flushing fails.
- Replace ignored local-reference and removed root Apache-license/NOTICE links with upstream
  commit-pinned source links and scientific DOI links.
- Keep paper and source snapshot hashes as development provenance without requiring local
  reference collections in a source distribution. Oracle regeneration verifies pinned upstream
  downloads and removes temporary source/executables automatically. The original 28-record
  snapshot matches with both download and verified-local inputs; current regeneration covers 38.

### Removals and Deprecations

- Delete the ignored historical source checkout and nested Git history, old oracle source/patch/
  executables, temporary plan checks/PDF review scratch, duplicate native build caches, and QEMU
  core. Retain primary comparison data, manifests, native installed binaries, and local papers.
- Archive the accepted plan with `git mv` to [goodsell_native.md](archive/goodsell_native.md),
  preserve the original user plan and amendments, and repair referring documentation links.

- Honor the user's removal of root `LICENSE.Apache-2.0` and `NOTICE`. Native source uses AGPL-3.0;
  documentation and other non-code materials retain CC-BY-4.0.

### Decisions and Failures

- The initial post-cleanup pytest run had 835 passes and one Bandit B310 failure on the fixed
  HTTPS/pinned-revision/filename-allowlist oracle download. A narrow audit comment resolved it
  without changing behavior; all 42 targeted checks and the final full 837-test run pass.

- Treat the original approved plan as historical intent and append amendments/evidence rather
  than rewriting it. Final integration acceptance must precede archival.
- Keep legacy source/executables only through final comparison work; preserve images, hashes,
  commands, and upstream provenance when removing those temporary references.
- The documented public RCSB 2HHB mmCIF render exposed a source-atom 125 coordinate-validation
  disagreement. D11 removes the redundant synthetic parser roundtrip that changed decimal
  coordinates; the corrected public 2HHB render passes at 922 x 750. The original synthetic
  example also renders successfully at 800 x 800.

### Developer Tests and Notes

- README refresh verification passes the locked release build, offline sample render,
  public RCSB hemoglobin download/render, default-framing example, executable help, and
  224 focused README/link/ASCII/whitespace checks. PNG dimensions match the documented
  800 x 800 and 922 x 750 outputs; the refreshed hemoglobin output was visually inspected.
- Manager-reported native checks pass: formatting, check, clippy, 42 unit tests, and 3 CLI
  integration tests covering formats, gzip, overrides, errors, and atomic-save failures.
- The original oracle snapshot records 28 scenes/variants rendered twice with repeated hashes;
  current regeneration records 38. All 14 scenes in the earlier corrected-reference snapshot
  passed bounds and floating-point trap builds; the five later references were not included in
  that historical trap run. This is oracle evidence, not a native cross-platform acceptance claim.
- Documentation verification passes the crate-front doctest, generated Rust API docs, executable
  help, and 223 focused README/link/guidance/ASCII/whitespace checks. Public RCSB download and
  hemoglobin rendering pass; the resulting PNG was visually inspected.
- Clean-source macOS arm64 formatting/check/tests/Clippy/install and outside-source CLI rendering
  pass. Linux x86_64 cross-build, 42 unit/3 integration/1 documentation tests, install, and actual
  emulated CLI rendering pass; both 320 x 240 sample PNGs have identical SHA-256. Physical x86_64
  execution and configured remote CI runs were not observed. Direct x86_64 compiler execution
  segfaulted in QEMU; an arm64 Linux cross-compiler enabled the successful target checks. Current
  public rustdoc comments were added after the clean source snapshot; final macOS checks were
  refreshed, while the Linux snapshot remains implementation-equivalent rather than byte-identical
  to current `src/`.
- M12 matches all 19 oracle masks; 13 small scenes preserve atom ownership, with depth error at
  most 0.0000128 pixel and at most one-byte RGB differences. Matched hemoglobin has 900 differing
  RGB pixels of 691500, traced to
  D10 identity changes and source/native precision. Final 800 x 800 local runs take 0.31 seconds
  for hemoglobin, 0.25-0.27 seconds for protein/DNA 1LMB, and 0.51-0.53 seconds for expanded
  1CWP assembly 1 (227040 spheres, 60 operator instances). This is local evidence, not a speedup
  or arbitrary-input guarantee.
- Independent visual assessment passes; it traces the hemoglobin D10 effect and precision-tie
  pixel. Raster is 2.9-7.3% of measured public render time, so the approved plan explicitly defers
  tile binning and threading until a measured workload justifies them. Final integration ACCEPT,
  legacy cleanup, and plan archival are complete. Full evidence is in
  [goodsell_fidelity.md](active_plans/reports/goodsell_fidelity.md),
  [goodsell_visual_assessment.md](active_plans/reports/goodsell_visual_assessment.md), and
  [goodsell_native_validation.md](active_plans/reports/goodsell_native_validation.md).

- Fresh independent [goodsell_integration_review.md](active_plans/reports/goodsell_integration_review.md)
  returns ACCEPT: 837 pytest tests, 42 Rust unit tests, 3 CLI integration tests, 1 doctest, fmt,
  all-target check/Clippy, missing-docs rustdoc, and diff whitespace pass. Post-cleanup CIF
  execution produces a 320 x 240 PNG with SHA-256
  `1c9c892e48799866bdfb99e446f9f7a6c9366ef832997bbad6f70a6603d59efe`, matching the
  clean-source macOS and emulated Linux samples. All 13 ledger milestones are accepted.
- Post-archival documentation checks pass: `source source_me.sh && python3 -m pytest -q
  tests/test_markdown_links.py tests/test_guidance_doc_format.py tests/test_ascii_compliance.py
  tests/test_whitespace.py tests/test_readme_first_paragraph.py` (226 passed), and
  `git diff --check`. The retained README WebP exists; all authored plan links resolve to the
  archive. Product sources and tests were unchanged during this documentation closeout.
