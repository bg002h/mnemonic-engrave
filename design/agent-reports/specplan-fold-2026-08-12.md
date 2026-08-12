# Fold report — spec + plan, 2026-08-12

Folder: the review's own author (fable), folding
`design/agent-reports/journeys-fable-specplan-review.md` into
`design/SPEC_systemwide_payloads.md` and
`design/IMPLEMENTATION_PLAN_systemwide_payloads.md`. Documents only — no code
touched in either repo; both trees verified clean apart from the two documents
and this report.

## Gates

- `scripts/spec-check.py design/SPEC_systemwide_payloads.md` — **GREEN** (29
  citations resolved, 3 rules single-defined, bare terms clean, tests 1..23
  without gaps, all invariants hold).
- **The gate was RED on entry**, before any fold edit: three citations had
  drifted under the spec (`gui/gui.go:758` ×2 → `refreshCands` now at `:817`;
  `gui/derive_xpub.go:82` → `func seedEntryFlow` now at `:88`). Fixed in the
  fold. Residue for a tooling commit (not mine — the brief forbade touching
  code/scripts): the script's `EXPECT` table still pins the two old keys, which
  now match nothing, so those pins are dead — re-key them to `:817`/`:88` to
  restore content-pinning (pinned count dropped 10 → 7).
- `./scripts/plan-build-gate.sh` — **not required and not run**: neither
  document contains a ```` ```rust ```` block before or after the fold
  (verified by grep: 0 in each). New plan signatures use ```` ```text ````,
  matching the plan's existing convention.
- Green-line commands corrected in the plan were **executed before being
  written**: `cargo test -p mnemonic-engrave sysw` → 64 passed (the old line's
  `-p me-cli` is the directory, not the package, and cargo rejects it).

## The four rulings, folded

1. **ASCII passphrases narrowed (operator, 2026-08-12; already landed at
   `b34944d`).** §12.5 gains the token rule (every whitespace-separated token
   of the normalised string is a BIP-39 English word) plus a dated paragraph
   with the reason (the device only ever grew a word keyboard; an ASCII
   passphrase sealed an unopenable payload) and the explicitly-unchanged list
   (two words legal, below `[cliff]`, warn-only). The 215-byte cap's
   declared-on-both-sides-enforced-on-neither history is recorded there and in
   §6.2.2. Ripples folded: §1 decision 8 gains the round-trip's fourth leg;
   §8a marked SUPERSEDED (one keyboard is now sufficient by construction);
   §6's mode list, §6.2.1's mode table and its "below by definition" sentence
   corrected (a user-supplied five-word passphrase is now above `[cliff]`);
   §6.2.2's range row marked subsumed; §6.2.3's device half marked moot;
   §5.6's `--passphrase-ask` row updated. §13 row **D4**, marked as running
   the opposite direction from D1–D3 (it adds a refusal, buying openability).

2. **Post-engrave reminder dropped (operator, 2026-08-12; coverage.rs already
   marks test 13 `Dropped`).** §5.5's opening paragraph replaced with the
   withdrawal and its reason (no provenance survives `take()`; the session
   reshape not worth it). Spec test 13 rewritten as WITHDRAWN, number kept for
   id stability. §11's "it is a REMINDER" clause corrected to
   operator-initiated-and-unprompted. §13 row **D5**. Consequence folded with
   it: §3.2's "every screen names its source" is SCOPED to the screen where
   the record enters the program — the every-screen reading needed the same
   provenance plumbing the operator declined; the entry-screen reading needs
   none, which is what makes F3 buildable (plan stage 10d) without reopening
   the ruling.

3. **`admits()` — MY RULING: per-site hard-coding is the enforcement design;
   the table stays normative as the ORACLE.** §13 row **D7**, labelled "ruled
   at fold"; §3.3.2 gains the ruling block; §3.3.2a's "one function" is
   revised to "one TABLE" with the history kept.
   Reasoning: wiring `admits()` in would add a run-time check that **cannot
   fail** — every consumption site hard-codes one class and every hard-coded
   class is an admitted cell, so the call returns true at every site that
   exists, while the site that would need it (a future wrong one) can simply
   omit the call. The spec's own §3.1 argument (two functions beat a boolean
   because the wrong value can't compile) says structural beats behavioural
   here: a site that has no way to name a refused class cannot reach it. But I
   did NOT demote the table to "descriptive": a descriptive table records what
   code happens to do and drifts with it; this table still *governs* — a call
   site naming a non-admitted class is a defect — and plan stage 13d makes
   that a machine check (an AST test reconciling every `syswOffer`/`take` call
   site against `admitted`, the same trick as test 16). "Refused with a named
   reason" is re-read as absence-of-offer, with the reasons living in §3.3.2's
   notes. The review's reachability findings are recorded in the spec
   (admitted cells are permissions, not UI promises; the §3.1 seam-type vs
   Codex32Secret inconsistency is named, not papered over) and the
   carrier-ready cells get plan stage 13a–c.

