# Encrypted Payload Delivery — Plan B Phase B2a (unlock and the secret session) — Implementation Plan

**Status:** DRAFT — R0 round 0 folded, re-review pending. **No code before 0C/0I.**

| round | verdict | report |
| --- | --- | --- |
| 0 | **1C / 4I / 6M / 3N** — all folded | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round0.md` |

**Round 0's Critical was the plan's own "interpretation" section**, and its
justification was false rather than merely debatable: the record was held across
the engrave because the retry prompt supposedly needed it, when `engraveJob`
holds `plate.Spline` and never reads the record again. The correct design was
also the simpler one — `clear(rec)` the moment the plate exists — and it needed
no interpretation of §10.2.2 at all. **The lesson to carry: an "interpretation"
section is a smell.** Twice now on this feature, prose explaining why a
requirement could not be met literally has turned out to rest on an unverified
claim about how existing code behaves, and both times one `grep` would have
settled it.

Two of the four Importants were **tests that could not fail** — a fixture that
never reached the code path it was named for, and a mutation table naming a
counter that no longer sits in the path under test. Both are the same class B1's
whole-diff review found twice.

**Descends from:** `SPEC_encrypted_payload_delivery.md` §10, which is GREEN and
normative. This plan implements §10.2 steps 5–9 plus §10.2.2, and closes **F-77**
and **F-79**. It does **not** restate requirements — where this plan and §10
disagree, §10 wins and this plan is defective.

**Predecessor:** `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB1.md`
(merged, `78949e7`). B1 is the unsealed path with UI; B2a is the half that holds
a secret.

**Successor:** B2b — §10.2.4's residency-keyed idle wipe, F-80's remaining B2
item, F-76. **THE FEATURE IS NOT OPERATOR-COMPLETE UNTIL B2b. Do not tag a
release after B2a.**

---

## Why the phase boundary is here

B1 was cheap to review because **no secret was ever resident**. B2a is the
opposite: it is the phase where seed material lives in SRAM, and every task
below exists either to put it there or to get it out again.

| | B1 (shipped) | **B2a (this plan)** | B2b |
| --- | --- | --- | --- |
| §10.1 detection + menu entry | ✅ | — | — |
| §10.2 steps 1–3 (`Inspect`, hash) | ✅ | — | — |
| §10.2 step 4 (`ct_len == 0` warning) | ✅ | — | — |
| plate list (paged) + engrave public records | ✅ | extended | — |
| §10.2 steps 5–6 (12 words, checksum gate) | — | ✅ | — |
| §10.2 step 7 (KDF + progress) | — | ✅ | — |
| §10.2 steps 8–9 (AEAD open, retry loop) | — | ✅ | — |
| §10.2.2 secrets-first session lifecycle | — | ✅ | — |
| **F-77** encrypted-section card grouping | — | ✅ **gating** | — |
| **F-79** 64 KB retention | — | ✅ | — |
| §7.1 in-situ KDF rate on RP2350B | — | ✅ | — |
| §10.2.4 residency-keyed idle wipe | — | — | ✅ |
| F-80 `layoutMainPager` pixel pin, F-76 | — | — | ✅ |

**What B2a ships without, and it must be said plainly:** §10.2.4's timer does not
exist yet. Between the moment a secret record is decrypted and the moment its
plate leaves the screen, the **only** control on residency is §10.2.2's wipe.
That is why Task 6 makes the wipe a `defer` registered before anything can
return, and why the §10.2.2 reading in "The one place this plan interprets the
spec" below matters more in B2a than it will in B2b.

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
- **The `gui` test suite must not gain 31 s per test.** `Opener.KDF`
  (`seal/open.go:24`) is the sanctioned seam; Task 3 adds the equivalent seam for
  the chunked path. The whole `gui` suite is ~12 s today (measured); one real
  100,000-iteration derivation would nearly triple it.
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
builds `newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, opts)`
(`gui/engraver.go:64`), and the engrave loop iterates `e.spline`
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
not interpreted. Residency collapses from "the whole ~21-minute cut, plus
indefinitely on a paused or failed screen" to "the few milliseconds between
decrypt and plate construction, plus however long the Cut/Skip choice is on
screen". `unlockSecretPlate`'s `defer` stays as the backstop for every path that
never reaches a plate at all, and becomes idempotent.

This also matters for B2b: `SecretsResident()` now goes false as the cut starts,
so §10.2.4's residency key means what it says instead of staying true for the
entire engrave.

**Do not overclaim what this buys.** The `Plate` still encodes the secret — it is
a geometric rendering of the very words about to be cut into steel, and it must
exist for the duration of the cut. An SWD reader with the machine open during an
engrave can reconstruct the seed from `plate.Spline` whether or not the record
buffer is zeroed. What the early wipe removes is the *record*, the *only* copy
that outlives the plate and the one §10.2.2 names. The unwipeable derived copies
are **F-83**, they are not closed by this, and no phase of this feature closes
them — that needs a plate pipeline over `[]byte`.

**Consequence for the retry story, stated so Task 9.6 tests the truth:** because
the job holds the spline, an operator who pauses mid-cut **can resume the same
plate without re-entering the passphrase**. What needs a fresh unlock is
re-cutting after *leaving* the engrave screen, when the plate is gone too. That
is the honest reading of §10.2.2's "re-cutting needs a fresh unlock", and it is
strictly better than the alternative: the pause/resume path never had a seed
record resident to protect.

---

## Task 1 — F-77: publish the §6.3 grouping for the ENCRYPTED section (GATING)

**Why it gates:** §10.2.2's plate labels are unimplementable without it for any
multisig payload. `AdmittedRecord.HRP`/`CardIndex`/`CardTotal`/`PlateIndex`/`PlateTotal`
are populated for `SectionPublic` only, because pass 3 runs only there
(`seal/record.go:214`). And the encrypted section is full of cards: vector C's
secret set is `ms1`×1 / `mk1`×2 / `md1`×3, vector F's is `ms1`×3 / `mk1`×6 /
`md1`×6 — **twelve of vector F's fifteen secret records are cards** (measured
from `seal/testdata/vectors.json`).

**Reuse `groupRecords`/`cardKey`/`labelCards`. Do NOT re-derive classification in
`gui`** — that is the two-code-paths divergence `Opener.Inspect`'s doc comment
exists to prevent, and Task 4a of the B1 plan rejected it for the public section
on the same grounds.

### 1a. Why this is LABEL-ONLY and must never reject

`decodePublicSet` is **not** extended over the encrypted section, and
`labelEncryptedCards` swallows a grouping error rather than returning it. Three
reasons, and the third is the binding one:

1. §10.2.1's table requires DECODE for the **public** section only. The encrypted
   section's admission surface is `mdmkText`, `ms1`, or a BIP-39 mnemonic, with
   no decode step.
2. `cardKey` (`seal/record.go:353`) fails closed for anything that is not an
   md1/mk1 card, and the encrypted section legitimately carries `ms1` and
   mnemonics. Grouping the whole section would reject every real payload.
3. **Rejecting on a grouping failure would change ADMISSION**, which the
   Rust-primary rule (`CLAUDE.md`) puts in Rust first, with test vectors. Adding
   *labels* changes no wire format, no identity algorithm, no validation and no
   admission — it publishes a partition that is already computed. That is
   plumbing, and the rule does not bind it.

### 1b. The new file

`seal/label_encrypted.go`, new file.

```go
package seal

