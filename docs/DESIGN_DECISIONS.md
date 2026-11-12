# Design decisions

<!-- VENDORED HEADER: START -->
Record each durable decision about how this code and repository are shaped, once it is settled, with
the reasoning a later reader needs. Guidance Neil Voss states belongs in
[HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md), dated history in `docs/CHANGELOG.md`, open discussion in
`docs/active_plans/decisions/`. [PROPAGATED HEADER - ENTRIES BELOW ARE YOURS]
<!-- VENDORED HEADER: END -->

Write each decision as a level-three heading with these four fields. `Owner` names the
authoritative code or contract document, rather than a person.

```markdown
### <decision title>

**Decision.** <the durable direction>

**Why.** <the reason it was chosen>

**Consequence.** <the constraint a future change preserves>

**Owner.** <the authoritative code or contract doc>
```

### Native licensing and reference lifetime

**Decision.** Implement the documented mathematics and behavior as new native Rust code under
[LICENSE.AGPL-3.0](../LICENSE.AGPL-3.0), with scientific attribution in
[REFERENCE_SPEC.md](REFERENCE_SPEC.md).
The legacy Fortran is a temporary development reference and comparison oracle.

**Why.** The user's 2026-11-11 amendment specifies an AGPL reimplementation and deletion of the
old code when done. The reference was inspected; this is not a clean-room claim.

**Consequence.** After final fidelity evidence, delete legacy source and compiled copies,
including oracle output copies. Preserve images, hashes, commands, and provenance. Developer
regeneration may fetch a temporary upstream snapshot; end users need no legacy code or Fortran.
If copied upstream code is ever distributed, preserve its applicable license and notices.

**Owner.** The dated amendment in
[goodsell_native.md](archive/goodsell_native.md) and
[Cargo.toml](../Cargo.toml).

### Single-crate root layout

**Decision.** Keep `Cargo.toml`, `Cargo.lock`, and `src/` at the repository root.

**Why.** The user's 2026-11-11 layout preference removes an unnecessary `oxillustrate/` layer
from this single-crate repository.

**Consequence.** Run Cargo commands from the repository root; keep package `oxillustrate` and
binary `illustrate` names. Generated builds use root `target/`.

**Owner.** [Cargo.toml](../Cargo.toml) and
[goodsell_contract.md](active_plans/decisions/goodsell_contract.md).

### Owned synchronous rendering pipeline

**Decision.** Expose `load_structure`, `prepare_scene`, and `render` through the root library,
with owned `Structure`, `Scene`, and `RenderedImage` values. The CLI calls the same pipeline.

**Why.** Separate parsing, molecular selection, and pixel mathematics without requiring callers
to depend on parser-library types or a running application.

**Consequence.** Render local coordinates offline or fetch an entry through the optional RCSB
loading operation. Preserve
author/label namespaces and stable source identity across assembly expansion and scene preparation.

**Owner.** [src/lib.rs](../src/lib.rs) and
[goodsell_contract.md](active_plans/decisions/goodsell_contract.md).

### Deterministic source corrections

**Decision.** D1 initializes background/scratch state and flat-depth fog; D3 uses physical maximum
radius plus one Angstrom for native automatic depth burial.

**Why.** Undefined legacy state and mixed-unit automatic centering cannot define portable native
behavior. Preserve the remaining source equations and operation order.

**Consequence.** Compare oracle depth only with explicitly matched camera centers, and record
corrected-reference provenance separately from unmodified legacy observations.

**Owner.** [REFERENCE_SPEC.md](REFERENCE_SPEC.md) and
[goodsell_oracle.md](active_plans/reports/goodsell_oracle.md).

### Stable structural subunit identity

**Decision.** D10 uses stable structural chain identity for subunit outlines, including a chain's
hetero atoms, instead of the legacy counter that increments when adjacent input chain labels change.

**Why.** Hemoglobin HEM records follow all protein records and repeat chains A-D. The old counter
creates separate subunits for hemes belonging to those same chains, adding internal black arcs.

