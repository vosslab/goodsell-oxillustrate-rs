# Design decisions

<!-- VENDORED HEADER: START -->
Record each durable decision about how this code and repository are shaped, once it is settled, with
the reasoning a later reader needs. Guidance Neil Voss states belongs in
[HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md), dated history in `docs/CHANGELOG.md`, open discussion in
`docs/active_plans/decisions/`. [PROPAGATED HEADER - ENTRIES BELOW ARE YOURS]
<!-- VENDORED HEADER: END -->

Write each decision as a level-three heading with these four fields. `Owner` names the
authoritative code or contract document, rather than a person.

```markdown
### <decision title>

**Decision.** <the durable direction>

**Why.** <the reason it was chosen>

**Consequence.** <the constraint a future change preserves>

**Owner.** <the authoritative code or contract doc>
```

### Native licensing and reference lifetime

**Decision.** Implement the documented mathematics and behavior as new native Rust code under
[LICENSE.AGPL-3.0](../LICENSE.AGPL-3.0), with scientific attribution in [NOTICE](../NOTICE).
The legacy Fortran is a temporary development reference and comparison oracle.

**Why.** The user's 2026-11-11 amendment specifies an AGPL reimplementation and deletion of the
old code when done. The reference was inspected; this is not a clean-room claim.

**Consequence.** After final fidelity evidence, delete legacy source and compiled copies,
including oracle output copies. Preserve images, hashes, commands, and provenance. Developer
regeneration may fetch a temporary upstream snapshot; end users need no legacy code or Fortran.
If copied upstream code is ever distributed, preserve its applicable license and notices.

**Owner.** The dated amendment in
[goodsell_native.md](active_plans/active/goodsell_native.md) and
[oxillustrate/Cargo.toml](../oxillustrate/Cargo.toml).
