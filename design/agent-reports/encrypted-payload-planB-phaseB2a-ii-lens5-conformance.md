# B2a-ii whole-diff review — LENS 5: SPEC CONFORMANCE AND PLAN FIDELITY

Reviewer: independent subagent (opus), 2026-08-08.
Diff: `feat/encrypted-payload-b2a-ii`, `421dca8..d0baf13` (9 commits: 4 implementation,
5 review folds — the brief said 10/6, `git log --oneline` says 9/5).
Normative: `design/SPEC_encrypted_payload_delivery.md`. Plan:
`design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md`.

**Verdict: 0 Critical / 1 Important / 3 Minor / 2 Nit.**

All work was done in a private copy (`/tmp/lens5-*`, since deleted). The shared
worktree was never written to — verified `git status --porcelain` empty and
`diff -rq` of my copy against the original clean before deletion.

---

## 1. §10.2 steps 5–9, §10.2.1, §10.2.2, §10.2.3, §10.3 — clause by clause

Every normative sentence in scope, with the code that implements it. Line numbers
are from the branch tip.

### §10.2 step 5 — "Enter the existing 12-word BIP-39 entry flow."
`unlockPassphraseFlow` (`gui/unlock_kdf.go:70`) → `emptyBIP39Mnemonic(12)` +
`inputWordsFlow(ctx, th, m, 0, "")`. **Implemented.** It deliberately does *not*
call `seedEntryFlow`, whose 12/24 picker §8 forbids; the plan's "Carried-forward
citations" section already settled that and the code matches it. The `""` title
preserves `"Word %d of %d"`, which the harness's `mustReach("Word 1 of 12")`
depends on.

### §10.2 step 6 — "Validate the BIP-39 checksum … No KDF is run."
`unlockAttemptOnce` (`gui/unlock_kdf.go:217`): `if !isMnemonicComplete(m) ||
!m.Valid() { return errUnlockChecksum }`, textually above `passphraseBytes` and
`unlockDerive`. **Implemented.** Message: `"Not a valid passphrase, check the
words."` (`:274`, and `:103` for the in-flow re-prompt) — matches the spec string.
**Verified by mutation** (my own run, §4 below): deleting the gate makes
`TestUnlockChecksumGateRunsNoKDF` fail *on the counter*, not the return value.

### §10.2 step 7 — "PBKDF2 with a progress indicator … the screen must say so"
`unlockDerive` (`:162`): one `d.Step(kdfStepIterations=500)` per frame, `%d%%`
via `ctx.Styles.progress`, and `unlockKDFLead` (`:148`) which estimates from *this*
derivation rather than from §7.1's table. **Implemented.** `ctx.WakeupAt(time.Now())`
keeps the loop running rather than idling. `d.Done()*100/d.Total()` cannot divide
by zero: `Total()` is `h.Iterations`, which `Inspect` has already bounded to
[100 000, 2 000 000] for a sealed payload, and `NewDeriver` clamps to ≥ 1 anyway.

### §10.2 step 8 — AEAD open over `AAD = header ‖ public section`, fail closed, both readings, hash on screen
`seal.Opener.UnlockWithKey` (`seal/unlock_key.go:31`) takes the AAD from
`blob[:split]` — the blob's own bytes — so it binds the header *and* every public
record. On `Open` error it returns without touching `p`. `unlockSealedFlow`'s
`errors.Is(err, seal.ErrAuthentication)` arm renders `unlockRetryBody`
(`gui/unlock_kdf.go:238`), which is literally *"Wrong passphrase, or this payload
has been altered."* plus the record count, the SEALED/UNSEALED shape and
`seal.FormatHash(p.Hash)`. **Implemented**, and it degrades correctly when
`pub_len == 0` (`!p.HasHash` → no invented constant digest, per §10.2 step 3's
"furniture" argument).

### §10.2 step 9 — split and allow-list the decrypted section
`UnlockWithKey` → `SplitSection` → cross-section `MaxRecords` check → `AdmitSection(recs,
SectionEncrypted)`. **Implemented.** The cross-section total is computed here
because it is the only place both counts exist — the plan's reasoning, faithfully
transcribed.

