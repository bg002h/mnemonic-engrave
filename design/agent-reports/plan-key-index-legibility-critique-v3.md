# Adversarial critique v3 — PLAN_key_index_legibility.md @ 5b786d4

Reviewer: independent read-only agent, 2026-08-19. Scope per brief: audit the §8
ledger row-by-row, re-verify v3's corrections by experiment, and test the plan's
own claim that "every claim below is pasted command output." Everything marked
**executed** was run on this machine against the release binaries
(`mnemonic-toolkit/target/release/mnemonic`, `descriptor-mnemonic/…/md`,
`mnemonic-key/…/mk`, `mnemonic-engrave/…/me`, `…/me-preview`) and a fixture
rebuilt exactly as `transcript_pathological.sh` builds it (3 md1 chunks, 11
cards → 30 mk1 chunks, stub `5b48af35`). Both journey scripts were **run in
full** in an isolated scratch copy — the first time any round has executed §2's
acceptance vehicle.

## Verdict

**v3 is the first version whose middle is genuinely executable — §2 (one
sentence short), §3 (one value wrong), §4 (clean) — but its headline §0 block
is, for the third consecutive round, not a real command output, and the §8
ledger contains manufactured dispositions.** The weakest item is **§0/§1's
verify-bundle transcript**: the invocation it shows (`--md1 ×3 … --mk1 ×2`)
**exits 4 with `mismatch`** — executed — because supplying `--mk1` triggers a
stub-bind check the journey's `mk encode`-built cards can never pass; the output
block v3 prints (✓ line + warning, no ✗ row, no verdict) corresponds to no run
of that command. The real success form (md1 chunks **positional**, **no**
`--mk1`) exits 0 in 16–18 ms — never 4 ms. An implementer scripting §1 from §0
ships a journey step that prints `mismatch`.

The ledger is ~83% sound: 30 of 36 rows are supported by the body. Five are not
(D6 cites text §2 does not contain; D7 claims a command the plan nowhere gives;
D9 claims a mechanism the code contradicts; F6 claims a narrowing that did not
happen; H4 cites the wrong section), one is half-supported (D5 — placement yes,
scope written in exactly the per-plate form r0-v2 warned against). The header
claim "every prior finding" is false: at least the round-1 stub-cross-check,
fill-in/hand-marking carriers, the tr() variant, the producer-side follow-up,
and v2's N1/N2 have no row and no disposition anywhere in v3.

Genuinely sound, verified by experiment: the corrected root cause (`d09840c3…`
reproduced exactly; preimage confirmed 65-byte `[chain_code‖pubkey]`), the
16-hex floor and its error text, the 24/31 grep counts, `0xe03a5` as a real
chunk-set id, the §2 assertion string (the golden card decodes to
`aabbccdd/48'/0'/0'/2'`), the `--accept-search-time 4000` **form** (no suffix →
seconds), and — executed here for the first time — **both committed transcripts
regenerate byte-identically** (path prefix aside), so §2's acceptance is
satisfiable today.

## Ledger audit

Labels: D/F/MA/H = architect-r0 @ 2d9fe3e; v2-\* = critique-v2 @ 53b2e82.

