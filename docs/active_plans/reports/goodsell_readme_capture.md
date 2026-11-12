# Molecular illustration documentation captures

The later DPI/content-trimming implementation regenerates these images with three pixels of
padding. Current dimensions, hashes, and verification are in [print_trim.md](print_trim.md).
The dated receipts below describe the earlier captures.

## Transparency and contour correction

The user's 2026-11-12 review identified two misleading examples. The asset named
`transparent_backgrounds.webp` was an opaque light/dark comparison, and the whole-capsid render
had excessive black contours. The following changes supersede the earlier capture receipts.

- Replace the comparison with actual native lossless RGBA WebP, rendered with
  `--pdb 2hhb --scene examples/hemoglobin.json --background transparent`. Transparent pixels
  expose the browser's page background. There is no baked-in color, checkerboard, or label.
- Use `"outlines": {"kernel": 3}` in the cowpea capsid, rhinovirus, GroEL/GroES, and nucleosome
  scenes. This changes only the existing contour neighborhood from 5 x 5 to 3 x 3. Subunit
  boundaries and conical shadows remain enabled; molecular geometry, colors, camera, source
  mathematics, global defaults, and the explicit hemoglobin reference scene are unchanged.

Matched 1600 x 1000 renders of local 1CWP assembly 1 isolate the cause. Measurements use Pillow's
8-bit grayscale values over one fixed foreground mask derived from the original opaque render;
near-black means a grayscale value below 32. The original native PNG reproduces its prior hash.

| Single change | Mean grayscale | Near-black foreground |
| --- | ---: | ---: |
| Original settings | 106.73 | 25.33% |
| Shadows disabled | 114.14 | 24.65% |
| Contours disabled | 149.77 | 4.97% |
| Shadow floor raised from 0.2 to 0.7 | 107.10 | 25.24% |
| Contour kernel changed from 4 to 3 | 146.91 | 5.17% |

The contour pass is the main source of darkness in this image. Raising the shadow floor is
insufficient. Kernel 3 retains subtle atom contours and the subunit boundaries while avoiding
the dense black texture. Contact sheets for these pass comparisons and three other large
assemblies remain in ignored `output/style_review/`.

Fresh native `--pdb` captures succeeded for all five changed images. Four opaque documentation
images use `cwebp -q 90` under `/tmp`; the transparency asset is the native encoder's exact output
so the documentation directly demonstrates alpha preservation. All five final assets were viewed.

| Asset slug | Dimensions | Bytes | Encoding |
| --- | --- | ---: | --- |
| transparent_backgrounds | 922 x 750 | 410430 | Native lossless RGBA WebP |
| cowpea_capsid | 1600 x 1000 | 249620 | Quality-90 RGB WebP |
| gallery_rhinovirus | 1600 x 1000 | 243904 | Quality-90 RGB WebP |
| gallery_groel_groes | 1200 x 750 | 80078 | Quality-90 RGB WebP |
| gallery_nucleosome | 1200 x 750 | 83822 | Quality-90 RGB WebP |

Current WebP SHA-256 values:

```text
transparent_backgrounds 14b39b553c3cfee5170a82681ea06922460bb8002dfa1d51ad14b3a6321c1248
cowpea_capsid f8e843a64e64b4b5ec8f348ddec58e178d3c91c21f7e6735fdd4f2db038d9c8f
gallery_rhinovirus 3de47cda9de2ca5fa51092e3518f2801df5d76e87263c93b8d70fe2b99ea477c
gallery_groel_groes a82d7fd249d65b37693d7a98154a53c942c228a894eb21b3aa9b72d6c4f88565
gallery_nucleosome 31d635bc3f0890f90da246b3c5f1dd3d027c13f957e86962f3146092c0642a4f
```

Verification: the current native PNG and documentation WebP decode to identical RGBA bytes;
their alpha includes transparent, partial-coverage, and opaque pixels. Corrected native renders
match the local contour-ablation captures exactly, with clear borders. Three temporary capture
checks passed and were removed. `webpinfo` reports no errors for all five images. The locked
release build and all 50 Rust tests pass; `source source_me.sh && pytest tests/` passes all 1003
repository checks. Shell syntax and `git diff --check` pass. No Rust or vendored files changed.
Browser layout, Linux execution, and Fortran-oracle comparisons were not repeated.

## Native feature update