### §10.2 step 10 — "Wipe the derived key, the passphrase buffer, and PBKDF2 intermediates on every exit path"
`defer clear(pass)` (`gui/unlock_kdf.go:221`), `defer clear(key)` (`:227`),
`defer d.Wipe()` (`:166`). **Implemented in code, but with no test that can
fail — see Important I1.**

### §10.2's closing rule — "The passphrase prompt is conditional on `ct_len > 0` and nothing else."
`unlockPayloadFlow` gates the whole of steps 5–9 on `p.Header.Sealed()`
(`gui/unlock_flow.go:97`). **Implemented**, and instrumented:
`unlockPassphraseHook` fires at prompt entry and
`TestUnlockNeverPromptsWhenNothingIsEncrypted` asserts it never fired for vector E,
with `TestUnlockPromptsOnASealedPayload` as the positive control. Confirmed by my
own mutation (§4).

### §10.2.1 — the allow-list
Unchanged from Phase A; B2a-ii routes through it rather than around it. The
irreversible branch is closed at the layer that enforces it:
`TestEncryptedDebugCommandRejectsTheBundle` (`seal/open_test.go:439`) drives a
`command: lock-boot` at position 3 of 6 through the *whole* `Open` pipeline and
asserts `ErrRecordNotPermitted` with `p == nil`. At the gui layer that surfaces as
`unlockSealedFlow`'s `default:` arm → *"Payload unreadable."* → `return false`, so
no plate list is built. `gui`'s `testPlatform.LockBoot` panics, so a gui test that
somehow reached it would fail loudly. **Satisfied.**

### §10.2.2 — the session lifecycle
| clause | code | verdict |
| --- | --- | --- |
| every secret offered FIRST, consecutively | `unlockSecretSession` (`gui/unlock_session.go:77`) collects `at` by `seal.IsSecret`, loops, then `unlockPlateListFlow` runs *after* it returns (`gui/unlock_flow.go:114-115`) | implemented |
| plural, not singular | `at` is a slice; vector F's three `ms1` are pinned by `TestSecretSessionOffersEverySecretFirstAndInOrder` | implemented |
| Cut or Skip; either way the record leaves RAM before the next is offered | `defer p.WipeSecretAt(i)` at `:105`, plus `clear(rec)` at plate construction (`:184`, `:269`) | implemented |
| a cancelled or failed engrave wipes too | `clear(rec)` fires *before* `Engrave`, so cancel/fail are the same state | implemented |
| plate list labels by CLASSIFIED type and index, never the sealer's claim, never the contents | `plateLabel` (`gui/unlock_platelist.go:43`) off `AdmittedRecord`'s classifier-derived HRP/card fields | implemented |
| records already cut are marked, as a convenience not a guarantee | `unlockPlate.cut` → `" (cut)"`; set only on `unlockEngraveFlow == true` | implemented |
| leaving by ANY path wipes everything | `defer p.Wipe()` (`gui/unlock_flow.go:85`), registered immediately after `Inspect` | implemented |

### §10.2.3 — the unauthenticated warning
Unchanged from B1; still gated on `!p.Header.Sealed()` and still reached before
the plate list. **Implemented.**

### §10.3 — UI constraints
- Fourth-nav-affordance panic: every `layoutNavigation` call B2a-ii adds or
  changes carries ≤ 3 `NavButton`s — `unlockDerive` 1, `unlockPlateListFlow` 3,
  `SeedScreen.Confirm` 1–2 (+ a separate single-button `nav2`), `ChoiceScreen` 2.
  **Respected.**
- Plate list uses `bundleReviewFlow`'s paged shape with Back/Page/OK.
  **Respected.**
- **Back IS Lock**: `backBtn.Clicked` → `return` → `unlockPayloadFlow` returns →
  `defer p.Wipe()`. Icon is `assets.IconDiscard`, pinned on *pixels* by
  `TestPlateListBackIconIsDiscardNotBack` (which first asserts the two references
  differ, so it discriminates in both directions). **Implemented.**

### §11.2 — REQUIRED ASSERTIONS, one by one

