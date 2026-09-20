# Whole-diff review — fold A (descriptor-mnemonic, the Rust PRIMARY), composer fable review r0

**Verdict: 0 Critical / 1 Important / 3 Minor / 3 Nit.** No counterexample
exists for either change: I could not construct a path list with at most one
key-less path that the new `validate()` refuses, nor one with two that it
admits, nor a seating where a present fingerprint is overwritten, an absent one
is filled with the wrong value, or the completed card's wallet id differs from
the fully-seated mint of the same keys. Five mutations, each caught by the
intended test. The one Important is a **missing migration case in the
CHANGELOG**, with a measured reproduction — documentation-only, no code change
required.

Independent reviewer (opus). I did not write the diff, the brief or the
implementation report. Repo `/scratch/code/shibboleth/descriptor-mnemonic`,
range `922778ad..4de55155` (`3d22b714` the key-less cap, `4de55155` the
fingerprint fill). Read-only; nothing committed; no sub-agents; no `.jsonl`
read. Worktrees `/scratch/code/shibboleth/.tmp/review-rust` (tip) and
`…/review-rust-base` (base `922778ad`), own `CARGO_TARGET_DIR`s under
`/scratch`, `TMPDIR=/scratch/code/shibboleth/.tmp`; nothing built under `/tmp`.
Every mutation reverted (`git status --porcelain` empty in both worktrees before
removal); both worktrees removed and pruned; target dirs deleted.

---

## The ONE QUESTION — answered

### 1. Finding 1: can a path list get a WRONG admit/refuse verdict?

**No, over 21,060 lists.** I re-measured independently of the controller's 22
and the implementer's 859, with a wider atom set — six keyed atoms
(`1of1`, `2of3`, `3of5,unsorted`, `1of2,older=10`, `2of2,after=700000`,
`1of1,sha256=…`) and six key-less atoms spanning **all four hash kinds**
(`sha256`, `hash256`, `hash160`, `ripemd160`) and **four lock kinds**
(`older` blocks, `older` 512-s units, `after` height, none) — over every ordered
`wsh` list of 2, 3 and 4 paths containing at least one key-less path, driven
through `md compose --experimental` on **both** binaries:

```
cases 21060  divergences 0
tallies (>=2 keyless, old_ok, new_ok):
  (False, True,  True )  5904     <= 1 key-less: admitted by BOTH
  (True,  False, False) 15156     >= 2 key-less: refused by BOTH
```

Zero admit/refuse divergences. The code implements exactly the measured rule and
nothing narrower or wider: `keyless` collects a path only when `p.keys.is_none()
&& p.hash.is_some() && wrapper != Tr`, and the refusal fires on
`keyless.len() >= 2` at `crates/md-codec/src/compose/mod.rs:549-551`.

**The mint-time rule does NOT reach decode** — the invariant the base commit
(`922778ad`, "mint-time policy must not reach decode") exists to protect. A
*real* device-minted 2-key-less card exists: the C-1 harness evidence in
`/scratch/code/shibboleth/.tmp/fable-funds-work/cases.json`, case
`wsh-two-keyless`, four md1 chunks. On md-codec 0.45.0 / md-cli 0.17.0 it still
decodes, renders and derives byte-identically to 0.44.2 / 0.16.2:

```
md decode     -> wsh(or_i(pkh(@0/<0;1>/*),or_i(sha256(cc6a…123d),and_v(v:sha256(5a23…4a3d),older(5)))))   (identical)
md descriptor -> …#qxt582nd                                                                               (identical)
md address    -> bc1qvh6mravpql6n08d4gmc7256wqedlknza8kvsu0pxkqasc467mdzqd7pjs0                           (identical)
md inspect --json / md decompose <that descriptor>                                                        (identical)
```

