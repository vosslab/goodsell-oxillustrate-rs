#!/usr/bin/env bash
# Rebuild the poster-inspired gallery as native lossless WebPs with alpha.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
cargo build --release --locked
mkdir -p docs/screenshots

for entry in 1lz1:lysozyme 1gfl:gfp 1igt:antibody 1bl8:potassium_channel \
    4tna:trna 1aoi:nucleosome 1aon:groel_groes 4rhv:rhinovirus; do
    pdb_id=${entry%%:*}
    name=${entry#*:}
    ./target/release/illustrate --pdb "$pdb_id" --scene "examples/gallery/$name.json" \
        --output "docs/screenshots/gallery_$name.webp"
done
