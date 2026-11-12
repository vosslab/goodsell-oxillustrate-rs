# Example inputs

[sample.pdb](sample.pdb), [sample.cif](sample.cif), and [sample.xml](sample.xml) are original
synthetic four-atom geometry demonstrations created for this project. They contain identical
coordinates, elements, author chain/residue identities, occupancy, and model. They are not an
experimentally determined structure, a valid peptide, or a scientific structural model.

```bash
cargo run --release -- examples/sample.pdb --output output_sample.png
```

[hemoglobin.json](hemoglobin.json) encodes the published Illustrate 2hhb demonstration's
first-match appearance and rendering settings, translated to the native JSON format. Supply a
local 2HHB structure separately; no third-party structure or rendering implementation is bundled
here. The native automatic camera uses physical radii; deterministic reference comparisons use
an explicit matched center as recorded in the fidelity report.

[render_passes.rs](render_passes.rs) is a development-only diagnostic probe. It reads a local
structure and JSON settings, then exports the actual native production passes and timings.
It does not add a public debugging interface to the library.

The [gallery/](gallery/) scene files reproduce the eight examples in
[docs/GALLERY.md](../docs/GALLERY.md). They select poster-inspired colors, views, and assemblies.
Run [devel/render_gallery.sh](../devel/render_gallery.sh) from the repository to refresh the
native PNGs and quality-90 documentation WebPs.

[cowpea_capsid.json](cowpea_capsid.json) selects assembly 1 of PDB 1CWP with a smaller contour
neighborhood for a lightly outlined whole-capsid image. The nucleosome, GroEL/GroES, and
rhinovirus gallery scenes use the same contour setting. Other rendering defaults are unchanged.
