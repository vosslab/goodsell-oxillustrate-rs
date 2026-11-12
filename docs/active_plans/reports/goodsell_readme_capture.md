# README hemoglobin image

The README image is a quality-90 WebP converted from the native CLI rendering of
the public RCSB 2HHB structure with `examples/hemoglobin.json`.

Reproduce the PNG with:

```bash
./target/release/illustrate output/2hhb.cif --scene examples/hemoglobin.json --output output/hemoglobin.png
```

The 922 x 750 [hemoglobin.webp](../../screenshots/hemoglobin.webp) was inspected after encoding.
It is about 110 KB and shows outlined pastel spheres and red heme groups. The README
embed uses a repo-relative path and descriptive alt text inside the screenshot-docs
managed block.
