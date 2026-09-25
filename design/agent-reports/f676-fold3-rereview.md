# Re-review: F-676 fold 3 (mnemonic-toolkit `f675-f677`)

- **Reviewer:** Sonnet 5 (independent context)
- **Date:** 2026-09-24
- **Tree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, HEAD `21669d61`
  (fold 3, on `4eceab6b`), `git diff 4eceab6b..21669d61`
- **Question:** is NEW-1 fixed, so the installer can never overwrite a binary
  another cargo package owns, and did fold 3 introduce a new defect? Not a
  fresh audit.
- **Inputs:** `design/agent-reports/f676-fold2-rereview.md` (NEW-1 Critical +
  test-14 Minor); fold report `design/agent-reports/f676-fold3.md`.
- **Settled, not re-derived:** I1, M2, M1, M3–M6 (confirmed fixed by the prior
  re-review); the design and "no unpublished binary reported as success"
  property (re-checked only where fold 3 touched the path).

All commands were run directly against the worktree, mutating and restoring
`scripts/install.sh` with `git checkout --` (byte-identical md5
`68205490ac047d0e218a44fac27360bb` before/after every mutation). `gawk`
5.4.1 is the system default; `mawk` 1.3.4-20240819, BusyBox 1.37.0, `dash`
0.5.13.5 and `shellcheck` 0.11.0 were fetched via `nix build nixpkgs#<pkg>`.
Real network + real `cargo`/`git` were used throughout (no mocked cargo for
the funds-safety claims); one component-table test suite run used the
worktree's own stubs.

## NEW-1 → fixed, confirmed adversarially and end-to-end

