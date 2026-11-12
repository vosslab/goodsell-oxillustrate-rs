#!/usr/bin/env bash
# Rebuild the poster-inspired gallery with the native CLI and quality-90 documentation images.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
cargo build --release --locked
mkdir -p output_gallery docs/screenshots
gallery_tmp=$(mktemp -d /tmp/oxillustrate_gallery.XXXXXX)
trap 'rm -rf "$gallery_tmp"' EXIT

for entry in 1lz1:lysozyme 1gfl:gfp 1igt:antibody 1bl8:potassium_channel \
    4tna:trna 1aoi:nucleosome 1aon:groel_groes 4rhv:rhinovirus; do
    pdb_id=${entry%%:*}
    name=${entry#*:}
    ./target/release/illustrate --pdb "$pdb_id" --scene "examples/gallery/$name.json" \
        --output "output_gallery/$name.png"
    cp "output_gallery/$name.png" "$gallery_tmp/$name.png"
    cwebp -quiet -q 90 "$gallery_tmp/$name.png" -o "$gallery_tmp/$name.webp"
    cp "$gallery_tmp/$name.webp" "docs/screenshots/gallery_$name.webp"
done