// F-77 — §6.3's card grouping, published for the ENCRYPTED section.
//
// AdmittedRecord's label fields were populated for SectionPublic only, because
// pass 3 (decodePublicSet) is the sole place a grouping is computed and it runs
// only there (record.go:214). But §6.3 admits md1/mk1 into the encrypted
// section explicitly, and the vectors carry them: vector C's secret set is
// ms1 x1 / mk1 x2 / md1 x3, vector F's is ms1 x3 / mk1 x6 / md1 x6. Without
// this, §10.2.2's secret-session plate labels are unimplementable for every
// multisig payload.
//
// LABEL-ONLY, and that is normative here rather than a shortcut:
//
//   - It runs over the ClassMDMK SUBSET. cardKey fails closed for anything that
//     is not an md1/mk1 card, and the encrypted section legitimately carries ms1
//     and bare mnemonics, so grouping the whole section would reject every real
//     payload.
//   - A grouping failure is DISCARDED, not returned. §10.2.1 requires the decode
//     for the public section only; turning a label failure into a rejection
//     would change ADMISSION, and admission changes land in Rust first with test
//     vectors (the Rust-primary rule). Publishing a partition that is already
//     computed changes no behaviour at all.
//
// A record whose card cannot be read therefore keeps its zero label fields, and
// gui's plateLabel already renders that as "record N" rather than mislabelling
// it as an md1 (gui/unlock_platelist.go:50-55).
func labelEncryptedCards(out []AdmittedRecord) {
	// Stringifying an md1/mk1 copies PUBLIC data by §6.3 — an xpub or a wallet
	// policy, not key material — which is why the same conversion is already
	// done unremarked for the public section (record.go:217-220). ms1 and
	// mnemonic records are never converted here.
	at := make([]int, 0, len(out))
	strs := make([]string, 0, len(out))
	for i, r := range out {
		if r.Class != ClassMDMK {
			continue
		}
		at = append(at, i)
		strs = append(strs, string(r.Record))
	}
	if len(strs) == 0 {
		return
	}
	g, err := groupRecords(strs)
	if err != nil {
		return
	}
	// labelCards indexes by position within the slice it is handed, so the
	// subset is labelled in its own coordinates and scattered back. Reusing it
	// rather than reimplementing the card/plate arithmetic is the point: two
	// implementations of "which card is this" is exactly what F-77 exists to
	// avoid.
	sub := make([]AdmittedRecord, len(strs))
	labelCards(sub, g)
	for j, i := range at {
		out[i].HRP = sub[j].HRP
		out[i].CardIndex = sub[j].CardIndex
		out[i].CardTotal = sub[j].CardTotal
		out[i].PlateIndex = sub[j].PlateIndex
		out[i].PlateTotal = sub[j].PlateTotal
	}
}
```

### 1c. The call site — a three-line fragment in `seal/record.go`

Insert immediately after the existing `if section == SectionPublic { … }` block
(`seal/record.go:214-241`) and before `return out, nil`:

```go
	// F-77 — pass 3's grouping, published for the encrypted section too. It is
	// LABEL-ONLY: see labelEncryptedCards. No decode, and no new rejection.
	if section == SectionEncrypted {
		labelEncryptedCards(out)
	}
```

### 1d. Update `AdmittedRecord`'s doc comment — it currently states the opposite

`seal/record.go:101-112` says the fields are "0 for every record in the encrypted
section — INCLUDING ClassMDMK ones … See F-77." That sentence is what Task 1
falsifies. Replace the "and 0 for every record in the encrypted section" clause
with:

```go
	// HRP is 'd' (md1) or 'k' (mk1) for a ClassMDMK record in EITHER section,
	// and 0 for every other record. §6.3 admits md1/mk1 into the encrypted
	// section explicitly, and vector F's secret set is ms1 x3 / mk1 x6 / md1 x6,
	// so twelve of its fifteen secret records ARE cards. Grouping the encrypted
	// section is LABEL-ONLY (labelEncryptedCards): no decode runs there and no
	// grouping failure rejects a payload. F-77, closed in B2a.
```

> **Leaving that comment stale is not cosmetic.** It is the sentence a B2b author
> would read to decide whether a secret record can be labelled, and it would tell
> them no.

### 1e. Tests first

`seal/label_encrypted_test.go`, new file.

```go
package seal

import (
	"testing"

	"seedhammer.com/codex32"
)

// F-77. The encrypted section's md1/mk1 records must carry the same card
// labels the public section's do, or §10.2.2's secret-session plate list cannot
// name a plate on any multisig payload.
//
// Vector C is the discriminating fixture on the small side: its secret set is
// ms1 x1 / mk1 x2 / md1 x3, so it exercises the SUBSET path (an ms1 sits among
// the cards) as well as the labelling.
func TestEncryptedSectionCardsAreLabelled(t *testing.T) {
	v := vectorNamed(t, "C")
	out, err := AdmitSection(bs(v.Secret), SectionEncrypted)
	if err != nil {
		t.Fatalf("admit vector C's secret section: %v", err)
	}
	if len(out) != 6 {
		t.Fatalf("vector C's secret section is %d records, want 6", len(out))
	}
	var cards, secrets int
	for i, r := range out {
		switch r.Class {
		case ClassMDMK:
			cards++
			if r.HRP != 'd' && r.HRP != 'k' {
				t.Errorf("record %d is a card with HRP %q, want 'd' or 'k'", i, r.HRP)
			}
			if r.PlateTotal < 1 || r.PlateIndex < 1 || r.PlateIndex > r.PlateTotal {
				t.Errorf("record %d has plate %d/%d, which is not a 1-based index",
					i, r.PlateIndex, r.PlateTotal)
			}
			if r.CardTotal < 1 || r.CardIndex < 1 || r.CardIndex > r.CardTotal {
				t.Errorf("record %d has card %d/%d, which is not a 1-based index",
					i, r.CardIndex, r.CardTotal)
			}
		case ClassCodex32Secret:
			secrets++
			// An ms1 is not a card and must keep its zero label. A non-zero HRP
			// here would mean the subset filter leaked.
			if r.HRP != 0 || r.CardIndex != 0 || r.PlateIndex != 0 {
				t.Errorf("record %d is an ms1 but carries card labels: %+v", i, r)
			}
		}
	}
	if cards != 5 || secrets != 1 {
		t.Fatalf("vector C's secret section classified as %d cards + %d secrets, want 5 + 1",
			cards, secrets)
	}
}

