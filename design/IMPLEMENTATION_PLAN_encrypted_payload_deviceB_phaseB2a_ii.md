> # RETIRED 2026-08-26 — operator ruling. DO NOT BUILD FROM THIS DOCUMENT.
>
> *"I don't think encrypted_payload_deviceB is relevant any more."*
>
> **The post-wipe hang is known and deliberately accepted, not overlooked.**
> `full unlock → wipe → re-enter Sealed Payload` hangs deterministically on real
> hardware (`HARDWARE_RESULT_2026-08-09_phaseB2b.md`), and the phase's commits
> ARE merged into the fork's `main`, so the behaviour is present in flashed
> firmware. It was raised on 2026-08-26 and the operator ruled it closed the same
> day, on the reasoning that **if it is real it will happen again** — a
> deterministic hang does not hide, and it will return with a live
> reproduction attached rather than as an entry in a retired plan.
>
> **Do not re-open it as a new discovery** — the point of this note is that
> the next reader finds a decision rather than a defect.
>
> `DESIGN_b2b_payload_read_allocation.md` remains as the diagnosis
> (`XIPReader.Read`'s 64 KiB allocation, `sysw/read_tinygo.go:31`) should the
> feature ever be revived.

# Encrypted Payload Delivery — Plan B Phase B2a-ii (unlock and the secret session) — Implementation Plan

**Status:** GREEN for its content. **Inherits the R0 loop below; the split is
editorial.**

| round | verdict | report |
| --- | --- | --- |
| 0 | 1C / 4I / 6M / 3N — all folded | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round0.md` |
| 1 | 0C / 2I / 4M / 3N — all folded | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round1.md` |
| 2 | 0C / 1I / 0M / 2N — all folded, **GREEN** | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round2.md` |

**Provenance.** This document and its sibling `…_phaseB2a_i.md` are the two
halves of `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md`, which went
through the three R0 rounds above and closed GREEN at commit `861c99a`. The task
numbering is **unchanged** (its sibling holds Tasks 1–3, this file Tasks 4–9) so
that every finding in those three reports still resolves against a task number.
Nothing in this document changed with the split.

**PREREQUISITE: B2a-i must be merged first.** This phase calls `seal.NewDeriver`
(Task 3), reads `AdmittedRecord`'s labels on encrypted records (Task 1), and
takes the payload region from a `seal.Reader` rather than a retained slice
(Task 2).

---

## Why the seam is where it is

**B2a-i cannot decrypt. This phase is where seed material becomes resident**, and
every task below exists either to put it there or to get it out again. Splitting
there rather than anywhere else in the nine tasks means a reviewer of B2a-i never
has to reason about residency at all, and a reviewer of this document reasons
about nothing else.

**This phase is NOT separable any further.** §10.2.2's session lifecycle ships
with the unlock that creates it: a build that decrypted but did not offer and
wipe secrets would leave seed material resident with no lifecycle, which is
strictly worse than not decrypting. That is `CONTINUITY_2026-08-08.md`'s argument
for the original B2a/B2b seam and it binds inside B2a just as hard.

> **THE FEATURE IS STILL NOT OPERATOR-COMPLETE AFTER THIS PHASE.** §10.2.4's
> residency-keyed idle wipe is B2b's. Do not tag a release when this merges.

---

## Decisions taken before this plan (operator, 2026-08-08)

Recorded here because a reviewer must not re-litigate them, and because each
closed a real fork in the design.

1. **The KDF is chunked, with a real progress bar** (Task 3). `pbkdf2.Key` is one
   blocking call: the frame loop stops for ~31 s and the screen freezes, which is
   exactly the "machine has hung" reading §10.2 step 7 exists to prevent. B2a
   reimplements PBKDF2's single-block loop over `crypto/hmac` so it can run *N*
   iterations per frame. It also lets the passphrase arrive as `[]byte` and be
   zeroed, which `Unlock(… passphrase string)` makes impossible.
2. **F-79 is fixed by retaining nothing** (Task 2): an 8-byte magic probe at GUI
   start, the region read on flow entry, `clear` on exit. Not the reslice
   variant.
3. **F-80's B2 items split**: the "already cut this session" marks and the
   Back-is-Lock affordance land in **B2a** (Task 7); the `layoutMainPager`
   pixel pin does not — it needs a rasterising check, which is F-78's work.
4. **§10.2.2's wipe-on-any-exit stands as written.** A cancelled secret plate
   costs twelve words and a ~31 s KDF to retry. It costs no reboot and destroys
   nothing: the sealed blob is untouched in flash, and "wipe" means zeroing the
   decrypted copy in RAM.
5. **A bare BIP-39 mnemonic record engraves by composing directly** (Task 6) —
   `SeedScreen.Confirm` → `masterFingerprintFor(m, mainnet, "")` → `engraveSeed`
   → `NewEngraveScreen` — **not** by reusing `backupWalletFlow`, whose
   `for { … if Engrave { return } }` loop re-presents a cancelled plate and so
   contradicts §10.2.2 directly.

---
## Global Constraints

Phase A's and B1's global constraints carry forward **unchanged** and are not
repeated in full. The load-bearing ones for B2a:

- **All Go work runs under `nix develop --command …`.** `nix` is NOT on `PATH` —
  use `/nix/var/nix/profiles/default/bin/nix`.
- **`go.mod` says `go 1.25.10`; TinyGo is 0.41.1.** The host `go` in the dev
  shell is not the firmware compiler. A screen that builds on the host can still
  fail under TinyGo.
- **Stage paths explicitly. Never `git add -A`.**
- **Never a bare `go test ./... -update`.** Scope with `-run`, then `git status`.

### The green criterion, measured — do not restate it as "clean"

Measured at `78949e7` on a clean tree. Each of these is **already true before
B2a starts**, so "it went red" means B2a broke it:

| command | the honest expectation |
| --- | --- |
| `CGO_ENABLED=0 go test ./...` | **exit 1**, with exactly TWO `[setup failed]` lines — `cmd/kdfbench` and `cmd/sealread`, both `import "machine"` and host-unbuildable. A third failure is a regression. |
| `CGO_ENABLED=0 go vet ./seal/` | clean. |
| `CGO_ENABLED=0 go vet ./gui/` | **exit 1**, one pre-existing diagnostic: `gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)`. Any *other* diagnostic is B2a's. |
| `gofmt -l <files B2a touched>` | empty. Six unrelated files are already unformatted (`gui/bip85_test.go`, `gui/md1_expand_fuzz_test.go`, `gui/multisig_build_test.go`, `gui/multisig_match.go`, `gui/multisig_testhelpers_test.go`, `md/template_guard_test.go`); they are not B2a's to fix. |
| TinyGo device build | `nix develop --command tinygo build -size full -print-stacks -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` |

> **`gofmt -l` reports by PRINTING and exits 0 either way.** `gofmt -l x && echo
> clean` always prints "clean". Test the output: `out=$(gofmt -l …); [ -z "$out" ]`.
> This repo has been burned by that exact overclaim (commit `cee1598` names it).

### B2a-specific

- **`gui` gains no new third-party dependency.** `crypto/hmac` and `crypto/sha256`
  are already linked (`bip85/bip85.go:20`, `slip39/combine.go:143`); `crypto/aes`
  and `crypto/cipher` were pulled in by Phase A's `seal/crypto.go` at a measured
  ~1.6 KB. Task 3 adds no primitive that is not already resident.
- **The chunked path's KDF seam is `newDeriver`, added in §5c (Task 5)** — not
  `Opener.KDF`, and not Task 3. B2a derives through `seal.NewDeriver` and opens
  through `UnlockWithKey`, neither of which consults `Opener.KDF`, so the
  existing `countingKDF` instrument is no longer in the device path at all.
  *(An earlier draft of this bullet said Task 3 added it, and paired that with
  "one real 100,000-iteration derivation would nearly triple the ~12 s gui
  suite". Both were wrong — the ~31 s of §7.1 is a DEVICE figure. Measured on
  this host rather than argued: `DeriveKey` at 100,000 iterations is **13.1 ms**
  and at 300,000 is **35.5 ms**; the chunked deriver covers 100,000 in
  **10.2 ms across 200 `Step` calls, drawing 199 frames**. So the gui cost is
  the frame pumping, not the KDF, and a test may safely use the §6.2 floor.
  R0 round 0 I2, round 1 M1.)*
- **Touch, not buttons, for anything with a keyboard.** `gui/start_screen_touch_test.go:11-23`
  and `gui/passphrase_flow_test.go:19-33` are binding: the SH2 has no directional
  buttons, tests must lay out at `sh2DisplaySize` (`gui/gui_test.go:390`), and a
  240×240 default pushes keyboard keys off-canvas.

---

## Carried-forward citations that have DRIFTED — corrected here, once

Machine-resolved during recon. Every one of these appears in the SPEC, the
continuity doc, or B1's plan at a **stale** line. Use the corrected value; do not
re-derive it, and do not "fix" the shipped records (per F-72's precedent they are
annotated, not rewritten).

| cited as | actually | cited by |
| --- | --- | --- |
| `gui/gui.go:2801` `idleTimeout` | **`gui/gui.go:2879`** | SPEC §10.2.4, CONTINUITY §5 |
| `cmd/controller/platform_sh2.go:368` `AppendEvents` | **`:369`** | CONTINUITY §5, B1 plan |
| `gui/passphrase_flow.go:605` `wipeBytes` | defined at **`gui/slip39_polish.go:342`**; `:605` is a *call site* | SPEC §10.2 step 10 |
| `gui/gui.go:968` `NewKeyboard` | **`gui/gui.go:983`**; `:968` is a struct field | SPEC §8 |
| `cmd/controller/platform_sh2.go:564` `PayloadReader` | **`:573`** | B1 plan |
| `cmd/emu/platform.go:189` `PayloadReader` | **`:203`** | B1 plan |
| `seal/record.go:186` "pass 3 runs for SectionPublic alone" | the `if section == SectionPublic` is at **`seal/record.go:214`** | F-77, B1 plan, CONTINUITY §4 |
| `gui/gui.go:1553` the `unlockPayload` dispatch case | **`gui/gui.go:1595`** | B1 plan |
| `gui/gui.go:1726` the `unlockPayload` title case | **`gui/gui.go:1791`** | B1 plan |

Also corrected, because the plan below depends on it: **SPEC §8 says the device
"reuses the existing 12-word seed-entry flow unmodified".** `seedEntryFlow`
(`gui/derive_xpub.go:82`) unmodified shows a **12/24 word-count picker** first,
which B2a must not show — §8's own text says the passphrase is twelve words. What
is reusable unmodified is one level down: `inputWordsFlow` (`gui/gui.go:641`),
whose length comes entirely from `len(mnemonic)`. Task 4 uses that.

---
## The record is wiped BEFORE the engrave, not after it (R0 round 0, C1)

**An earlier draft of this plan held the secret record across the engrave and
wiped it when `EngraveScreen.Engrave` returned, calling that "the one place this
plan interprets the spec". Both the reading and its stated justification were
wrong, and this section records why so it is not reintroduced.**

**The record is not needed once the plate exists.** `NewEngraveScreen(ctx, plate)`
builds `newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0)`
(`gui/gui.go:2633`), and the engrave loop iterates `e.spline`
(`gui/engraver.go:170`). **Nothing reads the record bytes after `toPlate` /
`engraveSeed` returns.** So a retry, a resume, and a re-cut all replay the
spline — zeroing the record before `Engrave` is entered costs nothing and
requires no change to `EngraveScreen`. The earlier draft's claim that this "makes
the retry prompt a lie, because the record it offers to re-cut is already zeroed"
was simply false.

**And waiting for `Engrave` to return does not satisfy §10.2.2 anyway.** Back
while the job is running does **not** exit the screen — `gui/gui.go:2651-2656`:

```go
		for backBtn.Clicked(ctx) {
			st := s.job.Status()
			if st.State != engraveRunning {
				break frames
			}
			s.job.Stop()
		}
