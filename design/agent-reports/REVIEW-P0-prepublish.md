# REVIEW — P0 pre-publish gate (`mnemonic-io-lib 0.1.0`)

**Reviewed:** commit `1db1e81` on `review/p0-prepublish`, the whole P0 diff
(29 files, +3569/−217), against `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`
§4 rows 1–10 and §6 conditions 1–10. Reviewer had no prior context on this work.
**Method:** executed, not read — every gate was made to fail by mutating the
shipped code, and every §6 measurement was re-taken independently rather than
transcribed from `IMPL-P0-log.md`. Nothing below is quoted from the
implementer's report without an independent re-measurement.

**Verdict: SAFE TO PUBLISH — 0 Critical / 0 Important / 3 Minor / 2 Nit.**
None of the five findings is in the crate's shipped behaviour; two are in
`me-cli` (not being published), one is a test-coverage note whose underlying
claim this review verified by execution, two are cosmetic.

---

## 1. The publish surface (brief item 3) — CLEAN

| check | result |
| --- | --- |
| `[dependencies]` of `mnemonic-io-lib` | **empty** — no git dep, no path dep, nothing to refuse |
| `me-cli`'s git dep (`mt-codec`) | irrelevant: `me-cli` is not being published, and the **packaged `Cargo.lock` is pruned** — extracted from `target/package/mnemonic-io-lib-0.1.0.crate` and read: only the `tempfile` dev-dep chain, no git source anywhere |
| `cargo package -p mnemonic-io-lib` | succeeds, **including the isolated verify build** (compiles from the packaged form) |
| files shipped | 11 — `Cargo.toml{,.orig}`, `Cargo.lock`, `.cargo_vcs_info.json`, 7 `src/*.rs`. Nothing extraneous |
| version / license / description | `0.1.0` / `MIT OR Unlicense` / present; repository + homepage set; 4 keywords, valid category |
| standalone | `cargo nextest run -p mnemonic-io-lib`: 2/2 |

`cargo publish` was **not** run, in any form; `cargo package` is local-only.

## 2. Public API (brief item 4) — verified on code lines

Comment-aware scan (grep for `EXIT_|Class`, then discard `//`-prefixed lines):
**0 hits on code lines** across all 7 files. `no_records_guard` returns
`Result<Vec<String>, String>` (`records.rs:52`); `write_block` returns
`WriteBlock` (`exit.rs:63`); `mode_of`/`stdout_mode` return the raw mode
(`fd.rs:46,73` — no mask, confirmed by mutation, below). No exit integer, no
`Class` name, no binary→code mapping anywhere in the crate.

## 3. Every gate proven able to fail (brief items 1, 2, 5)

Twelve mutations, each applied to the shipped code with an exactly-once match
assertion, run, and reverted. Final tree confirmed pristine (`git status`
empty) and green (**423/423, 1 skipped** — the pre-existing
`preview::planted_path_sidecar_ignored`).

| row | mutation | result |
| --- | --- | --- |
| 2 + 7 site 1 | `refuse_write_block` Terminal arm 2→3 | **full suite: 413 run, 3 failed — all three in the NEW `terminal_destination.rs`, every pre-existing test green.** F-265 site 1 reproduced and now caught |
| 7 site 2 | WorldReadable arm 2→3 | `a_world_readable_stdout_is_usage_two` RED |
| 7 site 3 | `read_records` `--in` error 2→3 | `an_unreadable_in_file_is_usage_two` RED |
| 7 site 4 | stdin error 2→3 | `unreadable_stdin_is_usage_two` RED |
| 7 site 5 | `emit` write failure 2→3 | `a_failed_write_is_usage_two` RED |
| 3 | `& 0o044` mask reintroduced in `fd.rs` | the 0620 assertion RED |
| 4 | `Terminal(_)` + hard-coded `Bearer` (F-259 re-written) | `a_wipe_image_is_never_called_bearer` RED |
| 5 | zsh recipe reverted to the plan's prescribed `fc -W; sed -i; fc -R` | purge gate RED **at the purge assertion**, HISTFILE shown holding the planted secret — the harness genuinely runs zsh |
| 6 | guard call disabled | **225/450 cross-product rows leak** + ordering + override-scope RED |
| 6 | trim+lowercase dropped | **90/450 leak, 0 canonical** — the near-miss axis is load-bearing |
| 8 | drop walk 3 / drop `Admission` / `Class`-keyed cards | incomplete-mt1 / false-refusal / no-cosigner gates each RED |
| 9 | splice → append | byte-equality gate RED |

**The 35 new tests are real** (item 2): `comm` over `#[test]` fn names at
`d281b0b` vs `1db1e81` — **35 added, 0 removed, 0 renamed**; and the mutations
above show them failing when their production change is reverted. Rows 1, 9b,
10 are regression-gated as the plan itself states (§4 M5), verified
structurally: `pub const EXIT` count in `main.rs` = 0, `me-cli/src/io.rs` gone,
crate builds standalone.

