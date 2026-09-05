# Hashlock H2 — independent adversarial post-implementation execution review

**Subject:** fork `hashlock-h2` @ `17b3979` (base `main` `c4a64fc`), engrave records
`hashlock-h2` @ `2fc2051`. Spec `SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`), plan
`IMPLEMENTATION_PLAN_hashlock_H2_device.md` (R0 GREEN `1cb05b8`), implementer's report
`design/agent-reports/hashlock-H2-implementation-report.md`.

**Method.** Read-only on both branches; every build, run and mutation in my OWN detached
worktree `/scratch/code/shibboleth/.tmp/h2-review` at `17b3979` (plus three throwaway
worktrees at `c4a64fc`, `f283e3a` and `e1bf137` for baselines). Go 1.26.7 from
`/scratch/code/shibboleth/.toolchain/go`. Every mutation reverted; `git status --porcelain`
empty at the tip after the last one. No sub-agents, no `.jsonl` read, nothing committed, no
phrase or preimage byte in anything kept. All four worktrees removed at the end.

**VERDICT: NOT GREEN — 0 Critical / 2 Important / 3 Minor / 2 Nit.**

Neither Important is a digest divergence. **I could not construct one.** Twenty screen
derivations across ten corpus rows and both methods landed the host's constant exactly, the
phrase rule matches the host byte for byte in order and in counting, no Back edge loses a
path or a phrase, and no hash is assigned before HOLD. Both Importants are *gates*: a
spec-normative operator-facing value that no test can fail on, and a control this stage drew
that does nothing.

---

## The one question

> Can you construct typed bytes, a method, a Back sequence or a composer state for which the
> device's digest differs from the host's for the same intent, the operator's work is lost
> where the spec says intact, a hash is assigned before HOLD, or a test reports PASS on a
> defect it names — and does every claim in the implementation report hold at the tip?

**Digest divergence: no.** Constructed and refuted below (lockstep table, §2 trace).
**Work lost: no.** Constructed and refuted (Back-edge table, including an existing path
carrying a prior hash).
**Assigned before HOLD: no.** Mutation M1 proves the guard is real and named.
**A test reporting PASS on a defect it names: no** — but **two values a test never names**
survive mutation of the whole 1222-test `gui` suite (I-1).
**Report claims: every measurable one holds.** All fifteen I re-measured are exact,
including three firmware figures to the byte.

---

## Findings

### I-1 — the confirm modal's `chars: <n>` and `first8..last8` survive mutation of the entire `gui` suite

Spec §4.5 makes both normative and states what each is *for*:

> `chars: <n>` is the phrase's byte count — **the one signal that shows a stray space when
> the operator later reconciles against the host card's `phrase_chars`** (journey M-5).

> The 64 visible bits (`first8..last8`) are a transcription check…

§8 (acceptance) rests on the second: *"the `first8..last8` shown equals `ms hashlock`'s on
the host for the same phrase and method"*. Both are produced by fresh H2 code —
`hashlockFirst8Last8` (`gui/composer_hashlock.go:131-134`) and the `len(phrase)` argument at
`:65` — and **neither is asserted by any Go test.** `hashlockFirst8Last8` has exactly one
caller, and it is production:

```
$ grep -rn 'hashlockFirst8Last8' --include=*.go .
gui/composer_hashlock.go:65:  body := composerCopyHashlockConfirm(hashlockFirst8Last8(h), m.String(), len(phrase),
gui/composer_hashlock.go:131: func hashlockFirst8Last8(h [32]byte) string {
```

The copy-table row and the modal-fit row both pass the token in as a **literal**
(`composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100, …)`,
`gui/composer_copy_test.go:130`, `gui/modal_fits_test.go:388`), so they exercise the body
*builder* and never the derivation of its first argument.

