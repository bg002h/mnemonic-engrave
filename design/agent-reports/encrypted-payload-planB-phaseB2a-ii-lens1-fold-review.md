# B2a-ii — lens 1 C1 fold review (commit c785322)

Scope: **one question** — did the fold fix C1, and did it introduce a new defect?
Not a fresh audit. I1, M1, the `clear(blob)` mutant and F-86's `%` glyph were
declared out of scope and are not re-reported. `go test ./...`, vet and gofmt were
declared settled and were not re-run as a suite (I did run `./gui/` repeatedly as
the mutation harness; it was green at HEAD each time).

All mutation work was done in a private copy (`cp -a` of the worktree into the
session scratchpad), never in `/scratch/code/shibboleth/seedhammer-wt-b2aii`.
Copy deleted after use.

```
FOLD VERDICT: FIXED
NEW DEFECTS: 0 Critical / 0 Important / 1 Minor / 1 Nit
```

---

## 1. Is `clear(m)` correctly placed? — YES

`gui/unlock_session.go:221-226`:

```go
	clear(rec)
	clear(m)
	if unlockMnemonicHook != nil {
		unlockMnemonicHook(m)
	}
	NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
```

Every read of `m` is upstream of `:222`, and each one is finished with it:

| read | line | still needs `m` after? |
| --- | --- | --- |
| `ss.Confirm(ctx, th, m)` | `:189` | No. `SeedScreen` is `{NoEdit bool; selected int; words []Clickable}` (`gui/gui.go:2288-2299`) — it takes the mnemonic as a **parameter**, not a field, and stores no words. `ss` itself is dead after `:191`. |
| `masterFingerprintFor(m, …)` | `:196` | No. Returns `uint32` (`gui/gui.go:540-550`). |
| `engraveSeed(params, m, mfp)` | `:202` | No. It converts out (`words[i] = bip39.LabelFor(w)`, `seedqr.QR(m)`, `gui/gui.go:516-538`) and returns a `Plate`. |

`plate` does not alias `m`. `Plate` is `{Duration uint64; Spline bspline.Curve;
Conf engrave.StepperConfig}` (`gui/gui.go:488-514`) and `toPlate`
(`gui/gui.go:3016-3030`) builds it from `engrave.PlanEngraving(...)` **eagerly** —
`spline` and `attrs` are computed inside `toPlate`, nothing is deferred to engrave
time. So the whole geometry exists before `:222`, and nothing after `:222` reads
the words except the test hook.

Machine-checked, not reasoned: with the fold in place `go test ./gui/` is green,
i.e. the plate renders and the engrave screen comes up after `m` has been zeroed.

## 2. Are the three early returns still covered? — YES, and the fourth does not need covering

Enumerated from source:

| return | line | after `defer clear(m)` (`:183`)? | plate built? |
| --- | --- | --- | --- |
| `bip39.Parse` error | `:178` | **No — before it** | no |
| `!ss.Confirm(...)` | `:190` | yes | no |
| `masterFingerprintFor` error | `:199` | yes | no |
| `engraveSeed` error | `:205` | yes | no (returns `Plate{}`) |

The comment's "the three early returns above" counts the three that lie between
the defer and the clear, which is the correct count. The fourth (`:178`) precedes
the defer and needs no cover: `bip39.Parse` returns `nil, err` on **all three** of
its error paths (`bip39/bip39.go:261`, `:267`, `:272`), so there is no `m` to
zero. Verified by reading `Parse`, not from its name.

`defer clear(m)` also evaluates its argument at the `defer` statement, and `m` is
never reassigned, so the deferred call clears the same backing array. Double-zero
is a no-op. Claim sound.

## 3. Is the new test sound, and can it fail? — YES. Mutation-verified, five ways

`TestMnemonicWordsAreZeroWhenThePlateReachesEngrave`,
`gui/unlock_session_test.go:637-670`. Baseline: PASS (0.02s).

