# Oracle regeneration lifecycle

The developer oracle now works after removal of the local historical source checkout:

```bash
./devel/run_reference_oracle.sh
```

The existing shell wrapper establishes the repository Python environment. Functional work remains
in [devel/reference_oracle.py](../../../devel/reference_oracle.py). Only this developer workflow
requires `gfortran`; the native library, CLI, installation and CI have no Fortran requirement.

## Pinned temporary inputs

Regeneration uses upstream `ccsb-scripps/Illustrate` revision
`e469df4f794f8edbca992a9b2ab534d11d27cdd6`. Each required file is copied from the ignored local
reference checkout if present, otherwise downloaded from its commit-pinned public raw URL.
Every byte hash is checked before compilation. A mismatching local or downloaded file aborts.

| File | SHA-256 |
| --- | --- |
| `illustrate.f` | `6af0ee0aa2142b72a0fe2655cda371b1ffc17c75f24fb36fab7987fd264fe88b` |
| `2hhb.pdb` | `b2d9a4a3ec1cf16759c2ffc85f1088ef2100e02ececedf20025b9e54d23f3db6` |
| `2hhb.inp` | `7439e5f258504193f244e6cf9a7bd11389fce10a62c8faaed10591329c7fce84` |

The manifest records each file's full public URL, hash and local/download origin. Both source
variants and executables live in `TemporaryDirectory`, automatically removed on success or
exception. The short deterministic patch recipe remains in the Python generator; no complete
historical source or source patch is newly retained. Image, opacity, coordinate, control, depth/atom,
compile/run log and manifest evidence remains in `output_oracle/`.

Recorded compiler and renderer commands describe temporary paths from that run. They are provenance,
not literal commands that can be replayed after cleanup; use the launcher to regenerate.

The new deterministic source hash is
`c84e2bf5854d06d4baf982e7b56fac440e3f72e314a69587af802ede4881f69e`.
Only its leading comment changed from the prior source hash: it now points to the Python patch
recipe rather than a retained `deterministic.patch`. All D1 formulas and scene inputs are unchanged.

## Verification on 2026-11-11

```bash
source source_me.sh && python3 -m pyflakes devel/reference_oracle.py tests/_temp/test_oracle_lifecycle.py
source source_me.sh && python3 -m pytest -q tests/_temp/test_oracle_lifecycle.py
./devel/run_reference_oracle.sh
```

All three temporary checks passed in 14.34 seconds: reject altered source before generation;
remove temporary source/executable after simulated compiler failure; and regenerate with a genuinely
absent reference path using the three public downloads. The last check wrote
`output_oracle_lifecycle/`, compared all 28 complete render records to the previous manifest, and
verified no source, patch or executable remained. Each of the 28 scenes ran twice. Image, opacity,
depth/atom, input/control hashes and geometry metadata were all exactly unchanged.

The subsequent standard launcher regenerated `output_oracle/` through the verified-local path.
All 28 render records also matched the independent download run. Pyflakes passed.
Cross-compiler repeatability, native fidelity and release acceptance are separate checks.

## Complete verifier lifecycle on 2026-11-11

The oracle now also generates the five isolated hemoglobin references (plain, shadow, contour,
subunit and residue) in both source variants before deleting its temporary executables. The
native verifier consumes their persistent evidence directly and never invokes Fortran. Current
regeneration produces 38 reference records (19 scenes per variant), each rendered twice; the
28-record comparisons above describe the earlier lifecycle check.

```bash
./devel/run_reference_oracle.sh
CARGO_HOME=/Users/vosslab/.cache/goodsell_cargo cargo build --release --bin illustrate --example render_passes
source source_me.sh && python3 devel/verify_rendering.py
source source_me.sh && python3 -m pytest -q tests/_temp/test_oracle_verifier_lifecycle.py
source source_me.sh && python3 -m pytest -q tests/_temp/test_oracle_lifecycle.py
```

