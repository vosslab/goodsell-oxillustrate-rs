# Install

The root Cargo package builds the `illustrate` executable and the `oxillustrate` Rust library.
Run the following commands from a source checkout containing [Cargo.toml](../Cargo.toml).

## Requirements

- Rust 1.99.1 or newer, with Cargo; the crate uses Rust edition 2024.
- Network access for the first dependency download, unless Cargo already has the locked crates.
- A local PDB, PDBx/mmCIF, or PDBML XML file for rendering; gzip input is supported.
- `curl` only for the optional public-data download shown in [USAGE.md](USAGE.md).

The native build uses Rust dependencies from [Cargo.lock](../Cargo.lock). It does not require
Python, Fortran, a GPU, or the ignored local reference collections.

## Build from source

```bash
cargo build --release --locked
```

The resulting executable is `target/release/illustrate`. Keeping it in the checkout is sufficient;
no global installation is required. The library is available to Cargo builds as `oxillustrate`.

## Verify install

```bash
./target/release/illustrate --help
```

Successful help output identifies the required structure input and `--output` PNG path.
For a complete render, follow [USAGE.md](USAGE.md).

## Developer checks

```bash
cargo fmt --check
cargo check --all-targets --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo doc --no-deps --locked
```

Rust API documentation is generated under `target/doc/oxillustrate/`. Run repository checks locally:

```bash
source source_me.sh && pytest tests/
```

Python is development tooling, not a renderer dependency.

## Verification status

Clean-source macOS arm64 build, formatting, tests, Clippy, installation, and outside-source CLI
rendering pass. Linux x86_64 cross-build, tests, installation, and actual emulated CLI rendering
also pass; the sample PNG is byte-identical across those executions. The source snapshots use
cached dependencies and fresh build targets, without legacy reference files.

Physical x86_64 Linux execution was not observed. Direct x86_64 Rust
compiler execution failed under QEMU; the successful fallback cross-compiled using an arm64 Linux
compiler and ran the resulting x86_64 tests/executable under emulation. See
[goodsell_native_validation.md](active_plans/reports/goodsell_native_validation.md) for commands,
hashes, logs, and verification limits.
