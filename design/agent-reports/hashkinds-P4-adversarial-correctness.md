# hashkinds P4 — adversarial correctness review of `git diff main..hashkinds-p4`

**VERDICT: NOT GREEN — 0 Critical / 6 Important / 6 Minor / 2 Nit.**

Repo: `/scratch/code/shibboleth/seedhammer`, branch `hashkinds-p4` (6 commits, 50 files,
+2285/−602). Spec: `mnemonic-engrave/design/SPEC_hashlock_kinds.md`.

**On the ONE QUESTION.** I could not construct a case where the diff produces a wallet
committed to a digest or kind other than the one the operator chose, a truncated or padded
digest, a lock lowering to the wrong script, or a decode reporting the wrong kind. The
padding trap is closed properly: `md.HashLock` is genuinely non-comparable (`_ [0]func()`),
every one of the ten production `.Digest()` call sites reads at the kind's width, and no
call site appends through the returned slice. `md.HashKind.tag()` maps
sha256→`0x1D`/hash256→`0x1F`/ripemd160→`0x20`/hash160→`0x1E`, which matches `md/md.go:69-72`
and the body widths at `md/md.go:464,474`. I independently recomputed **88 digest values**
from `hashlock/testdata/hashlock-v0.8.json` in python3 `hashlib` (sha256, sha256d,
ripemd160, ripemd160∘sha256) — **0 mismatches** — and re-verified the vendored
`record_class_vectors.json` against `mnemonic-engrave` `73f3e7d8`: sha256 `d6766fdd…`,
81 rows, pin agrees.

What is wrong is **completeness**: three normative spec surfaces (§8, §13.3, §12 item 2)
are absent, one shipped walk is broken by the diff, one existing gate was falsified by the
diff, and one new Back leg breaks §7.1's stated contract.

Everything below was reproduced by execution. Probe files were written, run, and deleted;
the tree is as I found it (one pre-existing untracked artifact noted at the end).

---

## Critical

None.

---

## Important

### I-1 — §8, "The 20-byte warning", is not implemented anywhere

**Where:** absent. `gui/composer_copy.go`, `gui/composer_hash.go`,
`gui/composer_hashlock.go`.

**Evidence.** §8 is normative: *"Fires for `ripemd160`/`hash160` when the digest was **not**
device-derived: payload-supplied and typed digests warn; the phrase route stays silent.
Provenance comes from `composerState.hashlockHeld`."* That is a fork-only structure, so §8
is unambiguously a phase-4 item (§9 row 4).

Three greps over production code (`_test.go` excluded):

```
grep -rn "KindRipemd160|KindHash160" gui/ sysw/ md/ hashlock/   -> 1 hit: the row-order list
                                                                   gui/composer_hash.go:96
grep -rn "DigestLen()" (production)                             -> 10 hits, none a predicate
grep -rn "20-byte|20 byte" (production)                         -> comments only
```

There is no copy function for it (`grep -n "^func composerCopy" gui/composer_copy.go` lists
41; none is the 20-byte warning), no predicate, and no test. The word "warning" appears in
this area only for H6 §8.5's QR warning and the method warning.

**Counterexample.** Compose a path, choose `ripemd160` on the kind screen, type a 40-hex
digest from a slip of paper (the canonical not-device-derived case §8 names). The device
assigns the lock and proceeds to consent with no warning at any point. Same for a payload
`hash:ripemd160:<40hex>` record taken from band 1.

**Why it matters.** A spec section that says "fires" and never fires is an unmet guarantee,
and this is the one warning standing between an operator and a 40-hex value they may have
produced by truncating a 64-hex sha256 digest.

---

### I-2 — §13.3 is not implemented: the census row still describes all four kinds identically

**Where:** `gui/composer_copy.go:881` (`composerCopyPreimagePlateRow`), `:888`
(`composerCopyPreimageNotOnAnyPath`), `:893` (`composerCopyPreimageDeclined`);
`gui/composer_preimage_plate.go:413-427`.

**Evidence.** §9's phase-4 row names the deliverables as *"the locator `hash` row, the
confirm and reconciliation bodies, the masked pick lead, **and the census row**"*. §11's
gate table lists *"the §8.3 census row | the composer's plate inventory"*. §13.3: *"It is
the operator's only inventory of what they must store apart, and today it describes all four
constructions identically — two plates for one preimage under two kinds differ by sixteen
hex characters and nothing else."*

The locator, both confirm bodies, the reconcile body and the masked pick lead all gained a
kind. The census rows did not: `composerCopyPreimagePlateRow(path int, first8last8, form
string)` has the same three parameters it had before the diff, and `composerCopyTable` in
`gui/composer_copy_test.go` still asserts the kind-less text.

