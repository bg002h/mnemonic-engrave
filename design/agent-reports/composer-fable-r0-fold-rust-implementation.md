# Fold, part A — descriptor-mnemonic (the Rust PRIMARY): implementation report

**Verdict: BOTH FINDINGS IMPLEMENTED, GREEN.** Branch `fable-r0-fold` in
`/scratch/code/shibboleth/.tmp/fold-rust`, base
`922778ad0400957840da26081846fc761deaca41`, tip
`4de55155f7bb5dda6896f733331f9ccd97275490`. Two commits, one per finding.
Nothing pushed; nothing on `master`; no sub-agents; no `.jsonl` read; all
building under `/scratch`, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/fold-rust-target`.

---

## Commits

| SHA | Subject |
| --- | --- |
| `3d22b714` | md-codec 0.45.0: a policy holds at most one key-less spend path |
| `4de55155` | md-cli: host seating fills an ABSENT slot fingerprint from the seated card |

Diff against base: 15 files, +1069 / −25.

---

## The gate

Run on the committed tree (tip `4de55155`), once, captured:

```
Summary [  19.275s] 1354 tests run: 1354 passed, 3 skipped
```

- `cargo nextest run --locked --no-fail-fast` → **1354 passed, 0 failed, 3 skipped**
  (base was 1346; the fold adds 8 tests — 8 in `compose_keyless_cap.rs`,
  4 in `i2_partial_template_completion.rs`, 4 new unit rows in
  `seat/compose.rs`, minus the pre-existing rows they sit beside).
- `cargo fmt --all --check` → **clean** (no output, exit 0).
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  under the **pinned** toolchain 1.85.0 (`clippy 0.1.85`, which is what
  `.github/workflows/ci.yml` runs) → **clean**.

**One deviation worth stating on clippy.** A bare `cargo clippy` on this box
resolves clippy **0.1.98**, not the `rust-toolchain.toml` pin, and 0.1.98
reports 4 errors — `manual_slice_fill`-class ("the loop variable `c` is only
used to index `a`", ×2) in `crates/md-codec/tests/parity_smoke.rs`, plus
`clippy::contains_key`/`useless_borrows_in_formatting` in
`crates/md-cli/src/seat/mod.rs`. **All four are pre-existing and in files this
fold does not touch** (neither path appears in the diff stat above); they are
lints that do not exist in 0.1.85. The gate is the pinned clippy, and it is
clean. I did not fix them — out of scope, and they would be a separate commit.

---

## Finding 1 — refuse a second key-less path at validate time (`3d22b714`)

### Machine check BEFORE implementing

The brief states the rule from measurement; I re-measured it independently
rather than taking it, because the error message names two specific paths and
the "wherever they sit" clause is what makes that sound. **859 `wsh` path lists**
of 2 to 4 paths, built from five keyed and four key-less atoms, driven through
`md compose --wrapper wsh --experimental` at the base's md-cli 0.16.2:

```
tallies (>=2 keyless, refused, via-readback):
   (False, False, False) 355
   (True, True, True) 504
