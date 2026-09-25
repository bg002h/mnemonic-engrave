# F-687 re-review — fold 1 (ms `79343e2`, toolkit `6034922e`)

Agent-written (Sonnet 5), 2026-09-25. Brief: `design/briefs/f687-rereview.md`.

**One question:** did fold 1 fix the F-687 review's findings (M1–M5, N1–N4), and
did it introduce a defect, especially a wrong or empty passphrase, or a stdin
read twice or by the wrong input?

**Answer:** All 9 findings are fixed and verified against the binaries. No
wrong-passphrase or double-stdin-read regression was found. Four new items
surface, all Nit/Minor and none blocking: an over-refusal when both a secret
flag's path and stdin are `/dev/null` (no real double-read risk since
`/dev/null` is stateless); a parity gap between ms and toolkit for a
space-separated leading-dash literal under `--allow-argv-secret` (toolkit's
clap layer refuses it as a usage error; ms accepts it via its side-channel
architecture — pre-existing, not introduced by this fold); and a test-coverage
gap where `/proc/self/fd/0` is never exercised by name in either repo's
automated suite (confirmed harmless — it's fully subsumed by the inode
fallback on Linux, verified by mutation).

Binaries built from the two fold-1 SHAs (`cargo build --locked`, debug;
toolkit reports `mnemonic 0.104.0`, ms reports `ms 0.19.1`), each with its own
`CARGO_TARGET_DIR` under `/scratch/code/shibboleth/review-f687b-scratch/`.
Neither worktree was left modified: `git status` is clean and `HEAD` is
unchanged in both (`ms` `79343e2`, toolkit `6034922e`) — verified after every
mutation round.

---

## 1. M1 — stdin by path or inode, plus false positives

**Fixed.** Every named form and the inode fallback are refused correctly on
both binaries:

- `ms derive --in /dev/stdin|/dev/fd/0|/proc/self/fd/0 --template bip84
  --passphrase -` with the ms1 on stdin: all three refuse with `error: cannot
  read both the entropy source and --passphrase from stdin`.
- By inode: `ms derive --in card.ms1 --passphrase - < card.ms1` (stdin
  redirected from the exact file also named by `--in`) refuses the same way.