**Counterexample (executed).** Two paths carrying the *same* preimage under two kinds —
exactly §13.3's stated case. `composerPreimageCensusLines` returns:

```
Plus 2 preimage plate(s), cut first and NOT part of this backup:
path 1  abababab..abababab  phrase, hardened
path 2  cdcdcdcd..cdcdcdcd  phrase, hardened
```

Two plates, one preimage, two different hash functions, and the rows differ only by the
sixteen hex characters §13.3 says are not enough. For contrast, the consent screen from the
same composition — which the diff *did* fix — reads:

```
Path 1: 3 key(s), custom
  hash sha256 abababab..abababab
Path 2: 1 key
  hash ripemd160 cdcdcdcd..cdcdcdcd
```

**Why it matters.** §5's both-or-neither rule is stated as binding on *every* screen, and
the census is the screen the operator reads while deciding what to store apart from what.

---

### I-3 — `cmd/emu/shots_composer.js` asserts a row label this diff retired; the composer screenshot walk now throws

**Where:** `cmd/emu/shots_composer.js:667`.

```js
must(hashRows, "Type 64 hex", "the type-it row");
```

**Evidence.** `gui/composer_hash.go:246` now defines `composerHashRowHex = "Type a digest"`
and `composerHashRows` appends it at `:459`. A repo-wide grep for the retired literal returns
exactly one live site — this one. `must` (`shots_composer.js:138`) *throws* when the needle
is absent. The diff edits this very file **two lines below** (the `"The hash must be
SHA-256"` → `"The preimage must be"` fix at `:670-673`), so the stale line was read past.

