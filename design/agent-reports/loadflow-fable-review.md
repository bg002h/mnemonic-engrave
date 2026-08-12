# Review — syswLoadFlow diff (`b14662a..HEAD`, branch `sysw-port`) + two claims

Reviewer: fable (independent), 2026-08-12.
Scope: the 10-file diff and the two claims in the brief. No fresh audit outside it.
Everything below marked **measured** was run in the repo, not argued; experiment
code and commands are in the appendix. The tree was restored to pristine HEAD
afterward (`git status` clean, full `gui`+`sysw` suite green with
`SYSW_REQUIRE_VECTORS=1`).

## Verdict in three lines

1. **Q1: one Critical.** The sealed path cannot accept any passphrase —
   `make(bip39.Mnemonic, 24)` zero-fills, and `Word(0)` is "abandon", so entry
   terminates after one word and the KDF input is always `abandon ×24` (or
   `<w1> + abandon ×23`). Everything else in the flow checks out against §12,
   including both `[compared]` routes, measured.
2. **Q2(a): the F-146 diagnosis is wrong.** `iter.Pull` establishes exactly the
   ordering the entry says is missing. The assertions could not fail for two
   different, mundane reasons: they read `ret`/`ctx.sysw` **before the flow had
   returned** (parked mid-modal), and the natural mutant site — the flow's own
   malformed-region branch — **never executes**, because both shipped Readers
   pre-reject junk inside `Read()`. Both measured.
3. **Q2(b): nothing is blocked.** All five "missing" cases were written and run
   today with the existing harness. Three pass at HEAD. The two sealed ones
   **fail at HEAD — they catch the Critical** — and pass with the one-line fix.

---

## Q1 — the flow against §12

### C1 (Critical) — `gui/sysw_load.go:81`: no sealed payload can ever be opened on the device

`m := make(bip39.Mnemonic, 24)` produces 24 slots of `bip39.Word(0)`, which is
the wordlist entry **"abandon"**, not the empty marker. `inputWordsFlow`
(`gui/gui.go:769`) defines "empty" as `-1` in both places that matter:

- `entered()` counts slots `!= -1` → returns **24 always**, so the `n == 0`
  back-out at `sysw_load.go:92` is unreachable;
- the accept-advance loop (`gui/gui.go:857-865`) skips slots `!= -1` → after the
  **first** accepted word it runs `selected` off the end and returns.

So whatever the operator does — types one word, types twelve, presses done,
presses back — the flow reconstructs `pass` from 24 slots, 23 of which are the
pre-filled "abandon". `sysw.Open` then fails for every real payload:
`[compared]`'s AEAD route is unreachable in practice, and a sealed
`pub_len == 0` payload (the S-D shape) is **unconsumable on the device** — the
exact D1 failure ("a payload no device could ever consume") that §13 demoted a
security rule to prevent, back through a different door.

The house allocator exists and is used everywhere else: `emptyBIP39Mnemonic`
(`gui/gui.go:713`) — the other two `make(bip39.Mnemonic, …)` sites in gui are
the preview seed and the allocator itself. **Fix is one line.**

**Measured:** a test entering S-E's real passphrase ("abandon about") via
queued rune/button events hangs at HEAD inside the "That passphrase did not
open this payload" modal (`sysw_load.go:105`, stack in appendix); with
`emptyBIP39Mnemonic(24)` substituted, right-passphrase (session, `compared`,
mnemonic takeable) and wrong-passphrase (refusal, no session) both pass. The
fix was then reverted; it is the author's to make.

### I1 (Important) — F-146 is misfiled; see Q2(a) below. Its "blocked" hold on F-145 should be lifted.

### I2 (Important) — §8a's keyboard choice is absent: an ASCII-passphrase payload is unconsumable

Spec §8a: "The operator picks the keyboard at unlock: BIP-39 word, or free-text
ASCII." The flow offers only `inputWordsFlow` (`sysw_load.go:90`) — there is no
path to `NewPassphraseKeyboard` and no way to type a character outside the word
keyboard. The host ships the producing mode **today**: `me sysw pack
--passphrase-ask` (`crates/me-cli/src/main.rs:158,822`) seals under arbitrary
ASCII, and the host side even documents its own guard against unopenable
artifacts (`crates/me-cli/src/sysw/mod.rs:170-180` refuses the empty
passphrase for precisely this reason). A payload sealed with any non-wordlist
passphrase can never be opened on this flow — same D1 class as C1, surviving
C1's fix. If the keyboard choice is deliberately a later stage, the plan does
not say so (no stage for it in `IMPLEMENTATION_PLAN_systemwide_payloads.md`),
and until it lands the mode decision 8 fought to restore is device-dead.

### Clean results (real results, per the brief)

