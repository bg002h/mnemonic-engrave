# Brief: hashlock-phrase brainstorm, R0 round 1 -- fold verification (sonnet, single agent, read-only)

You verify a FOLD. You do not re-review the design, and you do not re-derive
facts already measured. You are READ-ONLY except for the one report you write
at the end.

## The one question

**Did the fold commit `d2e8f68` address each of the 15 findings of the round-0
report exactly as its own dispositions table claims, without introducing a new
defect, a contradiction, or a wrong number?**

## Inputs (repo `/scratch/code/shibboleth/mnemonic-engrave`)

1. The round-0 report, persisted at `d13819e`:
   `design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`
   (C-1, I-1..I-6, M-1..M-6, N-1, N-2; plus "Questions for the operator" 1-5).
2. The fold: `git diff d13819e..d2e8f68 -- design/BRAINSTORM_hashlock_phrase.md design/FOLLOWUPS.md`
   and the folded record `design/BRAINSTORM_hashlock_phrase.md` at `d2e8f68`,
   whose section 7 is the dispositions table you are checking.
3. The fold commit message: `git show -s --format=%B d2e8f68`.

## Settled -- do not re-litigate

- Rulings L1-L14 in section 2 are the operator's. Three were made IN RESPONSE
  to this report and are dispositions, not folds to second-guess: L12 (C-1:
  sha256 warns always, never refuses), L13 (I-1: no `--salt` this cycle), L14
  (I-4: preimage singles carry id `hash`). Verify that the record STATES them
  and that the copy and sections follow them; do not argue the choice.
- The controller's machine-checks in the fold commit message and in section 7
  were run; you may re-run any of them (they are one-liners) but a match is
  not a finding.

## What to check, per finding

For each of C-1, I-1..I-6, M-1..M-6, N-1, N-2 and reviewer questions 2 and 5:
find the fold's text for it (section 7 names the section), and judge:

- **FIXED** -- the record now says what the disposition claims, in the place
  it claims, and nothing elsewhere in the record still says the superseded
  thing.
- **PARTIAL** -- addressed in one place, but a duplicate or dependent sentence
  elsewhere still carries the old claim (grep the whole record for the old
  phrasing; name the line).
- **NOT** -- the disposition claims something the text does not do.

Then three cross-cutting checks:

1. **Numbers.** Recompute every figure the fold introduced from the report's
   cited rates (8,865.7 kH/s at 999 iterations; 21,975.5 MH/s; 2 bits per
   character): the 8.9e4 and 1.1e10 guesses/s, the 124,060 ratio, 72 days at
   40 bits hardened, 50 seconds sha256, 12,900 years at 56 bits hardened, 38
   days sha256, 13.5 hours for 2^32, "six diceware words is ~77 bits"
   (12.925 bits per word), the entr/mnem/preimage length sets
   (50/56/62/69/75, 51/58/64/70/77, 75), "16..46 payload bytes". Paste the
   command and output for each; a mismatch is a finding with the right value.
2. **Contradictions.** Any two sentences in the folded record that disagree
   (e.g. a floor stated as a refusal in one place and a warning in another; the
   id `entr` still claimed for preimage singles anywhere; `[u8; 32]` without
   `Zeroizing` anywhere; "try both" surviving; a coordinator still named in
   3.5).
3. **New defects.** Anything the fold's new text asserts that is false or
   unsupported -- check its citations (`crates/ms-cli/src/cmd/verify.rs:99`,
   `derive.rs:434`, the four `_ => unreachable` sites, `seal/wire.rs:13,17`,
   BIP-93's bracket) exist and say what the record says.

## Severity

Critical / Important / Minor / Nit as the constellation uses them; a wrong
number in a security floor is Important; a stale duplicate sentence is Minor
unless it contradicts a ruling (then Important).

## Output -- write this file as your FINAL action, then return a 4-line summary plus the path

`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-brainstorm-R0-r1-fold-verification.md`

Structure, exactly:

1. Header: date, model, `git rev-parse --short HEAD`, the two commits compared.
2. One counts line: `FIXED:<n> PARTIAL:<n> NOT:<n>` and one `C:<n> I:<n> M:<n> N:<n>` for new findings.
3. A table: finding | verdict | where in the record (line) | note.
4. Numbers: each recomputation with command + output.
5. New findings, numbered, each with claim / evidence (line) / remedy.
6. "Confirmed clean" -- the cross-cutting checks that passed, one line each.

Rules: edit nothing; never read any `*.jsonl`; no fresh audit of the design,
the codebase, or the sections the record marks as still to walk (4.4-4.6).
