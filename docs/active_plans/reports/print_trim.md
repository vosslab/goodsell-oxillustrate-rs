# Print resolution and content trimming

Completed 2026-11-12 on macOS arm64. Implementation owner: the current repository agent.
The native library and CLI now export print resolution and always trim unused image space.
The user's follow-up removes the initial trim/no-trim controls; whitespace is added afterward.
Follow-up verification passes all 52 Rust tests, 1017 repository checks, formatting, all-target
compilation, Clippy, rustdoc, and the release build. The raw-pass probe verifies cropped production
RGB; a new native hemoglobin PNG is byte-identical to the earlier cropped output. The current
large-print command yields 1352 x 1748 pixels at 300 dpi (4.51 x 5.83 inches).

## Scope and decisions

- In scope: integer `dpi` in owned settings, scenes, and images; default 300; CLI override;
  PNG pHYs metadata and sRGB; WebP EXIF X/Y resolution with inch units. DPI never resamples.
- In scope: unconditional post-render trimming to all atom/outline coverage, retaining three pixels
  of available padding. There is no trim setting or full-canvas output flag. Empty scenes
  retain the existing typed error. Cropping preserves detached ions and matching-background atoms.
- In scope: final-size/print-size diagnostics, focused contract tests, real protein print exports,
  refreshed documentation images, and current CLI/API guidance.
- Out of scope: mandatory configuration files and YAML. CLI controls cover everyday output;
  the existing optional JSON format saves detailed appearance without adding another parser.
- Out of scope: SVG, explicitly declined by the user for this per-pixel renderer. PDF pages,
  CMYK separation, bleed, and additional physical-unit sizing flags are unnecessary here:
  lossless transparent PNG, pixel dimensions, and DPI provide the image for a publishing app.

## Files and behavior

`src/settings.rs`, `src/scene.rs`, `src/scene_preparation.rs`, `src/render.rs`, and
`src/rendered_image.rs` carry DPI as an owned integer. Rendering borrows the scene, owns its
scratch buffers, and copies the final cropped RGB/alpha rows without filtering. Crop bounds
come from winning atoms and nonzero outline opacity, independently of background RGB values.
No rendering equation, radius, color, camera, or projection changes.

`src/command_line.rs` applies explicit DPI flags over optional JSON. DPI metadata is
validated both at scene validation and before encoding; invalid input preserves existing output.
ASVS 2.2.1: positive, representable resolution and the existing checked canvas pixel limit are
enforced before allocation/output. `png` was already locked/transitive; its direct encoder exposes
physical dimensions without manual PNG chunk manipulation. WebP's existing encoder writes a
66-byte TIFF resolution IFD into its supported EXIF chunk. Both formats keep atomic replacement.

`examples/render_passes.rs` exports raw full-canvas passes sharing depth-buffer coordinates and
checks their cropped RGB against the public renderer. No configuration override is needed. No vendored files,
reference repositories, Git index, or workflows are modified.

## Verification and real exports

All 52 Rust tests pass, including two focused contracts for coverage-preserving trimming,
white-on-white atoms, extra-canvas invariance, clipped geometry, print metadata, CLI/JSON precedence, and invalid-DPI
output preservation. The existing alpha tests continue to cover white/dark compositing.

Independent Pillow checks on actual hemoglobin exports pass:

| File under ignored output_print | Pixels | Resolution | Intended print size |
| --- | --- | --- | --- |
| hemoglobin_300.png | 2022 x 1748 | PNG 299.9994 dpi | 6.74 x 5.83 inches |
| hemoglobin_600.webp | 2022 x 1748 | EXIF 600 dpi, inch units | 3.37 x 2.91 inches |

PNG density is stored as integer pixels/metre, hence the tiny rounding difference from 300.
The two hemoglobin files decode to identical RGBA bytes. The transparent hemoglobin capture
matches an independent alpha-bounds crop with three pixels of padding exactly. The newly cropped
capsid matches the pre-existing native contour-3 render after an independent trim, pixel for pixel.
Three temporary checks passed and were removed after verification.

The print command uses a larger canvas and always trims the final image. The initial exact-canvas
check preceded the user's removal of that option; the current command is:

```bash
./target/release/illustrate --pdb 1lmb --width 2400 --height 1800 --dpi 300 \
  --output output_print/protein_print.png
```

The production pass probe succeeds on the four-atom sample and verifies cropped composition
while retaining raw full-canvas pass exports. Automated geometry/shading/outlines tests pass;
the Fortran oracle is not rerun.

## Refreshed documentation captures