- **`[compared]` (§12.2) — both routes, neither widened nor narrowed.**
  Route 1: `compared = true` iff `h.Sealed()` **and** `sysw.Open` succeeded
  (`sysw_load.go:112-114`); no cliff condition (not narrowed); an unsealed
  parse-success does not set it (not widened — **measured**, E4b: declined
  digest → `compared == false`, `take` refuses). Route 2: gated on
  `h.PubLen > 0`, set only when `confirmReviewScreen` returns true; that
  helper and `ChoiceScreen.Choose` both fail closed on `ctx.Done`
  (`multisig_build.go:580`, `gui.go:1637`), so interruption cannot mint a
  `compared`. Both routes are decided at the single site, and
  `ctx.sysw` has exactly one assignment in the codebase (`sysw_load.go:135`).
  `take`'s `!compared` refusal held under test (E4b). Caveat: route 1 is
  *practically* unreachable until C1 is fixed.
- **`[digest-shown]` (§12.4) — exact.** Shown iff `h.PubLen > 0`
  (`sysw_load.go:120`); grep confirms this is the only display site of the
  sysw digest (the `unlock_*.go` hits are the frozen Sealed Payload's own
  `seal.FormatHash`). The digest recomputation via `strings.Split` is
  byte-identical to `splitRecords` for `PubLen > 0` (`Join∘Split` = identity;
  UTF-8 already validated by the `Open` that necessarily precedes it).
- **Bounds — provable, twice over.** `ParseHeader` caps `PubLen, CtLen ≤ 8191`
  before any arithmetic (`sysw/header.go:50`), so `TotalLen ≤ 16450` cannot
  wrap even a 32-bit int; the flow refuses `len(blob) < h.TotalLen()` before
  slicing `region`; therefore `region[52 : 52+int(h.PubLen)]` at
  `sysw_load.go:121` is in bounds (`TotalLen ≥ HeaderLen+PubLen` by
  construction). Independently, both shipped Readers return only
  `boundBlob`-validated bytes, so the slice is bounded a second time. KDF
  iterations are bounded 100k–2M before any KDF work. Nothing reachable with
  attacker region bytes panics, over-reads, or (given the §5.2 wording test)
  misleads.
- **Frozen surface untouched — verified.** `git diff b14662a..HEAD -- seal/
  gui/unlock_kdf.go` is empty; no `unlock_*.go` production file is in the
  diff; the compile-time guard `var _ [1]struct{} = [qaProgram -
  unlockPayload]struct{}{}` stands at `gui/gui.go:219`; `loadPayload` was
  inserted **before** `bip85Derive` so `unlockPayload` keeps its conditional
  last place and `lastNav` bounds; the six test edits are carousel-order
  mirrors only.
- The earlier worry that an **unsealed** payload might display the
  weak-passphrase warning (since `pass == ""` ⇒ `weak = true`) is unfounded:
  `syswFlags` requires `sealed && weak` for F2 (`gui/sysw_admit.go:85`).

### Minor

- **M1** `gui/unlock_program_test.go:211` (this diff): condition updated to
  `got != 9` but the message still says `want 8`.
- **M2** `gui/sysw_load.go:55-65` (ParseHeader + truncation branches) are
  **dead code under both shipped Readers** — `FileReader.Read` and
  `XIPReader.Read` both run `boundBlob` → `ParseHeader` internally and error
  first, so every structurally-bad region surfaces as the `r.Read()` branch
  ("Could not read the payload region."). Harmless as defense-in-depth, but it
  silently re-routed the malformed-region test onto a different branch and
  message than the one it appears to target — and it is where a mutant goes to
  not die (see Q2a). Worth a comment, or a conscious decision.
- **M3** `inputWordsFlow` returns the same count for "done" and "back", so once
  C1 is fixed, an operator who types two words and presses **Back** (to abort)
  gets a wrong-passphrase attempt and an error modal instead of an abort. No
  session is created either way; UX only.