// Vector F is the one that discriminates plural from singular: 15 secret
// records, ms1 x3 / mk1 x6 / md1 x6, with the six mk1 records spanning THREE
// cosigner cards. A flat mk1 1/6..6/6 conflates them, which is §6.4's
// incomplete-backup-believed-complete hazard wearing a label.
func TestEncryptedMultisigCardsAreDistinguishable(t *testing.T) {
	v := vectorNamed(t, "F")
	out, err := AdmitSection(bs(v.Secret), SectionEncrypted)
	if err != nil {
		t.Fatalf("admit vector F's secret section: %v", err)
	}
	if len(out) != 15 {
		t.Fatalf("vector F's secret section is %d records, want 15", len(out))
	}
	seen := make(map[[2]int]bool)
	var mk int
	for _, r := range out {
		if r.Class != ClassMDMK || r.HRP != 'k' {
			continue
		}
		mk++
		if r.CardTotal != 3 {
			t.Fatalf("mk1 record reports %d cards of its HRP, want 3", r.CardTotal)
		}
		key := [2]int{r.CardIndex, r.PlateIndex}
		if seen[key] {
			t.Fatalf("two mk1 records share card %d plate %d — the three cosigner "+
				"cards have been conflated", r.CardIndex, r.PlateIndex)
		}
		seen[key] = true
	}
	if mk != 6 {
		t.Fatalf("vector F carries %d mk1 records, want 6", mk)
	}
}

// A record the grouping cannot read must NOT reject the payload: §10.2.1
// requires the decode for the public section only, and turning a label failure
// into a rejection would change ADMISSION, which lands in Rust first.
//
// THE FIXTURE MUST REACH groupRecords, and an earlier draft of this test did
// not (R0 round 0, I4): it used a mnemonic-only section, so labelEncryptedCards
// returned at `len(strs) == 0` and the mutant "return the grouping error instead
// of discarding it" survived the entire suite.
//
// codex32.AssembleMD1(nil) is the fixture that does reach it. Measured against
// the real packages, not assumed:
//
//	assembled          = "md1t7yjcvgk6xetg" (16 bytes)
//	codex32.ValidMD    = true          -> Classify = ClassMDMK, so it is IN the subset
//	md.ParseChunkHeader = "md: bit stream truncated"
//	cardKey            = ErrUndecodableCardSet: record 0: md: bit stream truncated
//
// Note the seal package's existing smuggledMD1 fixture CANNOT serve here:
// md.ParseChunkHeader SUCCEEDS on it ({Version:0 Chunked:false ChunkSetID:0},
// err=nil), so cardKey returns cleanly and the error path is never taken.
func TestUnreadableEncryptedCardDoesNotReject(t *testing.T) {
	v := vectorNamed(t, "C")
	var realMD1 string
	for _, r := range v.Secret {
		if codex32.ValidMD(r) {
			realMD1 = r
			break
		}
	}
	if realMD1 == "" {
		t.Fatal("vector C carries no md1 record; the premise of this test is broken")
	}
	broken := codex32.AssembleMD1(make([]byte, 0))
	if Classify([]byte(broken)) != ClassMDMK {
		t.Fatalf("the fixture classifies as %v, not a card, so it never reaches the grouping",
			Classify([]byte(broken)))
	}
	if _, err := cardKey(broken, 0); err == nil {
		t.Fatal("the fixture's card key resolves; it cannot exercise the failure path")
	}

	out, err := AdmitSection([][]byte{[]byte(broken), []byte(realMD1)}, SectionEncrypted)
	// (a) a grouping failure must NOT reject.
	if err != nil {
		t.Fatalf("a label failure rejected the payload: %v — §10.2.1 requires the "+
			"decode for the public section only, and this is an ADMISSION change", err)
	}
	// (b) every record is still admitted.
	if len(out) != 2 {
		t.Fatalf("admitted %d records, want 2", len(out))
	}
	// (c) and the whole subset falls back to zero labels rather than to wrong
	// ones — groupRecords is all-or-nothing, so one unreadable card costs the
	// labels of every card beside it. gui's plateLabel renders that as
	// "record N" (gui/unlock_platelist.go:50-55), never as a mislabelled md1.
	for i, r := range out {
		if r.HRP != 0 || r.CardIndex != 0 || r.PlateIndex != 0 {
			t.Errorf("record %d carries a label (%c card %d/%d plate %d/%d) although the "+
				"grouping failed", i, r.HRP, r.CardIndex, r.CardTotal, r.PlateIndex, r.PlateTotal)
		}
	}
}
```

### 1f. ONE existing test asserts the opposite, and it must be INVERTED

**`TestEncryptedRecordsCarryNoGrouping` (`seal/grouping_test.go:103`) fails under
Task 1, by design — measured, not predicted.** Running the plan's own code
against a scratch tree with the §1c fragment applied, it is the **only** failure
in the whole `seal` suite, and it fails twelve times:

```
--- FAIL: TestEncryptedRecordsCarryNoGrouping (0.00s)
    grouping_test.go:118: secret record 3 carries a grouping (k card 1/3 plate 1/2); pass 3 does not run for the encrypted section
    …
    grouping_test.go:118: secret record 14 carries a grouping (d card 1/1 plate 6/6); pass 3 does not run for the encrypted section