counterexamples: 0
```

Every list with ≥2 key-less paths was refused after lowering, every list with
≤1 was emitted, and every refusal came through the F-600 read-back. The rule is
exact, not a heuristic, and non-adjacency does not rescue it.

I also probed for a shape that still reaches the read-back after the new rule —
mixed absolute lock kinds across paths, 32-slot wsh and tr, eight single-key
paths, nine-key paths — and found none. Recorded below as a residue.

### What landed

- `md-codec` `compose::validate()` now refuses ≥2 key-less paths with its own
  variant `ComposeError::TooManyKeylessPaths { first, second }`, **after** the
  `NoKeyedPath` check and after the in-loop `KeylessUnderTr`, so the more
  specific rule still wins where one applies. Both orderings are pinned by
  tests.
- The message, in the F-600 voice and saying what is true:

  > `paths 2 and 3 both have no key; side by side they lower to a malleable `or_i` that `md encode` refuses and no wallet will import. Give one of them a key, a timelock does not help, or fold them into one path`

- **The md-cli read-back stays**, as the brief says. Its *hint* did not: it said
  *"give one of them a key, a timelock, or fold them into one path"*, and a
  timelock is exactly what does not work (measured above — `[keyed, K, K+older(5)]`
  is refused). Since the two-key-less shape can no longer reach the read-back,
  the hint also named a cause that can no longer occur there, so it now names
  the parser's reason and stops guessing. Flagged as a small consequent change,
  not in the brief.
- CHANGELOG entries for both crates; **md-codec 0.44.2 → 0.45.0**, **md-cli
  0.16.2 → 0.17.0**, the `=0.45.0` path-dep pin, and `Cargo.lock` refreshed
  with `cargo update -p md-codec -p md-cli --offline`.

### The conformance vector — NAME IT WHEN VENDORING

```
crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json
```

Four cases: three **refused** (`[keyed, K, K]`; `[keyed, K, K+older(5)]`; and
`[K, keyed, K]` — the non-adjacent one, which the port needs or it will
implement "adjacent" and pass) and one **admitted** control
`[K+older(5), 2of3]` with its exact rendered template. It also records the
`precedence` rules and the `error_fields` shape.

Notes for the fork implementer:

- The file name begins `compose_`, so `scripts/vendor-compose-vectors.sh`
  picks it up with no change to the script.
- `md/compose_vectors_pin_test.go`'s hand-maintained `composeVectorNames` will
  need `"compose_refusal_keyless_cap"` added, or the pin fails with a
  compose-named file the list does not carry. That is by design in that test.
- It is **hand-written, not generated**: `md vectors` exports admitted policies
  (template, card, addresses) and a refusal has none of those. It is therefore
  excluded from `md-cli/tests/vector_corpus.rs`'s corpus-drift diff (with a
  comment saying why), exactly as `bip341-wallet-test-vectors.json` already is.
  Its anti-drift gate is instead
  `crates/md-codec/tests/compose_keyless_cap.rs::every_case_in_the_conformance_vector_behaves_as_recorded`,
  which **drives** every case — including the exact operator-facing message —
  rather than reading it.

### RED output, before the fix

`cargo test -p md-codec --test compose_keyless_cap`, with the enum variant and
its `Display` arm in place but no rule in `validate()`:

```
running 7 tests
test a_keyless_path_under_tr_still_reports_the_tr_rule ... ok
test a_policy_with_no_keyed_path_still_reports_that ... ok
test a_timelock_on_the_second_keyless_path_does_not_rescue_it ... FAILED
test one_keyless_path_is_still_admitted ... ok
test the_refusal_says_a_timelock_does_not_help ... FAILED
test the_two_keyless_paths_need_not_be_adjacent ... FAILED
test validate_refuses_a_second_keyless_path ... FAILED

---- a_timelock_on_the_second_keyless_path_does_not_rescue_it stdout ----
panicked at crates/md-codec/tests/compose_keyless_cap.rs:109:6:
called `Result::unwrap_err()` on an `Ok` value: 1
---- the_refusal_says_a_timelock_does_not_help stdout ----
panicked at crates/md-codec/tests/compose_keyless_cap.rs:135:87:
called `Result::unwrap_err()` on an `Ok` value: 1
---- the_two_keyless_paths_need_not_be_adjacent stdout ----
panicked at crates/md-codec/tests/compose_keyless_cap.rs:123:87:
called `Result::unwrap_err()` on an `Ok` value: 3
---- validate_refuses_a_second_keyless_path stdout ----
panicked at crates/md-codec/tests/compose_keyless_cap.rs:86:87:
called `Result::unwrap_err()` on an `Ok` value: 1

