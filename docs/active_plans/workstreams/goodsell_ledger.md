# Goodsell native implementation ledger

This ledger tracks the approved work in [goodsell_native.md](../active/goodsell_native.md). It is separate from the plan so that the plan remains the source of product intent.

## Status

| ID | Owner | Status | Prerequisites | Evidence |
| --- | --- | --- | --- | --- |
| M1 | reference_spec | review corrections | None | `reference_spec` corrections complete; fresh re-review by `spec_review` pending. |
| M2 | oracle | in progress | M1 | Oracle work underway. |
| M3 | native_contracts | contract investigation | M1 | Awaiting release of shared contract. |
| M4 | pending | pending | M3 | Pending. |
| M5 | pending | pending | M3 | Pending. |
| M6 | pending | pending | M4, M5 | Pending. |
| M7 | pending | pending | M6 | Pending. |
| M8 | pending | pending | M7 | Pending. |
| M9 | pending | pending | M8 | Pending. |
| M10 | pending | pending | M8 | Pending. |
| M11 | pending | pending | M9, M10 | Pending. |
| M12 | pending | pending | M11 | Pending. |
| M13 | pending | pending | M12 | Pending. |

## Decisions

| ID | Decision | Rationale | Affected milestones | Evidence |
| --- | --- | --- | --- | --- |
| P1 | Treat `goodsell_native.md` as the source of user intent. | The user supplied a complete approved plan. | M1-M13 | This ledger and plan artifact. |
| P2 | This plan transcription uses inherited context; later bounded work uses smaller `gpt-6.1-sol` or `gpt-6-luna` delegates. | Preserve the complete approved plan accurately, then use the smallest capable delegate. | M1-M13 | Dispatch records. |
| D1 | Correct undefined source state by initializing the background halo and `type(0)`, resetting per-pixel and kernel 3/4 accumulators, and using `pfogh` for occupied pixels and `1` for background pixels when occupied depth spread is zero. | Undefined or invalid Fortran behavior must be deterministic in native rendering. | M1, M3, M8-M10 | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); pending review. |
| D2 | Use the shared Rust contract to be recorded at `docs/active_plans/decisions/goodsell_contract.md`. | Native milestones must implement one consistent contract. | M3-M13 | Contract decision record; awaiting release. |
| D3 | Native automatic-depth centering uses physical maximum radius in Angstroms plus one; preserve the original double-scaled equation as legacy documentation. Keep rotate, center, translate, scale order; match camera centers for raw-depth comparisons. | Correct the automatic-depth offset without changing operation order or raw-depth comparison conditions. | M1, M3, M7-M8, M12 | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); pending review. |

## Decision propagation

| Task | Constraint | Restatement | Application and evidence | Next owner |
| --- | --- | --- | --- | --- |
| M1-M13 | P1: the active plan controls scope and acceptance. | Pending each owner confirmation. | Pending. | Assigned owner. |
| M1-M13 | P2: use a bounded smaller delegate after transcription. | Pending each dispatch. | Pending. | Manager. |
| M3, M8-M10 | D1: initialize the background halo and `type(0)`; reset per-pixel and kernel 3/4 accumulators; apply the exact flat-depth fog weights. | Contract and implementation must state each initialization/reset and use `pfogh` for occupied pixels and `1` for background pixels at zero occupied depth spread. | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); contract investigation and pass work. | `native_contracts`, `oracle`, future render-pass owners |
| M3 | D2: use the shared Rust contract. | Match the contract decision record once released. | `docs/active_plans/decisions/goodsell_contract.md`; awaiting release. | `native_contracts` |
| M7-M8, M12 | D3: use Angstrom `rmax` plus one for native automatic-depth centering, with legacy equation retained. | Preserve rotate, center, translate, scale order; raw-depth oracle comparisons use an explicitly matched camera center; full-scene projections may frame radii-inclusive bounds. | [REFERENCE_SPEC.md](../../REFERENCE_SPEC.md); pending review. | `native_contracts`, `oracle`, future render-pass owners |

## Review route

Each milestone requires a fresh specification reviewer, then a different fresh quality reviewer. Corrections and re-reviews use fresh agents. A fresh integration review accepts the completed composition before M13 closes the plan.
