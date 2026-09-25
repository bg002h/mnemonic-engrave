# F-676 fold 3: NEW-1 (Critical) and the M2 test-coverage Minor

- **Implementer:** Opus 5.5
- **Date:** 2026-09-24
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`
- **Commit:** `21669d61` "F-676 fold 3: never auto-force over another package's binary (NEW-1); pin the lying-mktemp class". It sits on `4eceab6b`.
- **Files:** `scripts/install.sh`, `scripts/install-verify.test.sh`, `CHANGELOG.md`.
- **Not pushed, tagged or merged.**
- **Re-review folded:** `design/agent-reports/f676-fold2-rereview.md`.

## NEW-1 → fixed

**Cause, confirmed.** The auto-`--force` decided "untracked" with
`grep -q "^\"$pkg " "$ROOT/.crates.toml"`. A binary that cargo tracks under
a different package therefore read as untracked, and was overwritten with a
false message.

**Fix.** A new `crates_owner <bin>` reads cargo's own records in `$ROOT`,
never the package name, and prints exactly one of three answers. The source
path acts on it:

| Answer | Meaning | Action |
|---|---|---|
| `none` | No `.crates.toml` and no `.crates2.json`; or `.crates.toml` parses, no package lists the bin, and `.crates2.json` does not name it | `--force`, noting "which no cargo record in `<root>` claims". This is the binary-install case. |
| `pkg <id>`, where `<id>` is this `$pkg` | cargo already tracks it as this package | No `--force`; cargo upgrades it or reports "already installed". |
| `pkg <id>`, another package | a different package owns it | **Refuse this component**, count it failed, name the owner, and print both ways out: `cargo uninstall --root "<root>" <owner>` and re-running with `--force`. Only a user-supplied `--force` overrides, and it says whose binary it replaces. |
| `unknown` | Malformed or unreadable `.crates.toml`, `.crates2.json` without `.crates.toml`, or the two disagreeing | No `--force`, with a note. cargo then refuses safely if the file is in the way. |

**The parser handles cargo's real format.** My first version read only
single-line entries. Run against a real cargo-written `.crates.toml`, it
returned `unknown` for every bin, because cargo writes a package with
several bins as a **multi-line** array. That failed safe but was useless.
It is now a small awk state machine. Accepted forms:
- the `[v1]` header
- `"<id>" = ["<bin>", …]`
- `"<id>" = [` followed by `"<bin>",` lines and a closing `]`

Any other line, a duplicate header, an entry before the header, or an
unclosed array makes the whole file `unknown`.

**Proof against real cargo records.** In one root I installed:
- `fake-mk` (the reviewer's fixture: bin `mk`, via `cargo install --path`)
- `multi` (bins `alpha` and `beta`, which cargo wrote as a multi-line array)
- `ms-cli` from the `ms-cli-v0.19.0` git tag

cargo wrote both `.crates.toml` and `.crates2.json`. The `crates_owner`
function extracted from `install.sh` returned identical results under gawk
and BusyBox awk:

| bin | result |
|---|---|
| `mk` | `pkg fake-mk 0.1.0 (path+file://…/fake-mk)` |
| `alpha`, `beta` | `pkg multi 2.3.4 (…)` |
| `ms` | `pkg ms-cli 0.19.0 (git+…?tag=ms-cli-v0.19.0#4f1e99eb…)` |
| `nonexistent` | `none` |

**Real end-to-end, under dash, with real cargo:**
- **The reviewer's repro** (`--from-source --only mk` into that root):
  rc=1, and the output reads
  `error: …/bin/mk belongs to cargo package 'fake-mk 0.1.0 (…)', not mk-cli; not replacing it …`.
  cargo was never run, and `fake-mk`'s `mk` is byte-identical (md5 before
  and after `862920e6…`; it still prints `I am fake-mk`).
- **Both printed ways out, each on a copy of that root:**
  - `cargo uninstall --root … fake-mk`, then re-run: rc=0, `mk 0.13.0`.
  - Re-run with `--force`: rc=0, with the note
    "--force given: replacing …, owned by cargo package 'fake-mk …'" and
    cargo's "Replaced package `fake-mk` … with `mk-cli v0.13.0`".
- **My own binary-then-source case still works:** `--only mk,ms` binary
  into a spaced root, then `--from-source --only mk,ms`. rc=0, "2
  installed", `mk 0.13.0` / `ms 0.19.0`, and `.crates.toml` now tracks
  both. No stray directory from word-splitting.

