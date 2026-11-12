# Illustrate development oracle

## Outcome and reproduction

M2 produces original Fortran and explicitly corrected deterministic references in
`output_oracle/`. Run from the repository:

```bash
./devel/run_reference_oracle.sh
```

[run_reference_oracle.sh](../../../devel/run_reference_oracle.sh) only establishes the repository
and Python environment (`source source_me.sh`); functional work lives in
[reference_oracle.py](../../../devel/reference_oracle.py). The script compiles and renders supplied
hemoglobin plus 13 small scenes, twice each, in isolated directories. Each run checks complete P3
images, channel ranges, and repeated image/opacity hashes. The deterministic runs also check repeated
depth/atom hashes, map dimensions, finite nonpositive depths, atom IDs, and occupied pixels.
Fortran is a development oracle dependency; the native renderer has no Fortran runtime requirement.

The generator uses only Python's standard library and an installed `gfortran`. This run uses
`GNU Fortran (Homebrew GCC 16.2.0) 16.2.0`, compiling with `-O0 -std=legacy`. It uses argument vectors
without a shell for compiler and renderer calls (ASVS 1.2.5). It overwrites the same named generated
artifacts on reruns. The repository's `/output*/` ignore rule keeps the large evidence out of Git.

## Source and attribution

The untouched source is `OTHER_REPOS/Illustrate/illustrate.f`, from
`git@github.com:ccsb-scripps/Illustrate.git`, revision
`e469df4f794f8edbca992a9b2ab534d11d27cdd6`.

```text
SHA-256 illustrate.f:
6af0ee0aa2142b72a0fe2655cda371b1ffc17c75f24fb36fab7987fd264fe88b
SHA-256 2hhb.pdb:
b2d9a4a3ec1cf16759c2ffc85f1088ef2100e02ececedf20025b9e54d23f3db6
SHA-256 upstream Apache license / root LICENSE.Apache-2.0:
c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4
```

[LICENSE.Apache-2.0](../../../LICENSE.Apache-2.0) is a byte-for-byte copy of the complete upstream
Apache-2.0 license. [NOTICE](../../../NOTICE) retains David S Goodsell's 2019 copyright, upstream
project/revision, funding acknowledgment, and scientific citations. Existing AGPL-3.0 and CC-BY-4.0
files remain intact; the repository license map identifies their respective covered material.
The source, input, implementation, and algorithm boundaries are documented in
[REFERENCE_SPEC.md](../../REFERENCE_SPEC.md). No website or paper illustration is redistributed.

## Original and deterministic variants

`output_oracle/illustrate_original` compiles the unchanged upstream source. Its output is a
platform/compiler observation, including undefined behavior, rather than a portable pixel contract.
`output_oracle/illustrate_deterministic.f` retains upstream notices and adds a prominent modification
notice. `output_oracle/deterministic.patch` records every source change. Text decoding normalizes
upstream CRLF into LF in the copied variant; the original byte hash above remains the provenance.

The deterministic changes implement agreed D1:

- Extend `type` to include index zero and initialize background `type(0)=0`.
- Initialize radius maximum, centering mode/vector, and previous chain character.
- Initialize all image buffers, including the allocated halo, using background depth -10000,
  atom zero, BIOMT one, and zero color/opacity.
- Clear residue/subunit/contour opacity and each contour candidate before evaluation. Kernels 3/4
  still use the original central neighborhood nine times and the original divide-by-six opacity
  boost. All coefficients, sample strides, thresholds, and arithmetic order remain unchanged.
- Replace fog spread by `max(zmax-zmin,1.e-20)`. For an occupied zero-spread surface this yields the
  front fog retention `pfogh`; background retains weight one. The one-pixel flat scene exercises it.
- Log raw pre-clamp depth extrema and export clamped depth/atom maps before shading.

The final corrected source SHA-256 is
`ba605987419b55145961990fd0b3844a251351cfd841b8aaba56fac7dca0f90f`.

Original and corrected outputs differ in contour 3/4, identity outlines at the allocated halo,
flat fog, combined scenes, and hemoglobin. The original programs repeat identically on this machine,
which does not make their uninitialized scratch/background accesses portable. No native Rust parity
claim follows from these reference images alone.

## Scene settings and coverage

The base generated PDB has four carbon spheres, radius 1.8 Angstrom, RGB .8/.5/.3:

| Atom | Chain | Residue | x | y | z |
| --- | --- | --- | --- | --- | --- |
| 1 | A | 1 | -2 | -2 | -4 |
| 2 | B | 30 | 0 | 0 | -3 |
| 3 | B | 1 | 2 | 1 | -1 |
| 4 | C | 60 | -2 | 3 | -6 |

Defaults are auto-center, 6 pixels/Angstrom, explicit 96 rows/112 columns, white background/fog,
fog retention 1/1, shadows off. Outline ramps use low/high 3/10 (residue 3/8), difference endpoints
0/5 Angstrom, and residue difference 10 when enabled. Disabled ramps use 100000/100001, above any
count in these fixtures. Shadow-on uses decrement .01, slope 2, depth threshold 1, brightness floor .2.

