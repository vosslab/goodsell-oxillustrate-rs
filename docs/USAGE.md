# Usage

Render a local molecular structure to PNG with `illustrate`, or call the same pipeline from Rust.
Build the executable first using [INSTALL.md](INSTALL.md). Commands below run from the repo root.

## Quick start

```bash
mkdir -p output
./target/release/illustrate examples/sample.pdb --output output/sample.png
```

The original synthetic four-atom example writes an 800 x 800 image with automatic framing, white background, warm chain colors,
conical shadows, and contour, subunit, and residue outlines. Native defaults hide H, D, and water
and retain other hetero atoms. The first model in source order is selected.

Replace the sample with your local structure path, or use the public hemoglobin example:

```bash
curl -L https://files.rcsb.org/download/2HHB.cif -o output/2hhb.cif
./target/release/illustrate output/2hhb.cif --output output/hemoglobin.png \
  --scene examples/hemoglobin.json
```

[hemoglobin.json](../examples/hemoglobin.json) specifies a 922 x 750 image, source-style
orientation and ordered pink/peach protein and red-heme rules.

## Command-line options

```text
illustrate <INPUT> --output <OUTPUT> [OPTIONS]
```

| Option | Behavior |
| --- | --- |
| `--output`, `-o` | Required destination PNG filename |
| `--scene` | Read partial scene settings from a JSON file |
| `--width`, `--height` | Override image dimensions in pixels |
| `--model` | Select a positive deposited model number |
| `--assembly` | Select an assembly ID defined by the input |
| `--help`, `--version` | Show command help or package version |

Explicit flags take precedence over JSON settings. Deposited coordinates are the default;
assembly selection requires metadata in the input. For example:

```bash
./target/release/illustrate output/2hhb.cif --output output/assembly.png \
  --assembly 1 --width 1000 --height 1000
```

## Scene settings

JSON may provide only changed fields; unknown keys are errors. For example, save this as a scene
file and pass its path with `--scene`:

```json
{
  "width": 1000,
  "height": 800,
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

## Rust API

```rust,no_run
use oxillustrate::{SceneSettings, load_structure, prepare_scene, render};

fn main() -> Result<(), oxillustrate::Error> {
    let structure = load_structure("structure.cif")?;
    let scene = prepare_scene(&structure, &SceneSettings::default())?;
    render(&scene)?.save_png("structure.png")?;
    Ok(())
}
```

Run `cargo doc --no-deps --locked --open` to browse the public API. `Scene` owns prepared spheres;
rendering does not reread the input. Diagnostics are retained on the prepared scene and printed
by the CLI.

## Inputs, output, and limits

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