**Related hole closed in the same decision.** cargo is now always given
`--root "$ROOT"` (`cargo_root`). Previously it got `--root` only when the
user passed one. With a cargo `install.root` config, cargo would then have
installed into, and applied `--force` in, a different root from the one
`crates_owner` inspected. **Behaviour change:** a user who relied on cargo's
`install.root` config without `--root` or `CARGO_INSTALL_ROOT` now gets
source builds in `$CARGO_HOME`, which is where binary installs already went.

## Minor (test 14) → fixed

Test 14 now covers:
- `mktemp` exiting 0 but printing a **relative** path (`relative/evil-path`,
  run with the test dir as cwd)
- `mktemp` exiting 0 with an **absolute path it never created**

Both are refused with an empty `rm`/`mkdir` log, and no `relative/` appears.

A new case reaches `install_binary`'s own guard, which no test reached
before. A curl stub deletes the temp root during the first component's
download, so the second component is refused with "temporary directory …
is gone" and is never fetched.

## Tests added (install-verify, now 47)

| Test | Case |
|---|---|
| 31 | The real cargo `.crates.toml` shape (paths shortened), with `fake-mk` owning `mk`: refused, cargo not run, file untouched, owner and `cargo uninstall` line present. |
| 32 | Owner inside a multi-line bin array: refused. |
| 33 | User `--force`: cargo gets `--force`, and the note names the owner. |
| 34 | `mk` tracked as `mk-cli`: cargo runs without `--force`. |
| 35 | Four malformed `.crates.toml` shapes (not TOML; no header; unterminated string; unclosed array): no `--force`, with a note. Also `.crates2.json` alone, and `.crates2.json` naming `mk` when `.crates.toml` doesn't: no `--force`. |
| 36 | No records at all: `--force`, with the corrected note. |
| 16 | Its note assertion was updated to the new wording. |
| 30 | Extended: with no `--root`, cargo gets `--root $CARGO_INSTALL_ROOT` explicitly. |

## Mutations: 13/13 killed

Each mutation was applied to a copy of `scripts/`, asserted to match once,
and required to turn `install-verify` red.

| # | Mutation | Test that caught it |
|---|---|---|
| F1 | Fold 2's package-name grep restored | 16, 31 |
| F2 | Another owner auto-forced | 31, 32 |
| F3 | Unreadable records force | 35 |
| F4 | `.crates2.json` without `.crates.toml` read as no record | 35 |
| F5 | `.crates2.json` disagreement ignored | 35 |
| F6 | Multi-line arrays not read | 31, 32 |
| F7 | Malformed lines ignored | 35 |
| F8 | User `--force` no longer overrides | 33 |
| F9 | Own package forced | 16, 34 |
| F10 | No explicit `--root` for cargo | 30 |
| G1 | `make_tmp_root` accepts any non-empty path | 14 |
| G2 | `install_binary`'s guard removed | 14 (temp root deleted mid-run) |
| G3 | G1 + G2 together | 14 |

G1 and G3 are the re-review's #4 and #6, which survived before. G3
reproduces the original defect inside the test dir
(`rm -rf relative/evil-path/mk; mkdir -p relative/evil-path/mk/x`), and the
suite now goes red on it.

## Gates

| Gate | Result |
|---|---|
| install-verify under bash, dash 0.5.13.5, BusyBox 1.37.0 `sh`, yash 2.61 | 47/47 each |
| msrv-guard and man-step under each shell | 6/6 and 4/4 |
| install-assets (under dash) | 32/32 mappings + 17/17 Linux glibc floors |
| shellcheck 0.11.0 on `install.sh` | clean |
| Examples golden | regenerated, byte-identical |
| fmt | 1.95.0, rc=0 |
| clippy | 1.85.0, `-D warnings`, rc=0 |
| nextest | 4057 passed, 20 skipped |

Not re-run, because this fold changed neither docs nor the pin table: the
manual audit, flag lint, quickstart and sibling-pin-check.

## Concerns

- **The `install.root` behaviour change** described above.
- **`.crates2.json` is not parsed.** It is used only as a disagreement
  signal. If a future cargo drops `.crates.toml` and keeps only
  `.crates2.json`, every existing binary reads `unknown`: no `--force`,
  so cargo refuses and names the owner. That fails safe, but a user would
  then need `--force` for the binary-then-source case.
- **The owner id is echoed as cargo wrote it.** An id containing `"` is
  treated as malformed (`unknown`), which also fails safe.

## Housekeeping

`tk-worktrees/f676-fold3-scratch` (245 MB: fixture crates, the real-cargo
roots, `mutate.py`, logs) was deleted with `find -delete`. No `/tmp/tmp.*`
dirs remain. The worktree is clean.