| # | mutation | result |
| --- | --- | --- |
| M1 | delete `clear(m)` at `:222` (the controller's claim) | **FAIL** — `word 0 is still 138 at Engrave entry` |
| M2 | keep `clear(m)` but fire the hook **before** it | **FAIL** — same message. The test pins the *ordering*, not merely the presence of a `clear`. |
| M3 | delete `defer clear(m)` at `:183`, keep the explicit clear | PASS (expected — see the note in §7) |
| M4 | hook aliases (`atEngrave = []bip39.Word(m)`) **and** `clear(m)` deleted | **FAIL** — the alias is defensive rather than load-bearing at this timing, because the defer has not fired while the engrave screen is up |
| M5 | revert the fold entirely (pre-fold code: defer only), run the **whole** `./gui/` package | **FAIL, exactly one test** — this one, 15.7s. Nothing else in the package changes state. |

M5 is the one that matters: the new test is a genuine regression test for C1 and it
is the *only* thing in `gui/` that distinguishes the pre-fold code from the folded
code.

**Could it pass for the wrong reason?** I checked each channel:

- *Hook never fired* → `atEngrave` stays `nil` → the `atEngrave == nil` Fatal. Also
  covered independently: `h.mustReach("Insert a blank plate")` would have failed
  first if `Engrave` were never entered.
- *Wrong vector / empty slice* → the `len(atEngrave) != 24` guard. Vector A
  (`seal/testdata/vectors.json`) is `"bacon" × 24`, and `bip39.LabelFor(138) ==
  "BACON"` (resolved by running it, not by reading the wordlist). This is the
  **ideal** vector for an all-zero assertion: index 0 is `ABANDON`, a real BIP-39
  word, so a vector like the classic `abandon×23 + about` would have satisfied 23
  of the 24 assertions vacuously. Vector A satisfies none of them — every one of
  the 24 words is 138.
- *Record already zero at the source* → `unlockedPayload` (`:98-102`) fatals if any
  record arrives zero, so the premise "the seed was ever there" is pinned upstream.
- *Copy taken at the wrong moment* → the hook copies with `append([]bip39.Word(nil),
  m...)` at the instant it fires, which is after `clear(m)` and immediately before
  `NewEngraveScreen`. M2 proves the instant is the asserted one.
- *Cross-test contamination* → `t.Cleanup` nils the hook, and there is **no**
  `t.Parallel` anywhere in `gui/` (grepped), so the package-level hook cannot race.
- *Screen-match ambiguity* → `"Engrave Seed"` is drawn once, `gui/gui.go:2517`
  (`layoutTitle(..., "Engrave Seed")`), inside `SeedScreen`. `uiContains`
  (`gui/gui_test.go:516-521`) strips spaces from the needle to match the
  space-free `ExtractText` output, which is why `"EngraveSeed"` is the right
  literal.

The two premise guards are **sufficient** for this test. `atEngrave == nil` and
`len == 24` together exclude "hook never fired", "hook fired with an empty or
truncated mnemonic", and "wrong vector"; the vector-A word value excludes the
vacuous pass.

## 4. Does `unlockMnemonicHook` leak in production? — NO

- `nil` in production. Grepped: the only references anywhere in the tree are the
  declaration (`gui/unlock_session.go:41-47`), the guarded call (`:223-225`), and
  the test's assignment + `t.Cleanup` (`gui/unlock_session_test.go:641`, `:644`).
  Nothing else reads or writes it.
- It fires **after** the wipe. The value it hands out is the already-zeroed slice,
  so even a hostile hook installed in production could not observe seed material
  through this seam. That is strictly better than `unlockSecretHook`, which by
  design is handed live bytes at the `"offered"` stage.

## 5. Volunteered claims — 1 Nit, 1 Minor; everything else true

Checked against source, not against each other:

| claim | verdict |
| --- | --- |
| `§10.2.4`'s `SecretsResident()` scans `p.Secret` only | **True** — `seal/session.go:29-41`, and it additionally skips non-`IsSecret` classes. |
| `p.Wipe()` scans `p.Secret` only | **Nit-false** — see N1. |
| `clear` is idempotent, the double-zero is free | **True.** |
| the defer covers the three early returns, where no plate was built | **True** — table in §2. |
| "THE DEFECT WAS THE PLAN'S. §6b carries `defer clear(m)` verbatim" | **True** — `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md:1176`, under `### 6b. The new file` (`:997`); the plan has no `clear(m)` before `Engrave`. |
| "mutation-checked: word 0 is still 138 (138 = bacon)" | **True** — reproduced independently (M1); `bip39.LabelFor(138) == "BACON"`, run. |
| "unlockMnemonicHook, mirroring unlockSecretHook" | **True.** |
| I1: `masterFingerprintFor` byte-identical at `421dca8` | **True** — `git show 421dca8:gui/gui.go` matches the current body exactly. |
| I1: "five callers" | **True** under the natural reading — six call sites, but `gui/gui.go:2154` and `:2163` are both inside `backupWalletFlow` (`:2149`), giving five distinct calling functions (`bip85.go:137`, `seedxor_polish.go:66`, `slip39_polish.go:432`, `unlock_session.go:196`, `backupWalletFlow`). |

### M1 (Minor) — the plan still carries the Critical, and nothing records that it must not

**WHERE** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md:1176`
(§6b), and the absence in `design/FOLLOWUPS.md`.

**DEFECT** The commit message states "The plan will be corrected separately." The
plan is unchanged — line 1176 is still `defer clear(m)` with no pre-`Engrave`
clear — and `grep` over `design/FOLLOWUPS.md` finds no entry for it (the only C1
hit at `:1126` is an unrelated R0 round-0 item). The promise exists only in a
commit message.

**EVIDENCE** `awk 'NR>=1165 && NR<=1205'` on the plan (quoted §6b body above);
`grep -n "clear(m)" design/FOLLOWUPS.md` → no match.

**CONSEQUENCE** §6b is a whole-file block written to be lifted verbatim. Anyone
re-deriving `unlock_session.go` from the plan — a later phase, a rework, a
reviewer checking code against plan — reproduces the exact Critical that took the
whole cut to find, and the plan will read as the authority. This is precisely the
"a decision agreed in a message dies with the agent" failure mode.

**FIX** Either correct §6b in place (add `clear(m)` beside `clear(rec)` at the
plan's `:1205` block and delete nothing else), or file a follow-up with an owning
phase. Correcting it in place is one line and removes the need to remember.

### N1 (Nit) — `p.Wipe()` does not scan `p.Secret` only

**WHERE** `gui/unlock_session.go:215-216` (the new comment), and the same sentence
in the commit message.

**DEFECT** "p.Wipe() and §10.2.4's `SecretsResident()` both scan `p.Secret` only".
`Payload.Wipe` (`seal/open.go:56-63`) loops `p.Secret` **and** `p.Public`.

**EVIDENCE**
```go
func (p *Payload) Wipe() {
	for _, r := range p.Secret { clear(r.Record) }
	for _, r := range p.Public { clear(r.Record) }
}
```

**CONSEQUENCE** None to behaviour — the load-bearing half of the claim (neither
reaches `m`, a GUI local) is true, and the C1 finding stated it correctly. But this
file's comments are read as a map of the wipe surface, and a future reader chasing
"what does `Wipe` cover" gets a wrong answer from a comment that is otherwise
unusually careful.

**FIX** "…both scan the payload's own records only, and `m` is not one of them."

## 6. Does the codex32 arm need the same treatment? — NO. Read, not assumed

`unlockEngraveCodex32` (`gui/unlock_session.go:130-166`) has **nothing clearable**
beyond `rec`, which it already clears at `:163`:

- `codex32.New(string(rec))` — `string(rec)` allocates an immutable Go string; the
  conversion itself is the copy and it cannot be zeroed.
- `codex32.String` is `struct { s string }` (`codex32/codex32.go:16-18`) — one
  immutable string, no byte slice, no `[]Word` analogue.
- `s.Split()` returns strings; `backup.SeedString{Title, Seed, Font}` holds strings.

There is no mutable buffer to which `clear` could be applied, so the fold correctly
left this arm alone. The arm's existing HONEST CAVEAT at `:125-129` already states
exactly this and is accurate.

## 7. What I checked and found sound

- `clear(m)` placement: after the last read, before `Engrave`, no aliasing into
  `plate`, `SeedScreen` or the engrave job. (§1)
- Early-return coverage complete, including the pre-defer path where `Parse`
  returns `nil`. (§2)
- The new test fails for the right reason under five mutations, including a full
  revert-the-fold run over the whole package. (§3)
- The hook is unreachable in production and is handed post-wipe data. (§4)
- No new imports, no behaviour change outside the two lines, no signature change.
  The diff is `+2` executable lines in `unlock_session.go` (`clear(m)` plus the
  guarded hook call) and one new test.
- **Observation, not a finding:** M3 shows nothing in the suite pins `defer
  clear(m)` — deleting it leaves the package green, because no test drives the
  Confirm-cancel / fingerprint-error / `engraveSeed`-error returns to a point
  where `m` can be observed. That gap predates the fold (the defer was already
  untestable), and the fold's comment now explicitly justifies keeping the defer,
  so the justification is unpinned. Not a defect in the fold; worth a Nit-level
  test only if someone is already in this file.
