# Recon — coordinator-compat plan 1b: what the design no longer says correctly

Recon only. No code, no design edits. Measured 2026-09-23 against
descriptor-mnemonic `main` **d269c556** (md-codec 0.47.0, md-cli 0.19.0) and
the seedhammer fork `main` **2c9eed3**. Toolchain 1.85.0,
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/recon-1b-target`. "Design" means
`design/DESIGN_coordinator_compatibility.md` at engrave `10102369`. Line numbers
are that file's unless another file is named.

## Headline

1. **Measured: the key cannot see one Nunchuk verdict.** Two kind-1 cards with
   **byte-identical `SkeletonKey`s** get opposite Nunchuk verdicts: libnunchuk
   `a7cfb49` (the 2.1.1 pin) REFUSES one and ACCEPTS the other. The only
   difference between them is the order of the leaf pubkeys (§3c). The design's
   blind-spot list (294-307) has no entry for this. Its "D1/D2 containment"
   (309-310) also cannot catch it, because the conflict is between two pieces
   of evidence, not between a rule and evidence. Key membership is design-fixed
   ("a plan may not choose it", 312), so **this is a design question to settle
   before 1b**, not something the plan can decide.
2. **The Skeleton does distinguish kind 0 from kind 1, and it has to.** 1a's
   conformance gate still holds at d269c556: 48 vectors, including 2 of kind 1,
   and all four gate commands are green.
3. **Two more hand-written Liana classifiers exist than the design knows
   about.** md-cli's `liana_refuse_or_warn` (F-449 stage 2) is one, and the fork
   side has more (§2). A verbatim port of the Go Liana rule onto the Rust shape
   would also **double-count** an unlocked path (§1d).

---

## 1. What F-449 (or anything since 6cbd49d8) made false

### 1a. Skeleton / SkeletonKey versus kind 0 and kind 1

**It distinguishes the two, in two independent places.** Measured with
`cargo run -p md-codec --example dump_skeleton_keys`, which printed 48 lines
(transcript: `.tmp/recon-1b-keys.tsv`). The NUMS twin and the kind-1 card of the
same wallet differ in exactly two tokens:

    keyed_compose_preset_kofn_recovery  tr(50929b74…3ac0,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(older-blocks#1))}) ␟ [[[0,1,2]][[3]]][[0][1][2][3]] ␟ Nums
    keyed_tr_liana_kofn_recovery        tr(UNSPENDABLE(liana),{…identical…})                                                                          ␟ [[[0,1,2]][[3]]][[0][1][2][3]] ␟ LianaUnspendable

- **Template.** `render.rs:201-207` emits `LIANA_UNSPENDABLE_MARKER`
  (`nums.rs:32`, `UNSPENDABLE(liana)`) for kind 1 and the NUMS hex for kind 0.
  Both render modes go through that site.
- **Trailer.** `skeleton.rs:270-276`'s `key_path_kind_label` gained
  `LianaUnspendable` (`skeleton.rs:275`), fed by `policy_shape.rs:284-288`
  (`InternalKey::{Slot→Xpub, NumsPoint→Nums, LianaUnspendable→LianaUnspendable}`).
- **Path 0 is correct for kind 1.** The key-path branch is pushed only for
  `InternalKey::Slot` (`policy_shape.rs:301`), so a kind-1 card has no unlocked
  path 0. That matches the Liana v15.0 evidence.
- **It needs to distinguish them.** They are different wallets: F4 of the
  stage-4 plan measured Template-IDs `f99cc42e…` against `8107216…`, and the
  addresses differ. Evidence from one must not answer for the other. Across the
  48 vectors the trailer counts are NotTaproot 28, Xpub 10, Nums 8,
  LianaUnspendable 2.

**What the key still conflates, measured (see §3c):** kind-1 cards whose leaf
pubkeys are in different orders. Nunchuk's verdict depends on that order.

### 1b. The 1a conformance gate at d269c556 — it holds

| check | result |
| --- | --- |
| `cargo test -p md-codec --test skeleton_key_conformance --test skeleton_key --test policy_shape --test partitions --test render_abstract` | 2 + 13 + 19 + 8 + 5 pass |
| `cargo nextest run --locked -p md-codec` | **637 passed**, 3 skipped |
| `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --document-private-items --all-features` | rc 0 |
| `cargo clippy --locked --all-targets -- -D warnings` | rc 0 |
| `cargo fmt --check` | rc 0 |
| GitHub CI on d269c556 (`gh run list --commit`) | CI / release / fuzz-smoke all `success` |

Corpus: 48 `keyed_*.conformance.json` files, 2 of them kind-1
(`keyed_tr_liana_kofn_recovery`, `keyed_tr_liana_nested_two_recoveries`, added
at `430ea478`).

Caveats the plan author should carry:

- The floor is still `checked >= 40`
  (`tests/skeleton_key_conformance.rs:718`). The module doc still says "46 of
  them" (`:4`, and at `:41ff`).
- The descriptor route recognises kind 1 with a **test-private** recogniser,
  `is_liana_unspendable_key` (`:240-269`), which has no near-miss vector
  (**F-655**; its claim that the mutation survives 2/2 was not re-measured
  here). A second copy of the same leaf-walk ships in md-cli
  (`decompose/walk.rs:259-282`). Both call the single derivation
  `md_codec::nums::liana_unspendable_xpub` (`nums.rs:50`).
- **The gate runs over vendored vectors, not over evidence.** Design §2 (475-477)
  says "for every evidence row". Measured: **0 of 11** Liana live-gate rows and
  **1 of 56** matrix shapes (`X12-wsh-samefp-across-paths`) have their
  descriptor in the 48-vector corpus. The evidence descriptors are multipath
  with different keys. The vectors are per-chain (`/0/*`).
- **The descriptor-to-`Descriptor` walker is test-private**
  (`tests/skeleton_key_conformance.rs`). md-codec has **no library route** from
  descriptor text to a key. md-cli has one, but design §5 step 1 (633-637) and
  plan 1a Task 5 forbid a step-1 gate from depending on md-cli. So 1b's table
  generator needs either the walker (and a recogniser) promoted into md-codec,
  or evidence keyed from md1 chunks. Plan 1a Task 5 Step 3 named this as "a
  finding, not a fixup".

### 1c. Design text that is now false, or whose citation drifted

| design line | claims | now (measured) | moved by |
| --- | --- | --- | --- |
| 330-339 | a `tr` "path" includes "the key path as path 0 **when the internal key is not NUMS**… gated on `!is_nums`" | gated on `InternalKey::Slot` (`policy_shape.rs:301`). Kind 1 is not NUMS **and has no path 0**. The code is right; the sentence is wrong. | F-449 1b |
| 217, 223 | `Body::Tr { is_nums, key_index, tree }` (`tree.rs:49-57`) | `Body::Tr { internal_key: InternalKey, tree }` at `tree.rs:69`; `enum InternalKey { Slot(u8), NumsPoint, LianaUnspendable }` at `tree.rs:23-30`. The design marks it CORRECTED, but the cite still points at old code. | F-449 1b |
| 280, 364 | `canonicalize.rs:168` | `canonicalize.rs:159` | F-449 |
| 605 | `validate.rs:449-458` | `validate.rs:444-453` (same text: "stopped being READABLE…") | F-449 |
| 432 | `composerLianaOutsideModelClass` at `gui/composer_consent.go:381` | **`:396`** (+15 at stage 4) | F-449 4 |
| 207 | `branchOf`, `md/policy_shape.go:239-245` | **`:265-271`** (valid at `7b6f2fb`) | F-449 3 |
| 361, 366 | `render.rs:52`, `:159`, `:171` | `:146`, `:280`, `:299`. Already `:146/:267/:286` at `6cbd49d8`, so stale **since 1a**, not F-449. | 1a + F-449 |
| MEASUREMENTS 82-85 | `render.rs:159/:171`, hash renderers `:287/:306` | `:280/:299`, `:426/:466` | 1a + F-449 |
| 341-345 | the serialization is "template, U+001F, then the partitions" | **three** fields: template ␟ partitions ␟ key-path kind (`skeleton.rs:313-319`). `md shape-key`'s spelling must follow the code, not this paragraph. | 1a |
| 569 | CONTINUE live on every page, `gui/multisig_build.go:1898` | still `:1898` | — (holds) |
| 518-524, 545 | the KNOWN_RELEASES gate "would fail today on Liana (8.0 verified, 15.0 known)" | **false for Liana.** v15.0 is verified (289/289, and the live gate pins `v15.0`/`4684d5cb`, `scripts/liana-live-gate.sh:72`). `git ls-remote --tags` on liana today: the newest release tag is **v15.0**. For Nunchuk: libnunchuk upstream HEAD `33f7dc3` is **204 commits** past `a7cfb49`, and `src/descriptor.cpp` changed (`ParseTrDescriptor` key-path handling and musig; `GetUnspendableXpub` and the round-trip check are untouched). Whether the **app** moved past 2.1.1 was not measured. | stage 2 re-measure |
| 488-494 | "The device is shipping a present-tense claim about Liana… from a 21-month-old reading" | half-fixed: §8x now says "Liana (as of v15.0)" (`gui/composer_copy.go:316`). **Every Core claim is still unversioned** ("Bitcoin Core imports…", in 5 bodies; see §2). | F-449 4 |
| 510-517 | "Nunchuk's and Core's [harnesses] still live in `/scratch/.tmp`" | Core now has a **committed one-shot script**, `design/evidence/f449-stage4/core-kofn-import.sh` (one descriptor, dev build). Nunchuk's harness is still only at `/scratch/code/shibboleth/.tmp/fable-nunchuk-lib/build/fableharness`. | F-449 4 |
| 294-307 | the key's blind spots: lock values, digests, renderer | **missing: the leaf-key order** (§3c, measured collision) | F-449 (kind 1 made it reachable) |
| 229-233, 309 | a disagreement over an ambiguous case "surfaces as a D1/D2 build failure" | D1-D4 (449-454) are all **rule vs evidence**. Two evidence rows with one key and opposite outcomes have **no class**. | F-449 |
| 456-459 (D3) | "refused for Y" can be compared with a rule's reason | Liana's refusal text for **every** kind-1 refusal in evidence is the one generic string "Descriptor is not compatible with a Liana spending policy." (live gate: 4 of 4 refusals plus the control; probes: 5 of 5). CONTINUITY_f449_stage2:57-61 says it covers ≥3 causes. **D3 cannot attribute a reason on kind-1 refusals.** | F-449 2/4 |
| 633-635 | the table generator is a step-1 `xtask` | no `xtask` exists. Plan 1a put the one key implementation in `crates/md-codec/examples/dump_skeleton_keys.rs`. | 1a |
| 14 | "Not a spec yet" | still true. 1a was planned straight from the design at `83ab725c`. 1b would be too, unless the operator wants a spec first. | — |

### 1d. Traps in the "rule moves to Rust" step (§2, 431-433)

- **Double count.** The Go rule does `if shape.KeyPath == md.KeyPathSpendable
  { unlocked++ }` (`gui/composer_consent.go:422`) because the Go walk has no
  key-path branch. The Rust shape already carries that branch as an unlocked
  `Branch` (`policy_shape.rs:301-309`, empty `locks`). A verbatim port counts a
  spendable key path **twice**. md-cli already documents this
  (`cmd/compose.rs:625-636`: the conjunct is "REDUNDANT with the walk"). The Go
  and Rust `branches` lengths also differ for every `tr` Xpub policy, which
  plan 3's cross-language vectors must allow for.
- **Kind-1 rulings to carry over.** Class 2 fires on `KeyPathNUMS` only
  (`:412`), and the unlocked count adds `KeyPathSpendable` only (`:422`), so
  kind 1 is neither (stage-4 F2, pinned by `TestComposerLianaClassRulings`).
- **A second host-side Liana classifier already ships:** `liana_refuse_or_warn`,
  `crates/md-cli/src/cmd/compose.rs:638-666` (hashlock and every-path-timelocked
  → warn; §6 refusals via `validate_unspendable_shape`, `validate.rs:585`).
  F-644's md-cli half asks to *widen* it. 1b's registry must subsume it, or it
  becomes the second copy of one rule.
- **Still open from 1a, and not filed:** final-review M-1 (deferred-minor #1,
  "owning phase: before plan 1b's first rule reads `k`/`n`"). `plain_multi`
  sets `br.n = nkeys` with no `nkeys == slots.len()` guard
  (`policy_shape.rs:454-457`; `plain_multi` returns `indices.len()` at
  `:540-544`, duplicates included). A grep of both FOLLOWUPS files for
  `plain_multi`/`nkeys`/`plan 1b` finds nothing. **The 1a report's six
  deferred minors were never filed.**

---

## 2. The "five hand-written notices" — what exists now

Scanned: string literals (comments stripped) of every `func` in `gui/*.go` and
`md/*.go`, excluding tests, for coordinator names.

**At fork `7b6f2fb` (when the design was written) seven `composerCopy*` bodies
named a coordinator**, plus `composerCopyNothingChecked` with the generic word.
The design's "(§8a, §8f, §8x and two more)" is therefore ambiguous. The likely
five are §8a, §8f, §8x, MIXED LOCK BASES and §8g (two bodies). §8s's Core
sentence is a sixth candidate.

**At fork `2c9eed3`** (`gui/composer_copy.go` unless noted):

| func:line | § | names | kind of claim |
| --- | --- | --- | --- |
| `composerCopyKeylessPath` :94 | §8a | Core, Nunchuk, Liana | refusal (all three) |
| `composerCopyNUMS` :199 | §8f | Core, Nunchuk, Liana | Core imports; Nunchuk cannot import NUMS |
| **`composerCopyLianaKeyPath` :220** | §8y (F-633 status) | Liana v15.0, Core, Nunchuk | **NEW at stage 4.** Liana (as of v15.0) and Core import; "Nunchuk imports it only when the keys happen to be in sorted order" |
| **`composerCopyUnspendableRowNUMS` :241** | §0b row 1 | Core, Liana, Nunchuk | **NEW.** Core imports; Liana and Nunchuk do not |
| **`composerCopyUnspendableRowLiana` :247** | §0b row 2 | Liana v15.0, Core, Nunchuk | **NEW.** "…Nunchuk only by chance." |
| `composerCopyLianaKeyDropped` :254 | §0b reset | "Liana key" | **key-kind name, not a verdict** |
| `composerCopyLianaUnmet` :264 | refusal | "Liana key" | **key-kind name, not a verdict** |
| `composerCopyMixedLockBases` :284 | lens-2 I-2 | Nunchuk, Core | Nunchuk refuses; Core imports |
| `composerCopyOutsideLianaModel` :314 | §8x | Liana v15.0, Core | Liana class; "Bitcoin Core imports it." |
| `composerCopySameSeedThreshold` :324 | §8g #1 | Liana | "Liana will refuse it." |
| `composerCopySameSeedBelow` :332 | §8g #2 | Liana | "Liana will refuse it." |
| `composerCopyDuplicateKeys` :633 (text :675-676) | §8s | Core | "Bitcoin Core refuses such a descriptor." |
| `composerCopyNothingChecked` :460 | §8l | "coordinator" | generic |

Outside `composerCopy*`:

- **`composerUnspendableDropCause`, `gui/composer_unspendable.go:141`:** "Liana
  would not import this policy (" + class + ")." This is a verdict string.
- Key-kind labels only: `md1KeyPathLine` (`gui/md1_inspect.go:212`, "Key path:
  Liana key") and `policySummaryLines` (`gui/template_engrave.go:167`,
  "Key-path: none (Liana key)").

**Stage 4 added three verdict-bearing bodies and one non-`composerCopy*`
verdict. That makes eight coordinator-verdict bodies, or eleven if the key-kind
mentions are counted.** Consequences for design §4's r3 M-3 gate ("no
`composerCopy*` body names a coordinator", 584-587):

- As worded, it would **fire** on `LianaKeyDropped` and `LianaUnmet`, which
  name the *key kind*, not a verdict. It needs a kind-name exemption, or a
  rename of the kind.
- It would **miss** `composer_unspendable.go:141`, which is not a
  `composerCopy*`.
- Five bodies carry **unversioned Core positives**. The concurrent Core
  boundary report (below) has raw data showing Core ≤25.2 refuses every
  `tr`+tapscript-miniscript shape. So these are present-tense claims that are
  false for some Core an operator may run.

---

## 3. Coordinator facts on the F-449 record: evidence or prose?

No evidence registry exists in code. `EvidenceRow`, `CoordinatorId`,
`enum Verdict`, `KNOWN_RELEASES` and `shape-key` get **0 hits** under
descriptor-mnemonic `crates/`. So none of the facts below is an `EvidenceRow` in
§1A's sense. None carries §1A's `renderer`, `measured_at` or `library_rev`
fields. The table records how durable each one is today.

### 3a. Liana v15.0 — committed evidence, partly re-run as a gate

| fact | where | status |
| --- | --- | --- |
| kind-1 accepted: kofn, tiered, same-seed-two-paths, X19, **nested two-recovery** | `design/evidence/f449-stage2/liana-live-gate-expected.jsonl` (line 1 = tag `v15.0`, commit `4684d5cb`; 11 verdict rows incl. 2 controls) | **committed + re-run** by `scripts/liana-live-gate.sh` |
| kind-1 refused: hashlock-gated, decaying-multisig, hash160, X20 | same file | committed + re-run. **Message is the generic string for all four** |
| F-644 shapes ×4 and §7's one-timelocked-leaf refused; nested and pk-primary accepted | `design/evidence/f449-stage4/liana-probes{.sh,-in.txt,-out.jsonl,-recomputed.txt}` (7 rows) | **committed one-shot. NOT in the live gate** |
| the same 9 cases, vendored into the codec | descriptor-mnemonic `crates/md-codec/tests/fixtures/liana/cases.json` (9 entries) | vendored. `vendor-liana-evidence.sh` rebuilds it wholesale from a hardcoded name list (CONTINUITY_f449_stage2:62-63) |
| nested taptree, as a codec vector | `crates/md-codec/tests/vectors/keyed_tr_liana_nested_two_recoveries.*` | vector (F-640 closed) |

### 3b. Bitcoin Core and kind 1

- **Committed:** `design/evidence/f449-stage4/core-kofn-import{.sh,.txt}`. One
  descriptor (live-gate row 2). Self-reported
  `Bitcoin Core daemon version v30.99.0-64a7c7cbb975`, a **dev build**.
  `importdescriptors` succeeded and `getnewaddress` 0..2 equal Liana's receive
  addresses. This is a one-shot, not a gate. **A dev build is not a release an
  operator can run.** The design's `Version`, "the one the APPLICATION
  displays" (532-536), says nothing about dev builds.
- **Concurrent and UNCOMMITTED, not verified by me:**
  `design/agent-reports/coord-compat-core-boundary.md` and
  `design/evidence/coord-compat-core-boundary/core-*.json` (10 releases,
  24.2-31.1, written at 13:11-13:12 today by another agent; untracked). The raw
  JSON shows: kind-1 chain-0 `getdescriptorinfo` ok from **26.0**; kind-1
  **multipath** refused through **28.4** (`tr(): Key path value '<0;1>' is not
  a valid uint32`) and ok from 29.4. So **Core's verdict is renderer-dependent
  too.** Design §1 (96-100) presents renderer dependence as Nunchuk's property
  only. Its report states the tapscript-miniscript boundary as 26.0.
- **Prose only:** device copy `composerCopyLianaKeyPath` and `…RowLiana`
  ("Bitcoin Core import[s] it"), unversioned.

### 3c. Nunchuk and kind 1 — prose in the repo; measured here for the first time

**In the repo it is prose only:** SPEC §7 (`SPEC_liana_unspendable_internal_key.md:661-664`),
fable r0 M-5 (`agent-reports/f449-spec-r0-fable-adversarial.md:101`), the
stage-4 plan's own "never run on kind 1"
(`IMPLEMENTATION_PLAN_f449_stage4_device.md:3385`), and the device copy.
The source it rests on checks out: libnunchuk `a7cfb49`
`src/descriptor.cpp:689-712` (`GetUnspendableXpub`: `std::sort` +
`std::unique`) and `:638-646` (the re-render must equal the input, "Failed to
verify wallet descriptor").

**Measured now** with the existing harness
(`/scratch/code/shibboleth/.tmp/fable-nunchuk-lib/build/fableharness`, libnunchuk
`a7cfb49` = Nunchuk 2.1.1's pin, `HOME` sandboxed). Inputs were built by
`.tmp/recon-1b-nunchuk-probe.py` from `cases.json[0]`; the script asserts the
Liana recipe reproduces `expected_xpub`. Output is in
`.tmp/recon-1b-nunchuk-probe.out`. **Scratch files, not committed.**

| row | internal key | leaf pubkeys | Nunchuk `ParseWalletDescriptor` |
| --- | --- | --- | --- |
| `k1-liana-unsorted` (the live-gate kofn descriptor) | Liana recipe | not ascending | **REFUSE** (`Failed to verify wallet descriptor`; code -1017) |
| `pr1746-nunchuk-form` | PR-1746 recipe, same leaves | not ascending | **ACCEPT** |
| `k1-liana-sorted` (same 4 keys, permuted so the pubkeys ascend) | Liana recipe (= PR-1746, asserted) | ascending | **ACCEPT**, `template=DISABLE_KEY_PATH`, receive and change 0..2 **equal `md address`'s** for the same card |

Then both kind-1 descriptors were carried through
`md decompose --emit commands` → `md encode` (8 strings each, recognised as
`UNSPENDABLE(liana)`), and their `SkeletonKey`s were computed with a throwaway
binary over `md_codec::skeleton` (`.tmp/recon-1b-skelkey/`):

    k1-liana-unsorted  tr(UNSPENDABLE(liana),{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(older-blocks#1))}) | [[[0][1][2]][[3]]][[0][1][2][3]] | LianaUnspendable
    k1-liana-sorted    tr(UNSPENDABLE(liana),{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(older-blocks#1))}) | [[[0][1][2]][[3]]][[0][1][2][3]] | LianaUnspendable

**The keys are identical and the verdicts are opposite.** Leaf-pubkey order is
key material. No `Skeleton` field records it, and no rule can compute it from
the `Skeleton`. So a Nunchuk × kind-1 cell cannot be expressed by a rule, and a
measured positive keyed this way would wrongly extend to about (n!−1)/n! of the
cards sharing that key. Two options, and choosing between them is the design's
job, not 1b's (312): **(i)** add a "leaf keys sorted-unique" component to the
key, which changes key membership and the serialization; or **(ii)** rule that
Nunchuk × `LianaUnspendable` is always `Unproven`. That is silent, which ruling
3 allows, and the device copy's "only by chance" stays documentation.

---

## 4. Nunchuk's own unspendable form (PR-1746): where it belongs

- **md1 cannot encode it.** SPEC §4a (`SPEC_liana…:525-527`: "libnunchuk's
  PR-1746 form (a real wallet Nunchuk builds, for which md1 has no wire
  encoding)"). The kind field is one bit, and a third recipe "would require
  version 12" (SPEC §3d, `:339-341`), which is the last usable generation
  (§3a, `:260-265`).
- **What happens to it today, measured** (row 2 above): `md decompose
  --emit template` yields `tr(@0/<0;1>/*,…)`, a plain **Slot** with an
  origin-less `@0`. `--emit commands` refuses ("1 key(s) state no origin").
  `md encode` of the template **succeeds** (a template-only card, 5
  placeholders), and `md decode` reads it back as a Slot. A PR-1746 wallet
  therefore reaches the classifier only as **`KeyPathKind::Xpub` plus an
  unlocked branch 0**. That is a spendable-looking key path. The design already
  names this ambiguity (229-233).
- **What the design implies.** The key is "Computed from a DECODED md1 only"
  (110). A form md1 cannot carry has no key and no verdict of its own. Encoding
  it is a normative wire change: risk-set (c), its own codec cycle, and it
  spends the last version. **It is not in 1b, 2 or 3.** It belongs on the
  design's "Open" list as a future wire question, not a blocker.
- **1b's obligation.** No rule may emit a Nunchuk `Imports` for a `tr` with an
  `Xpub` internal key on the assumption that the key spends, and the
  template-only case is `Unproven { KeysAbsent }` anyway (a2). The kind-1
  ordering question in §3c *is* 1b's to receive, but it needs a design ruling
  first.

---

## 5. FOLLOWUPS.md entries owned by, or bearing on, this cycle

The file was parsed by header plus `**Status:**` line. The only entry whose
Status names the cycle as owner is **F-633**.

| entry | status / owner | relevance to 1b |
| --- | --- | --- |
| **F-633** (`FOLLOWUPS.md:19129`) | OPEN, **owned by the coordinator-compat cycle** (§3). Copy half done at stage 4 (`a054ead`). "OPEN for the KNOWN_RELEASES gate." | KNOWN_RELEASES is mechanism 4, **plan 2** (design 545; Scope 663-666 "the plan that closes F-633"). Not 1b. Its v15.0 note that a class-4 "a hash lock" is less specific than Liana's message is the D3 example (456-459). |
| F-644 (`:19669`) | OPEN. md-cli half owned by "the next descriptor-mnemonic release" | **Overlaps 1b directly**: it wants per-shape Liana warnings on `md compose`, which is 1b's verdict. Re-own it to 1b, or expect the second-classifier problem (§1d). |
| F-655 (`:19861`) | OPEN, next dm release | the test-private kind-1 recogniser the conformance gate depends on has no near-miss vector |
| F-634 (`:19252`) | OPEN, "next md-codec validation pass (NOT coordinator-compat plan 1a, which only surfaced it)" | not owned. `skeleton()` accepts `wsh(tr(…))` |
| F-624 (`:18979`) / F-626 (`:19002`) | OPEN; docs / copy-pass owners | the design's own evidence for the renderer on the tuple (96-100) and for `ImportsAltered` (405-409). Not owned. |
| F-625 (`:18990`), F-629 (`:19057`) | OPEN; Nunchuk UI walk / demo | Nunchuk context only |
| F-508 (`:16990`), F-509 (`:17029`) | OPEN, **no owning phase in the Status line**; both "RESOLVED AGAINST CURRENT CORE… a version floor" | these are the Core multipath and tapscript-miniscript floors that the Core RuleSet is supposed to encode. Candidates to re-own to the cycle. |
| **unfiled** | 1a final review (`agent-reports/coord-compat-1a-final-review.md:368-377`), six deferred minors, #1 owned "before plan 1b's first rule reads `k`/`n`" | not in either FOLLOWUPS. #1 is still live (§1d). |
| records | F-449 Status is still `OPEN` (`FOLLOWUPS.md:15686`) though CONTINUITY_f449_stage2:13 says complete | not this cycle's, but a grep for open items will show it |

---

## 6. What plan 1b's scope is, according to the design

- **Scope §1** (659-662): plan 1 = "md-codec + md-cli (steps 1-2) — the
  canonicaliser, the verdict types, the generated table, `md shape-key`, the
  CLI verdict and the none-case refusal."
- **§5 step 1** (629-637), after the port, render mode and Skeleton that 1a
  shipped: "the four `Verdict` kinds, and the generated `verdicts` table with
  vectors pinning every `(shape, coordinator, version)` cell". The generator is
  a step-1 artifact, and `md shape-key` must agree with it by test. *Gate:* the
  conformance test and the rule/evidence build.
- **§5 step 2** (638-641): `md shape-key`; the verdict on `md compose` and
  `md descriptor`; `md compose`'s none-case refusal and `--md-only`. *Gate:* CLI
  vectors for the flag path, template-only `Unproven { KeysAbsent }`, and
  `md descriptor` NOT refusing (§4, 599-609).
- **§3 mechanisms 1-2** (542-543): provenance on every verdict and no
  open-ended span belong to plan 1.
- **Plan 1a's own hand-off** (`IMPLEMENTATION_PLAN_…1a_skeleton.md:42-44`,
  `:730-735`): Verdict kinds, `ReadSet`, `EvidenceRow`, `MeasuredAt`, the rules,
  the table, `md shape-key`, `md compose`'s refusal, D1-D4, (a2) and (a3).
- **Not 1b:** harnesses, KNOWN_RELEASES, renderer gate, evidence snapshots
  (mechanisms 3-6 → plan 2, 544-547); everything in the fork (plan 3).
- **§5's four-command gate** (618-625) applies to every step. It is green at
  d269c556 (§1b).

**Blocking a spec, per the design** (675-688), updated:

- Core boundary: the concurrent report says 26.0 (uncommitted).
- K: a fork measurement, so it does not block 1b.
- **Add from this recon:** the Nunchuk leaf-order question (§3c).
- **Add from this recon:** how evidence that lives in mnemonic-engrave reaches
  a table built and gated inside md-codec, with no descriptor route in the
  library (§1b).
