# Illustrate reference specification

## Scope and source precedence

This is the mathematics and behavior reference for the native Rust Goodsell renderer. It separates executable
behavior from explanatory prose and identifies source defects instead of silently treating them as
intended mathematics. Line numbers below refer to the commit-pinned upstream
[illustrate.f](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/illustrate.f). Its executable arithmetic takes precedence over
its comments, the website, and the papers when they disagree. D1, agreed during M1, defines
deterministic corrections for undefined or out-of-bounds Fortran behavior: initialize the background
halo and `type(0)`; reset per-pixel and kernel 3/4 contour accumulators; and guard flat-depth fog.
D3, agreed during M1, replaces the legacy double-scaled automatic-depth radius with the physical
maximum radius in Angstroms plus one. Other source formulas remain intact unless stated below.
Exact byte equivalence to undefined Fortran state is not a target.

The native renderer is a new implementation under [LICENSE.AGPL-3.0](../LICENSE.AGPL-3.0).
The inspected Fortran is a temporary development oracle and goal reference. After final fidelity
comparisons, delete its source and compiled copies while preserving images, hashes, commands, and
scientific provenance. This specification records scientific attribution and upstream provenance.

The full textual content of these development snapshots was read. Links below identify durable
upstream sources; local filenames describe the inspected copies, not distribution dependencies:

