# Encrypted Payload Delivery — Plan B Phase B2a-i (the headless substrate) — Implementation Plan

**Status:** GREEN for its content. **Inherits the R0 loop below; the split is
editorial except where marked.**

| round | verdict | report |
| --- | --- | --- |
| 0 | 1C / 4I / 6M / 3N — all folded | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round0.md` |
| 1 | 0C / 2I / 4M / 3N — all folded | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round1.md` |
| 2 | 0C / 1I / 0M / 2N — all folded, **GREEN** | `design/agent-reports/encrypted-payload-planB-phaseB2a-R0-round2.md` |

**Provenance.** This document and its sibling `…_phaseB2a_ii.md` are the two
halves of `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md`, which went
through the three R0 rounds above and closed GREEN at commit `861c99a`. The task
numbering is **unchanged** (this file holds Tasks 1–3, its sibling Tasks 4–9) so
that every finding in those three reports still resolves against a task number.

**One substantive change came with the split and did NOT inherit that GREEN** —
Task 3 now folds `DeriveKey` onto the `Deriver` rather than leaving two PBKDF2
implementations in one package. It is marked **[SPLIT-NEW]** where it appears and
is the only thing in this document a re-review needs to look at.

---

## Why this phase exists, and what makes it cheap to review

**Nothing in B2a-i can decrypt.** No key is derived into a cipher, no AEAD is
opened, and no secret record is ever resident. That is the same property that
made B1 cheap to review, and it is the reason the seam is here rather than
anywhere else in the nine tasks.

