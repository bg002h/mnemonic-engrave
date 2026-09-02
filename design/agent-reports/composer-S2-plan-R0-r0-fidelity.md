# composer S2 plan — R0 round 0, FIDELITY + DESIGN lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S2_fork_codec.md` (mnemonic-engrave, master `bb0c07f`)
**Against:** `design/STAGED_PLAN_wallet_policy_composer.md` §S2; `design/SPEC_wallet_policy_composer.md` §4c, §4f, §5, §7c/§7d/§7e/§7f, §9 items 1/2/8, §12 items 1/6/7/8
**Rust primary read:** descriptor-mnemonic `66bdf2f4`, `crates/md-codec/src/compose/{mod,lowering,tr,presets}.rs`
**Fork read:** `bg002h/seedhammer` `169073c`
**Reviewer:** independent; did not author the plan. Compile/test/citation facts taken as settled per the brief and not re-derived.

Counts: **1 Critical / 3 Important / 5 Minor / 2 Nit.**

---

## Lens 1 — fidelity of `md/compose.go` to `md-codec::compose`

I read `compose/mod.rs` (453 lines), `lowering.rs` (302), `tr.rs` (54) against the plan's `md/compose.go` (plan lines 870-1504) line by line, on the axes the brief named. Result: the port is faithful on every axis the vectors cannot see, except the refusal-message operand base (N-1).

| Rust | Go | verdict |
| --- | --- | --- |
| `validate()` arm ORDER: empty → >8 paths → per-path {threshold, lock-only, keyless-under-tr} → per-path lock → no-keyed-path → >32 slots → legacy shape | identical order, `ValidatePathList` (plan 1152-1195) | ✓ |
| legacy check `sole && sorted`, `sorted` read from `paths.first()` | `list.Paths[0].Keys != nil && …Sorted`, reachable only when non-empty | ✓ equivalent |
| `number()` — `[first] ++ (0..n).filter(≠first)`, `by_path` returned in LISTED order | `numberSlots(list, -1|ik)`, same | ✓ (`first = -1` is unreachable as an index, matching `Option::None`) |
| `path_body` — parts `[KEYS, sha256, LOCK]`, folded right: `and_v(v:part, acc)` from the second-to-last backwards | identical fold | ✓ |
| `wsh_chain` — `or_d` iff the HEAD path `is_bare_multi()` (n ≥ 2, no hash, no lock), else `or_i`; `sorted_legal = sole && is_bare_multi` | identical | ✓ |
| `tr::internal_key_path` — first-listed `is_bare_single`; `spine` right-leaning; `Body::Tr{is_nums: ik.is_none(), key_index: 0}` | identical (`ik < 0`) | ✓ |
| `experimental()` — keyless always; unsorted only where `sole_sorted_legal(i)`; tr closure `m == 1 && Some(i) != ik && is_bare_multi` | identical (`i != ik`) | ✓ |
| `origins()` `taken` semantics — seeded from ALL declared origins (duplicates included), then unseated slots take the lowest free account in ascending slot order, each fill pushed onto `taken` | identical `resolveOrigins` | ✓ — this is the "declared origin equals a default" case the brief flagged; `TestComposeWithFillsTheLowestFreeAccount` pins exactly it |
| pairwise invariant — same origin ⇒ both fingerprints present AND different | identical, and the asymmetric one-card case is one of the three negative rows | ✓ |
| shared-vs-divergent collapse — `windows(2).all(…)`; n = 1 ⇒ Shared | `allSame` loop from i = 1; n = 1 ⇒ Shared | ✓ |
| `TlvSection::new_empty()` (all-`None`) + fingerprints | zero `tlvSection` + `fpPresent`/`fingerprints` | ✓ (`tlv.rs:43-54` confirms no non-default field) |
| `UseSitePath::standard_multipath()` = `[{false,0},{false,1}]`, `wildcard_hardened: false` | identical literal | ✓ (`use_site_path.rs:72-85`) |
| `Lock::operand()` bands and the `0x400000` type flag | identical; Go additionally range-checks `> 0xffff` because Go widens `u16` → `uint32` | ✓ superset, correct |
| `ComposeError` Display operands | **0-based in Go, 1-based in Rust** | ✗ see N-1 |

Not ported and correctly so: `template_with_origins` (the Go side emits no text, by design and by the package invariant), `presets.rs` (see M-5).

---

