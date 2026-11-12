# Independent visual assessment

## Acceptance verdict

**PASS for M12 visual acceptance, assessed 2026-11-11.** The native Rust output preserves the
material appearance of Illustrate: space-filling atoms, restrained pastel regions, soft recessed
shadows, depth cueing, and selective black boundaries. No material unexplained departure or
render-pass correction is required in the inspected evidence. Pixel identity is not this verdict's
criterion. The documented D1, D3, and D10 changes remain explicit exceptions to legacy behavior.

This assessment is independent of the implementation and numeric-fidelity work. It changes only
this report. It permits the manager to finish the planned temporary Fortran-source cleanup after
preserving the images, commands, hashes, and provenance required by
[REFERENCE_SPEC.md](../../REFERENCE_SPEC.md).

## Materials and direct inspection

Read repository instructions, Markdown/repository style, [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md),
[goodsell_fidelity.md](goodsell_fidelity.md), the deterministic-correction discussion in
[goodsell_oracle.md](goodsell_oracle.md), and `output_fidelity/manifest.json` and `benchmarks.json`.
Read the relevant identity assignment in `src/scene_preparation.rs` and identity-outline comparison
in `src/outlines.rs` to connect the visible heme change to its responsible pass.

The following actual local images were opened with `view_image`:

- `tests/_temp/reference_review/illustrate-09.png`: manuscript page 9, Figure 2, the strongest
  direct style target and demonstration of the separate outline methods.
- `tests/_temp/reference_review/illustrate-10.png` and `illustrate-11.png`: supplementary visual
  context. Page 10 visibly contains the myoglobin comparison, virus, and membrane-channel views;
  page 11 contains membrane protein, packed proteins, ribosome, and nucleosome views. Their
  printed captions describe the other page's artwork; assessment follows visible content.
- `output_fidelity/spheres_contact.webp`: all 13 matched small-scene oracle/native/difference
  rows, including independent subunit/residue contours and all four depth kernels.
- `output_fidelity/hemoglobin/comparison.webp` and `hemoglobin_subunit/comparison.webp`:
  complete and isolated-subunit oracle/native/difference panels.
- `output_fidelity/hemoglobin/image.png`, `shaded.png`, and `outline.png`: scientific native
  final, shaded-color, and outline-opacity passes at their original 922 by 750 dimensions.
- `output_fidelity/hemoglobin_subunit/image.png`, `hemoglobin_contour/image.png`,
  `hemoglobin_residue/image.png`, and `hemoglobin_shadow/image.png`: isolated native passes.
- `output_fidelity/fog/image.png`: the original small fog fixture.
- `output_fidelity/scale_contact.webp`, `benchmarks/1lmb/image.png`, and
  `benchmarks/1cwp_assembly1/image.png`: native default-style scale examples, including the
  original 800 by 800 complex and expanded assembly images.

WebP sheets support visual navigation, not exact pixel measurements. The independent checks below
read the scientific native PNG and reference P3 images directly with Pillow/NumPy, using the
repository's `source source_me.sh && python3` environment.

## Criterion findings

Observed features and their visual judgments are separated below.

| Criterion | Observed evidence | Judgment |
| --- | --- | --- |
| Shapes and silhouettes | Matched spheres and complete hemoglobin have the same lobed outlines, projecting atoms, central clefts, and overlap order. All 19 manifest masks match exactly. | PASS; no missing volume or placement defect is visible. |
| Cavities and depth | Hemoglobin's central and lower clefts remain dark and open. The isolated shaded pass retains recessed brown/pink regions without requiring black outlines. The protein/DNA complex retains gaps and overlapping components. | PASS; cavities remain readable rather than filled by flat color. |
| Pastel colors | Hemoglobin retains pink and peach protein regions and saturated red selected hemes. The native default examples use related warm pink, peach, tan, and mauve regions. | PASS; the source sample's color selection remains recognizable and the default palette retains the intended family of colors. |
| Shadow distribution | Isolated hemoglobin shadows deepen recesses throughout the molecule rather than forming a consistent lit side or bright specular spots. The small shadow row matches visually. | PASS; the result reads as soft conical occlusion, without glossy highlights. |
| Depth fog | The fog fixture lightens the rear upper spheres relative to the front sphere. Complete hemoglobin retains depth separation and dark cavities. | PASS; no washed-out foreground, fog discontinuity, or visible undefined flat-depth artifact is present. |
| Subunit boundaries | The small subunit row groups same-identity atoms while retaining the outer boundary. Hemoglobin preserves major chain edges, with the specific heme boundary change discussed below. | PASS with intentional D10 identity semantics. |
| Residue boundaries | Isolated residue and contour passes produce visibly different internal edge patterns. The complete image combines selective internal edges with strong silhouettes. | PASS; boundaries remain selective rather than outlining every atom equally. |
| Depth contours | All four small-kernel rows visually agree with their deterministic references, including sparse kernel 3 and stronger kernel 4 at these settings. Hemoglobin contour PNG follows exposed depth steps. | PASS; no unexplained kernel omission or edge displacement is evident. |
| Combined Goodsell style | Complete hemoglobin retains layered atomic patches, soft depth, limited pastel regions, and thin dark separations, consistent with the visual vocabulary of manuscript Figure 2. | PASS; different depicted molecules do not prevent the style comparison. |