**Constructed counterexample, and it is a survivor.** Two mutations at once —
`return s[:8] + ".." + s[:8]` (the digest's tail half replaced by its head) and
`len(phrase)+1` (the character count off by one):

```
$ scripts/gui-shard-test.sh ./gui/ 24     # UNMUTATED
    1222 top-level tests
    partition verified exhaustive: 1222 == 1222
RESULT: ok -- all 1222 tests ran across 24 shards

$ scripts/gui-shard-test.sh ./gui/ 24     # BOTH MUTATIONS APPLIED
    1222 top-level tests
    partition verified exhaustive: 1222 == 1222
RESULT: ok -- all 1222 tests ran across 24 shards
```

Under the mutation the operator's screen reads `hash b867db87..b867db87 method: sha256
chars: 29` for a 28-character phrase whose true digest ends `edbc96cb` — and CI is green.

**What partially covers it, and why that is not enough.** `cmd/emu/walk_hashlock_phrase.js`
*does* assert the abbreviated token (`ANCHOR_SHA_H = "b867db87..edbc96cb"`, walk lines
75-79, 290/304/313) and would catch the `first8..last8` half. It does **not** assert
`chars:` at all — the walk merely records it in `out.typed`. And the walk is not in CI:
`.github/workflows/test.yml` only does `GOOS=js GOARCH=wasm go vet ./cmd/emu/` and
`cmd/emu/build.sh`; nothing runs playwright. So `first8..last8` is gated by a manually-run
walk, and `chars: <n>` is gated by **nothing at all** — this repo's "a gate that has never
executed is a hypothesis" applied one level out.

**Why Important and not Critical.** No wrong digest can ship: the assigned value is correct
on 20/20 screen derivations (table below) and the CI lockstep pins the derivation. A wrong
`chars` or `last8` yields a **false mismatch** at reconciliation — the operator goes back to
the host and finds the digests agree. That is the safe direction. But it is a normative
screen value on the stage whose entire purpose is host/device reconciliation, with no
executable gate.

**Remedy, one edit, verified.** Add two assertions to the existing
`TestHashlockPhraseRouteSetsTheCorpusDigest` (`gui/composer_hashlock_test.go:331`), which
already holds the confirm frame:

```go
body := h.mustReach("Write down this phrase")
want := "hash " + tc.want[:8] + ".." + tc.want[56:]
if !strings.Contains(normalizeDrawn(body), normalizeDrawn(want)) { t.Errorf(...) }
if !strings.Contains(normalizeDrawn(body), normalizeDrawn(fmt.Sprintf("chars: %d", len(tc.phrase)))) { t.Errorf(...) }
```

I ran exactly this (as a scratch test) against both mutations; it fails on both, e.g.
`row 0: the confirm modal drew "hashb867db87..b867db87method:sha256chars:28…", want "hash
b867db87..edbc96cb"`. Spec §7.2's screen-test list should gain the same line, since the gap
is in the spec as much as in the code.

---

### I-2 — F-481 GATES: the `show` key is drawn, tappable, flips to `hide`, and reveals nothing

**Graded Important, as a claims defect — not deferred to H3.**

**Measurement, reproduced independently on the touch harness** (not only on the emulator,
where the implementer found it). Ten characters typed on the `Hashlock phrase` screen:

```
Fragment="abcdefghij" MaxHeight=201
masked frame  = "qwertyuiopasdfghjklzxcvbnmABCspaceshowThisscreendoesthathashingforyou.
                 Useaphraseyouhaveneverusedanywhereelse.10/100Hashlockphrase"
NO MASKED READOUT DRAWN                       (0 asterisks in the frame)
revealed=true
revealed frame = "…ABCspacehideThisscreendoesthathashingforyou.…10/100Hashlockphrase"
REVEAL DRAWS NOTHING; label flipped to hide -- the key is live
```

**The band budget, measured, so the deficit is a number and not a description:**

```
display = (480,320), keyboard grid page0 = (340,182)
leadingSize=44  leadBand=44  counterBand=23  ->  kbd.MaxHeight=201
readout budget = MaxHeight - grid.Y - readoutGap = 201 - 182 - 8 = 11 px
one readout line needs 19 px          DEFICIT = 8 px
```

`PassphraseKeyboard.Layout` (`gui/passphrase_keyboard.go:454-473`) then binary-searches
leading runes off the readout until it fits 11 px, and drops every one.

**Why it is not the secret-handling class the spec waived.** Spec §4.2 says *"The keyboard's
reveal (`show`) key is inherited as-is: secret-handling, non-gating (fidelity N-2)"* — a
warrant written for a **working** reveal key, whose only cost is that a secret can be shown.
What shipped is a different object: a control that is drawn, hit-tests, changes its own
label, and does nothing. That is the class this repo has already ruled on, in its own words,
in the file this screen's keyboard comes from (`gui/passphrase_keyboard.go:96-113`,
`NewLineKeyboard`):

> the gear was drawn there, was tappable, and did NOTHING AT ALL — **a live-looking control
> that swallows the press, on the machine where the next thing the operator approves is cut
> into steel**. … Removing it, rather than wiring it up, is deliberate and NOT a
> placeholder … **drawing a dead key is the one answer that is wrong under every outcome.**

The fork removed a key for exactly this and wrote down why; this stage added the same shape
back, on a new screen, for a secret the operator must transcribe by hand and can never
recover (§4.5: *"Without both, this path can never be spent"*). It also removes the *masked*
readout, so the operator has no on-screen evidence of anything they typed beyond `n/100`.
Under the standing severity rule, "defects in what a tool *claims* to have done" still
block, and a key labelled `show` that shows nothing is that.

**One-line remedy, applied and verified.** Delete
`content, _ = content.CutBottom(8)` (`gui/composer_hashlock.go:158`) — it reclaims exactly
the 8 px deficit:

```
masked frame  = "…ABCspaceshow**********Thisscreendoesthathashingforyou.…10/100Hashlockphrase"  (asterisks: 10)
revealed frame = "…ABCspacehideabcdefghijThisscreendoesthathashingforyou.…10/100Hashlockphrase"
REVEAL NOW WORKS
$ go test -run 'TestHashlock|TestPassphraseKeyboard|TestWhichHash' ./gui/
ok  seedhammer.com/gui  33.005s
```

