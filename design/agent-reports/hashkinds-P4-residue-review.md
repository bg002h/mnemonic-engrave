# hashkinds P4 residue review — `8225081..b361b22` (3 files, +215/-15)

**VERDICT: NOT GREEN — 0 Critical / 2 Important / 4 Minor / 3 Nit.**

Scope: `cmd/emu/walk_hashlock_phrase.js`, `gui/walk_copy_anchors_test.go`,
`engrave/h6_qr_test.go` only, at `b361b22`. Nothing before `8225081` was
re-audited.

## What I ran (everything below is measured, not read)

Scratch tree, no repo modification:

```sh
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH
rm -rf /tmp/shrev && mkdir -p /tmp/shrev
cd /scratch/code/shibboleth/seedhammer && git archive b361b22 | tar -x -C /tmp/shrev
```

| check | result |
| --- | --- |
| `go test -count=1 -v -run TestEmulatorWalksQuoteCopyThatStillExists ./gui/` | PASS, **6 subtests actually ran** (confirmed by `-v`) |
| `gui` whole package, `gui-shard-test.sh ./gui/ 24` | `ok — all 1333 tests ran across 24 shards`, 25 s wall |
| `gofmt -l .` | exactly the 5-file baseline (`gui/transaction*.go`, `mt/mt*.go`) — new file clean |
| `node --check cmd/emu/walk_hashlock_phrase.js` | ok |
| `go vet ./gui/ ./engrave/` | 5 `testing.ArtifactDir requires go1.26` lines — **identical at `8225081`**, pre-existing, out of scope |
| `ANCHOR_HARD_RIPEMD160` vs the corpus | `hashlock/testdata/hashlock-v0.8.json` `derivation[0].hardened_h_ripemd160` = `09e7bb5051d89788fb4e4b374126721dbcc2946b` — **exact match**, 40 chars. The oracle is genuine, not device-supplied. |
| `KIND_ROW_RIPEMD160 = 2` | `gui/composer_hash.go:132` `composerHashKinds = {KindSha256, KindHash256, KindRipemd160, KindHash160}` — row 2 is ripemd160 ✓ |
| `md.HashKind.Token()` (`md/compose.go:206`) | `sha256 / hash256 / ripemd160 / hash160` — the four regex alternatives are the real tokens ✓ |

### The two loosened regexes are SOUND — that hunt came back empty

Driven with `node` against the **real** production strings (dumped by a throwaway
`gui` test calling `composerCopyHashlockConfirm` / `…Reconcile` /
`…PreimageConfirm` / `composerCopyPreimagePlateRow` for all four kinds):

* 16/16 combinations (4 kinds × 4 screen bodies) return the correct token, plus
  both pre-cycle (kind-less) shapes. The commit's "verified against all three
  frame shapes" holds, and in fact all four kinds do.
* **Alternation order is provably irrelevant here**: no alternative is a prefix
  of another (`sha256`, `hash256`, `ripemd160`, `hash160`), so `hash256` can
  never be shadowed by `sha256`, and `hash160` never by `hash256`.
* **The row-absent vs digest-wrong distinction still holds.** With the header
  row deleted but the rest of the body intact, `drawnToken` THROWS on all 12
  body×kind combinations — including the reconcile body, whose prose contains
  `hashlock` and `--kind sha256` and still produces no false token. The kind
  pick screen (`sha256 64 hex / hash256 64 hex / …`) also throws.
* `censusPlateToken` returns `THROW:no-digest` for a dropped digest at all four
  kinds and `THROW:no-row` when path 1's row is absent.
* Brute force, 20 000 random digests × 4 kinds, both functions: **0 mis-parses.**

No finding here.

---

## Important

### I-1 — `gui/walk_copy_anchors_test.go:39-41`: the gate claims a coverage it does not have, and the gap is the same class it was built to stop

The doc block asserts:

> "This does not gate every string a walk uses; screen TITLES and fixed labels
> are not composer copy. **What it gates is every fragment that comes from a
> `composerCopy*` body**, which is where all three failures were."

That is false. Measured by extracting the 81 `composerCopy*` function bodies
from `gui/*.go` and intersecting them with every string literal passed to
`waitFor` / `must` / `mustNot` / `raceFor` in `cmd/emu/*.js`:

| walk | rows in the table | `composerCopy*` fragments it asserts on that are **not** in the table |
| --- | --- | --- |
| `walk_hashlock_phrase.js` | 4 | **9** |
| `shots_composer.js` | 2 | **6** |
| 5 other walks | 0 | 5 (`EXPERIMENTAL`) |
| **total** | **6** | **20** |

The 9 in the hashlock walk: `One phrase per policy` (`composerCopyHashlockConfirm`),
`check the digest matches` and `If they differ` (`composerCopyHashlockReconcile`),
`No hash record in the payload` + `ms hashlock on the host`
(`composerCopyHashlockNoPayloadLead`), `Keep each preimage plate apart from the
policy plates and from the others.` (`composerCopyPreimageKeepApart`),
`Nothing outside this device` (`composerCopyNothingChecked`), `Deriving`
(`composerCopyHashlockDerivingLead`), `method: ` / `phrase: ` / ` characters`
(`composerCopyHashlockConfirm`, `composerCopyPreimagePlateLead`).

**Reproduction — three production rewordings that break the walk and leave the
gate green.** In `/tmp/shrev`, each applied alone, then reverted:

```sh
# (a) the confirm modal's reuse line -- walk_hashlock_phrase.js:537 waits for it
sed -i 's/"One phrase per policy. Never use it as a passphrase/"One phrase for each policy. Never use it as a passphrase/' gui/composer_copy.go
go test -count=1 -run TestEmulatorWalksQuoteCopyThatStillExists ./gui/   # -> ok

# (b) the reconcile instruction -- walk_hashlock_phrase.js:621 waits for it
sed -i 's/and check the digest matches. /and check that the digest matches. /' gui/composer_copy.go
go test -count=1 -run TestEmulatorWalksQuoteCopyThatStillExists ./gui/   # -> ok

# (c) §8.3's census heading -- walk_hashlock_phrase.js:733 waits for it verbatim
sed -i 's/cut first and NOT part of this backup:/cut first and NOT part of this backup./' gui/composer_copy.go
go test -count=1 -run TestEmulatorWalksQuoteCopyThatStillExists ./gui/   # -> ok
```

All three print `ok seedhammer.com/gui`. All three leave the walk syntactically
perfect and broken at a specific frame — (a) 60 s in, (b) at the reconcile
wait, (c) at the census — which is byte-for-byte the failure mode the file's
own header calls "the class that has now bitten three times, each time
silently."

Control: the gate's *positive* half genuinely works. Rewording
`Write down the phrase` → `Write down your phrase` in `composerCopyHashlockConfirm`
FAILS the `Write down the phrase` row; dropping `--kind` from
`composerCopyHashlockReconcile` FAILS the `run ms hashlock --kind` row; editing
the walk away from the copy FAILS the same row from the other side. The
mechanism is right — the **table is four rows where the stated rule needs
thirteen**, and the comment tells the next reader the opposite.

**Why Important, not Minor:** this is a defect in what the gate *claims* to
have done. A maintainer who reads lines 39-41 will believe `composerCopy*` is
covered and will not add a row when they add the next `waitFor`. The stated
rule ("A WALK MAY NAME PRODUCTION COPY ONLY THROUGH THIS TABLE", line 33) is
also already violated by the very file that states it.

**Fix (either is enough):** (i) fill the table to match the claim — the 20 rows
are enumerable mechanically with the script sketched above; or (ii) if the
intent was only to anchor the four known drifts, say *that*, and delete the
"every fragment that comes from a `composerCopy*` body" sentence. Option (i)
plus a meta-test that greps the walks for `waitFor`/`must` literals and fails
on any that lives in a `composerCopy*` body and not in the table is the only
version that closes the class rather than adding a fourth fix.

### I-2 — `cmd/emu/walk_hashlock_phrase.js:830-833`: the 40-hex assertion is dead code — it cannot fail

```js
826  if (last.digest !== ANCHOR_HARD_RIPEMD160) { …throw… }
830  if (last.digest.length !== 40) {
831    throw new Error(`a ripemd160 digest reached the seam as ${last.digest.length} hex ` +
832      "characters, not 40 -- the alloc-gate padding is observable again (§7.3).");
833  }
```

Line 830 is reachable only when line 826 did **not** throw, i.e. only when
`last.digest === ANCHOR_HARD_RIPEMD160`. That constant is 40 characters
(measured: `node -e 'console.log("09e7bb5051d89788fb4e4b374126721dbcc2946b".length)'`
→ `40`). String equality implies equal length, so `last.digest.length !== 40`
is **unreachable**. Verified by simulating the two checks in source order:

```
correct digest          -> PASS
padded to 64 hex        -> FAIL:corpus-mismatch   (never reaches the length branch)
```

`8e582dd`'s message advertises coverage this line does not add: *"It also
asserts the digest is 40 hex, because a 64 would mean the alloc-gate padding
became observable again."*

**Mitigation, stated plainly so this can be weighed honestly:** nothing escapes
— a padded 64-hex digest still fails, one line earlier, and the message prints
both `stored` and `corpus`, so the padding is visible in the failure. This is a
dead assertion and a false coverage claim, **not** a false PASS. Graded
Important under the brief's explicit rule ("a gate that cannot fail … is
Important"); downgrade to Minor is defensible with the above in hand.

**Fix:** two lines — move the `length !== 40` check **above** the corpus
comparison. It then fails first, with the diagnostic it was written to give.

---

## Minor

### M-1 — `gui/walk_copy_anchors_test.go:54-57`: the `ripemd160` row's walk-side half is vacuous

`ripemd160` occurs **15 times** in `cmd/emu/walk_hashlock_phrase.js` — 6 in
comments, 2 inside the two regex literals (lines 389, 458), 1 in the constant
name `ANCHOR_HARD_RIPEMD160`, 1 in `KIND_ROW_RIPEMD160`'s comment. So
`strings.Contains(walkFile, "ripemd160")` is satisfied by the file's own
scaffolding.

Reproduction:

```sh
sed -i '/must(modal, "ripemd160", "the confirm modal names the kind it derived/d' \
  cmd/emu/walk_hashlock_phrase.js
go test -count=1 -run TestEmulatorWalksQuoteCopyThatStillExists ./gui/   # -> ok
```

The one assertion that row exists to protect (line 807) can be deleted and the
row still passes. Its production half is fine — removing the kind from
`composerCopyHashlockConfirm` does fail it.

Contrast with the other three rows, which are tight: `run ms hashlock --kind`
occurs **once**, `32-byte preimage` twice, `Write down the phrase` twice — all
at assertion sites.

**Fix:** anchor on a fragment unique to the assertion, e.g. the token the modal
draws (`09e7bb50..bcc2946b`), or `"ripemd160", "the confirm modal names the kind`.

### M-2 — `cmd/emu/walk_hashlock_phrase.js:389, 458`: the four kind tokens are hardcoded in both regexes with nothing binding them to `md.HashKind.Token()`

A fifth kind — or a renamed token — makes `censusPlateToken` report
*"the census carries no row for path N … §8.3's block did not report the plate
that was accepted"* for a row that is present and correct. That is exactly the
mis-diagnosis lines 431-439 split the two failure messages apart to prevent.
`drawnToken` degrades the same way ("no `hash [<kind>] <first8>..<last8>` token
in the frame").

The new boundary table catches a renamed `ripemd160` or `sha256` on the
*production* side (its `produced` expressions call `Token()`), but nothing
catches a **new** kind, and nothing ties the walk's alternation to the Go enum.
This is a follow-up, not a gate: the failure is loud and safe, just mislabelled.

### M-3 — `cmd/emu/walk_hashlock_phrase.js:806-834`: `runKindTrial()` asserts nothing about what the ripemd160 confirm modal DREW

It captures `modal`, checks only `must(modal, "ripemd160", …)` (line 807), and
returns the frame unasserted. The walk's own header doctrine (lines 57-75,
F-485) is that "what the screen says is not what the policy holds" are two
independent claims — and the 20-byte width is precisely the newly-risky display
path: `short8`'s own doc (lines 351-358) records that `h[56:]` PANICS on a
40-hex digest, so `hashlockFirst8Last8` had to be changed to slice from the end.
The single trial that exercises a 40-hex digest end to end never checks the
abbreviation the screen painted.

One line closes it: `must(modal, short8(ANCHOR_HARD_RIPEMD160), …)`.

Mitigation: `hashlockFirst8Last8` is covered in CI at both widths
(`gui/composer_state_hook_test.go:195`, `gui/composer_hashlock_plates_test.go:207`,
`gui/composer_preimage_plate_test.go:297`), so this is missing overlap rather
than an uncovered path.

### M-4 — `cmd/emu/walk_hashlock_phrase.js:810-815`: `runKindTrial()` does not pin the composition's shape

