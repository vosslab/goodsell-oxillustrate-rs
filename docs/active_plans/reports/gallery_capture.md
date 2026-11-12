# Poster-inspired gallery capture

Captured 2026-11-12 on macOS arm64 using the current native release binary.
[GALLERY.md](../../GALLERY.md) is linked from the root README and gives a biological caption,
PDB source, scene-file selection, and working native CLI command for every image.

After the user's review of overly dark assemblies, the nucleosome, GroEL/GroES, and rhinovirus
scenes and images were refreshed with contour kernel 3. Their new captures and pass-isolation
evidence are recorded in
[goodsell_readme_capture.md](goodsell_readme_capture.md#transparency-and-contour-correction).
The original capture sizes and repeatability observations below describe the initial kernel-4
versions. No other gallery settings or assets changed during this correction.

## References and selection

Both one-page local PDFs were text-extracted and rendered at a 1920-pixel edge for visual review:

```text
LOCAL_ONLY/2014-mol-mach-poster.pdf
a857f85a1625e785bdeaa5a3e3bce3cc834b1b513c98819b94f9d37b70fe8ca5
LOCAL_ONLY/2002-poster_full.pdf
b9c04c2666aff448e110b5ae5143563e7cdc0d07c40147106c58f1242aabb697
```

These are reference hashes, not distribution dependencies. Both PDFs remain ignored and untracked.
The gallery credits David S. Goodsell/RCSB PDB and links the public
[PDB-101 poster page](https://pdb101.rcsb.org/learn/flyers-posters-and-calendars/poster/molecular-machinery-a-tour-of-the-protein-data-bank).
All gallery images are newly rendered coordinates. Camera framing is independent for each
molecule; the page explicitly avoids implying the posters' common physical magnification.

| Subject | PDB | Poster | Selection and visual purpose |
| --- | --- | --- | --- |
| Human lysozyme | 1LZ1 | Both | Compact single-chain enzyme, warm yellow |
| GFP | 1GFL | 2014 | Chain A only, green barrel |
| Intact antibody | 1IGT | Both | Coral heavy chains, gold light chains, lavender sugars |
| KcsA channel | 1BL8 | 2002 | Assembly 1; view down the pore, alternating blue/lavender subunits |
| Phenylalanine tRNA | 4TNA | Both | Folded RNA in coral, magnesium in gold |
| Nucleosome | 1AOI | Both | Assembly 1; gold DNA around blue histones |
| GroEL/GroES | 1AON | Both | Assembly 1; blue cage and coral cap |
| Rhinovirus 14 | 4RHV | Both | Assembly 1; distinct coat-protein colors across the complete shell |

RCSB entry pages and coordinate metadata were checked for subject names, chain identities, and
assembly choices. For example, antibody A/C are light chains and B/D heavy chains; rhinovirus
author chain IDs are numeric strings 1-4. GFP's deposited partner is deliberately omitted.

## Capture and repeatability

```bash
CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo ./devel/render_gallery.sh
bash -n devel/render_gallery.sh
```

The harness builds the locked release and uses `illustrate --pdb` with eight JSON scenes under
`examples/gallery/`. PNGs remain in ignored `output_gallery/`; temporary PNGs/WebPs are processed
under `/tmp` and removed automatically. Only final quality-90 documentation assets are copied
into `docs/screenshots/`. The script changes images, not gallery prose.

Two complete harness runs succeeded. A second-run hash comparison changed only the potassium
channel, whose camera was deliberately adjusted from [0,0,0] to [0,-21,0] after inspecting the
first render and deriving the pore-axis direction from the subunit centroids. The other seven
WebPs were byte-identical between captures. Final channel output was visually reinspected.

## Image verification

All eight encoded WebPs were viewed. Native PNG non-white bounding boxes stay at least 28 pixels
inside every edge; no molecular outline is clipped. Each WebP is static, decodes without errors,
and is below the 1920-pixel and 1 MB limits. All eight assets total 976736 bytes and have live
gallery embeds. No previously existing screenshot was removed or overwritten.

| Image slug | Dimensions | Bytes |
| --- | --- | ---: |
| gallery_lysozyme | 1200 x 750 | 61784 |
| gallery_gfp | 1200 x 750 | 52122 |
| gallery_antibody | 1200 x 750 | 123598 |
| gallery_potassium_channel | 1200 x 750 | 65894 |
| gallery_trna | 1200 x 750 | 49578 |
| gallery_nucleosome | 1200 x 750 | 121018 |
| gallery_groel_groes | 1200 x 750 | 134046 |
| gallery_rhinovirus | 1600 x 1000 | 368696 |

The additional gallery command using `--background transparent` and native `.webp` output was
executed successfully for the nucleosome. Native application source and vendored files were
read-only for this gallery work. Temporary camera-inspection code was removed after verification.

`bash -n devel/render_gallery.sh` and `git diff --check` pass. The exact
`source source_me.sh && pytest tests/` command passes: 1003 tests in 1.66 seconds. Both local
poster paths pass `git check-ignore`; `git ls-files LOCAL_ONLY OTHER_REPOS` returns no paths.

Not repeated: Rust tests, Fortran-oracle comparisons, Linux execution, or a browser-rendered
documentation review. Actual native renders, image inspection, and repository checks supply the
evidence for this documentation task.