| finding | v3's claim | supported? | evidence |
| --- | --- | --- | --- |
| D1 type | §2 `String` + rationale | **YES** | §2 "Decided: representation"; me-cli deps verified (no `bitcoin`, `Cargo.toml:22-27`) |
| D2 manifest fields | `card_fingerprint`/`card_path`; golden + SPEC | **YES** | §2 "Decided: JSON" + "golden and the spec BOTH change" |
| D3 `None` fallback | exact string | **YES** | §2 table row 2 |
| D4 empty path | exact string | **YES** | §2 table row 3 |
| D5 collision | "placement specified" | **HALF** | placement yes ("after the chunk counter"); **scope is written per-plate** — "plates whose (fingerprint, path) pair is not unique within the bundle" marks **30/30** fixture plates (every multi-chunk card's plates share the pair with their own siblings). r0-v2 D5 warned this exact reading; the needed "group by chunk set first" sentence is absent. See Hole H-4 |
| D6 multi-wallet / multi-stub | "§2: irrelevant to this label; ignored deliberately" | **NO** | §2 contains no such statement — zero mentions of multi-wallet, multi-stub, or "irrelevant". The disposition exists only in the ledger row. Worse than silence: it manufactures a decision |
| D7 restore invocation | "§3 + §0 transcript give the full command" | **NO** | §0's only transcript is **verify-bundle's** (and it is the failing form). No restore invocation appears anywhere in v3: `--account 0`, the bare (non-`@N=`) cosigner form search mode requires, and `--network` are stated nowhere. r0 D7's items are still unwritten |
| D8 which target | §3 `--expect-wallet-id` | **YES** | §3 change 2; address form refused (F2, reproduced by prior rounds) |
| D9 which number decides | "moot — `--accept-search-time` removes the estimate gate" | **NO** | executed + source: `cap_decision` (`permutation_search.rs:388-415`) still gates on the **estimate**; the accept value replaces the 3600 s ceiling with 4000 s (+11%). My run printed `est. ≤ 909.0652032s` — under 3600, so the 4000 was never even consulted. The gate is not removed; it is nudged. See Hole H-3 |
| D10 doc location | §4 file + heading | **YES** | §4 deliverable bullet |
| D11 canonicalisation | encoded-template index; measured identity | **YES** | §4 D11 bullet (identity verified by r0) |
| D12 release mechanics | §2 bump + CHANGELOG | **YES** | §2 "Decided: release mechanics"; SPEC §11 lockstep verified at `SPEC_me_bundle_phaseA.md:121-123` |
| F1 seed required | §0 | **YES** | §0 + `restore.rs:1396` (verified by r0-v2) |
| F2 `--search-address` refused | §0 | **YES** | reproduced by two prior rounds; "84 ms" is the fast end of a measured 82–208 ms range (v2-N1, unaddressed) |
| F3 wrong target id | §1 rewritten around verify-bundle | **YES (direction)** | but the §0 block §1 builds on is itself wrong — Hole H-1 |
| F4 dependency conditional | `String` makes it unconditional | **YES** | §2; Cargo.lock has no `bitcoin` under me-cli |
| F5 manifest contract | golden + SPEC named | **YES** | §2 names `bundle-md1-mk1.json`, `cli.rs:308`/`:745`, SPEC §6 — all verified real (`assert_eq!(v, expected)` at both lines) |
| F6 `@N` reason over-broad | "claim narrowed" | **NO** | §2's sentence — "a keyless template carries no key order" — is the **same general claim** r0 F6 refuted (a keyless md1 CAN carry per-`@N` Divergent origin paths and a fingerprints TLV: `origin_path.rs:91-96`, `tlv.rs:27-28`) and r0-v2 F6 called "RESTATED… now actively harmful". Nothing was narrowed; the sentence got shorter. "Survived three rounds" misstates two of those rounds |
| F7 fixture correction | §4 limit 2 | **YES** | `@4` = `[b8688df1/48'/0'/0'/2']`, adjudicated in r0 |
| F8 recoverable shape | §0 | **YES** | explicit mode exit 0, re-executed this round |
| MA-1 collision untested | §2 acceptance: a test | **YES** | and the fixture is producible — executed: two different xpubs encode with the same asserted `--origin-fingerprint aabbccdd` + path |
| MA-2 assertion criterion | exact text | **YES** | `mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2` is producible: the golden card MK1_A/B decodes to exactly `aabbccdd` / `48'/0'/0'/2'` / 2 chunks (executed). Note the `manifest.rs:228` unit fixture is hand-built (`string: Some("mk1a")`), so the new fields must be typed into it — fine, but worth a sentence |
| MA-3 golden/schema | both change | **YES** | §2 |
| MA-4 other journey | both transcripts | **YES** | §2 acceptance names both; satisfiability proven this round (below) |
| MA-5 §4 deliverable | deliverable + sentence + acceptance | **YES** | §4 |
| MA-6 exit codes | all four named | **YES** | §3; codes match prior measurements (0/1/2/4) |
| MA-7 prefix floor | 16 hex | **YES** | executed: 8 hex → exit 4 floor error; 16 hex → exit 0 |
| H1 §3 prerequisite | dependency on §1 | **YES** | §3 "Dependency" |
| H2 two transcripts | §2 acceptance | **YES** | as MA-4 |
| H3 double regeneration | §6 once | **YES** | §6 |
| H4 §2 decision not orderable | "resolved — the decision is made in §2" | **NO (mismapped)** | r0's H4 was about the **convention** section's re-derivation question invalidating §3's target — v1's §2 = v3's **§4**. The true resolution is §4 limit 2 ("a decision, not a step"), which r0-v2 H5 already recorded. The row points at the checklist section, where no such decision exists — a transcription error in the artifact meant to prevent transcription errors |
| v2-C1 verify-bundle prints it | §1 rewritten | **YES** | direction correct; block wrong (H-1) |
| v2-C2 root cause disproven | §0: flattening + omitted fps | **YES, executed** | privacy-preserving cards → `d09840c3aa78035368f2dfb4bc271a27`, identical first recv (reproduced exactly); preimage xpub is `Option<[u8; 65]>` "(32 chain-code ‖ 33 compressed pubkey)" in `canonicalize.rs`; `mk decode` prints `origin_fingerprint:  (omitted, privacy-preserving mode)` verbatim |
| v2-I2 estimate variance | `--accept-search-time` | **HALF** | addressed in kind; the chosen value 4000 defeats the stated purpose (H-3). critique-v2's recommendation was "generous (say, 2h)" |
| v2-I4 empty-path card | §2 rendering table | **YES** | "empty path" + "both absent" rows present |
| PDF captions `@N` | document vs checklist | **YES** | `build_pdf_pathological.py:301` renders `@{ki} [{fp}/{path}]` (verified); the checklist does not |

**Rows that do not exist** (the header says "every prior finding, resolved or
explicitly rejected"): round-1 C2 (F-210 / acceptance-vehicle regeneration — a
round-1 **Critical**, no row, and "F-210" appears nowhere in v3), round-1 I1
(sortedmulti — dissolved by deletion, unrecorded), I2 (lint — resolved in §4,
unrecorded), I3 (coupling — resolved in §5 "Acknowledged tension", unrecorded),
M4 + miss #4 (stub cross-check in `me bundle` — **no disposition anywhere**),
N2 (PDF static-text rot — still live, see below), misses #2/#3/#5/#6/#7
(`--labels`/card-index generalization, fill-in `@__` ceremony table, keyed-md1
exact-`@N` branch, **tr() variant**, hand-marking line — none dispositioned);
v2-I1 (dissolved, unrecorded), v2-M1 (resolved in §5, unrecorded), v2-N1
(false precision — no row **and recommitted**: "84 ms", "4 ms"), v2-N2
(checkout-vs-registry citation — no row and repeated: §0 cites
`mnemonic-key/crates/mk-codec/src/key_card.rs:34-57` while the build uses
registry `mk-codec 0.4.1`, Cargo.lock verified); r0-v2's D8 (privacy-preserving
test-fixture provenance — still unowned by any §2 step).

