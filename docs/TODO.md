# Development priorities

## Completed priorities: 2026-11-12

1. **Transparent backgrounds and alpha output.** New scenes default to transparent. PNG preserves
   alpha and molecular outlines without a white matte. `--background transparent|white|black`
   overrides scene settings; explicit RGB scene backgrounds remain opaque.
2. **Native WebP output.** `.webp` selects lossless native encoding, including transparency.
   `.png` remains available. Encoded output matches the selected extension.
3. **Native PDB downloads.** `--pdb 1lmb` downloads mmCIF from RCSB and renders without `curl`.
   Local files remain supported; the two input forms are mutually exclusive.

These priorities come from the user's 2026-11-12 guidance in
[HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md). See [USAGE.md](USAGE.md) for working commands.
The completed plan and verification are in
[image_output_downloads.md](archive/image_output_downloads.md) and
[image_output_downloads.md](active_plans/reports/image_output_downloads.md).
