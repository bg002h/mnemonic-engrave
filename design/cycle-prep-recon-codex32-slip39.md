# cycle-prep recon — 2026-06-17 — codex32-input-polish + slip39-input-enablement

**Source repo:** the SeedHammer fork `bg002h/seedhammer` (`/scratch/code/shibboleth/seedhammer`) — these are FIRMWARE cycles, not constellation CLIs.
**Fork HEAD (origin/main) at recon time:** `599ec9a`  ·  **upstream/main:** `86a58ab`
**Local branch:** `main`  ·  **Sync state:** up-to-date (0 ahead / 0 behind origin/main)  ·  **Untracked:** none (worktree clean)
**Underlying recon:** `design/RECON_seedhammer_slip39_codex32_input.md` (ultracode workflow, source-verified vs SLIP-0039 + BIP-93).

Slug(s) verified: `codex32-input-polish`, `slip39-input-enablement`. **Drift expectation: NONE** — the recon was written against this same fork tree minutes earlier; this pass re-confirms every load-bearing citation and adds the brainstorm-gate framing. (No `design/FOLLOWUPS.md` slug exists for either — the "WHAT" derives from the recon + the maintainer's stated PR-#34 closing concerns.)

---

## Per-slug verification

### codex32-input-polish
- **WHAT:** polish the (already fork-enabled) on-device CODEX32 entry — error-class feedback, char-counter/live-field-parse, pre-engrave confirmation, Button3-accept, keyboard charset — and (separately) add the missing multi-share k-of-n recovery.
- **Citations (all vs fork main `599ec9a`):**
  - `gui/gui.go:623` `func inputCodex32Flow` — **ACCURATE**.
  - `gui/gui.go:624` keypad alphabet `"1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` — **ACCURATE**.
  - `gui/gui.go:633` `codex32.New(kbd.Fragment)` per-keystroke validation — **ACCURATE**.
  - `gui/gui.go:628` `okBtn := &Clickable{Button: Button2}` (OK on Button2) — **ACCURATE** (see Cross-cutting #1 re Button3).
  - `gui/gui.go:672` title static `"Input Codex32 Share"` — **ACCURATE**.
  - `gui/gui.go:1872` menu `"CODEX32"` **uncommented** (fork-enabled by #34) — **ACCURATE**.
  - `gui/gui.go:1724` `case codex32.String:` → `gui/gui.go:1731` `backupSeedStringFlow` (verbatim TEXT+QR engrave; only `id` from `Split()` used) — **ACCURATE**.
  - **Multi-share gap:** `codex32.Interpolate` callers = `cmd/biptool/main.go:127,:334` only; **zero `gui` callers** — **ACCURATE** (confirms a single share is engraved verbatim, never reconstructed).
  - codex32 pkg: `New`@`codex32.go:98`, `Interpolate`@`:188`, `shortChecksumLen=13`/`longChecksumLen=15`@`:45-46` — **ACCURATE**.
  - `codex32/mdmk.go` is fork-only (PR #35), a DIFFERENT BCH scheme (`POLYMOD_INIT = 0x23181b3`, NUMS targets; pure verifiers, no interpolation) — **ACCURATE**; must not be conflated with codex32.
  - **Protocol facts** (bech32 charset, `ms1` HRP, threshold `0`/`2-9` with `1` invalid, index `s`=unshared, BCH short=13/long=15, GF(32) Lagrange) — spec-verified vs BIP-93 in the recon; **ACCURATE**. One **DIVERGENCE (low-severity, safe):** the firmware's long-code length gate (total 125–127) is narrower than BIP-93's data-part rule — over-restrictive, never mis-accepting.
- **Action for brainstorm spec:** base the cycle on **fork `main`** (`599ec9a`) — codex32 is only enabled there (#34). Two separable cycles (see scope). Cite source SHA `599ec9a`.

### slip39-input-enablement
- **WHAT:** enable on-device SLIP-39 share entry/recovery — currently fully disabled and missing its entire share-crypto layer.
- **Citations (all vs fork main `599ec9a`):**
  - `gui/gui.go:684` `func inputSLIP39Flow` (+ helpers `emptySLIP39Mnemonic`@`:503`, `completeSLIP39Word`@`:851`, `updateValidSLIP39Keys`@`:895`) — **ACCURATE** (live, but unreachable).
  - Disabled wiring: menu choice commented `gui/gui.go:1872` (`/* , "SLIP-39" */`); engrave `case slip39.Share:` commented `gui/gui.go:1693-1694`; `case 3` ParseShare commented `gui/gui.go:1904`; NFC-scan ParseShare commented `gui/scan.go:64` — **ACCURATE**.
  - **The entire SLIP-39 share crypto is ABSENT:** `slip39/slip39.go` exports only `Word`, `Mnemonic`, `LabelFor`, `ClosestWord`, `valid` — grep for `ParseShare|Share|RS1024|Shamir|CombineMnemonics|Interpolate` under `slip39/` returns **nothing** — **ACCURATE** (the in-tree package is wordlist-only).
  - go-slip39 resource comment `gui/scan.go:62` (`github.com/gavincarr/go-slip39 adds ~55kb ... unicode`) — **ACCURATE**; **no go-slip39 dependency in `go.mod`** — **ACCURATE**. The ~55KB is traceable to `regexp`→`unicode` for a trivial `^\d{3,6}$` check (spec-/proxy-verified in the recon).
  - 128-bit-only cap `const maximumLength = 20` — **ACCURATE** but note it lives in the **commented** engrave branch (`gui/gui.go:1697-1698`), i.e. an intended-but-inactive guard; 256-bit (33-word) shares wouldn't fit the plate.
  - **Protocol facts** (1024-word list ✓ `slip39/wordlist.txt`; 20-word/128-bit, 33-word/256-bit; RS1024 over GF(1024); GF(256) Shamir; two-level group/member thresholds; Feistel/PBKDF2 encryption; ext-backup-flag 2024 amendment) — spec-verified vs SLIP-0039 in the recon; **ACCURATE**.
- **Action for brainstorm spec:** if pursued at all (see scope — recommended NOT to), base on fork `main`/`upstream/main` (disabled state identical). Cite source SHA `599ec9a`.

---

## Cross-cutting observations
1. **Base-branch / Slice-1 ordering (the most important).** Slice 1's BIP-39 polish (Button3-accept, "Word N of M" title, match-count, candidate-restricted keyboard) lives on `feat/bip39-entry-polish` (off `upstream/main`) and is **NOT merged to fork `main`** (PR #36 was closed). So fork main's input flows are still pre-Slice-1 (all three `okBtn` are Button2 at `:543`/`:628`/`:688`; static titles). A codex32-polish (or slip39) cycle off fork main therefore does **not** inherit Slice 1's polish. The brainstorm must decide: **(a)** first merge Slice 1 into fork main (so the shared keyboard + Button3 + title/match-count baseline is present), then build codex32-polish on top; or **(b)** branch off fork main and re-apply the relevant Slice-1 patterns within the codex32 cycle. Option (a) is cleaner (no duplication; the keyboard is shared).
2. **No FOLLOWUPS slug / no drift.** Neither cycle has a `design/FOLLOWUPS.md` entry; the recon is the source-of-intent and was written against this exact tree, so all citations re-confirm ACCURATE. No DRIFTED-by-N or STRUCTURALLY-WRONG findings.
3. **Constellation lockstep invariants are N/A** (firmware, not the m-format CLIs): no GUI `schema_mirror`, no `docs/manual` mirror. The relevant locksteps here are: the **shared `Keyboard` widget** (charset/confirm-button changes ripple across BIP-39/codex32/slip39 — Slice 1 already moved the keyboard to commit-on-Center), the **R0 gate**, and **on-device QA** (no hardware yet — touch/D-pad entry is unvalidated).
4. **codex32 multi-share recovery is a correctness gap, not polish** (Interpolate never reached from GUI). It must be its own cycle, separate from the cosmetic polish.
5. **No upstream PRs** (user directive after #36 closed): both cycles are fork-side only.

---

## Recommended brainstorm-session scope
- **Cycle A — codex32-input-polish (RECOMMENDED, do first).** Low-risk, **no crypto**: Button3-accept (or inherit via merging Slice 1 — see #1), live error-class feedback (map `codex32.New`'s currently-unexported sentinel errors), char-counter + live field parse (id/threshold/index), pre-engrave confirmation screen, keyboard-charset tidy. **Sizing ~S–M** (~150–250 LoC + tests). Touches `gui/gui.go` (`inputCodex32Flow`) + minor `codex32` API exports (error classifier / partial-parse helper). **Prereq decision:** resolve the Slice-1 base (#1) first.
- **Cycle B — codex32-multi-share-recovery (OPTIONAL, separate R0 slice).** The only correctness fix: multi-share collection UI + cross-share validation surfacing + `codex32.Interpolate(shares,'S')` + decide what to engrave (recovered seed as BIP-39/SeedQR vs the `S` codex32 secret). **Sizing ~L** (net-new flow design; crypto already present, no resource concern). Do AFTER Cycle A.
- **Cycle C — slip39-input-enablement (NOT RECOMMENDED / defer).** Requires porting+auditing an entire security-critical crypto layer (RS1024, GF(256) Shamir, Feistel/PBKDF2, two-level combine) — either integrate go-slip39 (substantiated ~55KB+3-module resource cost on RP2350/TinyGo) or write it in-tree, **plus** net-new multi-group UX, **plus** a plate-space limitation (256-bit shares don't fit). Both maintainer objections hold. **Sizing ~XL**, security-critical. Recommend deferring unless there's a concrete user need; if pursued, it demands the R0 gate + the full SLIP-0039 test-vector corpus.
- **SemVer:** firmware version is `-ldflags`-injected (no committed constant) — no source bump. These are additive fork features.
- **Ordering / dependencies:** (#1 Slice-1 base decision) → Cycle A → optionally Cycle B. Cycle C independent and deprioritized.

---

## Process note (project standard)
cycle-prep is recon only. Any spec/plan written for Cycle A/B/C MUST pass the opus-architect **R0 gate to 0C/0I before implementation** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Per the refined ultracode policy, the recon/design phases fan out with the source-verification guard; implementation is single-subagent + mandatory adversarial execution review.