**Consequence.** Preserve the stable identity correction without a compatibility setting. Matched
hemoglobin geometry remains exact in silhouette; subunit-only output deliberately differs.

**Owner.** [goodsell_fidelity.md](active_plans/reports/goodsell_fidelity.md) and
[src/scene_preparation.rs](../src/scene_preparation.rs).

### Preserve PDBx coordinate precision

**Decision.** D11 converts original PDBx/mmCIF category rows directly through the shared atom-table
path used by XML, removing the synthetic pdbtbx coordinate roundtrip.

**Why.** The roundtrip changed `-3.361` into `-3.3609999999999998` and then rejected valid real
structures as a coordinate-validation disagreement. It also added avoidable projection work.

**Consequence.** Validate the original fields once while preserving their parsed values and full
identity. Real RCSB 2HHB, 1LMB, and 1CWP inputs load; a decimal-coordinate regression covers the
observed failure. Do not reintroduce a second parser projection as validation.

**Owner.** [src/cif_input.rs](../src/cif_input.rs) and [src/atom_table.rs](../src/atom_table.rs).

### Defer unmeasured raster optimizations

**Decision.** Keep the current single traversal of prepared spheres; defer persistent tile binning
and threading until a measured workload demonstrates a raster bottleneck.

**Why.** On the measured 800 x 800 hemoglobin, protein/DNA, and expanded 227040-sphere virus
assembly, raster takes 2.9-7.3% of public render time. Added indexing/concurrency does not address
the dominant measured work.

**Consequence.** Retain reproducible phase timings and revisit this choice when larger workloads
justify the implementation and maintenance cost; make no general performance extrapolation.

**Owner.** [goodsell_fidelity.md](active_plans/reports/goodsell_fidelity.md).

M12 evidence: raster accounts for 2.9-7.3% of measured public render time, so the approved plan
now explicitly defers tile binning and threading until a measured workload justifies them.

### Transparent defaults and native image formats

**Decision.** Background is an optional RGB value: absent/null is transparent and an explicit
color is opaque. One `--background transparent|white|black` option overrides JSON. Keep RGB pixels
and optional straight alpha in the owned image; choose PNG or lossless WebP by output extension.

**Why.** Slide and poster reuse benefits from transparency. Optional color preserves existing
RGB scene files without separate color/opacity settings. The existing image crate supplies both
encoders; lossless WebP preserves the rendered pixels without a quality setting or native libwebp.

**Consequence.** Atom pixels are opaque. Outline-only pixels carry black and outline coverage as
alpha, rather than a white matte. Both encoders share atomic replacement and buffer validation.
Rust callers provide `Some(rgb)` for opaque backgrounds and `alpha: None` for opaque RGB images.

**Owner.** [settings.rs](../src/settings.rs), [render.rs](../src/render.rs), and
[rendered_image.rs](../src/rendered_image.rs). Encoder capabilities:
[image WebP encoder](https://docs.rs/image/latest/image/codecs/webp/struct.WebPEncoder.html).

### Direct RCSB coordinate loading

**Decision.** `--pdb ID` and `load_pdb` fetch mmCIF from a fixed RCSB HTTPS endpoint. Use the
synchronous `ureq` client, verified TLS, no redirects, a 60-second timeout, and a 1 GiB body limit.
Parse in memory using the existing CIF adapter; write no coordinate cache.

**Why.** A known accession should reach a molecular image without curl. mmCIF preserves modern
identifiers and assembly operations. This single-entry workflow needs no async runtime, arbitrary
URL support, cache policy, or retry controls.

**Consequence.** Validate legacy and extended accession syntax before networking. Report transport,
HTTP, and parse causes with the accession; leave existing output intact on failure. Local input
remains offline. The vossvolvox-rust Python downloader was a read-only behavior reference.

**Owner.** [pdb_download.rs](../src/pdb_download.rs) and
[command_line.rs](../src/command_line.rs). Endpoint contract:
[RCSB file downloads](https://www.rcsb.org/docs/programmatic-access/file-download-services).
