# F-674 review — independent verification of the four-item residue burn-down

Reviewer: sonnet, independent (no prior authorship on this cycle). Scope: the
four F-674 items as fixed in descriptor-mnemonic `6fc93083..b662be1c`
(md-codec 0.48.2 / md-cli 0.20.2) and mnemonic-engrave `57123659..9fd7f32c`.
Sources read: `design/FOLLOWUPS.md` F-674 entry, both diffs, both implementer
reports (`f674-dm-impl.md`, `f674-nunchuk-walk-impl.md`). Every claim below was
independently re-derived, not taken from the reports' prose, except where
explicitly marked "per report" (uncontested, non-machine-checkable narrative).

## 1. Item 3 — measured dates

**Citations in `design/evidence/measured-at.json` cross-checked against git,
independently, for every non-obvious date:**

| claim | independently verified |
|---|---|
| Liana 8.0 measured 2026-09-19 | `.tmp/fable-liana-parse-out.jsonl` mtime `2026-09-19 23:22:54 -0700` (confirmed via `ls -la --time-style=full-iso`); report commit `4298cae9` at `2026-09-20T00:00:09-07:00` (confirmed via `git log`) |
| Nunchuk measured 2026-09-19 | `.tmp/fable-nunchuk-harness-out.txt` mtime `2026-09-19 21:27:48 -0700`; report commit `60e089f6` at `2026-09-19T21:35:44-07:00` |
| Liana v15.0 measured 2026-09-20 | mtime `2026-09-20 09:32:32 -0700`; commit `606ab180` at `2026-09-20T09:34:00-07:00` |
| Core e2e measured 2026-09-23 (not -24) | `e2e-live-site-wallets.md:3` reads "Date 2026-09-23" (confirmed by reading the file); recording commit `4db2f17d` is `2026-09-23T20:09:35-07:00`, which is `2026-09-24` UTC — this is exactly the bug being fixed |
| F-640 record override, 2026-09-23 | commit `0c51d3cc` at `2026-09-23T02:32:10-07:00`, after stage-2 plan GREEN (`fa8bba6f`, `02:02:25-07:00`) |
| core-boundary, coord-compat-1b, f449-stage2/4 all 2026-09-23 | each report file grepped directly, states its own date or is corroborated by its recording commit timestamp |

All citations check out. The commit-time evidence alone (permanent, in git)
already establishes each date; the `.tmp` mtimes are corroborating, not
load-bearing — the implementer's own "Concerns" note flags this correctly and
I agree it is adequate as recorded (not a defect).

**Mechanism verified by reading `crates/md-codec/src/coordinator/mod.rs`:**
`verdicts()` merges consecutive verified versions into one run only while `at`
(which embeds `MeasuredAt`) is `PartialEq`-equal. Liana's `verified` list is
exactly `&["8.0", "15.0"]` (checked in `registry.rs`) — no version between
them. Before the fix both were (accidentally) dated 09-20, so they merged into
"8.0-15.0"; after the fix they carry different dates (09-19 vs 09-20) and no
longer merge, becoming two lines. Since there is no intermediate verified
version, splitting the span into `{8.0}` and `{15.0}` loses no coverage
information — it was never continuous. Verdict content (`ImportsAltered`,
`as_read.threshold == (2,4)`) is pinned unchanged on both new lines by the
updated test — **confirmed: split only, no verdict changed.**

**Machine-checks run independently, not reused from the reports:**
- Full test suite: `cargo nextest run --workspace --all-features` (toolchain
  1.85.0, `CARGO_TARGET_DIR` under `/scratch/code/shibboleth/.tmp/`) →
  **1569 passed, 0 failed, 4 skipped.**