**Counterexample.** Run the S4 composer shots walk. Step 12 ("Path 2's hash lock, from the
payload's own `hash:` record") reaches `Which hash?`, and the third `must` throws
`the type-it row: the screen does not carry "Type 64 hex"`. The walk produces no shots from
`c05-hash-rule.png` onward and never reaches the stub, template-ID or consent assertions.

**Why it matters.** §11 lists *"the literal `"Type 64 hex"`, in code **and** test"* as a gate
that must move deliberately. It moved in `gui/` and in `composer_hash_test.go` and did not
move here, so the composer's own acceptance walk is dead. `go test ./...` cannot see it —
this walk is driven against the emulator, not by the Go suite — which is precisely why it
survived a green suite.

---

### I-4 — §12 acceptance item 2 is unmet, and for the sha256/hash256 pair it is unbuildable through the seam as built

**Where:** `cmd/emu/walk_hashlock_phrase.js:296`; `cmd/emu/composer_js.go:64`.

**Evidence, part 1 — no walk composes a non-sha256 kind.** The only `pickKind` call in the
walk is `await pickKind(0)` at `:296`, and `pickKind`'s own doc says *"`i` indexes
gui.composerHashKinds — 0 is sha256, the default."* `grep -n "pickKind" walk_hashlock_phrase.js`
returns the definition at `:446` and that single call. No other `.js` walk was added by the
diff (diffstat: `shots_composer.js` and `walk_hashlock_phrase.js` only).

§12 item 2 is explicit that the assertion is the acceptance: *"a walk that composes one and
never asserts the kind satisfies the sentence and gates nothing."* Neither half happens.

**Evidence, part 2 — the seam cannot carry the kind.** §7.3's table says all three layers
"become kind-aware together". The Go hook did (`ComposerPathHashes() []*md.HashLock`). The
JS bridge did **not**:

```go
out = append(out, hex.EncodeToString(h.Digest()))   // cmd/emu/composer_js.go:64
```

The page receives a hex string and nothing else. It became *width*-aware, not kind-aware.

**Counterexample.** Write the walk §12 item 2 asks for, for `hash256`. `pickKind(1)`, derive,
read `shComposerPathHashes()`. You get 64 hex. A `sha256` composition of the same preimage
also yields 64 hex. The two digests differ, so a walk *could* compare against the corpus's
`hardened_h_hash256` column and catch a wrong digest — but it cannot assert *"the composition
stores that kind"*, which is the clause §12 item 2 actually writes, and which §3-F4 names as
the failure with "no structural signal whatever". A walk that composed `sha256` while the
screen said `hash256` would read identically at the seam.

**Why it matters.** This repo's own rule: *"a plan may not close while any of its own gates
has never been run"*, and its corollary, *"a gate that cannot fail is not a gate."* §7.3
exists specifically so this gate *can* fail; the JS layer's half of it was not finished.

(The Go-level end-to-end `TestComposerCanBuildARipemd160HashlockOnTheDevice`
(`gui/composer_hashlock_test.go:1702`) is good and does assert the kind — but §12 item 2
asks for an emulator walk, on the device path, and names the KAT row.)

---

### I-5 — Back from the pad reopens the kind screen scrolled past the kinds above the chosen one, and one page press silently moves the selection to sha256

**Where:** `gui/composer_paged.go:314-318` (`composerPickScreenFrom`: `start := sel`), with
the page-wrap clamp at `:381-388`; reached from `gui/composer_hash.go:589` and `:571`.

**Evidence.** `composerPickScreenFrom` seeds both the cursor **and the page origin** from
`initial`. The page-wrap clamp then reads the *previous* page's `shown`.

**Counterexample (executed, `gui` harness at `sh2DisplaySize`).**

1. `Path 1 hash` → `Type a digest` → §8i modal → kind screen.
2. Tap row 3, `hash160`. Pad opens: `0 of 40 hex`. Correct.
3. Press Back (Button1). §7.1: *"Back from the hex pad → the kind screen, kind still
   selected."* The screen that draws is:

   ```
   Hash function
   All four take a 32-byte preimage. sha256 is the usual choice.
   hash160   40 hex
   ```

   **One tappable row.** `sha256`, `hash256` and `ripemd160` are not on the screen. Measured:
   `plateHitPoints` = 1.
4. Press Button2 to page (the only way to see the other three). All four rows now draw —
   and the highlighted row is **`sha256`**, not `hash160`. Nothing announced the change.
5. Press Button3 (take) without tapping a row. The pad opens at **`0 of 64 hex`**.

Frames captured verbatim:

```
pad after choosing hash160:  "0123456789ABCDEF 0 of 40 hex Hashlock"
kind screen after Back:      1 tappable row,  "...sha256 is the usual choice. hash160 40 hex"
after page:                  4 tappable rows, "...sha256 64hex hash256 64hex ripemd160 40hex hash160 40hex"
after take:                  "0123456789ABCDEF 0 of 64 hex Hashlock"
```

The mechanism is `start := sel` (step 3) plus `if sel >= start+shown { sel = start }` at
`:386` evaluating `3 >= 0+1` with the *stale* `shown = 1` after the wrap (step 4).

**Scope, measured rather than assumed.** I checked whether this can silently swap a
**same-width** pair, which would be §3-F4's funds-loss case:

- `initial = 1` (`hash256`): `shown = 3`, `start+shown = 4 = len`, wrap to 0, `1 >= 0+3` is
  false → the cursor **stays** on `hash256`. **Does not reproduce.** (Executed.)
- `initial = 2` (`ripemd160`): `shown = 2` → cursor reverts to `sha256`; 40 hex vs 64 hex, so
  the operator is blocked at the pad rather than silently mis-locked.
- `initial = 3` (`hash160`): as above; 40 vs 64.

So this is not a wrong result — the silent landing is always `sha256` (64 hex) and the two
kinds it can displace are both 40 hex. It is an Important navigation defect: §7.1's Back
contract ("kind still selected") is broken by one ordinary press, and
`TestComposerHashKindScreenDrawsAllFourRows` — the gate §7.5 asks for — only ever exercises
`initial = 0`, so it cannot see any of this.

---

### I-6 — the diff falsified `engrave/h6_qr_test.go`'s QR-text copy; the constant-time QR module-budget gate now samples content the firmware no longer emits

**Where:** `engrave/h6_qr_test.go:35` (`h6QRText`), used by `h6ShapesFor` (`:274`) and
`TestConstantTimeQRBudgetBoundsEveryPayload` (`:196`).

```go
func h6QRText(method, phrase string) string {
	return "hashlock v1\n" + method + "\nphrase: " + phrase   // the PRE-DIFF format
}
```

`hashlock.QRText` now emits a fourth line, `\nhash: <kind>` (`hashlock/hashlock.go:251-264`).
The diff **removed** the identical second copy in `backup/hashlock_test.go` (good) and left
this one, which the file's own header claims is safe: *"backup/hashlock_test.go asserts the
same text against hashlock's own constants, so a parameter change cannot leave both lying."*
That claim is now false — this is the copy left lying.

**Measured (executed against the real `hashlock.QRText`):**

| case | production bytes | what the gate samples |
| --- | --- | --- |
| sha256, 100-char phrase | 148 | 135 |
| hardened + sha256, 100-char phrase | 207 | 194 |
| hardened + ripemd160, 100-char phrase | **210** | 194 |
| base, sha256 / hardened | 48 / 107 | 35 / 94 |

`h6ShapesFor`'s comment — *"A `sha256` plate is 35 + len(phrase) bytes and a `hardened` one
94 + len(phrase)"* — is false for every plate the firmware now cuts (48 and 107-110).

**Counterexample.** `TestConstantTimeQRBudgetBoundsEveryPayload` asserts, per its own header,
*"EVERY DIM §8.6 CONTENT REACHES"*. It reports PASS while every payload it encodes is
13-16 bytes shorter than any plate the device produces, so the content class this cycle
introduced is outside the sample. `ConstantQR` **refuses** when `len(modules) > nmod`
(`engrave/engrave.go:583-585`), and that refusal surfaces as
`composerCopyPreimagePlateRefusal()` — a preimage plate that cannot be cut.

I ran the real content through `ConstantQR` to see whether the gap is already a breach — it
is not: observed/budget at dims 29..53 = 356/391, 490/547, 623/684, 795/843, 935/1013,
1151/1199, 1324/1399 over 25 samples per (method, length). So this is a coverage hole, not a
demonstrated failure. It is Important because the gate reports PASS on a question it no
longer asks, which is exactly the false-PASS class this project treats as blocking, and
because the 53-dim budget's buffer is 20 modules over a campaign that needed 7.7M samples to
converge on the *old* content class.

(`TestQRStaysAtFiftyThreeModules` in `hashlock/methodline_h6_test.go` is new and good — it
pins the *version* at 53 for all four kinds. It does not touch the module budget.)

---

## Minor

### M-1 — §11's explicitly phase-4 fix to the compose-vector pin's stale count was not made

`scripts/vendor-compose-vectors.sh:29` and `md/testdata/compose_vectors.provenance.json:6`
both still read *"if the file count is not 156"*. Measured: the pin lists **161** files and
`md/compose_vectors_pin_test.go:91` enforces 161.

§11 is explicit that this belongs here: *"Both the generator … and the file it writes … live
in the **fork**, and the re-pin is run from there — so this is a **phase 4** item, not phase 1
as an earlier draft said. It is fixed in this cycle rather than filed."* Neither file is in
the diff. Behaviourally harmless (the test enforces the true number); recorded because the
spec scheduled it to this phase.

### M-2 — `composerHexEntry`'s doc comment contradicts the function it documents

`gui/composer_hash.go:151-154`, immediately above the signature that now takes a `kind`:

> *"IT RETURNS A sha256 LOCK, and the pad's fixed 64-character bound is what says so:
> `Which hash?` offers no kind … A kind pick would come with its own character bound."*

The paragraph is a survival from the pre-kind-screen draft and sits four lines below a
paragraph saying the opposite ("THE KIND ARRIVES FROM THE SCREEN BEFORE IT"). A maintainer
auditing which screens can produce which kind is told the wrong answer by the function that
decides it.

### M-3 — `hashlockLockOf`'s doc comment claims every call site passes sha256; one does not

`gui/composer_hashlock.go:256-260`: *"EVERY CALL SITE PASSES md.KindSha256 TODAY."*
Measured — five call sites, and `gui/composer_hashlock.go:79` passes the operator's chosen
`kind`. Same class as M-2, on the function the spec designates as the fork's one named
kind→digest dispatch.

### M-4 — `ConstantQR`'s capacity comment is falsified by the new QR line

`engrave/engrave.go:505-507`: *"ECC-L caps at 230 bytes at v9 and the §8.6 worst case is 194
… so there are 36 bytes of headroom."* Measured after the diff: the worst case is **210**
(hardened + `ripemd160` + a 100-character phrase) and the headroom is **20** bytes. The bound
still holds (210 < 231), so nothing breaks — but this comment is the written justification
for the `dim > 53` refusal and it now states a retired number.

### M-5 — `sysw.HashRecord` has no caller and no test

`sysw/composer_records.go:247` is the §6 producer rule's Go half (bare for sha256, tagged
otherwise). `grep -rn "HashRecord("` over the repo returns only its definition, the parser,
and `ParseHashRecord` call sites. Nothing emits a `hash:` record through it and no test
exercises it, so §6's "input is liberal, output is conservative" has no Go-side gate. (The
fork may legitimately never emit one — but then the function is dead code in firmware.)

