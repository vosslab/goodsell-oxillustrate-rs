# Native installation validation

Validation date: 2026-11-11. Package: root `oxillustrate` 26.11.0.

## CI coverage

The workflow in [native.yml](../../../.github/workflows/native.yml) runs stable
Rust formatting, locked all-target compilation, tests, and all-target Clippy with
warnings denied. It installs the CLI into the runner's temporary directory, then
renders `examples/sample.pdb` at 320 by 240 pixels from outside the source tree.
No Fortran compiler, reference checkout, or legacy executable is needed.

The matrix uses `macos-15` (arm64) and `ubuntu-24.04` (x64), verified against
[GitHub's runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
`actions/checkout@v7` was selected after checking the current official release,
[v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1).
The stable toolchain and component syntax follow
[dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain).
These labels and action references were checked live on the validation date.

No remote workflow was triggered. These are configured gates, not a claim of a
successful GitHub Actions run. A failing gate blocks merge: fix the corresponding
format, build, test, lint, installation, or render failure and rerun both jobs;
do not disable a gate or skip a platform to obtain a passing result.

## Local macOS arm64

The host reports macOS 27.2.1 (26B131), `uname -m` = `arm64`, and Rust/Cargo
1.99.1. The compiler host is `aarch64-apple-darwin`.

An independent source tree at `/private/tmp/goodsell_native_source.rnsstO`
contains only `Cargo.toml`, `Cargo.lock`, `src/`, `tests/render_cli.rs`, native
examples, and license files. It has no `.git`, ignored reference data, oldcode,
or preexisting build outputs. Installation uses a new target directory, with
already downloaded dependency sources reused from the host Cargo cache. This
is clean-source and fresh-target validation, not a fresh dependency download.

```bash
export CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo
export CARGO_TARGET_DIR=/Users/vosslab/nsh/goodsell-oxillustrate-rs/output_native/target_macos
cargo install --path /private/tmp/goodsell_native_source.rnsstO --locked \
  --root /Users/vosslab/nsh/goodsell-oxillustrate-rs/output_native/install
cargo fmt --manifest-path /private/tmp/goodsell_native_source.rnsstO/Cargo.toml --check
cargo check --manifest-path /private/tmp/goodsell_native_source.rnsstO/Cargo.toml --locked --all-targets
cargo test --manifest-path /private/tmp/goodsell_native_source.rnsstO/Cargo.toml --locked
cargo clippy --manifest-path /private/tmp/goodsell_native_source.rnsstO/Cargo.toml --locked --all-targets -- -D warnings
```

The final source snapshot includes the CIF validator correction (D11).
Formatting, compilation, tests, Clippy, and installation all exited 0.
`file output_native/install/bin/illustrate` reports `Mach-O 64-bit executable
arm64`. Tests passed: 42 library tests, 3 CLI integration tests, and 1
documentation test. The final gate log is `output_native/logs/macos_final.log`.

The installed executable was then run with working directory `/private/tmp`,
outside both source trees:

```bash
cd /private/tmp
/Users/vosslab/nsh/goodsell-oxillustrate-rs/output_native/install/bin/illustrate \
  /private/tmp/goodsell_native_source.rnsstO/examples/sample.pdb \
  --output /Users/vosslab/nsh/goodsell-oxillustrate-rs/output_native/outside_source/sample_macos.png \
  --width 320 --height 240
```

Exit status was 0 with no diagnostics. `file` confirms a 320 by 240 RGB PNG.
Visual inspection confirms four colored, outlined spheres. SHA-256:
`1c9c892e48799866bdfb99e446f9f7a6c9366ef832997bbad6f70a6603d59efe`.

Logs are generated under `output_native/logs/` and are intentionally untracked.

## Local Linux x86_64 attempt

The existing Podman machine is an AppleHV arm64 Linux VM on this arm64 Mac,
with Rosetta disabled. An official `docker.io/library/rust:1.99.1` image was
pulled with `--platform linux/amd64`. The image reports `uname -m` = `x86_64`.
This is QEMU user-mode emulation, not native x86_64 hardware.

The source snapshot was copied into `output_native/clean_source/` because the
VM cannot mount the host's `/private/tmp` path. That source was mounted
read-only at `/source`; generated Cargo files were directed into
`output_native/` through `/validation`.

The container failed at `rustc -vV`, before compilation, with QEMU's
`uncaught target signal 11 (Segmentation fault)` and shell exit 139. A separate
retry with `QEMU_CPU=max` failed identically. This toolchain-emulation failure
does not establish a Rust application failure or a passing Linux build.
The detailed command/output is in `output_native/logs/linux_validation.log`.

## Linux cross-build and emulation

A bounded fallback used the same official image's `linux/arm64` variant. Its
Rust compiler host is `aarch64-unknown-linux-gnu`; this avoids executing the
compiler through QEMU. The container installed the x86_64 Rust standard library
and Debian cross-linker/libc, then built `x86_64-unknown-linux-gnu`. All package
changes were confined to the disposable container.

The container mounted the same clean source read-only and `output_native/` at
`/validation`. Environment:

```bash
CARGO_HOME=/validation/linux_arm_cargo
CARGO_TARGET_DIR=/validation/target_linux_cross
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
QEMU_LD_PREFIX=/usr/x86_64-linux-gnu
```

Commands inside `podman run --rm --platform linux/arm64`:

```bash
rustup target add x86_64-unknown-linux-gnu
apt-get update
apt-get install -y --no-install-recommends gcc-x86-64-linux-gnu libc6-dev-amd64-cross
cargo check --manifest-path /source/Cargo.toml --locked --all-targets --target x86_64-unknown-linux-gnu
cargo test --manifest-path /source/Cargo.toml --locked --target x86_64-unknown-linux-gnu
cargo install --path /source --locked --target x86_64-unknown-linux-gnu --root /validation/install_linux_cross
cd /validation
/validation/install_linux_cross/bin/illustrate /source/examples/sample.pdb \
  --output /validation/outside_source/sample_linux.png --width 320 --height 240
```

The first emulated test launch lacked the x86_64 dynamic loader; adding the
standard `QEMU_LD_PREFIX` above resolved that environment problem. Compilation,
tests, installation, and render then exited 0. Tests passed: 42 library tests,
3 CLI integration tests, and 1 documentation test. `file` identifies the
installed executable as ELF 64-bit x86-64, dynamically linked with interpreter
`/lib64/ld-linux-x86-64.so.2`. The installed CLI ran with working directory
`/validation`, outside `/source`.

The Linux output is a 320 by 240 RGB PNG and is byte-identical to the macOS
output (SHA-256 `1c9c892e48799866bdfb99e446f9f7a6c9366ef832997bbad6f70a6603d59efe`).
Logs: `output_native/logs/linux_cross_validation.log` and
`output_native/logs/linux_cross_final.log`. Both retained source snapshots matched `src/`,
manifest, and lockfile at validation time. The current tree later added public rustdoc comments in
nine source files; those documentation-only changes do not alter the tested implementation. Final
macOS checks were refreshed against the current tree. Physical Linux x86_64 execution and remote CI
remain unobserved.

This proves a Linux x86_64 cross-build and actual emulated test/CLI execution.
It does not prove execution on physical x86_64 hardware. The configured
`ubuntu-24.04` CI job runs that target natively, but its execution has not been
observed in this session. The containerized GCC is test tooling only; native
application users need no Fortran toolchain.
