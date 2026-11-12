# Code architecture

## Overview

[Cargo.toml](../Cargo.toml) defines one safe, synchronous Rust package: library `oxillustrate` and
binary `illustrate`. [src/lib.rs](../src/lib.rs) re-exports owned molecular, settings, scene, image,
and error types; dependency-specific parser objects remain private.

## Data flow

```text
local PDB / mmCIF / PDBML (+ gzip)
  -> load_structure -> Structure
  -> prepare_scene + SceneSettings -> Scene
  -> render -> RenderedImage -> save_png
```

[src/command_line.rs](../src/command_line.rs) reads JSON, applies explicit flag overrides, protects
input paths, prints scene diagnostics, and invokes the same library operations used by callers.

## Major components

| Component | Responsibility |
| --- | --- |
| [src/input.rs](../src/input.rs) | Bounded local reading, gzip decoding, format dispatch |
| [src/pdb_input.rs](../src/pdb_input.rs) | PDB adapter |
| [src/cif_input.rs](../src/cif_input.rs) | PDBx/mmCIF adapter; tokens in [src/cif_tokens.rs](../src/cif_tokens.rs) |
| [src/xml_input.rs](../src/xml_input.rs) | PDBML XML adapter |
| [src/atom_table.rs](../src/atom_table.rs) | Shared atom field conversion and owned-record projection |
| [src/assembly_metadata.rs](../src/assembly_metadata.rs) | Shared assembly metadata conversion |
| [src/molecular.rs](../src/molecular.rs) | Owned atom identities, residues, assemblies, and transforms |
| [src/assembly.rs](../src/assembly.rs) | Model selection and bounded assembly-expression expansion |
| [src/settings.rs](../src/settings.rs) | JSON types, defaults, and numeric validation |
| [src/scene_preparation.rs](../src/scene_preparation.rs) | Alternate selection, appearance, assembly copies, camera, and spheres |
| [src/raster.rs](../src/raster.rs) | Source-compatible sphere depth and winning-atom rasterization |
| [src/shading.rs](../src/shading.rs) | Conical shadow sampling and depth fog |
| [src/outlines.rs](../src/outlines.rs) | Independent contour, subunit, and residue opacity passes |
| [src/pdb_download.rs](../src/pdb_download.rs) | Validated RCSB accession, bounded synchronous HTTPS, owned mmCIF parsing |
| [src/render.rs](../src/render.rs) | RGB composition with optional straight alpha for transparent backgrounds |
| [src/rendered_image.rs](../src/rendered_image.rs) | PNG/lossless WebP encoding and atomic destination replacement |

## Contracts and corrections

Source coordinates use x=row/down, y=column/right, and z=toward viewer. Assembly products act
rightmost first; author and label identifiers remain distinct. Scene preparation converts molecular
Angstrom coordinates and radii to image pixels before rendering.

[REFERENCE_SPEC.md](REFERENCE_SPEC.md) records source equations. Native D1 initializes undefined
source scratch/background state and handles flat-depth fog. D3 uses physical radius plus one
Angstrom for automatic z burial. Explicit matched camera centers are required for oracle depth
comparisons. These corrections are deliberate behavior, not claims of byte-identical legacy output.

## Testing and extension

Module tests exercise parsing, assemblies, scene preparation, rasterization, shadows, fog, and
outlines. [tests/render_cli.rs](../tests/render_cli.rs) checks the executable pipeline and failures.
[goodsell_oracle.md](active_plans/reports/goodsell_oracle.md) documents the separate Fortran oracle;
reference tools are development-only.

Add format handling at the input adapters, appearance/camera behavior at scene preparation, and
pixel mathematics in its owning render pass. Keep the public loading/preparation/rendering pipeline in
[src/lib.rs](../src/lib.rs). For developer commands, see [INSTALL.md](INSTALL.md).