- `./scripts/vendor-coord-evidence.sh --check /scratch/code/shibboleth/me-worktrees/f674`
  (current engrave HEAD `9fd7f32c`, which includes items 1+2's later commits)
  → **`vendor-coord-evidence: fresh`, exit 0.** Confirms the "engrave commit
  not compared, only input hashes" design actually holds against a moved HEAD.
- Re-ran the persisted `/scratch/code/shibboleth/.tmp/f674rev/corpus.sh`
  output (`corpus.out`) independently by reading it, not trusting the report's
  summary: **28/816 mainnet files differ, every line either a date move
  (09-24→09-23, 09-20→09-19) or the Liana 8.0/15.0 split — no verdict text
  changes.** Matches the report's numbers exactly.

## 2. Item 4 — network messages

Built md-cli 0.20.2 from the dm branch tip (`b662be1c`) with
`PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH` and
`CARGO_TARGET_DIR` under `/scratch/code/shibboleth/.tmp/`. `md --version` →
`md 0.20.2`. Ran live against the built binary (not just unit tests):

- `decompose`, mainnet xpub under `--network testnet|signet|regtest`: each
  refusal names the network **actually passed** — "but --network says
  testnet" / "signet" / "regtest" respectively (previously all said
  "testnet").
- `decompose`, testnet tpub under `--network mainnet`: "Re-run with
  `--network testnet`, `--network signet` or `--network regtest`" — correctly
  offers all three, since a tpub's version bytes cannot disambiguate them.
- `encode --key`, mainnet xpub under `--network testnet|signet|regtest`: each
  says "expected testnet|signet|regtest xpub version 043587CF, got
  0488B21E" — the network named matches the flag passed.
- `encode --key`, testnet tpub under `--network mainnet`: "expected mainnet
  xpub version 0488B21E, got 043587CF."

All four networks (mainnet, testnet, signet, regtest) confirmed named
correctly in both refusal directions, live, not just via the unit tests.
Independently reran the persisted signet/regtest decompose corpus diff:
**80/136 files differ, every line "says testnet" → "says signet"/"says
regtest" — no other change.**

## 3. Item 1 — the Nunchuk sentence (evidence + fork copy)

`design/evidence/f674-nunchuk-same-seed/verdicts.txt` (read directly):
**4 unsorted wallets REFUSE, 4 sorted wallets ACCEPT (addresses_vs_md=EQUAL
for all 4), all 8 PR-1746 control twins ACCEPT, mismatches: 0.** This
supports the claim exactly as stated: repeated seed changes nothing, key
order alone decides the outcome.

