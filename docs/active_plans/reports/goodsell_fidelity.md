# Native rendering fidelity and scale evidence

## Reproduction and evidence boundary

Run from the repository, after generating the deterministic oracle described in
[goodsell_oracle.md](goodsell_oracle.md):

```bash
./devel/run_reference_oracle.sh
CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo cargo build --release --bin illustrate --example render_passes
source source_me.sh && python3 devel/verify_rendering.py
source source_me.sh && python3 devel/benchmark_rendering.py
```

The verifier calls the production loader and scene preparer. The development-only
[examples/render_passes.rs](../../../examples/render_passes.rs) includes the actual raster,
shading, and outline modules by path; it contains no independent renderer or public debug API.
Every diagnostic composition is checked against the public `render` result. It exports native
mask, shaded RGB, outline opacity, final RGB PNGs, clipped depth/owner TSV, source owner mapping,
and phase timings. PNG/TSV are measured scientific outputs. Quality-90 WebP comparison sheets
are disposable visual evidence, never inputs to numeric comparisons.

The source camera's logged additive center is negated for the native subtractive explicit center.
Image dimensions, source rule colors/radii, shadow/fog parameters, and outline thresholds match.
This deliberately avoids comparing the changed D3 automatic depth centering to legacy centering.
The small fixtures are 112 columns by 96 rows; hemoglobin is 922 by 750.

`output_fidelity/manifest.json` retains exact commands, input/settings/output hashes, production
module hashes, and per-scene metrics. `output_fidelity/spheres_contact.webp` has oracle, native,
and difference amplified eightfold for every small fixture. Each individual scene directory has
`comparison.webp` and separate preview and scientific pass outputs. Five additional deterministic
hemoglobin references isolate plain color, shadow, contour, subunit, and residue behavior. Their
complete cards, PDB input, Fortran logs, P3 pixels, and depth maps are under
`output_oracle/deterministic/hemoglobin_*/`. The oracle generates these while its temporary
executable exists; the native verifier only reads the persisted evidence. Regeneration now covers
19 scenes in each source variant (38 reference records), each rendered twice. The launcher and
all 19 native comparisons passed with the old retained deterministic executable deleted; all prior
reference hashes and native numeric metrics remained unchanged.

## Numeric fidelity on 2026-11-11

All 19 scenes have identical occupied/background masks. All 13 small fixtures have identical
atom ownership. Their maximum common foreground depth error is 0.0000128 pixel (flat scene:
0.00000035 pixel), consistent with source single precision versus native double precision.

| Scene | Different RGB pixels | Maximum channel error, out of 255 |
| --- | ---: | ---: |
| Plain, shadow, contour 1/2/3/4, flat depth, subunit | 0 each | 0 |
| Fog | 9 | 1 |
| Residue | 71 | 1 |
| Combined | 1 | 1 |
| Clipped | 3 | 1 |
| Edge | 1 | 1 |
| Hemoglobin, complete | 900 / 691500 | 253 |
| Hemoglobin, plain | 1 | 26 |
| Hemoglobin, shadow only | 38 | 22 |
| Hemoglobin, contour only | 6 | 26 |
| Hemoglobin, subunit only | 2587 | 255 |
| Hemoglobin, residue only | 1314 | 26 |

Hemoglobin retains exactly 4556 spheres. Maximum common depth error is 0.000098533 pixel.
Native sphere indices are first-match-rule sorted while source IDs are retained input order;
the verifier translates both back to input atom indices before comparing ownership. Exactly one
pixel differs in ownership: zero-based row 439, column 602. The source selects HIS B63 CD2,
serial 1555; native selects HIS B63 NE2, serial 1557. Source depth is -383.503998 and native depth
is -383.5039688. The almost tied surface depths and distinct carbon/nitrogen base colors explain
the one plain-pixel difference; it is not a silhouette or placement error.

The larger subunit differences have an explicit semantic cause (D10). The source increments `su`
whenever the current selected chain character changes in input order. HEM records follow polymer
records and repeat author chain A/B/C/D; source therefore treats a heme as a different subunit
from its own chain's protein. Native stable structural chain identity joins them. Isolated subunit
comparison shows the resulting removed internal black arcs. The complete image differs at only
0.1302% of pixels, with mean absolute channel error 0.043558 out of 255. The subunit-only mean is
0.333794. Residue ramps and shadows retain small single-precision byte-rounding differences.
This is source-matched geometry with an intentional identity correction, not byte-identical
legacy rendering. Independent visual acceptance remains separate from these measurements.

## Local release scale measurements

