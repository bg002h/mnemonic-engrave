# Continuity — 2026-08-11b, after the post-release review round and post-merge polish

Supersedes `CONTINUITY_2026-08-11.md`, **which contains one error**: it lists
F-110 among the items "closed during the cycle". F-110 was never closed. The
triage reviewer refuted it from the shipped code's own comments; see F-110's
entry. Read this file instead.

Carries only what cannot be re-derived from the repo.

## 1. Where everything is

| thing | state |
| --- | --- |
| `mnemonic-engrave` `master` | `ef574c2`, pushed, `test (rust + go)` green |
| released | `v0.5.0` (unchanged — see §4) |
| fork `bg002h/seedhammer` `main` | `97e38c1`, pushed, `Test` + `Build image` green |
| fork tag | `fork-v0.0.0-g93ee004` (unchanged) |
| hardware last flashed | `v0.0.0-g747cf48`, validated 2026-08-10 — **now well behind `main`** |

Rust suite 185 passed / 0 failed. Fork host suite exit 0, 49 ok, 0 FAIL.
`gofmt -l` (excluding `third_party/`) is **0** — it was 6 all cycle, so
"gofmt is clean" is usable as a gate again for the first time.

Merged this round: `seed-residue-pins`, `font-rendering-polish`,
`f103-effective-input`, `f114-severity`. All deletable.

## 2. The Critical, because it is the one thing a user could have lost money to

**`me seal` was emitting backups the device cannot open.** A BIP-39 mnemonic
record was *accepted* on its whitespace-normalised form and *emitted* as
supplied; the device splits on a single ASCII space. Double space, TAB, NBSP,
VTAB, ideographic space and newline all sealed with exit 0 and were refused on
the machine **after the ~31 s KDF**, shown as §6.4 "payload unreadable" — which
§2.2 item 4 teaches the operator to read as **tampering**. Fixed in `ad8f95f`.

**It was the surviving half of a fix already made.** The uppercase guard one
line above argues this exact case in its own comment — `normalise` lowercases
*and* collapses whitespace — and only the case half was acted on. Generalises:
**when a fix is justified by a two-part property, check both parts.**

## 3. What the review round changed, in one place

Six reviews, each persisted to `design/agent-reports/` and committed verbatim
before its fold. The full record is in `FOLLOWUPS.md`'s
*"Reconciliation — 2026-08-11"* section; the short version:

- **fable whole-Phase-2: 0C / 0I**, and it delivered the copy inventory —
  35 resting places traced from the single entry point, 2 named by no prior
  report. It found no reachable-after-wipe defect.
- **F-109 answered and DOWNGRADED to Minor.** No secret in the residue, against
  controls whose first version scored zero because it was a static literal —
  the exact false negative a control exists to catch. ~13.5 KB of the "missing"
  35 KB was probe placement, not residue. **Host Go, not TinyGo** — that caveat
  is why it is downgraded rather than closed.
- **F-103 CLOSED**, F-122 filed for what it left open.
- **F-114 CLOSED as not-a-defect**; **F-121 filed** — the emulator does not home.
- **F-78, F-86, F-95, F-119, F-87, F-94, F-104 item 2 CLOSED.**
- Four Rust Importants closed: unscrubbed secret records, `normalise` leaking
  three allocations, `bip39`'s `zeroize` feature off, the untested mk1
  pristine-BCH guard.

## 4. Decisions waiting on the operator

1. **The published `v0.5.0` archives self-report `me 0.4.0`.** The tag was cut
   with `Cargo.toml` at `0.4.0`. Source is fixed; the *published* artifacts are
   not, because re-tagging a public release is an outward-facing call. Options:
   re-tag `v0.5.0`, or let `v0.5.1` carry the correct string.
2. **Hardware has not run any of this.** The machine is on `g747cf48`; `main` is
   `97e38c1` with the idle-input change, the font regeneration, and the seed
   pins on top. The idle change and the `%` glyph are both operator-visible.
3. **The closure candidates from triage are recorded, not applied** — F-75,
   F-60, F-63, F-72, F-82, F-71, plus three bullets. Each carries a one-command
   check. F-65, F-66 and F-76 are now *due* (both gating conditions met).

## 5. What is still open, ranked by what it would cost to be wrong

- **F-110** — overdue, both halves named as open holes by the shipped code
  (`gui/engraver.go:126-132`, `engrave/engrave.go:1722-1730`, the latter
  carrying a measurement: 4 orphaned arrays → 23 arrays / 119,891 knots).
- **F-122** — a flickering panel still produces genuine edges. Wants a **bench
  capture of the `ft6x36` stream under a film**, which is free and has been
  recorded-but-never-run twice now.
- **F-120** — the device admits 27 codex32 lengths in 48–90, `me` admits 10,
  **22 diverge**; reverse set empty, so it cannot produce an unopenable backup.
  Needs a design call, and the narrowing lives in `ms-codec`, not here.
- **F-88 / F-90 / F-104 item 2b** — the remaining unwipeable-garbage class, plus
  one un-inventoried sibling copy in `LastWordCandidates`.
- **F-115** — 68 of 175 citations in `FOLLOWUPS.md` use forms the cite gate's
  regex never attempts, so its silence over that file is not coverage.

## 6. Process facts this round earned

- **Never judge a command by a piped `tail`'s exit status.** The fork's host
  suite had been red since 2026-08-07 and **CI failed on the exact SHA that was
  tagged and released**, unseen, because every local run was
  `go test ./... 2>&1 | tail -40`. This is `empty-output-is-not-absence` in a
  new costume: not a search that failed to look, but a *failure that could not
  reach the status it was judged by*. I then repeated the shape in a polling
  loop that treated an empty API response as "finished".
- **A brief's "settled facts" are not settled if the code disagrees.** I handed
  the triage reviewer a wrong anchor (F-110 closed) and it refuted me. Worth
  keeping in future briefs: *"treat these as settled unless the code says
  otherwise, and say so if it does."*
- **Fixing a cosmetic bug exposed three false-passing tests.** F-86's `%` had
  rendered as zero pixels; three unrelated tests' regexes had silently come to
  depend on that. They passed for the wrong reason for as long as the bug lived,
  and only the repair could reveal it.
- **A mutation that dies to exactly one test is the measure of a real gap.**
  Reverting F-103's predicate to the shipped `len(evts) > 0` is caught by one
  test in a 48-package suite — the new regression. The defect that could stop a
  funds-safety wipe had nothing standing against it.
- **A bulk edit must report its substitution count.** The regex that stamped
  closure markers onto headings fired twice for F-109, mangling a subsection
  heading. Counting caught it; a silent success would not have.
