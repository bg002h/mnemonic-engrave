# Re-review: F-676 fold 2 (mnemonic-toolkit `f675-f677`)

- **Reviewer:** Sonnet 5 (independent context)
- **Date:** 2026-09-24
- **Tree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, HEAD `4eceab6b`
  (fold 2, on `d3f043e7`), `git diff d3f043e7..4eceab6b`
- **Question:** did fold 2 fix each finding of the branch review
  (`design/agent-reports/f675-677-toolkit-review.md`, 0C/1I/6M/7N, M2 raised
  to blocking), and did it introduce a new defect? Not a fresh audit of the
  branch.
- **Fold report re-run:** `design/agent-reports/f676-fold2.md`.

All commands below were run directly against the worktree (mutating and
restoring `scripts/install.sh` with `git checkout --`), never via the fold's
own (deleted) `mutate.py`. Tools that were missing locally (`dash`, `busybox`,
`yash`, `shellcheck`) were fetched via `nix build nixpkgs#<pkg>` and symlinked
into a scratch `bin/` dir — versions match the fold's claims (dash 0.5.13.5,
BusyBox 1.37.0, yash 2.61, shellcheck 0.11.0).

## Finding-by-finding

| ID | Verdict | Evidence |
|---|---|---|
| **I1** | **Fixed** | See below — real end-to-end runs. |
| **M2** | **Fixed** (live code); **test-suite mutation gap found** | See below. |
| **M1** (BusyBox wget) | Fixed | Real install with only BusyBox `wget` on PATH (no curl) fetched and installed `ms` and `mk` end to end, rc=0. `wget --help \| grep -q -- '--https-only'` correctly returns false for BusyBox and true for GNU wget. |
| **M3** (mutations) | Fixed | See mutation table below; 6 independently constructed mutations, 4 killed as claimed. |
| **M4** (musl GUI) | Fixed | `asset_for` no longer maps `linux-*-musl` for `mnemonic-gui` (confirmed via `install-assets.test.sh`: `mnemonic-gui@linux-x86_64-musl` and `@linux-aarch64-musl` both report "no binary (expected; builds from source)"). |
| **M5** (PATH warning) | Fixed | Real install into a root not on `$PATH` printed the warning with correct bash/zsh/fish snippets; a second install with `$BIN_DIR` prepended to `$PATH` printed nothing extra (install-verify test 28, 36/36 pass, also reproduced live). |
| **M6** (no-cargo message) | Fixed | Confirmed via install-verify test 29 (36/36 pass); message lists the reason per component and ends `--exclude md,mnemonic-gui`. |
| **N1** (directory at bin path) | Fixed | Reproduced live: `--root` with `bin/mk` pre-existing as a directory — refused, "not replacing it", directory and its contents left untouched (both for a binary install and, separately, for `--from-source`, where cargo itself also refuses: see the new finding below). |
| **N2** (partial-copy on interrupt) | Not tested (fold's own admission stands) | Not independently re-tested; low-risk, matches the fold's own "untested: not deterministic" note. |
| **N3** (`--root ""`) | Fixed | `--root ""` and `--root=` both exit 2 with "requires a non-empty argument" (install-verify test 27, confirmed in the 36/36 pass). |
| **N4** (Windows GUI skip) | Unchanged, as fold states | Not independently verifiable (no Windows GUI binary to run here); install-verify test 23 confirms the run-check still applies to Windows CLI binaries. |
| **N5** (duplicate digest) | Fixed | Reproduced live via install-verify test 25 (36/36 pass): an asset listed twice with different digests is refused with "lists it with more than one digest". |
| **N6** (`.crates.toml` residue) | Not fixed, as stated | Confirmed not addressed; matches fold's own "known gap" framing. Its remedy (see NEW-1 below) needed correcting regardless. |
| **N7** (`man-pages.yml`) | Not fixed, out of scope | Confirmed pre-existing and outside this diff. |

### I1 — fixed, confirmed end-to-end

1. **Glibc floors take effect before download.** `MNEMONIC_INSTALL_GLIBC=2.36`
   on `linux-x86_64-gnu` (`--list`): only `mnemonic-gui` falls to source (md's
   floor 2.34 is met). At `2.30` (below both floors): both `md` and
   `mnemonic-gui` fall to source. A `--dry-run` at 2.30 shows no
   `[dry-run] download` line for either — the floor decision happens at
   `plan_for` time, before any network call.
2. **Real simulated Debian-class host, full default install (all 5), real
   network + real cargo build:** `MNEMONIC_INSTALL_GLIBC=2.35`,
   `linux-x86_64-gnu`, no `--only`/`--exclude`. Result: **rc=0, "5
   installed."** The four CLIs installed as binaries; `mnemonic-gui` printed
   the floor note and built from source (a real `cargo install --git
   .../mnemonic-gui --tag mnemonic-gui-v0.59.0`, finished successfully).
   This is the exact scenario the original review measured as "4 installed,
   1 failed," rc=1 — now rc=0.
3. **The reviewer's own repro, reproduced verbatim, into a root with a
   space:**
   - `--only mk --root "…/root with space"` (binary): rc=0, `mk 0.13.0`
     installed.
   - `--from-source --only mk` (the printed recipe) on the same root: rc=0.
     Printed `note: replacing …/bin/mk, which cargo does not track
     (--force).`; cargo output shows `Replacing …/bin/mk` /
     `Replaced package \`unknown\` with \`mk-cli v0.13.0 (...)\``.
     `.crates.toml` now tracks `mk-cli`. No stray `…/src` directory (the
     space-splitting bug is gone — `cargo_root` passes `--root` as one
     argument).
   - A second `--from-source --only mk`: rc=0, `Ignored package \`mk-cli
     v0.13.0 (...)\` is already installed, use --force to override` — no
     `--force` passed this time, matching the fold's claim exactly.

### M2 — fixed for every constructed real-world failure; one test-suite mutation gap found

Constructed 8 temp-dir failure scenarios as a non-root user, with `rm` and
`mkdir` wrapped to log every call (`M2_LOG`), run against the **unmodified**
worktree `install.sh`:

| Scenario | Result |
|---|---|
| A. Both `mktemp` forms fail (stub always exits 1) | Refused, rc=1, **0 rm/mkdir calls logged** |
| B. `TMPDIR=/nonexistent-dir`, real GNU mktemp | Refused, rc=1, 0 calls |
| C. `TMPDIR` is a regular file | Refused, rc=1, 0 calls |
| D. `TMPDIR` is a directory with no write permission (chmod 500) | Refused, rc=1, 0 calls |
| E. `mktemp` prints a **relative path**, exits 0 | Refused, rc=1, 0 calls |
| F. `mktemp` prints **nothing**, exits 0 | Refused, rc=1, 0 calls |
| G. First `mktemp -d` call fails; templated fallback call prints a relative path | Refused, rc=1, 0 calls |
| H. `mktemp` prints an absolute path but never creates it (lies) | Refused, rc=1, 0 calls |

All 8 fail closed with an empty `rm`/`mkdir` log; no `/mk`, `/md`, `/ms` etc.
appear at the filesystem root afterward. This matches and exceeds the fold's
own 2 tested cases.

**However**, a genuine gap in the *shipped test suite's* mutation coverage:
`install-verify.test.sh` test 14 only exercises "`mktemp` always fails
outright" and "`TMPDIR` does not exist" — both produce **empty** `TMP_ROOT`.
I mutated `make_tmp_root()` and `install_binary()`'s guards (both
independently and together — see mutation table) and in every case
**test 14 stayed green**, because its fixture never makes `mktemp` "succeed"
with a relative or nonexistent path. When I removed **both** guards
(`make_tmp_root`'s `case "$TMP_ROOT" in /?*)` checks and `install_binary`'s
own re-check) and re-ran my own scenario E (relative-path `mktemp`) against
the **mutated** code, it reproduced the *original* defect exactly: `rm -rf
relative/evil-path/mk; mkdir -p relative/evil-path/mk/x` relative to cwd —
and `install-verify.test.sh` still reported OK (rc=0). The live, unmutated
code is safe (proven above); the committed test suite does not mechanically
pin the specific "lying/relative mktemp" attack class the original review
and this brief call out, so a future refactor could silently reintroduce it.
**Classified Minor** (gate-coverage gap, not a live defect — same class as
the original review's X14/X15 "gate gap, not a live defect").

Also: `install_binary`'s own `TMP_ROOT` re-check is currently unreachable
dead code from the test suite's perspective — `make_tmp_root` runs once,
pre-loop, and `exit 1`s on failure before `install_binary` is ever called, so
no committed test exercises `install_binary`'s independent guard. It is
real, correct, defense-in-depth code (matters if `TMP_ROOT` is deleted
externally between components in a multi-component run), just untested.
**Nit.**

## NEW finding

### NEW-1 (Critical): the `--from-source` auto-`--force` can silently destroy a user's own binary that cargo tracks under a *different* package

The brief specifically asked: "the `--force` decision for `--from-source`
(can it overwrite something it shouldn't, such as a binary cargo *is*
tracking...)?" — **yes.**

`install.sh`'s new logic (lines ~759–764):
```sh
src_force="$FORCE"
if [ -z "$src_force" ] && [ -e "$BIN_DIR/$bin$EXE" ] \
   && ! grep -q "^\"$pkg " "$ROOT/.crates.toml" 2>/dev/null; then
    src_force="--force"
    echo "note: replacing $BIN_DIR/$bin$EXE, which cargo does not track (--force)." >&2
fi
```
This only checks whether the file is tracked under **this exact package
name** (`$pkg`, e.g. `mk-cli`). If a user has, at any earlier time, run
`cargo install` (from crates.io, git, or a local path) for a **completely
unrelated crate** that happens to produce a binary of the same short name
(`mk`, `md`, `ms`, `mnemonic`, or `mnemonic-gui`) into the same `--root`,
cargo genuinely tracks that binary — just under a different package — and
the grep does not match, so the script concludes "cargo does not track it"
and force-installs over it.

**Reproduced live:**
1. Built a local crate `fake-mk` with `[[bin]] name = "mk"`, installed it
   with plain `cargo install --path … --root R`. `R/.crates.toml`:
   `"fake-mk 0.1.0 (path+file://…)" = ["mk"]`.
2. Ran `install.sh --from-source --only mk --root R`. Output: `note:
   replacing R/bin/mk, which cargo does not track (--force).` — **false**;
   cargo did track it. Cargo then printed `Replaced package \`fake-mk
   v0.1.0 (...)\` with \`mk-cli v0.13.0 (...)\``. The user's `fake-mk`
   binary is gone, unrecoverable, and `.crates.toml` now shows only
   `mk-cli`. rc=0 — reported as a normal successful install.
3. **Control:** re-created the same fixture and ran cargo's own default
   behavior (`cargo install --git … --tag … mk-cli --root R`, no `--force`,
   which is what the *pre-fold* installer effectively did since `$FORCE`
   defaults empty). Cargo refuses on its own, and **names the real owner**:
   `error: binary \`mk\` already exists in destination as part of \`fake-mk
   v0.1.0 (...)\`. Add --force to overwrite.` Nothing is touched.

This is a **new** regression introduced by fold 2, not present in
`d3f043e7`: the pre-fold code never auto-added `--force`, so this exact
collision would have failed closed (as cargo's own control run shows) rather
than silently overwriting a tracked, unrelated binary. The message printed
to the user (“which cargo does not track”) is factually wrong in this case.

Checked the neighboring case the brief also asked about — **a directory** at
the destination: this fails safe. `cargo` itself refuses to move a file onto
a directory (`error: failed to move … Is a directory (os error 21)`), so the
run ends "0 installed, 1 failed," rc=1, and the directory and its contents
are untouched. No defect there.

**Severity:** classified Critical under this project's own rubric ("wrong
result / data loss" blocks). It is real, permanent data loss of a file the
installer has no way to reconstruct, triggered by a plausible-if-narrow
precondition (a name collision on `mnemonic`/`md`/`ms`/`mk`/`mnemonic-gui`
between an unrelated cargo package and the pinned constellation component,
in the same `--root`), reached only via the fallback/`--from-source` path.
Not a secret-handling defect, so the 2026-08-27 carve-out does not apply.

**Suggested fix (not authoritative):** don't infer "untracked" from a
package-name grep; check whether the binary is tracked by *any* package
other than `$pkg` (or just capture and inspect cargo's own refusal message
for "as part of \`$pkg\`" vs. a different name) before deciding to add
`--force`. This is the same class of gap N6 already named for the binary
path (a stale `.crates.toml` entry) but here it runs in the opposite,
destructive direction.

## Mutation spot-checks (6 run; brief requires ≥6, including platform-table and M2)

Each: mutate `scripts/install.sh` in place with `python3`/`sed` (asserted to
match exactly once), run the affected test(s), confirm red, `git checkout
--` to restore, confirm `md5sum` matches the pre-mutation baseline.

| # | Mutation | Test | Result |
|---|---|---|---|
| 1 | Platform table: `macos-x86_64` mapped to the arm64 asset (X14-class) | `install-verify.test.sh` test 22 | **Killed** — `FAIL mapping macos-x86_64: got '...arm64...' want '...amd64...'` |
| 2 | `glibc_floor`: md's `2.34` entry deleted (I1-class) | `install-verify.test.sh` test 15 | **Killed** — `FAIL glibc floors: got 'binary binary source binary binary'` |
| 3 | `cargo_root`: `--root` unquoted again (reintroduces the space-splitting bug) | `install-verify.test.sh` test 30 | **Killed** — argv shows `--root /tmp/.../sp` and `ace` as separate tokens |
| 4 | `make_tmp_root`'s absolute-path/`-d` guards widened to `[ -n ... ]` only (M2a-class) | `install-verify.test.sh` test 14 | **Survived** the shipped test (both its cases produce empty `TMP_ROOT` either way); **caught live** by `install_binary`'s own independent re-check when fed a relative path (defense-in-depth held) |
| 5 | `install_binary`'s own `TMP_ROOT` re-check removed entirely | `install-verify.test.sh` test 14 | **Survived** — this guard is never exercised by any committed test (see M2 discussion above); currently dead code, not a live defect |
| 6 | Both #4 and #5 removed together (M2b-class, matching the fold's own description) | `install-verify.test.sh` test 14 | **Survived the shipped suite** (rc=0, both "ok"); **live-fired** against my own relative-path `mktemp` scenario and reproduced the original defect exactly (`rm -rf relative/evil-path/mk; mkdir -p relative/evil-path/mk/x`, relative to cwd) |

Worktree confirmed clean (`git status --short`) and `install.sh` byte-
identical (`md5sum 5ea7fdf9acfc7012322677ef04d175d8`) after every restore.

## Gates re-run, with numbers

| Gate | My result | Fold's claim |
|---|---|---|
| `install-assets.test.sh` (network) | 41 ok, 0 FAIL; "17 Linux assets checked for their glibc floor"; "32 of 32 mapped assets verified" | 32/32 + 17/17 |
| `install-verify.test.sh` under bash | 36 ok, 0 FAIL | 36/36 |
| … under `dash` 0.5.13.5 | 36 ok, 0 FAIL, rc=0 | 36/36 |
| … under BusyBox 1.37.0 `sh` | 36 ok, 0 FAIL, rc=0 | 36/36 |
| … under yash 2.61 | 36 ok, 0 FAIL, rc=0 | 36/36 |
| `install-msrv-guard.test.sh` under dash | 4 ok, 0 FAIL | "6/6" (fold's count includes sub-cases differently; all pass) |
| `install-man-step.test.sh` under dash | 2 ok (39 pages, 0 shadow) | 4/4 equivalent, all pass |
| `ci/repro/residue.test.sh` | 21 passed, 0 failed | 21/21 |
| `ci/repro/vendor-freshness.test.sh` | 8 passed, 0 failed | 8/8 |
| `shellcheck 0.11.0` on `install.sh` | clean, rc=0 | clean |
| Real end-to-end install (`--only mk`, real network) into a spaced `--root` | rc=0, `mk 0.13.0` installed | matches |
| Real full default install (all 5), simulated glibc 2.35, real network + real GUI `cargo install` | **rc=0, "5 installed."** | matches ("rc=0" claim independently reproduced from scratch) |
| BusyBox-`wget`-only, no-curl install (`--only mk,ms`) | rc=0, both installed | matches |
| `cargo nextest run --locked --workspace` | **4057 tests run: 4057 passed, 20 skipped** | 4057 passed, 20 skipped — exact match |
| `cargo fmt --check` | rc=0 | rc=0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`, **pinned rustc 1.85.0** (`rustup which --toolchain 1.85.0 cargo-clippy`) | rc=0 | rc=0 (1.85.0) |

Note: my *first* clippy attempt used this machine's default (unpinned)
toolchain (rustc/clippy 1.98.0) and found an unrelated `manual_div_ceil` lint
in `crates/wc-codec/src/sync.rs` — a file untouched by this diff, and a lint
that doesn't exist in the pinned 1.85.0 clippy. This is a local-toolchain
artifact (a papercut already on record: "local clippy is not the pinned
clippy"), not a finding; the pinned-toolchain run above is clean.

## What this re-review did not (re-)do

- Did not re-verify N2 (partial-copy-on-interrupt) or N4 (Windows GUI
  `--version` output) — both require conditions not reproducible in this
  environment (a deterministic mid-`cp` interrupt; a Windows host), and the
  fold's own report already flags both as unmeasured. Not gating.
- Did not re-derive the "no way to install an unpublished binary while
  reporting success" property from scratch (per the brief, settled) — spot-
  checked only the paths the fold touched (glibc floor gating, the digest
  duplicate refusal, the directory refusal), all confirmed above.
- Did not re-run the manual/doc-CI gates (`make audit`, `sibling-pin-check`,
  Examples golden, etc.) — unrelated to the two blocking findings (I1, M2)
  or the new finding (all three are in `install.sh`'s logic, not docs/CI
  plumbing), and the fold's report already shows these green with specific
  commands.

## Housekeeping

All scratch work was done under `/scratch/code/shibboleth/review-fold2-scratch/`
(plus `nix build` store paths, which are shared Nix store content, not
scratch). Deleted with `find /scratch/code/shibboleth/review-fold2-scratch -delete`
after this report was written. The worktree `git status` is clean; nothing
was committed, pushed, or merged. No real installs landed outside explicit
`--root` scratch directories — the user's actual `~/.cargo/bin` was not
touched by any command in this review.

## Answer

- **I1: fixed**, confirmed end-to-end including the exact original repro.
- **M2: fixed** for every real-world failure mode constructed (8 scenarios,
  0 rm/mkdir calls in any of them); one Minor test-suite mutation-coverage
  gap noted (the shipped test doesn't pin the "lying mktemp" attack class,
  though the live code handles it).
- **M1, M3–M6, N1, N3, N5: fixed** and independently reproduced.
- **N2, N4, N6, N7: unchanged**, as the fold itself states.
- **One new Critical defect** introduced by fold 2's own new code
  (NEW-1): `--from-source`'s auto-`--force` can silently overwrite a
  user's own cargo-tracked binary when it collides in name with a
  constellation component but is a different package — reproduced live,
  contrasted against cargo's own safe default behavior, printing a false
  "which cargo does not track" message while destroying the file.

**ready to ship: no**