test result: FAILED. 3 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
```

The three that passed are the controls (single key-less admitted, `tr`
precedence, `NoKeyedPath` precedence) — they pass in RED on purpose, so a rule
that refused every key-less path would not have satisfied this file.

### An old test that the new gate made stale

`md-cli/tests/cli_compose_encode_gate.rs::compose_refuses_to_emit_a_template_encode_would_reject`
asserted the read-back's own wording (`"`md encode` refuses"`). The shape it
drives is now refused one layer earlier, so those assertions were about a path
it no longer takes. It is **retargeted, not deleted**: the contract it pins
(non-zero exit, empty stdout, the message names the paths, the rule, and a
remedy that works) is unchanged and stronger, and its doc records which layer
refuses and the mutation for each half.

---

## Finding 2 — host seating fills an ABSENT fingerprint (`4de55155`)

### What landed

`crates/md-cli/src/seat/compose.rs::compose` now makes **two** edits to the
keyless policy instead of one: `tlv.pubkeys`, and an **absent** per-slot entry
in `tlv.fingerprints` taken from the card that seated that slot. The module doc
is rewritten to state the rule in three clauses — declared is never
overwritten; absent is filled; nothing is invented when the card states no
master — and to record why it matters (`to_miniscript` drops the whole origin
when the fingerprint is absent, so the completed key rendered with no
`[fp/path]` at all).

A card's fingerprint is carried **verbatim, all-zero included**: `00000000` is
md1's absent-master sentinel (md-codec 0.44.1) and transcribing it is what
keeps the host's id agreeing with the device's, which writes the same bytes.
Stated in the module doc.

### The three tests the brief named

(a) **`the_completed_descriptor_carries_an_origin_on_the_host_seated_key`** and
**`the_completed_card_declares_a_fingerprint_for_every_slot`** — the C13 shape:
a partial template (two seated `key:` slots at account 0, one unseated at the
lowest free account 1) completed from a host-minted **3-chunk** card; the
completed descriptor carries `[fp/path]` on the third key, and `md inspect
--json` shows three fingerprints.

(b) **`the_completed_wallet_id_equals_a_full_seating_of_the_same_three_keys`** —
both sides are **built in the test**: route A is the host completion, route B
is `md encode <template> --key @i --fingerprint @i` for all three slots.

(c) **`seating_never_overwrites_a_fingerprint_the_policy_already_declares`**
(unit, in `seat/compose.rs`). Pinned as **inheritance at `compose`, refusal at
the CLI** — which is what the existing code does. A2 (`satisfy::satisfies`)
refuses a card whose fingerprint disagrees with a declaration, so no CLI input
can reach that code path; the test therefore calls `compose` directly with the
assignment A2 would reject, and first **asserts that `satisfies` really does
reject it** so the row cannot pass by testing a benign case. Plus
`a_fingerprint_free_card_leaves_an_absent_declaration_absent` — a
privacy-preserving card invents nothing.

Also `the_c13_fixture_really_is_a_partial_template`, which measures the
premise (@2 undeclared, the card states a fingerprint) so nothing above can
pass for the wrong reason.

### Fixture

```
crates/md-cli/tests/fixtures/seating/v-partial-c13.txt
```

Added to `crates/md-cli/tests/fixtures/seating/generate.sh` as a new block;
re-running the whole generator left **every other fixture byte-identical**
(`git status` showed only `generate.sh` modified and the one new file), which
is that script's own rot check.

It reproduces **the report's own template stub `b02b4403`** — the template id
is key-stable, so substituting `b8688df1` for the report's `3f635a63` (not
present in `fixtures/pathological/keys.txt`) does not move it. That is the
evidence the shape is the reported one rather than a lookalike.

### Behaviour change, MEASURED not reasoned

CHANGELOG says a host-completed partial template's id changes from 0.16.x, and
why. I measured the blast radius with before/after binaries rather than
asserting it:

- **34 seating fixtures driven through `md descriptor --from-mk1` on both
  binaries. 5 change their composed wallet id:** `v-ap-row1-e2e` (52bcbf43 →
  f0e43ed2), `v-bound-seat` (52bcbf43 → f0e43ed2), `v-ce1` (f35be217 →
  2537939a), `v-mix` (01dcbc20 → 89a3408b), `v-partial-c13` (ed6e9919 →
  af5424ab).
- **All five keep byte-identical addresses** (`md address`, both binaries) —
  origin metadata is not script content, so no funds move.
- **0 of 34 fixtures changed their `--emit md1` exit code.**
- **No stale id literal anywhere:** `grep -rn "52bcbf43\|f35be217\|01dcbc20\|ed6e9919" crates/ design/`
  returns nothing, so no test, fixture header or design doc quotes a value that
  moved.

### The one regression class this could have opened — probed and closed

Filling fingerprints could make two slots inherit the **same** `(fingerprint,
origin path)` with **different** xpubs, which `validate_origin_key_consistency`
refuses at encode — turning a seating that used to emit into one that fails. I
constructed it directly (a two-slot fp-free policy at one shared path, two
cards at that path minted with the same `73c5da0a` and different xpubs) and ran
it on both binaries:

```
--- before rc=1  md: seating refused: cards 85959 … and f5483 … both declare origin
                 [73c5da0a/48'/0'/0'/2'] yet carry DIFFERENT xpubs. …
--- after  rc=1  (identical)
```

`satisfy::check_no_impossible_card_pair` refuses the **card set** ahead of A2/A3
for every seating, so the input never reaches `compose`. The class is
unreachable, before and after.

### RED output, before the fix

`cargo test -p md-cli --test i2_partial_template_completion`:

```
running 4 tests
test the_fixture_header_still_records_this_mint ... ok
test the_completed_descriptor_carries_an_origin_on_the_host_seated_key ... FAILED
test the_completed_card_declares_a_fingerprint_for_every_slot ... FAILED
test the_completed_wallet_id_equals_a_full_seating_of_the_same_three_keys ... FAILED