4. **§5.3.2's decode gate — MY RULING: it survives, DEMOTED from refusal to
   flag input.** New §12.6 `[mdmk-decode]`: a `ClassMDMK` record is
   decode-confirmed only when its complete card set reassembles and decodes by
   the real decoder; anything else is UNCONFIRMED and counts as SECRET for
   flag evaluation — F1 fires on it in a plaintext container — and nothing is
   refused. §13 row **D6**; §5.3.2 rewritten around the reference; test 14
   rewritten to the demoted form with its false S-I placement recorded; §11
   gains the residual-risk bullet (a dismissed warning still leaves smuggled
   entropy in flash — the claim is the warning, never protection).
   Reasoning: withdrawal wholesale was the wrong answer because the gate is
   not a strength mechanism — it is what keeps F1 *truthful*, and §13
   explicitly kept F1's warning job; deleting the gate silently guts a kept
   control in exactly its adversarial case (the smuggling bypass EPD measured
   and closed). But the refusal cannot survive §13 either: it is a security
   mechanism whose only visible effect is stopping an operator, and — decisive
   — transcribed verbatim it REFUSES legitimate payloads, since a single card
   of a chunked set (which `bundleFlow` legitimately seeds with) cannot
   reassemble. "Warn, don't block" has an exact fitting shape here:
   fail-toward-the-flag. The device answers the question it can answer
   (confirmed / not confirmed) and warns on the rest; the accepted cost — an
   innocent partial set warns — is recorded in D6. The old §5.3.2 mechanism
   text (AdmitSection pass-3) is withdrawn with its reason: it transcribed
   `seal` machinery the sysw container never had; R1-I2's index lesson is kept
   as a quoted block. Rust-first per the Rust-primary rule: plan stage 7
   (Rust + vector S-J + coverage re-point + `pack`/`show` warnings), stage 8
   (Go port + flag wiring).

## The main job — the plan's missing stages, and the journey map

New stages, each with files, signatures, and executed-shape green commands:

- **7/8** — `[mdmk-decode]`, Rust then Go (ruling 4).
- **9** — §8c's count confirmation + Back ≠ `done` (`inputWordsFlow` regains
  the second return stage 5 promised and the implementation dropped; the
  confirmation screen lands in `sysw_load.go` before the KDF; test 22).
- **10** — the CONSUMING half of NFC: 10a `scan.go` `text:`/`pass:` cases
  (prefixes before sniffers), 10b the §3.1 `Scanned` option at
  `syswSeedPicker`, 10c `engraveObjectFlow` routing + object-accepting entries
  for Engrave Text / BIP-39 Password, 10d F3/F4 finally fireable (first
  production `srcNFC`). Explicit non-goal: the verify seam stays typed-only.
- **11** — the §5.3.2 erase item. Named plainly as the tree's FIRST flash
  write (grep: no erase/program call exists outside third_party), with a
  mandatory cheap hardware rehearsal and risk-set review before the first real
  erase; spec §5.5 gains the matching note (device erase = plain sector erase;
  the fills belong to `me sysw wipe`).
- **12** — §7's word-plate verify in `gui/plate_verify.go` (named so the
  existing test-16 AST scan covers it from birth), §7.2 menu verbatim, §7.2.1
  selection, §7.1.1 provenance rendering, tests 2/3/17; integration at the end
  of `backupWalletFlow`. Spec §7.2 gains the scoping note (word plates only —
  the bundle verifies re-derive and need every word).
- **13** — carrier-ready admitted cells (Cdx32→Backup Wallet at
  `newInputFlow`; Passph→seam via `passphraseFlow`; MDMK→Multisig at the two
  non-verify `bundleGatherFlow` sites, with the verify sites explicitly
  excluded) plus 13d's two reconciliation tests: the admission-oracle AST test
  (D7's mechanism) and the coverage-witness test (the review's closing
  recommendation — run against the pre-fold tree it reproduces the review's
  missing-test finding as a command).

**The journey map** is the structural fix and sits ahead of the stages: nine
journeys (J-A..J-I, letters following the review), every step naming its
owning stage or a recorded reason, with the rule stated that a step naming
neither is a plan defect. Two gaps are deliberately OPEN and recorded rather
than staged: Cdx32→seam (blocked on §3.1's seam type — a design change, not a
fold) and MDMK→Single-Sig (no carrier exists in that program; measured — the
non-verify gather sites are Multisig's and Bundle's). Both fail closed.

## Also folded (from the review's ranked findings)

- §5.6 `--allow-weak` row: stale "refuses with a non-zero exit" replaced with
  accepted-and-ignored per D3 (Important-6; the code was right).
- §5.6 gains the three shipped-but-undocumented flags (`--in`, `--iterations`,
  `--region`) and the delivery paragraph — the `picotool load --verify -t bin
  -o 0x10D00000` invocation, marked NOT YET REHEARSED (Minor-7): the review
  found the write step documented nowhere at all.
- §3.3.3 F2 row now cites the store's `weak` (the row omitted `sealed`; the
  code was right — review Q1).
- §8.3 test 5 rewritten to the D3 behaviour (same staleness class as the
  `--allow-weak` row; it still said "refuses").
- Plan stage 1/2 green lines corrected (wrong package name; missing
  `--region`) — both executed before being written down.
- Three drifted citations fixed (the red baseline above).

## Not folded, deliberately

Review Minors 1 (dead FROM PAYLOAD offer after a declined comparison), 2
(`engravePassphraseFlow` silent truncation), 4 (test 21's missing buffer), 5
(`session.weak` stored value vs its §3.2.1 definition), 8 (emulator sysw
source), 9 (cosmetics) are code-side findings outside this fold's mandate
(documents only) and outside the brief's fold list; they remain in the
persisted review for FOLLOWUPS triage. No FOLLOWUPS.md edit was made — the
brief scoped edits to the spec and the plan.

## Verification notes

Every load-bearing claim written into the documents was re-measured against
the trees before writing: `admits()` caller count (grep — definition and tests
only), `passphraseFlow`/`newInputFlow`/`engraveObjectFlow`/`inputWordsFlow`/
`syswSeedPicker` locations and the Back/`done` twin `return entered()` lines,
the `bundleGatherFlow` call-site split (verify vs non-verify), the absence of
any flash-write call in the fork, `decode_public_set`'s grouping shape and its
single-card refusal tests, `report_strength`'s actual output strings, the
crate name, and coverage.rs's Device/DeviceUnbuilt/Dropped rows.