```

It calls `Stop()` and keeps rendering; the state settles to `engraveStopped` and
the screen offers to resume. Only a *second* Back returns. So under the earlier
reading, the abort-mid-plate path — which §10.2.2 names explicitly as "the
machine's most ordinary recovery" — left the decrypted seed resident and let the
operator resume **without a fresh unlock**, which is the exact opposite of the
price §10.2.2 says is deliberate.

**Therefore: `clear(rec)` the moment the plate is built.** §10.2.2 is implemented,
not interpreted. **The record being cut** stops being resident for the duration of
its own ~21-minute plate, and for the unbounded time a paused or failed screen
can sit there. `unlockSecretPlate`'s `defer` stays as the backstop for every path
that never reaches a plate at all, and becomes idempotent.

**Be precise about what this does NOT collapse** (R0 round 1, M2). An earlier
draft of this paragraph claimed residency falls to "the few milliseconds between
decrypt and plate construction" and that "`SecretsResident()` now goes false as
the cut starts". **Both are true only for a single-secret payload.**
`unlockSecretSession` offers secrets one at a time and wipes each as its plate
leaves, so while plate 1 of 3 is cutting, records 2 and 3 are **untouched and
resident** — and `SecretsResident` scans all of them. That is exactly what
§10.2.2's cost paragraph already says: "~21 minutes for single-sig, **~63 for a
2-of-3**." Vectors F and G each carry three `ms1` records, so this is the
ordinary multisig case and not an edge one.

The accurate statement, and the one B2b must design its timer against:
**`SecretsResident()` goes false when the LAST secret's plate is built.** What
the early wipe removes is one plate's worth of residency at a time — the record
currently being cut — not the session's.

**Do not overclaim what this buys.** The `Plate` still encodes the secret — it is
a geometric rendering of the very words about to be cut into steel, and it must
exist for the duration of the cut. An SWD reader with the machine open during an
engrave can reconstruct the seed from `plate.Spline` whether or not the record
buffer is zeroed. What the early wipe removes is the *record*, the *only* copy
that outlives the plate and the one §10.2.2 names. The unwipeable derived copies
are **F-83**, and they are **not a defect awaiting a fix**: the operator accepted
them as unavoidable on 2026-08-08. The plate must exist in RAM for as long as the
needle is moving, and a plate pipeline over `[]byte` would relocate the secret
rather than remove it, because the spline still encodes it. See F-83 in the
follow-up register for what that means for the threat model, and F-85 for the
SPEC amendment it owes.

**Consequence for the retry story, stated so Task 9.6 tests the truth:** because
the job holds the spline, an operator who pauses mid-cut **can resume the same
plate without re-entering the passphrase**. What needs a fresh unlock is
re-cutting after *leaving* the engrave screen, when the plate is gone too. That
is the honest reading of §10.2.2's "re-cutting needs a fresh unlock", and it is
strictly better than the alternative: the pause/resume path never had a seed
record resident to protect.

---
## Task 4 — the passphrase: twelve words, and the checksum gate BEFORE the KDF

### 4a. Twelve words, not a 12/24 picker

`seedEntryFlow` (`gui/derive_xpub.go:82`) shows `"12 WORDS" / "24 WORDS"` first,
and §8 says the passphrase **is** twelve words — there is no choice to offer.
What is reusable unmodified is `inputWordsFlow` (`gui/gui.go:641`), whose length
is `len(mnemonic)` and whose `title` parameter is a documented additive seam
(`gui/gui.go:762-764`: `""` renders `"Word %d of %d"`, non-empty replaces it).

### 4b. The gate is `isMnemonicComplete(m) && m.Valid()`, and BOTH halves matter

Measured, not assumed:

- `inputWordsFlow` returns on **Back** at `gui/gui.go:692-694` with whatever has
  been typed. `seedEntryFlow` only re-prompts when *every* word is still `-1`. So
  typing one word and pressing Back yields an 11×`-1` mnemonic that a caller can
  mistake for a completed one.
- On that partial, `Valid()` returns **false** (no panic) and `String()` returns
  `"abandon           "`, which `seal.NormalisePassphrase` silently repairs to
  `"abandon"` — a well-formed, wrong, one-word KDF input, ~31 s of waiting, and a
  tag failure indistinguishable from a wrong passphrase.
- `Entropy()` (`bip39/bip39.go:158`) **panics** on an invalid mnemonic, and a
  panic on the device is a brick. B2a never calls it.

Every existing consumer already gates this way — `gui/singlesig_derive.go:40`,
`gui/multisig_derive.go:33`, `gui/seedxor_polish.go:56`, `gui/gui.go:2350`.

### 4c. The passphrase is a `[]byte` the caller can zero

`Mnemonic.String()` (`bip39/bip39.go:166`) produces exactly §8.1's form — measured:
`seal.NormalisePassphrase(m.String()) == m.String()` for a complete 12-word
mnemonic — but it produces a **Go string**, which cannot be zeroed. B2a builds
the same bytes into a buffer it owns, preallocated so `append` never regrows and
leaves a stale partial copy behind.

**Steps:**

- [ ] **4.1** Write the entry and gate as part of Task 5's new file (they are one
      unit: the gate exists to protect the KDF and is meaningless apart from it).
- [ ] **4.2** Tests are Task 5's — the gate's discriminating case
      (`beef`×11 + `bacon`) cannot be typed on the keyboard, because
      `LastWordCandidates` restricts the final slot to the 128 checksum-valid
      words. It is driven through the seam Task 5 defines.

---

## Task 5 — unlock: the progress screen, the AEAD open, and the retry loop

### 5a. `seal` gains an entry point that takes a KEY, not a passphrase

`Opener.Unlock` (`seal/open.go:176`) derives and opens in one call, so a caller
that wants to show progress cannot get between the two. B2a splits the second
half out. **The split is a refactor, not a fork:** `Unlock` is rewritten to call
it, so there remains exactly one implementation of "open, split, allow-list".

`seal/unlock_key.go`, new file.

```go
package seal

import (
	"errors"
	"fmt"
)

// ErrNotSealed is UnlockWithKey called on a payload that carries no encrypted
// section. It is a programming error, not an operator-visible condition: §10.2
// step 4 stops before the passphrase when ct_len == 0, so reaching here means a
// caller skipped that check.
var ErrNotSealed = errors.New("seal: payload carries no encrypted section")

// UnlockWithKey is §10.2 steps 8-9 against a key the caller already derived.
//
// It exists because §10.2 step 7 requires a progress indicator over a ~31 s
// derivation, and a single call that derives and opens leaves no seam to draw
// a frame in. Unlock is now this function plus the derivation, so there is one
// implementation of the open-split-allow-list pipeline and not two.
//
// On failure p is left INTACT — Header, Public and Hash stay valid — which is
// what lets Phase B keep the §6.6 hash on screen through the retry loop
// (§10.2 step 8).
//
// The key is the caller's to wipe. This function neither zeroes nor retains it.
func (o Opener) UnlockWithKey(blob []byte, p *Payload, key []byte) error {
	h := p.Header
	if !h.Sealed() {
		return ErrNotSealed
	}
	end := HeaderLen + int(h.PubLen) + int(h.CtLen) + TagLen
	split := HeaderLen + int(h.PubLen)
	// The offsets come from p.Header, which came from a DIFFERENT call
	// (Inspect). Nothing forces the caller to hand back the same blob, so
	// bound-check the one actually passed before slicing it: on a device a
	// panic is a brick.
	if len(blob) < end {
		return fmt.Errorf("%w: region holds %d bytes, the header declares %d",
			ErrTooShort, len(blob), end)
	}
	// AAD = header || public section (§6.1a), taken from the blob's own bytes,
	// so it binds version, algorithm ids, iteration count, salt, IV, both
	// lengths AND every public record.
	plaintext, err := Open(key, h.IV[:], blob[:split], blob[split:end])
	if err != nil {
		// Fail closed. ErrAuthentication, and Phase B must offer BOTH readings.
		return err
	}
	// The plaintext buffer is ours; the records copied out of it are wiped by
	// Payload.Wipe, which Phase B owns.
	defer clear(plaintext)
	recs, nSec, err := SplitSection(plaintext)
	if err != nil {
		return describeRecordCount(err, p.nPub, nSec)
	}
	// §6.4's 1..24 cap is over the TOTAL across BOTH sections, which
	// SplitSection cannot see — this is the only place the cross-section total
	// is known.
	if total := p.nPub + nSec; total > MaxRecords {
		return recordCountError(total, p.nPub, nSec)
	}
	admitted, err := AdmitSection(recs, SectionEncrypted)
	if err != nil {
		return err
	}
	// Wipe any secrets a PREVIOUS unlock left here before dropping the
	// reference. Overwriting p.Secret makes those bytes unreachable, so Phase B
	// calling p.Wipe() faithfully at session end would still miss them.
	for _, r := range p.Secret {
		clear(r.Record)
	}
	p.Secret = admitted
	return nil
}
```

Then modify `seal/open.go` — `Unlock`'s body from its `derive := o.KDF` line
(`seal/open.go:205`) to the end is replaced by:

```go
	derive := o.KDF
	if derive == nil {
		derive = DeriveKey
	}
	key := derive(NormalisePassphrase(passphrase), h.Salt[:], int(h.Iterations))
	// §10.2 step 10: wipe the derived key on EVERY exit path.
	defer clear(key)
	return o.UnlockWithKey(blob, p, key)
```

with the now-unused `nPub` and `split` locals removed from `Unlock` (they moved
into `UnlockWithKey`). **`end` stays**, because its `len(blob) < end` guard stays
— an earlier draft said to remove all three and then kept the guard, which does
not compile (R0 round 0, M1). The guard is now redundant with `UnlockWithKey`'s,
deliberately: `Unlock` remains a public entry point and bound-checks its own
argument. The `if !h.Sealed() { return nil }` early return also stays, because
`Unlock`'s contract is "a payload with nothing encrypted is not an error" while
`UnlockWithKey`'s is "you gave me a key for a payload with no ciphertext".

### 5b. `seal` gains the session predicates §10.2.2 and §10.2.4 both need

`seal/session.go`, new file.

```go
package seal

// §10.2.2's session lifecycle and §10.2.4's residency timer both need one
// question answered the same way: which records are SECRET, and is any of them
// still in memory. Answering it here rather than in gui is the same rule F-77
// enforces for card grouping — the UI must not re-derive what the classifier
// already decided.

// IsSecret reports whether a classification is seed material.
//
// md1 and mk1 are NOT secret wherever they travelled: §6.3's table is explicit
// that an xpub and a wallet policy leak privacy but do not spend coins, and
// §11.2 requires vector F's THREE ms1 records to be the ones offered first —
// its twelve mk1/md1 records are ordinary plates. Encrypting them is defence in
// depth, not protection of key material.
func IsSecret(c Classification) bool {
	return c == ClassCodex32Secret || c == ClassMnemonic
}

