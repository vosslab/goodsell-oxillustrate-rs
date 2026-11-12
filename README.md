# goodsell-oxillustrate-rs

Create Goodsell-style protein images for seminar slides, posters, and lab reports. Turn molecular coordinates into PNG figures with soft colors and clear outlines, using a command-line tool.

Show the shape of a protein, distinguish its chains, and highlight bound molecules. Atoms appear
as space-filling spheres, with shadows and outlines that make molecular boundaries easier to see.

<!-- screenshots:begin (managed by screenshot-docs) -->
![Hemoglobin with pink and peach protein chains and red heme groups](docs/screenshots/hemoglobin.webp)
<!-- screenshots:end -->

Human hemoglobin, PDB entry 2HHB. Protein chain pairs are pink and peach; heme groups are red.
The commands below reproduce this image.

## Quick start

The current version installs from source. You need Rust 1.99.1 or newer with Cargo for the
one-time build, plus `curl` to download this example. See [docs/INSTALL.md](docs/INSTALL.md)
for setup. Run these commands from this repository's folder:

```bash
cargo build --release --locked
mkdir -p output
curl --fail --location https://files.rcsb.org/download/2HHB.cif --output output/2hhb.cif
./target/release/illustrate output/2hhb.cif --scene examples/hemoglobin.json \
  --output output/hemoglobin.png
```

Open `output/hemoglobin.png` in your image viewer. The 922 x 750 PNG can be inserted into a
presentation, poster, or report. After the build, use the same `illustrate` program for new images;
you do not need Python, a GPU, or a running molecular viewer.

## Use your own protein

Download a structure's PDBx/mmCIF coordinate file, or use a PDB file you already have.
Replace `your_protein.cif` with its filename:

```bash
./target/release/illustrate your_protein.cif --output output/my_protein.png
```

This creates an 800 x 800 image with automatic framing and different colors for different chains.
The first model and deposited coordinates are used by default. Waters and hydrogen atoms are
hidden; other bound molecules are retained. PDBML XML and gzip-compressed coordinate files also work.

## Make the figure yours

- Make a larger image with `--width 1600 --height 1200`.
- Show a biological assembly with `--assembly 1`, when that assembly is defined in the file.
  Deposited coordinates alone may show only part of the complete complex.
- Set chain colors, highlight a ligand, or rotate the view with a JSON scene file.
  [examples/hemoglobin.json](examples/hemoglobin.json) supplies the settings for the image above.

Follow [docs/USAGE.md](docs/USAGE.md) for scene-file examples and command options. It also shows
rendered examples of lambda repressor bound to DNA and a complete virus capsid, with commands
you can reuse for each.

## Help and scientific method

- [docs/INSTALL.md](docs/INSTALL.md): installation, requirements, and tested platforms.
- [docs/USAGE.md](docs/USAGE.md): image size, colors, orientation, models, and assemblies.
- [docs/REFERENCE_SPEC.md](docs/REFERENCE_SPEC.md): rendering method and reference comparisons.
- [docs/FILE_STRUCTURE.md](docs/FILE_STRUCTURE.md): further project documentation and source layout.

This tool makes static PNG illustrations. It does not create ribbons, molecular surfaces,
animations, or interactive views.

## Citation and license

The rendering method follows Goodsell DS, Autin L, Olson AJ (2019),
[*Illustrate: Software for Biomolecular Illustration*](https://doi.org/10.1016/j.str.2019.08.011).
This is a new Rust implementation; the legacy code was used for comparison and removed.

Source code: [LICENSE.AGPL-3.0](LICENSE.AGPL-3.0).

Documentation and other non-code materials: [LICENSE.CC-BY-4.0](LICENSE.CC-BY-4.0).