---- the_completed_descriptor_carries_an_origin_on_the_host_seated_key stdout ----
no origin for 28645006 (key xpub6F6g…) in:
wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH/<0;1>/*,[b8688df1/48'/0'/0'/2']xpub6DXuQW1Q2JpZxfSWqC4Mt3brVA7zgxBsuuhnyTb5pQkAfr7DEMQLDryFy63hhhHCHT7CikTkHkphPTtav74CnBcqYNASXSj3Tbt4P5ect9r/<0;1>/*,xpub6DXuQW1Q2JpZwtNiGZ3gCp3GBFPZRd48NFedS6wPPkhFabkDxE7YiDG5VHpqTSCKudYSPavk3bVNC2eDLkZHiek2LSBDaRNAteAb8rAhfyD/<0;1>/*))#68yxl5as

---- the_completed_card_declares_a_fingerprint_for_every_slot stdout ----
assertion `left == right` failed: the completed card does not declare all three fingerprints
  left: [(0, "73c5da0a"), (1, "b8688df1")]
 right: [(0, "73c5da0a"), (1, "b8688df1"), (2, "28645006")]

---- the_completed_wallet_id_equals_a_full_seating_of_the_same_three_keys stdout ----
assertion `left == right` failed: a host-completed partial template mints a different
wallet than the same three keys seated in full
  left: "ed6e99192d02b8f10c3f632d952b817c"
 right: "af5424abf777f721eadad475d7ea490d"

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo test -p md-cli --bin md seat::compose`:

```
---- seat::compose::tests::seating_fills_an_absent_fingerprint_from_the_seated_card stdout ----
panicked at crates/md-cli/src/seat/compose.rs:444:9:
assertion `left == right` failed: @2's fingerprint was not inherited from the card that seated it
  left: None
 right: Some([40, 100, 80, 6])

test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 312 filtered out
```

The three no-overwrite / nothing-invented rows passed in RED, as controls
should.

### An old test that this narrowed

`composition_touches_only_the_pubkeys_tlv` → renamed
`composition_touches_only_the_pubkeys_and_absent_fingerprints`. Its
`tlv.fingerprints` assertion still holds on `PATHOLOGICAL` only because that
fixture declares a fingerprint for **every** slot — so the row now **measures
that premise first** (`declared.len() == policy.n`) before asserting the TLV
comes through untouched. Without that it would have become a row that passes
whether or not a fill exists.

---

## Deviations from the brief, with reasons

1. **The md-cli F-600 hint text was rewritten** (brief says only "the md-cli
   readback stays" — it does). Two reasons, both measured: the hint offered *a
   timelock* as a remedy and a timelock does not work; and the shape it names
   ("two key-less hash paths") can no longer reach it, so it described a cause
   that cannot occur there. The read-back itself is untouched and still runs.
2. **`cli_compose_encode_gate.rs`'s F-600 test was retargeted** rather than left
   asserting the old message. It was the only test the new refusal made stale;
   the contract it pins is unchanged and its doc now records both layers and a
   mutation for each.
3. **`composition_touches_only_the_pubkeys_tlv` was renamed and given a premise
   assertion** — see above.
4. **The clippy command run is the pinned-toolchain, CI-shaped one**
   (`--workspace --all-targets --all-features` under 1.85.0) rather than the
   brief's bare `cargo clippy --all-targets`, because a bare invocation on this
   box silently resolves clippy 0.1.98. Both were run; the pre-existing 0.1.98
   findings are recorded above.
5. **A fourth refusal case (`two_keyless_split_by_a_keyed_path`) is in the
   vector beyond the brief's two**, because without it a port can satisfy the
   corpus with an "adjacent pair" rule and still cut the malleable policy the
   report found in a different ordering.

## What I could not do, and residue for whoever follows

- **The F-600 read-back now has no known reachable input.** I probed for one —
  mixed absolute lock kinds across paths, 32-slot `wsh` and `tr`, eight
  single-key paths, nine-key paths, unsorted variants — and every candidate
  composed cleanly. It is kept as defence in depth (the brief says it stays,
  and it guards resource limits / repeated keys / timelock mixing should the
  lowering ever change), but it is now **untested by any CLI input**, and I did
  not manufacture one. Worth a follow-up entry if the project wants every gate
  to have a live test.
- **The Go port is untouched**, as scoped. Part B needs: the `validate()` rule
  in `md/compose.go`, the vector vendored (`scripts/vendor-compose-vectors.sh`
  picks it up by glob) and `"compose_refusal_keyless_cap"` added to
  `md/compose_vectors_pin_test.go`'s `composeVectorNames`. Finding 2 is
  host-only (`md-cli` seating) and has no Go counterpart.
- **Nothing pushed.** Branch `fable-r0-fold` exists only in the worktree at
  `/scratch/code/shibboleth/.tmp/fold-rust`; `master` is untouched.
- **The `mk` binary used to generate the new fixture** is
  `/scratch/code/shibboleth/mnemonic-key/target/debug/mk` (`mk 0.13.0`), which
  is what `generate.sh` documents and what the existing fixtures were built
  with. The fixture is committed, so no test run needs `mk`.