// SecretsResident reports whether any secret record still holds non-zero bytes.
//
// This is §10.2.4's timer condition, and it is keyed on RESIDENCY rather than
// on which button was last pressed — which is what makes an aborted engrave
// safe: cancel a secret plate mid-cut, §10.2.2 wipes the record, and this goes
// false because the secret is ACTUALLY GONE, not because a button was pressed.
//
// B2a has no timer (that is B2b), but the predicate ships here because it is
// the definition the wipe must satisfy, and a test can assert on it.
func (p *Payload) SecretsResident() bool {
	for _, r := range p.Secret {
		if !IsSecret(r.Class) {
			continue
		}
		for _, b := range r.Record {
			if b != 0 {
				return true
			}
		}
	}
	return false
}

// WipeSecretAt zeroes one record's bytes. §10.2.2 wipes per RECORD as each
// plate leaves the screen, not per session, so Payload.Wipe is too coarse for
// the offer loop — it is the right thing only on the way out.
//
// Out-of-range is a no-op rather than a panic: on a device a panic is a brick.
func (p *Payload) WipeSecretAt(i int) {
	if i < 0 || i >= len(p.Secret) {
		return
	}
	clear(p.Secret[i].Record)
}
```

### 5c. The `gui` side

`gui/unlock_kdf.go`, new file.

```go
package gui

import (
	"errors"
	"fmt"
	"image"
	"log"
	"time"

	"seedhammer.com/bip39"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/seal"
)

// §10.2 steps 5-8: twelve words, the checksum gate, the ~31 s KDF with a real
// progress indicator, and the retry loop that keeps the §6.6 hash on screen.

// kdfStepIterations is how much of the derivation runs between two frames.
//
// §7.1 measured 9,715 iterations/sec on RP2350 silicon, so 500 iterations is
// ~51 ms of work per frame -- about 19 fps, which reads as motion, and well
// under the ~250 ms at which a touch starts to feel ignored. At the 300,000
// default that is 600 frames.
const kdfStepIterations = 500

var (
	// errUnlockChecksum is §10.2 step 6: the words are not a valid BIP-39
	// mnemonic, so NO KDF is run. Distinct from a tag failure because the
	// operator's next action differs -- retype, versus suspect the payload.
	errUnlockChecksum = errors.New("unlock: passphrase is not checksum-valid")
	// errUnlockCancelled is the operator leaving, at any point.
	errUnlockCancelled = errors.New("unlock: cancelled")
)

// newDeriver is the KDF seam, and it is NOT optional (R0 round 0, I2).
//
// §11.2 requires "BIP-39 checksum rejection happens without invoking the KDF"
// and §11.3 makes both the "checksum check removed" and "KDF run before the
// checksum gate" mutants mandatory -- each asserted by INSTRUMENTATION, because
// both orders return the identical error and a return-value assertion is a
// guaranteed false PASS over exactly the defect.
//
// The existing instrument (Opener.KDF, counted by countingKDF in
// seal/open_test.go:14-20) is no longer in the path: B2a derives through
// seal.NewDeriver and opens through UnlockWithKey, neither of which consults
// Opener.KDF. This variable is the replacement, in the same in-file style as
// unlockEngraveHook and unlockSecretHook. Production is always seal.NewDeriver;
// a test swaps it and counts.
var newDeriver = seal.NewDeriver

// unlockPassphraseHook fires when the word-entry screen is ENTERED, and exists
// for one required negative: §11.2's "Vector E reaches the plate list with the
// keyboard flow NEVER ENTERED -- asserted by instrumenting the prompt entry
// point, not by return value. A scripted fake platform will happily feed twelve
// words into a prompt that should not exist and still reach the plate list, so
// a return-value assertion reports PASS over exactly the defect." nil in
// production.
var unlockPassphraseHook func()

// unlockPassphraseFlow takes §8's twelve words. It returns ok == false only
// when the operator backs out; a checksum-invalid entry is reported and
// re-prompted here, because that is a typo and not a decision.
//
// It does NOT reuse seedEntryFlow, which opens with a 12/24 word-count picker.
// §8 says the passphrase is twelve words; there is no choice to offer. What it
// does reuse unmodified is inputWordsFlow, whose length is len(mnemonic) and
// whose title parameter is a documented additive seam.
func unlockPassphraseFlow(ctx *Context, th *Colors) (bip39.Mnemonic, bool) {
	if unlockPassphraseHook != nil {
		unlockPassphraseHook()
	}
	// The screen's identity is established HERE, before entry, and the title
	// passed to inputWordsFlow stays "" (R0 round 0, M4). The title parameter is
	// an either/or: gui/gui.go:765-770 renders `layoutTitlef("Word %d of %d")`
	// when it is empty and `layoutTitle(title)` when it is not. Passing
	// "Passphrase" would REPLACE the only per-word progress on the screen, so
	// the operator would type twelve words with no idea how many remain, on the
	// screen that gates a ~31 s KDF. Empty is also what makes §8's "reuses the
	// existing 12-word seed-entry flow unmodified" literally true, and it keeps
	// the existing `uiContains(content, "Word 1 of")` negative assertion working.
	showNotice(ctx, th, unlockTitle,
		"Enter the 12-word passphrase for this payload.\n\n"+
			"These words are the payload's passphrase. They are NOT a seed and no "+
			"wallet is derived from them.")
	for !ctx.Done {
		m := emptyBIP39Mnemonic(12)
		inputWordsFlow(ctx, th, m, 0, "")
		// inputWordsFlow returns on Back with whatever has been typed, so an
		// incomplete mnemonic is the ordinary shape of "the operator left".
		// Treating it as an error would report a typo they did not make.
		if !isMnemonicComplete(m) {
			clear(m)
			return nil, false
		}
		if !m.Valid() {
			// Structurally hard to reach: LastWordCandidates restricts the
			// final slot to the 128 checksum-valid words. Not impossible, and
			// the cost of being wrong is a 31 s wait ending in a message that
			// blames the payload.
			clear(m)
			showError(ctx, th, unlockTitle, "Not a valid passphrase, check the words.")
			continue
		}
		return m, true
	}
	return nil, false
}

// passphraseBytes builds §8.1's normalised form -- twelve lowercase words,
// single-space separated, no trailing space -- into a buffer the CALLER owns
// and can zero.
//
// Mnemonic.String() produces byte-identical output (measured:
// seal.NormalisePassphrase(m.String()) == m.String()), but it produces a Go
// STRING, which cannot be zeroed. That is the whole reason this exists.
//
// The capacity is fixed so append never regrows: a regrow would leave a stale
// copy of the first half of the passphrase in an orphaned array that nothing
// can reach to wipe. Twelve words of at most eight letters plus eleven
// separators is 107 bytes.
func passphraseBytes(m bip39.Mnemonic) []byte {
	b := make([]byte, 0, 128)
	for i, w := range m {
		if i > 0 {
			b = append(b, ' ')
		}
		start := len(b)
		b = append(b, bip39.LabelFor(w)...)
		// The wordlist is stored uppercase; lowercase in place rather than via
		// bytes.ToLower, which would allocate a temporary per word.
		for j := start; j < len(b); j++ {
			if c := b[j]; c >= 'A' && c <= 'Z' {
				b[j] = c + ('a' - 'A')
			}
		}
	}
	return b
}

// unlockKDFLead is §10.2 step 7's "the screen must say how long it will take".
//
// The estimate is measured from THIS derivation rather than read off §7.1's
// table, so it stays right on a part whose rate differs from the RP2350A the
// table was measured on -- which is precisely the residual caveat §7.1 still
// owes.
func unlockKDFLead(done, total int, elapsed time.Duration) string {
	if done <= 0 || elapsed <= 0 {
		return "Unlocking. This takes about 30 seconds."
	}
	// Multiply BEFORE dividing: `elapsed/done` truncates to whole nanoseconds
	// first, and on a fast host that rounds to 0 and the screen reads "About 0
	// seconds left." int64 overflows only past ~10^10 ns of elapsed time.
	left := time.Duration(int64(elapsed) * int64(total-done) / int64(done))
	return fmt.Sprintf("Unlocking. About %d seconds left.", int(left.Seconds()+0.5))
}

// unlockDerive runs the KDF a slice at a time, drawing a frame between slices.
// It returns the derived key, which the CALLER must zero, or ok == false if the
// operator left.
func unlockDerive(ctx *Context, th *Colors, h seal.Header, pass []byte) ([]byte, bool) {
	d := newDeriver(pass, h.Salt[:], int(h.Iterations))
	// Registered before anything can return. Key() hands back a copy, so this
	// does not zero the result out from under the caller.
	defer d.Wipe()
	backBtn := &Clickable{Button: Button1}
	start := time.Now()
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return nil, false
		}
		if d.Step(kdfStepIterations) {
			// §7.1 still owes an in-situ rate on RP2350B silicon. Logging it
			// here makes the operator's real unlock that measurement, in the
			// real call path, rather than a benchmark's idealised loop. The
			// iteration count travels in the header and is public, so this
			// leaks nothing.
			log.Printf("seal: kdf %d iterations in %s", d.Total(), time.Since(start))
			return d.Key(), true
		}
		dims := ctx.Platform.DisplaySize()
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, unlockTitle)
		pctOp, pctSz := widget.Label(&ctx.B, ctx.Styles.progress, th.Text,
			fmt.Sprintf("%d%%", d.Done()*100/d.Total()))
		leadOp, leadSz := widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text,
			unlockKDFLead(d.Done(), d.Total(), time.Since(start)))
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconDiscard},
		}...)
		// BEFORE ctx.Frame, and the order is LOAD-BEARING. ctx.Frame IS the
		// yield, and Run reads the deadline for the frame it has just been
		// handed (`wakeup := ctx.Wakeup`, gui/gui.go:2972) BEFORE its own
		// ctx.Reset(). A WakeupAt placed AFTER Frame governs the NEXT frame, so
		// frame 1 inherits what the preceding screen left -- Run's own
		// ctx.WakeupAt(idleWakeup), i.e. THREE MINUTES. The derivation parks at
		// 500/300,000 iterations and the screensaver takes the screen.
		//
		// AN EARLIER DRAFT OF THIS PLAN HAD IT AFTER, AND THAT WAS A CRITICAL --
		// worse than the blocking pbkdf2.Key it replaced, which at least
		// finished in ~31 s, and it would have counted the park as derivation
		// time in Task 9.3's §7.1 measurement (~1,400 it/s instead of ~9,715).
		// EngraveScreen.Engrave has the correct order (gui/gui.go:2733 before
		// :2741); match it. A frame-COUNT assertion cannot see this -- the count
		// is 199 either way -- so pin the DEADLINE.
		ctx.WakeupAt(time.Now())
		ctx.Frame(op.Layer(
			nav,
			titleOp,
			pctOp.Offset(image.Pt((dims.X-pctSz.X)/2, (dims.Y-pctSz.Y)/2-leadSz.Y)),
			leadOp.Offset(image.Pt((dims.X-leadSz.X)/2, (dims.Y+pctSz.Y)/2)),
			op.Color(&ctx.B, th.Background),
		))
	}
	return nil, false
}

// unlockAttemptOnce is §10.2 steps 6-9 for ONE attempt.
//
// It takes the mnemonic rather than reading it, which is the only way to test
// the checksum gate: LastWordCandidates restricts the keyboard's final slot to
// the 128 checksum-valid words, so a test cannot TYPE `beef` x11 + `bacon`.
// §11.4 requires that case to be rejected with no KDF run, asserted by
// instrumenting the KDF and not by return value.
func unlockAttemptOnce(ctx *Context, th *Colors, blob []byte, p *seal.Payload, m bip39.Mnemonic) error {
	// §10.2 step 6, BEFORE the ~31 s KDF. Both halves matter: Valid() alone is
	// false on a partial mnemonic too, but NormalisePassphrase silently repairs
	// a partial into a well-formed WRONG passphrase, and the operator then
	// waits 31 s for a message that blames the payload.
	if !isMnemonicComplete(m) || !m.Valid() {
		return errUnlockChecksum
	}
	pass := passphraseBytes(m)
	defer clear(pass)
	key, ok := unlockDerive(ctx, th, p.Header, pass)
	if !ok {
		return errUnlockCancelled
	}
	// §10.2 step 10: the derived key is zeroed on every exit path.
	defer clear(key)
	var o seal.Opener
	return o.UnlockWithKey(blob, p, key)
}