| §11.2 assertion | present? |
| --- | --- |
| decrypt A, B, C, D; parse E | yes — `TestUnlockWithKeyReproducesUnlock` (A,B,C,D,F,G) + `TestUnlockWithKeyRefusesAnUnsealedPayload` (E) |
| vector E reaches the plate list with the keyboard flow NEVER ENTERED, by instrumentation | yes — `TestUnlockNeverPromptsWhenNothingIsEncrypted` + positive control |
| §6.6 hash literal for D and E | yes — from `seal/testdata/vectors.json`'s `pubhash_sealed`/`pubhash_unsealed` (retyping is forbidden by `seal/testdata/README.md`) |
| every secret offered before any public plate; each zeroed after its plate leaves, incl. cancelled and failed | yes — `TestSecretSessionOffersEverySecretFirstAndInOrder`, `…WipesEachBeforeTheNextIsOffered`, `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`, `…CancelledEngraveLeavesNothing`. "Failed" collapses into "cancelled" because the wipe precedes `Engrave` |
| idle timer paused/armed | **B2b** — correctly recorded as deferred, not claimed |
| vector F: three secrets, consecutive, each zeroed before the next | yes |
| BCH-valid-but-undecodable `md1`; uppercase refusal; E-shape hash sensitivity; 8191-LF alloc bound; §6.2 bounds before the KDF; unencrypted-shape rules; tag mismatch; erased region; secret-in-the-clear; §6.4 container cases; pre-split alloc count; vector C positive; space-grouped rejection | yes — all in `seal`, unchanged and still green |
| BIP-39 checksum rejection without the KDF | yes — `TestUnlockChecksumGateRunsNoKDF`, on the counter |
| classifier allow-list / `LockBoot` never reached | yes — see §10.2.1 above |
| **wipe on every exit path: the plaintext record buffer, the derived key, and the passphrase buffer, asserted on the buffers** | **record buffer only, on one of four exits — I1 and M1 below** |
| too many records is not "unreadable" | yes — `TestUnlockDistinguishesTooManyRecords`, `TestUnlockTooManyRecordsIsNotReportedAsAWrongPassphrase` |

---

## 2. Shipped files vs the plan's whole-file Go blocks