### M-6 — `TestHashLockIdentityIgnoresPadding` cannot fail on padding

`hashlock/hashlock_test.go:~380-440`. Its header says it *"pins what Go cannot express in the
type system: the alloc-gate padding must not be observable."* Both locks it compares are
built by `md.NewHashLock` from the same 20-byte slice, and `NewHashLock`
(`md/compose.go:281-287`) allocates a fresh zero `[32]byte` before copying — so no
constructor in the tree can produce non-zero padding and the named property has no
reachable negative case. What the test *does* check (kind is part of identity; wrong widths
are refused) is real and valuable; only the padding claim in its name is unfalsifiable.

---

## Nit

### N-1 — a test comment misattributes the false PASS it was written to fix

`gui/composer_gates_test.go:1745-1748` explains that searching the whole consent screen for
`"ripemd160"` passed under the early-`break` mutation *"because the consent screen also draws
the policy's miniscript, which spells `ripemd160(...)` itself."* Executed: the consent lines
for a two-kind policy contain no miniscript at all. The token appears because
`composerBranchLines` (`gui/composer_consent.go:100-104`) now draws
`hash ripemd160 cdcdcdcd..cdcdcdcd` itself. The scoping fix is right; the reason recorded
for it is not, and this tree treats a wrong reason as the thing that decays.