// unlockRetryBody is §10.2 step 8's message. It MUST offer both readings and
// keep the §6.6 hash on screen: the tag also authenticates the public section
// (§6.1a), so a tampered public card fails here too -- ~31 s after the hash was
// displayed. Reporting only "wrong passphrase" invites the operator to retype
// three times and conclude the blob is corrupt, losing the one signal §2.2
// item 4 exists to raise.
func unlockRetryBody(p *seal.Payload) string {
	msg := "Wrong passphrase, or this payload has been altered.\n\n"
	if !p.HasHash {
		// pub_len == 0: there is no hash to compare, and inventing one would be
		// the empty-record-set constant -- furniture, per §10.2 step 3.
		return msg + "Check the words and try again."
	}
	return msg + fmt.Sprintf("Public data hash (%d records, %s):\n\n%s\n\n"+
		"Compare this against the value you recorded.",
		len(p.Public), unlockShape(p), seal.FormatHash(p.Hash))
}

// unlockSealedFlow is §10.2 steps 5-9 with the retry loop. It returns true only
// when p.Secret has been populated.
//
// A false return MUST NOT fall through to the plate list: p.Public on a sealed
// payload is a legitimate record set, and engraving it while dropping the
// encrypted half is §6.4's incomplete-backup-believed-complete, the worst
// available outcome. That is the same rule B1's Task 6 enforced with a terminal
// screen, and it does not relax now that unlocking exists.
func unlockSealedFlow(ctx *Context, th *Colors, blob []byte, p *seal.Payload) bool {
	for !ctx.Done {
		m, ok := unlockPassphraseFlow(ctx, th)
		if !ok {
			return false
		}
		err := unlockAttemptOnce(ctx, th, blob, p, m)
		// The mnemonic is []Word, so clear() reaches it; wipeBytes takes []byte
		// and does not compile against it.
		clear(m)
		switch {
		case err == nil:
			return true
		case errors.Is(err, errUnlockCancelled):
			return false
		case errors.Is(err, errUnlockChecksum):
			showError(ctx, th, unlockTitle, "Not a valid passphrase, check the words.")
		case errors.Is(err, seal.ErrAuthentication):
			showError(ctx, th, unlockTitle, unlockRetryBody(p))
		case errors.Is(err, seal.ErrTooManyRecords):
			// §6.4 requires this be distinguishable from "unreadable": the
			// count is authenticated plaintext, so naming it leaks nothing, and
			// conflating a too-large wallet with an attack would send the
			// operator chasing a compromise that did not happen.
			showError(ctx, th, unlockTitle,
				"This payload declares more records than the machine accepts.")
			return false
		default:
			showError(ctx, th, unlockTitle, "Payload unreadable.")
			return false
		}
	}
	return false
}
```

### 5d. Wiring, in `gui/unlock_flow.go`

Fragment replacing the sealed dead-end (`gui/unlock_flow.go:61-66`):

```go
	// §10.2 steps 5-9. On failure or cancellation this returns WITHOUT reaching
	// the plate list -- see unlockSealedFlow's contract.
	if p.Header.Sealed() {
		if !unlockSealedFlow(ctx, th, blob, p) {
			return
		}
		// F-79: the blob's last use was UnlockWithKey's AAD and ciphertext. Zero
		// and drop it BEFORE the session, so the engrave that follows does not
		// run with the region still on the heap. This releases it only because
		// the deferred clear above is a CLOSURE reading `blob` at exit; a
		// `defer clear(blob)` would still hold the array (I1).
		//
		// Safe because AdmitSection COPIES every record
		// (seal/record.go:207, `append([]byte(nil), r...)`), so p.Public and
		// p.Secret do not alias this buffer.
		clear(blob)
		blob = nil
		// §10.2.2 -- every secret record offered FIRST, consecutively, each
		// wiped as its plate leaves the screen by any route.
		unlockSecretSession(ctx, th, p)
		unlockPlateListFlow(ctx, th, unlockPlates(p))
		return
	}
```

and a `defer p.Wipe()` immediately after `Inspect` succeeds, so **every** exit
from the flow — Back, an error, `ctx.Done`, a panic unwind — zeroes every record
in both sections (§10.2.2: "Lock, Back, an error, `ctx.Done`" are one exit, not
four).

### 5e. Test-infrastructure work this task owns

- **`sealTestVector` gains `passphrase` and `iterations`** (`gui/unlock_program_test.go:37-46`).
  Without them a gui test must retype `"beef beef …"`, which
  `seal/testdata/README.md:19-24` forbids.
- **`runUnlock` gains a touch-driven sibling** — the passphrase keyboard needs
  `runUITouch` and `sh2DisplaySize`, per the binding rule in Global Constraints.
  `ppHarness` (`gui/passphrase_flow_test.go:39`) is the closest existing shape,
  but it hooks `passphraseWidgetHook`, which the *word* keyboard does not use;
  clone the harness rather than bending it.
- **`runUnlock` keeps its `[]byte` parameter** and writes the bytes to
  `t.TempDir()`, handing `unlockPayloadFlow` a `seal.FileReader` over the file.
  An earlier draft said Task 2's `seal.Reader` change was "one line per call
  site" using `payloadReaderFor` — it is not: two existing call sites pass blobs
  no vector name can produce (`gui/unlock_flow_test.go:171-172`,
  `tc.mangle(sealVectorBlob(t, "E"))`, and `:195`). Writing a temp file inside
  `runUnlock` makes it genuinely one edit (R0 round 0, N3).
- **`gui` needs its own sealer, because `sealForTest` is unreachable from it**
  (R0 round 0, I3). `sealForTest` is unexported in `seal/open_test.go`, and a
  `_test.go` file is not part of the importable package — no import makes it
  visible to `package gui`. `gui`'s only blob source today is
  `payloadReaderFor`/`sealVectorBlob` over the vector file.

  **And no vector can supply Task 7's fixture.** Measured: A(pub 0/sec 1),
  B(0/1), C(0/6), D(5/1), E(5/0), F(0/15), G(12/3) — C's and F's encrypted cards
  sit on `pub_len == 0` payloads, and D's and G's encrypted records are `ms1`
  only. **No vector carries `md1`/`mk1` in BOTH sections**, so `unlockPlates`'
  `mixed` flag is false for every one of them and the `(sealed)` suffix has no
  reachable test. Adding an eighth vector is a Rust-primary change and does not
  belong in B2a.

  So Task 5 adds `gui/seal_fixture_test.go` (below). Note also that the
  "low-iteration blob" motivation was wrong: the *host* derives 100,000
  iterations in tens of milliseconds — the ~31 s figure is device-only — so the
  fixture uses the §6.2 floor and the suite cost is the frame pumping, not the
  KDF.

`gui/seal_fixture_test.go`, new file.

```go
package gui

import (
	"crypto/aes"
	"crypto/cipher"
	"strings"
	"testing"

	"seedhammer.com/seal"
)