All twelve molecular images are regenerated after the user's 2026-11-12 transparency correction,
completed at 14:31 UTC. Explicit white backgrounds are removed from all ten bundled scenes.
All final captures are native lossless WebPs with RGBA, EXIF 300 dpi, transparent pixels, and
partial-alpha outlines. The helper now writes WebP directly and needs no converter. Lossless
native output preserves exact edge RGB/alpha values. The architecture-map SVG is separate from
the molecular renderer and remains unchanged; ignored reference images are outside this recapture.

The earlier comparison matched already-trimmed captures from the print implementation. It did
not mean that cropping was absent. The source-canvas and final-output columns below make that
distinction explicit. Replacing an opaque background with alpha leaves the crop dimensions
unchanged. Eleven files now differ from the prior opaque/quality-90 previews; the already-native
alpha example remains identical. Hemoglobin and its transparency example intentionally share
the same native RGBA pixels and bytes.

| Asset slug | Source canvas | Final pixels | Bytes |
| --- | --- | --- | ---: |
| hemoglobin | 922 x 750 | 844 x 730 | 410384 |
| lambda_dna | 960 x 600 | 426 x 548 | 145910 |
| cowpea_capsid | 1600 x 1000 | 952 x 948 | 768132 |
| transparent_backgrounds | 922 x 750 | 844 x 730 | 410384 |
| gallery_lysozyme | 1200 x 750 | 920 x 698 | 284206 |
| gallery_gfp | 1200 x 750 | 626 x 698 | 230522 |
| gallery_antibody | 1200 x 750 | 960 x 698 | 412638 |
| gallery_potassium_channel | 1200 x 750 | 698 x 698 | 309264 |
| gallery_trna | 1200 x 750 | 706 x 698 | 180654 |
| gallery_nucleosome | 1200 x 750 | 750 x 698 | 306910 |
| gallery_groel_groes | 1200 x 750 | 502 x 698 | 298560 |
| gallery_rhinovirus | 1600 x 1000 | 962 x 948 | 871202 |

SHA-256 values for the four usage/README captures:

```text
hemoglobin 25994bcc3c8e0a4ce64faa8a672066f005c45b7320dec92ea95b3cbf525fd03d
lambda_dna c3cab083f1c6c3e8b00a32f3031f8945286dccfdd975a83a3fca221b228c47ae
cowpea_capsid b7861ce9b7a0d378caaadd24a60d26537fff7a0f282578de1a08b11d88e973ae
transparent_backgrounds 25994bcc3c8e0a4ce64faa8a672066f005c45b7320dec92ea95b3cbf525fd03d
```

The complete current manifest is `output_print/alpha_capture_manifest.json`; light/dark review
sheets and `alpha_webpinfo.txt` remain alongside it. All twelve final WebPs were inspected over
light and dark backgrounds; no white matte or clipping was observed. All fit the 1920-pixel and
1 MB per-image budgets. Trimming preserves detached tRNA magnesium and three pixels of padding.
Five temporary checks confirm RGBA/partial alpha, unchanged crop dimensions, live embeds,
300 dpi, source-owned oracle backgrounds, and white composites matching prior native opaque
renders within one byte per channel. PNG counterparts are also regenerated in ignored `output/`
and `output_gallery/`; all twelve match their native WebPs pixel for pixel, with alpha and DPI
preserved. Those temporary checks are removed after verification. The final repository run passes
1017 checks in 2.50 seconds. `webpinfo` reports no errors for all twelve native WebPs.

Recapture commands:

```bash
CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo ./devel/render_gallery.sh
./target/release/illustrate --pdb 2hhb --scene examples/hemoglobin.json \
  --output docs/screenshots/hemoglobin.webp
./target/release/illustrate --pdb 2hhb --scene examples/hemoglobin.json \
  --output docs/screenshots/transparent_backgrounds.webp
./target/release/illustrate --pdb 1lmb --width 960 --height 600 \
  --output docs/screenshots/lambda_dna.webp
./target/release/illustrate --pdb 1cwp --scene examples/cowpea_capsid.json \
  --output docs/screenshots/cowpea_capsid.webp
```

The application Rust remains unchanged during this recapture. The earlier print-feature checks
below are retained as implementation evidence; Rust, Linux, Fortran, and live GitHub/browser
execution are not repeated for the documentation capture.
Formatting, locked all-target compilation, all 52 Rust tests, Clippy with warnings denied,
rustdoc with missing docs denied, `git diff --check`, and shell syntax pass. The exact
`source source_me.sh && pytest tests/` command passes 1014 checks in 3.81 seconds after temporary
checks are removed. `webpinfo` reports no errors across all twelve final assets.
Physical printer output, publisher acceptance, browser layout, and Linux execution are not checked.