Caveat, in this repo's own words (*text extraction cannot see clipping*): I verified the
readout draws in **extracted text**. The 8 px was slack below a bottom-aligned grid, so the
geometry should be confirmed on a shot or with a raster assertion before it is called done —
and whichever fix is taken, the fixed screen needs a test that fails without it, or I-1's
shape recurs here.

---

### M-1 — two §4 copy departures ship with no follow-up, while two others have one

The stage filed F-478 (§4.5's drop-order destination) and F-479 (§4.5's other-path line) for
copy that departs from the spec. Two more departures are unrecorded anywhere:

| spec | spec says | code does |
| --- | --- | --- |
| §4.1 | *"the screen's lead reads: No hash record in the payload…"* with *"(**second lead line** only when `len(digests) == 0`)"* | `composerHashRows` **replaces** `"Which hash?"` outright (`gui/composer_hash.go:169-171`), so with no payload record the screen never asks its question. This is what moved `composer_gates_test.go`'s pump target. |
| §4.2 | lead is *"Use a phrase you have never used anywhere else."* | `composerCopyHashlockPhraseLead` prepends *"This screen does that hashing for you."* (`gui/composer_copy.go:369`) |

Both were decided in the plan (plan lines 1085, 1187; build-gate fix 9) and are defensible —
the §4.2 addition answers the §8i modal the operator has just dismissed. Neither is a defect
in behaviour. The gap is bookkeeping: `TestComposerCopyIsVerbatimFromTheSpec` compares the
code against a table the implementer transcribed, so it cannot see a spec departure, and
these two therefore have no record at all. **Remedy:** one F-482 entry, owning phase H3,
naming both with their replacement text, exactly as F-478/F-479 do.

### M-2 — `DeriveHardened` and `PreimageHardened` fail OPEN on a dead `Deriver`

`hashlock.go:47-50, 58-65` end with `copy(x[:], d.Key())` and no nil check. `seal.Deriver`
returns `nil` from `Key()` when dead or incomplete, and its own comment
(`seal/pbkdf2.go:109-112`) states the contract this violates:

> After Wipe it is a TERMINATING no-op: it reports complete so a caller's `for !d.Step(n)`
> loop cannot spin forever … and Key() then returns nil, **so the caller fails closed rather
> than proceeding with a plausible-looking wrong key.**

Here a dead Deriver yields `Step→true`, `Key→nil`, `x` all-zero, `ok=true` — an all-zero
preimage, and `Digest` of it assigned as the path's hash. **Unreachable at this tip** (the
Deriver is function-local and `Wipe` is deferred), so it is not a defect today; it is a
latent contradiction of the contract, one line to close:
`if k := d.Key(); k == nil { return x, false }`. Owning phase H3.

### M-3 — secret handling (non-gating by the operator's 2026-08-27 ruling)

- The phrase lives in `kbd.Fragment`, an **immutable Go string** that no code can zero, for
  the life of the phrase screen and across every Back into it via `initial`
  (`gui/composer_hashlock.go:139-140`); `[]byte(kbd.Fragment)` makes a second copy per OK.
- `seal.isPreimageRecord` (`seal/record.go:296-299`) does `codex32.New(string(r))` on a
  record `Classify` has already stringified, adding one more unwipeable copy of possibly-seed
  material on the refusal path.