// A sealer for package gui.
//
// seal's own sealForTest is unexported in a _test.go file, so no import reaches
// it from here (R0 round 0, I3), and the canonical vectors cannot express every
// shape B2a must test -- in particular NO vector carries md1/mk1 in BOTH
// sections, which is the only shape that exercises the plate list's "(sealed)"
// disambiguation.
//
// This is deliberately NOT a second implementation of the format: it composes
// seal's own Header.Encode and DeriveKey, so a wire-format change breaks it the
// same way it breaks production. Production has no sealing path and must not
// grow one -- a device that can seal is a device that can be made to emit a
// payload (seal/crypto.go:12-16).
func sealBlobForTest(t *testing.T, public, secret []string, passphrase string, iterations uint32) []byte {
	t.Helper()
	pub := []byte(strings.Join(public, "\n"))
	h := seal.Header{PubLen: uint32(len(pub))}
	if len(secret) > 0 {
		pt := []byte(strings.Join(secret, "\n"))
		h.Iterations = iterations
		// Fixed salt and IV: a test fixture must be deterministic. §7.2's
		// one-key-one-message rule is not weakened, because nothing here is
		// ever a real payload -- and production cannot reach this code at all.
		for i := range h.Salt {
			h.Salt[i] = byte(i + 1)
		}
		for i := range h.IV {
			h.IV[i] = byte(i + 0x40)
		}
		h.CtLen = uint32(len(pt))
		hdr := h.Encode()
		aad := append(append([]byte(nil), hdr[:]...), pub...)
		key := seal.DeriveKey(seal.NormalisePassphrase(passphrase), h.Salt[:], int(iterations))
		block, err := aes.NewCipher(key)
		if err != nil {
			t.Fatalf("aes: %v", err)
		}
		gcm, err := cipher.NewGCM(block)
		if err != nil {
			t.Fatalf("gcm: %v", err)
		}
		// Seal appends the tag, which is exactly the wire layout (§6).
		return append(aad, gcm.Seal(nil, h.IV[:], pt, aad)...)
	}
	hdr := h.Encode()
	return append(append([]byte(nil), hdr[:]...), pub...)
}
```

> **This fixture is machine-checked, and it was checked before this plan went to
> a reviewer.** Executed against the real packages, with vector D's five public
> records and its `ms1`:
>
> ```
> sealed    Inspect ok, pub=5, Sealed()=true, UnlockWithKey ok, secret[0]=codex32 secret
>           hash = a26e d22b b747 dfd0 2367 06ad 14c1 9679
> unsealed  Inspect ok, pub=5, Sealed()=false
>           hash = 70f3 e35a acf7 47db c40f 8376 91aa 61e0
> ```
>
> Those two digests are **vector D's `pubhash_sealed` and vector E's
> `pubhash_unsealed`, byte for byte** — so `sealBlobForTest` agrees not merely
> with production but with the normative §11.4 vectors, which is the property
> that makes it safe to build Task 5–7's assertions on. R0 round 1 reproduced
> the round-trip independently and additionally confirmed the both-sections
> shape (public `mk1`×2 + encrypted `ms1`/`mk1`/`md1`, 528 bytes) that no vector
> can supply. Task 5.1 re-asserts it in the suite so a later edit cannot break it
> silently.
- **`TestSealedPayloadStopsAtATerminalScreen` (`gui/unlock_flow_test.go:211`)
  asserts the exact behaviour this task removes.** It is replaced, not deleted:
  the new assertion is that a sealed payload reaches the *passphrase* screen and
  that cancelling it returns to the menu without constructing the plate list.

**Steps:**

- [ ] **5.1** Write `seal/unlock_key.go` + `seal/session.go` and their tests
      (`UnlockWithKey` reproduces `Unlock` on vectors A–D, F, G; `ErrNotSealed`
      on E; `SecretsResident` true after unlock and false after wiping each).
      Run `nix develop --command go test ./seal/`.
      **Also write `gui/seal_fixture_test.go` and assert it round-trips** through
      `seal.Opener.Inspect` + `UnlockWithKey` in **both** shapes before any test
      depends on it (R0 round 1, N3 — §5e assigned this here and this step did
      not list it). If the fixture and production disagree about the format,
      every Task 5–7 test measures the fixture instead of the code.
- [ ] **5.2** Refactor `Unlock` onto `UnlockWithKey`. Re-run — **every existing
      `seal` test must pass unchanged**; they are what proves the refactor did
      not fork the pipeline.
- [ ] **5.3** Write the gui tests (checksum gate with a KDF counter; the retry
      loop keeping the hash on screen; cancel never reaching the plate list).
      Expect FAIL.

      **The counter is a test-local swap of `newDeriver`**, restored with
      `t.Cleanup` in the `unlockEngraveHook` idiom. Assert the count is **0**
      after a checksum-invalid attempt and **1** after a valid one; a
      return-value assertion cannot tell those apart.

      **Also write the vector-E negative here** (R0 round 1, I2). §11.2: "Vector
      E reaches the plate list with the keyboard flow **NEVER ENTERED** —
      asserted by instrumenting the prompt entry point, not by return value. A
      scripted fake platform will happily feed twelve words into a prompt that
      should not exist and still reach the plate list, so a return-value
      assertion reports PASS over exactly the defect." Set
      `unlockPassphraseHook`, drive vector E to the plate list, assert the hook
      **never fired**. Task 8's `ct_len == 0` row names this test.

      **Budget the frame count.** `unlockDerive` draws one frame per
      `kdfStepIterations = 500`. **Measured, not derived: a 100,000-iteration
      derivation is exactly 200 `Step` calls and 199 drawn frames.** The house
      idiom is `pumpUntil(frame, want, 32)` — 32 is far too few here and would
      look like a hang. Pump **≥256**.

      A counting `newDeriver` may also return a deriver over a *smaller*
      iteration count to keep a test short — **but only for tests that do not
      need the unlock to SUCCEED** (R0 round 1, N2). The header's count is inside
      the AAD, so a key derived at a different count fails the tag and the flow
      can never reach the plate list. That is fine for the three negatives above;
      it is wrong for Task 7's `(sealed)` test, which needs a real unlock.
- [ ] **5.4** Write `gui/unlock_kdf.go` and the `unlock_flow.go` fragment.
- [ ] **5.5** `nix develop --command go test ./gui/ ./seal/`, then the TinyGo
      device build.
- [ ] **5.6** Mutation check.

      | mutant | must be killed by |
      | --- | --- |
      | checksum gate moved after `unlockDerive` | the KDF-counter assertion, **not** the return value — both orders return the same error |
      | `errors.Is(err, seal.ErrAuthentication)` arm made the default | the too-many-records test, which would otherwise read as "wrong passphrase" |
      | `unlockRetryBody` drops the hash | the retry-loop screen assertion, anchored on `", SEALED):"` and asserting `"UNSEALED"` is absent |
      | cancel falls through to the plate list | the cancel test, asserting the list is never constructed |
      | `clear(blob)`/`blob = nil` removed | not test-observable — record it as a surviving mutant rather than inventing a test that appears to cover it |

- [ ] **5.7** Commit.

---

## Task 6 — §10.2.2: the secret session

**Every record that classifies as a secret is offered FIRST, consecutively, and
each is wiped as its plate leaves the screen — by any route.**

Secret means `seal.IsSecret`: `ms1` and a bare BIP-39 mnemonic. **An encrypted
`mk1`/`md1` is not a secret** — §6.3's table is explicit, and §11.2 requires
vector F's *three* `ms1` records to be the ones offered first while its twelve
`mk1`/`md1` records are ordinary plates.

### 6a. Neither existing secret-engrave path can be reused whole

- **`engraveCodex32` (`gui/codex32_polish.go:215`)** offers a `codex32Recover`
  branch that calls `recoverCodex32Flow`, which waits on **physical NFC shares**.
  A payload-sourced share has no tags to tap — the same F-76 shape that made B1
  refuse `mdmkFlow`.
- **`backupSeedStringFlow` (`gui/gui.go:2198`)** loops
  `for { if Engrave { return } }`, so a **cancelled** engrave re-presents the
  plate forever. Under §10.2.2 the record is wiped on cancel, so the loop would
  offer to re-cut a record that is about to be zeroed.
- **`backupWalletFlow` (`gui/gui.go:2151`)** has the same loop, plus a passphrase
  prompt and a fingerprint choice that do not belong on a payload-sourced seed.

**Compose the pieces directly**, exactly as B1 did for `mdmkFlow`.

### 6b. The new file

`gui/unlock_session.go`, new file.

```go
package gui

import (
	"fmt"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/backup"
	"seedhammer.com/bip39"
	"seedhammer.com/codex32"
	"seedhammer.com/font/constant"
	"seedhammer.com/seal"
)

// §10.2.2 -- the secret session. Every record that classifies as a secret is
// offered FIRST, consecutively, and each is wiped as its plate leaves the
// screen BY ANY ROUTE: Cut, Skip, Cancel, a failed engrave, an error, ctx.Done.
//
// Why plural is load-bearing: a 2-of-3 wsh-sortedmulti has THREE ms1 cards
// (vector F). Under a singular implementation the operator engraves one of
// three, the plate list then shows only mk1/md1 so nothing looks missing, and
// they store an incomplete backup of a 2-of-3 believing it complete -- §6.4's
// own "worst available outcome".
//
// Why the wipe is keyed on the plate leaving rather than on completion: aborting
// mid-plate to re-seat shifted steel is the machine's most ordinary recovery,
// and keying on completion would leave the seed resident in a state nothing
// guards. Re-cutting then needs a fresh unlock -- twelve words and a ~31 s KDF --
// ONCE THE ENGRAVE SCREEN IS LEFT. A plate merely PAUSED resumes from the
// spline: the record was zeroed before the plate reached the screen, so there is
// nothing left to re-protect.
// That is the price, it is deliberate (operator, 2026-08-07, reaffirmed
// 2026-08-08), and it costs no reboot: the sealed blob is untouched in flash.

// unlockSecretHook is a test-only seam. It observes each stage with the record's
// live bytes, so a test can assert on the BUFFER -- that the record is non-zero
// when offered and zero once its plate has left -- rather than on a return
// value, which cannot tell a wipe from a missing wipe. nil in production.
// Mirrors unlockEngraveHook, the sanctioned in-file seam.
var unlockSecretHook func(stage string, idx int, record []byte)

// unlockMnemonicHook observes bip39.Parse's []Word copy at the moment the plate
// is handed to Engrave. It exists because that copy is a LOCAL: no test could
// reach it, which is how a live seed passed a green suite once already. nil in
// production.
var unlockMnemonicHook func(m bip39.Mnemonic)

// unlockSecretLabel names a secret plate by its CLASSIFIED type and its index
// among secrets -- never by anything the sealer asserted, and never by
// rendering the record's contents.
func unlockSecretLabel(c seal.Classification, i, n int) string {
	name := "secret"
	switch c {
	case seal.ClassCodex32Secret:
		name = "ms1"
	case seal.ClassMnemonic:
		name = "seed words"
	}
	if n > 1 {
		return fmt.Sprintf("%s %d/%d", name, i+1, n)
	}
	return name
}

// unlockSecretSession offers every secret record, in order, before the plate
// list is built.
func unlockSecretSession(ctx *Context, th *Colors, p *seal.Payload) {
	at := make([]int, 0, len(p.Secret))
	for i, r := range p.Secret {
		if seal.IsSecret(r.Class) {
			at = append(at, i)
		}
	}
	for n, i := range at {
		unlockSecretPlate(ctx, th, p, i, unlockSecretLabel(p.Secret[i].Class, n, len(at)))
	}
}

// unlockSecretPlate offers ONE secret plate and wipes it on the way out.
//
// The wipe is a defer registered before anything can return. That is what makes
// "by any route" a property of the code rather than of the author remembering
// every branch -- and the branches are not obvious: EngraveScreen.Engrave has
// two returns but five reachable exits, including a panic unwind.
func unlockSecretPlate(ctx *Context, th *Colors, p *seal.Payload, i int, label string) {
	defer func() {
		p.WipeSecretAt(i)
		if unlockSecretHook != nil {
			unlockSecretHook("wiped", i, p.Secret[i].Record)
		}
	}()
	if unlockSecretHook != nil {
		unlockSecretHook("offered", i, p.Secret[i].Record)
	}
	cs := &ChoiceScreen{
		Title:   label,
		Lead:    "SECRET seed material",
		Choices: []string{"Cut this plate", "Skip"},
	}
	// Back and Skip are the same outcome, and both wipe. There is deliberately
	// no third option: §10.2.2 gives the operator Cut or Skip, and a "later"
	// that kept the record resident is the state this section exists to prevent.
	choice, ok := cs.Choose(ctx, th)
	if !ok || choice != 0 {
		return
	}
	switch p.Secret[i].Class {
	case seal.ClassCodex32Secret:
		unlockEngraveCodex32(ctx, th, p.Secret[i].Record)
	case seal.ClassMnemonic:
		unlockEngraveMnemonic(ctx, th, p.Secret[i].Record)
	}
}

// unlockEngraveCodex32 cuts one ms1 record.
//
// It does NOT reuse engraveCodex32, whose codex32Recover branch waits on
// physical NFC shares that a payload-sourced record does not have -- the same
// dead end that made B1 refuse mdmkFlow (F-76). Nor backupSeedStringFlow, whose
// `for { if Engrave { return } }` loop re-presents a CANCELLED plate: under
// §10.2.2 that record is already being wiped, so the retry would offer to cut
// nothing.
//
// HONEST CAVEAT, and it is the same one gui/ms1_decode.go:19-20 already
// carries: codex32.String holds the share as a Go string, and backup.SeedString
// and the Plate derived from it hold further copies. None can be zeroed. What
// this function guarantees is that seal's OWN buffer is zeroed by the caller's
// defer and the derived copies are dropped; TinyGo's GC decides the rest.
func unlockEngraveCodex32(ctx *Context, th *Colors, rec []byte) {
	s, err := codex32.New(string(rec))
	if err != nil {
		// Unreachable behind §10.2.1's allow-list, which admitted this record
		// via codex32.New in the first place. Named rather than assumed.
		showError(ctx, th, unlockTitle, "This record is not a readable codex32 secret.")
		return
	}
	id, _, _ := s.Split()
	params := ctx.Platform.EngraverParams()
	plan, err := backup.EngraveSeedString(params, backup.SeedString{
		Title: id,
		Seed:  s.String(),
		Font:  constant.Font,
	})
	if err != nil {
		showError(ctx, th, unlockTitle, "This record does not fit any plate size.")
		return
	}
	plate, err := toPlate(plan, params)
	if err != nil {
		showError(ctx, th, unlockTitle, "This record does not fit any plate size.")
		return
	}
	// §10.2.2, and it must be HERE rather than after Engrave returns. The plate
	// carries the geometry: newEngraverJob holds plate.Spline
	// (gui/engraver.go:64) and the engrave loop iterates e.spline (:170), so
	// nothing reads these bytes again. Waiting for Engrave would leave the seed
	// resident for the whole ~21-minute cut -- and Back while running does NOT
	// return (gui/gui.go:2651-2656 calls Stop() and keeps rendering), so the
	// abort-mid-plate path §10.2.2 calls the machine's most ordinary recovery
	// would leave it resident indefinitely and let the operator resume without
	// a fresh unlock.
	clear(rec)
	// ONE engrave, then return regardless of the outcome.
	NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
}

