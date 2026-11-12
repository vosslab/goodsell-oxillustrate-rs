# Image output and download verification

Verified 2026-11-12 on macOS arm64, Rust 1.99.1, root package `oxillustrate` 26.11.0.
All requested features are implemented; no implementation remains pending.
Completed plan: [image_output_downloads.md](../../archive/image_output_downloads.md).

## Delivered behavior

- Transparent backgrounds by default. JSON RGB triples remain opaque; JSON null is transparent.
  One `--background transparent|white|black` option overrides the scene.
- PNG and native lossless WebP selected by extension, with exact alpha preservation and shared
  atomic output. RGB bytes remain in `RenderedImage.pixels`; optional straight alpha is separate.
- `--pdb ID` and library `load_pdb` fetch RCSB mmCIF directly over verified HTTPS. Legacy and
  extended IDs are accepted case-insensitively. Local files remain an offline alternative.
- No input cache, arbitrary URL option, external curl process, async runtime, or lossy-quality
  setting. The requested one-entry image workflow succeeds without these mechanisms.

Primary service/encoder contracts were checked against
[RCSB downloads](https://www.rcsb.org/docs/programmatic-access/file-download-services),
[ureq](https://docs.rs/ureq/latest/ureq/), and the
[image WebP encoder](https://docs.rs/image/latest/image/codecs/webp/struct.WebPEncoder.html).
The vossvolvox-rust Python E2E downloader was inspected read-only as a behavior reference.

## Native checks

Commands ran from the repository root, with Cargo cache
`CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo`:

```bash
cargo fmt --check
cargo check --locked --all-targets
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
RUSTDOCFLAGS='-D missing_docs' cargo doc --no-deps --locked
cargo install --path . --locked --root output/install_current
source source_me.sh && pytest tests/
```

Results: 44 unit tests, 5 integration tests, and 1 doctest pass. All compilation, formatting,
Clippy, documentation, and installation commands pass. Pre-archival repository pytest:
953 passed in 2.06 seconds. After plan archival and link repair, the exact full command passes
again: 957 tests in 1.72 seconds. The completed plan is staged only at its existing archive path;
the old active path is absent from the index, avoiding the prior collection failure.

Permanent tests exercise actual encoder round trips, opaque/alpha compositing over white and
black, absence of a matte in outline pixels, invalid RGB/alpha buffers, unsupported output
extensions, WebP's dimension limit, output preservation, CLI source conflicts, background
overrides, accession validation, and bounded UTF-8 response reading. Fast tests use no network.

## Live commands and image evidence

```bash
mkdir -p output
./target/release/illustrate --pdb 1lmb --output output/lambda_download.webp \
  --width 960 --height 600
./target/release/illustrate --pdb pdb_00001lmb --output output/lambda_download.png \
  --width 960 --height 600
./target/release/illustrate --pdb 2hhb --scene examples/hemoglobin.json \
  --background transparent --output output/hemoglobin_transparent.webp
./target/release/illustrate output/2hhb.cif --scene examples/hemoglobin.json \
  --background transparent --output output/hemoglobin_transparent.png
./target/release/illustrate --pdb 1cwp --assembly 1 --background white \
  --output output/capsid_download_check.png --width 1600 --height 1000
```

All exit 0. A separate `pdb_99999999` request returns HTTP 404 and exit 1 without replacing an
existing output. CLI white-PNG and black-WebP examples also succeed. The freshly installed
`output/install_current/bin/illustrate` downloads `1lmb` and saves WebP with working directory
`/private/tmp`, outside the source tree.

Native transparent hemoglobin PNG and WebP decode to exactly the same 922 x 750 RGBA bytes.
`webpinfo` confirms one lossless frame with alpha. Opaque hemoglobin's PNG SHA-256 remains
`e87db3684062dfbd19fc37e666c860de7e5729581b35e9334875a230488b85c2`;
the complete opaque capsid remains
`927185a122b27df3e3cfb0e681cd0a90b812e9d75519c4dc66ca113170bf9ee3`.

Transparent source hashes:

```text
PNG  46e405192d5771a3c3e151e10c9943c8f48067789382a49e0dd11cc5110e0c13
WebP 14b39b553c3cfee5170a82681ea06922460bb8002dfa1d51ad14b3a6321c1248
```

[transparent_backgrounds.webp](../../screenshots/transparent_backgrounds.webp) is a 1600 x 1000,
169428-byte quality-90 comparison of that same alpha render over white and dark slide colors.
The encoded image was visually inspected; there is no white border on the dark background.
Its SHA-256 is `240b81b4a453ec484836c92bf5b2739697ed1294c7ccd88d7e83157286936481`.
The comparison is embedded in [USAGE.md](../../USAGE.md). Temporary capture code is removed;
native generated PNG/WebP examples remain in ignored `output/`.

## Evidence limits

No Linux/physical x86_64 rerun, new Fortran-oracle comparison, or performance claim is made.
TLS trust and timeout use the HTTP client's verified defaults/configuration; a deliberately
invalid-certificate server and stalled network were not induced. Actual RCSB HTTPS success and
404 behavior were observed. The installed render uses a local Cargo dependency cache; it is not
a fresh-machine Rust setup test. No hosted workflows or vendored/master-template edits were made.
