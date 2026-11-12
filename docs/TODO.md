# Development priorities

## High priority

1. **Transparent backgrounds and alpha output.** Support native RGBA PNGs so molecular
   illustrations can sit on colored slides or posters. Preserve molecular outlines without
   leaving a white fringe; verify the result over both light and dark backgrounds.
2. **Native WebP output.** Save WebP directly from the renderer, including alpha when a
   transparent background is requested. Keep PNG available and ensure the encoded format
   matches the requested output.

These priorities come from the user's 2026-11-12 guidance in
[HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md). Neither feature is implemented yet: the renderer currently
stores RGB pixels and encodes PNG only. Documentation WebPs are converted afterward with `cwebp`.