// unlockEngraveMnemonic cuts one bare-mnemonic record.
//
// Composed rather than delegated to backupWalletFlow, which prompts for a
// BIP-39 passphrase, offers a fingerprint choice, and loops back to re-Confirm
// on a cancelled engrave. The plate produced here is the one backupWalletFlow's
// Skip-passphrase path produces.
func unlockEngraveMnemonic(ctx *Context, th *Colors, rec []byte) {
	m, err := bip39.Parse(rec)
	if err != nil {
		showError(ctx, th, unlockTitle, "This record is not a readable BIP-39 mnemonic.")
		return
	}
	// bip39.Parse returns a SECOND copy of the secret as []Word -- as complete and
	// as wipeable as seal's. clear() reaches []Word where wipeBytes ([]byte) does
	// not compile.
	//
	// This defer covers the EARLY RETURNS below, where no plate is ever built. It
	// is NOT the wipe that matters: see clear(m) beside clear(rec) further down,
	// and read the warning there before moving either.
	defer clear(m)
	// §6c FLIPS THIS to &SeedScreen{NoEdit: true}. It is written as the plain
	// constructor here for one reason: NoEdit is added to SeedScreen by a
	// FRAGMENT in gui/gui.go, and a whole-file block that referenced it would
	// not type-check against the unmodified fork -- the plan's build gate would
	// fail for a reason that is not a defect. The two-line flip is §6c's, and it
	// is a reviewer's execution pass rather than a machine-checked one. Do not
	// ship without it: the edit nav button (Button2, or a tap on its slot) opens
	// the word editor, and editing an authoritative payload seed produces a
	// self-consistent plate that does not restore the payload's wallet.
	ss := new(SeedScreen)
	if !ss.Confirm(ctx, th, m) {
		return
	}
	// The BARE fingerprint. A payload-sourced seed gets no passphrase prompt:
	// §8's twelve words are the payload's passphrase and are NEVER seed
	// entropy, and offering a second, different passphrase here is exactly the
	// confusion §8 says the UI must not create.
	mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams, "")
	if err != nil {
		showError(ctx, th, unlockTitle, "Couldn't derive the fingerprint for this seed.")
		return
	}
	params := ctx.Platform.EngraverParams()
	plate, err := engraveSeed(params, m, mfp)
	if err != nil {
		showError(ctx, th, unlockTitle, "This seed does not fit any plate size.")
		return
	}
	// §10.2.2 — see unlockEngraveCodex32 for why this is before Engrave and not
	// after. BOTH copies go here, and that is the whole point: rec is seal's
	// []byte and m is bip39.Parse's independent []Word of the same seed.
	//
	// AN EARLIER DRAFT OF THIS PLAN ZEROED m ONLY ON THE DEFER ABOVE, AND THAT
	// WAS A CRITICAL. A defer fires when the function RETURNS -- after Engrave --
	// so a full copy of the seed stayed live for the whole ~21-minute cut and
	// indefinitely on the paused or failed engrave screen: exactly the residency
	// these two lines exist to remove, on exactly the abort-mid-plate path
	// §10.2.2 calls the machine's most ordinary recovery. Nothing else could
	// reach it -- p.Wipe() and §10.2.4's SecretsResident() both scan p.Secret,
	// not this local -- so the B2b timer condition would read FALSE while the
	// seed was live. Measured at Engrave entry: len=24, non-zero words=24.
	//
	// Three R0 rounds and a scoped review passed over it because every one of
	// them was watching rec. clear is idempotent, so the defer's double-zero is
	// free. Found by the B2a-ii whole-diff review, lens 1 C1.
	clear(rec)
	clear(m)
	if unlockMnemonicHook != nil {
		unlockMnemonicHook(m)
	}
	NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
}
```

### 6c. The `SeedScreen` fragment (R0 round 0, M5)

Modify `gui/gui.go` — add one field to `SeedScreen` and one guard in `Confirm`:

```go
	// NoEdit suppresses the edit affordance. Zero value is EDITABLE, so every
	// existing caller keeps today's behaviour; only a payload-sourced seed sets
	// it. For a TYPED seed, editing a word is a typo fix. For an authoritative
	// one it is corruption, and because Confirm's caller then derives a
	// fingerprint from whatever is on screen, the resulting plate is internally
	// self-consistent and does not restore the payload's wallet.
	NoEdit bool
```

**The guard goes on the CLICK HANDLER, not on the layout** (R0 round 1, I1):

```go
		if !s.NoEdit && editBtn.Clicked(ctx) {
			inputWordsFlow(ctx, th, mnemonic, s.selected, "")
			continue
		}
```

at `gui/gui.go:2331`, and the nav slot is skipped as well so the icon disappears.
**Skipping only the slot does not close the route.** `Filter.matches`
(`gui/event.go:155-159`) gates a `buttonEvent` on button identity alone, with no
bounds check, so `editBtn.Clicked(ctx)` keeps consuming `ButtonFilter(Button2)`
and `ButtonFilter(Center)` whether or not anything was drawn for it.

> **An earlier draft of this section said "on a touch-only SH2 a centre tap opens
> the editor". That is FALSE and is recorded so it is not reintroduced.**
> Production on the SH2 emits **only** `gui.PointerEvent`
> (`cmd/controller/platform_sh2.go:413`); `Center` is a `Button`, and the fork's
> only producer of one is `cmd/controller/debug_sh2.go:82`, a debug build. A
> pointer event reaches a `Clickable` solely by hit-test against a drawn
> `op.Input` region, and `editBtn`'s only region is the nav slot. The **centre**
> of the seed screen is the word list, which registers its own
> `op.Input(&ctx.B, &s.words[i])` (`gui/gui.go:2467`) and merely moves the
> selection.
>
> The real affordance is the **edit nav button** — `Button2`, or a tap on its
> slot. The threat M5 identified is unchanged; only the route was described
> wrongly.

Then flip the one call site in `gui/unlock_session.go`:

```go
	ss := &SeedScreen{NoEdit: true}
```

> **These edits are FRAGMENTS and are therefore unchecked by the build gate** —
> §6b's whole-file block deliberately writes `new(SeedScreen)` so it type-checks
> against the unmodified fork. This is the one place in the plan where a
> whole-file block and a fragment must be applied together to be correct, and it
> is called out here so it is not half-applied.

**Test — and the earlier one could not fail** (R0 round 1, I1). It asserted that
a centre tap does not reach word entry, which is **vacuously true today with no
change at all**, and its positive control ("with it clear, it still does") could
never pass. Assert both routes instead:

- with `NoEdit` set, **neither** a tap on the edit nav slot **nor**
  `press(&ctx.Router, Button2)` reaches word entry;
- with `NoEdit` clear, **both** still do — this is the existing scan-path
  behaviour and it must not regress.

The `Button2` half is the one that discriminates a guard on the click handler
from a guard on the layout.

### 6d. Tests — asserted on the BUFFERS, never on a return value

Fragment additions to a new `gui/unlock_session_test.go`. The load-bearing cases,
each traceable to §11.2:

| case | vector | asserts |
| --- | --- | --- |
| every secret offered before any public plate | **F** (15 records, `ms1`×3) | the three `ms1` records are offered consecutively, in order, before the plate list is constructed |
| **each is zeroed before the next is offered** | **F** | at the moment record *k+1* is offered, record *k*'s buffer reads all-zero |
| Skip wipes | F | offer → Skip → buffer zero |
| Back from the Cut/Skip choice wipes | F | offer → Back → buffer zero |
| **the buffer is zero WHILE the engrave screen is up** | F | offer → Cut → pump to the engrave screen → assert the record reads all-zero **without leaving it**. This is the assertion that pins the early wipe; every after-the-fact check passes whether the wipe is before or after `Engrave`. |
| a cancelled engrave leaves nothing | F | offer → Cut → cancel → buffer zero (necessarily true given the row above, asserted anyway because it is §10.2.2's literal bullet) |
| a mnemonic secret engraves | **A** | reaches `SeedScreen`, then the engrave screen; buffer zero afterwards |
| `SecretsResident()` is false when the session ends | F | the §10.2.4 predicate B2b will key on |
| no secret is ever in the plate list | F, C, G | `unlockPlates` contains no `ms1` and no mnemonic |
| **the plate list is never constructed on a cancelled unlock** | D | cancel the passphrase → menu, no list |

> **Mutate in BOTH directions.** B1's whole-diff review found two tests that
> could not fail, and one of them was *one-directional* — it caught the opposite
> mutant, which is exactly why it read as correct. For "every secret first",
> assert both that the secrets come first **and** that a mutant offering only the
> first one fails.

**Steps:**

- [ ] **6.1** Write `gui/unlock_session_test.go`. Expect FAIL (undefined).
- [ ] **6.2** Write `gui/unlock_session.go`.
- [ ] **6.3** `nix develop --command go test ./gui/`.
- [ ] **6.4** Mutation check.

      | mutant | must be killed by |
      | --- | --- |
      | only the first secret offered (`at = at[:1]`) | the vector-F offer-order test — nothing else in the suite has more than one secret |
      | **`clear(rec)` moved to after `Engrave` returns** | the **resident-during-engrave** test: drive to the engrave screen and assert the buffer is already zero *while it is still on screen*. Nothing else sees it — every after-the-fact assertion passes under both. |
      | `defer p.WipeSecretAt(i)` deleted entirely | **the Skip test and the Back test**, which return before any plate is built. *(An earlier draft named the cancelled-engrave test; `Engrave` returns on cancel, so a merely-moved defer still runs. R0 round 0, M2.)* |
      | `IsSecret` widened to include `ClassMDMK` | **the vector-F offer-order test**, which sees 15 offers instead of 3. *(Not the "no secret in the plate list" test: `unlockPlates` filters on `Class != ClassMDMK`, not on `IsSecret`, so the twelve entries stay put. R0 round 0, M2.)* |
      | `IsSecret` narrowed to `ClassCodex32Secret` only | the vector-A mnemonic test |
      | Back from Cut/Skip returns without wiping | the Back test |

- [ ] **6.5** Commit.

---

## Task 7 — the plate list after the session

Three changes, all to `gui/unlock_platelist.go` plus one new file.

1. **It lists the encrypted section's cards too.** After the secret session, RAM
   holds public records *and* whatever `mk1`/`md1` travelled encrypted (vector C:
   five of six; vector F: twelve of fifteen). Leaving them out would be §6.4's
   incomplete-backup-believed-complete with the operator's own payload.
2. **Records already cut this session are marked** (§10.2.2, and F-80's B2 item).
   The mark is a **convenience, not a guarantee** — it does not survive a power
   cut and the UI must not imply it does.
3. **Back reads as leaving the session, not as stepping back one screen**
   (§10.3, F-80's B2 item). It is drawn with `assets.IconBack` today.

### 7a. The Back icon: `assets.IconDiscard`, not a new asset

There is no lock glyph in `gui/assets` (checked: the set is arrow-{up,down,left,right},
circle, circle-filled, hammer, icon-{back,checkmark,discard,edit,gear,hammer,info,left,right,progress},
key-backspace, logo-small, nav-btn-{primary,secondary}). **`assets.IconDiscard`
already exists and already means this**: `gui/gui.go:2316` uses it for discarding
a seed, and `gui/freetext_flow.go:1097,1196` for clearing entered text. "Discard
this session and everything in it" is exactly what Back does here.

Drawing a bespoke lock glyph is a font/asset cycle (F-78's territory), and
substituting an existing icon that already carries the meaning is not a
workaround — it is the cheaper correct answer. If a reviewer disagrees, the
alternative is an asset task, not a different existing icon.

### 7b. The new file

`gui/unlock_plates.go`, new file.

```go
package gui

import "seedhammer.com/seal"