Extracted every ```go block under a "new file" anchor and diffed against the
branch tip:

```
seal/unlock_key.go       vs plan §5a   IDENTICAL (74 lines)
seal/session.go          vs plan §5b   IDENTICAL (53 lines)
gui/unlock_kdf.go        vs plan §5c   IDENTICAL (291 lines)
gui/unlock_plates.go     vs plan §7b   IDENTICAL (83 lines)
gui/unlock_flow.go:95-117 vs plan §5d  IDENTICAL (the 23-line fragment, byte for byte)
gui/gui.go §6c fragments vs plan       APPLIED VERBATIM (+ one additive comment)
gui/unlock_session.go    vs plan §6b   DIFFERS — see below
```

`gui/unlock_session.go` is the only whole-file block that differs, and every delta
is a **declared review fold** carried in its own commit message:

- `991bee8`/`3c477b9` rewrote the honest-caveat inventory (lens 1 D1, I1);
- `d0baf13` replaced the across-all-secrets index with a per-class one (lens 1 pass 3);
- `c785322` moved `clear(m)` off the defer to beside `clear(rec)` (lens 1 C1);
- `5633a79` bounded the `unlockSecretHook("wiped", …)` index.

No silent divergence. `seal/open.go`'s tail matches §5a's replacement text
including the retained `end` guard and the retained `if !h.Sealed() { return nil }`.

---

## 3. The four declared deviations

**(a) Commit sequencing of §5d's fragment — SOUND.** Task 5's commit cannot
reference `unlockSecretSession` or `unlockPlates`, so `ce023ca` ships a truncated
fragment that `return`s where Tasks 6/7 will attach, with a comment saying so.
Measured, not argued:

- all nine commits type-check — `go vet ./gui/ ./seal/ ./bip39/` clean at every
  one, modulo the pre-existing go1.26 `ArtifactDir` line;
- all nine are green — `go test ./gui/ ./seal/ ./bip39/` `ok` at every one;
- the branch tip's fragment is **byte-identical** to the plan's §5d block
  (`diff` empty).

No intermediate commit leaves seed material resident: `defer p.Wipe()` lands in
`ce023ca`, the same commit that first decrypts.

**(b) `click` rather than `press` in §6c — CORRECT, AND THE PLAN WAS WRONG.**
`Clickable.Next` sets `clicked = !e.Pressed && c.Pressed` (`gui/widget.go:74-79`),
and `press()` (`gui/event_test.go:57`) emits only `Pressed: true`. The plan's
`press(&ctx.Router, Button2)` could therefore never click: the negative half would
have been vacuously true and the positive control ("with `NoEdit` clear, both
still do") could never have passed — the same shape as R0 round 1's I1. The test
uses `click()` and says why at `gui/unlock_session_test.go:574-577`. I verified the
substitution is what discriminates: mutating the guard back to `if
editBtn.Clicked(ctx)` fails **only** the Button2 subtest —
`"Button2 still reaches word entry with NoEdit set"`.

**(c) §6d's plate-list row moved to Task 7 — SOUND.** The row is "no secret is
ever in the plate list (F, C, G)" and it asserts on `unlockPlates`, a symbol Task 7
creates; it cannot compile inside Task 6. It exists as
`TestUnlockPlatesNeverIncludesASecret`, covers C/F/G, carries a premise check
("vector %s carries no secret to exclude") and additionally pins the entry count
at `len(Public) + len(Secret) - secrets`.

**(d) Mutation runner left uncommitted — NOT SOUND, Minor M3 below.**

---

## 4. §11.3 rows this phase owns — do the named killers actually kill?

§11.3's table has **27** rows (measured: `sed -n '1547,1573p' SPEC… | grep -c '^| '`),
matching the plan's claim. Task 8 claims 11 of them, ten killed and one deferred to
B2b. I re-ran **five** of the named killers plus one extra, in a private copy, each
substitution asserted to have matched exactly once and each file restored from a
file copy:

| §11.3 mutant | mutation applied | result |
| --- | --- | --- |
| only the first secret record offered | `at = at[:1]` in `unlockSecretSession` | **KILLED** — `TestSecretSessionOffersEverySecretFirstAndInOrder`: `never reached "SECRET seed material"; last frame "…ms11/3"`, plus 5 others |
| `ms1` not wiped after its plate | `clear(rec)` moved below `Engrave` in `unlockEngraveCodex32` | **KILLED** — `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`: `record 0 is STILL RESIDENT while its engrave screen is up: "ms10entrsqq…"` |
| passphrase prompted when `ct_len == 0` | `unlockPassphraseFlow(ctx, th)` inserted on the unsealed path | **KILLED** — `TestUnlockNeverPromptsWhenNothingIsEncrypted` (+ all three `TestUnauthenticatedWarning*`) |
| BIP-39 checksum check removed | gate deleted from `unlockAttemptOnce` | **KILLED** — `the checksum-invalid passphrase ran the KDF 1 times`, and `a partial passphrase ran the KDF 1 times; the isMnemonicComplete half of the gate is missing` |
| wipe omitted on the Back exit path | `defer p.Wipe()` deleted | **KILLED** — `a PUBLIC record survived the flow's exit: "mk1qpz63tpq…"` |
| (extra) `NoEdit` guards the layout only | `!s.NoEdit &&` removed from the click handler | **KILLED** — Button2 subtest only, which is the discriminating half |

Every named killer I checked kills, and each fails with the diagnostic it was
written to produce rather than incidentally. **I found no false kill.** I did find
a class of mutant nobody ran — I1.

---

## 5. In the diff but not in the plan

Three files the plan never names are touched. All three are declared in the commit
that carries them, none is a silent drive-by:

- `bip39/bip39.go` + `bip39_test.go` — `Parse` preallocated to `make(Mnemonic, 0, 24)`
  (`d0baf13`, lens 1 pass 3). Not a normative-behaviour change (no wire format,
  identity, validation or admission semantics move), so the Rust-primary rule does
  not bind the change itself — but see N2.
- `gui/gui.go` `deriveMasterKey` / `masterFingerprintFor` / the `SeedScreen`
  validity probe — `defer wipeBytes(seed)`, `defer mk.Zero()`, and zeroing the key
  that used to be discarded into `_` (`3c477b9`, lens 1 I1). **I checked these for
  correctness because they are shared funds-path code:** `hdkeychain.NewMaster`
  (btcutil/v2 v2.0.0) HMACs the seed and does not retain it; `ECPubKey` returns a
  freshly parsed `*btcec.PublicKey`, not an alias of `k.pubKey`; and
  `bip32.Fingerprint(pkey)` runs before any defer. `masterFingerprintFor` derives
  its own local `mk` from a `bip39.Mnemonic`, so no caller's key is zeroed.
  `gui_test.go:754-758`'s bare-vs-passphrase fingerprint assertions would catch
  corruption, and they pass. **Sound.**