Structurally: `compose::validate` has exactly three callers —
`compose/mod.rs:599`, `compose/mod.rs:610` and `compose/presets.rs:30` — none of
which is on any decode, `chunk::reassemble`, `encode_payload_for_identity` or
`decompose` path. None of the six presets can produce a key-less path at all
(every preset's every path carries `keys`), so no shipped archetype is refused.

### 2. Finding 2: can a seating get a WRONG fingerprint?

**No.** Five mutations, each caught by exactly the test that claims it:

| # | mutation applied to the tip | result |
| --- | --- | --- |
| M2 | drop the "DECLARED is never overwritten" guard (`fingerprints.retain(…)` instead of `continue`) | **FAIL** `seating_never_overwrites_a_fingerprint_the_policy_already_declares` (1 of 11) |
| M-wrong | fill an ABSENT slot with `0xdeadbeef` instead of the card's fp | **FAIL** 4 unit rows + 3 of 4 `i2_partial_template_completion` rows, incl. the id equality |
| M5 | invent `00000000` when the card states no master | **FAIL** `a_fingerprint_free_card_leaves_an_absent_declaration_absent` |
| M3 | narrow the cap to ADJACENT key-less pairs | **FAIL** `the_two_keyless_paths_need_not_be_adjacent` **and** `every_case_in_the_conformance_vector_behaves_as_recorded` |
| M4 | move the cap ahead of `NoKeyedPath` | **FAIL** `a_policy_with_no_keyed_path_still_reports_that` |

Baseline before mutating: 18/18 across `compose_keyless_cap`,
`i2_partial_template_completion`, `cli_compose_encode_gate`, `vector_corpus`,
plus 11/11 `seat::compose` unit rows.

**The fill is permutation-consistent, so it cannot change an A3 verdict.**
`comparison_form` (`seat/compose.rs:160-176`) serialises through
`canonical_payload_bytes`, which includes the `Fingerprints` TLV — nothing is
blanked there. It stays sound because md-codec's canonicalisation permutes every
per-`@N` TLV map atomically with the tree, so a filled fingerprint travels with
its key. The live proof is `V_BOUND_SEAT`, which is exactly the adversarial
shape (two interchangeable `sortedmulti` slots, **fingerprint-free**
declarations, **two different masters** `73c5da0a` / `b8688df1`):
`comparison_form_absorbs_a_swap_inside_one_sorted_group` still passes and
`v-bound-seat`'s seating still succeeds, while its id moved. No fixture changed
its exit code, and `v-csid-warn` still refuses as ambiguous.

**Route A == route B generalises beyond C13.** The brief's test (b) pins the id
equality for one fixture. I checked the property across the whole corpus:
host-complete each fixture (`--emit md1`), `md decompose` the resulting
descriptor, re-mint fully seated from the decomposed template + keys +
fingerprints, and compare `wallet_policy_id`. **10 of 10** round-trippable
fixtures agree, including all five whose ids moved:

```
EQUAL: 10 ['v-ap-row1-e2e','v-b1-cross','v-b1-shape','v-b1-wallet','v-b1-warn',
           'v-bound-seat','v-ce1','v-legacy-p2sh','v-mix','v-partial-c13']
NOT EQUAL: []
```

That is the check that would catch a fill putting the *right* fingerprint in the
*wrong* slot — the C13 test alone cannot see it.

### 3. Blast radius — independently reproduced

Driving all 36 seating fixtures on both binaries (25 carry both `md1` and `mk1`
lines and drive; 11 are refusal/partial fixtures my harness skipped):

- **5 of 25 move their composed wallet id**, the same five and the same values
  the implementer reports: `v-ap-row1-e2e` 52bcbf43→f0e43ed2, `v-bound-seat`
  52bcbf43→f0e43ed2, `v-ce1` f35be217→2537939a, `v-mix` 01dcbc20→89a3408b,
  `v-partial-c13` ed6e9919→af5424ab.
- **All five keep byte-identical addresses** (sha256 of `md address --count 3`
  output identical old/new for each).
- **0 of 25 changed their exit code**, on `md descriptor` or `--emit md1`.
- **No stale id literal**: `52bcbf43|f35be217|01dcbc20|ed6e9919` and the four
  moved BIP-380 checksums (`9x35tt0c`, `7axfw7ag`, `7wu5u59q`, `68yxl5as`) appear
  nowhere in `descriptor-mnemonic/crates`, `descriptor-mnemonic/design`,
  `mnemonic-engrave/design` or the `seedhammer` fork — only inside the
  implementer's own report.

**Consumers, each checked:**

| consumer | verdict |
| --- | --- |
| **policy SHAPE id** | **unchanged** — `aad0e0e0`, `a235ee75`, `b02b4403` identical old/new. SHAPE-confirmed stubs keep working. |
| **`--verify-against`** | an already-engraved 0.16.2 completed card verifies **SPEND-EQUAL, exit 0**, against the 0.17.0 seating of the same inputs (origin metadata is excluded from the comparison by design). |
| **already-engraved completed card** | still decodes, inspects and derives the same address on 0.17.0. |
| **`md vectors` corpus** | unchanged — `vectors_output_matches_committed_corpus` passes with only the new refusal file excluded. |
| **the fork** | Go's `md/compose.go:400 Bind(pubkeys, fingerprints)` takes the fingerprints its caller already knows and writes one per slot, so the device's fully-seated mint already produces what the host now produces. The change moves the host TOWARD the device. No Go counterpart to fill; the implementer's "host-only" scoping holds. |
| **WALLET-confirmed stubs** | **regresses — see I-1.** |

### 4. Does the CHANGELOG say what an operator with an already-completed 0.16.x card should expect?

**Partly.** It says the id changes and that addresses do not. It does not say
what happens to the artifacts the operator already holds. See I-1.

---

## Findings

### I-1 (Important) — mk1 key cards that were WALLET-CONFIRMED at 0.16.2 now raise three "different wallet — verify before trusting" warnings, and the CHANGELOG does not mention it

**Where:** `crates/md-cli/src/seat/disposition.rs:78-80` compares each card's
`policy_id_stubs` against the **composed wallet id** first and the shape id
second. The fill moves the composed wallet id for exactly the partial-template
case this fold targets. `CHANGELOG.md`, the md-cli `[0.17.0]` entry, documents
the id move but not this consequence.

**Input (constructed, reproducible).** The `v-partial-c13` policy card, with its
three key cards re-minted at the id an operator would legitimately have stubbed
them with at 0.16.2 — the composed wallet id `ed6e9919` that 0.16.2 itself
printed for that seating:

```sh
mk encode --xpub <KEY 1>  --origin-fingerprint 73c5da0a --origin-path "m/48'/0'/0'/2'" --policy-id-stub ed6e9919
mk encode --xpub <KEY 5>  --origin-fingerprint b8688df1 --origin-path "m/48'/0'/0'/2'" --policy-id-stub ed6e9919
mk encode --xpub <KEY 10> --origin-fingerprint 28645006 --origin-path "m/48'/0'/1'/2'" --policy-id-stub ed6e9919
md descriptor <policy md1> --from-mk1-file <those three cards>
```

**Observed, md-cli 0.16.2:**

```
note: composed wallet id ed6e9919 · policy shape id b02b4403
note: 3 card(s) WALLET-CONFIRMED — stub matches this exact composed wallet: 5f8aa, b3fdd, d0295
```

**Observed, md-cli 0.17.0 (this diff), same inputs, same address
`bc1qr2talz63an0e8xxny3kdskpsax0h0ern76gys65xc8hv4fk926tq2h69mw`:**

```
note: composed wallet id af5424ab · policy shape id b02b4403
warning: card 5f8aa's stub matches neither this policy's shape id nor the composed wallet id — minted under different origin metadata (legitimate), or a different wallet; verify address 0 before trusting.
warning: card b3fdd's stub matches neither this policy's shape id nor the composed wallet id — …
warning: card d0295's stub matches neither this policy's shape id nor the composed wallet id — …
```

**Expected:** the CHANGELOG's migration note tells the operator this will happen
and what it means, so three warnings on their own correct backup are an expected
consequence of a documented change rather than an unexplained alarm on a
funds-safety affordance. No code change is needed and none is recommended —
re-deriving the 0.16.x id would be a second implementation of a rule this fold
retired.

**Why Important and not Minor.** The direction is fail-safe (a good card set
loses a confirmation; a bad one never gains one), and the warning's own first
clause — *"minted under different origin metadata (legitimate)"* — happens to
name the true cause. But `disposition.rs`'s module doc frames stub confirmation
as the defence against an adversarial minter, and the CHANGELOG entry that
exists specifically to say *"the id changes, here is what that costs you"* is
silent on the one artifact class that pays. That is a **missing case** in a
behaviour-change note, and it is the exact question the brief asked. Cheap fix:
one paragraph.

**Suggested content (all three clauses measured above, not reasoned):**
already-engraved completed cards still decode and still report **SPEND-EQUAL**
under `md descriptor --verify-against` (exit 0); the **policy shape id does
not move**, so SHAPE-confirmed stubs are unaffected; only mk1 cards stubbed with
the *composed wallet id* of a host-completed partial template need re-minting,
and until they are the seating prints the "stub matches neither" warning.

### M-1 (Minor) — the new refusal pre-empts `LegacyWrapperShape` and `TooManySlots`, and then prescribes a remedy that does not work for those inputs

The cap is checked at `compose/mod.rs:549-551`, which is **after** `NoKeyedPath`
(`:538`, pinned by a test) and `KeylessUnderTr` (in-loop, pinned by a test) but
**before** `TooManySlots` (`:553`) and `LegacyWrapperShape` (`:565`), neither of
which has a precedence test. Measured:

```
$ md compose --experimental --wrapper sh --path 2of3 --path keyless,sha256=H1 --path keyless,sha256=H2
0.16.2: md: legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr
0.17.0: md: paths 2 and 3 both have no key; … Give one of them a key, a timelock does not help, or fold them into one path

$ md compose --experimental --wrapper wsh --path 9of9 --path 9of9 --path 9of9 --path 9of9 --path keyless,sha256=H1 --path keyless,sha256=H2
0.16.2: md: this wallet would have 36 key slots; the wire holds at most 32
0.17.0: md: paths 5 and 6 both have no key; … fold them into one path
```

Both still refuse, so this is not a wrong answer. But an operator who follows
the new remedy hits a *second* refusal: folding the two key-less paths into one
leaves `sh` with two paths (still `LegacyWrapperShape`) and leaves the slot case
at 36 slots (still `TooManySlots`). That is precisely the defect the implementer
fixed in the F-600 hint and stated as a principle in
`cli_compose_encode_gate.rs:143-146` — *"A remedy that does not work is worse
than no remedy"* — reintroduced by their own change for two other inputs. Fix:
move the cap below the `TooManySlots` / `LegacyWrapperShape` checks, or gate it
on `!list.wrapper.is_legacy()`, and add the two precedence rows beside the two
that exist.

### M-2 (Minor) — the documented MUTATION in `cli_compose_encode_gate.rs` is wrong about which assertions fail

`crates/md-cli/tests/cli_compose_encode_gate.rs:99-102` says: *"delete the
`TooManyKeylessPaths` arm in `compose::validate` -> the last two assertions fail
(the message becomes the parser's 'Miniscript is malleable' quoted by the
read-back)"*. I ran that mutation. The test **does** fail — but at `:133`, the
`err.contains("paths 2 and 3")` assertion, which is the **second** of four:

```
panicked at crates/md-cli/tests/cli_compose_encode_gate.rs:133:5:
the refusal does not name the offending paths:
  template parse error: … Miniscript is malleable …
```

The assertion at `:139` (`err.contains("malleable")`) **survives** the mutation,
because the read-back quotes a parser message that contains the word. So the
failing pair is assertions 2 and 4, not "the last two". The gate is real; only
its own coverage claim is inaccurate — and a mutation note that credits an
assertion with coverage it does not have is the class this project records under
*"assert the mutation APPLIED"*. One-line doc fix.

### M-3 (Minor) — the fill grows the emitted card, so a seating at the 64-chunk wire cap can newly refuse, and nothing says so

Measured chunk counts for a keyed card at `--group-size 0`, without vs with a
declared fingerprint per slot:

```
n=3   6 -> 6   (+0)
n=6  11 -> 12  (+1)
n=9  17 -> 18  (+1)
n=11 20 -> 21  (+1)
```

`compose/mod.rs:33-46` already records that 10 of 1000 generated in-limits
policies needed 65-67 chunks and were refused at `md encode`, so the 63-64 band
is populated. A partial template sitting there at 0.16.2 can now cross into
`Error::TooManyChunks` and refuse where it used to emit. This is fail-safe and
inherent to the fix — the bytes are required for correctness — but it is not in
the CHANGELOG and the implementer's 34-fixture blast-radius sweep cannot see it
(no fixture is near the cap). Worth one CHANGELOG clause or a follow-up.
**I could not construct the exact crossing input**: it needs ~30 distinct xpubs
and `fixtures/pathological/keys.txt` holds 11.

### N-1 (Nit) — the vector's `precedence` block is prose no case drives

`crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json:40-43` asserts
two precedence rules, and the four `cases` exercise neither. The Rust side pins
both (`a_keyless_path_under_tr_still_reports_the_tr_rule`,
`a_policy_with_no_keyed_path_still_reports_that`) — but the Go port vendors the
**JSON**, not the Rust tests, so a port can satisfy every case and still order
`TooManyKeylessPaths` ahead of `KeylessUnderTr`. Both orderings refuse, so no
wrong answer; two more cases would close it, exactly as the fourth (non-adjacent)
case closes the adjacency gap.

### N-2 (Nit) — `refused_when` is looser than the code

`…keyless_cap.json:34` reads *"two or more paths have `keys == null`, at any
positions, whatever locks or hashes they carry"*. `validate()` classifies a path
as key-less only when `keys.is_none()` **and** `hash.is_some()`; a
`keys == null, hash == null` path is `LockOnlyPath`, refused inside the loop
before the cap is reached. A port implementing `refused_when` literally reports
`TooManyKeylessPaths` for `[keyed, lock-only, lock-only]` where Rust reports
`LockOnlyPath`. Both refuse.

### N-3 (Nit) — a premise assertion that an OR weakens

`crates/md-cli/tests/i2_partial_template_completion.rs`,
`the_fixture_header_still_records_this_mint` checks each of the first two
fingerprints with
`contains("--fingerprint @0={fp}") || contains("--fingerprint @1={fp}")`. The
disjunction would pass if both were declared at `@0`. Harmless against the
committed fixture; pairing each fingerprint with its own slot index would make
the premise say what it means.

---

## The five declared deviations — each judged

1. **The md-cli F-600 hint text was rewritten.** **ACCEPTED.** The old hint
   offered a timelock, and `[keyed, K, K+older(5)]` is refused (measured here in
   the 21,060-case sweep and pinned by
   `a_timelock_on_the_second_keyless_path_does_not_rescue_it`). The read-back
   itself is untouched and still runs. I independently checked the implementer's
   residue claim that it now has no known reachable input: mixed absolute lock
   kinds (`after=700000` + `after=1700000000t`), mixed relative units
   (`older=10` + `older=10u`), 32 slots as 4×9-key paths, and 8×4-key paths all
   compose cleanly at 0.17.0. The residue is correctly reported, not concealed.
2. **`cli_compose_encode_gate.rs`'s F-600 test was retargeted.** **ACCEPTED** as
   a retarget — the contract it pins (non-zero exit, empty stdout, names the
   paths, the rule and a working remedy) is unchanged and stronger, and the test
   does fail under the mutation. Its mutation *description* is a finding: M-2.
3. **`composition_touches_only_the_pubkeys_tlv` renamed and given a premise
   assertion.** **ACCEPTED.** Verified: the row now asserts
   `declared.len() == policy.n` before asserting the TLV comes through
   untouched, so it can no longer pass vacuously once a fill exists.
4. **The clippy command run is the pinned, CI-shaped one.** **ACCEPTED**, and
   re-run independently with
   `$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` first on PATH:
   `clippy 0.1.85 (4d91de4e48 2025-02-17)`,
   `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
   → **clean**; `cargo fmt --all --check` → **clean**. The four 0.1.98-only
   lints they describe are in `md-codec/tests/parity_smoke.rs` and
   `md-cli/src/seat/mod.rs`, neither of which this diff touches.
5. **A fourth refusal case (`two_keyless_split_by_a_keyed_path`) beyond the
   brief's two.** **ACCEPTED**, and it earns its place: under the M3 adjacency
   mutation `every_case_in_the_conformance_vector_behaves_as_recorded` FAILS.
   Without that case the vector would have been satisfiable by an "adjacent
   pair" port.

---

## What I ran, with its numbers

- **Built both ends** of the range (`4de55155` and `922778ad`) in separate
  worktrees with separate `CARGO_TARGET_DIR`s under `/scratch`.
- **21,060-case compose sweep**, old vs new binary, 2-4 path `wsh` lists over 12
  atoms (4 hash kinds, 4 lock kinds): **0 divergences**; 5,904 admitted by both,
  15,156 refused by both. Plus hand-driven `tr`, `sh`, `sh-wsh` and 3-key-less
  shapes: no divergence.
- **Targeted suite at the tip**: 18/18 across `compose_keyless_cap`,
  `i2_partial_template_completion`, `cli_compose_encode_gate`, `vector_corpus`;
  11/11 `seat::compose` unit rows. (I did not re-run the full 1354 — the
  controller is doing that in parallel, per the brief.)
- **Six mutations**, each applied to the tip and reverted: M2, M-wrong, M5, M3,
  M4 (all caught, table above), plus **M6** — dropping the `if filled` guard at
  `seat/compose.rs:130` so `tlv.fingerprints` is always `Some(...)`. M6 is
  **semantically inert** on this corpus: 11/11 and 4/4 still pass and the same
  five fixtures (no more, no fewer) change their emitted card. The guard is
  correct and defensive but nothing currently distinguishes it; noted, not filed.
- **36-fixture blast radius** on both binaries (25 drivable): 5 ids move,
  addresses byte-identical, 0 exit-code changes. Values and names match the
  implementer's report exactly.
- **Route-A == route-B id equality** across the corpus: 10/10.
- **Decode-safety**: a real device-minted 2-key-less md1 card from the C-1
  harness evidence decodes, renders, derives and decomposes identically on the
  tip.
- **Consumer probes**: `--verify-against` (SPEND-EQUAL, exit 0), shape-id
  stability, stub disposition (I-1), `md inspect`, `md address`, `md decompose`.
- **Gate**: pinned `clippy 0.1.85` clean, `cargo fmt --all --check` clean.
- **Chunk-growth measurement** for M-3 at n = 3, 6, 9, 11.

## What I could not verify

- **The `00000000` verbatim-carry claim** at `seat/compose.rs:35-37` — *"`00000000`
  is md1's absent-master sentinel … transcribing it is what makes the host's id
  agree with the device's, which writes the same bytes."* I confirmed the Rust
  half is self-consistent: `mk encode --origin-fingerprint 00000000` mints, and
  `md encode --fingerprint @i=00000000` accepts and encodes it, and
  `validate_origin_key_consistency` exempts it (`validate.rs:441-460`). I did not
  execute the **device** side, so "agrees with what the device writes" rests on
  `md/compose.go:415-433`'s `Bind` merge semantics read, not run.
- **The exact input that crosses the 64-chunk cap** (M-3): needs ~30 distinct
  xpubs; the fixture corpus holds 11.
- **The full 1354-test suite, fmt and clippy as a single gate run** — deliberately
  not re-run, per the brief; I ran the pinned clippy and fmt (both clean) and the
  targeted binaries.
- **Part B (the Go port)** is out of this range and untouched, as scoped. For the
  hand-off: the vector is
  `crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json`, and
  `md/compose_vectors_pin_test.go`'s `composeVectorNames` needs
  `"compose_refusal_keyless_cap"` added or the pin fails — I confirmed that list
  is hand-maintained in the fork.