// The post-session plate list's model.
//
// After §10.2.2's secret session, RAM holds the public records AND whatever
// md1/mk1 travelled in the encrypted section -- five of vector C's six records,
// twelve of vector F's fifteen. §6.3 is explicit that md1/mk1 are not secret
// wherever they travelled, so they are ordinary plates and leaving them out of
// the list would be §6.4's incomplete-backup-believed-complete with the
// operator's own payload.

// unlockPlate is one entry: a record safe to leave resident, plus what the
// operator needs to see about it.
type unlockPlate struct {
	rec seal.AdmittedRecord
	// idx is the record's position within its OWN section, which is what
	// plateLabel's fallback branch numbers when a record somehow carries no
	// card labels.
	idx int
	// sealed marks a record that came from the encrypted section, and is set
	// only when BOTH sections carry cards -- see unlockPlateLabel.
	sealed bool
	// cut records that this plate has already been engraved THIS SESSION. A
	// convenience, not a guarantee: it does not survive a power cut, and
	// §10.2.2 requires the UI not imply that it does.
	cut bool
}

// unlockPlateLabel is plateLabel plus the two suffixes B2a adds.
//
// It WRAPS plateLabel rather than replacing it: the card/plate arithmetic and
// the md1/mk1 naming are §6.3's grouping rendered, and having two functions
// that both decide what a card is called is the divergence F-77 exists to
// prevent.
func unlockPlateLabel(r seal.AdmittedRecord, idx int, sealed, cut bool) string {
	s := plateLabel(r, idx)
	if sealed {
		// Card indices are computed PER SECTION, so a public mk1 1/2 and an
		// encrypted mk1 1/2 can both exist on one payload. Rendered without
		// this the list shows the same label twice and the operator cannot tell
		// which plate they are about to cut. Added only when both sections
		// actually carry cards, so the ordinary payload gains no noise.
		s += " (sealed)"
	}
	if cut {
		// Deliberately a WORD and not a glyph: F-78 measured that "·"
		// contributes zero pixels in this font, and a mark the operator cannot
		// see is worse than no mark at all.
		s += " (cut)"
	}
	return s
}

