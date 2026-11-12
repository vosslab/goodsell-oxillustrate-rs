# Final integration review

## Verdict

**ACCEPT, 2026-11-11.** No concrete product blocker or unexplained architectural drift was found.
The manager may complete M13 documentation closeout and archive the approved plan.

What this change does: the root native Rust package loads local molecular coordinates, prepares
owned scene geometry, renders deterministic Goodsell-style spheres, and atomically saves PNG.
The executable and library use the same pipeline without Python, Fortran, or reference folders.

## Scope and rules

Fresh independent review covered [goodsell_native.md](../../archive/goodsell_native.md), its user
amendments, [goodsell_contract.md](../decisions/goodsell_contract.md), the root manifest and crate
facade, connected CLI/load/render/save paths, tests, shipped examples, public documentation,
native CI, and the recorded visual/platform/cleanup evidence.

Applied root AGENTS guidance and repository, Rust, Markdown, Python, and pytest style documents.
Python commands used `source source_me.sh && python3`. The permanent offline oracle regression
protects accepted evidence on regeneration failure; its inline inputs and temporary paths fit the
pytest policy. No source, test, configuration, or shared closeout document was modified here.

## Composition findings

- Root Cargo package and AGPL-3.0-only metadata match the explicit user amendments. No workspace
  wrapper or parser-specific public types were introduced. Library and binary roots remain thin;
  all 22 source modules are under 999 lines, with the largest at 552.
- `load_structure -> prepare_scene -> render -> save_png` is connected through both API and CLI.
  CLI flags override JSON before validation; errors reach stderr. Save validation and temporary
  output preserve prior data, and CLI protection covers both coordinate and scene input paths.
- Adapter, identity, model/alternate selection, assembly, preparation, raster, shading, outline,
  and command-level behavior have focused permanent tests. Earlier detailed milestone reviews and
  independent visual assessment supply pass-specific acceptance; this review does not substitute
  a new visual judgment for that assessment.
- D1, D3, D10, and D11 are documented refinements rather than hidden compatibility changes. D12's
  measured deferral of persistent bins/threading appears explicitly in the plan and contract;
  KISS and use-the-scientific-method principles support retaining the verified renderer.
- `git ls-files LOCAL_ONLY OTHER_REPOS` returns no paths; `git check-ignore` confirms both folders.
  The legacy checkout, temporary tests, removed Apache license/NOTICE, old oracle source/patches/
  executables, and QEMU core are absent from the checked locations. Local papers remain preserved.
- The oracle manifest contains 38 render records. Fidelity contains 19 scenes with zero recorded
  mask differences; all five production module hashes and the diagnostic probe hash match current
  files. This checks evidence correspondence, not a fresh oracle rerun.

## Fresh verification

Cargo commands ran from the root with `CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo`.

| Exact command | Actual result |
| --- | --- |
| `source source_me.sh && python3 -m pytest -q tests` | 837 passed in 1.46 seconds |
| `cargo test --locked` | 42 unit, 3 integration, 1 doctest passed |
| `cargo fmt --check` | PASS |
| `cargo check --locked --all-targets` | PASS |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| `RUSTDOCFLAGS='-D missing_docs' cargo doc --no-deps --locked` | PASS |
| `git diff --check` | PASS |

Post-cleanup execution:

```bash
./target/release/illustrate examples/sample.cif \
  --output /private/tmp/goodsell_final_integration_sample.png --width 320 --height 240
file /private/tmp/goodsell_final_integration_sample.png
shasum -a 256 /private/tmp/goodsell_final_integration_sample.png
```

Exit 0; 320 x 240 RGB PNG. SHA-256:
`1c9c892e48799866bdfb99e446f9f7a6c9366ef832997bbad6f70a6603d59efe`.
This equals the previously recorded clean-source macOS and emulated Linux sample outputs.
The previous Bandit B310 failure is resolved; the full fresh pytest gate passes.

## Planned closure and limits

The manager should update the ledger's accepted M2/M12 reviews and M13 status, replace stale
pending-cleanup/test statements with the actual outcomes, record this acceptance, then use
`git mv` to archive the completed plan and repair referring links. These are the already planned
closure steps, not new product blockers. Only the human commits.

Not repeated: source downloads, oracle generation, benchmarks, full independent image inspection,
or clean-target platform builds. Prior reports retain those evidence boundaries.
Physical x86_64 hardware and remote CI execution remain unobserved; Linux execution evidence is
cross-build plus actual QEMU execution. No stronger platform claim is accepted here.