`run()` asserts `before.length !== 1` (line 574) with the message "the walk
built a different policy than it thinks". `runKindTrial()` instead takes
`stored[stored.length - 1]` and never checks the count. On an empty array that
is `undefined`, which the next check catches — but as "the ripemd160 path holds
no hash after the hold", the wrong diagnosis for "the walk is not where it
thinks it is". No false pass; the kind and corpus-digest assertions that follow
are strong.

---

## Nit

### N-1 — `cmd/emu/walk_hashlock_phrase.js:386-388`: the comment's stated hazard does not exist

> "The token itself is unambiguous ([0-9a-f] exactly 8, then `..`, then 8),
> which is what lets the kind be skipped without knowing its length —
> **`hash256` is hex-shaped and would otherwise be a real hazard here.**"

`hash256` contains `h` and `s`, which are not in `[0-9a-f]`; it can never be
consumed by the digest group. Measured:

```
/^[0-9a-f]+$/.test("hash256")                      -> false
hash256 REMOVED from the alternation, hash256 frame -> THROW (no match)
```

Removing `hash256` from the alternation produces a **false negative** (the row
reads as absent), never a mis-parse. The conclusion — keep the tight token — is
right; the reason given for it is not.

### N-2 — `cmd/emu/walk_hashlock_phrase.js:636-637`: stray double blank line

`8e582dd` inserted a bare `+` blank line before the H6 arm banner, leaving two
consecutive blank lines. Nothing else in the file does this.

### N-3 — `cmd/emu/walk_hashlock_phrase.js:787`: the stale-`emu.wasm` message drifted from `run()`'s

`runKindTrial()` says "stale or wrong emu.wasm; serve on a FRESH port";
`run():510` says "…rebuild from the hashlock-h5 branch and serve on a FRESH
port". Copy-paste divergence in a message the operator sees at the moment the
walk is unusable. Arguably an improvement (the branch hint is stale); worth
making them one string either way.

---

## Checked and clean — no finding

* **`trial()`'s new `kindRow = 0` default (line 297).** All four existing call
  sites — `run():535, 542, 549, 558` — pass two arguments, so `kindRow` is `0`
  and `pickKind(0)` is exactly the hardcoded `pickKind(0)` it replaced.
  `composerHashKinds[0] == md.KindSha256`. `chooseRow` taps the *i*-th target
  rectangle, so the row→kind mapping does not depend on which row
  `composerHashKindPick` opens selected on. Semantics preserved.
* **`runKindTrial()`'s copied boot sequence (lines 785-804).** Compared
  statement-by-statement against `run():507-532`: identical through
  `waitFor("Type a hashlock phrase")`, minus the two `must()` checks on the
  no-payload lead, which `run()` still covers. All 7 probed `window.sh*`
  functions are actually used by the code below. Nothing stale.
* **`runKindTrial()`'s kind discrimination.** `must(modal, "ripemd160")` does
  discriminate: the `hash256` and `hash160` confirm bodies contain neither
  `ripemd160` as a substring, and §8's `composerCopyTwentyByteUnseen` warning
  (the only other screen naming a kind) does not fire on the phrase route.
* **The `..`-relative path** `filepath.Join("..", "cmd", "emu", tc.walk)`
  (`walk_copy_anchors_test.go:81`) resolves correctly — Go runs tests with cwd
  = package dir. A missing/renamed walk is `t.Fatalf`, not a silent skip.
* **Stale-string sweep across all 15 `cmd/emu/*.js`**: no remaining
  `Write down this phrase`, `run ms hashlock with this phrase`, `Type 64 hex`,
  or `must be SHA-256` outside one explanatory comment in `shots_composer.js`.
* **`engrave/h6_qr_test.go:27-32`** — the corrected comment is accurate:
  `hashlock` imports only stdlib + `golang.org/x/crypto/ripemd160` +
  `seedhammer.com/md` + `seedhammer.com/seal` (no `engrave`), and the file does
  import `seedhammer.com/hashlock` and call `hashlock.MethodLine` immediately
  below. Comment-only change; no behaviour touched.
* **The whole `gui` package is green** at `b361b22` (1333/1333), so the new test
  file introduces no symbol collision or regression.

## Scratch artefacts

`/tmp/shrev` (tree at `b361b22`), `/tmp/shbase` (tree at `8225081`),
`/tmp/frames.json` (real production copy for all four kinds),
`/tmp/gui-shard.log`. Nothing under
`/scratch/code/shibboleth/seedhammer` was modified.
