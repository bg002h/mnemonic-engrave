# F-679 toolkit re-review — fold 1 (`473bca17`)

- **Reviewer:** Sonnet 5 (independent of the fold's author)
- **Date:** 2026-09-25
- **Target:** worktree `/scratch/code/shibboleth/tk-worktrees/f679-681`, `git diff 8ed3494f..473bca17` (32 files, 350+/117-)
- **Question:** did fold 1 fix I-1 and M-1..M-3 of `design/agent-reports/f679-toolkit-review.md`, and did it introduce a defect, above all in its new gate? Not a fresh audit.
- **Method:** installed the pinned binaries from this branch (`sh scripts/install.sh --root <scratch> --no-man`: 5 installed, mnemonic 0.104.0 / md 0.20.3 / ms 0.19.1 / mk 0.13.0 / mnemonic-gui 0.62.0, matching the fold's claim). All reruns below were against those binaries. Scratch (`/scratch/code/shibboleth/review-f679tk2-scratch/`, 353 MB) was deleted with `find -delete`; two throwaway detached worktrees (pre-fold toolkit tree, mnemonic-gui at `mnemonic-gui-v0.62.0`) were removed with `git worktree remove --force`. The `f679-681` and `mnemonic-gui` working trees are unmodified (`git status --short` clean in both). Nothing pushed, committed, or edited on the branch.

## Verdict

**0 Critical / 1 new Important / 0 new Minor.** I-1 and M-1..M-3 are fixed as claimed; the new gate is sound and correctly wired. The fold's own chapter-12 "toolkit minimum" claim is factually wrong, in the same direction of error the original I-1 fixed (an unverified version-gate assertion) but not the same severity (the wrong direction here is "you need more than you actually do," not "the remedy breaks the GUI").

## 1. The new gate (`check_cli_pins.py`, lint phase 13, `sibling-pin-check.yml`)

**Can it fail — reproduced all 4 of the fold's claimed cases, byte-for-byte:**

| Case | Fold claimed | Reproduced |
|---|---|---|
| Current tree, unchanged | rc=0, "67 version mention(s) in 99 files, 30 history line(s)" | identical, rc=0 |
| Pre-fold tree (`8ed3494f`) + only the new script/history dropped in | rc=1, 36 findings (35 off-pin + 1 stale history) | identical: rc=1, 36 findings, same 35 sites + the same stale-history line |
| `install.sh` ms pin bumped to 0.19.2, manual untouched | rc=1, 8 findings | identical: rc=1, 8 findings, same sites |
| `pinned-upstream.toml` GUI tag set to v0.61.0 | rc=1, 1 finding | identical: rc=1, 1 finding |

**Can it pass vacuously — no.** Three failure-closed probes, all non-zero exit:
- `install.sh` with no `component_info` lines at all → **rc=2**, explicit "cannot read pins for [...]" (fails closed rather than treating missing pins as OK).
- `pinned-upstream.toml` corrupted (invalid TOML) → uncaught `tomllib.TOMLDecodeError`, **rc=1** (ugly, but non-zero — CI still reds).
- Every `.md` file's content replaced with a version-free sentence (simulates the `MENTION` regex matching nothing) → **rc=1, 30 findings**, one per history-list entry that "matches no line naming an off-pin version." This is a real, verified self-check: if the mention-regex ever breaks silently, the 30 history entries go from "used" to "orphaned" and the gate fails loudly instead of reporting 0 findings. This is the strongest anti-vacuity property in the script and it works as designed.

**Does it run in CI on a partial-touch push — yes, unconditionally.** `sibling-pin-check.yml`'s `on:` block has `push:` / `pull_request: {branches: [master]}` / `workflow_dispatch:` with **no `paths:` filter**, so `check_cli_pins.py` runs on every push regardless of what changed — an install.sh-only push and a manual-only push both trigger it. (`manual-gui.yml` additionally runs the same check as lint phase 13, but only when its own path-derived `changes` job says something gated moved; `sibling-pin-check.yml` is the one the fold's own claim rests on, and it has no such gate.) Verified by reading both workflow files directly, not inferred from a comment.

**Full `make lint` run (13/13), independently executed** against a fresh detached worktree of `mnemonic-gui` at tag `mnemonic-gui-v0.62.0` (commit `9f569e1`, matching the review's cited commit) and the freshly-installed pinned CLIs: all 13 phases OK, including `13/13 cli-pin-consistency` printing the identical "67 version mention(s) ... 30 history line(s)" line.

**The 30-line history allowlist — read all 30, all genuinely historical.** Every entry names a version that differs from the current pin (toolkit 0.104.0 / md 0.20.3 / ms 0.19.1 / mk 0.13.0) and is phrased as "since/ships with/as of <old version>" — an introduction-date statement, not a current-pin claim. Checked the three most syntax-adjacent-looking ones (`4n-word-card.md` "toolkit v0.74.0", `91-glossary.md` "toolkit v0.74.0+", `69-derive.md` "ms-cli v0.5+") in full surrounding context: all three are "this feature shipped with / requires at least" phrasing, not "you are pinned to." No current claim is hiding in the allowlist.

## 2. I-1 — fixed, with one new factual defect in the added content

All 15 originally-false "pinned to X" sentences and banners are corrected (confirmed by the gate's own rc=0 and by reading the diff). The chapter-12 table, the troubleshooting list, and the three overview banners now state the real pins.

**New Important, found by spot-checking the brief's specific ask** ("spot-check that toolkit 0.103.x rejects `--allow-argv-secret`, if an old release binary is easy to get"): **it does not.** An old release binary was easy to get (`gh release download` from `bg002h/mnemonic-toolkit`). Toolkit 0.103.2, 0.101.0 and 0.100.0 all already list `--allow-argv-secret` in `--help`, and 0.103.2 **accepts it in a real run** (`mnemonic bundle --descriptor ... --passphrase test --allow-argv-secret` → rc=0, normal output). Bisecting down: 0.97.0/0.95.0/0.90.0/0.85.0/0.80.0 do **not** have the flag; 0.99.0 does. The toolkit's own `CHANGELOG.md` at tag `mnemonic-toolkit-v0.99.0` confirms it under "**NEW: `--allow-argv-secret`**, a global flag" (2026-09-17) — five minor releases before the 0.104.0 the manual now cites.

The manual's new chapter-12 row (`10-foundations/12-relation-to-cli.md:40`) says: *"`0.104.0` — the GUI adds `--allow-argv-secret` to every secret-bearing run, and older `mnemonic` rejects that flag."* That is false for toolkit 0.99.0 through 0.103.2 — those "older" binaries already accept the flag fine. The fold attributed this to "the review established that the flag first ships in v0.104.0," but the review's own hedge was scoped to `toolkit v0.13.0` ("This is inferred: I did not install 0.13.0") — a claim about the ancient tag the old manual pointed to, which the fold generalized into an unqualified "any older mnemonic" claim without re-verifying it.

**Why Important, not Critical:** the error runs the safe direction — it tells a user on 0.99.0–0.103.x that they need to upgrade when they don't, rather than sending them somewhere that breaks the GUI (the shape of the original I-1). It doesn't cause data loss or a broken run; it's an incorrect "minimum version" claim in the exact artifact (`#version-pinning`) this fold exists to make trustworthy, verified reproducibly against real released binaries rather than inferred.

I-1's other four sites (chapter-82 table, `11-what-is-mnemonic-gui.md`, `31-first-launch.md`, `33-help-icons...md`) and the ms/md/mk rows were not re-litigated beyond the gate's pass — they were the review's own item and are unchanged by this new finding, which is specific to the toolkit row's added "minimum" clause.

## 3. M-1..M-3 — 5 of 11 corrected claims re-run against the pinned binaries, all match exactly

| Claim | Command | Result | Matches fold? |
|---|---|---|---|
| M-1: `restore --md1` depth-2 taproot restores | bundled `taproot-4leaf.desc` (from `mnemonic-gui-v0.62.0`) into 25 md1 chunks, `mnemonic restore --md1 <c1> --md1 <c2> ... ` (repeatable flag, not space-joined — confirmed via `--help`: "Repeat for chunked cards") | **rc=0**, watch-only `tr(...)` descriptor reconstructed, 12-cosigner UNVERIFIED advisory | Yes |
| M-2 canonical refusal | `bundle --descriptor 'wpkh(@0/<0;1>/*)' --slot @0.entropy=<32B> --account 1 --allow-argv-secret` | `error: --account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.` rc=2 | Yes, byte-exact |
| M-2 non-canonical acceptance | same but `--descriptor 'wsh(pk(@0/<0;1>/*))'` | rc=0, origin `m/48'/0'/1'/2'` | Yes |
| M-3 `ms combine` message | `ms split -k2 -n3 --phrase ...` then `ms combine` with 1 of 2 shares | `error: not enough shares: have 1, need 2` rc=1 | Yes, byte-exact |
| M-3 `ms split` K-range message | `ms split -k 1 -n 3 --phrase ...` | `error: invalid threshold 1; K-of-N shares require k in 2..=9` rc=1 | Yes, byte-exact |

(Note: `mnemonic --md1` and `mnemonic bundle --descriptor`/`--slot` both require the flag repeated per chunk/slot, not a space-joined list under one flag — a shell-quoting trap I hit and corrected before these counted as real reproductions.)

## 4. Gates — all re-run independently, all green

- **manual-gui `make lint`** (13/13), against a fresh detached `mnemonic-gui-v0.62.0` worktree and the freshly-installed pinned CLIs: identical output to the fold's report, including phase 13.
- **CLI manual `make audit`** (`docs/manual`), against the same pinned binaries: rc=0 — markdownlint/cspell/lychee clean, flag-coverage 10 exemptions (all `mk-cli 0.13.0`-unreleased, as documented), `verify-examples` 62/62, `anchor-check` 9 danglers matching the stated baseline (no new, no shrunk).
- **Installer tests under dash 0.5.13.5** (fetched via `nix run nixpkgs#dash`, same version the fold cites): `install-verify.test.sh` 47/47 ok, `install-msrv-guard.test.sh` 6/6 ok, `install-man-step.test.sh` 4/4 ok (with `shellcheck` 0.11.0 on `$PATH` — one check self-skips without it), `install-assets.test.sh` 59 ok. `shellcheck 0.11.0` on `install.sh` independently: clean.

All counts match the fold report exactly; no drift found.

## Per-finding disposition

- **I-1 (review):** fixed.
- **M-1, M-2, M-3 (review):** fixed (5/11 corrected claims re-run, all exact).
- **New Important (this round):** the chapter-12 toolkit-minimum clause ("0.104.0 ... older `mnemonic` rejects that flag") is false for toolkit 0.99.0–0.103.x, which already accept `--allow-argv-secret`. Fix: either drop the specific "older mnemonic rejects that flag" claim (state only that the pin is 0.104.0, full stop, as md/mk's rows already do — "the pinned tag ... I did not bisect the exact release"), or correct the minimum to 0.99.0 with a citation to that tag's `CHANGELOG.md`.
- **No other new findings.** The gate's design, wiring, and vacuity resistance all hold up under adversarial probing; nothing else in the fold's diff was found wrong.

ready to ship: no