- `seal/record.go` `Classify` — `clear(m)` on `bip39.Parse`'s `[]Word`
  (`3c477b9`, lens 1 M1). Local, no contract change.

The one behavioural detail that is *not* declared anywhere is M2 below.

---

## FINDINGS

### I1 — Important — §10.2 step 10's three wipes have no test that can fail; §11.2's "the derived key, and the passphrase buffer MUST read as zeroed" is unimplemented

**Where:** `gui/unlock_kdf.go:166` (`defer d.Wipe()`), `:221` (`defer clear(pass)`),
`:227` (`defer clear(key)`); spec §10.2 step 10 and §11.2's "Wipe on every exit
path" bullet.

**Defect.** §11.2 requires, verbatim: *"the plaintext record buffer, **the derived
key, and the passphrase buffer** MUST read as zeroed. Asserted **on the buffers
themselves**, via a fake platform that drives the flow to each exit — never on a
return value."* The record buffer is asserted (`TestUnlockFlowWipesEveryRecordOnExit`,
`gui/unlock_plates_test.go:425`). **The derived key and the passphrase buffer are
asserted nowhere.** The only test that touches the passphrase buffer at all,
`TestPassphraseBytesIsSection81sNormalisedForm` (`gui/unlock_kdf_test.go:473-479`),
calls `clear(got)` *itself* and then checks the result is zero — that tests the Go
builtin, not the flow. The plan never scheduled the assertion either (no
occurrence of "passphrase buffer" in it), so the gap was inherited, not introduced.

**Evidence — measured, in a private copy, suite run to completion:**

```
mutant: delete `defer clear(pass)` AND `defer clear(key)` from unlockAttemptOnce
  $ go test ./gui/ ./seal/
  ok  seedhammer.com/gui   17.073s
  ok  seedhammer.com/seal  13.234s          <-- SURVIVED

mutant: delete `defer d.Wipe()` from unlockDerive
  $ go test ./gui/
  ok  seedhammer.com/gui   17.472s          <-- SURVIVED
```

Neither mutant appears in the plan's §5.6 table nor in Task 8's recorded set, so
the phase's headline "30 mutants run across Tasks 5-8, 29 KILLED, 1 SURVIVING" is
short by three survivors, all of them on a normative §10 step rather than on the
one non-observable heap-residency row the plan predicted.

**Consequence.** This is the *only* code path on which a seed passphrase and a
seed-decrypting AES-256 key exist in RAM at all, and §10.2 step 10 is the clause
that says they must not persist. Today the code is correct; nothing can tell you
when it stops being. B2b is the immediate risk: F-89 requires the idle timer to
**UNWIND the flow**, which means reshaping `unlockDerive`/`unlockAttemptOnce`'s
return paths — exactly the edit that drops a `defer` — and every gate would stay
green while the passphrase and key stayed live in a machine an operator has walked
away from. That is the §2.2 item 9 window this whole feature's threat model turns on.