Consistent with ruling L15 ("no scrub discipline beyond what the composer does by
construction") and with `Classify`'s own documented compromise. Logged, per the ruling, not
gating.

### N-1 — `composerCopyHashlockRefusal`'s default arm can print a Go error to an operator

`gui/composer_copy.go:376-392` switches on the five sentinels and falls through to
`return err.Error()` — `"hashlock: the phrase is longer than 100 characters"`. Unreachable
today (`ValidatePhrase` returns only those five); a sixth sentinel would ship device copy
with a package prefix. A `default:` that returns the §2 rule sentence, or a test asserting
all five arms, closes it.

### N-2 — `hashlock.IsMS1Shaped` folds case with Unicode `strings.ToLower`; the host uses `to_ascii_lowercase`

`hashlock.go:114` vs ms-cli `argv_guard.rs:149` (`raw.trim().to_ascii_lowercase()`). For
non-ASCII input the two differ (`İ` U+0130 lowercases to two runes in Go, unchanged in Rust),
which would move `len(t)` past `minMS1Len`. **Unreachable on the phrase path** — rule 2
refuses every non-ASCII byte before rule 3 runs, and `ValidatePhrase` is the function's only
production caller — but `IsMS1Shaped` is exported, so a future caller inherits a divergence
from the host. A doc line, or `strings.Map` over ASCII only.

---

## The lockstep table (item 1)

Host column = **recomputed from scratch in Python** (`hashlib.pbkdf2_hmac('sha256', phrase,
b"ms-hashlock-v1", 100000, 32)`, `sha256`), not read from the corpus — a third
implementation, so a uniformly-wrong corpus could not agree with it. All 11 rows matched the
vendored constants (`bad: 0 of 11`). Package column = `go test ./hashlock/` (9 tests, ok,
0.229s). Screen column = **every typeable corpus row driven through `composerAddPath` on the
touch harness**, both methods: keyboard taps, method pick, warning modal, HOLD, then the
path's `Hash` read back in full 64 hex.

| # | phrase (chars) | sha256 pkg | sha256 SCREEN | sha256 host(py) | hardened pkg | hardened SCREEN | hardened host(py) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | `correct horse battery staple` (28) | ok | `b867db87..edbc96cb` | `b867db87..edbc96cb` | ok | `3cf5d421..b70a4c12` | `3cf5d421..b70a4c12` |
| 1 | `z` (1) | ok | `c27cd49c..1ff723ca` | same | ok | `af384a82..a5453b3a` | same |
| 2 | `twenty characters!!!` (20) | ok | `5b891cd8..3b1bdde4` | same | ok | `f00137a8..d39eb4a2` | same |
| 3 | 64 printable, not hex (64) | ok | `895d7861..126ec335` | same | ok | `bd10cd48..e57b58fd` | same |
| 4 | 65 printable (65) | ok | `671c8142..6ac64ff1` | same | ok | `4a84ddc8..37bf6984` | same |
| 5 | 100 printable (100) | ok | `76001f8e..7f31cde4` | same | ok | `70a53953..e68b4525` | same |
| 6 | 101 printable (101) | ok (refused) | **refused** `A hashlock phrase is at most 100 characters.` | refused | — | — | — |
| 7 | `  a  b ` (7) | ok | `5f74bd9f..b294fe69` | same | ok | `07ca621d..d65941f6` | same |
| 8 | `correct-horse,battery staple` (28) | ok | `c0ed353a..ebfdf468` | same | ok | `528a12a1..1af350df` | same |
| 9 | `a-b,c` (5) | ok | `082f6172..89a9fe16` | same | ok | `8680bbf9..c1841c37` | same |
| 10 | `Correct Horse Battery Staple` (28) | ok | `95d44470..2297a7ff` | same | ok | `36d5ad9d..28cacea2` | same |

**20 screen derivations, 20 exact matches** (`TestRV_ScreenLockstepEveryRow`, 56.5 s). The
three non-fixed-point rows (7, 8, 10) all land their own digest, so no screen-layer case
fold, trim or separator strip exists. I separately confirmed the **drawn** token on rows
0, 1, 2, 8, 10: the confirm modal carries `hash <first8>..<last8>` and `chars: <n>` matching
the corpus in every one.

**Refusal rows, typed on the harness, phrase intact afterwards** (counter unchanged, path
count 1 after every dismissal):

| corpus row | typed | screen says |
| --- | --- | --- |
| 0 empty | *(nothing)* | `Type a hashlock phrase, or press Back.` |
| 5 ` ~` accepted | ` ~` | reaches `Which method?` |
| 6/7 64-hex, both cases | lower- and UPPER-case 64 hex | `That is a preimage in hex, not a phrase. Use the Type 64 hex row.` |
| 8 `beef` accepted | `beef` | reaches `Which method?` |
| 9/10 plate lower/UPPER | the `kind[0].ms1` plate | `That is a preimage plate, not a phrase. On the host, run ms hashlock with it and load the hash: record it prints.` |
| 11 grouped by 5 | plate grouped | same refusal |
| 12 leading/trailing spaces | `  plate  ` | same refusal |
| 13 grouped by 2 (110 chars) | plate grouped by 2 | **same refusal** — the shape test precedes the cap |
| 14 too long | 101 × `k` | counter reads `101/100`, `A hashlock phrase is at most 100 characters.` |

Rows 1-4 (`café`, `0xff`, TAB, DEL) are unreachable from a keyboard that emits only
`0x20..0x7E`; the package test drives them.

## Normalisation trace (item 2)

`kbd.Fragment` (a `string`, appended one rune at a time by `commit`, `NO ToUpper — case
preserved`, `passphrase_keyboard.go:266`) → `phrase := []byte(kbd.Fragment)`
(`composer_hashlock.go:149`) → `hashlock.ValidatePhrase(phrase)` → unchanged bytes to
`hashlock.PreimageSHA256(phrase)` / `hashlock.DeriveHardened(phrase, …)`.

No `TrimSpace`, `ToLower`, `Fields`, `Join`, `NormalisePassphrase`, `unicode` call or
byte-altering copy touches the phrase on this path. `IsMS1Shaped` folds a **copy**
(`string(phrase)`) and returns a bool. Grepped: `seal.NormalisePassphrase` has no caller in
`gui/composer_hashlock.go` or `hashlock/`.

**Rule order** — Go `ValidatePhrase` (`hashlock.go:80-102`) vs host `validate_phrase`
(ms-cli `hashlock_phrase.rs:118-142` @ `cd0a60f`): empty → printable-ASCII → **ms1-shape →
cap** → 64-hex, identical, and mutation M5 (swap shape and cap) reds corpus refusal row 13
by name.

**Counting.** Cap: Go `len(phrase)` (bytes); host `s.len()` (bytes) — and both run after the
printable-ASCII rule, so bytes ≡ characters on both sides. Counter: `len(kbd.Fragment)`
bytes, denominator `hashlock.PhraseMaxChars` (`composer_hashlock.go:161-162`), unclamped —
`101/100` measured on screen. `PhraseMaxChars = 100` = ms-cli `HASHLOCK_PHRASE_MAX_CHARS`.

**Shape predicate.** Go trim→lower→strip{space,\t,\n,\r,-,,}→`len ≥ 48`→`ms1` prefix→bech32.
Host: `raw.trim().to_ascii_lowercase()` → `strip_display_separators` (all Unicode whitespace
+ `-` + `,`) → `t.len() >= 48` → `starts_with("ms1")` → charset. Equivalent on ASCII (see
N-2 for the one non-ASCII corner, unreachable here). No checksum on either side. ✔

## The Back-edge table (item 3)

Driven twice: through **`composerAddPath`** (creation) and through **`composerHashEdit`
alone at an existing path already carrying a hash** (`TestRV_ExistingPathHashSurvivesEveryBack`,
mine — the case no shipped test covers).

| edge | spec §4.6 says | device does | path | phrase | prior hash |
| --- | --- | --- | --- | --- | --- |
| Back at the confirm modal | → method pick, phrase intact, nothing assigned | `Which method?` | kept | intact | unchanged |
| Back on a declined method modal | → method pick, phrase intact | `Which method?` | kept | intact | unchanged |
| Back at the method pick | → phrase screen via `initial` | `Hashlock phrase`, counter `28/100` | kept | intact | unchanged |
| Back mid-derivation (countdown) | → method pick, nothing assigned | `Which method?` | kept | intact | unchanged |
| Back at the phrase screen | → `Which hash?`, phrase dropped | `Type a hashlock phrase` row list | kept | dropped | unchanged |
| Back at `Which hash?` (creation) | the ONLY `false`; creation deletes the path | returns false; `len(Paths) == 0` | deleted, as specified | — | — |
| Back at `Which hash?` (existing path) | `false` only | returns false; caller at `composer_shape.go:346` ignores it | kept | — | **unchanged** |
| Back from hex entry | route-internal | `continue` → `Which hash?`, path intact | kept | — | unchanged |

`composerHashEdit` returns `false` at exactly one `return` site (`composer_hash.go:206`),
annotated as such. **W-7 seat-discard interplay:** `composerPathEdit`'s hash arm is still
`composerEditCanRenumber(…) && !composerShapeGuard(…)`, unchanged by this diff; the
`composer_gates_test.go` needle move (`"Which hash?"` → `"Path 1 hash"`, forced by M-1's
§4.1 lead swap) does **not** weaken it — mutation M9 below proves the test still fails when
the guard is put back on the hash arm.

## HOLD and assignment (item 4)

`st.list.Paths[idx].Hash` is written at one place only, inside
`if composerConfirmScreen(…)` (`composer_hashlock.go:66-69`). Mutation M1 hoists it above
the `if`:

```
--- FAIL: TestHashlockBackContractKeepsThePath (2.03s)
    composer_hashlock_test.go:442: hash assigned before HOLD
```

`hashByPhrase` lifecycle, measured (`TestRV_HashByPhraseLifecycle`): set on assignment;
`No hash lock` on the last hashed path clears it via `composerHashByPhraseSync`; `No hash
lock` on one of two hashed paths does **not**; a path *removed* does not resync, and a
**payload-row or hex edit over a phrase-set hash** does not either. All of those leave the
flag over-sticky, which makes §4.7 name one artifact too many, never one too few — the
documented, safe direction (`composer_hash.go:180-198`), with per-path provenance filed as
F-480 (H3). No path makes it stale-FALSE while a phrase-set hash is live.

## The relation line (item 5)

`hashlockRelationLine(payload, h)` is called with `rows.digests` — `composerPayloadDigests(ctx.sysw)`,
the **payload's** `hash:` records, never `st.list` (`composer_hashlock.go:65`, `:91-104`).
`match := -1`, first equal record wins, rendered `matches hash %d`, `i+1` (1-based). The
shipped test drives the *second* record matching and the neither-matching case, plus the
no-records arm as a unit — the three cases needed to kill `match := 0`, `%d` on `i`, and
`if true`. ✔ Index base and comparand both correct.

## Deriving (item 6)

- **Iteration accounting is exactly 100,000.** `NewDeriver` computes U_1 and sets `done = 1`
  (`seal/pbkdf2.go:96-102`); `Step` runs while `done < total` — 1 + 99,999. Proven by the
  Python agreement above, which is the only check that can see an off-by-one here.
- **`Wipe` on every exit path** — `defer d.Wipe()` in both `PreimageHardened` and
  `DeriveHardened`, so Back mid-derivation (progress → false) wipes too. (See M-2 for the
  contract corner this leaves open.)
- **The progress callback's false return** stops promptly and returns the zero value —
  `TestDeriveHardenedAbandonsWhenProgressSaysStop` pins `calls == 3` (not 199), `ok == false`,
  `x == [32]byte{}`, and is the only test that can see it.
- **Countdown copy** — zero-state `"Deriving. This takes about 10 seconds."` hoisted into a
  frame drawn *before* the first `Step`, then `About N seconds left.` §4.4 ✔
- **No UI starvation under `-scheduler tasks`.** `ctx.KeepAwake()` + `ctx.WakeupAt(time.Now())`
  precede `ctx.Frame` (order load-bearing, F-93's fix). Mutation M6 removes `KeepAwake`:
  `Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?).
  180 frames drawn, last = "89%About21secondsleft.Deriving"` — the r0 Critical, reproduced
  verbatim. Back is sampled every 500 iterations ≈ 51 ms at the measured 9,715 it/s.

## Mutation table (item 7)

Each applied in my own worktree, measured once, reverted; tree clean afterwards.

| # | mutation | scope run | result |
| --- | --- | --- | --- |
| M1 | assign `Hash` **before** `composerConfirmScreen` | `TestHashlockBackContract…`, `…SetsTheCorpusDigest` | **FAIL** `hash assigned before HOLD` ✔ |
| M2 | `hashlockFirst8Last8` → `s[:8]+".."+s[:8]` | the six H2 test prefixes | **PASS — survivor** (I-1) |
| M3 | `chars` → `len(phrase)+1` | whole `gui` suite, 24 shards | **PASS — survivor**, 1222/1222 ok (I-1) |
| M4 | M2 + M3 together | whole `gui` suite, 24 shards | **PASS — survivor**, 1222/1222 ok (I-1) |
| M5 | cap check **before** the ms1 shape test | `./hashlock/` | **FAIL** `row 13 rule ms1-shaped: got …longer than 100 characters want …not a preimage plate` ✔ |
| M6 | delete `ctx.KeepAwake()` from `Deriving` | `TestHashlockDeriveKeepsAwake…` | **FAIL** `Run exceeded 100000 ticks … parked (screensaver?)` ✔ |
| M7 | delete the `errors.As` arm from `unlockSealedFlow` | `TestUnlockNames…`, `TestUnlockNotPermittedBody…` | **FAIL** `never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"` ✔ (Task 7's RED, reproduced) |
| M8 | drop `RecordNotPermittedError.Unwrap` | `./seal/` | **FAIL ×4**, incl. `the typed error no longer matches ErrRecordNotPermitted -- every existing caller is broken` ✔ |
| M9 | put `composerShapeGuard` back on `composerPathEdit`'s hash arm | `TestComposerLockAndHashEdits…` | **FAIL** `the hash lock editor was never reached. Last frame: "EDITINGTHESHAPECLEARSTHEKEYS…"` ✔ — the pre-existing test's needle move is sound |
| M10 | drop `content.CutBottom(8)` (the F-481 **remedy**, not a defect) | H2 + keyboard + which-hash | readout and reveal both work; `ok seedhammer.com/gui 33.005s` |

**Does the lockstep test recompute its own expectations?** No —
`TestDerivationRowsLockstep` compares against `mustHex(t, r.HardenedX)` etc., the JSON
**constants**, and additionally asserts `DeriveHardened == PreimageHardened` so a mutation of
one and not the other is visible. It compares **full 32-byte values**, never prefixes.
`loadCorpus` re-hashes the file against the pinned literal on every call, so corpus drift on
either side reds the suite. ✔

**`holdConfirm`'s explicit release** (`composer_hashlock_test.go:145-190`) is real and
load-bearing: `EventRouter` tracks one global pointer contact and reuses a stale
`pressedTag` while pressed, so without the release the second hold in a sequence routes to a
dead `Clickable`. The comment records the measurement. ✔

**Do the modal-fit rows use the production builder?** Yes — `modal_fits_test.go:386-390`
calls `composerConfirmBody(composerCopyHashlockConfirm(…))`, the same two functions
production calls, through `confirmWarningBody` (the renderer that actually draws it), with
the longest variant (relation line + other-path line + `chars: 100`). ✔

**Was the §4.5 copy drop actually forced?** Yes, and I re-measured all three states rather
than trusting the plan (`TestRV_ConfirmBodyFitMeasurements`):

```
SPEC 4.5 unshortened (reuse block whole + reconciliation line)  drawnInFull=false drew=484/504 headroom=0
drop-order step 1 only (reuse shortened, reconciliation kept)   drawnInFull=true  drew=384/384 headroom=64
SHIPPED longest variant                                         drawnInFull=true  drew=337/337 headroom=107   (margin 80)
```

Identical to the plan's table to the character. Step 1 alone leaves 64 < the 80-character
margin, so step 2 was required; the shipped body clears it with 107. The reconciliation
line's relocation to its own post-HOLD `showError` (rather than §4.5's named §8h
destination, which `composerEveryPathHashed` makes unreachable on a mixed policy) is correct
and is recorded as F-478.

## Records (item 8)

| claim | measured |
| --- | --- |
| corpus sha256 == ms `cd0a60f`'s file | `git show cd0a60f:crates/ms-codec/tests/vectors/hashlock-v0.8.json \| sha256sum` = `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30` = the vendored file = the provenance pin ✔ |
| provenance pin shape | repo, remote, path, `"commit": "cd0a60f"`, sha256, `derivation_rows: 11`, `refusals_rows: 15`, `recorded_at` ✔ |
| fork CHANGELOG | none, on any branch: `find -iname 'CHANGELOG*'` and `git log --all --diff-filter=A -- 'CHANGELOG*'` both empty ✔ — the line is under a `CHANGELOG` heading in `17b3979`'s message ✔ |
| FOLLOWUPS continue from F-476 | F-477…F-481 present, each with an owning phase of **H3** ✔ |
| no follow-up still owned by H2 | `grep -c 'owning phase: \*\*H2\*\*' design/FOLLOWUPS.md` → **0** ✔ |
| firmware, baseline `c4a64fc` | `1551336 31796 31004 \| 1583132 62800` — the report's baseline, to the byte ✔ |
| firmware, `e1bf137` | `1564424 31852 31004 \| 1596276 62856` — the report's number, to the byte ✔ (**+13,144 flash / +56 RAM**) |
| firmware, tip `17b3979` | `1565424 31852 31004 \| **1597276** 62856` — **not claimed by the report**, measured here: Task 7 costs **+1,000 B flash, +0 B RAM**; total over `c4a64fc` **+14,144 B (+0.893 %) / +56 B** |

## Report claims (item 9)

Every measurable claim re-derived at the tips. **All hold.**

| report says | I measured |
| --- | --- |
| `gui` 1205 at `c4a64fc` | 1205 ✔ |
| `gui` 1206 at `f283e3a` (Task 3) | 1206 ✔ |
| `gui` 1220 at `e1bf137` (Task 4) | 1220 ✔ |
| `gui` 1222 at `17b3979`, partition exhaustive, all shards ok | 1222 == 1222, `RESULT: ok -- all 1222 tests ran across 24 shards` ✔ |
| `hashlock` 9 tests ok | 9 named tests, `ok … 0.229s` ✔ |
| `./hashlock/ ./codex32/ ./seal/ ./sysw/` ok | ok / ok / ok / ok ✔ |
| vet clean but for three go1.25 `ArtifactDir` notes | exactly those three ✔ |
| `gofmt -l` five pre-existing files | exactly those five ✔ |
| corpus shape `{kind:1, derivation:11, refusals:15, lengths_by_door:7, lockstep:4}` | identical ✔ |
| the walk's control `correct horse battery stapl` → `c8043156..253e7389` | `sha256(sha256(b"correct horse battery stapl"))` = `c8043156…253e7389` ✔ (independent Python) |
| every walk digest | all four are corpus constants I re-derived in Python ✔ |
| Task 7 RED `"Payloadunreadable.SealedPayload"` | reproduced (M7) ✔ |
| Task 7 `Unwrap` mutation reds three seal tests | reds **four** (three in `record_not_permitted_test.go` plus H0's `TestAdmitSectionRefusesAPreimagePlateAsUnknown`); the report's "all three seal tests" refers to its own new file — not a false claim, but the fourth is worth knowing ✔ |
| F-481's mechanism (`kbd.MaxHeight` leaves under one line) | confirmed, with the number: 11 px available, 19 px needed ✔ |
| firmware +13,144 B / +56 B at `e1bf137` | exact ✔ |

**No false count found.** The report's own reconciliation notes (the plan's "14" vs 15 new
`gui` tests; 1205 + 15 = 1220) check out.

## The twelve deviations, with a verdict each

| # | deviation | verdict |
| --- | --- | --- |
| D1 | corpus copied from ms HEAD `504ff46`, not `cd0a60f` | **Accepted.** Bytes verified identical to `cd0a60f`'s; I re-verified the sha independently. Pin is accurate. |
| D2 | trailer order — message trailers instead of `git commit -s` | **Accepted.** All six commits carry a DCO sign-off first, then the Claude trailers, matching `main`. |
| D3 | `hashByPhrase` field moved Task 4 → Task 3 | **Accepted, and it is a real plan defect.** Task 3's own block calls `composerHashByPhraseSync`, which assigns the field; Task 3 cannot compile as ordered. Reported honestly with the measured compile error; belongs in the plan's errata. |
| D4 | Task 3's stub dropped its locator first line | **Accepted.** File replaced wholesale in Task 4; block checker 26/26. |
| D5 | Task 4 Step 2's Expected line does not reproduce | **Accepted.** Still RED at the same checkpoint, by compile failure rather than at runtime; plan errata. |
| D6 | `NormalisePassphrase` mutation fails 6, not 4 | **Accepted.** Scope claim exact (only the two named rows); the third assertion per row is `DeriveHardened != PreimageHardened`, added after the plan measured it. |
| D7 | Task 2's `!f.Unshared` mutation not compilable as stated | **Accepted.** `_, perr := ParsePrefix(...)` is the same behavioural mutation and produced the plan's exact line. |
| D8 | Task 1's `codex32.New` mutation had no code in the plan | **Accepted.** The plan's named rows 11-13 fail; the two extra failures are tests added by a later fold, correctly explained. |
| D9 | `holdConfirm`-release mutation fails rather than hangs | **Accepted.** The harness's frame-pump bound converts a stuck hold into a bounded failure; same mechanism, caught either way. |
| D10 | the walk embeds the mixed-case digest as a constant | **Accepted.** `python3 -m http.server` cannot serve `../hashlock/testdata`; every constant is named with its corpus field, and nothing in the walk recomputes a digest. |
| D11 | the walk's key grid probed by tapping, not `shScreen` | **Accepted, and the honest half of F-481's discovery.** The digest is the oracle: one mistyped character changes SHA-256 completely, so trial 1 landing on the corpus constant is a 28-press proof. |
| D12 | the walk's `Deriving` post-condition is a race, not an assertion | **Accepted.** Demanding the countdown frame would fail on a device that is merely fast; `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` gates that screen on a clock the test controls. |

Two further **departures from the SPEC** (not from the plan) are unrecorded — see **M-1**.
The plan authorises both; nothing files them.

## Task 7 — F-474, verified (item 11)

- **Seal error fields.** `RecordNotPermittedError{Index int, Class Classification, Section
  Section, Preimage bool}`, returned from `AdmitSection`'s allow-list arm, `Unwrap`ing to
  `ErrRecordNotPermitted` — additive, every `errors.Is` site intact (M8 proves it).
  `Index` is `i`, 0-based as `me` counts; `isPreimageRecord` runs only on the refusal path,
  which returns immediately. `Error()` names the **class** and, for a preimage, the kind in
  parentheses — so H0's `TestAdmitSectionRefusesAPreimagePlateAsUnknown` still holds the fact
  it exists to hold.
- **The copy.** `Record N is <noun>. This payload cannot be unlocked here. Nothing was
  opened.` — true: `AdmitSection` wipes every record it copied and returns none. The preimage
  noun is H0's reader words, `a hashlock preimage, not a seed`.
- **The modal fit.** `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` runs
  `assertModalBodyFits(… errorScreenBody …)` on **all four** body rows, and the table drives
  three indices (0, 1, 7, 2) and four kinds — so a body that hardcoded its index or ignored
  `Preimage` dies here, which the flow test alone could not see.
- **Was the RED real?** Yes. M7 reproduces it exactly:
  `never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"`. The
  `errors.As` arm sits after the two `errors.Is` arms for unrelated sentinels; no shadowing.
  `var notPermitted` is declared inside the retry loop, so it cannot leak across iterations.
- **F-475 re-schedule recorded?** Yes — `design/FOLLOWUPS.md`: *"(owning phase: **H3** —
  re-scheduled from H2, see below)"*, with the original scheduling argument kept rather than
  overwritten. Condition held: H2 vendors `hashlock-v0.8.json` and never touches
  `codex32_seam_vectors.json`. **F-474 CLOSED by fork `17b3979`.** `grep -c 'owning phase:
  **H2**'` → 0: the phase reconciles clean.

## Closing counts

| severity | n | items |
| --- | --- | --- |
| Critical | **0** | — |
| Important | **2** | I-1 (no gate on `chars: <n>` / `first8..last8`), I-2 (F-481 gates) |
| Minor | **3** | M-1 (two unrecorded §4 copy departures), M-2 (fail-open on a dead `Deriver`), M-3 (secret handling, non-gating) |
| Nit | **2** | N-1 (refusal default arm), N-2 (`IsMS1Shaped` Unicode fold) |

**NOT GREEN.** Both Importants are cheap: I-1 is four lines in an existing test (written and
proven to kill both survivors); I-2 is one line deleted plus a test that fails without it.
Neither touches the derivation, the rule, the Back contract or the HOLD gate — all four of
which I attacked directly and could not break.

---

*Housekeeping: `/scratch/code/shibboleth/.tmp/h2-review`, `h2-base`, `h2-e1bf137` and
`h2-t3` were removed with `git worktree remove --force` after the last measurement. Nothing
was committed on either branch, and no phrase, preimage or record byte appears in this file
beyond the corpus's own public test vectors.*
