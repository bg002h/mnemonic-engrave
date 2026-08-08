# Continuity — 2026-08-08

Supersedes `CONTINUITY_2026-08-07b.md` for the **encrypted payload delivery**
feature. That doc handed off "host and device Phase A both shipped, Phase B not
started". Since then **Phase B1 was planned, gated, implemented, verified on the
SeedHammer II, and merged**, and **F-73 is closed**.

Its §3 operational traps and §5 do-not-re-open list are **superseded where they
concern F-73** and otherwise still current.

---

## 1. THE HEADLINE — B1 is merged; the device shows real payloads on real hardware

```
mnemonic-engrave  master  (design only this cycle)
seedhammer        main    78949e7   47 packages ok, 2 sanctioned setup failures
```

**Neither repo is pushed.** Both trees clean. See §6.

| | state |
| --- | --- |
| **Plan A** — host `me seal` / `me hash` | merged, pushed (previous cycle) |
| **Plan B Phase A** — device headless core | merged, pushed (previous cycle) |
| **Plan B Phase B1** — unsealed path, UI | **merged (`78949e7`), NOT pushed** |
| **Plan B Phase B2** — passphrase, KDF, session lifecycle | not started; needs its own plan + R0 gate |
| **F-73** | **CLOSED** on the SH2 |

## 2. WHAT B1 ACTUALLY DOES

`§10.2 steps 1–4` plus the plate list and engrave. Detection at GUI start →
a conditionally-shown **Sealed Payload** menu entry → `Inspect` → the §6.6 hash
screen → §10.2.3's unauthenticated warning (only when `ct_len == 0`) → a paged
plate list → engrave one public record.

**No secret is ever resident.** B1 derives no key and decrypts nothing, which is
why §10.2.2's wipe lifecycle and §10.2.4's idle timer are *out of scope* rather
than half-built — and why it was cheap to review.

A **sealed** payload stops at a terminal screen saying unlocking is unavailable.
It does **not** fall through to the plate list: `p.Public` on a sealed payload is
a legitimate record set, and engraving it while dropping the encrypted half is
§6.4's incomplete-backup-believed-complete.

New files: `gui/unlock_flow.go`, `gui/unlock_platelist.go` (+ tests),
`seal/grouping_test.go`. Modified: `gui/gui.go`, `seal/record.go`,
`cmd/controller/platform_sh2.go`, `cmd/emu/platform.go`, `gui/text_program_test.go`.

## 3. THE HARDWARE RESULT — F-73 IS CLOSED

Full record: `design/HARDWARE_RESULT_2026-08-07_phaseB1.md`. Four checks on the
SH2 (chipid `0x77c483b745abf55c`, RP2350**B** QFN80, secure boot 1):

1. **Write + readback at `0x10E00000`, no firmware involved.** `MNEMBLOB`,
   `pub_len=1125` — independently right (12 records = 1114 chars + 11 LF) —
   `kdf`/`aead`/`iterations` all zero per §6.2's unsealed rule. **Impossible on
   the 4 MB Pico**, so it is new information.
2. **Region erased** → entry absent, 8 dots, every other program reachable,
   **Engrave Bundle unmoved in slot 5**.
3. **Payload loaded** → entry present, 9 dots, "12 records", UNSEALED, the
   §10.2.3 warning, and the hash `fc10 4898 39dc 6da3 8f56 575d 45f7 655b`
   **byte-identical to the host's**.
4. **Erased again** → entry gone. The menu tracks region state, not a cache.

**Why check 3 is the whole ballgame:** an aliased or erased read cannot produce a
matching 128-bit digest. So the hash match is *simultaneously* the proof that the
XIP read reaches the right 1125 bytes 14 MB into flash. It is also the first
end-to-end confirmation that §6.6 agrees across host and RP2350 silicon.

**Check 2 confirmed a design decision, not just an absence.** B1 knowingly
departed from the Phase A plan's carried "insert the program *before*
`bip85Derive`, not appended" and appended instead, moving the compile-time guard.
Had it inserted mid-enum, everything after would have shifted and Engrave Bundle
would have moved. It did not.

**`kdfbench` was NOT run, deliberately** (operator decision). Its own header says
to run it on a Pico 2 / Pico Plus 2 and **not** the SH2, and argues the rate
transfers because PBKDF2 is compute-bound and cache-resident while A/B differ
only in package, pins and flash banking. Both existing figures already came from
a Pico 2. SPEC §7.1 is amended: the rate is **still owed before release**, to be
confirmed **in situ during B2** by timing the real unlock KDF — stronger
measurement, different method.

## 4. THE LESSON THIS CYCLE TAUGHT — folds, and the part nobody asked for

Continuity §4 last cycle recorded that **most defects are authored by folds**.
This cycle sharpens it:

| round | verdict | who authored the defect |
| --- | --- | --- |
| R0 round 0 | 0C / 4I | the original draft |
| R0 round 1 | 0C / 1I | **round 0's fold** |
| R0 round 2 | 0C / 1I | **round 1's fold** |

Both fold-authored defects were in content **volunteered beyond what the reviewer
asked for** — not in the fixes themselves. Round 1's was a claim about
`AdmitSection`'s data flow that did not exist; round 2's was a claim that
encrypted-section records "can be `ms1` or a bare mnemonic, neither of which is a
card at all", which SPEC §6.3, `permitted()` and vectors C and F all contradict.

