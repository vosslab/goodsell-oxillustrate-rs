# Native crate contract

## Ownership and public boundary

The standalone package [Cargo.toml](../../../oxillustrate/Cargo.toml) builds library `oxillustrate`
and binary `illustrate`, using Rust 2024 and toolchain floor 1.99.1. Safe synchronous Rust is
required. Private modules expose owned domain types through `lib.rs` re-exports. Parser library
objects never escape adapters. `load_structure`, `prepare_scene`, and `render` are the intended
three operations; M3 defines their data contracts, while later work implements and wires them.
The initial binary provides help/version and rejects unknown arguments; it does not render yet.

## Molecular data

`Structure` owns atoms, assembly definitions, a `BTreeMap<String, Transform>` of operations, and
text diagnostics. `Atom.source_id` is the stable source identity, distinct from dense render
indices. Every atom retains coordinates (Angstrom), element, effective name, both optional
namespace names, model, occupancy, alternate location, kind, and its `ResidueId`.

`ResidueId` preserves optional author/label chains and residue names, raw author sequence text,
optional parsed author number, optional label number, and insertion code. Effective names prefer
author names where present. Missing namespace information remains absent; PDB provides author
identity. Arbitrary chain strings are preserved. `AtomKind` distinguishes polymer, hetero, water.

An assembly generator stores its chain namespace, targeted chains, and original operation
expression. Assembly code resolves that expression without discarding it. Transform rotations are
row-major matrices acting on column vectors: `p' = R*p+t`. Composition must preserve the source
expression order; translations are Angstrom. Deposited coordinates are the default.

## Settings and defaults

All setting objects reject unknown JSON fields. Partial settings use explicit defaults:

| Field | Default / meaning |
| --- | --- |
| width, height | 800 x 800 pixels |
| model, assembly | first model; deposited coordinates |
| orientation | x/y/z source-card degrees, all zero, effective application Z then Y then X |
| translation | zero Angstrom, after rotation and centering |
| framing.center | absent: automatic rotated x/y bbox center and physical zmax+maxradius+1 burial |
| framing.scale | absent: fit sphere bounds; supplied value is positive pixels/Angstrom |
| framing.margin | 30 pixels per side for automatic scale |
| background | white RGB in [0,1] |
| atom_rules | ordered, first match wins; empty uses native radius/palette and visibility defaults |
| fog | white; front/back molecular weights both 1 (no fog) |
| shadows | enabled; decrement .0023, slope 2, pixel threshold 1, brightness floor .2 |
| outlines | three independently enabled passes; kernel 4 |
| contour | thresholds 3/10; depth endpoints 0/5 Angstrom |
| subunit | thresholds 3/10 |
| residue | thresholds 3/8; residue-number difference 6000 |

An explicit center is subtracted in rotated coordinates and replaces automatic z burial.
Translation follows centering, then scale. D3 uses physical radius for automatic burial, correcting
the source's mixed-unit double scaling. Oracle scenes supply an explicit center to reproduce it.

Rules match optional element, namespace-qualified chain, effective residue/atom name, namespace
qualified inclusive numeric residue range, and kind. Missing namespace defaults to author.
Optional rule color, positive radius, and omit modify base appearance. `omit: false` explicitly
restores atoms hidden by native defaults; absent omit preserves default visibility. Native defaults
hide H, D and water, retain other hetero atoms, and use the approved restrained chain palette.

Validation rejects nonfinite numeric parameters, out-of-range colors/weights, invalid thresholds,
nonpositive dimensions/scale/radii, reversed residue ranges, empty assembly IDs, and nonpositive
model numbers. Selection existence and finite coordinates are checked at their owning input/scene
boundaries. A zero fog spread uses front weight for occupied pixels and weight one for background.
An all-clipped or offscreen image returns `EmptyScene`.

## Render contracts

`Scene` owns dimensions, spheres, background, fog/shadow/outline settings, pixels per Angstrom, and
text diagnostics. `Sphere.center` is `[row down, column right, z toward viewer]` in pixels,
including viewport offset. This preserves Illustrate x=row/y=column. Sphere radius is pixels;
color is [0,1]. It retains source atom index and assembly instance independently, structural
residue/chain IDs, and an optional numeric residue for the strict residue-difference pass.

The raster owner supplies a buffer with depth and optional dense sphere index for each pixel.
Larger z wins; a deterministic finite background sentinel starts at -10000 and moves farther only
when necessary. Save raw occupied extrema before clamping surface depth to zero for clipping/fog.
`RenderedImage` contains width, height, and top-to-bottom row-major RGB bytes.

Checked dimension multiplication rejects more than 16,777,216 pixels (approximately 400 MiB of
scratch at the planned buffers). Every actual allocation still uses fallible reservation and
checked byte arithmetic. `Error` distinguishes contextual file I/O, format parsing, invalid
settings/selections, empty scenes, allocation failure, and image encoding; error causes remain
available through standard error sources without exposing dependency-specific types.

## Dependency refresh evidence

On 2026-11-11 official docs.rs crate pages reported clap 4.6.7, serde 1.0.229, serde_json 1.0.154,
pdbtbx 0.12.0, quick-xml 0.42.0, image 0.25.10, flate2 1.1.11. Manifest floors use open-ended `>=`;
Cargo.lock records the exact resolution. Unneeded default features are disabled; image enables
PNG only and flate2 selects its Rust backend. pdbtbx requires its compression feature because
version 0.12.0 has unconditional gzip exports/imports that fail without it; other defaults remain
disabled. Cargo checks use writable
`CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo` in this managed environment.
