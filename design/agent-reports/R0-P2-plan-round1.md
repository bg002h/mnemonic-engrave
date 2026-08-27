# R0 — `IMPLEMENTATION_PLAN_P2_ms_adopts.md`, round 1 (fold-check)

**Scope, as briefed:** did the fold at `61f7cd3` fix each round-0 finding, and
did it introduce a new defect? Not a fresh audit. Round-0's report (`802cd78`)
and the fold's own account (`FOLD-P2-round0.md`, in `9cf9f53`) are taken as
input; the diff `802cd78..61f7cd3` on `design/IMPLEMENTATION_PLAN_P2_ms_adopts.md`
is what was reviewed for propagation, plus independent re-measurement of every
load-bearing claim in the fold's report.

**Severity ruling applied** (operator, 2026-08-27): secret-handling harm
(argv leakage, missed spellings, material to stderr/history, world-readable
destinations) is logged, never gate-holding. A gate that cannot fail, a
refusal that does not refuse, a false PASS, or a tool claiming a path that
does not run still blocks. Applied throughout below.

**Build/environment.** `cargo build --locked` clean in both
`/scratch/code/shibboleth/mnemonic-secret` (7c12f66) and
`/scratch/code/shibboleth/mnemonic-engrave` (main checkout, for `me`). All
binaries invoked by absolute path
(`/scratch/code/shibboleth/mnemonic-secret/target/debug/ms`,
`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`), never the
`cargo`-installed `ms`. Exit codes captured to variables/files, never read
through a pipe.

**Already machine-checked on the merged tree (not re-run as discovery, per
brief):** `plan-table-check.sh` → 0 (56 rows, 0 malformed);
`plan-cite-check.sh` → 0 (38/38 resolved); `plan-stepref-check.sh` → 0; the
14-files/33-`Command::new`/0-`ms` recount for C-2's decline.

---

## PER-FINDING VERDICT

- **C-1** (`=`-joined bypass) — **LANDED.** Re-graded Critical → Minor under
  the ruling (correct application — this is exactly the "material leaks"
  class), then closed anyway with a real fix (fourth normalisation: split on
  every `=`). Logged as F-302, present in `design/FOLLOWUPS.md:12843`.
