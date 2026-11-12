# Usage

Render a local molecular structure or an RCSB entry to PNG or WebP with `illustrate`.
Build the executable first using [INSTALL.md](INSTALL.md). Commands below run from the repo root.

## Quick start

```bash
mkdir -p output
./target/release/illustrate examples/sample.pdb --output output/sample.png
```

The original synthetic four-atom example uses an 800 x 800 canvas with automatic framing and warm chain colors,
conical shadows, and contour, subunit, and residue outlines. Its background is transparent.
Native defaults hide H, D, and water
and retain other hetero atoms. The first model in source order is selected.
Output is trimmed to the molecule and its outlines, with three pixels of padding where the
canvas allows. Add any page margins or whitespace in your slide or publishing application.

Replace the sample with your local structure path, or use the public hemoglobin example:

```bash
./target/release/illustrate --pdb 2hhb --output output/hemoglobin.png \
  --scene examples/hemoglobin.json
```

[hemoglobin.json](../examples/hemoglobin.json) specifies a 922 x 750 canvas, source-style
orientation and ordered pink/peach protein and red-heme rules.
This reference scene fixes `framing.scale` at 12 pixels per Angstrom. Remove that field when
editing it for automatic fitting to a larger print canvas.

## Transparency and output format

The output filename selects `.png` or `.webp`, case-insensitively. WebP is encoded natively and
losslessly; both formats preserve alpha. New scenes have a transparent background by default.
Use `--background white` or `--background black` for an opaque image, or
`--background transparent` to override an RGB background in a scene file.

```bash
./target/release/illustrate --pdb 1lmb --output output/protein.webp
./target/release/illustrate --pdb 1lmb --background white --output output/protein_white.png
```

In JSON, `"background": null` means transparent; an RGB triple such as `[1,1,1]` means opaque.
Existing RGB-background scene files keep their appearance. Atom surfaces remain opaque;
outline-only pixels carry black with fractional alpha, avoiding a white fringe when composited.
Depth fog still uses the scene's explicit fog color; transparency does not recolor molecular atoms.
In a browser, transparent areas show the page background. All bundled scenes and documentation
images have real alpha, with no colored background or checkerboard baked into the files.

## More molecular examples

These images were rendered by the same `illustrate` program. The first shows lambda repressor
bound to DNA (PDB 1LMB), with colors distinguishing the chains. The second shows the complete
biological assembly 1 of cowpea chlorotic mottle virus (PDB 1CWP), using lighter contour settings.
All three images have transparent backgrounds; their surroundings follow the page color.

<!-- screenshots:begin (managed by screenshot-docs) -->
![Lambda repressor bound to DNA, with different colors for each chain](screenshots/lambda_dna.webp)
![Complete cowpea chlorotic mottle virus capsid, showing its repeating protein subunits](screenshots/cowpea_capsid.webp)
![Hemoglobin with a transparent background that shows the browser page behind the molecule](screenshots/transparent_backgrounds.webp)
<!-- screenshots:end -->

Download the coordinates and make the images:

```bash
mkdir -p output
./target/release/illustrate --pdb 1lmb --output output/lambda_dna.png \
  --width 960 --height 600
./target/release/illustrate --pdb 1cwp --scene examples/cowpea_capsid.json \
  --output output/cowpea_capsid.png
./target/release/illustrate --pdb 2hhb --scene examples/hemoglobin.json \
  --background transparent --output output/hemoglobin_transparent.webp
```

The first two commands use 960 x 600 and 1600 x 1000 canvases, respectively; the final PNGs are
cropped to content. The virus scene
selects assembly 1 to build the complete capsid from the deposited coordinates and assembly
operations. That choice is saved in [cowpea_capsid.json](../examples/cowpea_capsid.json).

## Command-line options

```text
illustrate [INPUT | --pdb ID] --output <OUTPUT> [OPTIONS]
```

| Option | Behavior |
| --- | --- |
| `--pdb` | Download one RCSB entry by case-insensitive accession; replaces local INPUT |
| `--output`, `-o` | Required destination `.png` or `.webp` filename |
| `--background` | Override JSON with `transparent`, `white`, or `black` |
| `--scene` | Read partial scene settings from a JSON file |
| `--width`, `--height` | Override render canvas dimensions in pixels, before trimming |
| `--dpi` | Print resolution in pixels per inch; default 300; does not resample |
| `--model` | Select a positive deposited model number |
| `--assembly` | Select an assembly ID defined by the input |
| `--help`, `--version` | Show command help or package version |

Explicit flags take precedence over JSON settings. Deposited coordinates are the default;
assembly selection requires metadata in the input. For example:

```bash
./target/release/illustrate --pdb 2hhb --output output/assembly.png \
  --assembly 1 --width 1000 --height 1000
```

## Printing and content trimming