```

**Invert it; do not delete it, and do not weaken it.** Its own doc comment says
what it is for: "That is the trap Phase B2 inherits (F-77), and this test is what
makes it a measured fact rather than a recollection." It pins a **documented
gap**, not a requirement — and F-77 is the ticket to close that gap. Closing it
is what B2a is for.

This is emphatically **not** the "fix the test rather than the code"
anti-pattern that B1's round-1 fold warned about. The distinction is whether the
test encodes an *invariant* or a *known deficiency*:

- The ordering constraint next to it — grouping must run **after** the
  allow-list, because `cardKey` fails closed and
  `TestPublicSectionRefusesASecret` asserts `ErrRecordNotPermitted` — is an
  invariant, and Task 1 does not touch it. `labelEncryptedCards` runs after the
  loop, exactly as pass 3 does.
- `TestEncryptedRecordsCarryNoGrouping` encodes a deficiency, with the follow-up
  number that closes it written in its own comment.

The replacement keeps the premise check that gives the test its force — **vector
F's secret section must hold exactly 12 cards of 15 records**, or the test would
pass just as green on a payload with no cards at all.

**Steps:**

- [ ] **1.1** Write `seal/label_encrypted_test.go` above. Run
      `nix develop --command go test ./seal/ -run 'Encrypted'`. Expect FAIL:
      `undefined: labelEncryptedCards` is not the failure — the tests call
      `AdmitSection`, so expect *label assertions* to fail with zero HRPs.
      **Measured, running exactly this:** `record 1 is a card with HRP '\x00',
      want 'd' or 'k'` … and `vector F carries 0 mk1 records, want 6`.
- [ ] **1.2** Write `seal/label_encrypted.go`. Add the call site in
      `seal/record.go`. Update the `AdmittedRecord` doc comment.
- [ ] **1.3** Invert `TestEncryptedRecordsCarryNoGrouping` per §1f — rename it
      (`TestEncryptedRecordsCarryTheirGrouping`), assert every `ClassMDMK` record
      carries a 1-based card/plate identity and every non-card record still
      carries none, and keep the `cards != 12` premise check.
- [ ] **1.4** `nix develop --command go test ./seal/`. **Every other Phase A test
      must pass UNCHANGED** — measured: with the fragment applied and only this
      one test still in its old form, it is the sole failure in the package.
      **If a SECOND test needs editing, the change was not additive and Task 1 is
      wrong.**
- [ ] **1.5** Mutation check. Apply each, confirm a named test fails, restore
      from a **file copy** (never `git checkout`), and assert the substitution
      matched before running — a silently-failing `sed` reads exactly like a
      surviving mutant.

      | mutant | must be killed by |
      | --- | --- |
      | `labelEncryptedCards` body emptied | `TestEncryptedSectionCardsAreLabelled` |
      | subset filter widened to every record (drop the `ClassMDMK` continue) | **`TestEncryptedSectionCardsAreLabelled`** — vector C's `ms1` reaches `cardKey`, the whole grouping fails, and every label comes back zero. *(An earlier draft named `TestUnreadableEncryptedCardDoesNotReject` "because the section starts rejecting". It does not: the error is discarded by design. R0 round 0, M2.)* |
      | `labelCards` fed `g.keys` order instead of `g.perRecord` | `TestEncryptedMultisigCardsAreDistinguishable` |
      | the grouping error returned instead of discarded | `TestUnreadableEncryptedCardDoesNotReject` — **only** with the §1e fixture as corrected; the mnemonic-only version never reached `groupRecords` |

- [ ] **1.6** Commit. `git add seal/label_encrypted.go seal/label_encrypted_test.go seal/record.go seal/grouping_test.go`

---

## Task 2 — F-79: retain nothing

`uiFlow` probes once at startup and **holds 65,536 bytes for the GUI's whole
lifetime** (`gui/gui.go:1541-1546`; `XIPReader.Read` allocates
`clampRegion(RegionLen)`, `seal/read_tinygo.go:41-52`). §6.2's own caps make the
largest legal blob `52 + 8191 + 8191 + 16` = **16,450 bytes**, so ~49 KB of it is
provably erased flash. Measured on the B1 branch: `ram 69300`, ~451 KB free of
the RP2350B's 520 KB — **~14% of the free heap, held permanently.**

**Payload-present plus a running engrave is the one configuration hardware has
never exercised**, and `validateMdmk` builds three full plate plans at once. If
that combination exhausts the heap the failure is an out-of-memory *during* an
engrave.

### 2a. `Reader` gains a magic-only probe

Fragment, `seal/read.go` — extend the interface (`seal/read.go:27-29`):

```go
// Reader yields the payload region's bytes. Phase B holds one of these and
// never learns where the bytes came from.
type Reader interface {
	// Probe reports whether the region begins with MNEMBLOB (§6.1). It exists
	// so §10.1's startup detection costs 8 bytes rather than 64 KB: Read's
	// result is retained for as long as the caller holds it, and holding the
	// whole region for the GUI's lifetime is ~14% of free heap (F-79).
	//
	// It is deliberately COARSER than Read: present-but-corrupt probes true,
	// the menu entry appears, and the flow then reports "payload unreadable".
	// Collapsing the two would hide a tampered payload behind an invisible menu
	// entry, which is precisely the signal §2.2 item 4 exists to raise.
	Probe() bool
	Read() ([]byte, error)
}
```

Fragment, `seal/read_tinygo.go` — the probe maps 8 bytes of XIP and copies
nothing (execute-in-place flash is directly addressable; the copy in `Read`
exists only because the caller keeps the bytes across a flash write):

```go
// Probe reads only the magic. unsafe.Slice over XIP is a MAPPING, not a copy,
// so this allocates nothing at all — which is the whole point of F-79.
func (XIPReader) Probe() bool {
	region := unsafe.Slice((*byte)(unsafe.Pointer(uintptr(PayloadAddr))), clampRegion(len(Magic)))
	return hasMagic(region)
}
```

Fragment, `seal/read_host.go`:

```go
// Probe reads only the magic, matching XIPReader.Probe's cost profile as
// closely as a file can. An absent or unreadable file is "no payload" (§6.1) —
// never an error, because on the device the two are indistinguishable.
func (r FileReader) Probe() bool {
	f, err := os.Open(r.Path)
	if err != nil {
		return false
	}
	defer f.Close()
	buf := make([]byte, len(Magic))
	if _, err := io.ReadFull(f, buf); err != nil {
		return false
	}
	return hasMagic(buf)
}
```

### 2b. `uiFlow` retains the reader, not the bytes

Fragment replacing `gui/gui.go:1541-1546`'s `var payload []byte` block:

```go
	// §10.1 detection. Probed ONCE, here, not per frame: the region cannot
	// change while the GUI runs (writing it requires picotool and a reboot),
	// and "absent -> the feature is invisible" is a startup property.
	//
	// F-79: the READER is retained, the BYTES are not. XIPReader.Read allocates
	// the whole 65,536-byte region and at most 16,450 of it can ever be
	// meaningful (§6.2's caps); holding that for the GUI's lifetime is ~14% of
	// free heap, and payload-present PLUS a running engrave is the one
	// configuration hardware has never driven to completion.
	var payloadReader seal.Reader
	if r := ctx.Platform.PayloadReader(); r != nil && r.Probe() {
		payloadReader = r
	}
