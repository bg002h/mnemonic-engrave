# IMPLEMENTATION — F-533: refuse taproot internal-key reuse

**Result: the plan was implemented as written. All fork gates green. The corpus
distribution is exactly the plan's acceptance table.**

Worktree: `/scratch/code/shibboleth/.tmp/seedhammer-f533`
Branch: `f533-bip388-reuse`, four commits on top of fork `main` at `e4ab97d`
(the plan's recorded baseline — the fork had not moved, so the GREEN had not
expired).
Tip: `e84c6101afbc6b4e38b27b4c90a0b7d0854f6563`

Nothing pushed, nothing tagged, no CI config touched. `mnemonic-engrave` was
not modified; this report is the one file written there.

I did not disagree with any of the plan's design decisions. One thing the plan
did **not** contain is recorded below as **An eighth place**, and one item is
outside my write scope: **FOLLOWUPS.md**.

---

## 1. What changed and where

### Commit 1 — `476249f` `md: pin the two taproot key-reuse vectors fork-side (F-533, F-529)`

The plan's F-529 decision, executed before the predicate.

| file | what |
| --- | --- |
| `md/testdata/forkbuilt/keyed_tr_multi_a.md1.txt` | NEW. The vendored chunk set, fork-side. |
| `md/testdata/forkbuilt/keyed_tr_sortedmulti_a.md1.txt` | NEW. Same. |
| `md/f533_pinned_vectors_test.go` | NEW. `pinnedChunks` / `md1Lines` loaders, a **drift gate** (pin == vendored while both exist; skips with a reason once the re-vendor removes the vendored copy) and a **shape gate** (taproot body, `is_nums=false`, internal key's slot occurring inside the taptree). |
| `md/duplicate_keys_test.go` | `vectorChunksFor` prefers the pin. |
| `gui/taproot_script_path_test.go` | `loadVectorChunks` prefers the pin. |

Putting the preference in the two loaders rather than at each call site means
every current and future reader of those two vectors survives the re-vendor,
in both packages. `md/testdata/forkbuilt/` was chosen because F-531 already
established it (`md/dup_seat_fixture_test.go`) and because
`md/compose_vectors_pin_test.go:135` fails on any file in `testdata/vectors/`
that is not in the provenance pin — so the copies could not live there.

No non-test file is touched by this commit.

### Commit 2 — `98992e7` `md: the taproot internal key is a use-site, and BIP 388 counts it (F-533)`

`md/duplicate_keys.go`:

* new kind `DuplicateTaprootInternalKey`, added at the end of the `iota` block,
  with its own doc (why it is not a Core verdict, what its harm is, what would
  falsify it — nothing Core does);
* the `tagTr` arm now, **after** the existing `duplicateInTapTree` check,
  counts `b.keyIndex` against the taptree **only when `!b.isNums`** and returns
  `(b.keyIndex, DuplicateTaprootInternalKey)`;
* the file header paragraph that said the remedy was "a SECOND predicate …
  not a change to this one" is replaced (plan place 3);
* `DuplicateKeySlot`'s own doc comment is rewritten — it described a
  within-one-expression rule that is no longer the whole of it (plan place 7).

**Precedence.** The within-leaf finding is tested first. Both harms can be
present at once and only one sentence is shown; keeping the older,
better-measured one first also means this change moved **no** policy that
already reported a duplicate, so the entire corpus delta is the two vectors
that reported `DuplicateNone` before. Pinned by
`TestAWithinLeafDuplicateStillWinsOverTheInternalKey`.

`md/f533_internal_key_reuse_test.go` (NEW): five hand-built rows (placeholder
internal key reused; reused at `@1` rather than `@0`; NUMS beside a leaf using
`@0`; a different slot in every leaf; no taptree at all), the two pinned
vectors, and the precedence pin.

`md/duplicate_keys_test.go`: the two rows asserting `DuplicateNone` for
`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` now assert
`DuplicateTaprootInternalKey` (plan place 4). Their rationale keeps Core's
measurement and names **which authority decides each row**. The test was
**renamed** `TestDuplicateKeySlotMatchesBitcoinCore` →
`TestDuplicateKeySlotOnTheCorpusKeyReuseVectors`: two of its three rows are no
longer Core's answer, and a name promising they are is the next reader's trap.
The Core 31.1 table is unchanged and still carried.

### Commit 3 — `b850bdf` `gui: BIP 388's voice for the third duplicate kind, and the gates that count it`

`gui/composer_copy.go` — the new kind's own branch **before** the
unconditional return:

```
Check before funding: slot @N fills both the key path and a script key,
which BIP 388 forbids.
```

"Key path" is this firmware's own word for the taproot internal key —
`composerCopySeatKeyPathPrompt` says *"slot @N, key path (spends alone)"* on
the screen the operator seated it from. "Internal key" is BIP 341's word and
one this device has never shown them. The function's doc now says "THREE
HARMS, THREE SENTENCES" and names which of the three is not Core's.

Gates that enumerate kinds by hand, each given the new row:

* `gui/composer_copy_test.go` — the verbatim §8s table;
* `gui/modal_fits_test.go` — both forms, keyed (warning + refusal) and keyless
  (warning alone);
* `gui/composer_flow_test.go` — `TestDuplicateWarningNamesTheRightHarm` gets a
  row asserting the sentence **says** "BIP 388" and **does not say** "Bitcoin
  Core refuses";
* `gui/composer_flow_test.go` — `TestConsentWarnsOnDuplicateKeys`' two taproot
  rows (plan place 5), plus its condition, which read
  `got != (tc.want == md.DuplicateRefusedByCore)` — identical while the taproot
  rows were `DuplicateNone`, and would now have asserted that a
  BIP-388-forbidden wallet gets **no** warning. It is `!= md.DuplicateNone` now.

`gui/duplicate_seat_address_test.go` needed **no** edit, as the plan's round 4
recorded. Confirmed by running it.

`gui/f533_taproot_reuse_address_test.go` (NEW) — see §4, the branch check.

Comment updates: `gui/policy_address.go:48-56` (plan place 1) and
`gui/wallet_policy.go:345` (plan place 2).

### Commit 4 — `e84c610` `gui: the two F-533 vectors are refused by POLICY, not missing a capability`

See **An eighth place** below.

---

## 2. Corpus distribution — BEFORE and AFTER, verbatim

Command, run from `/scratch/code/shibboleth/mnemonic-engrave`:

```
python3 scripts/policy-generate.py --corpus --fork /scratch/code/shibboleth/.tmp/seedhammer-f533
```

**BEFORE** (my worktree at `e4ab97d`, before any edit):

```
corpus: 70 vectors
  expand           1
  ok/agrees        45
  ok/refused       2
  source           22
every vector matches the baseline
```

**AFTER** (the committed tree, `e84c610`):

```
corpus: 70 vectors
  expand           1
  ok/agrees        45
  source           24
  MOVED    keyed_tr_multi_a: baseline {'device': 'ok', 'rust': 'refused'}, now {'device': 'source'}
  MOVED    keyed_tr_sortedmulti_a: baseline {'device': 'ok', 'rust': 'refused'}, now {'device': 'source'}
A MOVED line is the device or the primary changing its mind about a policy. Read it before rewriting the baseline.
```

Against the plan's acceptance table:

| bucket | plan wants | measured |
| --- | --- | --- |
| `ok/refused` | **absent**, not "0" | **absent** — the line is not printed |
| `source` | 22 → **24** | 22 → **24** |
| `ok/agrees` | **45, unchanged** | **45** |
| `expand` | 1 | 1 |

Only the two target vectors MOVED. `ok/agrees` holding at 45 is the
over-refusal guard and it held.

**`--write-baseline` was NOT run.** The baseline is
`mnemonic-engrave/design/policy-corpus-baseline.json`, which the brief reserves
to the controller. The script `return 1`s while the baseline still records the
old verdicts for those two vectors; that non-zero is the *intended* delta
awaiting the controller's baseline commit, not a failure.

### How I confirmed the run built MY worktree

An A/B, not an inference. The same command pointed at the **unchanged shared
checkout** on the same tree state:

```
$ python3 scripts/policy-generate.py --corpus --fork /scratch/code/shibboleth/seedhammer
corpus: 70 vectors
  expand           1
  ok/agrees        45
  ok/refused       2
  source           22
every vector matches the baseline
```

`--fork` is the checkout the probe is `go run` from (`probe_batch`,
`cwd=fork`) **and** the source of `md/testdata/vectors`. Two different paths,
two different answers, from one command differing only in that flag.

---

## 3. The reddening set — enumerated by name, and how taproot-ness was decided

**Taproot-ness came from the DECODED WRAPPER.** A throwaway internal test in
package `md` reassembled each of the 70 `*.phrase.txt` vectors and asked
whether `d.tree.body` is a `trBody`, then read `isNums`, `keyIndex`, and the
slots occurring in `*b.tree` via `countKeySlots`. Results were joined against
`policy-corpus-baseline.json` by stage. The tool was deleted before the gates
were run; it is not in any commit.

It reproduces the plan's round-3 correction exactly:

* **23** decoded-wrapper taproot vectors; **18** of them `ok/agrees` — the plan
  records the real figure as 18 against a name-grep's 17.
* A name-grep for `tr` returns **24**, and the two sets differ in three places:
  `tr_keyonly` and `tr_with_leaf` match the name but do not decode at all
  (`md: wire version mismatch`; both are already `source`), and
  `keyed_compose_preset_kofn_recovery` is taproot but the name does not say so.
  That last one **is** in the reddening set, so the name-grep would have
  under-counted it.

**Enumeration** — NUMS internal key (`isNums=true`) whose taptree references
slot 0, which is the slot the bogus zero collides with when `!isNums` is
dropped. Nine vectors:

| vector | stage |
| --- | --- |
| `compose_tr_thirty_two_slots` | `source` — **EXCLUDED, cannot redden** |
| `keyed_compose_preset_kofn_recovery` | `ok/agrees` |
| `keyed_compose_tr_hash_leaf` | `ok/agrees` |
| `keyed_compose_tr_nums_three_leaves` | `ok/agrees` |
| `keyed_compose_tr_sole_sortedmulti_a` | `ok/agrees` |
| `keyed_compose_tr_two_path_distinct_fingerprints` | `ok/agrees` |
| `keyed_compose_tr_two_path_nums` | `ok/agrees` |
| `keyed_compose_tr_unsorted_sole_leaf` | `ok/agrees` |
| `keyed_tr_pathological` | `ok/agrees` |

**Reddening set = enumeration ∩ `ok/agrees` = 8**, matching round 2's
measurement. `keyed_compose_tr_sole_sortedmulti_a` and
`keyed_compose_tr_unsorted_sole_leaf` — round 1's confirmed C1 members — are in
it, and they are a floor, not the set.

Separately, the vectors the NEW predicate reports (`isNums=false`, `keyIndex`
occurring in the taptree, previously `DuplicateNone`) are exactly **two**:
`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a`. No other vector is affected in
either direction, which is why `ok/agrees` is unchanged.

---

## 4. Mutations — every one grep-counted as APPLIED before its result was trusted

Each was applied by `sed`, the marker counted, the suite run, the file
restored from a pristine copy, and the marker counted back to 0.

| id | mutation | marker count | result |
| --- | --- | --- | --- |
| **M1** | one character of the pinned `keyed_tr_multi_a` chunk 0 (`md1fdg29ps9q` → `…9r`) | `md1fdg29ps9r` = 1 | drift gate **RED**, naming chunk 0 and printing both strings |
| **M2** | `if !b.isNums {` → `if true { // F533MUT2` | `F533MUT2` = 1 | `md` **RED** on 3 tests; corpus `ok/agrees` 45 → **37**, with the **eight MOVED exactly the reddening set above**, and `compose_tr_thirty_two_slots` correctly absent |
| **M3** | `if !b.isNums {` → `if false { // F533MUT3` | `F533MUT3` = 1 | `md` **RED** on 3 tests; corpus **`ok/refused` returns to 2**, "every vector matches the baseline" |
| **M4** | `return b.keyIndex,` → `return 0, // F533MUT4` | `F533MUT4` = 1 | **RED** only on the "@1 at the internal key" row |
| **M5** | the two blocks in the `tagTr` arm swapped, internal key first | `F533MUT5` = 1 | **RED** only on `TestAWithinLeafDuplicateStillWinsOverTheInternalKey` |
| **M6** | `if kind == md.DuplicateTaprootInternalKey {` → `if false && …` | `F533MUT6` = 1 | copy table + `TestDuplicateWarningNamesTheRightHarm` **RED**; the fallthrough printed exactly *"Check before funding: slot @3 repeats in one script, and Bitcoin Core refuses such a descriptor."* — the false sentence the plan predicted. `modal_fits` stayed green, correctly: it measures fit, not truth. |
| **M7** | M3 again, run against the new gui branch gate | `F533MUT7` = 1 | the gate fails at its own **precondition**, not at steps 3–4 (see below) |
| **M8** | the gate line in `complexAddressSource` → `… && false` | `F533MUT8` = 1 | steps 3 and 4 **RED**, printing the addresses the device would have shown: `bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt` and `bc1p588jmtx4ptv76t9sclt6gt33eyydvsrea4njyayerqj2frw5m5aq5gzycw`; steps 1 and 2 stayed green |

M2's corpus result is the plan's central acceptance claim and it is exact: the
`!isNums` mutation reddened **every member** of the reddening set — all eight,
by name, not a count taken on faith.

**M7 is recorded because my first mutation note was wrong.** I had written that
dropping the internal-key term would fail the new gui gate at steps 3 and 4.
Run, it fails at the precondition instead (`the fixture reports 0 (@0), not the
kind F-533 is about`) — which is the precondition doing its job, since without
it the test would pass for a policy F-533 is not about. The comment in
`gui/f533_taproot_reuse_address_test.go` now states both mutations and what each
actually does.

### The probe wraps the ROUTER — checked, not assumed

`cmd/policyprobe`'s header warns that wrapping the wrong branch "manufactures
device findings"; the inverse is the risk here, a corpus result that moved for
some other reason looking exactly like the gate working. So
`TestF533RefusalHappensAtTheBranchTheDeviceTakes` makes four separate claims per
vector, all measured green:

1. the flat `*bip380.Descriptor` route does **not** admit these, so
   `policyAddressAt` falls through to `complexAddressSource` — the branch the
   gate is on;
2. `complexAddressDeriver`, the body **below** the gate, **does** derive them,
   so the refusal is the gate and not an emitter that cannot do the work;
3. `complexAddressSource` refuses;
4. `policyAddressAt` — what the inspect screen and the probe both call —
   refuses.

M8 shows 3 and 4 failing while 1 and 2 stay green, which is what makes the
missing address the *gate's* doing.

---

## 5. Copy — the line count, measured

The 7-line page is the **word-wrapped-at-20** count, and the method was
validated against the two numbers already recorded in `gui/composer_copy.go`
before being used on the new sentence:

| body | chars | wrap-20 lines |
| --- | --- | --- |
| shipped Core sentence (comment says "96 chars, 6 lines") | 96 | **6** ✓ |
| its 122-char predecessor (comment says "8 lines") | 122 | **8** ✓ |
| shipped fewer-keys sentence | 115 | 7 |
| **new BIP 388 sentence** | **94** | **6** |

Six lines against a page that holds seven.

Through `showError` — the modal this body actually reaches since F-531, via
`duplicateRefusalBody` — the modal-fit gate reports, on the real frame:

```
F-533's inspect refusal, taproot internal key, keyed: 141 chars drawn in full, headroom 418 chars (margin 80)
F-533's inspect refusal, taproot internal key, keyless (warning alone): 77 chars drawn in full, headroom 476 chars (margin 80)
```

The new kind does **not** land on "This device can't derive addresses" or
"Complex policy - display only": `gatheredDescriptorFlow` returns through
`duplicateRefusalBody` before the routing, and
`TestF533SurfacesSayWhyThereIsNoAddress` asserts the modal carries both the
warning and `composerCopyNoAddressesDuplicateKeys`.

---

## 6. An eighth place — the plan named seven

`gui/policy_address_test.go`'s `TestEveryKeyedVectorReachesAnAddress` went
**red**:

```
--- FAIL: TestEveryKeyedVectorReachesAnAddress/keyed_tr_multi_a
    policy_address_test.go:197: keyed_tr_multi_a (tr(@0/48'/0'/0'/2'/<0;1>/*,multi_a(...))) reaches NO address route — an operator sees "display only"
```

It is not in the plan's list of seven, and nothing in those seven points at it.
It was found by running the gates, not by reading — it is one `go test ./gui/`
shard.

The fix is unambiguous and stays inside the plan's intent, so I made it rather
than stopping: F-531 built this test **two** maps precisely so a vector that
stops deriving must be classified by hand. The two vectors go in
`refusedByPolicy`, not `stillUnsupported`, and the difference is *checked*:
that branch re-derives through `complexAddressDeriver` and **fails if no
deriver exists beneath the gate**, then runs the full cross-language
conformance — every index of every chain, byte-equal to the Rust primary's
vendored addresses, with a zero-comparison treated as a failure. Both vectors
pass it. The device declines to show addresses it would have got right.

Flagging it because the plan's "seven places" list is now an **eight** places
list, and a future re-read of the plan will not find this one.

---

## 7. Gates

All run from `/scratch/code/shibboleth/.tmp/seedhammer-f533` with
`/scratch/code/shibboleth/.toolchain/go/bin` on PATH (`go version go1.26.7
linux/amd64`; `gofmt` from the same directory — it is not on the default PATH,
and a missing one prints nothing, which reads as a clean tree).

**`go build ./...`** — clean, no output.

**`go vet ./...`** — 10 lines, all `testing.ArtifactDir requires go1.26 or
later (file is go1.25)`, in files I did not touch. **Confirmed pre-existing** by
running the same command at fork `main` in the shared checkout: identical
output.

**`go test ./sysw/`**

```
ok  	seedhammer.com/sysw	0.039s
```

**Non-gui packages** (`go list ./...` = 76, minus `./gui` = 75, run via
`xargs go test`): **exit 0**, `55` ok, `20` "no test files", **0 FAIL**.
Includes `ok  	seedhammer.com/md	0.076s`.

**`./gui/` sharded ×24** —
`mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`:

```
=== enumerating tests in ./gui/ ===
    1340 top-level tests
    partition verified exhaustive: 1340 == 1340
=== running 24 shards in parallel (timeout 20m each) ===
...
=== wall: 23s ===
RESULT: ok -- all 1340 tests ran across 24 shards
```

The brief's figure is 1338. The count is **1340** because this work adds two
top-level gui tests (`TestF533RefusalHappensAtTheBranchTheDeviceTakes`,
`TestF533SurfacesSayWhyThereIsNoAddress`). 1338 + 2 = 1340, and the script's own
exhaustiveness assertion (`1340 == 1340`) is what says no test was dropped.

**`gofmt -l .`** — exactly the recorded five-file baseline, no more, no less:

```
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
```

`git status --porcelain` is empty: nothing uncommitted, and the two throwaway
measurement tests were deleted before the gates ran.

---

## 8. What I could not do

**`design/FOLLOWUPS.md` — plan place 6 — is UNDONE.** F-533's entry is in
`mnemonic-engrave`, which the brief puts off-limits. Four passages in it are now
false and belong to the controller:

1. the `ok/refused 2` code block and *"which is two vectors the device derives
   an address for and the Rust primary refuses"* — the bucket is gone;
2. *"So the device has one predicate serving two rules, and it is the wrong one
   for the refusal"* — it is one predicate serving two rules by design now, and
   the plan's round 1 settled that a second would be unsound in this wire
   format;
3. *"**What closing it needs** (and why it is not a one-line widening): a second
   predicate in `md` … Then the refusal moves onto it while the F-514 warning
   stays on `md.DuplicateKeySlot`"* — this is the decision the plan reverses;
4. the *"Re-measured 2026-09-16 at fork `ec11fab` — still exactly two"* block,
   whose distribution is now the BEFORE.

Also for the controller: the corpus baseline rewrite
(`--write-baseline`), which must land in its own commit **after** the
distribution above, never as the check itself.

**Not attempted, and out of scope by the plan's own text:** the
`KeyAtDisjointUseSites` port (round 1 I2 — declined), and any widening to a
slot occurring in two different leaves (the wire cannot make that shape
BIP-388-forbidden, and `TestDuplicateKeySlotScopesPerTapLeaf` still pins it as
legal).

**No device walk.** Everything here is host-side; the screens are asserted on
rendered frames by the modal-fit gate, not on hardware.

**One note on this file's path.** This agent is isolated in the engrave
worktree
`/scratch/code/shibboleth/mnemonic-engrave/.claude/worktrees/agent-af9f5a927d3318276`,
whose harness refuses direct writes to the shared checkout. The report was
therefore authored at the same relative path inside that worktree and copied to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/impl-F533-round1.md`,
where the brief asked for it. Both copies are byte-identical; the worktree one
can be deleted.