The later 2026-11-12 implementation adds native `--pdb` downloads, transparent backgrounds,
and PNG/lossless WebP output. Current README/usage commands use these features without curl.
The original gallery-capture receipt below describes the earlier run, including its external
quality-90 conversion for documentation assets. Hemoglobin and capsid opaque PNGs were reproduced
byte-for-byte by the updated renderer and live downloader. Gallery commands explicitly request
white backgrounds to preserve those existing views.

The initial `transparent_backgrounds.webp` compared the same native RGBA hemoglobin over white
and dark slide colors. That opaque 1600 x 1000 comparison was misleading under this filename;
the correction above replaces it with actual alpha output. Native PNG and lossless WebP source
pixels match exactly. This fourth asset has a live usage-guide embed.

## Original gallery capture

Capture date: 2026-11-12; receipt timestamp: 06:37 UTC. Root package `oxillustrate` 26.11.0;
macOS arm64, Rust 1.99.1. Skills: `readme-docs` and `screenshot-docs`.

The README now addresses a biology master's student making a protein figure. Its first result
is an image of real hemoglobin, followed by using the student's own coordinates and changing
size, assembly, colors, or view. Build internals and validation reports stay in supporting docs.

## Capture route and assets

`REPO_TYPE` is `rust`; the application is a CLI that produces images. Capture uses its actual
PNG output, converted directly to static WebP with `cwebp -q 90`. No molecular image is synthesized,
retouched, cropped, or composited. Static images demonstrate the result without animation.

| Asset | Dimensions | Bytes | Live embed |
| --- | --- | ---: | --- |
| [hemoglobin.webp](../../screenshots/hemoglobin.webp) | 922 x 750 | 111788 | README |
| [lambda_dna.webp](../../screenshots/lambda_dna.webp) | 960 x 600 | 42230 | Usage guide |
| [cowpea_capsid.webp](../../screenshots/cowpea_capsid.webp) | 1600 x 1000 | 336398 | Usage guide |

All three images were freshly rendered and visually inspected after WebP encoding.
Hemoglobin retains the existing image's bytes. The new views show lambda repressor bound to DNA
and assembly 1 of cowpea chlorotic mottle virus. Titles and assembly definitions are present in
the downloaded coordinate files. The capsid is rendered at higher resolution so its atomic
surface remains legible. Every asset is below the 1920-pixel edge and 1 MB budgets.

## Reproduce the native outputs

Run from the repository root. The public RCSB coordinate downloads succeeded during capture:

```bash
cargo build --release --locked
mkdir -p output
curl --fail --location https://files.rcsb.org/download/2HHB.cif --output output/2hhb.cif
curl --fail --location https://files.rcsb.org/download/1LMB.cif --output output/1lmb.cif
curl --fail --location https://files.rcsb.org/download/1CWP.cif --output output/1cwp.cif
./target/release/illustrate output/2hhb.cif --scene examples/hemoglobin.json \
  --output output/hemoglobin.png
./target/release/illustrate output/1lmb.cif --output output/lambda_dna.png \
  --width 960 --height 600
./target/release/illustrate output/1cwp.cif --assembly 1 --output output/cowpea_capsid.png \
  --width 1600 --height 1000
```

These commands use the existing application as the capture harness. No separate script or
application changes are needed. Cargo used the existing local dependency cache; downloads used
a 60-second timeout. This is an actual local build/render check, not a clean installation test.

Coordinate SHA-256 values:

```text
2hhb.cif d41ec279c2fb8e9b53e6ab4c1e6ecca52e20346a3f3808a6d31d9e8307dd3810
1lmb.cif d7a7078d944fffea2f882a56224ed0c34dcec1d3867e3ad8d27c903a7a9f8e21
1cwp.cif 7b077300a5d9443d587fefaa331d300fd9050e17e461d645b217ca465f3e0425
```

Rendered PNG SHA-256 values:

```text
hemoglobin.png e87db3684062dfbd19fc37e666c860de7e5729581b35e9334875a230488b85c2
lambda_dna.png 748b53b2de066cd07005e57e373bc2e174f2bb438c442e1deaa689b40419e9d6
cowpea_capsid.png 927185a122b27df3e3cfb0e681cd0a90b812e9d75519c4dc66ca113170bf9ee3
```

## Refresh documentation images

`cwebp` is a documentation-only prerequisite. Process temporary PNGs under `/tmp`, then copy
the encoded assets into the managed folder:

```bash
mkdir -p /tmp/goodsell_readme
cp output/hemoglobin.png output/lambda_dna.png output/cowpea_capsid.png /tmp/goodsell_readme/
cwebp -q 90 /tmp/goodsell_readme/hemoglobin.png -o /tmp/goodsell_readme/hemoglobin.webp
cwebp -q 90 /tmp/goodsell_readme/lambda_dna.png -o /tmp/goodsell_readme/lambda_dna.webp
cwebp -q 90 /tmp/goodsell_readme/cowpea_capsid.png -o /tmp/goodsell_readme/cowpea_capsid.webp
cp /tmp/goodsell_readme/hemoglobin.webp docs/screenshots/
cp /tmp/goodsell_readme/lambda_dna.webp docs/screenshots/
cp /tmp/goodsell_readme/cowpea_capsid.webp docs/screenshots/
```

Final WebP SHA-256 values:

```text
hemoglobin.webp 8a22f475a662de06a0115c03e22fc3470d245507f51b88c07829dcda143b382d
lambda_dna.webp d88308fcfffc278c44c41101eef011a2d272274b85f8367aa8dd8213a7480f4d
cowpea_capsid.webp 5d29c5bbb0eacd655faf70daef36cfdf07c2725a42d8df72c31bd291b2468582
```

Managed blocks in [README.md](../../../README.md) and [USAGE.md](../../USAGE.md) retain their
exact sentinels, descriptive alt text, and document-relative paths. Each of the three managed
assets has a live embed; there are no unused managed or `reference_` images to prune.
Temporary PNG copies are removed after final inspection; generated native PNGs remain under
ignored `output/`. No vendored or master-template files are changed.

## README editorial review

The score is an editorial assessment for the student's task, not a product validation score.

| Category | Before | After | Evidence |
| --- | ---: | ---: | --- |
| Purpose, audience, value | 11 | 15 | Opens with making figures for slides, posters, and reports |
| Project identity | 15 | 15 | Goodsell appearance, hemoglobin example, and scientific citation |
| Proof and demonstration | 15 | 20 | Fresh hemoglobin plus linked protein-DNA and capsid examples |
| First success | 8 | 11 | Real protein and expected PNG replace the synthetic four-atom start |
| Usage orientation | 8 | 10 | Explains chain colors, hidden atoms, and deposited versus assembly views |
| Navigation | 8 | 10 | Four task-oriented routes replace the developer evidence index |
| Adoption context | 5 | 5 | Source build, prerequisites, static output, and licenses remain explicit |
| Presentation and currency | 10 | 10 | Current commands, short sections, explanatory prose, and alt text |
| Total | 80 | 96 | Strongest gain: proof and demonstration, 15 to 20 |

Remaining opportunity: simplify the first-time Rust setup for readers who have never used Cargo.
Owner: `setup-install-usage-docs`; target: [INSTALL.md](../../INSTALL.md). Add verified macOS/Linux
Rust installation steps and a first-time setup check. Success means a reader with no Rust setup
can reach the existing build command. Verify on an appropriate fresh environment before claiming
that result. This capture run does not establish a novice installation test.

## Verification results

- Locked release build and executable help pass. README first success renders the pictured
  922 x 750 hemoglobin PNG; both usage-gallery commands render at their stated dimensions.
- The own-protein command was exercised with downloaded hemoglobin, producing 800 x 800;
  the larger-image options produce 1600 x 1200.
- The bundled synthetic PDB, mmCIF, and XML examples produce identical 800 x 800 PNG hashes:
  `ede28b2e55d3531da5e65fc210641a81a147aa6ab9e49d1bf6ac6935c6f41d08`.
- `webpinfo -summary` reports no errors, one static frame, and the stated dimensions for all
  three assets. Inspection confirms visible chain colors, shadows, outlines, and uncropped shapes.
- Each target has one exact sentinel pair and stable embed paths; rewriting those same embed
  lines leaves the block unchanged. No stale managed assets remain.
- `source source_me.sh && pytest tests/`: 947 passed in 1.78 seconds, including README opening,
  Markdown links, guidance format, ASCII, whitespace, and vendored-header checks.
- The opening is 190 characters, with one H1. The five applicable README quality gates pass;
  the animation gate is not applicable. `git diff --check` passes.

Not repeated: Rust unit/integration tests, reference-oracle comparisons, benchmarks, fresh
installation, or platform builds. The application and vendored support files remain unchanged.