## Holes

### H-1 (Critical) — §0's verify-bundle block is not the output of the command it shows; the shown command fails

Plan §0 presents, as a **measured fact**, this invocation and output:

```
$ mnemonic verify-bundle --network mainnet --from phrase=- \
    --md1 ×3 --cosigner @N=… ×28 --mk1 ×2
✓ wallet_completed: completed WalletPolicyId ced2270948ecb5af…
warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED …
```

Executed, three runs, exact shape (3 `--md1`, 28 `--cosigner @N=`, 2 `--mk1` =
key-00's chunks): **exit 4, stdout `mismatch`, 15–17 ms**, with a check row the
plan's block omits:

```
✗ mk1_template_stub_bind: supplied mk1 stub(s) do NOT bind via the template-id
  stub (card mismatch or policy/template cross-mix)
```

Mechanism (`verify_bundle.rs:1072-1090`): any supplied `--mk1` triggers a bind
check requiring the cards' chunk-set-ids to equal
`{derive_mk1_chunk_set_id_for_slot(stub, i)}` — the per-slot ids `mnemonic
bundle` derives. The journey's cards are `mk encode` output; their csis are not
those values. Executed with `--mk1 ×2` **and** `--mk1 ×30`: both fail. So **no
`--mk1` value drawn from this journey can make the shown command exit 0.**

The **working** form passes the md1 chunks **positionally** and omits `--mk1`
(`--mk1` is `required_unless_present_any = ["bundle_json", "extra_strings"]`,
`verify_bundle.rs:183`). Executed, five runs: exit 0,
`OK (multisig template recomposed)` / `wallet-policy-id: ced2270948ecb5af0779249ac7181f4a`
/ `first recv: bc1qkuknuy…` / `your seed completes cosigner slot @0`, **16–18
ms**. That matches critique-v2's 17 ms. **"4 ms" matches no measurement in any
round** (8 runs here: 15–18 ms).

Why Critical: this is the plan's replacement for two previously-false headline
sections, under a bold "**Every claim below is pasted command output**" — and it
is a mosaic: real output lines (they match the positional form's stdout/stderr)
stitched to a command that produces `mismatch`. §1 step 1 tells the journey to
run "verify-bundle in explicit `--cosigner @N=` form" — an implementer copying
§0's block commits a journey step that exits 4. One-line fix: positional md1,
no `--mk1`, and state that `--mk1` must NOT be supplied (with the bind-check
reason).

### H-2 (Important) — §1 institutionalizes recording an *unverified* id, and the check its warning recommends is a no-op in the mode being taught

Executed, assignment swapped `@1↔@2`, same everything else: **exit 0,
`OK (multisig template recomposed)`, `wallet-policy-id: 78bb22e96c6cfeed56cbca05d0657f31`,
`first recv: bc1qenrhkw4m7h0jmalckt7cuahyckfg5a8lhdefg2y6c0vexacskjms37wg93`** —
a different wallet, cleanly blessed. An operator following §1's habit records
that id; at recovery the search dutifully reconstructs the wrong wallet, whose
addresses hold nothing. This is precisely the warning's text.

Worse — executed: explicit `@N=` mode **silently ignores `--expect-wallet-id`**.
Supplying the *wrong* id (`78bb22e96c6cfeed`) with the *correct* assignment
still exits 0 `OK` with no check row (the flag is threaded into the completion
ctx at `verify_bundle.rs:946` but never enforced on the explicit path). So the
warning's own remedy — "Record + check --expect-wallet-id" — does not function
in the mode §1 demonstrates; it works only in bare-`--cosigner` search mode.

The journey holds the ground truth one step earlier: §9b prints
coordinator-derived addresses from the keyed md1. The swapped run's `first recv`
differs from §9b's — that comparison is the *only* working backup-time check,
critique-v2's recommendation #1 named it, and v3 dropped it: §1's step prints
the warning verbatim but neither performs nor requires the address
cross-check, and no acceptance bullet covers it. Fix: §1's journey step asserts
printed-`first recv` == §9b's first derived address (one `grep`/`test` line),
and the README names the address comparison — not `--expect-wallet-id` — as the
check that works at backup time in explicit mode.

### H-3 (Important) — `--accept-search-time 4000` does not make "the gate the actual work"; it moves the cliff 11%

The form is valid — executed: `parse_search_duration` (`restore.rs:2185-2204`)
takes a bare integer as **seconds** (the help's "humantime duration" is doc
drift), and the run completed exit 0 in 15.4 s realized. But the printed
estimate this run was **909.1 s < 3600 s**, so the 4000 was never consulted:
`cap_decision` (`permutation_search.rs:388-415`) proceeds when estimate ≤
ceiling, and otherwise refuses unless accept ≥ estimate. **The gate remains the
single-threaded estimate**; 4000 merely replaces 3600. Measured estimate spread
for this identical command across rounds: 187.4 → 1284.9 s *on one fast 24-core
machine* — ~7× calibration noise with 3.1× headroom under 4000. On the slower
machine §3 itself worries about, the estimate scales with the machine and
crosses 4000 nearly as readily as 3600. §3's stated rationale ("the gate is the
*actual* work (≈16 s), not a variable estimate") is false as specified, and
ledger row D9 repeats the error. Fix: a genuinely generous value (`2h` per
critique-v2, or `24h` — the flag is an acknowledgment, not a budget), or name
exit 1 as an accepted journey outcome.

### H-4 (Important) — §2's collision rule, read as written, suffixes all 30 plates

§2: the `set <chunk_set_id>` suffix is "appended **only** to plates whose
`(fingerprint, path)` pair is not unique within the bundle." Uniqueness counted
over **plates** — the literal reading — marks every plate of every multi-chunk
card (its siblings share the pair): **30 of 30** in this fixture, where the 11
cards' pairs are in fact all distinct and the intended answer is **zero**
suffixes. r0-v2 D5 warned this exact failure ("a naive per-plate scan reports
every multi-chunk card as colliding with itself; detection must group by
`chunk_set_id` first") — v3 fixed the placement half and wrote the scope half in
the defective form. The §2 acceptance test (two cards sharing a pair) cannot
catch it: it asserts the suffix *appears* where expected, not that it is
*absent* on ordinary multi-chunk cards. Fix: one sentence ("uniqueness is
counted over chunk sets, not plates") plus a no-suffix assertion on the
ordinary golden card.

### H-5 (Minor) — Third-round recurrences the ledger does not even list

(a) Citation drift: `key_card.rs` cited from the `mnemonic-key` checkout; the
build uses registry `mk-codec 0.4.1` (Cargo.lock) — v2-N2, verbatim recurrence.
(b) False precision: "84 ms", "4 ms", "16 s" as point values after two rounds
flagged the pattern (v2-N1, r0-v2 FI-4) — and "4 ms" is not merely imprecise
but wrong (H-1). (c) `me bundle` cannot verify an asserted fingerprint —
executed: `mk encode` happily embeds `aabbccdd` over any xpub — so §2's label
prints an *asserted* origin as if it were a fact; one README sentence ("the
label repeats the card's claim, it does not verify it") would keep §2's honesty
claim true. (d) §1's "there is **no** no-target recovery path" — the
`--accept-search-time` hatch makes an address search *possible* (~10 days
single-thread estimate; ~10–20 h parallel); "no practical path" is the
defensible sentence, and this plan has twice been burned by absolutes.

## Claims checked

| v3 claim | Holds? | Evidence (executed unless noted) |
| --- | --- | --- |
| §0 verify-bundle block is pasted output | **NO** | shown invocation → exit 4 / `mismatch` / ✗ bind row, 3 runs; success requires positional md1 + no `--mk1` |
| "in 4 ms" | **NO** | 15–18 ms across 8 runs, two sessions (critique-v2: 17 ms) |
| warning text verbatim | **YES** | byte-identical on stderr |
| completed id `ced2270948ecb5af0779249ac7181f4a`; first recv `bc1qkuknuy…` | **YES** | positional form, exit 0 |
| "recovery from shuffled cards… exit 0 in 16 s" | **YES** | 15.4 s realized, card-contiguous shuffle 07 03 10 01 09 05 02 08 04 06 |
| 16-hex floor block (`ced22709` → floor error; 16 hex → completed) | **YES** | exit 4; error text is a truthful truncation of "restore: multisig-template-floor mismatch — derived --expect-wallet-id prefix too weak for this search: need ≥8 bytes …, got 4" |
| xpub metadata "never hashed"; causes = `--path bip48` flattening + omitted fps | **YES** | preimage field `xpub: Option<[u8; 65]>` "(32 chain-code ‖ 33 compressed pubkey)" in `canonicalize.rs`; critique-v2's byte-exact mutation experiment; this round: privacy cards → id moved, xpubs untouched |
| `d09840c3…` with identical addresses | **YES** | `d09840c3aa78035368f2dfb4bc271a27`, first recv identical |
| `mk decode` renders `(omitted, privacy-preserving mode)` | **YES** | verbatim |
| grep -c 'mk1 chunk': 24 / 31 | **YES** | exactly |
| `Cargo.toml:22-27` serde + serde_json, no bitcoin | **YES** | lines verified |
| golden byte-pinned at `cli.rs:308` and `:745` | **YES** | both `assert_eq!(v, expected…)` |
| `bundle.rs:279` decodes and discards KeyCard | **YES** | value dropped |
| golden's cards have cards → `skip_serializing_if` doesn't spare them | **YES** | MK1_A/B decode: `aabbccdd` / `48'/0'/0'/2'` / stubs `11223344` / 2 chunks |
| §2 assertion string producible | **YES** | above; `:228` unit fixture is hand-built and must gain the fields |
| `set 0xe03a5` a real value | **YES** | present in the fixture manifest (4 hits) |
| `--accept-search-time 4000` exists, parses, bypasses | **FORM YES / PURPOSE NO** | bare number = seconds; but ceiling→4000, gate still the estimate (909 s this run, under both) |
| chunk shuffle exits 1 | **YES (prior round)** | r0-v2 D9: "chunked-header malformed", exit 1 |
| "Nothing in the journey runs this, and no document mentions it" | **YES** | grep: zero hits in design/journeys |
| both transcripts regenerate (§2 acceptance satisfiable) | **YES — first execution** | both scripts run to completion in a scratch copy; transcripts **byte-identical** after path normalization; exit profiles match committed (19×0/1/1/1 and 22×0 + 1×3) |
| SPEC §11 lockstep exists | **YES** | `SPEC_me_bundle_phaseA.md:121-123` |

## What v3 still misses

1. **The correct §1 invocation** — nowhere in the plan (H-1), and the full §3
   restore invocation (`--account 0`, bare-cosigner form, `--network`) is also
   nowhere, despite ledger row D7 claiming otherwise.
2. **The backup-time address cross-check** (H-2) — the one check that works in
   the mode being taught; dropped from critique-v2's recommendation.
3. **The operator's own suggestion, still half-engaged.** §4 covers the
   creation-time convention; the PDF row acknowledges the captions. But the
   fill-in `@__` ceremony table, the hand-marking line, the
   `card-index.txt`/`--labels` generalization, and the `me bundle` stub
   cross-check (round-1 M4) still have no recorded yes/no — two rounds asked
   for "a sentence each."
4. **F-210** — zero mentions, while §2's acceptance leans on transcript
   regeneration. Happily the repair works (executed, byte-identical), but the
   plan asserts an acceptance nobody had run; per the house rule that was a
   hypothesis until this round. One sentence citing the verified regeneration
   (and F-210's still-open PDF residue) closes it.
5. **The tr() variant** — fourth consecutive round with zero mention;
   `wallet-policy-tr.txt` remains unexercised (including by me). Project memory
   explicitly warns one descriptor path mismeasures the other.
6. **The producer follow-up** (per-key origins in `md encode`, Rust-primary,
   with vectors) — the structural fix that would let `md inspect` converge on
   the completed-form id; critique-v2 miss #2 / rec #5; unmentioned.
7. **Round-1 N2 rot is still live**: `build_pdf_pathological.py:314-315` still
   says "The 25 public plates … eleven key cards at two chunks each" against a
   33-public / 30-chunk / 2-3-chunks-per-card reality — §2's regeneration walks
   straight through this file's output.

## Newly introduced problems

1. **The §0 verify-bundle block** — wrong invocation, mosaic output, "4 ms"
   (H-1). Introduced by this rewrite.
2. **`--accept-search-time 4000`** — new decision; value defeats its stated
   purpose (H-3).
3. **The per-plate collision wording** (H-4) — v2 left scope open; v3 closed it
   in the defective form.
4. **Ledger rows D6, D7, D9, F6, H4** — dispositions the body does not support
   (audit table). D6 and D7 manufacture content; the ledger's own header claim
   ("every prior finding") is false.
5. **"Survived three rounds"** (§2, the `@N` reason) — misstates two of the
   three rounds, which called that reason over-broad.

## Open / could not determine

- **Whether a slower machine's estimate actually crosses 4000 s** —
  extrapolated from the measured 7× single-machine spread, not reproduced on
  slow hardware (same caveat as both prior rounds).
- **Whether explicit-mode `--expect-wallet-id` being ignored is intended or an
  upstream toolkit defect** — either way §1 leans on warning text whose remedy
  is inert in that mode; if it is a defect, the fix belongs in
  `mnemonic-toolkit`, and §1's README should not promise the flag works there.
- **The provenance of "4 ms"** — no run in any round's record produces it.
- **The tr() fixture** — still unexercised, all four rounds.
- **Whether `me-preview`-dependent steps behave identically outside this
  machine** — the journey runs executed here used the committed sidecar binary;
  cross-machine reproducibility of the PNG/SVG steps was not probed (byte
  identity of the *transcripts* is what was verified).