These commands passed after deletion of the stale `output_oracle/illustrate_deterministic`
executable. Both snapshot checks passed: all original 28 complete reference records were identical;
the five moved deterministic references retained exact control, PDB, image, opacity and depth-map
bytes. All 19 native comparisons retained their numeric metrics, input/settings hashes, and image
hashes; all six sample image hashes also matched. Only measured timing values were excluded.
Every native diagnostic composition passed the probe's public-render equality check. Pyflakes,
ASCII inspection of the changed Python files, and `git diff --check` passed.
The three lifecycle checks also passed in 37.70 seconds with 38 records, including a fresh
download-only regeneration and verification that its temporary compiler/renderer paths were gone.

## Completed cleanup inventory

The stale `output_oracle/illustrate_deterministic` executable was deleted for the regression above.
Final cleanup is complete; [goodsell_cleanup.md](goodsell_cleanup.md) records removal of
`OTHER_REPOS/Illustrate/` including its history, all old oracle source/patch/executable copies,
and temporary lifecycle, verifier, staging, and visual-review checks under `tests/_temp/`.
The commands above and below that name those removed checks are historical verification records,
not commands runnable from the completed checkout.

Oracle image/data/log/manifest evidence, including `output_oracle_lifecycle/`, remains retained
and ignored. `LOCAL_ONLY/` papers and web references remain preserved. One 43-line permanent
offline preservation test remains in [test_reference_oracle.py](../../../tests/test_reference_oracle.py);
it needs no source download, Fortran compiler, or old executable. Regeneration uses verified
public pinned downloads after local-source deletion; native verification consumes saved evidence.

## Staged evidence replacement on 2026-11-11

Regeneration now completes all 38 validated render records in a sibling staging directory before
replacing `output_oracle/`. The 24-line `regenerate()` orchestration adds 18 lines to its previous
implementation; it uses the existing standard-library directory tools without another helper,
configuration option or dependency. Staging and backup names begin with `output_oracle_stage_`,
so the existing `/output*/` ignore rule covers both during regeneration.

The known scientific contact sheet `spheres.webp` is copied into the completed staged tree before
promotion. The old evidence directory is renamed to a sibling backup, and restored if the staged
directory rename raises `OSError`. The backup is deleted only after successful promotion. It is
outside automatic temporary cleanup: if the recovery rename also fails, the old evidence survives
at `output_oracle_stage_<id>.previous` for manual restoration. Temporary source/executable and
staging workspaces still clean up after normal completion or exceptions. This is exception
recovery for a developer workflow; it does not guarantee uninterrupted reads or process-crash
recovery during the two directory renames. Recorded render commands now also name ephemeral
staging directories, consistent with the manifest's existing temporary-command lifecycle note.

```bash
source source_me.sh && python3 -m pytest -q tests/_temp/test_oracle_staging.py tests/_temp/test_oracle_lifecycle.py -k 'not absent_reference'
source source_me.sh && python3 -m pyflakes devel/reference_oracle.py tests/_temp/test_oracle_staging.py
./devel/run_reference_oracle.sh
git diff --check
```

All six selected temporary checks passed: reject a pinned-source mismatch; clean up temporary
source after generation failure; preserve accepted manifest, image and contact sheet after partial
generation; restore them after promotion failure; retain their backup after rollback failure; and
replace the old evidence successfully while preserving the contact sheet. The four new staging
checks occupy 65 lines in `tests/_temp/test_oracle_staging.py` for closeout review/removal.

The standard launcher ran twice, each producing all 38 records with two renders per record.
Both complete `renders` lists and upstream provenance matched the accepted pre-staging snapshot
`/tmp/goodsell_oracle_before_staging.json` exactly. The final run exercised contact-sheet copying;
its bytes matched `/tmp/goodsell_oracle_spheres.webp`. No staging or backup directories remained.
Successful tree replacement removed the four deprecated root artifacts
`illustrate_deterministic.f`, `deterministic.patch`, `illustrate_original` and `illustrate_checked`.
The source checkout in `OTHER_REPOS/Illustrate/` was subsequently removed in final cleanup.
Pyflakes, ASCII inspection of the two changed Python files, and `git diff --check` passed.
The download-only test and Rust benchmarks were not repeated for this orchestration change.