```

`StartScreen.lastNav` is then set from `payloadReader != nil` rather than from
`payload != nil`, and the dispatch case (`gui/gui.go:1595`, which today reads
`unlockPayloadFlow(ctx, th, payload)`) becomes
`unlockPayloadFlow(ctx, th, payloadReader)`.

### 2c. The flow owns the region's lifetime

Fragment, the head of `unlockPayloadFlow` (`gui/unlock_flow.go:25-31`) — the
signature changes from `blob []byte` to `r seal.Reader`:

```go
func unlockPayloadFlow(ctx *Context, th *Colors, r seal.Reader) {
	blob, err := r.Read()
	if err != nil {
		// ErrNoPayload here means the region was erased between the startup
		// probe and now, which takes a picotool run and a reboot. Report it the
		// same as any unreadable region rather than inventing a third message.
		showError(ctx, th, unlockTitle, "Payload unreadable.")
		return
	}
	// F-79. The region is this flow's for the duration of the session and
	// nobody else's. clear() zeroes it rather than merely dropping it: the AAD
	// and the ciphertext both live in here, and TinyGo will not necessarily
	// collect it before the engrave that follows needs the heap.
	//
	// A CLOSURE, not `defer clear(blob)` (R0 round 0, I1). Deferred call
	// arguments are evaluated when the defer STATEMENT runs, so `defer
	// clear(blob)` would capture the slice header here and pin all 65,536 bytes
	// for the whole flow -- the §5d `blob = nil` would rebind the local and
	// release nothing, and F-79 would be reported closed while unfixed in
	// exactly the payload-present-plus-running-engrave configuration Task 9.5
	// exists to test. The closure reads `blob` at EXIT instead.
	defer func() { clear(blob) }()
	var o seal.Opener
	p, err := o.Inspect(blob)
```

**And the blob is released before the engrave**, not merely at the end — see
Task 5, which sets `blob = nil` after `UnlockWithKey` returns, its last use.

### 2d. Tests

Fragment additions to `gui/unlock_program_test.go` and `gui/unlock_flow_test.go`.
`runUnlock` (`gui/unlock_flow_test.go:17`) currently takes `[]byte`; it takes a
`seal.Reader` instead, and `payloadReaderFor` (`gui/unlock_program_test.go:88`)
already produces one from a vector name, so the change is one line per call site.

```go
// F-79. The startup probe must not read the region: a 64 KB allocation held for
// the GUI's lifetime is ~14% of free heap, and the menu only needs to know
// whether MNEMBLOB is there.
func TestStartupProbesWithoutReadingTheRegion(t *testing.T) {
	var reads, probes int
	r := &countingReader{inner: payloadReaderFor(t, "D"), reads: &reads, probes: &probes}
	p := newPlatform()
	p.payload = r
	ctx := NewContext(p)
	frame, quit := runUI(ctx, func() { uiFlow(ctx, "test") })
	defer quit()
	if _, ok := pumpUntil(frame, "Sealed Payload", 32); !ok {
		t.Fatal("the menu entry never appeared, so the probe did not report present")
	}
	if probes == 0 {
		t.Fatal("startup never probed")
	}
	if reads != 0 {
		t.Fatalf("startup called Read %d times; F-79 requires the region NOT be read "+
			"until the flow is entered", reads)
	}
}
```

**Steps:**

- [ ] **2.1** Write `countingReader` and `TestStartupProbesWithoutReadingTheRegion`.
      Run it. Expect FAIL — `reads` is 1 today.
- [ ] **2.2** Add `Probe()` to the interface and both implementations. Run
      `nix develop --command go test ./seal/`.
- [ ] **2.3** Rewire `uiFlow`, `unlockPayloadFlow` and every call site. Update
      `runUnlock` and the four `gui/unlock_*_test.go` files.
- [ ] **2.4** `nix develop --command go test ./gui/ ./seal/`.
- [ ] **2.5** TinyGo device build — `Probe` is the first new `unsafe.Slice` site
      since B1 and the build is the only thing that compiles it.
- [ ] **2.6** Commit.

---

## Task 3 — the chunked KDF (§10.2 step 7)

**`pbkdf2.Key` is one blocking call.** `seal.DeriveKey` (`seal/crypto.go:43`) has
no callback, no counter and no cancellation, so ~31 s of the frame loop simply
does not happen: the screen holds its last frame, touch queues, and the operator
sees exactly the hang §10.2 step 7 exists to prevent. Nothing in the fork covers
this — measured: there is no percentage readout and no indeterminate spinner
anywhere.

**What makes this safe to write rather than reckless:** the derived keys for
vectors **A, B, C, D, F and G** are in `seal/testdata/vectors.json` as literals.
A chunked implementation is pinned by six independent cross-implementation
vectors *and* by direct equality against the `crypto/pbkdf2` call it replaces,
before anything depends on it. It changes no wire format, no identity algorithm,
no validation and no admission, so the Rust-primary rule does not bind it —
the host has no progress bar to build.

### 3a. The arithmetic, so the step size is derived and not guessed

§7.1 measured **9,715 iterations/sec** on RP2350 silicon. At 500 iterations per
frame that is **51.5 ms** of work between draws, ≈19 frames/sec — comfortably
above the ~10 fps at which a progress bar reads as motion, and far below the
~250 ms at which a touch feels unresponsive. 300,000 iterations is 600 frames.

### 3b. The new file

`seal/pbkdf2.go`, new file.

```go
package seal

import (
	"crypto/hmac"
	"crypto/sha256"
	"hash"
)

// §7's PBKDF2-HMAC-SHA256, run in SLICES.
//
// DeriveKey stays as the one-shot form and is what the vectors pin. This is the
// same function decomposed so §10.2 step 7's progress indicator can be a real
// one: crypto/pbkdf2.Key blocks for ~31 s with no callback and no counter, so
// the frame loop stops dead and the operator sees the hang the step exists to
// prevent.
//
// Two properties make this a decomposition rather than new crypto:
//
//   - dkLen == KeyLen == sha256.Size, so RFC 8018's OUTER loop runs exactly
//     once and the block index is always 1. What is left is the inner
//     U_i = PRF(P, U_{i-1}) chain and an XOR accumulator — no branching, no
//     block bookkeeping, nothing to get subtly wrong that a vector cannot see.
//   - Every vector's derived key is a literal in testdata/vectors.json, and
//     pbkdf2_test.go asserts BOTH that this reproduces them and that it equals
//     DeriveKey iteration-for-iteration.
//
// It allocates nothing per Step: the HMAC is constructed once and Reset()
// restores its keyed state, and both buffers are arrays.
var _ [0]struct{} = [KeyLen - sha256.Size]struct{}{}

// Deriver is one in-progress derivation. The zero value is NOT usable; call
// NewDeriver.
type Deriver struct {
	mac   hash.Hash
	u     [sha256.Size]byte
	acc   [sha256.Size]byte
	done  int
	total int
}

// NewDeriver starts a derivation over the §8.1-normalised passphrase.
//
// passphrase is []byte and not string DELIBERATELY: it is the caller's buffer
// and the caller can zero it, which Unlock's string parameter makes impossible.
// The honest caveat: hmac.New folds the passphrase into an ipad/opad pair
// inside the hash state, and those are key-equivalent and not reachable to be
// zeroed. Wipe clears everything this type owns; it cannot clear that. Same
// defence-in-depth-not-a-guarantee framing as the rest of the firmware.
func NewDeriver(passphrase, salt []byte, iterations int) *Deriver {
	if iterations < 1 {
		// Unreachable behind ParseHeader, which bounds iterations to
		// [100_000, 2_000_000] before any KDF work (§6.2). Clamped rather than
		// panicked: on a device a panic is a brick.
		iterations = 1
	}
	d := &Deriver{
		mac:   hmac.New(sha256.New, passphrase),
		total: iterations,
	}
	// U_1 = PRF(P, S || INT_32_BE(1)). The block index is a literal 1 because
	// there is exactly one block; see the type comment.
	d.mac.Write(salt)
	d.mac.Write([]byte{0, 0, 0, 1})
	d.mac.Sum(d.u[:0])
	d.acc = d.u
	d.done = 1
	return d
}

// Step runs at most n further iterations and reports whether the derivation is
// complete. It never runs past total, so a caller that oversteps is harmless.
func (d *Deriver) Step(n int) bool {
	for i := 0; i < n && d.done < d.total; i++ {
		d.mac.Reset()
		d.mac.Write(d.u[:])
		// Sum appends into u's own backing array. Write has already consumed
		// the previous value, so overwriting it here is correct.
		d.mac.Sum(d.u[:0])
		for j := range d.acc {
			d.acc[j] ^= d.u[j]
		}
		d.done++
	}
	return d.done >= d.total
}

// Done and Total drive the progress indicator. Done counts iterations already
// applied, including the U_1 that NewDeriver performed.
func (d *Deriver) Done() int  { return d.done }
func (d *Deriver) Total() int { return d.total }

// Key returns a FRESH copy of the derived key, which the caller owns and MUST
// zero. It is a copy so that Wipe can be deferred at the point the Deriver is
// created without zeroing the result out from under the caller — the shape a
// shared buffer would make impossible to get right.
//
// It returns nil while the derivation is incomplete: a partial accumulator is
// not a short key, it is the wrong key, and returning it would fail as an
// indistinguishable tag mismatch ~31 s later.
func (d *Deriver) Key() []byte {
	if d.done < d.total {
		return nil
	}
	return append([]byte(nil), d.acc[:]...)
}

// Wipe zeroes everything this Deriver owns. See NewDeriver for what it cannot
// reach.
//
// done is reset so a post-Wipe Key() returns nil rather than 32 zero bytes.
// crypto.go:47-52 states the rule this obeys: "An all-zero key would be worse --
// it is a VALID AES key and hides the fault." Not reachable from unlockDerive,
// where Key()'s result is evaluated before the deferred Wipe runs, but this is a
// public seam and B2b will hold one of these across a timer.
func (d *Deriver) Wipe() {
	clear(d.u[:])
	clear(d.acc[:])
	d.mac.Reset()
	d.done = 0
}
```

### 3c. Tests first

`seal/pbkdf2_test.go`, new file.

```go
package seal

import (
	"bytes"
	"encoding/hex"
	"testing"
)

// keyed unpacks what a derivation needs from a fixture.
//
// Passphrase and DerivedKeyHex are *string on the fixture (seal/vectors_test.go:23,33)
// because vector E has NEITHER: it encrypts nothing, so no key exists.
// Returning ok == false rather than dereferencing keeps E in the loop as a
// deliberate skip instead of a nil panic.
func keyed(t *testing.T, v vector) (pass, salt, want []byte, ok bool) {
	t.Helper()
	if v.Passphrase == nil || v.DerivedKeyHex == nil {
		return nil, nil, nil, false
	}
	return []byte(*v.Passphrase), mustHex(t, v.SaltHex), mustHex(t, *v.DerivedKeyHex), true
}

// The chunked deriver must be BYTE-IDENTICAL to the one-shot DeriveKey the
// vectors pin. Asserted against the vector file's derived_key_hex literals
// rather than against DeriveKey alone, because agreeing with a wrong
// implementation is not a result.
func TestDeriverReproducesEveryVectorKey(t *testing.T) {
	var checked int
	for _, v := range loadVectors(t) {
		pass, salt, want, ok := keyed(t, v)
		if !ok {
			continue
		}
		d := NewDeriver(pass, salt, int(v.Iterations))
		for !d.Step(1000) {
		}
		if got := d.Key(); !bytes.Equal(got, want) {
			t.Errorf("vector %s: chunked key %s, want %s",
				v.Name, hex.EncodeToString(got), hex.EncodeToString(want))
		}
		d.Wipe()
		checked++
	}
	// Six of the seven vectors carry a key; only E does not. Asserted so a
	// fixture that silently lost its keys cannot leave this test green while
	// checking nothing.
	if checked != 6 {
		t.Fatalf("checked %d vectors, want 6", checked)
	}
}

// The step size must not change the result. A deriver that resynchronised on a
// block boundary, or that double-counted the U_1 NewDeriver performs, would
// agree with itself at one step size and disagree at another.
func TestDeriverIsStepSizeIndependent(t *testing.T) {
	v := vectorNamed(t, "A")
	pass, salt, want, ok := keyed(t, v)
	if !ok {
		t.Fatal("vector A carries no derived key")
	}
	for _, step := range []int{1, 2, 7, 499, 500, 100000, 1 << 20} {
		d := NewDeriver(pass, salt, int(v.Iterations))
		for !d.Step(step) {
		}
		if got := d.Key(); !bytes.Equal(got, want) {
			t.Errorf("step %d: key %s, want %s",
				step, hex.EncodeToString(got), hex.EncodeToString(want))
		}
		d.Wipe()
	}
}

// Vector B is iterations = 100001 where A is 100000. A deriver that treated the
// count as a constant, or that was off by one against DeriveKey, passes A and
// fails here.
func TestDeriverHonoursTheIterationCount(t *testing.T) {
	a, b := vectorNamed(t, "A"), vectorNamed(t, "B")
	if a.Iterations == b.Iterations {
		t.Fatal("vectors A and B no longer differ in iteration count; this test proves nothing")
	}
	pa, sa, wa, oka := keyed(t, a)
	pb, sb, wb, okb := keyed(t, b)
	if !oka || !okb {
		t.Fatal("vectors A and B must both carry a derived key")
	}
	da := NewDeriver(pa, sa, int(a.Iterations))
	for !da.Step(4096) {
	}
	db := NewDeriver(pb, sb, int(b.Iterations))
	for !db.Step(4096) {
	}
	if bytes.Equal(da.Key(), db.Key()) {
		t.Fatal("one iteration of difference produced the same key")
	}
	if !bytes.Equal(da.Key(), wa) || !bytes.Equal(db.Key(), wb) {
		t.Fatal("a derived key does not match its own vector")
	}
}

// An incomplete derivation must yield nil, never a partial accumulator. A short
// key is not a slightly-wrong key: it fails ~31 s later as a tag mismatch
// indistinguishable from a wrong passphrase.
func TestDeriverWithholdsAnIncompleteKey(t *testing.T) {
	v := vectorNamed(t, "A")
	pass, salt, _, ok := keyed(t, v)
	if !ok {
		t.Fatal("vector A carries no derived key")
	}
	d := NewDeriver(pass, salt, int(v.Iterations))
	if d.Step(10) {
		t.Fatalf("10 of %d iterations reported complete", v.Iterations)
	}
	if k := d.Key(); k != nil {
		t.Fatalf("an incomplete deriver returned a %d-byte key", len(k))
	}
	if d.Done() != 11 {
		t.Fatalf("Done reports %d after NewDeriver's U_1 plus Step(10), want 11", d.Done())
	}
}

// Wipe must zero what it owns, and Key's copy must survive it — the property
// that lets a caller `defer d.Wipe()` at construction.
func TestDeriverWipeLeavesTheReturnedKeyIntact(t *testing.T) {
	v := vectorNamed(t, "A")
	pass, salt, want, ok := keyed(t, v)
	if !ok {
		t.Fatal("vector A carries no derived key")
	}
	d := NewDeriver(pass, salt, int(v.Iterations))
	for !d.Step(4096) {
	}
	key := d.Key()
	d.Wipe()
	if !bytes.Equal(key, want) {
		t.Fatal("Wipe zeroed the key it had already handed out")
	}
	zero := make([]byte, len(d.acc))
	if !bytes.Equal(d.acc[:], zero) {
		t.Fatal("Wipe left the accumulator non-zero")
	}
	if !bytes.Equal(d.u[:], zero) {
		t.Fatal("Wipe left the U buffer non-zero")
	}
	// A wiped Deriver must not hand out 32 zero bytes: that is a VALID AES key
	// and it hides the fault (the rule crypto.go:47-52 states for DeriveKey).
	if k := d.Key(); k != nil {
		t.Fatalf("Key() after Wipe returned %d bytes, want nil", len(k))
	}
}
```

> **`vector` needs `DerivedKeyHex`, `SaltHex`, `Passphrase` and `Iterations`.**
> All four already exist on `seal`'s own fixture struct (`seal/vectors_test.go:21-38`).
> The **`gui`** side's narrower `sealTestVector` (`gui/unlock_program_test.go:37-46`)
> does **not** carry `passphrase` or `iterations` — Task 5 adds them, and must,
> because `seal/testdata/README.md:19-24` forbids retyping vector constants
> ("retyped constants are how a port silently forks").

**Steps:**

- [ ] **3.1** Write `seal/pbkdf2_test.go`. Run
      `nix develop --command go test ./seal/ -run Deriver`. Expect FAIL:
      `undefined: NewDeriver`.
- [ ] **3.2** Write `seal/pbkdf2.go`. Re-run. Expect PASS.
- [ ] **3.3** Mutation check.

      | mutant | must be killed by |
      | --- | --- |
      | `d.done = 1` → `d.done = 0` in `NewDeriver` | `TestDeriverReproducesEveryVectorKey` |
      | `d.acc[j] ^= d.u[j]` → `d.acc[j] = d.u[j]` | `TestDeriverReproducesEveryVectorKey` |
      | `d.mac.Write([]byte{0,0,0,1})` → `{0,0,0,0}` | `TestDeriverReproducesEveryVectorKey` |
      | `d.mac.Reset()` dropped from `Step` | `TestDeriverReproducesEveryVectorKey` |
      | `Key()` returns `d.acc[:]` instead of a copy | `TestDeriverWipeLeavesTheReturnedKeyIntact` |
      | `Key()`'s incomplete guard removed | `TestDeriverWithholdsAnIncompleteKey` |
      | `total` ignored, hardcoded to 100000 | `TestDeriverHonoursTheIterationCount` |

- [ ] **3.4** Timing sanity, on the host, so the step size is not a guess:
      `nix develop --command go test ./seal/ -run TestDeriverIsStepSizeIndependent -v`
      and record the wall clock. Record the number; do not describe it.
- [ ] **3.5** Commit.

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
		ctx.Frame(op.Layer(
			nav,
			titleOp,
			pctOp.Offset(image.Pt((dims.X-pctSz.X)/2, (dims.Y-pctSz.Y)/2-leadSz.Y)),
			leadOp.Offset(image.Pt((dims.X-leadSz.X)/2, (dims.Y+pctSz.Y)/2)),
			op.Color(&ctx.B, th.Background),
		))
		// Ask for the next frame immediately: this loop IS the work, and a
		// deadline in the future would idle the KDF instead of running it.
		ctx.WakeupAt(time.Now())
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

> **This fixture is a machine-checked claim, not a sketch.** Task 5.1 asserts it
> round-trips through `seal.Opener.Inspect` + `UnlockWithKey` before any test
> depends on it — if `sealBlobForTest` and production disagree about the format,
> every Task 5–7 test is measuring the fixture rather than the code.
- **`TestSealedPayloadStopsAtATerminalScreen` (`gui/unlock_flow_test.go:211`)
  asserts the exact behaviour this task removes.** It is replaced, not deleted:
  the new assertion is that a sealed payload reaches the *passphrase* screen and
  that cancelling it returns to the menu without constructing the plate list.

**Steps:**

- [ ] **5.1** Write `seal/unlock_key.go` + `seal/session.go` and their tests
      (`UnlockWithKey` reproduces `Unlock` on vectors A–D, F, G; `ErrNotSealed`
      on E; `SecretsResident` true after unlock and false after wiping each).
      Run `nix develop --command go test ./seal/`.
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

      **Budget the frame count.** `unlockDerive` draws one frame per
      `kdfStepIterations = 500`, so a full 100,000-iteration unlock needs **≥200
      frames** before the next screen exists. The house idiom is
      `pumpUntil(frame, want, 32)` — 32 is far too few here and would look like a
      hang. Either pump ≥256 or, better, have the counting `newDeriver` return a
      deriver over a small iteration count so the test measures the flow rather
      than the arithmetic (R0 round 0, N1).
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
// guards. Re-cutting then needs a fresh unlock -- twelve words and a ~31 s KDF.
// That is the price, it is deliberate (operator, 2026-08-07, reaffirmed
// 2026-08-08), and it costs no reboot: the sealed blob is untouched in flash.

// unlockSecretHook is a test-only seam. It observes each stage with the record's
// live bytes, so a test can assert on the BUFFER -- that the record is non-zero
// when offered and zero once its plate has left -- rather than on a return
// value, which cannot tell a wipe from a missing wipe. nil in production.
// Mirrors unlockEngraveHook, the sanctioned in-file seam.
var unlockSecretHook func(stage string, idx int, record []byte)

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
	// bip39.Parse returns a SECOND copy of the secret as []Word. seal's copy is
	// zeroed by the caller's defer; this one is this function's to zero, and
	// clear() reaches []Word where wipeBytes ([]byte) does not compile.
	defer clear(m)
	// §6c FLIPS THIS to &SeedScreen{NoEdit: true}. It is written as the plain
	// constructor here for one reason: NoEdit is added to SeedScreen by a
	// FRAGMENT in gui/gui.go, and a whole-file block that referenced it would
	// not type-check against the unmodified fork -- the plan's build gate would
	// fail for a reason that is not a defect. The two-line flip is §6c's, and it
	// is a reviewer's execution pass rather than a machine-checked one. Do not
	// ship without it: on a touch-only SH2 a CENTRE TAP opens the word editor,
	// and editing an authoritative payload seed produces a self-consistent plate
	// that does not restore the payload's wallet.
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
	// after. m is zeroed by this function's own defer; rec is seal's buffer.
	clear(rec)
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

and where `editBtn` is added to the nav slots, skip it when `s.NoEdit`. Note
`editBtn` carries `AltButton: Center`, so on a touch-only SH2 a **centre tap**
opens the editor — this is not a hard-to-reach affordance.

Then flip the one call site in `gui/unlock_session.go`:

```go
	ss := &SeedScreen{NoEdit: true}
```

> **These three edits are FRAGMENTS and are therefore unchecked by the build
> gate** — §6b's whole-file block deliberately writes `new(SeedScreen)` so it
> type-checks against the unmodified fork. This is the one place in the plan
> where a whole-file block and a fragment must be applied together to be
> correct, and it is called out here so it is not half-applied.

**Test:** with `NoEdit` set, tapping the centre of the seed screen does not reach
word entry; with it clear, it still does (the existing scan-path behaviour, which
must not regress).

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
  `unlockPlateLabel(e.rec, e.idx, e.sealed, e.cut)` **each frame**, so the "(cut)"
  mark appears as soon as a plate completes.
- `unlockEngraveFlow` returns `bool` (did the plate complete), and the OK branch
  sets `plates[sel].cut = true` on true.
- The nav's first slot becomes
  `{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconDiscard}`.

**Steps:**

- [ ] **7.1** Tests first: vector C (encrypted-only cards reach the list), vector
      G (public cards span four cards across pages), a `sealForTest` payload with
      cards in **both** sections (the `(sealed)` suffix appears and the two
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
| passphrase prompted when `ct_len == 0` | vector E reaches the plate list with the word entry **never entered**, asserted by instrumenting the entry point, not by return value |
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

Both gates apply and **both MUST be run before dispatch and after every fold.**

- **`scripts/plan-cite-gate.sh <plan>` resolves every `file:line` and
  `pkg.Symbol` in this plan against the real source and prints the line.** Its
  stated blind spot: it cannot tell you the line *says* what the plan claims.

  **It exits 1 on this plan, with exactly two failures, and both are expected:**

  ```
  FAIL  seal.Deriver     no func/type/const/var Deriver in seal/
  FAIL  seal.IsSecret    no func/type/const/var IsSecret in seal/
  ```

  Both are symbols **this plan creates** (Tasks 3 and 5). The gate has a
  `skip` branch for a whole package a plan creates, but not for a new symbol in
  an existing one. **Any third failure is real.** Every other citation resolves,
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
`gui/unlock_kdf.go` (247 lines), `gui/unlock_session.go` (184) and
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

## What B2a does NOT cover

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
- **F-83 — the derived plate is not wipeable (owning phase: ownerless residue).**
  `validateMdmk`, `backup.SeedString`, `engraveSeed` and `toPlate` all copy the
  record into Go strings and `Plate.Spline`, none of which can be zeroed; the
  existing `gui/ms1_decode.go:19-20` carries the same caveat for the display
  path. B2a zeroes the buffers it owns and drops the rest. Closing this properly
  means a plate pipeline over `[]byte`, which is a much larger change than any
  phase of this feature.

### Record defects to fix while touching these files

- **`design/FOLLOWUPS.md`: F-73 and F-74 are marked CLOSED in their headings but
  still sit above the `## Resolved` marker**, unlike F-67–F-70 which were moved.
  Move them.
- **`CONTINUITY_2026-08-08.md` §9 says "F-80's two B2 items".** There are
  **three** with an explicit `owning phase: B2` — the `layoutMainPager` pin, the
  Back icon, and the "already cut" marks. Two of the three land in B2a per the
  2026-08-08 decision; only the pixel pin is B2b's.