| | B2a-i (this plan) | B2a-ii | B2b |
| --- | --- | --- | --- |
| **F-77** encrypted-section card grouping | ✅ **gating** | — | — |
| **F-79** the 64 KB retention | ✅ | — | — |
| the chunked KDF (§10.2 step 7's engine) | ✅ | — | — |
| §10.2 steps 5–9 — words, progress, AEAD open, retry | — | ✅ | — |
| §10.2.2 secrets-first session lifecycle | — | ✅ | — |
| the plate list after the session | — | ✅ | — |
| §7.1's in-situ KDF rate on RP2350B | — | ✅ | — |
| §10.2.4 residency-keyed idle wipe | — | — | ✅ |

**The `Deriver` derives a key and nothing opens with it.** `crypto/aes` and
`crypto/cipher` are already imported by Phase A's `seal/crypto.go`, so B2a-i adds
no new primitive and no new reachable decrypt path. `Opener.UnlockWithKey` — the
function that turns a key into plaintext records — is **B2a-ii's**, deliberately,
because a phase that could decrypt without §10.2.2's session lifecycle would
leave seed material resident with nothing managing it. That is the same argument
`CONTINUITY_2026-08-08.md` used to refuse splitting unlock from the session, and
it binds here too.

**Each of the three tasks stands alone and is independently shippable.** F-77 is
additive metadata; F-79 is a retention fix with an operator-visible benefit and
no behaviour change; the `Deriver` is vector-pinned substrate. None depends on
another, so they can be reviewed and merged in any order — though F-77 is marked
gating because B2a-ii cannot label a secret plate without it.

---

## Decisions taken before this plan (operator, 2026-08-08)

All six are recorded in both halves because they are the context for the whole
of B2a. **Decisions 1 and 2 are the ones this phase implements**; 3, 4 and 5 are
B2a-ii's and appear here only so a reader is not sent to another document to
understand why the substrate looks as it does.

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
- **The chunked path's KDF seam is `newDeriver`, and it is B2a-ii's** (that
  phase's §5c), not this one's and not `Opener.KDF`. B2a derives through `seal.NewDeriver` and opens
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
	// for the whole flow -- B2a-ii's `blob = nil` would rebind the local and
	// release nothing, and F-79 would be reported closed while unfixed in
	// exactly the payload-present-plus-running-engrave configuration B2a-ii's
	// Task 9.5 exists to test. The closure reads `blob` at EXIT instead.
	defer func() { clear(blob) }()
	var o seal.Opener
	p, err := o.Inspect(blob)
```

**And the blob is released before the engrave**, not merely at the end — see
**B2a-ii's Task 5**, which sets `blob = nil` after `UnlockWithKey` returns, its
last use. In B2a-i the deferred closure is the whole of the fix: the sealed path
stops at B1's terminal screen, so there is no session to release it before.

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
> does **not** carry `passphrase` or `iterations` — **B2a-ii's** Task 5 adds
> them, and must,
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
### 3d. [SPLIT-NEW] `DeriveKey` becomes a wrapper — one PBKDF2, not two

**This is the only part of B2a-i that did NOT inherit the R0 GREEN.** It came
with the phase split and a re-review should look here.

Leaving `DeriveKey` on `crypto/pbkdf2.Key` and adding `Deriver` beside it puts
**two implementations of the same primitive in one package**, which must then
agree forever. That is precisely the divergence this codebase rejects everywhere
else — it is F-77's whole argument for reusing `groupCards` rather than
re-deriving grouping in `gui`, and it is what `Opener.Inspect`'s doc comment
exists to prevent ("Do NOT re-implement steps 1-3 in Phase B"). It also matters
more here than there: the two would be a *cryptographic* pair, and a divergence
between them is a wrong key, which surfaces as a tag mismatch indistinguishable
from a wrong passphrase.

Fold `DeriveKey` onto the `Deriver`, in `seal/crypto.go`:

```go
// DeriveKey runs PBKDF2-HMAC-SHA256 over the §8.1-normalised passphrase.
//
// It is the one-shot form of Deriver (pbkdf2.go), and it is a WRAPPER rather
// than a second implementation: two PBKDF2s in one package must agree forever,
// and a divergence between them is a wrong key that surfaces as a tag mismatch
// indistinguishable from a wrong passphrase. The vectors pin the pair.
//
// iterations ALWAYS comes from the header — never a constant, or vector B fails.
func DeriveKey(passphrase string, salt []byte, iterations int) []byte {
	d := NewDeriver([]byte(passphrase), salt, iterations)
	defer d.Wipe()
	// One call: Step never runs past total, so the largest legal iteration
	// count (§6.2's 2,000,000) completes here in a single pass.
	d.Step(iterations)
	return d.Key()
}
```

**`crypto/pbkdf2` AND `crypto/sha256` both leave `seal/crypto.go`'s import
block** — measured, not predicted: dropping only the first fails with
`"crypto/sha256" imported and not used`, because its sole use in that file was
the `sha256.New` argument to `pbkdf2.Key`. `seal/pbkdf2.go` keeps both.

Two further consequences, both deliberate:

- **The error branch disappears, and with it the `nil` return.** The old body
  returned `nil` on `pbkdf2.Key`'s error — "unreachable with an out-of-range
  iteration count or key length, and ParseHeader has already excluded both". The
  `Deriver` cannot fail: it clamps `iterations < 1` and its key length is fixed
  at `sha256.Size` by a compile-time assertion. `Key()` still returns `nil` on an
  incomplete derivation, which preserves the property `crypto.go:47-52` argues
  for — an all-zero key "is a VALID AES key and hides the fault".
- **The stdlib is no longer an independent cross-check** of the chunked loop.
  That is acceptable because the authority was never the stdlib: it is the six
  `derived_key_hex` literals in `testdata/vectors.json`, produced by the Rust
  implementation, which `TestDeriverReproducesEveryVectorKey` asserts directly.

- [ ] **3.5** Apply §3d. Run `nix develop --command go test ./seal/` — **every
      existing vector and crypto test must pass UNCHANGED**; they are what proves
      the wrapper did not fork the primitive. Confirm `crypto/pbkdf2` no longer
      appears in `seal`'s imports (`grep -rn 'crypto/pbkdf2' seal/`).
- [ ] **3.6** Mutation check the wrapper. **`d.Step(iterations - 1)` is NOT a
      mutant** — measured: it survives the whole suite, because `NewDeriver`
      already performed iteration 1, so `Step(total-1)` reaches exactly `total`.
      It is an equivalent implementation, and naming it here would have reported
      a surviving mutant against correct code. Use a real one: **`d.Step(iterations - 2)`**
      leaves the derivation one iteration short, so `Key()` returns `nil` and the
      payload fails closed — measured, **14 tests fail**. `d.Step(1)` likewise
      fails 14. Record the count.
- [ ] **3.7** Commit.

---

---

## Gate coverage — state this in the R0 brief

Both gates apply to this document and **MUST be run before dispatch and after
every fold.**

- **`scripts/plan-cite-gate.sh`** resolves every `file:line` and `pkg.Symbol`
  against the real source and prints the line. Its stated blind spot: it cannot
  tell you the line *says* what the plan claims. Expect failures only for symbols
  this document creates (`seal.Deriver`, `seal.NewDeriver`).
- **`scripts/plan-build-gate-go.sh`** type-checks the whole-file Go. This
  document's new files — `seal/label_encrypted.go`, `seal/label_encrypted_test.go`,
  `seal/pbkdf2.go`, `seal/pbkdf2_test.go` — are assembled into a scratch copy of
  the fork and run through `go build` and a baselined `go vet`.
- **The FRAGMENTS are the reviewer's execution pass**: the `seal/record.go` call
  site, the `AdmittedRecord` doc comment, the `Reader`/`Probe` methods, §3d's
  `DeriveKey` body, and `gui/gui.go`'s `uiFlow` probe.

**Machine-verified, and EXECUTED, before this document reached a reviewer** — do
not re-derive:

- All five `Deriver` tests pass: six of seven vectors reproduce
  `derived_key_hex` byte-for-byte (E has no key) at step sizes 1, 2, 7, 499, 500,
  100000 and 2²⁰, and vector B's 100,001 iterations differ from A's 100,000.
- F-77's grouping on vector F yields three `mk1` cosigner cards of two plates
  each and one `md1` card of six — the shape §10.2.2's labels need.
- `TestUnreadableEncryptedCardDoesNotReject` was **mutation-checked**: with the
  grouping error propagated it fails; the fixture it replaced passed under that
  same mutant.
- **§3d's wrapper**: with `DeriveKey` folded onto the `Deriver` and both imports
  dropped, `go test ./seal/` over the crypto, vector, wire, container and open
  suites is **clean**. Mutants `d.Step(iterations - 2)` and `d.Step(1)` each fail
  **14 tests**; `d.Step(iterations - 1)` is an equivalent implementation and
  correctly fails none.
- Host KDF cost, measured: `DeriveKey` at 100,000 iterations is **13.1 ms**, at
  300,000 **35.5 ms**; the chunked deriver covers 100,000 in **10.2 ms across
  200 `Step` calls**.

**Not covered:** no `gui` code in this document was executed. Task 2's `uiFlow`
and `unlockPayloadFlow` changes compile and vet; no screen was driven.

---

## What B2a-i does NOT cover

- **Everything that decrypts.** `Opener.UnlockWithKey`, the passphrase entry, the
  progress screen, the retry loop, §10.2.2's session and the extended plate list
  are **B2a-ii's**. This phase derives a key and opens nothing with it.
- **§10.2.4's residency-keyed idle wipe.** B2b.
- **F-76** — inspecting a payload-sourced card.
- **Any release tag.**

## Follow-ups filed by this document

- **F-82 — `seal.Deriver` has no Rust counterpart (owning phase: ownerless
  residue).** The chunked derivation is device-only: the host has no progress bar
  to draw. It produces byte-identical output, pinned by six vectors, so the
  Rust-primary rule does not bind it. Recorded so a future reader does not
  mistake the asymmetry for drift. **§3d widens this slightly** — `DeriveKey` is
  now also the chunked loop, so `seal` no longer calls `crypto/pbkdf2` at all
  while the Rust side still uses its own PBKDF2. Same reasoning applies: the
  vectors are the contract, not the implementation.