| Scene | Isolated change |
| --- | --- |
| plain | All outlines, fog and shadows disabled |
| shadow | Shadows enabled; otherwise plain |
| fog | Fog retention 1/.2; otherwise plain |
| subunit | Only chain/copy outline enabled |
| residue | Only residue outline enabled; strict difference >10 |
| contour_1 through contour_4 | Only the named contour kernel enabled |
| combined | Kernel 4, identity outlines, shadows and fog enabled |
| flat_depth | One sphere, radius .1; one occupied pixel and zero depth spread |
| clipped | Combined, bounding-box center, translation (0,0,-1); foreground crosses z=0 |
| edge | Combined, auto-center, translation (5,7,0); spheres meet the frame boundary |
| hemoglobin | Supplied ordered 15 selection cards and all supplied settings |

Hemoglobin uses auto-center, scale 12, z rotation 90 degrees, autosize request -30/-30, white
background/fog with fog 1/1, shadow .0023/2/1/.2, contour 3/10/kernel4/0/5, subunit 3/10, residue
3/8/6000. Only the final output filename changes to `image.pnm`. Its actual PPM size is 922x750.
Do not substitute native automatic centering when matching cameras: the reference's `auto` center
mixes a scaled radius maximum with Angstrom coordinates. Its exact center and scale are retained
in each `run_1.log`; use an explicit matched center for comparisons.

## Geometry and output order

Coordinates follow the source: x is image rows/down, y is columns/right, z points toward the viewer.
PPM dimensions are columns then rows. Output traverses x outermost, y next, RGB innermost, so it has
no additional vertical flip. The asymmetric fixtures make row/column transposition visible.

The operation order is BIOMT, source rotation matrix, center vector, user translation, then scale;
entire spheres with transformed centers z>=0 are omitted. Integer sprite offsets are added to
fractional centers and truncated toward zero after floating-point bounds checks. A strict greater
depth wins; final depth is clamped to min(z,0) before shading. Raw min/max are measured before that
clamp; the source caps its reported maximum at zero for fog. Deterministic logs additionally retain
both uncapped extrema as `oracle raw z range`.

Every deterministic scene has `depth_atom.tsv`: first line `width height`, followed by exactly one
`depth atom_id` line per PPM pixel in the same row-major order. Depths are clamped pixel-space z;
background is -10000/0. IDs are 1-based retained-atom indices, including overlaps after first-match
selection; they are not PDB serial numbers. These exports are diagnostic modifications, explicitly
distinct from unmodified upstream output. Header and field whitespace are Fortran list-directed.

## Evidence and limits

The generated `manifest.json` records compiler version, exact compile/render commands, upstream
revision/remote, source hashes, every input/control hash, output dimensions, image/opacity/map hashes,
and repeat results. `commands.inp`, generated `scene.pdb` or copied `2hhb.pdb`, original `image.pnm`,
`opacity.pnm`, and both run logs remain beside each scene. Total generated evidence is about 70 MB.

The final deterministic hemoglobin PPM SHA-256 is:

```text
98ab1d14c7553315c97926bc65d7340daa66960b781020d37ad46c17975f4cce
```

On 2026-11-11, all 28 renders pass two executions with identical hashes. The clipped fixture logs
uncapped occupied depths -20.2000103 through +7.79999924, followed by capped fog maximum zero.
The flat fixture has exactly one occupied pixel at -8.99999905. All 14 deterministic scenes also
pass the bounds/FPE-trap build below, and Pyflakes reports no issues.

The unmodified and deterministic hemoglobin and the small-scene contact sheet were rendered and
visually inspected. Disposable quality-90 WebP views are `output_oracle/spheres.webp` and
`output_oracle/{original,deterministic}/hemoglobin/image.webp`; the measured references are the
original P3 images, not lossy previews. Previews use locally installed Pillow as a one-time viewing
aid; it is not a generator dependency. `cwebp` cannot read the original textual P3 files directly.

An additional deterministic build was run over every scene with bounds and arithmetic traps:

```bash
gfortran -O0 -std=legacy -fcheck=all -ffpe-trap=invalid,zero,overflow \
  output_oracle/illustrate_deterministic.f -o output_oracle/illustrate_checked
for oracle_scene in output_oracle/deterministic/*; do
  (cd "$oracle_scene" && ../../illustrate_checked < commands.inp > checked.log 2>&1) || exit 1
done
source source_me.sh && python3 -m pyflakes devel/reference_oracle.py
```

These checks validate finite deterministic outputs and allocated-buffer bounds for these inputs.
They do not establish arbitrary-input safety or cross-compiler reproducibility. Larger radii,
malformed control cards, unsupported assemblies, empty scenes and repeated calculate commands are
outside this oracle suite. Native comparison is a separate M8 result.
