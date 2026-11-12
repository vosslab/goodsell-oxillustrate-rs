# Changelog

## 2026-11-12

### Additions and New Features

- Add `render_with_bounds` and optional CLI `--metadata PATH` for exact fixed-canvas frame
  registration after unconditional trimming. Report original `[width, height]` and native
  `[left, top]` crop origin; preserve `render`, `RenderedImage`, existing pixels, and defaults.
  Protect metadata from overwriting structural inputs, scene settings, or the image.
- Add native print resolution through `--dpi` and JSON `dpi`, defaulting to 300. Carry DPI
  through the public settings, scene, and image types. PNG writes physical pixel dimensions
  and sRGB; lossless WebP writes EXIF X/Y resolution and inch units. Use the existing transitive
  `png` crate directly for its metadata encoder; no new dependency tree is introduced.
- Trim rendered output automatically to atoms and outlines with three pixels of available
  padding. Preserve every pixel value and partial-alpha edge, and report final dimensions/print
  size on stderr. The initial trim override is removed after the user's simplifying correction.
- Add [GALLERY.md](GALLERY.md) with eight fresh molecular illustrations selected from the local
  2002 and 2014 Molecular Machinery posters: lysozyme, GFP, antibody, potassium channel, tRNA,
  nucleosome, GroEL/GroES, and rhinovirus. Include biological captions, PDB sources, reusable
  scene files, and native download/render commands. Link the gallery from the README.
- Add [devel/render_gallery.sh](../devel/render_gallery.sh) to regenerate the eight gallery images;
  the alpha correction makes these direct native lossless WebPs. Keep the two poster PDFs as
  ignored references; no poster image
  is copied into the gallery. Each molecule is framed independently, with that scale distinction
  stated on the page.
- Implement native `--pdb ID` / `load_pdb` RCSB downloads without curl. Fetch mmCIF with a
  synchronous TLS client, validated legacy/extended accessions, a 60-second timeout, and the
  existing 1 GiB input bound. Keep local input available and mutually exclusive with `--pdb`.
- Implement native RGBA PNG and lossless WebP encoding. Filename extension chooses the format;
  both encoders preserve alpha and use the existing atomic-save path. Preserve RGB pixels in
  the public image type and add an optional straight-alpha vector.
- Initially record transparent PNG and native WebP as top feature priorities in
  [TODO.md](TODO.md). At that checkpoint native output was opaque RGB PNG only;
  the implementation above completes those priorities.
- Add current CLI renders of lambda repressor bound to DNA and the complete cowpea chlorotic
  mottle virus capsid to the usage guide. Store quality-90 WebP captures with their commands,
  coordinate provenance, and image hashes in
  [goodsell_readme_capture.md](active_plans/reports/goodsell_readme_capture.md).

### Behavior or Interface Changes

- Remove explicit white backgrounds from every bundled scene and re-render all twelve molecular
  documentation images with real alpha. Write native lossless WebPs directly, preserving DPI
  and outline coverage without converter dependencies. Keep opaque backgrounds available as
  explicit user choices. The gallery and README now blend into light and dark pages.
- Default new scenes to transparent. `--background transparent|white|black` overrides JSON;
  JSON null means transparent and existing RGB background triples remain opaque. Rust scene
  backgrounds now use `Option<[f64; 3]>`. Partially covered outline pixels carry black with
  alpha coverage, removing a white matte without changing the opaque rendering equations.

### Fixes and Maintenance

- Verify all twelve transparent captures over light and dark backgrounds. Their dimensions
  retain automatic trimming, and their white composites match the previous opaque native PNGs
  within one byte per channel. The old hash comparison was against already-trimmed captures,
  not the original full-canvas images. Preserve the oracle background from its source cards
  independently of the transparent public examples. Update the current capture receipt in
  [print_trim.md](active_plans/reports/print_trim.md).
- Update print/size guidance and regenerate all twelve documentation images with automatic
  trimming. Keep source coordinates, molecular scale, colors, shadows, and outlines unchanged.
  Keep raw diagnostic/reference passes on their full canvas for depth-coordinate comparisons.
- Replace the misleading opaque `transparent_backgrounds.webp` comparison with actual native
  RGBA WebP. The usage guide now demonstrates transparency directly against the page background.