| Source | Provenance and role |
| --- | --- |
| [nihms-1538287.pdf](https://doi.org/10.1016/j.str.2019.08.011) | Goodsell, Autin, Olson, *Illustrate: Software for Biomolecular Illustration*, Structure 27 (2019), 1716-1720.e1; DOI 10.1016/j.str.2019.08.011. All 12 manuscript pages, including captions and references. Main method authority. |
| [bioinformatics_36_9_2909.pdf](https://doi.org/10.1093/bioinformatics/btaa009) | Tomasello, Armenia, Molla, *The Protein Imager*, Bioinformatics 36 (2020), 2909-2911; DOI 10.1093/bioinformatics/btaa009. All three pages. Comparison, not Illustrate implementation. |
| [overview.pdf](https://doi.org/10.1093/bioinformatics/btaa009) | Ten-page *The Protein Imager* interface supplement; referenced figures, representation table, and complete extracted text read. Comparison only. |
| [index.html](https://ccsb.scripps.edu/illustrate/) | Saved https://ccsb.scripps.edu/illustrate/ homepage; attribution, history, limitations. |
| [documentation.html](https://ccsb.scripps.edu/illustrate/documentation/) | Saved https://ccsb.scripps.edu/illustrate/documentation/; command descriptions and hemoglobin example. |
| [illustrate.f](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/illustrate.f) | Complete 956-line April 2019 simplified non-photorealistic Fortran program, including both matrix subroutines. |
| [README.md](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/README.md) | Complete upstream command documentation and attribution. |
| [LICENSE](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/LICENSE) | Complete upstream Apache-2.0 license. |
| [2hhb.inp](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/2hhb.inp) | Complete executable sample scene, distinct from illustrative comments. |
| [2hhb.pdb](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/2hhb.pdb) | Complete 5,613-record file scanned; 4,384 ATOM and 395 HETATM records, chains A-D, identity biological assembly 1. |

PDF text was extracted without changing sources:

```bash
pdftotext -layout LOCAL_ONLY/overview.pdf tests/_temp/reference/overview.txt
pdftotext -layout LOCAL_ONLY/bioinformatics_36_9_2909.pdf tests/_temp/reference/bioinformatics_36_9_2909.txt
pdftotext -layout LOCAL_ONLY/nihms-1538287.pdf tests/_temp/reference/nihms-1538287.txt
```

HTML text was extracted with Python's standard `HTMLParser`, excluding scripts/styles, into
`tests/_temp/reference/{index,documentation}.txt`; every PDB record was read with `Path.read_text()`
and counted and assembly/header records inspected. Python ran through
`source source_me.sh && python3`. Text review establishes algorithm content; no claim of visual
inspection of every PDF illustration is made. The papers contain no additional numerical kernel
coefficients beyond the Fortran. Hashes below preserve the identity of the inspected development snapshots.

### Source identity

SHA-256 hashes fix this specification to these snapshots:

```text
76ad613ed4f3d78f6a39893a0fed7230c99687f6d9b583865550336e57ec4637  bioinformatics_36_9_2909.pdf
55f959e44d4120a3c906c01b13a6e659e921da392dafd840704e658ec08bd776  nihms-1538287.pdf
be881d25863ab50a33585dae0b73e580a594fbaa6a0dff922fe9c9b90b3814b4  overview.pdf
cd48480923e8cb4bfa0dd47bbbc848969bc2de23fdab3f5c1ba7594dca9f2e33  documentation.html
82a89c2b7b50454d24e78b7fe1c9223342e5f5a8e4f75c056efcf966c532197e  index.html
6af0ee0aa2142b72a0fe2655cda371b1ffc17c75f24fb36fab7987fd264fe88b  illustrate.f
7439e5f258504193f244e6cf9a7bd11389fce10a62c8faaed10591329c7fce84  2hhb.inp
b2d9a4a3ec1cf16759c2ffc85f1088ef2100e02ececedf20025b9e54d23f3db6  2hhb.pdb
90188fab3543c12e07c696cb85d456dbd2ab4e5a20614b7a5403aaf7c129a129  README.md
c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4  LICENSE
```

## Command and atom interpretation

Commands are matched on the first three characters: `rea,tra,xro,yro,zro,sca,cen,wor,cal,ill`
(lines 129-166). Use `read`, transforms/centering, `world`, `illustrate`, `calculate` in that order.
Unknown commands incorrectly fall through to `ill` in the source; this is not a valid-input feature.
Read cards end at a literal leading `END` (193-203), despite prose saying any non-ATOM card ends them.

Each selection card contains six record-name characters, ten descriptor characters, then a numeric
list starting at column 18: inclusive residue bounds, RGB, and radius in Angstroms. Descriptors match
PDB columns 13-22 exactly except `-`, which matches any character. Record names match columns 1-6;
residue is columns 23-26; coordinates are columns 31-54 as three width-eight decimals
(196-209, 270-284, 341-343). Rules are tested in order. First match wins. A zero-radius match omits
the atom immediately, rather than trying a later card. An unmatched atom is omitted. Insertion and
alternate-location characters participate only insofar as descriptor positions match them; no
occupancy selection or alternate-location deduplication occurs.

Subunit IDs increment on each MODEL record and each change of chain character among retained atoms
(217, 299-306). Thus IDs describe consecutive chain runs, not globally unique chain labels. TER has
no independent effect. The source leaves `chainlast` initially undefined; deterministic parsing must
initialize it or explicitly recognize the first retained atom. The actual ordinal value matters
only through equality. Residue IDs are the parsed integers.

Biological assembly 1 is read from the fixed-column REMARK block (219-263). The chain allowlist is
global; each retained atom is drawn under every stored BIOMT matrix. Matrices are not individually
associated with separate chain lists. Missing BIOMT uses one identity (148-155, 496-497). This is a
legacy assembly parser, not a complete PDB/mmCIF assembly specification. Hemoglobin uses one identity
matrix with chains A,B,C,D, so these limitations do not alter that scene.

## Coordinate and projection contract

Use `H=ixsize` rows and `W=iysize` columns. The origin is upper left, x increases downward,
y rightward, and z toward the viewer (35). PPM dimensions are written `W H` (583). Projection is
orthographic. Scale `s` is **pixels per Angstrom**, contrary to the reciprocal wording in the
Illustrate paper's Sprites paragraph. Translations add; repeated scales multiply (348-351, 395-396).

For each input point `p`, first calculate `b = B*p + t_B` using BIOMT rows (639-644), then apply
`q_j = sum_i b_i * rm(i,j)` (647-649), then center, translate, and scale:

```text
c = s * (q + center_vector + user_translation)
R = s * atom_radius
```

Rotations use `a=-degrees*3.141592/180`, not a full-precision PI constant (361, 373, 385). With row
vectors the inserted matrices are:

```text
Mx = [[1,0,0], [0,cos(a),-sin(a)], [0,sin(a),cos(a)]]
My = [[cos(a),0,sin(a)], [0,1,0], [-sin(a),0,cos(a)]]
Mz = [[cos(a),-sin(a),0], [sin(a),cos(a),0], [0,0,1]]
```

`catenate(rm,M)` computes `rm <- M*rm` in ordinary row/column indexing (902-941). Consequently,
rotation cards are applied last-to-first to row-vector coordinates, as the header warns (32-34).
Centering and translation are applied afterward regardless of their command-card positions.

### Centering and image size

After radii are scaled (507-511), calculate min/max coordinates across all BIOMT copies after
rotation, before scaling or translation (515-544). The legacy source automatic-depth formula uses
maximum scaled radius `Rmax`:

```text
center_x = -(xmin+xmax)/2
center_y = -(ymin+ymax)/2
center_z(auto, legacy) = -zmax-Rmax-1
center_z(center) = -(zmin+zmax)/2
```

The legacy `auto` formula mixes scaled `Rmax` with unscaled coordinate extrema (548-556). Native D3
uses the physical maximum radius `rmax` in Angstroms instead:

```text
center_z(auto, native) = -zmax-rmax-1
```

Keep native operation order unchanged: rotate, center, translate, then scale. D3 changes only the
radius term in automatic depth centering. Raw-depth comparisons against the legacy oracle must use
an explicitly matched camera center to remove the uniform automatic-depth offset. Full scene
projections may intentionally frame bounds that include atom radii.

If either requested dimension is <=0, the source recomputes BOTH dimensions (561-567):

```text
H = trunc(-2*requested_H + 2*Rmax + (xmax-xmin)*s)
W = trunc(-2*requested_W + 2*Rmax + (ymax-ymin)*s)
```

Thus `-30,-30` means 30 pixels padding on each side in addition to the maximum radius. Dimensions
are capped at 3000, then rounded down to even values using integer division (570-575). Explicit
positive sizes are also capped when read (465-466). Autosizing runs only inside enabled centering.

### Sphere rasterization and clipping

For every radius/type, enumerate integer offsets `(u,v)` in
`[-trunc(R)-1,trunc(R)+1]` in x-major then y-major order. Retain offsets with
`sqrt(u*u+v*v)<=R`; surface height is `h=sqrt(R*R-(sqrt(u*u+v*v))**2)` (607-629).
Radii with `trunc(R)>100` abort the source. Retain arithmetic ordering if testing numerical parity.

Draw type index outermost, then atom input index, BIOMT index, then sphere offsets (607, 633-637).
Skip an entire sphere if its transformed center `c_z>=0`. Otherwise:

```text
xf = u+c_x+H/2; yf = v+c_y+W/2
if xf<1 or xf>H or yf<1 or yf>W: skip
row = trunc(xf); column = trunc(yf)
z = h+c_z
if z > depth[row,column]: replace depth, atom ID and BIOMT ID
```

Bounds compare the FLOAT coordinates before truncation (661-675). Ties do not overwrite, so earlier
type/atom/copy wins. Centers retain fractional positions but sphere offsets are integer; this is
not analytic per-pixel ray intersection. The depth buffer starts at -10000, atom ID 0, BIOMT ID 1
(592-600). Background metadata uses subunit=residue=9999 (139-140).

Find minimum occupied depth and maximum depth over the active buffer, then cap the maximum at zero
(701-711). Only afterward clamp every active depth to `min(z,0)` (718-722). This produces a flat
cut face where a front hemisphere crosses zero; it does not retain centers on or in front of the
plane. `zmin` is measured BEFORE clipping. The source does not recompute ownership at the cut face.

## Shadows and depth cueing

The shadow cone is screen-space, direction independent, and computed entirely from the clipped
depth buffer, not atom centers (727-744). For each occupied target pixel `p`:

```text
shadow = 1
for dx=-50..50 step 5:
  for dy=-50..50 step 5:
    q=p+(dx,dy)
    accept q only if 0<q.x<H and 0<q.y<W
    distance = sqrt(dx*dx+dy*dy)
    distance = 10000 if distance>50 or (dx,dy)==(0,0)
    dz=depth[q]-depth[p]
    if dz>rcone and distance*coneangle<dz+rcone:
      shadow -= pcone
shadow = max(shadow, pshadowmax)
```

The distance table is populated at lines 499-506. The sentinel distance, rather than an explicit
skip, is the literal test for outside-circle and self samples. Usual positive cone parameters make
these noncontributors. Sampling includes row/column 1 but excludes row H and column W. Background
targets retain shadow=1. There is no distance weighting, normal, randomized sampling, or adaptive
sample count. `coneangle` is a slope multiplier, not a degree/radian angle. `rcone` is used directly
against pixel depths and is **not multiplied by scale**, despite comments describing Angstroms.
The radius 50 and stride 5 are fixed pixels. `pshadowmax` is a minimum brightness multiplier;
smaller means darker. No Lambert/Phong lighting is performed by this released version.

For fog, `pfogdiff=pfogh-pfogl` (448), `spread=zmax-zmin` (711), and (746-747):

```text
fog_weight = pfogh - (zmax-depth[p])/spread * (pfogh-pfogl)
if depth[p]<zmin: fog_weight=1
```

`fog_weight` weights the molecular color; one means no fog. World input makes negative `pfogl`
positive and clamps background/fog components and fog endpoints only above one (430-444). D1 defines
the flat-depth fog guard exactly: when occupied depth spread is zero, occupied pixels use front
molecular-color weight `pfogh`; background pixels use weight `1`. The original source has no guard
for zero depth spread, so this behavior is a deterministic correction, not a Fortran-defined result.
Empty-buffer behavior remains for the Rust boundary to define before admitting that case.

## Three outline types

All outline calculations include background pixels; outlines can extend outside molecular
coverage. Define `clamp01(t)=max(0,min(1,t))` and `ramp(v,lo,hi)=clamp01((v-lo)/(hi-lo))`.
Threshold pairs require `hi>lo` for meaningful interpolation; Fortran does not validate them.

### Subunit and residue boundaries

Only evaluate when `1<x<H` and `1<y<W` (755-756). The neighborhood `N21` comprises all offsets
`-2<=dx,dy<=2` except the four corners where `abs(dx*dy)==4`. It includes the center.

```text
subunit_count = sum over N21 of
  [subunit(atom[p]) != subunit(atom[q]) OR biomt[p] != biomt[q]]
residue_count = sum over N21 of [abs(residue(atom[p])-residue(atom[q])) > resdiff]
subunit_alpha = ramp(subunit_count,r_low,r_high)
residue_alpha = ramp(residue_count,g_low,g_high)
identity_alpha = max(subunit_alpha,residue_alpha)
```

Square brackets mean 0/1. These are one count per neighbor, not one per failed identity test.
Residue comparison is strict `>`, although comment 485 says "or greater". Residue comparison is
not restricted to same chain or copy (760-773). The condition allows reading one pixel outside
the active image, since the radius is two but border exclusion only one. D1 supplies a background
halo there; preserve the documented condition rather than silently shrinking the valid region.

### Depth contour boundaries

Only evaluate when `2<x<H-1` and `2<y<W-1` (776-777). A 3x3 array `l[ax,ay]`, `ax,ay=-1..1`,
holds candidate responses. Convert the depth-difference endpoints to pixel units once:
`dmin=l_diff_min*s`, `dmax=l_diff_max*s` (715-716). For each candidate, clear its accumulator to
zero under D1. Kernels 1/2 use the shifted center `c=p+(ax,ay)`. Their response is absolute value
of the sum below divided by three (785-803); matrix rows are x offsets and columns y offsets.

```text
kernel 1:
  -0.8 -1.0 -0.8
  -1.0  7.2 -1.0
  -0.8 -1.0 -0.8

kernel 2:
   0.0 -0.1 -0.2 -0.1  0.0
  -0.1 -0.8 -1.0 -0.8 -0.1
  -0.2 -1.0  8.8 -1.0 -0.2
  -0.1 -0.8 -1.0 -0.8 -0.1
   0.0 -0.1 -0.2 -0.1  0.0
```

Kernel 1's weights sum to zero. Kernel 2's weights also sum to zero. Neither kernel uses `dmin`
or `dmax`. Kernel 2 plus shifted candidate centers reaches three pixels from p, so its source
border condition can read one pixel beyond active image bounds. D1 supplies the background halo.

Kernel 3 uses the complete 3x3 neighborhood; kernel 4 uses `N21`. Both use the ORIGINAL `p`,
not shifted `c`, for every candidate (805-830):

```text
response = sum over neighborhood:
  if abs(depth[p]-depth[q]) > dmin:
    min((abs(depth[p]-depth[q])-dmin)/(dmax-dmin),1)
  else: 0
```

For each of the nine candidate responses, calculate `a[ax,ay]=ramp(response,l_low,l_high)`.
Count candidates with `a>0`. If at least six are positive, `contour_alpha=clamp01(sum(a)/6)`;
otherwise `contour_alpha=a[0,0]` (832-844). This divides by **six**, not nine. With cleared
accumulators, kernels 3/4 produce nine identical candidates and therefore a 1.5 opacity boost
whenever their ramp is positive. Preserve this arithmetic as part of D1; changing to shifted
centers or dividing by nine would be a separate design change.

The original kernels 3/4 do **not** initialize `l` before adding, and then overwrite it with the
normalized opacity, causing prior pixels to affect later pixels (811, 825, 832). D1 explicitly
removes that accidental scan-order state; it is not justified as a byte-identical translation.
If `illustrationflag==0`, all outline opacity is zero. Otherwise final
`outline_alpha=max(contour_alpha,subunit_alpha,residue_alpha)` (747-848).

## Color and output order

After shadows, fog, and outlines are calculated, per RGB channel (849-861):

```text
shaded = fog_weight * shadow * base_color + (1-fog_weight)*fog_color
RGB = (1-outline_alpha)*shaded
opacity = max(atom_is_present ? 1 : 0, outline_alpha)
byte = clamp(trunc(RGB*255),0,255)
```

Outlines are black. Background uses type/color index 0, set to the world background color
(438); source `type(0)` is out of bounds: `type` is declared with lower bound 1 (93), unlike
`res` and `su`. Use explicit background type/color 0 in Rust; an initialized Fortran variant
must extend the type array lower bound to 0 before assigning that sentinel. Opacity is
written to a separate `opacity.pnm`, three identical channels (585-588, 879-889). Both outputs
are textual P3, max value 255, row-major x then y, RGB channels innermost (867-890). The `.pnm`
suffix does not imply binary P6. There is no built-in image downsampling or multisample antialiasing;
the contour ramps provide the described gray edge treatment.

## Defaults and hemoglobin scene

Initialized program values are not a complete usable preset: scale=1, identity rotation,
translation=(0,0,0), illustration disabled, `l_low=1`, `l_high=10`, `g_low=350000`,
`g_high=21000`, `l_diff_min=1`, `l_diff_max=50`, sizes=(0,0), subunit/residue background=9999,
and initial color table=(.5,.5,.5) (125-155, 187-192). Shadow parameters, fog endpoints, contour
kernel, residue threshold, subunit thresholds, centering vector/mode, and radius maximum do not
all have initialized defaults. Parenthesized typical values in comments are recommendations,
not assignments. A native preset must populate every required field explicitly.

The actual [2hhb.inp](https://raw.githubusercontent.com/ccsb-scripps/Illustrate/e469df4f794f8edbca992a9b2ab534d11d27cdd6/2hhb.inp) specifies:

| Setting | Value |
| --- | --- |
| Centering | auto |
| Translation | 0,0,0 Angstrom |
| Scale | 12 pixels/Angstrom |
| Rotation | z=90 degrees |
| Background / fog RGB | white / white |
| Front / back fog weights | 1 / 1 |
| Shadow enabled, decrement, slope, threshold, brightness floor | 1, .0023, 2, 1, .2 |
| Requested rows / columns | -30 / -30 (autosize with padding) |
| Contour low / high / kernel / depth endpoints | 3 / 10 / 4 / 0 / 5 Angstrom |
| Subunit low / high | 3 / 10 |
| Residue low / high / number difference | 3 / 8 / 6000 |
| Output | 2hhb.pnm and implicit opacity.pnm |

All 15 rules use inclusive residue range 0-9999 and first-match precedence:

| Priority | Selection | RGB | Radius (Angstrom) |
| --- | --- | --- | --- |
| 1 | HETATM water HOH | .5,.5,.5 | 0 (omit) |
| 2-3 | ATOM hydrogen name patterns `-H--------`, `H---------` | .5,.5,.5 | 0 (omit) |
| 4,7 | ATOM carbon (`-C--------`) restricted to A, then C | 1,.6,.6 | 1.6 |
| 5,8 | ATOM sulfur (`-S--------`) restricted to A, then C | 1,.5,.5 | 1.8 |
| 6,9 | Remaining ATOM in A, then C | 1,.5,.5 | 1.5 |
| 10 | Remaining ATOM carbon | 1,.8,.6 | 1.6 |
| 11 | Remaining ATOM sulfur | 1,.7,.5 | 1.8 |
| 12 | Remaining ATOM | 1,.7,.5 | 1.5 |
| 13 | HETATM `FE` in HEM | 1,.8,0 | 1.8 |
| 14 | HETATM carbon in HEM | 1,.3,.3 | 1.6 |
| 15 | Remaining HETATM in HEM | 1,.1,.1 | 1.5 |

Rules 4-6 occur together, then 7-9; the grouped table does not change their order. Other HETATM
records are unmatched and omitted. Applying the complete ordered sample rules retains 4,556
atoms before BIOMT replication (rule counts: 0,0,0,685,3,381,685,3,381,1448,6,792,4,136,32).
The 6000 residue threshold effectively suppresses internal
residue outlines for this protein; background residue 9999 can still differ by more than 6000.
The README's typical shadow floor .7 differs from the sample's .2, and comments' typical contour
high=8 differs from the sample's 10. Use the actual input values for the oracle.

## Feature and attribution boundaries

The Illustrate paper's STAR Methods (manuscript page 5) supports three outline counts and conical
screen-space ambient occlusion. Its "two-by-two neighborhood" prose describes radius-two vicinity;
the executable primary kernel is the 21-point radius-two neighborhood above, not a 2x2 array.
Figures 1-5 discuss visual simplification, subunit colors, sprites/compositing, and NGL previews.
Images composited in Photoshop and web interface features do not add native Fortran capabilities.

Protein Imager is a separate NGL interface with server-side **PyMOL ray tracing**, not this Fortran
renderer (Bioinformatics sections 2-3). Its six representations, advanced selections, proximity and
B-factor colors, perspective camera, sheen, animation, membrane construction, cloud projects,
superposition, distance labels, VRML2 mesh export, and five rendering styles belong to that system
(overview sections 1-2.6 and Table S1). A "Goodsell-like" coloring option does not establish
identical shadow kernels or an Illustrate backend. None of these features is required by the
Illustrate compatibility contract. The local Illustrate homepage itself lists Protein Imager as
another program with a similar style and warns about lack of CIF support and large-structure limits.

The upstream header (1-22) identifies copyright 2019 David S Goodsell and Apache-2.0. If upstream
code or a modified copy is distributed, preserve its applicable copyright, license, disclaimer,
attribution, and source notices. Apache section 4 requires a license copy with redistributed
derivatives, prominent change notices in modified files, retention of relevant source notices,
and applicable upstream NOTICE content if supplied. This upstream snapshot has no separate NOTICE.
The native Rust implementation uses the existing AGPL license; reference inspection alone is not
a reason to label original Rust source as Apache-licensed or copied Fortran.

D1 initializes the background halo and explicit `type(0)` color/type sentinel, resets per-pixel
scratch and kernel 3/4 contour accumulators, and applies the flat-depth fog guard above. These
corrections replace undefined or invalid source behavior. D3 changes only native automatic-depth
centering as specified above; the original equation remains documented as legacy.

The site requests the scientific citation: Goodsell DS, Autin L, Olson AJ (2019), *Illustrate:
Software for Biomolecular Illustration*, Structure 27, 1716-1720.e1, DOI
[10.1016/j.str.2019.08.011](https://doi.org/10.1016/j.str.2019.08.011). Keep this and Goodsell DS, Olson AJ (1992), *Molecular Illustration in
Black and White*, Journal of Molecular Graphics 10, 235-240, as scientific attribution here.
Funding in source: Damon Runyon-Walter Winchell Cancer Research Fund Fellowship
DRG 972, NIH R01-GM120604, and RCSB Protein Data Bank. The papers/website figures are not relicensed
by the program's Apache license; use them as references, without bundling their text or images as
project assets.

## Remaining verification limits

M1 is source analysis, not rendered parity evidence. M2 must compare an unmodified Fortran baseline
with a minimally initialized variant when undefined state affects results. Native tests must cover
strict thresholds, pixel-coordinate bounds, all three identity/contour masks, clipped spheres,
row/column orientation, transform order, fixed shadow sampling, and the explicit sample values.

D1 defines initialized halo/`type(0)`, accumulator resets, and flat-depth fog behavior. The
implementation contract must still select safe behavior for empty images, invalid threshold pairs,
nonpositive dimensions/scales/radii, unsupported assemblies, and repeated calculation. These are
not defined robustly by the reference. Do not infer additional configurable modes from these gaps.

## Native identity amendment: D10

Native subunit outlines use stable structural chain identity rather than the source's consecutive
chain-run counter. In hemoglobin, HEM records follow polymer records and repeat author chains A-D;
source `su` therefore treats each heme as separate from its own protein chain. Native identity
joins the chain's polymer and hetero atoms. This deliberately removes internal black arcs, with
no legacy compatibility setting. Matched masks/depth and isolated subunit evidence are recorded in
[goodsell_fidelity.md](active_plans/reports/goodsell_fidelity.md). Independent visual acceptance is
separate from this documented mathematical and identity comparison.


## Reference lifecycle amendment

The developer oracle can fetch the pinned upstream source and sample coordinates/control cards,
verifies their recorded SHA-256 hashes, and confines source and executables to an automatically
removed temporary workspace. A clean native source build does not require ignored local references,
Fortran, or network retrieval of those sources. Regeneration commands and persistent evidence are
in [goodsell_oracle.md](active_plans/reports/goodsell_oracle.md).