## Lens 2 — spec fidelity, clause by clause

Every STAGED_PLAN §S2 "Delivers" clause has a task (the plan's own self-review table at lines 2977-2991 is accurate as far as it goes). What follows is only where a clause is NOT fully discharged.

- **§12 item 1, "every CHUNK … byte for byte"** — discharged. All 28 family rows (26 vendored + 2 `no-corpus`) compare tree, path declaration, use-site/n, payload bytes and chunk strings. I independently confirmed the primary carries exactly 126 compose files (26 × {bytes.hex, phrase.txt, descriptor.json, template} + 22 × conformance.json) and 26 MANIFEST `compose_*`/`keyed_compose_*` names, so Task 1's counts are right.
- **§12 item 1, "every address byte for byte"** — NOT discharged, and the deferral is only partly honest. See M-2. The taproot half genuinely cannot be done in `md/` (the port's `address/` derives SortedMulti and Singlesig only, and `md/conformance_keyed_test.go:38-42` says so in its own scope note); that deferral is honest. The *wsh change-chain* half is available now and is skipped for no stated reason.
- **§12 item 6** — the producible half (§4f's invariant "is never produced", including the asymmetric one-card case) IS covered by `TestComposeRefusesWhatThePrimaryRefuses`'s three `ComposeWith` rows. The `seatKeyCards` leg, the address reproduction and the partially-seated named vector are deferred to S3 and named at plan line 2993. Honest.
- **§12 item 7** — fully discharged. I enumerated §4c's four bands against `TestLockCheckIsTheDeviceSideRangeGate`: OlderBlocks in {1, 65535} / out {0, 65536}; OlderUnits in {1, 65535} / out {0, 65536}; AfterHeight in {1, 499 999 999} / out {0, 500 000 000}; AfterTime in {500 000 000, 2 147 483 647} / out {499 999 999, 2 147 483 648}; plus an unknown-kind row. Every boundary value in and out, per kind. Complete.
- **§12 item 8** — the classification half is discharged; the device-visible-count half is named as S3 at line 2993. Honest. But the fixture it depends on is not where the plan says it is: **I-2**.
- **§9 item 1's second clause** — "carry the lock operand and the digest on `Branch` so §7e can render them" is claimed but not delivered: **C-1** (kind) and **I-1** (experimental marks).
- **§9 item 2, §9 item 8** — discharged (Task 3 in both contexts with an opcode mutation; Task 2 `ScriptType`/`DefaultOrigin` plus Task 7's tie test).

---

## Lens 3 — is the API something Stage 3 can build on?

Walked §7d's seating end to end against the produced surface: `Compose(list)` → `Slots()` for the "Slot @2, Path 1 key 2 of 3" prompts → per-source `SlotOrigin` → `ComposeWith` → `Chunks()` (keyless template) → `Bind` → `Chunks()` (keyed policy) → `ComposerStubs` → `mk.AppendStubs`. That path closes. Specifically checked and found **fine**:

- A seed filling several slots at hardened accounts by ordinal (§4f) needs only `DefaultOrigin(w, ordinal)`, which is exported.
- `Bind`'s `map[uint8][65]byte` chaincode‖pubkey form is **not** a deviation: `md.ExpandedKey.Xpub [65]byte` is the established exported convention and `gui/multisig_build_slots.go:407-419` (`bothSlotKey`) already builds it. No finding.
- `bip32.Path` → `[]md.PathComponent` for a `key:` record or an mk1 card is already solved by `gui/singlesig_derive.go:137-149` (`originComponents`). No finding.
- Partial seating (§7f's partially-seated form) does **not** need a partial `Bind`: the artifact is the keyless template with fingerprints on the seated slots, which `ComposeWith` produces directly. No finding.

Gaps: M-3 (no named route to per-slot resolved origins for §7c), N-2 (`Composed` aliasing).

---

## Lens 4 — honesty of the loader fix (Task 2, Step 2a)

**The claim is true, and I reproduced its mechanism.** `md/testdata_test.go` has exactly three `var arr []byte` arms (lines 308, 314, 371 — `Hash256Body`, `Hash160Body`, `buildTLV`'s pubkeys). Every vendored `.descriptor.json` writes those as hex STRINGS: `md/testdata/vectors/keyed_wsh_timelock_hashlock.descriptor.json` carries `{"kind": "Hash256Body", "data": "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad"}` and `"fingerprints": [[0, "73…"]]`. I scanned all 29 vendored `.descriptor.json` for any JSON array-of-ints outside `MultiKeys.indices`: none. The two hash-bearing vectors (`keyed_wsh_timelock_hashlock`, `keyed_tr_pathological`) are reached by no current `loadDescriptor` caller (callers: `wsh_multi_chunked`, `wsh_sortedmulti`, `wpkh_basic`, `wsh_with_fingerprints`, plus `encode_test.go`/`encode_multisig_test.go` lists), which is why the arms have never fired — the plan's "the existing tests do not reach these arms and stay `ok`" holds. The base64 failure mode it describes is also right: the 64-char hex string is legal base64 and decodes to 48 wrong bytes without an error.

---

## Lens 5 — blast radius of the `PolicyShape` split

`grep -rn --include='*.go' '\.Branches\|PolicyShape'` over the fork at `169073c` gives exactly **one** production consumer outside `md/`: `gui/template_engrave.go:142` `policySummaryLines`, called from `templateConsentLines` at `:86`, whose single call site is `gui/multisig_build.go:804` (`confirmReviewScreen`, the paged form). It **does** render per-branch text — `Spend paths: N` plus one `  i: <k-of-N | X key(s), custom> [+timelock] [+hashlock]` line per branch — so after Task 4 a consumed card carrying `or_d`/`or_i`/`or_b`/`or_c`/`andor` in a wsh script shows N lines where it showed one.

That change is **intended** (§9 item 1 says "and `andor`, for consumed cards") and is an improvement in content. Overflow is not a risk: `confirmReviewScreen` pages. But nothing pins it — see **I-3** and **M-1**.

Also checked, no finding: adding `ClassKey`/`ClassHash`/`ClassNow` after `ClassTx` is safe. `gui/sysw_admit.go`'s `admitted` is a default-DENY map, so the new classes are refused by every program until S3 adds rows; nothing in the tree enumerates `Class` exhaustively; `Classify`'s dispatch is inserted after the three shipped prefix arms and before `classifyConstellation`, and no constellation string begins `key:`/`hash:`/`now:`, so no existing record changes class.

---

## Lens 6 — task ordering / TDD honesty

Each red step fails for the stated reason given what landed before it: Task 1 (`INCONCLUSIVE: no provenance pin` — and the sibling test fails on missing files), Task 2 (`undefined: Compose`; `toComponents` used by the test does exist, `md/encode_multisig.go:195`), Task 3 (`tagPkH` has no arm in `emitFragment`, so `ErrScriptUnsupported`), Task 4 (`unknown field LockOperands`), Task 5 (`undefined: ComposerStubs`/`AppendStubs`), Task 6 (`undefined: ClassKey`). Task 7 admits it is green at once. **Nothing else is secretly green.** Task 3's hedge about the tap-leaf test's construction is appropriate rather than evasive.

---

# Findings

### C-1 — `Branch` cannot distinguish `older` from `after`, so §7e's required lock rendering is unbuildable

**Plan:** lines 1802, 1804, 1953, 2058-2062. **Spec:** §7e ("MUST name, per path in listed order: … its lock kind and value in operator units (§6b echo form)"); §9 item 1 ("carry the lock operand and the digest on `Branch` so §7e can render them"); §6b's echo table (four kinds: blocks / days / height / date).

Task 4 adds `LockOperands []uint32` and a single `Timelock bool`. The operand alone does not determine the kind, because §4c's bands overlap on the wire:

- `older(n)` blocks is `1..=65535`; `after(h)` height is `1..=499 999 999` — the whole blocks band sits inside the height band.
- `older(0x400000+u)` units is `4 194 305..=4 259 839` — also inside the height band.

So `LockOperands = [26280]` is equally `older(26280)` (≈ 6 months of blocks) and `after(26280)` (a block height in 2013). Nothing in the extended `Branch` resolves it, and §7e's consent is required to be derived from the DECODED md1 — it may not fall back to UI state. The plan's own motivating example makes the error concrete: line 1804 renders `older(26280)` as **"after 26280 blocks"**, mixing the absolute keyword with the relative unit.

Consequence: Stage 3 either cannot write the §7e lock line at all, or writes one that can misdescribe a spend condition on the composer's consent surface — the screen whose whole purpose is that the operator sees what they are committing to steel.

**Reproduction:** compose `wsh` with paths `[2-of-3, 1-of-1 + older(26280)]` and with `[2-of-3, 1-of-1 + after(26280)]`. Both are legal under §4c. Under Task 4 the two produce `Branches[1] == Branch{Keys:1, Timelock:true, LockOperands:[]uint32{26280}}` — byte-identical summaries for two different wallets. (`keyed_compose_wsh_two_path_or_d` is the first of the pair and is already vendored.)

**Fix hypothesis (verify before adopting):** carry the tag alongside the value — either `LockKinds []LockKind` parallel to `LockOperands`, or better `Locks []Lock` reusing Task 2's `Lock{Kind, Value}`, whose four kinds are exactly §6b's echo rows (`older` blocks / `older` units / `after` height / `after` date), so `collect` maps `tagOlder` + bit-22 and `tagAfter` + the 500 000 000 threshold once and §7e renders from a value rather than re-deriving the split at the screen. Add a test row pairing `older(N)` and `after(N)` at the same N.

---

### I-1 — `Branch` cannot distinguish `sortedmulti` from `multi`, so §7e's EXPERIMENTAL marks cannot come from the decoded md1

**Plan:** line 1802 ("unchanged: … `K`/`N`/`Keys`"). **Spec:** §7e ("MUST name, per path in listed order: … and the EXPERIMENTAL marks"); §5's "unsorted where sorted was legal" row; §8b.

`plainMulti` (`md/policy_shape.go:173-191`) returns K/N for `tagMulti`, `tagSortedMulti`, `tagMultiA` and `tagSortedMultiA` alike and discards which one it saw, and Task 4 does not change that. Of the two §5 EXPERIMENTAL conditions, only the keyless one is recoverable from the extended `Branch` (`Keys == 0`, which `TestPolicyShapeReportsAKeylessAlternativeHonestly` relies on); "unsorted where sorted was legal" is not. `Composed.Experimental()` does carry it, but §7e explicitly forbids deriving the consent from builder/UI state ("derived from the DECODED md1 … never from UI state").

**Reproduction:** `PolicyShapeChunks` on `keyed_compose_wsh_sole_sortedmulti` and on `keyed_compose_wsh_unsorted_sole` (both vendored by Task 1, same 2-of-3 shape, one sorted and one not) returns the same `[]Branch{{K:2,N:3,Keys:3}}`.

**Fix hypothesis:** a `Sorted bool` (or a `Multi` tag enum) set by `plainMulti`/`branchOf` when K/N are set, and one test row over that vendored pair.

---

### I-2 — Task 6's fixture does not exist at the path the plan copies it from

**Plan:** line 2326 (`cp /scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/testdata/record_class_vectors.json sysw/testdata/`), and the sha256 `a894e619…` asserted as settled at lines 2321 and 2413.

`crates/me-cli/testdata/` at mnemonic-engrave master (`bb0c07f`) contains `codex32_seam_vectors.json`, `descriptor_seam_vectors.json`, `record_corpus_pre_s2.json`, `seal_vectors.json`, `show_public_records_pre_s2.txt`, `sysw_vectors.json` — and **no `record_class_vectors.json`**. The file exists only on the unmerged S1 branch: `/scratch/code/shibboleth/wt-composer-s1` (branch `composer-s1`, tip `90560cb`), added by `b474a31`. Its sha256 there is `a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3` — the plan's value is right; only its location is wrong.

The plan's Baselines line already knows S1 has not merged ("mnemonic-engrave: the S1 merge commit (record it in the fold that follows S1's ship)"), but Task 6's step is unconditional and its failure handling covers only a *changed* hash, not an absent file. As written, Task 6 halts at its first command, and the STAGED_PLAN §S1 exit condition it depends on ("the lockstep fixture … is READY for S2") is a precondition this plan never states.

**Fix hypothesis:** state the precondition at the top of Task 6 ("S1 merged to mnemonic-engrave master; record the merge SHA in Baselines"), and either sequence Task 6 after that merge or point the `cp` at the branch with the merge SHA recorded in the provenance pin's `commit`/`file_commit` (which the pin already demands as full 40-character SHAs).

---

### I-3 — the split changes a SHIPPED consent screen for real cards, and the plan's own guard for that cannot fire

**Plan:** Task 4 Step 4 — "The four pre-existing tests … are UNTOUCHED and still pass; if one moves, the split changed a shape the shipped consent screen shows -- stop and record which."

That guard is inoperative for the class it names. The four pre-existing `md/policy_shape_test.go` tests use `keyed_wpkh`, `keyed_wsh_multi_2of3`, `keyed_wsh_sortedmulti_2of3`, `keyed_tr_keyonly`, `keyed_tr_with_leaf`, `keyed_tr_sortedmulti_a`, `keyed_tr_depth2`, plus three hand-built trees — **none contains an `or_*` or `andor`**. The fork *does* vendor three `or_*` cards (`md/testdata/vectors/keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`, `keyed_wsh_timelock_hashlock`), and no shape test loads any of them. On the `gui` side, `TestTemplateConsentLines` asserts only substrings that do not depend on branch count, and the other two consent tests build `md.PolicyShape` literals by hand. So the one behaviour change this task makes to a shipped, funds-facing consent surface — how many spend paths a consumed `or_*` card reports, and what each line says — will be observed by nothing, in either package, before or after.

**Reproduction:** `PolicyShapeChunks(loadPhraseChunks(t, "keyed_wsh_or_d_degrading"))` returns one `Branch` today and several after Task 4; `policySummaryLines` on it goes from one `  1: …` line to several, and `Spend paths: 1` to `Spend paths: N`. No assertion anywhere notices.

**Fix hypothesis:** add one test over `keyed_wsh_or_d_degrading` (and/or `keyed_wsh_or_b`) pinning the post-split `[]Branch`, so the shipped screen's new output is a recorded decision rather than a side effect. If a `gui`-side pin is wanted, it is one `policySummaryLines` golden and stays inside Task 7's existing carve-out from "do not touch `gui/`".

---

### M-1 — the evidence cited for "no existing expectation moves" measures the wrong thing

**Plan:** line 1802 — "none of its vectors contains an `or_*` or `andor`; measured: `grep -c 'Or\|andor' md/policy_shape_test.go` = 0 at `169073c`".

That grep counts occurrences in the **test file's source text**; it says nothing about the contents of the vendored vectors the file loads by name. The conclusion is nevertheless true — I checked the seven vectors individually — but the stated measurement does not establish it, and the same grep would have returned 0 if `TestPolicyShapeDescribesRealCards` had listed `keyed_wsh_or_d_degrading`.

**Fix hypothesis:** cite the vector-side measurement instead, e.g. `grep -l 'or_i\|or_d\|or_b\|or_c\|andor' md/testdata/vectors/*.template` → `keyed_wsh_timelock_hashlock`, `keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`, none of which appears in `md/policy_shape_test.go`.

---

### M-2 — the address leg skips the change chain, which is vendored and free

**Plan:** line 1639 (`want := rec.Chains["0"].Addresses`) and the loop `for i := uint32(0); i < 2; i++` at 1642. **Spec:** §12 item 1 — "addresses (receive 0..1, change 0..1) … the Go builder reproduces every template, every CHUNK and every address byte for byte".

The vendored conformance records carry both chains: `keyed_compose_wsh_three_paths.conformance.json` has `chains` = `{"0": 3 addresses, "1": 3 addresses}`. `derivedKeys(t, rec, chain, index)` is already parameterised on `chain`, and `EmitWitnessScriptChunks` does not care. Checking chain `"1"` at indices 0 and 1 costs two more loop iterations and doubles the Step 5 mutation oracle from 10 to 20. The plan's scoping note at line 2993 honestly names the taproot and non-pkh gaps but does not mention that the change chain of the five covered vectors is also skipped.

**Fix hypothesis:** loop `chain` over `{0, 1}` in `TestPkhWitnessScriptsReproduceRustsAddresses`, and update the Step 5 expected mutation count.

---

### M-3 — no named route from `Composed` to the resolved per-slot origins §7c must print

**Plan:** Task 2 "Interfaces → Produces (Stage 3 calls these)", lines 299-301. **Spec:** §7c — "Slot @0 expects a key at m/48'/0'/0'/3'  (unseated slots only)".

`Composed` exposes `Slots`, `InternalKeyPath`, `Experimental`, `Chunks`, `Stub`, `TemplateID`, `Bind`. The resolved origins live in the unexported `c.d.pathDecl`; the plan's own test reaches them as `c.d.pathDecl.divergent`, which package `gui` cannot do. The route does exist — `Chunks()` → `md.ExpandWalletPolicyChunks(chunks)` → `[]md.ExpandedKey{OriginPath}` — and it is arguably the *right* route, since §7e's self-check must read the decoded md1 anyway. But it is nowhere named, so a Stage 3 implementer reading the "Produces" list finds no way to write the §7c line.

**Fix hypothesis:** either add one sentence to Task 2's Interfaces naming the `Chunks()` → `ExpandWalletPolicyChunks` route for §7c/§7e, or add `func (c Composed) Origins() [][]PathComponent`. The former costs nothing and keeps §7e's "from the decoded md1" property structural.

---

### M-4 — the Go tag-coverage mirror is weaker than the primary's

**Plan:** line 552 — `if tag == "spine:0" || tag == "no-corpus" { continue }`.

The primary (`crates/md-codec/tests/compose_vectors.rs:77-89`) exempts only `SINGULAR_TAGS = ["spine:0"]` (`compose_support.rs:306`) from the two-vector rule; `no-corpus` passes it on its own merits, appearing in exactly two rows. Rust additionally asserts that each singular tag appears *exactly once* and that §12 item 1's required-tag list is fully present. The Go mirror exempts `no-corpus` and has neither extra assertion, so dropping one of the two `no-corpus` rows from the Go mirror would be caught only by the `len(composeFamily()) != 28` check, and a `spine:0` duplicated by a mirror typo would not be caught at all.

**Fix hypothesis:** drop the `no-corpus` exemption (the rule already passes) and add the "singular tag appears exactly once" assertion; the required-tag list is the primary's acceptance and need not be duplicated.

---

### M-5 — S3 will port the five presets with no vendored oracle

**Spec:** §4d (five archetypes + plain k-of-n, "pinned templates"); STAGED_PLAN §S0 ("the five presets as path lists with pinned templates"), §S3 ("the shape flow with presets"). **Rust:** `crates/md-codec/src/compose/presets.rs` exports six constructors (`plain_multisig`, `simple_timelocked_inheritance`, `kofn_recovery`, `tiered_recovery`, `hashlock_gated`, `decaying_multisig`).

None of the 26 corpus names is a preset vector, and this plan vendors none. A preset is a normative `PathList` shape, so a Go re-authoring in S3 with no pinned oracle is precisely the drift the Rust-primary rule exists to prevent: the archetype's *parameters* (which tier unlocks when, which head is bare) would be decided in Go. This is a staging question rather than a defect in Task 1-9, but S2 is the stage that owns "everything the Stage 3 GUI will call".

**Fix hypothesis:** either vendor a preset vector per archetype into `md/testdata/vectors/` alongside the compose corpus (they are `md compose --preset …` outputs the exporter can already write), or record explicitly in the plan that the preset shapes are S3's and name where their oracle will come from.

---

### N-1 — refusal messages number paths from 0 where the primary numbers from 1

**Plan:** lines 1165, 1170, 1173, 1178 (`fmt.Errorf("%w: path %d", …, i)`). **Rust:** `compose/mod.rs` `Display` uses `path + 1` for `LockOnlyPath`, `KeylessUnderTr`, `BadThreshold` and `LockOutOfRange`.

Not operator-visible: §8m's five bodies name no path index, and the device shows §8m, not md's error text. But the file's header bills itself "a line-for-line port … Rust is normative", and §7d insists "'Path N' in every seating and mapping prompt is the OPERATOR's listed path index" — so if S3 ever formats a wrapped `ErrCompose*`, the two halves of the constellation will disagree by one.

**Fix hypothesis:** `i+1` in the four wraps, matching Rust.

---

### N-2 — `Composed` copies alias one `*descriptor`

**Plan:** lines 1030-1036, 1074.

`Composed{d *descriptor}` is returned by value but `Bind` has a pointer receiver and mutates through `d`, so `c2 := c; c2.Bind(...)` also keys `c`. Harmless in the plan's own tests and probably in S3, but the type reads as a value.

**Fix hypothesis:** one doc line on `Composed` saying a copy shares the underlying descriptor and `Bind` is not copy-on-write.

---

## Closing

**1 Critical / 3 Important / 5 Minor / 2 Nit.** The lowering port itself is the strongest part of the plan and I found nothing wrong with it beyond N-1; the two §7e-facing findings (C-1, I-1) are both in Task 4's extension of `PolicyShape`, where the plan promises a rendering the added fields cannot support, and both fixes are one field plus one test row.