- Isolate the dark whole-capsid appearance to dense depth contours: changing the existing
  contour kernel from 4 to 3 reduces near-black foreground pixels from 25.33% to 5.17% on the
  matched 1CWP render. Add [cowpea_capsid.json](../examples/cowpea_capsid.json), update the three
  larger gallery scenes, and refresh all four images. Preserve shadows, subunit boundaries,
  the reference hemoglobin scene, global defaults, and renderer mathematics. Document the
  existing outline and shadow controls; record measurements and hashes in
  [goodsell_readme_capture.md](active_plans/reports/goodsell_readme_capture.md).
- Update student-facing commands to download and render directly; document native format and
  background selection, API additions, and completed feature priorities. Add an actual light/dark
  transparency comparison in [USAGE.md](USAGE.md).
- Rewrite the README for a biology master's student making a protein image: start with real
  hemoglobin, explain the resulting PNG, and guide structure, size, assembly, color, and view
  choices. Keep developer internals in supporting documentation and record the audience guidance.
- Refresh the existing hemoglobin capture from the current release build; its WebP bytes match
  the previous asset. Inspect all three encoded images; every managed image has a live embed.
- Trace the collection failure to the archived plan's missing old path in the Git index.
  The current index lists the existing archive path. The initial vendored helper override
  is subsequently removed as recorded below.
- Correct current README, installation, and validation documentation to use local checks.
  Record the user's workflow and vendored-file ownership guidance, and append a dated
  correction to the historical plan.
- Audit the benchmark downloader's fixed HTTPS RCSB URL and two hardcoded structure IDs;
  document the narrow Bandit B310 exception without changing download behavior.
- Synchronized shared style guides, tests, and repository support files from the starter template.

### Removals and Deprecations

- Remove the `--no-trim` flag and `trim` fields from JSON settings and public scene types.
  Always crop finished renders; add whitespace afterward. Keep raw developer pass exports
  separate from finished output and verify their cropped pixels against the production renderer.
- Archive the completed [image_output_downloads.md](archive/image_output_downloads.md) plan
  using `git mv`; retain its implementation and verification evidence in the linked report.
- The agent-added GitHub Actions workflow is removed; the user does not use GitHub workflows.
- Remove the added regression test for the local vendored helper override. The helper now
  matches its tracked version; no local vendored helper changes remain.

### Decisions and Failures

- Keep JSON optional; size, DPI, and background remain available through the CLI.
  Omit YAML, SVG, PDF, CMYK separation, bleed, and a second physical-units sizing interface.
  Existing pixel dimensions plus DPI provide print sizing in native PNG/WebP. The user
  explicitly declines SVG for this pixel-by-pixel renderer. Record the decision in
  [DESIGN_DECISIONS.md](DESIGN_DECISIONS.md).
- Use one background selector, native lossless WebP, and a fixed RCSB service without cache,
  arbitrary URLs, or async infrastructure. The suggested vossvolvox-rust repository supplies a
  read-only Python E2E download example; no code or master vendored files there are changed.
- Treating the plan's generic native-CI wording as a request for GitHub Actions was an agent
  assumption. The user's local workflow takes precedence.
- The earlier collection fix changed the vendored helper to check file existence before
  invoking content-reading callbacks. It also modified master files in starter-repo-template
  without user authorization: its `tests/file_utils.py`, `tests/meta/test_file_utils_discovery.py`,
  and `docs/CHANGELOG.md`. No further edits to that repository are made during this correction.

### Developer Tests and Notes

- Crop registration passes `cargo fmt --check`, `cargo check --locked --all-targets`,
  `cargo test --locked` (44 unit, 9 integration, 1 doctest),
  `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked --release`, and
  `RUSTDOCFLAGS='-D warnings -D missing_docs' cargo doc --no-deps --locked` on
  rustc/Cargo 1.99.1, aarch64-apple-darwin. Independent asymmetric sphere coverage reconstructs
  exactly from `[left, top]`; opaque white atoms retain matching bounds. CLI WebP pixels match
  the library, and conflicting destinations preserve input/image files. Repository
  `source source_me.sh && python3 -m pytest tests/ -q` passes all 1017 checks. Native release
  probe `examples/sample.pdb --width 100 --height 80 --output
  output/crop_registration/sample.webp --metadata output/crop_registration/sample.json`
  produces a 31 x 28 image with `canvas: [100, 80]`, `crop_origin: [33, 25]`.
  No movie, browser, Linux, or Fortran-oracle validation is repeated for this additive API.