**Fix.** Cheap, and the seam already exists. `newDeriver` is a swappable var
(`gui/unlock_kdf.go:51`) and `installKDFCounter` already wraps it. Have the wrapper
retain the `passphrase` slice header it is handed, and add a one-line
`unlockKeyHook func(key []byte)` beside `unlockPassphraseHook` (or return the
`*seal.Deriver` and read `d.Key()`'s buffer) so the test holds the key too. Then
drive `unlockAttemptOnce` to each of (a) success, (b) tag failure, (c) Back during
the derivation, and assert both buffers read all-zero after it returns — plus the
`Deriver`'s `u`/`acc` via `d.Key() == nil` after `Wipe`. Add the three mutants to
Task 8's row set and re-record the total.

---

### M1 — Minor — §11.2's "each exit" is exercised for one of the four named exits

**Where:** `gui/unlock_plates_test.go:425` `TestUnlockFlowWipesEveryRecordOnExit`.

**Defect.** §11.2 names four exits — "Lock, Back, an error path, and `ctx.Done`".
The test drives exactly one: Back on the plate list (which is also Lock, per
§10.3's amendment, so two of four collapse legitimately). No test leaves via an
error path or via `ctx.Done`.

**Evidence.** `grep -n "allZeroBytes" gui/*_test.go` finds flow-scope residency
assertions only at `unlock_plates_test.go:451,468`, both inside that one test.

**Consequence.** Low today: `defer p.Wipe()` is a single mechanism covering all
four, and I confirmed deleting it is killed. The exposure is that the *shape* the
spec asks for — one assertion per exit — is not what would catch a future
`p.Wipe()` that becomes conditional on how the flow was left, which is precisely
what B2b's timer-unwind introduces.

**Fix.** Two more subtests on the existing harness: call `quit()` mid-flow for
`ctx.Done`, and drive a record whose `validateMdmk` fails for the error path;
assert the same held buffer.

---

### M2 — Minor — the plate list's labels are not rebuilt "each frame", and three records say they are

**Where:** `gui/unlock_platelist.go:71-79` and `:108`; plan §7c; commit `7c71c7c`'s
mutation row 7.6.

**Defect.** Plan §7c mandates *"builds its labels with `unlockPlateLabel(…)` **each
frame**, so the '(cut)' mark appears as soon as a plate completes."* The shipped
code builds them once at entry and once after each `unlockEngraveFlow` returns.
The in-code comment nevertheless reads *"Labels are rebuilt EACH FRAME rather than
once up front"*, and the Task 7 commit records the mutant as *"7.6 labels built
ONCE instead of each frame … only the each-frame relabel makes visible."*

**Evidence.** `relabel()` appears exactly twice: `gui/unlock_platelist.go:79`
(entry) and `:108` (after the engrave). The frame loop body between `:116` and
`:175` never calls it.

**Consequence.** No behavioural difference — `cut` is the only mutable input and it
can only change at `:106` — so this is a record defect, not a bug. But it is an
undeclared deviation from a plan clause, and it is the third place in this diff
where a comment describes code that was written differently (cf. M3 in the lens-1
chain). A future reader adding a second mutable label input will trust the comment.

**Fix.** Reword the comment to "rebuilt after every engrave", correct the plan's
§7c bullet, and re-word the 7.6 mutation row — the mutant it actually kills is
"the post-engrave `relabel()` deleted".

---

### M3 — Minor — the mutation runner is uncommitted and the report it points at does not exist

**Where:** commit `3db3bfe`'s message; `design/agent-reports/`.

**Defect.** `STANDARD_WORKFLOW`/`CLAUDE.md`: *"When an artifact will be folded
repeatedly, commit the extractor as a script so the check is a command, not a thing
to remember."* Task 8's commit says *"The runner is NOT committed here … It is a
single self-contained Python file and is reproduced in the phase report."*

**Evidence.** `ls design/agent-reports/ | grep b2a-ii` returns only the four lens-1
files; there is no B2a-ii phase report. `ls scripts/` in `mnemonic-engrave` shows
`plan-build-gate.sh`, `plan-build-gate-go.sh`, `plan-cite-gate.sh` and no mutation
runner. The only other mention in the repo is
`design/agent-reports/MUTATION_planB_phaseA.md:15`, which says the Phase A runner
also lived in a scratchpad.

**Consequence.** The 30-mutant run is, right now, reproducible by nobody. B2b owns
§10.2.4 plus the F-89 unwind — the phase most in need of re-running exactly these
rows — and will re-derive the runner, with a different notion of "the substitution
matched" than the one this phase had to fix twice mid-run (rows 6.1/6.5, 6.7).

**Fix.** Commit it as `mnemonic-engrave/scripts/mutation-run.py` with the row table
as data, and have it print what it does not cover — the same shape as
`plan-build-gate-go.sh`. If the phase report is still owed, land the runner with it.

---

### N1 — Nit — `unlockSecretLabel`'s godoc carries the superseded paragraph above its replacement

**Where:** `gui/unlock_session.go:49-60`.

**Defect.** The pass-3 fold prepended a new doc paragraph without removing the old
one, so the function's godoc now reads:

```
// unlockSecretLabel names a secret plate by its CLASSIFIED type and its index
// among secrets -- never by anything the sealer asserted, and never by
// rendering the record's contents.
// unlockSecretLabel names a secret plate by its CLASSIFIED type and its index
// WITHIN THAT CLASS -- never across classes, and never by anything the sealer
// asserted.
```

**Consequence.** The first sentence states the exact behaviour the fold removed as
a defect ("index among secrets"), and it is the sentence `go doc` shows first. Per
`CLAUDE.md`'s "records are the weak half", a stale record beside a fixed behaviour
is how the defect comes back.

**Fix.** Delete lines 49-51; keep "never by rendering the record's contents" by
folding it into the surviving paragraph.

Two smaller stale records in the same class, from Task 7:
`gui/unlock_platelist.go:67` *"lists every public record"* (it now lists the
encrypted section's cards too) and `:21` *"B1 holds only public records, so there
is nothing to wipe on the way out"* (B2a-ii wipes on the way out).

---

### N2 — Nit — two of the plan's three gui file line-counts are wrong, and the Rust-primary check for the `bip39` fold is not recorded

**Where:** plan line 1696; commit `d0baf13`.

**Defect (a).** The plan's "Beyond the gate" section claims *"`gui/unlock_kdf.go`
(247 lines), `gui/unlock_session.go` (184) and `gui/unlock_plates.go` (83)"*.
Measured with `wc -l` on the plan's own blocks: **291, 237, 83**. A hand-count where
a tool was available, in a GREEN plan, which is the one thing `CLAUDE.md` names by
name. Harmless downstream — the shipped files are byte-identical to the blocks the
gate actually compiled — but it is the record class this project keeps getting
burned by.

**Defect (b).** `CLAUDE.md`'s Rust-primary rule: *"whenever a defect is found in a
Go port we **MUST always** check whether the same defect exists in the primary Rust
implementation."* `bip39` is in the ported set, and `d0baf13` fixes a defect found
in it (`Parse`'s `append` orphaning partial seeds). The fix is correctly Go-only —
it is memory hygiene, not normative behaviour, so nothing needs to land in Rust
first — but the commit does not record that the check was made. The rule says the
check is never skipped, not that it always finds something.

**Fix.** Correct the plan's counts with `wc -l` output (or drop the numbers), and
add one line to the phase report recording the Rust check for `Parse`.

---

## What I checked and found sound

- Every whole-file Go block in the plan is byte-identical to its shipped file
  except `gui/unlock_session.go`, whose every delta is a declared fold.
- The §5d fragment at the branch tip is byte-for-byte the plan's, and every one of
  the nine commits independently type-checks and passes `./gui ./seal ./bip39`.
- Six mutation rows re-run independently; all six killed by their named killer,
  each with the diagnostic it was written to produce.
- `clear(m)` cannot corrupt the mnemonic plate: `engraveSeed` copies to
  `[]string` from the immutable wordlist and `toPlate` materialises the spline
  before the wipe. Hoisting `clear(m)` above `engraveSeed` is killed by the suite
  (I ran it — `TestSecretSessionEngravesAMnemonic` fails), so the ordering that
  would silently engrave 24 × "abandon" is pinned.
- The `gui/gui.go` key-scrubbing folds are safe against the vendored
  `hdkeychain`: `NewMaster` does not retain the seed, `ECPubKey` does not alias
  `k.pubKey`, and the fingerprint is computed before any `defer`.
- Nav budgets: no `layoutNavigation` call in this diff exceeds three buttons.
- `d.Done()*100/d.Total()` cannot divide by zero on any reachable path.
- Section placement, AAD extent, the cross-section record cap, the "too many
  records" message, the `ct_len == 0` no-prompt rule, and the plate list's
  exclusion of every `seal.IsSecret` record are all implemented as §6/§10 specify.
- No secret can reach the plate list, and the plate list cannot be constructed on
  a cancelled or failed unlock.

## Out of scope, as briefed

Not re-reported: the wipe lens's C1/I1/M1/D1/D2 and pass-3 findings; F-83, F-86,
F-87, F-88, F-89; the surviving `clear(blob)` mutant; the absence of §10.2.4's idle
timer.
