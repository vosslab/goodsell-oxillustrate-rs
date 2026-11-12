# Native cleanup closeout

## Scope and deletion proof

The user authorized deletion of the old implementation after native acceptance. Before deletion,
`git ls-files OTHER_REPOS LOCAL_ONLY tests/_temp` returned no paths, and `git check-ignore`
confirmed the reference directories are ignored. `.git/index.lock` was absent. No Git index writes,
commits, staging, resets, or product-source edits were performed by this cleanup workstream.

Deleted these exact task-owned paths:

- `OTHER_REPOS/Illustrate/`, including its nested `.git` and `illustrate.f`: 34 files, 13 MiB.
- `tests/_temp/`: four temporary test modules, PDF extraction/review scratch, and the temporary
  M11 PNG, 7.9 MiB. Earlier visual review evidence remains recorded in its reports; these temporary
  viewed files are no longer present.
- `output_native/target_macos/`: 649 MiB duplicate build cache.
- `output_native/target_linux_cross/`: 794 MiB duplicate build cache.
- `output_native/qemu_rustc_20261112-043910_6.core`: 340,656,128-byte crash dump.

The clone was the only remaining `.f` source outside build caches. Oracle source copies,
`deterministic.patch`, and the old oracle executables were already absent before this cleanup.
After deletion, a repository inventory found no `.f`, `.f90`, `.patch`, `illustrate_original`, or
`illustrate_deterministic` files outside build caches, and no top-level staging/backup directories.
The exact deleted paths are absent. `OTHER_REPOS/` itself remains an empty ignored directory.

Retained `LOCAL_ONLY/` public papers and web references, all primary `output_oracle/` and
`output_fidelity/` data, `output_oracle_lifecycle/` corroborating data, installed native binaries,
logs, clean-source snapshots, actual images, manifests, and TSV measurements. Installed executables
are the native macOS arm64 Mach-O and Linux x86-64 ELF Rust builds. `output_native/` is now 40 MiB.

## Permanent test boundary

Added one 43-line offline test in `tests/test_reference_oracle.py`. It injects a partial generation
failure and verifies that accepted manifest/image bytes survive and temporary source/staging
directories are removed. Inputs are inline, paths use `tmp_path`, and source generation/network/
compiler work is mocked. No Fortran code, compiler, download, or persistent fixture is required.
The other temporary lifecycle/promotion/rollback/render checks were implementation proof and
were removed after their results had been recorded by their owners.

If this permanent contract test fails, fix regeneration so it preserves accepted evidence and
cleans its temporary workspace before retrying; do not weaken the preservation assertions.

## Final local checks

All Cargo commands used `CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo` from the repository root.

| Command | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --locked --all-targets` | PASS |
| `cargo test --locked` | PASS: 42 library tests, 3 integration tests, 1 doctest |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| `RUSTDOCFLAGS='-D missing_docs' cargo doc --no-deps --locked` | PASS |
| `source source_me.sh && python3 -m pytest -q tests` | Initial: 835 passed, 1 Bandit failure; final integration: 837 passed |
| `git diff --check` | PASS |

The initial pytest failure was Bandit B310 at `devel/reference_oracle.py:39`, the
`urllib.request.urlopen(url, timeout=60)` call. Its fixed HTTPS host, pinned revision, and fixed
filename allowlist were documented in a narrow audit comment without changing behavior. All 42
targeted Bandit checks then passed. Fresh independent
[goodsell_integration_review.md](goodsell_integration_review.md) passed all 837 pytest tests and
the Cargo gates above; the Bandit failure is resolved.

After deleting the clone, both real CLI commands passed:

```bash
./target/release/illustrate examples/sample.cif --output output_native/outside_source/final_sample.png
./target/release/illustrate output/2hhb.cif --output output_native/outside_source/final_hemoglobin.png --scene examples/hemoglobin.json
```

The sample PNG is 800 x 800 RGB and hemoglobin PNG is 922 x 750 RGB. These are post-cleanup local
execution checks; earlier independent visual review remains the visual acceptance evidence.

## Retained evidence hashes

These SHA-256 values were identical before and after cleanup:

| File | SHA-256 |
| --- | --- |
| `docs/screenshots/hemoglobin.webp` | `8a22f475a662de06a0115c03e22fc3470d245507f51b88c07829dcda143b382d` |
| `output_oracle/manifest.json` | `ce9d83821811a31edb3e3817ee5aca90e32d22811a5aa174fd0296b21303e971` |
| `output_oracle/spheres.webp` | `24e69097518c502b14028fdbc8b5e7e4cf5eb8009845828555b1ba60b67c4c73` |
| `output_fidelity/manifest.json` | `e4eba3d41c52e33df958f743b87168861d70b81b14473a097f958f2e6b4441e8` |

New local render SHA-256 values:

- `final_sample.png`: `ede28b2e55d3531da5e65fc210641a81a147aa6ab9e49d1bf6ac6935c6f41d08`.
- `final_hemoglobin.png`: `e87db3684062dfbd19fc37e666c860de7e5729581b35e9334875a230488b85c2`.

Physical x86 hardware and remote CI were not executed by this workstream. Fresh oracle downloads,
benchmarks, platform-network checks, and independent final integration review were not repeated.
Final independent integration ACCEPT and documentation/archival closeout are recorded in
[goodsell_ledger.md](../workstreams/goodsell_ledger.md).