### N-2 — `collect`'s tag/body cross-switch drops a digest while leaving `Hashlock = true`

`md/policy_shape.go:294-318`. If a node carried `tagSha256` with a `hash160Body`,
`NewHashLock` refuses, the `if l, ok := …` arm falls through, and the branch reports
`Hashlock = true` with no entry in `Hashlocks` — the pre-diff shape §7.4 was written to
retire. Unreachable today: `readNodeDepth` (`md/md.go:463-482`) fixes the body width per tag,
so no decoded node can mismatch. Noted only because every sibling arm in `collect` returns
`false` on a shape it does not understand, and this one returns `true`. The self-check's
count comparison would catch it on the compose path.

---

## Also observed, outside the diff

The fork working tree carries an untracked 5.8 MB `emu` binary (`?? emu`, mtime 2026-09-15
20:00). Not produced by this review's probes and not part of the diff; flagging it so it is
not committed by accident.

---

## What I checked and found sound (so it is not re-derived)

- **`md/compose.go`** — `DigestLen`/`Token`/`tag`/`HashKindFromToken` panic rather than
  default on an unknown kind; `NewHashLock` refuses a wrong-width digest at the boundary
  (never truncates, never pads); `Digest()` slices at the kind's width; `Equal`/`MapKey`
  carry the kind; the `_ [0]func()` genuinely makes `==` and map-key use compile errors.
  `pathBody`'s lowering switches on the **kind** and writes a body of the kind's width.
- **Padding leak sweep** — all ten production `.Digest()` call sites hex or copy the returned
  slice; none appends through it (which would write into the padding). `[56:]` survives only
  in test literals over known-64-hex constants.
- **Eleven §5 re-keying sites** — `phraseDigests`, `hashlockHeld`, `composerPreimagePlates`'
  `seen` and its `rest` sort, `composerEveryHashedPathHeld`,
  `composerEveryHeldPathHasAPhrase`, `composerPreimagePlatePick`, `hashlockDerivedDigest`,
  `payloadStatesDigest`, `hashlockRelationLine`, `hashlockOtherPathLine`,
  `composerHashInPayload`, `hashlockPlatesMatch`, the self-check — all on `MapKey()`/`Equal`.
  Nil receivers are safe (`Equal` guards).
- **§6 grammar** — `ParseHashRecord` splits on the *last* colon, rejects case rather than
  folding it (no `ToLower`), gives an unknown token its own error, and fails closed through
  `classifyComposer` → `ClassUnknown` → inert. `hash:sha256:<64hex>` parses to the same lock
  as the bare form; `HashRecord` emits bare for sha256.
- **§7.4** — `collect` records a lock for all four tags at the right width; the self-check
  compares through `Equal`, so kind divergence is a mismatch rather than a digest comparison.
- **§13.1** — the locator's `hash` row and `QRText` both name the kind; the QR stays at 53
  modules for all four kinds (`TestQRStaysAtFiftyThreeModules`, new).
- **§13.2** — confirm, reconcile and the masked pick lead all name the kind; the reconcile
  sentence names `ms hashlock --kind <token>`, and I verified against
  `mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:131-141` that the flag exists and parses
  exactly those four lowercase tokens.
- **KAT** — 88 per-kind digest values recomputed independently in python3 `hashlib`, 0
  mismatches; `TestDigestKAT` exercises both the four functions and `DigestOf`'s dispatch, and
  refuses to pass on zero rows.
- **Provenance pins** — `record_class_vectors.json` sha256 and 81-row count agree with
  `mnemonic-engrave` `73f3e7d8`; `hashlock-v0.8.json`'s sha256 pin is duplicated as a second
  independent witness in `hashlock_test.go`.