- **M4** (pre-existing code, newly reachable via this diff): `syswSession.has`
  deliberately skips the `compared` gate, so after a load-without-compare every
  program still shows the "FROM PAYLOAD" choice; picking it hits `take`'s
  refusal and falls through to manual entry with no explanation
  (`gui/sysw_session.go:84-95,105-115`). The flow's own modal says "no program
  will use it", which is true — but the dangling menu contradicts the comment
  at `sysw_load.go:139-141` ("rather than letting the operator discover it as
  a menu"). Safety holds (E4b measured `take` refusing); the operator
  experience does not match the stated intent.

---

## Q2(a) — F-146: the diagnosis is wrong; the observation was real

**What the entry claims:** reads of a captured `ret` or `ctx.sysw` from the
test goroutine are "unsynchronised and can be stale" because `runUITouch`
drives the flow as an `iter.Pull` coroutine; a mutant survived to prove it;
therefore flow outcomes need a done-channel before they can be asserted.

**What is actually true, measured:**

1. **`iter.Pull` provides the ordering.** Read `iter.go` in the shipped Go
   (1.26.3): `next()` and `stop()` each `coroswitch` into the coroutine and
   **block until it switches back**, with `race.Acquire/Release` around every
   handoff. Strict alternation: the two goroutines never run concurrently.
   Crucially, `stop()` (the harness's `quit`) sets `done`, resumes the
   coroutine, and **returns only after the seq function has run to
   completion** — `yield` returns false, `ctx.Done` is set, the flow unwinds,
   `ui()` returns, and only then does `stop()` come back. A read of `ret` or
   `ctx.sysw` **after `quit()`** is therefore fully ordered after every write
   the flow ever made. All experiment runs were under `-race`: zero reports.

2. **Why the assertions "could not fail", reason one — placement.** Before
   `quit()`, the flow is parked inside the error modal's `yield`; its `return`
   statement **has not executed**. Measured with a poisoned sentinel
   (`ret := true`): after `pumpUntil` finds the modal, `ret` still holds the
   poison — the flow simply hasn't returned. A pre-quit read is a read of
   code that hasn't run, not of memory that isn't visible.

3. **Why the mutant survived, reason two — the mutated branch never executes.**
   The malformed fixture (`MNEMSYSW` + zeros) passes `Probe()` (magic-only)
   but is rejected inside **`FileReader.Read()`** itself (`boundBlob` →
   `ParseHeader` → `ErrVersion`), so the flow exits through the `r.Read()`
   error branch at `sysw_load.go:49-53`. A mutant placed in the flow's own
   ParseHeader branch (`:55-60`) — the natural target for "the malformed-region
   path" — is live in the binary and **never runs**. Measured with a direct
   synchronous call (no coroutine anywhere): mutant present, `ret == false`,
   `ctx.sysw == nil`. "The mutation was verified to have landed" verified
   presence in the build, not execution — the same trap as the entry's own
   first false-survivor, one level down.

4. **The decisive pair.** With the mutant moved to the branch that actually
   fires (`:52`, session built + `return true` after `showError`):
   - existing `TestSyswLoadFlowRefusesAMalformedRegion…`: **PASSES** (gap real);
   - deleted assertions re-added **pre-quit**: **PASS** — mutant survives
     (the author's observation, reproduced);
   - same assertions **post-quit**: **FAIL — mutant killed**;
   - direct-call probe: **FAIL — mutant killed**, no harness involved.

**So: F-146 is misfiled.** Its mechanism ("no happens-before edge") is
contradicted by the implementation and by `-race`; its prescription (a done
channel) already exists — `quit()` *is* the completion wait; and its
conclusion ("no gui flow's return value or context mutation is under test
anywhere" as a *cannot*) is wrong on both counts: the same commit's own
boot-skip test asserts `ret`/`ctx.sysw` after `quit()` soundly, and the
direct-call idiom (`TestRecoverSLIP39BackoutRecognized`, and
`TestSyswLoadFlowIsSilentWithoutAPayload` itself) asserts flow return values
with no goroutine at all. What F-146 should say, if kept: *assert flow
outcomes after `quit()` or via direct synchronous calls; and when mutating,
prove the mutated line executes* (M2's dead branches are where mutants go to
survive). The entry's empirical instinct was right — the observation was real
— but both available explanations were ordinary sequencing, and the
concurrency story does not survive contact with `iter.go`.

## Q2(b) — F-145 is not blocked. All five cases run today; two catch the Critical.

Written this session against the existing harness, no new infrastructure:

| case | idiom | result at HEAD |
| --- | --- | --- |
| malformed region → false, no session | post-quit asserts; also direct call | **passes** (and kills a reachable mutant) |
| truncated region → false, no session | direct call, S-E blob cut short | **passes** |
| unsealed + digest, operator confirms | direct call, S-A, `B3` | **passes** — `compared`, record takeable |
| unsealed + digest, operator declines | direct call, S-A, `B1`,`B3` | **passes** — session, `!compared`, `take` refuses |
| sealed, right passphrase | direct call, S-E, runes+buttons | **FAILS — C1** (hangs in "did not open"); passes with the fix |
| sealed, wrong passphrase | direct call, S-E | **FAILS — C1** (same); passes with the fix |

The direct-call idiom: pre-queue events (`runes(&ctx.Router, "abandon")`,
`click(&ctx.Router, Button3)`, …) and call `syswLoadFlow` on the test
goroutine. It works because `EventRouter.Next` inspects only the queue head
and consumes strictly in order, and `ctx.Frame` no-ops without a callback.
This is the strongest finding available here: **writing the "blocked" tests
found the Critical.** Had the sealed cases been written when F-145 was filed,
C1 never reaches the machine.

---

## What this diff needs before it goes on the machine

1. **C1**: `emptyBIP39Mnemonic(24)` at `gui/sysw_load.go:81` (one line, verified).
2. Land the sealed right/wrong-passphrase tests (they exist in this report,
   appendix) — they are the regression net for C1.
3. Re-file F-146 per Q2(a); unblock and finish F-145's table above.
4. Decide I2 (§8a keyboard choice): implement, or stage it explicitly with the
   ASCII-sealed-payload incompatibility written down.

---

## Appendix — experiments (exact code, now removed from the tree)

All run via `nix develop --command go test ./gui/ -run … -v -race`; tree
restored and full suite re-run green afterward. Vector paths resolve to
`crates/me-cli/testdata/sysw_vectors.json` (S-A plaintext+text record;
S-E sealed under "abandon about", `pub_len 67`).

### The one-line fix, verified

```go
m := emptyBIP39Mnemonic(24) // was: make(bip39.Mnemonic, 24)
```

### Sealed, right passphrase (fails at HEAD → C1; passes with fix)

```go
func TestSyswLoadFlowSealedRightPassphrase(t *testing.T) {
	p := newPlatform()
	p.sysw = syswRegionFor(t, "S-E")
	ctx := NewContext(p)

	runes(&ctx.Router, "abandon")
	click(&ctx.Router, Button3) // accept word 1
	runes(&ctx.Router, "about")
	click(&ctx.Router, Button3) // accept word 2
	click(&ctx.Router, Button2) // done
	click(&ctx.Router, Button3) // digest: confirmed
	click(&ctx.Router, Button3) // warnings (weak passphrase): continue

	ret := syswLoadFlow(ctx, &descriptorTheme, ctx.Platform.SyswReader(), false)
	if !ret {
		t.Fatal("sealed load with the right passphrase returned false")
	}
	if ctx.sysw == nil || !ctx.sysw.compared {
		t.Fatal("[compared] not set by a successful AEAD open")
	}
	if body, ok := ctx.sysw.take(sysw.ClassMnemonic); !ok || body == "" {
		t.Error("the sealed mnemonic record is not takeable after open")
	}
}
```

At HEAD this times out with the flow parked at `sysw_load.go:105`
(`showError("That passphrase did not open this payload")`) — the queued runes
for "about" can never be consumed by a modal, because entry ended after one
word. Wrong-passphrase variant: replace "about" with "zoo", expect
`ret == false`, `ctx.sysw == nil` (passes with fix).

### Unsealed digest, both directions (pass at HEAD)

```go
// confirm:
click(&ctx.Router, Button3)
ret := syswLoadFlow(ctx, &descriptorTheme, ctx.Platform.SyswReader(), false)
// assert ret && ctx.sysw.compared && take(ClassFreeText) ok

// decline:
click(&ctx.Router, Button1) // digest: declined
click(&ctx.Router, Button3) // dismiss "Loaded, but not compared"
ret := syswLoadFlow(ctx, &descriptorTheme, ctx.Platform.SyswReader(), false)
// assert ret && ctx.sysw != nil && !ctx.sysw.compared && take refuses
```

### Truncated region (passes at HEAD)

```go
blob, _ := os.ReadFile(fullRegionPath) // S-E, padded
h, _ := sysw.ParseHeader(blob)
os.WriteFile(cut, blob[:h.TotalLen()-10], 0o600)
// direct call + click(Button3); assert !ret && ctx.sysw == nil
// NOTE: exits via the r.Read() branch — FileReader.boundBlob refuses first (M2).
```

### F-146 mechanism experiments

Mutant v1 (flow's ParseHeader branch, `sysw_load.go:58` area):
`ctx.sysw = &syswSession{loaded: true, compared: true}; return true`.
Direct-call probe with malformed region: **mutant never executes**
(`ret=false, sysw=nil`) — `FileReader.Read()` returns `ErrVersion` first.

Mutant v2 (the reachable `r.Read()` error branch, `sysw_load.go:52`):

| assertion placement | result |
| --- | --- |
| pre-`quit()` (the deleted position) | PASS — mutant survives |
| poisoned `ret := true`, read pre-`quit()` | `ret` still poison — flow had not returned |
| post-`quit()` | **FAIL — mutant killed** (`-race` clean) |
| direct synchronous call | **FAIL — mutant killed** |

`iter.Pull` (go1.26.3 `src/iter/iter.go:261-352`): `stop()` sets `done`,
`coroswitch`es in, and blocks until the seq function returns; every handoff is
bracketed by `race.Release`/`race.Acquire`. Post-`quit()` reads are ordered.