- Full transparent recapture passes 1017 repository checks and five temporary image/reference
  checks, removed afterward. Every WebP has RGBA, partial-alpha outlines, EXIF 300 dpi, and a
  live embed; regenerated PNG counterparts have identical decoded pixels. Inspect all twelve
  images over both light and dark backgrounds. Application Rust is unchanged in this recapture.
- After removing trim controls, all 52 Rust tests and 1017 repository checks pass, along with
  formatting, all-target compilation, Clippy, rustdoc, and the release build. The raw-pass probe
  verifies cropped production RGB. Hemoglobin's native PNG is byte-identical to its prior
  cropped output; the larger print command now reports 1352 x 1748 pixels after cropping.
- Print/trim verification passes formatting, locked all-target compilation, all 52 Rust tests,
  Clippy with warnings denied, rustdoc with missing docs denied, and 1014 repository pytest
  checks. Pillow independently reads PNG/WebP resolution and confirms identical RGBA pixels
  across DPI values. The documented 2400 x 1800 print command succeeds; native crop results
  match independent alpha/background-bounds trims. Refresh and inspect twelve final WebPs.
  Current evidence and platform/printing limits are in
  [print_trim.md](active_plans/reports/print_trim.md).
- Transparency/contour correction verification passes all 50 Rust tests and 1003 repository
  pytest checks. Five fresh native downloads/renders succeed, all final WebPs are visually
  inspected and decode without errors, and the transparent capture matches native PNG RGBA
  pixels exactly. Three temporary capture checks pass and are removed. No browser, Linux, or
  Fortran-oracle comparison is repeated; application Rust and vendored files are unchanged.
- Build the current release, render all eight gallery scenes twice through native `--pdb`, and
  inspect all final WebPs. The potassium-channel camera is adjusted to show the pore; the other
  seven captures reproduce their first-run hashes. All PNG bounds have at least 28 pixels of
  clear border; WebP metadata reports no errors. The transparent nucleosome/WebP command also
  succeeds. Gallery work changes no application Rust or vendored files.
- Gallery verification passes `bash -n`, `git diff --check`, and the exact
  `source source_me.sh && pytest tests/` command: 1003 tests. Poster PDFs remain ignored and
  untracked. Capture details are in [gallery_capture.md](active_plans/reports/gallery_capture.md).
- Native feature checks pass formatting, locked all-target compilation, 44 unit tests, 5 CLI/API
  integration tests, 1 doctest, Clippy with warnings denied, and rustdoc with missing docs denied.
  Fresh repository pytest passes: 953 tests before archival and 957 after archival/link repair.
  Real RCSB legacy/extended-ID requests and an
  installed outside-source render pass; HTTP 404 preserves existing output. PNG/WebP RGBA pixels
  match exactly. Opaque hemoglobin and the full capsid remain byte-identical to prior PNGs.
- Visually inspect the quality-90 light/dark transparency comparison. Native WebP is lossless
  with alpha. Current changes are checked on macOS arm64; Linux/platform and Fortran oracle
  validation were not repeated for this feature change.
- Student-focused README/capture verification passes the locked release build, executable help,
  public structure downloads, all three illustrated examples, own-protein and larger-image
  commands, and equivalent synthetic PDB/mmCIF/XML renders. Inspect all three quality-90 WebPs;
  `webpinfo` and `git diff --check` pass. Full repository pytest passes: 947 tests. The README
  editorial score is 80 to 96; fresh installation and the Rust test suite were not repeated.
- After the workflow cleanup and removal of the local vendored override, the exact
  `source source_me.sh && pytest tests/` command passes: 945 tests. No Rust code changed;
  Rust checks were not repeated. The master template's current state was not inspected again.
- Initially add a regression covering a tracked deletion with a content-reading callback;
  confirm the existing file is selected. This test is removed with the local vendored override.
- Reproduce the regression's original `FileNotFoundError`, then verify the fix with the exact
  `source source_me.sh && pytest tests/` command: 946 passed. The tracked-file set was larger
  than during the earlier 837-test implementation run; audit the newly included benchmark
  download call as part of the full run.

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