## Hemoglobin difference attribution

The strongest visible difference occurs around the red heme groups in the upper-left and
lower-central/right interior of the matched hemoglobin. In the isolated subunit comparison the
reference wraps these groups in black arcs; native output removes those arcs. In the complete
image other contour mechanisms retain some local edges, so the change is smaller and the hemes
remain conspicuous through their red color and depth context.

The responsible mechanism is **subunit identity preparation and the subunit-outline pass**, not
sphere rasterization or shadow sampling. Native chain IDs use the stable instance/author-chain/
label-chain tuple. The source's consecutive-chain-run identity separates later HEM records from
earlier protein atoms in the same author chain. D10 deliberately joins them. This semantic choice
reduces the heme's independent black border in a subunit-only image; it does not erase its atoms,
color, major chain boundaries, or the combined render's cavity structure. I judge this bounded,
documented change acceptable to the requested appearance rather than a material loss of style.

Independent direct-image checks add the following evidence to the manifest:

- The complete hemoglobin native PNG and oracle P3 hashes match their manifest entries. The
  isolated-subunit PNG and P3 hashes also match. All five recorded production module hashes match
  current files, tying these inspected outputs to the measured implementation.
- Complete hemoglobin has 841 pixels with a channel difference greater than one byte, all within
  zero-based rows 195-568, columns 301-623. Every one of those pixels also lies in the isolated
  subunit image's greater-than-one-byte difference support. This directly bounds the stronger
  complete-image departures to the demonstrated identity effect and its shared near-tie pixel.
- The isolated-subunit support has 2,587 changed pixels, within rows 194-572, columns 300-623.
  It does not extend to the outer silhouette. The isolated view visibly locates the removed arcs
  at the two exposed red heme regions.
- The manifest records 900 complete-image changed pixels out of 691,500, maximum channel error
  253, and mean absolute channel error 0.043558. A low mean alone would not explain the change;
  the isolated pass and spatial support provide that explanation.
- The manifest records exact masks and maximum common depth error below 0.000099 pixel for
  hemoglobin. The documented one ownership mismatch is a nearly tied HIS B63 atom surface at
  row 439, column 602, with different atom colors. I did not independently rerun depth/owner
  calculations; their provenance and cause are recorded in [goodsell_fidelity.md](goodsell_fidelity.md).

## Corrections and scope boundaries

No visual must-fix is identified. D1's initialized halo/background type, cleared per-pixel and
kernel 3/4 accumulators, and flat-depth fog guard correct undefined source behavior. The inspected
matched images use the deterministic source with these corrections; they do not establish byte
equivalence to unmodified undefined-state Fortran. The source-oracle report records differences
from that unmodified baseline separately. I did not reinterpret those artifacts as an intended
requirement or claim to have independently inspected every original-baseline image.

D3 uses physical radius plus one for native automatic burial rather than the legacy double-scaled
radius. Matched scenes use the documented explicit matched camera, so their close agreement does
not independently validate D3's automatic-depth equation. Native fitted-camera scale images show
coherent, complete silhouettes and usable depth treatment.

The 800-pixel expanded virus image is visually dense: closely spaced black contours reduce the
readability of individual atoms. Its large subunit color regions and complete roughly spherical
outline remain legible. This is an output-scale presentation limit, not evidence of a native
departure from a matched reference; no reference render for this assembly was supplied. For a
teaching figure requiring individual-atom detail, use a closer view or a larger exported image.
No renderer change is required by this observation.

The benchmark manifest records 60 assembly instances and 227,040 expanded spheres from 3,784
source atoms. Those counts support the scale example's scope. Timing/RSS results are contextual,
not visual acceptance criteria; this assessment neither reruns benchmarks nor makes a GPU claim.
Persistent tiles or threads are not needed to remedy any observed visual finding.

## Limits and recommendation

Accept M12 and retain this report with the numeric evidence before planned reference-source
cleanup. No new compatibility option, rendering pass, dependency, or visual tuning is recommended.
Protein Imager sheen, transparency, ribbons, and surfaces fall outside this Illustrate assessment.

This is direct visual review of the named images plus focused provenance/spatial checks. It is
not an all-input image survey, external user study, full test-suite run, new performance experiment,
or independent regeneration of the Fortran/native outputs. Disposable output images must remain
available separately if later reviewers need the exact visual evidence after source cleanup.
