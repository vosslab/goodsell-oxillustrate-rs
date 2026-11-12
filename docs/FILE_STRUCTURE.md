# File structure

## Top-level layout

| Path | Purpose |
| --- | --- |
| [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock) | Native package, dependency floors, and locked resolution |
| [src/](../src/) | Library, CLI, parsers, scene preparation, render passes, PNG output |
| [examples/hemoglobin.json](../examples/hemoglobin.json) | Scene settings and render-pass evidence example |
| [tests/](../tests/) | Rust CLI integration tests and repository maintenance checks |
| [devel/](../devel/) | Development and reference-comparison tools |
| [tools/](../tools/) | Repository support tools |
| [.](.) | User guides, architecture, reference mathematics, and project history |
| [source_me.sh](../source_me.sh) | Python development environment setup |
| [AGENTS.md](../AGENTS.md) | Repository agent instructions and canonical rule links |
| [LICENSE.AGPL-3.0](../LICENSE.AGPL-3.0) | Native source license |
| [LICENSE.CC-BY-4.0](../LICENSE.CC-BY-4.0) | Documentation and non-code license |

This is a single root crate; there is no nested package directory. See
[CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) for the module map and data flow.

## Examples and evidence

[hemoglobin.json](../examples/hemoglobin.json) contains the source-derived hemoglobin appearance
settings. [sample.pdb](../examples/sample.pdb), [sample.cif](../examples/sample.cif), and
[sample.xml](../examples/sample.xml) contain original synthetic four-atom geometry; provenance
is in [examples/README.md](../examples/README.md). [render_passes.rs](../examples/render_passes.rs) supports isolated render-pass evidence.
The public coordinate download and rendering command are in [USAGE.md](USAGE.md).

[docs/active_plans/](active_plans/) retains shared decisions, the workstream ledger, and evidence
reports. The completed plan is [goodsell_native.md](archive/goodsell_native.md); its original intent
and dated amendments remain historical. All M1-M13 milestones and final independent integration
review are accepted; [goodsell_ledger.md](active_plans/workstreams/goodsell_ledger.md) links evidence.

## Generated artifacts

[.gitignore](../.gitignore) excludes Cargo `target/`, `output*/`, and `tests/_temp/`.
These directories hold compiled products, rendered comparison evidence, and disposable checks.
Rust documentation is generated in `target/doc/`.

`LOCAL_ONLY/` and `OTHER_REPOS/` are ignored development reference collections, not distribution
or build dependencies. Durable external provenance appears in [REFERENCE_SPEC.md](REFERENCE_SPEC.md).
Oracle regeneration fetches verified pinned sources when needed and removes its temporary source
and executables automatically; only data, logs, maps, images, and provenance persist. Preexisting
legacy reference copies were removed after final comparisons and acceptance.

## Documentation map

- [../README.md](../README.md): project purpose and first render.
- [INSTALL.md](INSTALL.md) and [USAGE.md](USAGE.md): setup and practical workflows.
- [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md): public pipeline and module ownership.
- [REFERENCE_SPEC.md](REFERENCE_SPEC.md): scientific sources and executable mathematics.
- [CHANGELOG.md](CHANGELOG.md): dated changes and verification evidence.
- [HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md) and [DESIGN_DECISIONS.md](DESIGN_DECISIONS.md):
  durable user guidance and settled implementation choices.

Add native code under [src/](../src/), runnable examples under [examples/hemoglobin.json](../examples/hemoglobin.json), integration
coverage under [tests/](../tests/), and durable user documentation directly under [.](.).