// unlockPlates builds the list. Secrets are absent by construction -- they were
// offered and wiped by §10.2.2's session before this is called, and
// seal.IsSecret is the single definition of which those are.
func unlockPlates(p *seal.Payload) []unlockPlate {
	var pub, enc bool
	for _, r := range p.Public {
		if r.Class == seal.ClassMDMK {
			pub = true
		}
	}
	for _, r := range p.Secret {
		if r.Class == seal.ClassMDMK {
			enc = true
		}
	}
	mixed := pub && enc
	out := make([]unlockPlate, 0, len(p.Public)+len(p.Secret))
	for i, r := range p.Public {
		out = append(out, unlockPlate{rec: r, idx: i})
	}
	for i, r := range p.Secret {
		if r.Class != seal.ClassMDMK {
			continue
		}
		out = append(out, unlockPlate{rec: r, idx: i, sealed: mixed})
	}
	return out
}
```

### 7c. `unlock_platelist.go` fragments

- `unlockPlateListFlow` takes `[]unlockPlate` and builds its labels with
  `unlockPlateLabel(e.rec, e.idx, e.sealed, e.cut)` **on entry and again after
  each engrave** (`relabel()`), so the "(cut)" mark appears as soon as a plate
  completes.

  *An earlier draft said "each frame", and so did mutation row 7.6. Neither was
  ever true — `relabel()` is called twice, not per frame — and the mutant row 7.6
  actually kills is "the post-engrave `relabel()` deleted". Corrected in the
  firmware comment first; corrected here second, which is the wrong order and is
  why F-97 existed.*
- `unlockEngraveFlow` returns `bool` (did the plate complete), and the OK branch
  sets `plates[sel].cut = true` on true.
- The nav's first slot becomes
  `{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconDiscard}`.

**Steps:**

- [ ] **7.1** Tests first: vector C (encrypted-only cards reach the list), vector
      G (public cards span four cards across pages), a **`sealBlobForTest`**
      payload with cards in **both** sections — §5e's fixture, not `seal`'s own
      `sealForTest`, which is unreachable from `package gui` (R0 round 1, M3) —
      (the `(sealed)` suffix appears and the two
      `mk1 1/2` entries are distinguishable), and the `(cut)` mark appearing after
      a completed engrave and **not** after a cancelled one.
- [ ] **7.2** Write `gui/unlock_plates.go` and the fragments. **Three consequences
      an earlier draft left unscheduled** (R0 round 0, M3) — the first two are
      compile errors, the third is the stale-comment class §1d spends a task
      fixing:

      1. The **unsealed** call site `gui/unlock_flow.go:73`
         `unlockPlateListFlow(ctx, th, p.Public)` becomes
         `unlockPlateListFlow(ctx, th, unlockPlates(p))`. Both paths now build
         the list the same way, which is also what stops the unsealed path from
         silently keeping the old labels.
      2. `gui/unlock_platelist_test.go:126` passes `[]seal.AdmittedRecord` and
         must pass `[]unlockPlate`.
      3. `unlockEngraveFlow`'s call-site comment (`gui/unlock_platelist.go:173-181`)
         says the `string(rec.Record)` conversion is "HARMLESS HERE — B1 holds
         public data only — and **ACTIVELY WRONG in B2**, where the same call
         shape on a secret record makes an unwipeable copy". Task 7 deliberately
         routes **encrypted-section** `md1`/`mk1` through that exact call, which
         §6.3 says is correct — so as written the comment now tells a B2b reader
         the opposite of what B2a decided. Rewrite it: the conversion is
         admissible for `md1`/`mk1` from **either** section, because §6.3 makes
         them public data wherever they travelled, and remains wrong for
         anything `seal.IsSecret` admits — which is why the secret session never
         calls this function.
- [ ] **7.3** `nix develop --command go test ./gui/`.
- [ ] **7.4** Mutation check: drop the `(sealed)` suffix → the both-sections test
      fails; mark `cut` on cancel as well as completion → the cancel test fails;
      revert the icon to `IconBack` → the icon test fails.
- [ ] **7.5** Commit.

---

## Task 8 — the §11.3 mutation rows B2a owns

§11.3's table has **27 mutant rows** (measured,
`design/SPEC_encrypted_payload_delivery.md:1545-1573`). B1 closed the rows its
surface reached. These are B2a's, and **every one must name the test that kills
it** — a mutant with no named killer is a gap in the suite, not a passing result.

| §11.3 mutant | killed by |
| --- | --- |
| BIP-39 checksum check removed | Task 5's KDF-counter test on `beef`×11 + `bacon` |
| KDF run before the checksum gate | Task 5's KDF-counter test — **not** the return value |
| iteration count read as a constant | Task 3's `TestDeriverHonoursTheIterationCount` (vector B) |
| tag verification made unconditional-pass | `seal`'s existing flipped-ciphertext negative, re-run through `UnlockWithKey` |
| public section left out of the AAD | `seal`'s existing vector-D public-flip negative, ditto |
| only the first secret record offered | Task 6's vector-F offer-order test |
| `ms1` not wiped after its plate | Task 6's post-plate buffer assertion |
| wipe omitted on the Back exit path | Task 6's Back test **and** Task 5's `defer p.Wipe()` |
| passphrase prompted when `ct_len == 0` | **Task 5.3's vector-E negative**, which sets `unlockPassphraseHook` and asserts it never fired. Named, not merely described (R0 round 1, I2) — a return-value assertion passes over exactly this defect. |
| `ms1` accepted in the public section | `seal`'s existing refusal test |
| idle timer runs during engraving | **B2b** — no timer exists in B2a. Record as deferred, with its owning phase, rather than claiming coverage. |

**Procedural rules, restated because they have been got wrong here before:**
assert the substitution matched **before** running the test — a silently-failing
`sed` reads exactly like a surviving mutation — and restore from a **file copy**,
never `git checkout`.

- [ ] **8.1** Run every row. Record the results in the commit message, as a table,
      with the actual output pasted.
- [ ] **8.2** Any surviving mutant is a **blocking** finding, not a note.

---

## Task 9 — hardware: the in-situ KDF rate, and end-to-end unlock

**§7.1's residual is owed before release and is scheduled here.** The measured
9,715 it/s came from an RP2350**A** (Pico 2, QFN60); the SeedHammer II is an
RP2350**B** (QFN80). `cmd/kdfbench` is deliberately NOT run — it says in its own
header that it targets a Pico 2, running it on the SH2 would mean overwriting
the application firmware, and running it on a Pico 2 adds nothing because 9,715
it/s already came from one.

Task 5's `log.Printf("seal: kdf %d iterations in %s", …)` makes the operator's
real unlock the measurement — the number they actually experience, in the real
call path, on the real part.

**Flash with `~/bin/sh/sh2-flash`, never `picotool` by hand** — the build output
is unsigned and a hand-flashed image will not boot the machine.

**Procedure:**

- [ ] **9.1** `me seal` vector **F**'s shape (all-encrypted, three `ms1`) to a
      data-family UF2 for `0x10E00000`. F is the one that discriminates plural
      from singular, which is the property most worth proving on metal.
- [ ] **9.2** Load, reboot, open *Sealed Payload*, enter the twelve words.
- [ ] **9.3** Record, from the screen and from the log: the elapsed derivation
      time, the iteration count, and the computed rate. **This closes §7.1.**
- [ ] **9.4** Confirm all three `ms1` plates are offered consecutively, before
      any `mk1`/`md1` entry appears.
- [ ] **9.5** **Cut one secret plate to completion.** This is the configuration
      F-79 names as never having been exercised: payload present *plus* a running
      engrave. Confirm no out-of-memory.
- [ ] **9.6** **Cancel a secret plate mid-cut.** Confirm two distinct things,
      because they differ and an earlier draft of this step asserted only the
      second and would have observed the opposite:
      (a) the paused plate **resumes without re-entering the passphrase** — the
      job holds the spline, and the record was already zeroed when the plate was
      built; and
      (b) once you **leave the engrave screen**, re-cutting that record does
      require the twelve words again. That is §10.2.2's price, observed.
- [ ] **9.7** Enter a **wrong** passphrase and confirm the message offers both
      readings and keeps the hash on screen.
- [ ] **9.8** Record the results verbatim in `design/HARDWARE_RESULT_<date>_phaseB2a.md`.

> **Watch what you paste.** Two commit messages last cycle claimed results that
> were never checked, and a third quoted a mutation table that does not exist in
> any commit in this repository. Record what the screen actually showed,
> `&&`-chain the commands, and read the output *at* it.

---
## Gate coverage — state this in the R0 brief

> **RUN THE BUILD GATE WITH A FORK ROOT THAT HAS B2a-i MERGED:**
> `scripts/plan-build-gate-go.sh <this plan> <fork-with-B2a-i>`.
>
> Against the *unmodified* fork tier 1 **fails**, and the failure is not a
> defect: this document's Go calls `seal.NewDeriver` and reads `AdmittedRecord`'s
> encrypted-section labels, both of which B2a-i creates. Measured — assembling
> B2a-i's four whole files plus its `record.go` fragment into a scratch fork and
> re-running gives `PASS: go build ./gui ./seal clean` and `PASS: go vet
> introduces nothing new`. Until B2a-i is merged, that is how this gate is run.
>
> Two traps met while doing it, recorded so the next person does not rediscover
> them: the scratch dir is **rebuilt by every gate invocation**, so B2a-i's files
> must be regenerated before being copied; and `nix develop` prints
> `warning: Git tree '…' is dirty` on an uncommitted fork root, which the gate
> reads as a build failure because it treats any output as one. Commit the
> throwaway tree first.

Both gates apply and **both MUST be run before dispatch and after every fold.**

- **`scripts/plan-cite-gate.sh <plan>` resolves every `file:line` and
  `pkg.Symbol` in this plan against the real source and prints the line.** Its
  stated blind spot: it cannot tell you the line *says* what the plan claims.

  **It exits 1 on this plan, with exactly three failures, and all three are
  expected:**

  ```
  FAIL  seal.Deriver     no func/type/const/var Deriver in seal/
  FAIL  seal.IsSecret    no func/type/const/var IsSecret in seal/
  FAIL  seal.NewDeriver  no func/type/const/var NewDeriver in seal/
  ```

  All three are symbols **this plan creates** (Tasks 3 and 5). The gate has a
  `skip` branch for a whole package a plan creates, but not for a new symbol in
  an existing one. **Any fourth failure is real.** Every other citation resolves,
  including the seven corrected in "Carried-forward citations that have DRIFTED"
  above — each of which the gate reports `ok` while printing a line that does
  **not** say what the stale citation claimed, which is the gate's own blind
  spot working exactly as documented.
- **`scripts/plan-build-gate-go.sh <plan>` type-checks the whole-file Go.** The
  files anchored as new — `seal/label_encrypted.go`, `seal/label_encrypted_test.go`,
  `seal/pbkdf2.go`, `seal/pbkdf2_test.go`, `seal/unlock_key.go`, `seal/session.go`,
  `gui/unlock_kdf.go`, `gui/unlock_session.go`, `gui/unlock_plates.go` — are
  assembled into a scratch copy of the fork and run through `go build` and a
  **baselined** `go vet`. Undefined identifiers, wrong types, bad signatures and
  unused imports fail there.
- **Therefore the FRAGMENTS are the reviewer's execution pass, and they are
  where the risk is.** Tier 2 proves syntax only: a fragment naming a function
  that does not exist, passing a wrong type, or returning the wrong arity parses
  happily. This plan's fragments are the `seal/record.go` call site, the
  `seal/read*.go` `Probe` methods, `Unlock`'s tail, `gui/gui.go`'s `uiFlow`
  probe, and `gui/unlock_flow.go` / `gui/unlock_platelist.go`.

### Beyond the gate: the plan's Go was also RUN

The gate builds and vets. It does not execute. Three of this plan's claims are
executable, so they were executed — in the gate's own scratch tree
(`/tmp/plan-build-gate-go`), with the §1c fragment hand-applied. **A reviewer
does not need to re-derive any of this:**

1. **Task 3's chunked PBKDF2 reproduces every vector key.** All five `Deriver`
   tests PASS as written: six of seven vectors (all but E, which has no key)
   match `derived_key_hex` byte-for-byte, at step sizes 1, 2, 7, 499, 500,
   100000 and 2²⁰, and vector B's 100001 iterations produce a different key from
   A's 100000. **The design of Task 3 is verified, not proposed.**
2. **Task 1's F-77 grouping works, and produces the right shape.** With the
   fragment applied, vector F's twelve encrypted cards label as three `mk1`
   cosigner cards of two plates each (`k card 1/3`, `2/3`, `3/3`) and one `md1`
   card of six plates (`d card 1/1 plate 1..6/6`) — which is exactly what
   §10.2.2's labels require and what a flat `mk1 1/6..6/6` would have conflated.
3. **Exactly one pre-existing test fails, and it is the one §1f inverts.** The
   rest of the `seal` package — 74 tests including every vector test — passes
   unchanged. That is the measurement behind §1f's claim, and it is why "a
   SECOND failing test means Task 1 is wrong" is a usable tripwire rather than a
   hope.
4. **§1e's corrected fixture was MUTATION-CHECKED, not merely re-run** (R0 round
   0, I4). With `labelEncryptedCards` mutated to propagate its grouping error
   and `AdmitSection` mutated to return it, `TestUnreadableEncryptedCardDoesNotReject`
   **fails** with exactly the diagnostic it was written to produce:

   ```
   a label failure rejected the payload: seal: public records do not form a
   decodable card set: record 0: md: bit stream truncated — §10.2.1 requires the
   decode for the public section only, and this is an ADMISSION change
   ```

   The version this replaced passed under that same mutant. Both files were
   restored from a **file copy**, not `git checkout`, and the substitutions were
   asserted to have matched before the run — a silently-failing `sed` reads
   exactly like a surviving mutation.

**What this does NOT prove:** none of the `gui` code was executed. It compiles
and vets against the real fork — every identifier, signature and type in
`gui/unlock_kdf.go` (304 lines), `gui/unlock_session.go` (237) and
`gui/unlock_plates.go` (83) resolves — but no screen was driven, no frame drawn,
and no wipe observed. The gui tests in Tasks 5–7 are described, not written.
That is the reviewer's execution pass, and it is where the risk now sits.

> **B1's plan had NO Go type checking at all** and said so honestly; F-74 built
> the gate afterwards and observed that on the B1 plan "tier 1 had nothing to
> check because every whole-file block was elided with `...`". This plan is
> written to carry complete files wherever the change is a new file, precisely so
> that finding does not repeat.

**Machine-verified before this plan reached a reviewer** (do not re-derive):

- `AdmittedRecord`'s label fields are populated for `SectionPublic` only; the
  gate is `if section == SectionPublic` at `seal/record.go:214`.
- `permitted()` (`seal/record.go:171`) admits `ClassMDMK` **unconditionally**,
  not gated on section.
- `seal/testdata/vectors.json` carries all seven vectors A–G, `sha256`
  `333ac47e7f61d031c995b85510565bfffd86cd1992f09b0230c1484fffd4d4bc`. Secret-set
  composition, measured: C = `ms1`×1/`mk1`×2/`md1`×3; F = `ms1`×3/`mk1`×6/`md1`×6;
  G = `ms1`×3 with 12 public records.
- `EngraveScreen.Engrave` (`gui/gui.go:2644`) has exactly two `return`
  statements, `:2661` and `:2707`, and **no error return**; a failed engrave is
  state, not an exit.
- `Engrave`'s Back exit fires from five states, `engraveStopping` included.
- `wipeBytes` (`gui/slip39_polish.go:342`) takes `[]byte` and **does not compile**
  against `bip39.Mnemonic`; the builtin `clear` does, and is already `seal`'s
  idiom.
- `assets.IconDiscard` exists and is already used at `gui/gui.go:2316`.
- The `gui` suite is ~12 s; `go test ./...` exits 1 with exactly two setup
  failures; `go vet ./gui/` carries one pre-existing diagnostic.

---
## What B2a-ii does NOT cover

- **§10.2.4's residency-keyed idle wipe.** B2b. `seal.Payload.SecretsResident()`
  ships in Task 5 as the predicate it will key on, but nothing consults it yet.
  Carried forward, verified, so B2b does not re-derive it: **there is no
  last-physical-input accessor reachable from a flow.** `a.idle.start` is a field
  of an anonymous struct local to `Run`'s closure (`gui/gui.go:2884-2891`);
  `Context` has no time-of-last-input field; and `Event` (`gui/event.go:105-109`)
  carries no timestamp. Worse, a flow-local reconstruction is **lossy**:
  `EventRouter.Reset` (`gui/event.go:281-294`) discards every event no filter
  claimed, so a press on a button the current screen does not bind resets the
  screensaver's timer but is invisible to the flow — a flow-local timer therefore
  drifts *early* and can fire while the operator is present. And the screensaver
  does not unwind the flow: `gui/gui.go:2954-2959` `continue`s without calling
  `yield()`, so a flow stays blocked inside `ctx.Frame` with its stack, and its
  secret, fully live for as long as the saver runs.
- **F-76** — inspecting a payload-sourced card.
- **F-80's `layoutMainPager` pixel pin** — needs a rasterising check.
- **Any release tag.** §10.2.4 is a backstop, not an optional extra.

---
## Follow-ups filed by this plan

- **F-81 — WITHDRAWN before it was filed.** It described a residency window
  created by wiping after `Engrave` returned. R0 round 0 (C1) showed that design
  was both unnecessary and non-conforming, and the plan now wipes at plate
  construction, so the window does not exist. Recorded as withdrawn rather than
  deleted so a reader of the R0 report finds its disposition.
- **F-84 — `SeedScreen` gains `NoEdit` (owning phase: B2a, Task 6).** Not a
  deferral — it is implemented in Task 6 — but recorded because it changes a
  screen the NFC scan path also uses. Zero value stays editable, so every
  existing caller is unaffected by construction; the new field is set only by
  `unlockEngraveMnemonic`, where editing authoritative payload data would let an
  operator cut a self-consistent seed plate that does not restore the payload's
  wallet.
- **F-82 — `seal.Deriver` has no Rust counterpart (owning phase: ownerless
  residue).** The chunked derivation is device-only: the host has no progress bar
  to draw. It produces byte-identical output, pinned by six vectors, so the
  Rust-primary rule does not bind it — recorded so a future reader does not
  mistake the asymmetry for drift.
- **F-83 — the plate cannot be wiped until the engrave finishes. WITHDRAWN as a
  follow-up and recorded as an ACCEPTED LIMITATION (operator, 2026-08-08:
  "the one honest gap is unavoidable").**

  `validateMdmk`, `backup.SeedString`, `engraveSeed` and `toPlate` copy the
  record into Go strings and into `Plate.Spline`, none of which can be zeroed.
  `gui/ms1_decode.go:19-20` already carries the same caveat for the display path.

  **It is not a defect to be scheduled, and filing it as one would be dishonest
  bookkeeping.** The plate is a geometric rendering of the very words being cut
  into steel; it must exist, in RAM, for as long as the needle is moving. No
  ordering of wipes can change that, and a plate pipeline over `[]byte` would
  move the secret rather than remove it — the spline still encodes it. A
  follow-up that will never be actioned is noise in a register whose whole value
  is that every open item is real.

  **What is therefore true, stated once so nothing downstream overclaims:**
  during a secret engrave the seed is recoverable from SRAM by an attacker with
  physical access and an SWD probe (§2.2 item 9, live because `debug enable: 1`
  is measured in §3). §10.2.2's wipe removes the **record** — the only copy that
  outlives the plate — and that is the whole of what it claims. B2a-ii's
  `clear(rec)` at plate construction is what makes "the record" and "the plate"
  distinct lifetimes rather than one.

  **Owed to the SPEC, not to the code.** §2.2's "what this does NOT defend
  against" should carry this limitation in the operator's terms; the spec is
  GREEN, so that is an amendment, filed as **F-85** with owning phase *before the
  release tag*. Do not amend it as a side effect of an implementation commit.

- **F-85 — §2.2 does not name the during-engrave residency (owning phase: before
  the release tag).** See F-83. One paragraph in the SPEC's threat model, in the
  same register as items 9 and 11, saying that a secret plate's geometry is
  resident for the duration of its cut and that physical custody is the control.
  It changes no behaviour and no test; it closes the gap between what the machine
  does and what the operator has been told.

### Record defects to fix while touching these files

- **`design/FOLLOWUPS.md`: F-73 and F-74 are marked CLOSED in their headings but
  still sit above the `## Resolved` marker**, unlike F-67–F-70 which were moved.
  Move them.
- **`CONTINUITY_2026-08-08.md` §9 says "F-80's two B2 items".** There are
  **three** with an explicit `owning phase: B2` — the `layoutMainPager` pin, the
  Back icon, and the "already cut" marks. Two of the three land in B2a per the
  2026-08-08 decision; only the pixel pin is B2b's.