**F-265 is closed** (item 5): all five sites, five mutations, five REDs — and
the site-1 full-suite run confirms the pre-existing 388 alone would still miss
it, i.e. the new tests are the only thing standing there.

## 4. §6 conditions re-measured independently

Condition 2: 16/16 verbs present on `md`/`mk`/`ms`/`mt` (by absolute path; the
reviewer's own first harness produced 16× false ABSENT via zsh word-splitting —
re-measured with a fixed harness). Condition 3: `mnemonic inspect` bad HRP → 2,
undecodable `md1` → 1, `mnemonic decode` → 64. Condition 8: the ordering test
passes and mutation M-F makes it fail. Conditions 4, 5, 6, 9, 10: covered by
the row 8, 5, 4, 5, 7 mutations above. Condition 11 is this review.

Journey probes on the real binary: `me <mt1>` on argv → guard refusal rc 3 (by
design, bearer); the documented conversion survives on stdin
(`printf mt1… | me --hex` → rc 0). Bare `me` never accepted a positional
payload (its slot is a subcommand — identical pre/post), so no regression.

## 5. Findings

### Minor 1 — the override's surface detection can be spoofed by a flag VALUE, defeating the "binds only where declared" property
`crates/me-cli/src/main.rs:371` (`argv_override_applies`). The surface is taken
as "the leading run of non-flag tokens", so a flag value occupying that run is
mistaken for a subcommand word. Constructed counterexample, run on the shipped
binary:

```
me --out seal ms10entrs… --allow-argv-secret
  → rc 2, clap: error: unrecognized subcommand 'ms10entrs…'   ← full secret on stderr
```

"seal" here is `--out`'s value, not a surface; bare `me` does not declare the
flag, yet the guard stood down and clap echoed the secret — the exact property
`allow_argv_secret_binds_only_where_it_is_declared` asserts, violated on a
shape it does not cover. Graded **Minor**, not Important: every triggering argv
already carries the operator's explicit `--allow-argv-secret` (the declaration
that argv is safe where they are), the collision requires a value literally
`seal` (or `sysw` `pack` leading), and the scope property itself entered the
plan as round-11 **M**-2. Close: require the surface words to be argv's
*leading* tokens (`argv[1]`, `argv[2]`), not the leading non-flag tokens, plus
a cross-product row for the value-spoof shape.

### Minor 2 — the guard's refusal message says the flag "is declared on `sysw pack` alone"; `seal` declares it too
`crates/me-cli/src/main.rs:494` (message text). Measured: `me seal <ms1>`
prints *"That flag is declared on `sysw pack` alone, so it does not buy past
this refusal anywhere else"* — while `seal` declares `--allow-argv-secret`
(`main.rs:146`) and this same commit's own test proves `seal … --allow-argv-secret`
exits 0. A seal-fixture user is told their documented channel does not exist.
(The message's private-channel example also names `sysw pack` on every surface,
including `seal`'s refusal.) Close: derive the declared-surface list in the
message from the same fact `argv_override_applies` uses, or just say
"on `sysw pack` and `seal`".

### Minor 3 — the bash purge recipe ships in the published crate with no test executing it (zsh's is executed; bash's is not)
`crates/mnemonic-io-lib/src/remedy.rs:83-87`; `tests/history_purge.rs` runs
only the zsh recipe. **This review ran the emitted bash recipe** under a real
interactive bash on a pty, with a no-purge control: control → planted secret on
disk (harness records); recipe → secret gone. So the shipped claim is TRUE and
this is a coverage gap, not a defect — but it is the one finding touching the
published crate, and recipe text that nothing executes is how F-264 shipped the
first time. Close: a `the_emitted_bash_recipe_actually_purges_the_entry` twin
(the harness is already surface-shaped for it), owning phase P1 alongside F-273.

### Nit 1 — broken intra-doc link, visible on docs.rs
`crates/mnemonic-io-lib/src/channel.rs:5`: `[`super::exit`]` in the module's
inner doc does not resolve (`cargo doc`: *unresolved link to `super::exit`*,
the crate's only rustdoc warning). Use `[`crate::exit`]`.

### Nit 2 — the crate has no README
No `readme` field and no file; the crates.io page will show only the
description (docs.rs carries the substantial `lib.rs` doc, so this is
cosmetic). Optional: point `readme` at a short file before some later 0.1.x.

## 6. Out of scope, respected

F-267 and F-268 untouched (both verified still open in `FOLLOWUPS.md`, with
F-267's residue pinned by `the_two_shapes_out_of_reach_are_pinned`). Row 11 not
run — no `cargo publish`, no `--dry-run`. The §5 name-check is the operator's
to re-run at publish time.

## Verdict

**SAFE TO PUBLISH (0C/0I).** The crate is dependency-free, policy-free on code
lines, packages and verify-builds in isolation, ships nothing extraneous, and
every gate in the plan's §4 was demonstrated to fail against the shipped tree.
The three Minors are donor-side or coverage notes; none changes a byte of
`mnemonic-io-lib 0.1.0`.
