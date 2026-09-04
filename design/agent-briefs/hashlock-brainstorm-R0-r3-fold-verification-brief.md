# Brief: hashlock-phrase brainstorm, R0 round 3 -- fold verification of the security-software fold (sonnet, single agent, read-only)

You verify a FOLD. You do not re-review the design and you do not re-derive
measured facts. READ-ONLY except for the one report you write at the end.

## The one question

**Did the fold commit `c20ec9e` address each of the 20 findings of the round-2
security-software report exactly as its own dispositions table (record section
7.1) claims, without introducing a new defect, a contradiction, or a wrong
citation?**

## Inputs (repo `/scratch/code/shibboleth/mnemonic-engrave`)

1. The round-2 report, persisted at `e9d7895`:
   `design/agent-reports/hashlock-brainstorm-R0-r2-security-software-expert.md`
   (C-1..C-4, I-1..I-6, M-1..M-7, N-1..N-3; "Test plan additions" table;
   "Questions for the operator" 1-5).
2. The fold: `git diff e9d7895..c20ec9e -- design/BRAINSTORM_hashlock_phrase.md`
   and the folded record `design/BRAINSTORM_hashlock_phrase.md` at `c20ec9e`;
   section 7.1 is the dispositions table you check.
3. The fold commit message: `git show -s --format=%B c20ec9e`.
4. For citation checks: mnemonic-secret `/scratch/code/shibboleth/mnemonic-secret`
   (`7fc1e58`) and the fork `/scratch/code/shibboleth/seedhammer` (`70008da5`).

## Settled -- do not re-litigate

- Rulings L1-L22 in section 2 are the operator's. Three were made in response
  to this report and are dispositions, not folds to second-guess: L20 (C-1:
  `--in` = the ms1; two phrase channels; ms1-shaped phrases refused), L21
  (C-3: `--random` requires `--out` or `--json`), L22 (I-4 + C-2: H1b in
  engrave first, `DecodeMS1` unchanged, no new class). Verify the record
  STATES them and that every section follows them (4.1, 4.2, 4.4, 4.5, 4.6,
  5, 7.1); do not argue the choices.
- The controller's machine-checks in the fold commit message and in section
  7.1 were run; re-running any of them is fine, a match is not a finding.

## What to check, per finding

For each of C-1..C-4, I-1..I-6, M-1..M-7, N-1..N-3 and the reviewer's
question 5: find the fold's text (section 7.1 names the section), and judge
FIXED / PARTIAL (a superseded or dependent sentence elsewhere still carries
the old design -- grep the whole record; name the line) / NOT.

Specific propagation hazards to grep for, because the first two folds were
each caught on one: any surviving sentence that (a) says `--in` carries the
phrase or lists three phrase channels; (b) says the fork adds a `0x03` arm to
`DecodeMS1`, "every seed call site refusing it by name", or a "new secret
class"; (c) places `me`'s classifier change in H3 or after H2; (d) lets
`--random` run without `--out`/`--json`; (e) says the four `unreachable!`
sites are all refusals (r1's finding -- must still read decode/combine print,
payload_lang refuses); (f) the test-plan additions in 4.6 -- does every row of
the report's "Test plan additions" table appear, with its mutation, and at the
right stage (H1 / H1b / H2 / H4)?

Then three cross-cutting checks:

1. **Citations the fold introduced.** `argv_guard.rs` `SUBCOMMANDS: [&str; 12]`,
   `override_applies`, `flag_class`, `is_ms1_shaped`, the `--in FILE` remedy
   line; `parse.rs` `read_input` / `read_phrase_input` / `read_in_file` and
   `Source`'s `channel: ""`; `rust.yml` job platforms; the five `DecodeMS1`
   callers by file:line; `gui/codex32_polish.go` `showSecret`;
   `sysw/classify.go:48`; `gui/sysw_admit.go` `admits`;
   `gui/unlock_kdf.go:242` `unlockDerive` signature and `seal/wire.go`
   `SaltLen = 16`; `pbkdf2 0.12.2`'s default features (`~/.cargo/registry`).
   Each exists and says what the record says, or it is a finding.
2. **Numbers.** The hardened H `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`
   for `correct horse battery staple` (recompute with `python3 hashlib`);
   "six" admissions of `ClassCodex32Secret`; "five" callers; "[&str; 12]".
   Paste command and output.
3. **Contradictions.** Any two sentences in the folded record that disagree.

## Severity

Critical / Important / Minor / Nit as the constellation uses them; a
surviving sentence that contradicts a ruling is Important; a stale duplicate
that does not is Minor; a wrong citation is Important.

## Output -- write this file as your FINAL action, then return a 4-line summary plus the path

`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-brainstorm-R0-r3-fold-verification.md`

Structure, exactly:

1. Header: date, model, `git rev-parse --short HEAD`, the two commits compared.
2. One counts line `FIXED:<n> PARTIAL:<n> NOT:<n>` and one `C:<n> I:<n> M:<n> N:<n>` for new findings.
3. A table: finding | verdict | where in the record (line) | note.
4. Citations and numbers: each check with command + output.
5. New findings, numbered, each with claim / evidence (line) / remedy.
6. "Confirmed clean" -- the cross-cutting checks that passed, one line each.

Rules: edit nothing; never read any `*.jsonl`; no fresh audit of the design or
the codebases; do not run cargo builds.
