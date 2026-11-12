# Image output and PDB downloads

## Goal and decisions

Deliver transparent PNGs, native WebP, and `illustrate --pdb 1lmb --output protein.webp`.
The owner of all implementation, integration, and verification is the current root agent.

- Default to a transparent background for slide/poster reuse. `--background transparent|white|black`
  overrides scene JSON. JSON `background: null` means transparent; an existing RGB triple stays
  opaque, preserving existing scene files. One selector replaces separate background flags.
- Keep `RenderedImage.pixels` as RGB and add optional straight-alpha bytes. Foreground atoms stay
  opaque; outline-only pixels are black with outline coverage as alpha, avoiding a white matte.
- Infer PNG or WebP from the output extension. Retain `save_png`, add `save_webp` and `save`, and
  share existing atomic-save behavior. WebP is lossless through the existing image crate.
- Make a local input and `--pdb ID` mutually exclusive. Download mmCIF over verified HTTPS from
  fixed RCSB endpoints with a fixed timeout and the existing 1 GiB input bound. Accept legacy
  four-character and extended `pdb_` IDs, case-insensitively. Parse in memory; save no input cache.
- The suggested vossvolvox-rust reference has a Python E2E downloader, not a native downloader.
  Reuse its simple fixed-host/validated-ID approach as a reference; do not copy or edit its code.
- No async runtime, batch downloads, cache management, arbitrary URLs, lossy quality controls,
  atom translucency, or new render style. None is required for the requested single-image workflow.

## Work and success conditions

1. **Rendering and encoding:** update settings, scene, shading, render composition, image output,
   and affected examples/tests. Success: PNG/WebP decode with identical RGBA pixels; opaque output
   retains its RGB appearance; transparent outline edges composite correctly on light and dark.
2. **Native input and CLI:** add the synchronous downloader, a contextual download error, and
   CLI source/background selection. Success: a live lowercase `--pdb 1lmb` render works without
   curl; invalid IDs, incompatible sources, bad HTTP responses, and failed saves preserve inputs
   and prior outputs. Offline tests cover actual parsing, bounded reads, and CLI validation.
3. **Documentation and proof:** update README, usage/install/architecture docs, priority status,
   human guidance, decisions, and changelog. Produce actual transparent PNG/WebP examples plus
   light/dark composites. Complete all work here; archive this plan with the final evidence.

## Verification and file scope

Source: root Cargo manifest/lockfile, `src/`, `examples/render_passes.rs`, and
`tests/render_cli.rs`. Documentation: project-owned README/docs and image assets only.
Vendored files and other repositories remain unchanged.

Run formatting, locked all-target check, tests, Clippy with warnings denied, rustdoc, and repository
pytest. Use focused permanent tests for alpha/format integrity, input validation, and output
preservation. Run real RCSB success/not-found requests, compare opaque hemoglobin bytes against
the existing PNG, and visually inspect light/dark composites. A failure reopens its owning step;
correct it and rerun affected checks before completion. No hosted workflows are introduced.

## Status

All three work items are implemented and verified. The current root agent owns the completed
source, CLI/API, documentation, and capture work; no implementation is handed off.

- Rendering/encoding: optional RGB backgrounds default to transparent; PNG/WebP round-trip the
  same RGBA pixels. Integration tests compare composites over white and black and reject matte
  pixels. The quality-90 light/dark comparison is visually inspected. Existing explicit-white
  hemoglobin and capsid PNGs remain byte-identical.
- Download/CLI: actual legacy `1lmb`, extended `pdb_00001lmb`, hemoglobin, and assembly 1CWP
  downloads render. A real 404 preserves existing output. A freshly installed executable renders
  from outside the repository. No external downloader is used.
- Documentation/proof: README and usage commands use `--pdb`, all priorities are marked complete,
  the new comparison is embedded, and temporary capture code is removed. Vendored files and the
  suggested reference repository remain unchanged.

Formatting, locked all-target check, 44 unit tests, 5 integration tests, 1 doctest, Clippy with
warnings denied, missing-docs rustdoc, and the full repository pytest run pass. The pre-archival
pytest run contains 953 tests. This feature change is checked on macOS arm64; no new Linux or
Fortran-oracle claim is made. Final verification is recorded in
[image_output_downloads.md](../active_plans/reports/image_output_downloads.md).