Use PNG for a print-layout workflow. Both native formats are lossless and preserve transparency.
PNG records resolution in its standard
[physical pixel dimensions](https://www.w3.org/TR/png/#11pHYs) metadata; WebP records X/Y resolution
and inch units in [EXIF metadata](https://developers.google.com/speed/webp/docs/riff_container#metadata).
PNG also identifies its colors as sRGB. Programs may choose their own placement size, so check
the dimensions when inserting an image into your poster or document.

Print width in inches is final pixel width divided by DPI. For example, 2400 output pixels at
300 dpi spans 8 inches. To prepare a larger image, render on a 2400 x 1800 canvas:

```bash
./target/release/illustrate --pdb 1lmb --width 2400 --height 1800 --dpi 300 \
  --output output/protein_print.png
```

Output is always cropped to content. The CLI reports the final pixel dimensions
and physical size after saving. Cropping happens after all rendering passes, retains even faint
outline pixels, and leaves three pixels of available padding. It uses atom and outline coverage,
so atoms matching a white or black background are retained. Pixel colors and alpha are copied
without resizing. A molecule already touching a canvas edge has less padding there.

DPI metadata alone adds no detail: choose more pixels for a larger or sharper print. Increasing
the canvas dimensions provides more pixels when using automatic framing; an explicit
`framing.scale` fixes the molecular scale in pixels per Angstrom. The existing canvas limit is
16,777,216 pixels. Trimming can change the aspect ratio and printed size. Add any whitespace
afterward in your publishing application. PNG/WebP remain RGB formats; CMYK conversion and page layout
belong in your publishing application. SVG is outside this per-pixel renderer.

## Scene settings

JSON is optional and may provide only changed fields; unknown keys are errors. Everyday size,
resolution, background, and assembly controls are available on the CLI. Keep detailed coloring,
orientation, shadows, and outline choices in JSON. YAML is not supported.
For example, save this as a scene
file and pass its path with `--scene`:

```json
{
  "width": 1000,
  "height": 800,
  "dpi": 300,
  "orientation": [0, 0, 90],
  "fog": {"front": 1, "back": 0.5}
}
```

Colors are RGB triples in [0,1]. `orientation` uses source-compatible x/y/z degrees, applied Z,
then Y, then X. `framing.scale` sets pixels per Angstrom; absent scale fits sphere bounds with
30-pixel margins. `translation` follows rotation and centering. Source x points down image rows,
y points right along columns, and z points toward the viewer.

`atom_rules` use first-match selection by element, chain, residue, atom name, or kind. Chain and
residue-number selectors default to the author namespace; use `"namespace": "label"` explicitly
for label IDs. A matching rule changes only supplied appearance fields; `"omit": false` can restore
water or hydrogen. Full field definitions and defaults are documented on `SceneSettings` in
[src/settings.rs](../src/settings.rs) and in generated Rust API documentation.

### Lighter outlines and shadows

If a large assembly looks too dark or densely inked, use this in its scene file:

```json
{"outlines": {"kernel": 3}}
```

This uses a 3 x 3 contour neighborhood instead of the default 5 x 5 neighborhood. The smaller
neighborhood keeps fine surface detail from turning into thick black lines when many atoms fit
in a small image. The capsid scene and the larger gallery assemblies use this setting; shadows
and subunit boundaries stay enabled.

For softer shadows, use `"shadows": {"brightness_floor": 0.7}`. A larger floor limits how dark
shadows can become; the default is 0.2. To disable shadows entirely, use
`"shadows": {"enabled": false}`. To disable depth contours while retaining subunit boundaries,
use `"outlines": {"contour_enabled": false}`. These are scene settings passed with `--scene`.

## Rust API

```rust,no_run
use oxillustrate::{SceneSettings, load_structure, prepare_scene, render};

fn main() -> Result<(), oxillustrate::Error> {
    let structure = load_structure("structure.cif")?;
    let scene = prepare_scene(&structure, &SceneSettings::default())?;
    render(&scene)?.save("structure.webp")?;
    Ok(())
}
```

Replace `load_structure("structure.cif")` with `oxillustrate::load_pdb("1lmb")` to download by ID.
`RenderedImage.pixels` holds RGB bytes; its optional `alpha` vector holds one straight-alpha byte
per pixel. `RenderedImage.dpi` holds print resolution. The returned width/height are the cropped
dimensions. `save` selects the encoder by extension;
`save_png` and `save_webp` select it explicitly.
Run `cargo doc --no-deps --locked --open` to browse the public API. `Scene` owns prepared spheres;
rendering does not reread the input. Diagnostics are retained on the prepared scene and printed
by the CLI.

## Inputs, output, and limits

`--pdb` uses RCSB's HTTPS mmCIF service, preserving chain IDs and assembly definitions. It accepts
four-character IDs such as `1lmb` and extended IDs such as `pdb_00001lmb`. The response is parsed
in memory with a 60-second timeout and 1 GiB limit; no coordinate cache is written. A download
failure leaves an existing image untouched. Local-file rendering needs no network connection.
Service details: [RCSB file downloads](https://www.rcsb.org/docs/programmatic-access/file-download-services).

Input supports PDB/ENT, macromolecular PDBx/mmCIF, and PDBML XML, optionally gzip compressed.
CIF accepts one data block and no save frames. Alternate locations use highest mean occupancy
per residue, lexical ties, and retained shared blank-location atoms.

Create the output parent directory first. Saving uses a sibling temporary file followed by rename;
an existing destination is replaced only after successful encoding and flushing. The CLI rejects
an output path that resolves to its structure or JSON input. Errors print to stderr and exit nonzero.

Limits are 1 GiB decompressed input, 16,777,216 image pixels, 1,000,000 expanded assembly operations,
and 20,000,000 assembly atom references. Unsupported selections, empty visible scenes, and invalid
numeric settings return errors. These limits bound work; they do not guarantee sufficient memory
for every permitted input.

WebP additionally limits each image dimension to 16384 pixels. An unsupported format or size
returns an error and preserves an existing destination file.