**Reproducibility spot-check (not just trusting "run.sh reproduced it"):**
copied `measure.py` + the committed `wallets.tsv` into a scratch dir, built
md-cli 0.20.2 (the dm branch's own release build) onto `PATH`, and re-ran
`measure.py` directly. Output: **byte-identical `verdicts.txt`** to what is
committed (same 8 rows, `mismatches: 0`). So the evidence is not sensitive to
the md-cli version bump this same cycle ships.

**Minor finding (not blocking):** `run.sh`'s own reproduction gate,
`md --version | grep -qx 'md 0.20.1'`, is an **exact** pin. Verified directly:
`echo "md 0.20.2" | grep -qx 'md 0.20.1'` fails. Since this same F-674 cycle
releases md-cli 0.20.2, `run.sh`'s documented reproduction path is stale on
day one — anyone following its own instructions after picking up 0.20.2 gets
an immediate refusal, even though (per the spot-check above) the underlying
measurement is unaffected. This is the same "exact pin goes stale on the next
release" class that item 4's own `transcript_composer.sh` fix (below) just
addressed for a different file — ironic that it wasn't applied here too. Not
a data-correctness defect (confirmed above), so it does not block; worth a
follow-up to relax it (`>= 0.20.1`, same idiom as the transcript gate).

**Fork copy:** grepped `seedhammer/gui/composer_copy.go` at the fork's current
HEAD (`0287e3a`, unchanged by this cycle) — the sentence "Nunchuk imports it
only when the keys happen to be in sorted order" is present verbatim in both
the standard body (line 224–225) and the same-seed body
(`composer_copy_test.go:72`, the "SAME SEED, SAME PATH" golden string).
Matches the claim: the fork copy needed no edit and is consistent with the
new SPEC paragraph and the evidence.

## 4. Item 2 — the `liana-same-seed` arm and the transcript gate

Ran both live against the fork emulator (`/scratch/code/shibboleth/seedhammer/cmd/emu`,
built with Go 1.26.7 from `/scratch/code/shibboleth/.toolchain/go/bin`):

- `python3 design/journeys/capture_composer.py --arm liana-same-seed --emu ...`
  → **rc=0**, leg `liana-same-seed` (22s), "the consent withholds the Liana
  claim for a same-seed seating" / "the re-entered key-path choice withholds
  the Liana claim", **"all legs matched the host."**
- `python3 design/journeys/capture_composer.py --arm both --emu ...` → **rc=0**,
  5 legs (keyed-A, keyed-B, keyless, liana, liana-same-seed), all with
  byte-for-byte engraved output matching the host, **"all legs matched the
  host."**

**Transcript gate** (`design/journeys/transcript_composer.sh`, changed in
`9fd7f32c`): read the diff — it replaced an exact `"md 0.19.0"` string compare
with `sort -V` over `("0.19.0", "$MD_VER")`, taking the head and comparing to
`"0.19.0"`. Independently tested the idiom against boundary cases not in the
diff's own comment: `0.19.0` (equal, pass), `0.20.2`/`0.20.10`/`1.0.0` (newer,
pass), `0.18.0`/`0.9.0` (older, correctly fail) — including the double-digit
minor-version case (`0.20.10`) and a major-version bump (`1.0.0`), both of
which a naive string or lexicographic compare could mishandle. The gate is
correct.

## 5. False-pass check

Independently (not reusing the implementer's own mutation description)
reverted `crates/md-cli/src/decompose/mod.rs`'s network-naming fix in
`check_network` — replaced `crate::parse::keys::network_name(network)` with a
match on `want`'s `NetworkKind` (the old, pre-fix behavior) — and ran:

```
cargo test -p md-cli --bin md network_mismatch_names_the_network_the_user_passed
```

Result: **FAILED**, panicking on the exact pre-fix message ("but --network
says testnet" under a `Network::Testnet` case where the un-mutated test
expects "signet"/"regtest" to appear). Confirms the fix is load-bearing and
the test would catch a regression. Change reverted; `git status --short` in
the dm worktree is clean afterward.

## Other observations (non-blocking)

- **`design/FOLLOWUPS.md`'s F-674 entry only annotates items 3 and 4 as
  "Fixed, not yet shipped."** Items 1 and 2 have no equivalent status note,
  even though `86d63288` and `f243bce9` demonstrably close them (verified
  above). The `me.diff`'s `FOLLOWUPS.md` hunk is exactly 7 lines, matching
  only the item-3/item-4 annotations. Someone scanning the follow-up file
  today would read items 1 and 2 as still open. Minor — recommend adding the
  same "Fixed, not yet shipped" note to items 1 and 2 before this ships.
- The implementer's own flagged concern — the same-seed key-path row's prose
  still says nothing about Nunchuk, while the unseated row says "only by
  chance" — is correctly out of scope for F-674 and not a new defect.

## Cleanliness

Both worktrees left clean: `dm-worktrees/f674` at `b662be1c` (`git status
--short` empty), `me-worktrees/f674` at `9fd7f32c` (`git status --short`
empty, "working tree clean"). All scratch build/output dirs under
`/scratch/code/shibboleth/.tmp/f674review*` removed. Nothing committed by
this review.

## Counts

**0 Critical, 0 Important, 2 Minor** (stale exact-version pin in
`run.sh`'s reproduction gate; missing FOLLOWUPS.md status notes for items 1
and 2). Every claim in both implementer reports that was checked came back
true; every live command run matched the claimed behavior; one independently
re-applied mutation reddened as expected.

ready to ship: yes