The benchmark uses the public release CLI, fixed 800 by 800 output, default native fitted camera,
default shadows/outlines, and three sequential runs per input. These are absolute local elapsed
times and macOS maximum resident set sizes, not a Fortran speedup claim. `/usr/bin/time -l`
requires access to `sysctl kern.clockrate`; its sandbox-blocked first attempt was discarded, and
the actual measurement was rerun with the necessary sandbox permission. Hardware metadata and
complete time output are retained with the benchmark manifest and scene directories.

Measured hardware: Apple M4 Max, 14 CPU cores, 38,654,705,664 bytes RAM (36 GiB), macOS 27.2.1
(26B131), arm64, Rust 1.99.1. Final retained measurements:

| Input | CLI wall seconds, three runs | Maximum RSS bytes | Source atoms | Native spheres | Operator instances |
| --- | --- | ---: | ---: | ---: | ---: |
| Hemoglobin | 0.31 / 0.31 / 0.31 | 40,255,488 | 4779 | 4558 | 1 |
| 1LMB protein/DNA | 0.27 / 0.25 / 0.25 | 33,587,200 | 2334 | 2194 | 1 |
| 1CWP assembly 1 | 0.51 / 0.51 / 0.53 | 77,316,096 | 3784 | 227040 | 60 |

The default hemoglobin style has 4558 spheres; the matched source palette intentionally has a
different 4556 selection. Assembly 1 is explicitly selected by CLI `--assembly 1` and by probe JSON.
Its 227040 spheres equal 3784 source atoms times 60 operators, confirming actual expansion.
The mmCIF assembly metadata describes 360 polymer chain copies: three protein and three short
RNA chains per icosahedral asymmetric unit, repeated 60 times.

| Input | Load ms | Prepare ms | Public render ms | Raster alone ms |
| --- | ---: | ---: | ---: | ---: |
| Hemoglobin | 12.11 | 0.82 | 295.28 | 9.84 |
| 1LMB | 7.46 | 0.39 | 239.27 | 6.98 |
| 1CWP assembly 1 | 11.81 | 60.85 | 433.64 | 31.77 |

These phases are a separate diagnostic run. Diagnostic-pass time includes TSV I/O and is not a
production shading measurement. Raster is 3.3%, 2.9%, and 7.3% of public render time respectively.
The current raster traverses each sphere once, already removing source-style repeated full atom
scans per appearance type. Persistent tile binning or threading is not justified by these measured
workloads; defer it until a larger measured workload demonstrates a raster bottleneck. No speed
promise or extrapolation beyond the measured inputs follows from these local timings.

The other benchmark inputs are official RCSB PDB data:

- [1LMB](https://www.rcsb.org/structure/1LMB): lambda repressor/operator complex. Downloaded mmCIF
  explicitly lists two DNA polymer entities and a protein entity, not merely a protein named
  after a nucleic acid interaction.
- [1CWP](https://www.rcsb.org/structure/1CWP): cowpea chlorotic mottle virus. Its mmCIF calls
  assembly 1 a complete icosahedral assembly, using operators 1-60 on label chains A-F.
  The benchmark must explicitly select `--assembly 1` and verify expansion counts; deposited
  asymmetric-unit rendering is not evidence of assembly scale.

The first real-input attempt exposed unnecessary exact floating-point checks in synthetic
mmCIF reserialization for 1LMB atom 8 and 1CWP atom 200. The implementation owner removed that
redundant projection in D11; all final measurements above use the corrected loader with no
coordinate or parser diagnostics for either CIF. No Linux or GPU measurement was performed.

Input download SHA-256:

```text
1LMB.cif d7a7078d944fffea2f882a56224ed0c34dcec1d3867e3ad8d27c903a7a9f8e21
1CWP.cif 7b077300a5d9443d587fefaa331d300fd9050e17e461d645b217ca465f3e0425
```

## Focused verification

The temporary behavior check `tests/_temp/test_render_evidence.py` perturbs RGB, mask, ownership,
and depth independently and verifies their measured differences (one test passed). Pyflakes
passes both Python evidence tools. Diagnostic-versus-production byte equality is checked on
every scene by the probe. These checks do not replace independent visual assessment or a full
repository test suite.

The original synthetic examples are also rendered through every PDB/mmCIF/PDBML input path,
plain and gzip compressed (six inputs), at 128 by 128. The verifier requires exact pixel equality
and records all six PNG hashes and commands. The examples are geometry demonstrations, not
measured biomolecular structures. Their three plain-format default 800 by 800 outputs also
matched exactly in a separate initial check.