- **C-2** (`me`'s remedy is broken; row 8 called it correct) — **LANDED.**
  Split correctly: the blocking half (a refusal advising a path that does not
  run, and a gate whose stated basis — "the assertion `me`'s suite already
  makes" — does not exist) is folded, reclassified RED-first, and the entry
  MOVED past the ungrouped-stdout work. The logged half is F-301
  (`design/FOLLOWUPS.md:12796`), correctly marked "not the secret-handling
  class."
- **I-1** (§3's blocker discharged; row 4's gate cannot fail) — **LANDED.**
  §3 rewritten; the unfalsifiable gate replaced with four gates that can fail
  (manifest grep, a compiling `use` line, a fresh-`CARGO_HOME` build, full
  suite).
- **I-2** (override can't satisfy its own gate on `encode`/`split`) —
  **LANDED.** Mechanism changed from "remove" to "substitute with `-`," which
  clears the required-`ArgGroup` failure while staying inside §6d's actual
  wording. New assertions (stdin-closed control, override-alone-still-64
  control) are gates that can fail.
- **I-3** (no private route for `derive` phrase+passphrase) — **LANDED, with
  the "no private form" reasoning correctly declined**, not the finding. The
  residue is now named in §2.5/§8 (F-303); the two-command counter-route is
  gated on matching fingerprint, not bare `rc 0`.
- **I-4** (ungrouped-stdout gate unsatisfiable, twice) — **LANDED.** Both
  sites now read `ms encode --in <file>` with "no flags" correctly scoped to
  `me sysw pack`, matching SPEC §7's own wording.
- **I-5** (`repair`'s stdout is report+artifact, gate can't tell readings
  apart) — **LANDED.** `--out` on `repair` now ruled (artifact line alone,
  report stays on stdout), with a byte-pinned control. F-285 in
  `design/FOLLOWUPS.md` corrected too (see below).
- **I-6** (purge command has no allowlist) — **LANDED.** 12-word allowlist
  added with a broad-match fallback and a mistyped-verb gate row that is RUN.
- **I-7** (stale `me-cli` citations, 24 lines off) — **LANDED.** Both
  re-located by `git grep` on the emitted string and described rather than
  written back as numbers.
- **I-8** (0600 `--out` vs. `ms`'s own 0644 advisory) — **LANDED.** Out-of-scope
  bullet + F-304 (owning phase P3), with the byte-parity constraint stated as
  the reason P2 does not act.
- **M-1** ("six single-channel verbs") — **LANDED.**
- **M-2** ("eleven" over a list of twelve) — **LANDED.**
- **M-3** (driver column mislabelled/residue wrong) — **LANDED.**
- **M-4** (condition 15 misses a `src/` test) — **LANDED.**
- **M-5** (`channel::destination`: right verdict, wrong reason) — **LANDED.**
- **M-6** ("material's own characters" unsatisfiable) — **LANDED.**
- **M-7** (exit 4 asserted, absent from §1.1) — **LANDED.**
- **M-8** (56 rows extrapolated from 12) — **LANDED.**

**18 of 18: LANDED. 0 NOT LANDED. 0 PARTIAL. 0 WRONGLY FIXED.**

---

## INDEPENDENT RE-MEASUREMENT OF THE FOLD'S LOAD-BEARING CLAIMS

**1. The 92-row cross-product.** Rebuilt the full generator independently
(9 flag channels × 4 spellings × 2 join forms + 5 positional × 4 = 92,
confirmed against §1.4's 9-flag/5-positional inventory by hand-count) and ran
it against the tree's build. Result: **92 total, 84 exit 0, 8 exit 1, 0 of 92
leak** — exact match to the fold's numbers. Broke the 84 down further: **58
silent, 26 carrying `derive`'s advisory only** — exact match. The 8 non-zero
rows are UPPERCASE `--phrase` on `encode`/`verify`/`split`/`derive`, both join
forms, all exiting 1 with `unknown BIP-39 word at position 0` and no leak —
confirms these can only ever assert the guard's own refusal text, as the fold
claims.

**2. C-2's move.** Reproduced the emitted line verbatim:
`ms encode --phrase - < seed.txt | me sysw pack --out p.bin` → `me` rc 4,
`record 0 ... is not a form this container can place`, no `p.bin` written.
Reproduced the live control: `--group-size 0` + `me sysw pack
--no-passphrase --out p.bin` → rc 0, **102-byte** file at `-rw-------`
(0600) — exact match to both the round-0 report's and the fold's numbers.
Since the failure is caused by grouping (a property of `ms encode`'s default
output, independent of which flag supplies the phrase), the move's premise —
"at the old position neither the old nor the new advice runs" — holds
regardless of whether `--in` exists yet. Grepped the folded plan for `row 8`,
the old §3 blocker language, the old `channel::destination` reasoning, the
stale `2188`/`2184` line numbers, the old "stdout IS a canonical artifact"
phrasing, and the old "removes both" override wording — **zero hits on all
six**, confirming no superseded text survives the reorder or the other fixes.

**3. Declined reasonings.**
   - **Derive two-command route.** Reproduced independently with my own seed
     and passphrase (not the plan's): `ms derive --phrase <seed> --passphrase
     <pass>` and the two-command route `ms encode --phrase - --group-size 0`
     → card → `ms derive <card> --passphrase-stdin < pass.txt` both report
     **master_fingerprint: 64493bc6** — identical. The route is proved
     equivalent, not merely runnable, exactly as the fold claims.
   - **14-file count.** Pre-verified per brief; not re-derived.

**4. Recomputed figures.**
   - Driver lines/invocations: re-ran `grep -c '"\$MS"'` and `grep -o
     '"\$MS"' | wc -l` over all seven scripts independently — **18 lines, 20
     invocations**, exact match to the plan's table, including the two
     scripts (`derive-rcw-keys.sh`, `derive-hashvault-keys.sh`) where they
     diverge. Residue arithmetic (20 − 13 = 7, matching the enumerated
     2+3+2) checks out.
   - `ms --help`'s command list: **12** words exactly (`derive`, `encode`,
     `decode`, `inspect`, `verify`, `vectors`, `gui-schema`, `gen-man`,
     `repair`, `split`, `combine`, `help`) — matches row 7's allowlist
     verbatim.
   - `ms repair --ms1 <induced error>`: reproduced independently with a
     freshly hand-corrupted `ms1` — rc 4, two `#`-prefixed report lines then
     the corrected artifact — matches I-5's claim exactly.
   - Regression-gated count: text now says "TWO," lists pin + decline by
     name, and states the sibling remedy is no longer one of them — no
     stray "three" or "56" language found elsewhere in the document.
   - `crates/me-cli/src/main.rs`: emitted line at **2164**, comment at
     **2160** — confirmed directly against the file. `scripts/plan-cite-check.sh`:
     `ROOTS` array opens at **95**, `mnemonic-secret` entry at **101** —
     confirmed directly.
   - `origin/master` = `6c24e62823e6c1ac02aa3862cd6020674bf58544`;
     `git ls-tree -d` lists both `crates/me-cli` and `crates/mnemonic-io-lib`;
     `git grep -n 'fn write_private' origin/master -- crates/` → exactly one
     hit, `mnemonic-io-lib/src/write.rs:45`; `remedy.rs` confirms fish
     prescribed. All match I-1's re-measurement.
   - `grep -c mnemonic-io-lib crates/ms-cli/Cargo.toml` → **0** today,
     confirming row 4's new gate is a real, currently-failing assertion.

**5. F-285 correction.** Confirmed present in the diff
(`design/FOLLOWUPS.md`): the entry now reads "the three verbs whose stdout
**CARRIES** a canonical `ms1` or share string" (was "IS the artifact"), with
an explicit "Corrected 2026-08-27 in the R0 round-0 fold of the P2 plan (its
I-5)" note — exactly as the fold's report claims, in a file the C-2/I-1..I-8
diff would not otherwise have touched.

**6. I-6's allowlist-fallback design has precedent.** Checked `me`'s own
`argv_surface` (`crates/me-cli/src/main.rs:411-420`): on an unrecognized
subcommand word the loop `break`s and falls back to bare `"me"`, the same
shape `ms`'s new row 7 adopts. Not a novel or unvetted design choice.

---

## NEW DEFECTS INTRODUCED BY THE FOLD

**None found.** Searched specifically for: superseded 56-row/old-order/old-§3
language (zero hits, six targeted greps above); a fix that closes an
Important by opening a Critical (none of the nine Important fixes changes
behavior in a way that fails a check the plan itself now states); a
newly-written gate that cannot fail (row 4's replacement gate is
demonstrably false today, per the Cargo.toml grep above); a new citation that
resolves to the wrong content (four spot-checked, all exact).

---

## COUNTS

**0C / 0I / 0M / 0Nit.**

This closes the loop: all 18 round-0 findings landed, both declines are
independently reproduced as correct, every recomputed figure in the fold's
report matches independent re-derivation exactly, and no new defect —
blocking or otherwise — was found in the fold's diff.