- Control (no collision): `--in card.ms1` (a file distinct from stdin) with
  the passphrase piped for real succeeds, fingerprint `b4e3f5ed` (matches the
  review's TREZOR baseline).
- Toolkit `silent-payment --secret-file /dev/stdin --passphrase -` with the
  seed on stdin: refuses (`... or a --secret-file that is stdin ...`).
  `verify-bundle --bundle-json /dev/stdin --passphrase -` and `bundle
  --import-json - --passphrase -` (M4) both refuse the same way.

**False positives, per the brief's list:**

| scenario | result | verdict |
|---|---|---|
| `< f` plus `--in f` / `--secret-file f` (regular file, same inode) | refused | **arguably fine, and I call it fine.** For a *pipe* or a TTY, two independent opens of the same underlying object genuinely share consumable bytes (verified below for a TTY: the tty driver's input queue is per-device, not per-fd). For a *regular file*, two independent `open()`s have independent offsets, so there is technically no byte-draining race — but the guard can't cheaply distinguish "regular file, safe to reopen" from "special file that shares state" from the path alone, and refusing loudly is safe. Not a defect. |
| stdin closed (`<&-`) | **not falsely refused** — succeeds, derives the no-passphrase wallet (`73c5da0a`) at exit 0 | This is the *pre-existing, already-settled* empty-passphrase behavior (closed stdin reads as EOF, same as empty stdin), unrelated to the M1 path-matching logic under test here (the `--in` value in this case is a plain file, not a stdin-alias). Per the brief's settled scope, not reported as a finding. |
| stdin a TTY | **not falsely refused** — verified with a real pty (Python `pty.openpty`, raw mode, fed `TREZOR\n` + Ctrl-D): `--in card.ms1` (regular file) + `--passphrase -` over the TTY succeeds, fingerprint `b4e3f5ed` | No false refusal. |
| stdin `/dev/null` alongside `--in /dev/null` / `--secret-file /dev/null` | **refused** on both ms and toolkit | **New Nit (below).** Every open of `/dev/null` shares the same device+inode, so the inode fallback fires — but `/dev/null` is stateless (every reader gets independent EOF, always); there is no actual data to steal between two "readers." Contrast: `--secret-file /dev/null` with a *different* stdin (a real file, not `/dev/null`) succeeds normally — confirmed on both binaries. |

## 2. M2 — one predicate (ms guard/resolver unification)

**Fixed**, with one caveat found while checking parity.

`--passphrase " -"`, `"\t-"`, `"- "`, `"--"` — classification (material vs.
channel) is now **identical** in ms and toolkit for all four, confirmed via:

- No override: all four refused pre-parse on both CLIs (exit 1 ms / exit 2 toolkit — codes differ per each CLI's existing convention, unrelated to F-687).
- With `--allow-argv-secret`, `=`-joined spelling (`--passphrase=- `): **both** derive fingerprint `3ca82432` — byte-identical, matches the new vector row.
- `" -"` and `"\t-"` space-separated with the override: both derive the same fingerprints (`e20c1882`, `0d50a4ff` respectively) on ms and toolkit.

**Caveat (new Minor, not a fold-1 regression):** `--passphrase "- "` (dash-space, **space-separated**, not `=`-joined) with `--allow-argv-secret`:
- ms: succeeds, literal `"- "`, fingerprint `3ca82432`.
- toolkit: `error: unexpected argument '- ' found` — clap itself refuses to accept a token starting with `-` as a value in space-separated form.

Root cause, confirmed by reading the code: ms's `argv_guard::substitute()`
reroutes an admitted secret value through a side-channel and hands clap a
literal `"-"` placeholder — clap never sees the raw hyphen-leading value. The
toolkit's `argv_guard.rs` has **no equivalent substitution function** (grepped
for `fn substitute`/`side channel`/`admitted`: none); it only decides
refuse-or-not and, on "allow," lets clap parse the raw argv directly, so
clap's own default hyphen-value restriction applies. This architecture
predates F-687 fold 1 (unaffected by this diff) — not something the fold
introduced. Both CLIs fail *loudly* (a usage error), so no wrong or silent
passphrase results; it's a pre-existing parity gap surfaced by this specific
check. `--` (double-dash) also differs in wording between ms and toolkit but
both refuse it (exit 1 / exit 2) — same non-issue.

## 3. M3 — mutations

Re-ran G1/G2/G3 against the actual `every_vector_case_holds` test, applying
each mutation to the fold-1 tree, confirming red, then `git checkout` to
restore (verified clean after each):

| mutation | repo | result |
|---|---|---|
| G1: `resolve_env` resolves `@env:` a second time | toolkit | **RED** — `want cc24affa, got ... fp Some("b4e3f5ed")` |
| G1: same, ms | ms | **RED** — same assertion text |
| G2: `strip_one_newline` also strips a lone trailing `\r` | toolkit | **RED** — `want f934e53f, got ... fp Some("48efb44f")` |
| G3: same (shared fn), ms | ms | **RED** — identical assertion |

**New mutation:** dropped `"/proc/self/fd/0"` from `path_is_stdin`'s by-name
match arm (leaving `/dev/stdin` and `/dev/fd/0`), ms repo. Ran the whole
`f687_passphrase_channels.rs` test file (8/8 passed) **and** the whole
`ms-cli` package suite (all suites green, matching the earlier 640/640 total).
**The mutation survives — nothing goes red.** This is not a functional
regression: I independently confirmed on the real (un-mutated) binary that
`ms derive --in /proc/self/fd/0 ...` is refused correctly, and the reason the
mutation is invisible is that `/proc/self/fd/0`'s target and `/dev/stdin`'s
target both resolve (via `std::fs::metadata`'s symlink-following) to the exact
same underlying file, so the inode fallback always catches it too. Confirmed
by grep: neither `crates/mnemonic-toolkit/tests/cli_f687_passphrase_channels.rs`
nor `crates/ms-cli/tests/f687_passphrase_channels.rs` contains the string
`proc/self/fd` anywhere — the fold's own doc-comment and the review both name
it as a third distinct form, but no automated test pins that specific string.
**New Nit** (test-coverage precision only; runtime behavior is correct).

## 4. M4, M5, N1–N4 — spot-checks against the binaries

All confirmed as described in the fold report:

- **M4**: `bundle --import-json - --slot @0.phrase=@env:S --passphrase -`
  → `error: --passphrase - and --import-json - all read stdin; ...` (refused,
  not swallowed as JSON). Same guard confirmed on `verify-bundle --bundle-json
  /dev/stdin --passphrase -`.
- **M5**: `--passphrase` help now states the `-`/`@env:` rule on `bundle`,
  `verify-bundle`, all three `xpub-search` modes, and `slip39 split` (checked
  `--help` output directly on the binary for each).
- **N1**: `@env:BADVAR` with non-UTF-8 bytes → `env-var BADVAR referenced by
  sentinel is set but not valid UTF-8` on both ms and toolkit (exit 1). A
  non-UTF-8 argv byte → `error: argument N on argv (0 is \`mnemonic\` itself)
  is not valid UTF-8; refused (the value is not shown)`, exit 64, **the bytes
  never appear on stdout or stderr** (checked byte-for-byte on the toolkit
  binary via a raw-bytes subprocess call).
- **N2**: `slip39 combine --share - --share @env:S --passphrase -` →
  `"...across --share, --from, and --passphrase -)"`; the same with
  `--passphrase-stdin` keeps the old wording byte-for-byte.
- **N3**: literal `--passphrase TREZOR` refusal lists `--passphrase -
  (stdin; or --passphrase-stdin, or --passphrase @env:VAR)`.
- **N4**: the manual's `41-mnemonic.md` "How `--passphrase` is read" table
  matches the binaries' actual behavior (refusal exit code, `@env:` error
  cases, exact-`-`-only stdin rule, file-path-that-is-stdin coverage).

## 5. Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | **640/640 passed, 11 skipped** | **4072/4072 passed, 20 skipped** |
| clippy 0.1.85 (`~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` prepended), `--workspace --all-targets -- -D warnings` | clean | clean |
| `cargo fmt --all -- --check` | clean | clean |
| vector files | `cmp` passes — **byte-identical**, **37 cases** in both `crates/ms-cli/vectors/passphrase_channels.json` and `crates/mnemonic-toolkit/tests/vectors/passphrase_channels.json` | |

Both nextest counts match the fold report's own claim exactly (640/640·11 and
4072/4072·20).

## New findings

- **Nit.** `--secret-file /dev/null` / `--in /dev/null` alongside stdin also
  `/dev/null` is refused as a "second stdin reader," though `/dev/null` is
  stateless and no data can actually be stolen between two readers of it.
  Harmless (fails loudly, not silently); could special-case `/dev/null` out of
  `path_is_stdin` if the false refusal is judged worth removing, but it isn't
  worse than telling the user nothing.
- **Minor.** ms and toolkit diverge on `--passphrase "- "` (space-separated,
  not `=`-joined) under `--allow-argv-secret`: ms accepts it as the literal
  (fp `3ca82432`); toolkit's clap layer refuses it outright (`unexpected
  argument`) because the toolkit has no argv-substitution mechanism like ms's.
  Pre-existing architectural difference, not introduced by fold 1. Both fail
  safe. Recommend a follow-up noting the toolkit workaround is `=`-joined
  syntax, or adding ms-style substitution to the toolkit if parity is wanted.
- **Nit.** Neither repo's automated test suite exercises the literal string
  `/proc/self/fd/0` (grep: zero matches in either `f687`-tagged test file).
  Runtime behavior is correct and confirmed unaffected by removing it from the
  by-name match (subsumed by the inode fallback on Linux) — this is a
  test-precision gap only, not a functional one.
- Confirmed *no* new instance of a wrong/silent/double-read passphrase defect
  anywhere in this fold. The empty-passphrase-on-closed-stdin behavior
  observed while probing false positives is the same pre-existing,
  already-settled gap named in the original review (item 4) — not reported
  as a new finding per the brief's settled scope.

**ready to ship: yes**