**Parser correctness.** Extracted `crates_owner()` verbatim and ran it under
gawk, mawk and BusyBox awk against 17 fixtures, including a real
`cargo install --path` root with a genuine multi-bin package (`multi-bin-fake`,
bins `mk`+`other`, cargo's real indented multi-line-array format). All three
awk implementations agree on every **decision** (`none` / `pkg <owner>` /
`unknown`) for:

| Case | Result (all 3 awks agree) |
|---|---|
| single-line owner (the `fake-mk` repro) | `pkg fake-mk 0.1.0 (path+file://…)` |
| real cargo multi-line bin array (`alpha`,`beta` / real `mk`+`other`) | correct owner |
| package id containing spaces / `(git+…)` | correct owner, unaffected |
| package id with an embedded literal `"` (malformed) | `unknown` |
| two packages both claiming the same bin | `unknown` (conflict detected) |
| bin name is a prefix/suffix of another (`mk` vs `mk-cli`, `md` vs `mdx`) | exact match only — `mk` queried against a `mk-cli`-only entry returns `none`, not a false match |
| `.crates.toml` present, `.crates2.json` absent (and reverse) | `pkg …` / `unknown` respectively, as specified |
| malformed/unclosed/no-header/empty `.crates.toml` | `unknown` |
| `.crates2.json` disagrees with a `.crates.toml` "none" | `unknown` (the disagreement check fires only in the dangerous direction — see Minor note below, this is correct-by-design) |

**End-to-end, real cargo + real git, both printed ways out:**
- `cargo uninstall --root … fake-mk`, then re-run `--from-source --only mk`:
  rc=0, `mk-cli 0.13.0` installed, `.crates.toml` now tracks `mk-cli`.
- User `--force`: rc=0, note "`--force given: replacing …, owned by cargo
  package 'fake-mk …'`", cargo prints "`Replaced package \`fake-mk\` …
  with \`mk-cli\``", md5 of the binary changes as expected.
- The reviewer's original repro (no `--force`): rc=1, `fake-mk`'s `mk`
  byte-identical before/after, `note:`/`error:` correctly names `fake-mk` as
  owner and prints a working `cargo uninstall --root "$ROOT" fake-mk` line.

**Mutation spot-checks (4 of the fold's claimed 13, independently
reconstructed from the diff, not from `mutate.py` which the fold deleted):**

| # | Mutation | Test(s) | Result |
|---|---|---|---|
| F1 | Restore fold-2's package-name grep (drops `crates_owner` entirely) | 16, 31, 32, 33, 35, 36 | **Killed** — 8 tests went red |
| F2 | "Another owner" case auto-forces instead of refusing | 31, 32, 33 | **Killed** |
| F8 | User `--force` no longer treated as an override in the "another owner" branch | 33 | **Killed** |
| F10 | `cargo_root` reverts to the fold-2 conditional (`--root` only when `ROOT_ARG` set) | 30 (second assertion) | **Killed** |

## Test-14 Minor → fixed

Mutated both temp-root guards together (fold's G3: `make_tmp_root`'s two
`case "$TMP_ROOT" in /?*) [ -d "$TMP_ROOT" ] && return 0` checks widened to
`[ -n "$TMP_ROOT" ] && return 0`, and `install_binary`'s own independent
`TMP_ROOT` re-check deleted). Re-ran `install-verify.test.sh`: **3 real
failures**, exactly reproducing the original defect class:
- both "lying mktemp" cases (relative path; absolute-but-never-created path)
  now go through with real `rm -rf`/`mkdir -p` calls logged against the lie
  (`rm -rf relative/evil-path/mk; mkdir -p relative/evil-path/mk/x`, relative
  to the test's cwd)
- the "temp root deleted mid-run" case (curl stub deletes `TMP_ROOT` before
  the second component) no longer refuses

Suite reports `[install-verify.test] FAILED`. Restored, byte-identical to
baseline. This is the same defect the fold-2 re-review's mutations #4/#5/#6
showed surviving the pre-fold-3 test suite; it is now pinned.

## Gates re-run, with numbers

| Gate | Result |
|---|---|
| `install-verify.test.sh` under bash/`sh` | 47 ok, 0 FAIL |
| … under `dash` 0.5.13.5 | 47 ok, 0 FAIL |
| … under BusyBox 1.37.0 `sh` | 47 ok, 0 FAIL |
| `install-msrv-guard.test.sh` under dash | 4/4 ok |
| `install-man-step.test.sh` under dash | 2/2 ok (39 pages, 0 shadow) |
| `shellcheck 0.11.0` on `install.sh` | clean, rc=0 |
| `cargo fmt --check` | rc=0 |
| `cargo nextest run --locked --workspace` | 4057 tests run: 4057 passed, 20 skipped (exact match to fold's claim) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`, pinned rustc **1.85.0** | rc=0, clean |

Clippy note: `/usr/bin/cargo` (Arch pacman's rustc 1.98.0-adjacent toolchain)
shadows `rustup`'s shim on PATH here, and `rustup run 1.85.0 cargo clippy`
still resolved `clippy 0.1.98` — the known papercut ("local clippy is not the
pinned clippy"). Prepending
`~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` to `PATH` fixed it
(confirmed `clippy 0.1.85`, then ran clean). A local-toolchain artifact, not a
fold defect.

## NEW finding: mawk-specific display corruption in the "another owner" message (Minor)

**Not blocking**, but real, reproducible, and outside what the fold's own
"gawk and BusyBox awk" proof table covered (mawk was never tested there,
despite the brief explicitly asking for it).

**Root cause, isolated:** in `crates_owner()`, `sub(/" = \[.*$/, "", id)`
fails to match under mawk 1.3.4 specifically when the pattern `" = [` is the
very last content on the line — i.e., exactly the shape of a real
multi-line-bin-array header line as cargo writes it (`"<id>" = [` with
nothing after `[`). Confirmed via `match()`: mawk returns `0` (no match) for
that exact input; gawk and BusyBox awk return a real match at the correct
position. Appending even one character after `[` (a space, or the case
already reached end-of-array) makes mawk match fine — the bug is specific to
end-of-string immediately after `\[`.

**Effect, confirmed with a real cargo-installed multi-bin package**
(`multi-bin-fake`, bins `mk`+`other`, real `.crates.toml`):
```
error: …/bin/mk belongs to cargo package 'multi-bin-fake 0.1.0 (path+file://…)" = [', not mk-cli;
```
— the owner id has a stray `" = [` fragment appended, only under mawk, only
for a bin claimed inside a real multi-line array.

**Why this doesn't block:** the funds-safety property NEW-1 exists to
guarantee — never silently overwrite a binary another package owns — holds
under this corruption:
- The refusal still fires (rc=1), the file is untouched (verified: `mk` is
  byte-identical before/after, cargo is never invoked).
- The `cargo uninstall --root "$ROOT" ${owner_id%% *}` recipe is still
  correct, because `%% *` truncates at the *first* space, which occurs well
  before the corrupted trailing fragment (`multi-bin-fake` truncates
  correctly out of `multi-bin-fake 0.1.0 (…)" = [`).
- The "own package, no `--force` needed" case pattern (`"pkg $pkg "*`) also
  still matches correctly, since it only tests a prefix.

Only the *displayed* owner text is garbled — confusing, but not
data-lossy, and not a false success. **Gate-coverage note:** the shipped
test 32 (which specifically exercises a real multi-line array) would not
have caught this even run under mawk, because its assertion is
`grep -q "belongs to cargo package 'tools 1.0.0"` — a prefix match, blind to
trailing corruption. Worth a follow-up: fix the regex (e.g. anchor
differently, or avoid the "match a suffix with `.*$`" idiom mawk mishandles),
and tighten test 32/31 to assert the full owner string, not just a prefix.
mawk is a realistic exposure: it is Debian's traditional default `awk`
alternative, and this installer explicitly targets Debian-class hosts.

## `--root` / `install.root` / `CARGO_INSTALL_ROOT` behavior (checked, not blocking)

Confirmed live: with `HOME` pointed at a fixture carrying
`~/.cargo/config.toml`'s `[install] root = …` and no `--root` /
`CARGO_INSTALL_ROOT`, **bare `cargo install`** (no `--root` flag) honors
`install.root` and installs there — this is what the pre-branch installer
(`86c93f84`, before F-676) effectively relied on, since it never passed
`--root` at all. Fold 3's `cargo_root()` now **always** passes
`--root "$ROOT"` explicitly (`$ROOT` = `CARGO_INSTALL_ROOT` else `CARGO_HOME`
else `~/.cargo`, never `install.root`), and a CLI `--root` flag beats
`install.root` in cargo's own precedence — confirmed live: the same fixture,
run through `install.sh --from-source --only mk` with no `--root`, installs
into `$HOME/.cargo/bin`, **not** the `install.root`-configured directory.

This is a genuine behavior change from fold 2 (and from pre-branch), but:
- it is **necessary**: `crates_owner()` only works if cargo's records live in
  the same root the script inspects and reports as `BIN_DIR`; letting cargo
  honor `install.root` independently would silently point `.crates.toml`/
  `.crates2.json` at the wrong directory and make the "install root: X"
  banner false;
- it is **disclosed** in `CHANGELOG.md` ("cargo is always given `--root`
  explicitly, so it installs where the script looks") and **explicitly
  tested** (`install-verify.test.sh` test 30's second half, whose own comment
  says "not whatever cargo's own config would pick" — the fold clearly knew);
- the script's own runtime banner ("install root: …") is accurate at every
  point I checked — there is no silent divergence between what it prints and
  where the binary lands, only a divergence from the user's separate global
  cargo config;
- `--help`/the manual still do **not** mention `install.root` in either
  direction (this predates fold 3 too — pre-branch's `--help` never mentioned
  it either). A one-line addition to `--help`'s `--root` description would
  close this, but it's a documentation gap, not a functional regression, and
  the fold's own report already self-flags it as a "Concern."

Classified **Minor / follow-up**, not blocking.

## Housekeeping

All scratch work (fixture crates, real-cargo roots, the multi-bin fixture,
`nix build` result symlinks) lived under
`/scratch/code/shibboleth/review-fold3-scratch/`, removed with
`find /scratch/code/shibboleth/review-fold3-scratch -delete` (the `rm -rf`
hook blocks the direct form). Worktree `git status --short` is empty;
`scripts/install.sh` md5 matches the pre-review baseline
(`68205490ac047d0e218a44fac27360bb`). Nothing committed, pushed, or merged.

## Answer

- **NEW-1: fixed.** Confirmed adversarially (17 synthetic + 1 real-cargo
  multi-bin fixture, 3 awk implementations) and end-to-end with real
  cargo/git for both printed ways out and the original repro. The one
  decision-affecting property — never auto-force over a binary a *different*
  package owns — held in every case tried, including under a tampered/
  malformed `.crates.toml` (cargo's own state is the enforcement backstop for
  every path except `none → --force`, which is the one path cross-checked
  against `.crates2.json`).
- **Test-14 Minor: fixed.** Removing both guards together reproduces the
  original defect and the suite now goes red (3 failures) instead of staying
  green.
- **Mutation spot-checks (4 of 13: F1, F2, F8, F10): all killed** as claimed.
- **Gates:** 47/47 under bash, dash, BusyBox sh; shellcheck clean; fmt clean;
  clippy clean under the pinned 1.85.0 toolchain; nextest 4057
  passed/20 skipped, exact match.
- **Two NEW Minors, neither blocking:** (1) a mawk-specific regex bug
  corrupts the *displayed* owner id in the "another package owns it" message
  when the bin is claimed inside a real multi-line `.crates.toml` array — the
  refusal, the file's integrity, and the printed recovery recipe all remain
  correct; only the cosmetic text is garbled, and a follow-up should also
  tighten test 31/32 to assert full-string equality rather than a prefix.
  (2) `install.root` cargo config is now always overridden by this script,
  which is necessary for `crates_owner`'s correctness and is disclosed in
  CHANGELOG + tested, but still undocumented in `--help`/the manual.

**ready to ship: yes**
