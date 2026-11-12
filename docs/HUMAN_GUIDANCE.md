# Human guidance

<!-- VENDORED HEADER: START -->
Record the durable guidance Neil Voss states, or approves for preservation here, in his own words:
first person or close paraphrase, one to three lines per bullet. Material he supplies as a source
may inform [DESIGN_DECISIONS.md](DESIGN_DECISIONS.md) once it is settled, and an entry of uncertain
origin belongs there too. Rules: [REPO_STYLE.md](REPO_STYLE.md).
[PROPAGATED HEADER - ENTRIES BELOW ARE YOURS]
<!-- VENDORED HEADER: END -->

- 2026-11-11: Reimplement the code using [LICENSE.AGPL-3.0](../LICENSE.AGPL-3.0).
  The old code is barely usable and serves only as an implementation and goal reference;
  delete it when the new implementation and comparisons are done.

- 2026-11-11: Use the conventional single-crate root layout; the extra `oxillustrate/src/`
  directory layer adds no value. Keep the manifest, lockfile, and `src/` at the repository root.

- 2026-11-11: Remove root `LICENSE.Apache-2.0` and `NOTICE`; keep native code under AGPL-3.0
  and scientific attribution in the README and reference documentation.