**The rule to carry: the dangerous part of a fold is the part nobody requested.**

### Tests that cannot fail — both whole-diff Importants were this

- **`"SEALED"` is a substring of `"UNSEALED"`**, and `uiContains` is
  `strings.Contains`. Hardcoding `unlockShape` to `return "UNSEALED"` passed the
  entire suite. The gap was **one-directional** — the opposite mutant *was*
  caught — which is exactly why it read as correct.
- **`sel = start` was unfalsifiable.** Deleting it passed the entire suite,
  because the test asserted that *some* variant screen appeared, never *which*
  record.

Both were found by mutation, neither by reading. Both fixes were verified by
**re-applying the mutant**, not by the test passing.

### Overclaiming commit messages — three more this session

`5e2ac7b` claimed `sel = start` was "mutation-checked". It was not. And I wrote
`gofmt clean` in `f4dbeed` after running:

```
gofmt -l <path> && echo "gofmt clean"
```

`gofmt -l` reports by **printing**, not by exit status — it returns 0 either way,
so the `&&` always fires. It printed the offending filename and then printed
"gofmt clean" one line below it. **Test the output, not the status:**
`out=$(gofmt -l …); [ -z "$out" ]`. Both corrected in the record rather than
amended away.

### Rendering is invisible to this test suite

`uiContains` (`gui/gui_test.go:516`) compares **extracted text, not pixels**. No
screen test in `gui` can catch a missing or mis-drawn glyph. That is how the
invisible `·` survived in four shipped files — it was found by *measuring width*.
See F-78.

## 5. WHAT TO DO NEXT

1. **Push both repos** — see §6, this is the first thing.
2. **Plan B Phase B2** — passphrase entry, checksum on the §8.1-normalised form,
   the ~31 s KDF with progress, the retry loop keeping the hash on screen,
   §10.2.2's secrets-first lifecycle, §10.2.4's residency wipe. Own plan, own R0
   gate. **F-77 is GATING for it.** Carry forward:
   - **§10.2.4's timer does not exist in a flow-visible form.** `idleTimeout`
     (`gui/gui.go:2801`) drives the *screensaver* from `Run`'s frame loop and is
     invisible to flows. Chosen approach: a last-input timestamp on `Context`,
     with the engrave screen pausing the timer by simply not consulting it.
     `AppendEvents` (`cmd/controller/platform_sh2.go:368`) appends only on touch
     or stdin, so `a.idle.start` is a true last-physical-input time.
   - **Timer semantics, settled by operator decision 2026-08-07:** warning at
     3:00, wipe at 3:30, **paused while engraving** (§10.2.4 row 2).
   - `seedEntryFlow` returns `bip39.Mnemonic` = `[]Word` = `[]int` — scrubbed by
     manual zeroing, **not** `wipeBytes`.
   - Validate the BIP-39 checksum on the §8.1-normalised form **before** the KDF.
3. **Open follow-ups:** F-58…F-66, F-71, F-72, F-74, F-75, F-76, **F-77 (gating
   B2)**, F-78, **F-79 (fix before an operator sees it)**, F-80.
   **F-67…F-70 and F-73 are CLOSED.**

## 6. NOTHING IS PUSHED

Both repos have unpushed work. This is the largest loose end.

- `seedhammer` `main` — B1 and the merge (`78949e7`), plus two `cmd/sealread`
  commits.
- `mnemonic-engrave` `master` — the B1 plan and its three R0 rounds, the
  whole-diff review, `HARDWARE_INVENTORY.md`, `HARDWARE_RESULT_…`, the SPEC
  amendments (§10.3 Back-IS-Lock, §7.1's method change), F-74…F-80, and two
  tooling fixes.

Note from the previous cycle, still true: **pushes to `mnemonic-engrave` `master`
bypass a required status check** (`test (rust + go)`). Land future work via PR if
that check should actually gate.

## 7. STANDING CHANGES MADE THIS SESSION

- **`scripts/plan-cite-gate.sh` resolves `const` and `var`**, not just
  `func`/`type`. It was reporting a false FAIL on every constant — most of
  `seal`'s surface. Grouped declarations print as `ok*`, deliberately looser and
  labelled so.
- **`scripts/sh2-flash` resolves `nix`** instead of assuming it on `PATH`. Bare
  `nix` resolves in an interactive shell and not in the non-interactive one every
  agent and CI run uses, and the failure text accused the devshell.
- **`design/HARDWARE_INVENTORY.md` is new** — one home for board chipids.
  **Three** RP2350s answer to `2e8a:000f` here, and on 2026-08-07 two were in
  BOOTSEL simultaneously. The Pico 2 W (`0xb3d19289d3ec3f0e`) is the easy
  mistake: same form factor and 4 MB as the rehearsal Pico, but `secure boot: 0`
  so it runs unsigned images, and its LED is elsewhere.
- **SPEC §10.3 amended** — the plate list's three nav slots are Back / Page / OK,
  and **Back IS Lock**. The prior text demanded Back/Lock/OK *and*
  `bundleReviewFlow`'s paged shape, which is four affordances into a `[3]int`.
- **The plate list separator is `|`, not `·`** (operator decision) — `·` has no
  glyph and measures zero pixels.
- **Re-review is opt-in now** (user directive 2026-08-07): *ask before a second
  round of review*. Report, fold, gate, commit — then stop and ask. A round 2
  was skipped on this basis with nothing gating open.
