# FIX: developer and build gates that cannot fail (bh-me-I12, bh-me-I13, bh-me-I14, bh-me-I19)

- Source: bughunt 2026-11-19 keys **bh-me-I12, I13, I14, I19**, all graded **Important** (grade-me.md rows
  I12, I13, I14, I19; `design/agent-reports/bug-hunt-2026-11-19.md` §I12, §I13, §I14, §I19). Architect table
  row #52, **small** track, one Opus review round. Not class B. No T-ruling is named for this row and none is
  needed; Brian's product rulings B1-B10 do not touch these paths.
- `.github/` is allowed in this row's PR (architect table); §2 D-CI uses it. Say so in the PR description.
- Base: master `e09d538d` (`git fetch` 2026-11-22). The grade head was `ff5ecfea`; no commit since touches
  the four defect sites. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.
- Every new or changed printed text is in §4 and marked **PROPOSED (architect confirms)**. No flag, type or
  command is added. §7 lists the points this document decides that the bug report and rulings do not
  literally state.
- **Files the build changes** (for build ordering against other me rows):
  `crates/me-cli/src/preview.rs`, `scripts/build-preview.sh`, `scripts/gen-mt1-vectors.py`,
  `demo/sh2/build.sh`, `.github/workflows/release.yml`, `ci/code-docs-exempt.txt`, `crates/me-cli/CHANGELOG.md`.
  New files: `crates/me-cli/tests/preview_version_probe.rs`, `scripts/test/gen_mt1_vectors_test.py`,
  `scripts/test/demo-sh2-embed-stale.test.sh`, `scripts/test/fixtures/mk-codec-v0.1-vectors.json`.
  Neighbour note: row #50 (I18) changes `preview/main.go`, not `preview.rs`; no other me row in the table
  names these files.
- Already committed on this branch: this document, the gate script `scripts/fix-me-dev-gates-gate.sh`, and the
  four new files as evidence copies under `design/evidence/fix-me-dev-gates/` (the RED tests and the corpus,
  byte-identical to what the builder adds at the paths above).

## 1. The defects (measured on master e09d538d)

### 1.1 I12: the mt1 vector generator certifies itself without its corpus

`self_test` (`scripts/gen-mt1-vectors.py:228-258`) checks the independent mk corpus only `if mk_vectors.exists()`;
a missing file prints `  skip  mk corpus not found at <path>` and the self-test still passes. A present corpus
passes when every string verifies under some family (`(r + l) == len(strs) and len(strs) > 0`), so one string,
or a corpus of one family, passes. The HRP check (`:246-257`) is skipped silently when no regular-code string is
found. The default `--mk-vectors` (`:315`) is an absolute path in a sibling checkout on the author's machine, so a
standalone clone always takes the skip branch. No CI step runs the generator.

### 1.2 I13: the documented generation command overwrites both goldens with an empty corpus

The usage line (`:29`) is `gen-mt1-vectors.py --from-regtest <datadir> --rpcport <port> --out design/vectors`.
It has no `--tx`, so `main()` builds zero vectors and writes `"vectors": []` to `mt1_v1_vectors.json` and a
header-only `mt1_v1_vectors.md` (`:339-372`), exit 0, with no RPC made. `--tx` without `--from-regtest` reaches
`bitcoin-cli -datadir=None`.

### 1.3 I14: the demo build publishes a stale emulator after an embedded-asset change

`demo/sh2/build.sh:16-26` rebuilds `emu.wasm` only when a `*.go`, `go.mod` or `go.sum` under `$FORK` is newer
than it, and otherwise prints `== emu.wasm is current (<n> bytes)`. A `//go:embed` input (the fork's
`cmd/emu/sysw_test_payload.bin`, `sealed_test_payload.go:42`, `sysw_cards_payload.go`) is none of those, so an
embedded-only change ships the previous wasm, exit 0.

### 1.4 I19: the runtime and the build check trust a failed `--version` probe

`preview::sidecar_version` (`crates/me-cli/src/preview.rs:111-125`) ignores the exit status and accepts the
first stdout line that, after `trim()`, starts with `me-preview` (no separator needed), returning the rest
trimmed. So a probe that exits 7, `me-preview0.13.0`, ` me-preview 0.13.0`, or `me-preview 0.13.0` followed by
more lines is accepted, and `me bundle --preview` renders, exit 0. `scripts/build-preview.sh --check`
(`:37-46`) runs `"$OUT" --version 2>&1 || true`: a probe that exits 7 with the right stdout prints
`ok: me-preview 0.13.0 matches the crate`, exit 0; a good probe with any stderr prints `DRIFT`. The build path
(`:58-62`) captures `2>&1` too, and under `set -e` a failed probe ends the script silently with the probe's
exit code. SPEC_me_bundle_phaseB_preview.md §4.1 (L23) is the contract: "`me-preview --version` → prints a
single line `me-preview <SEMVER>` to stdout, exit 0".

## 2. Decisions

- **D-I12a (portable corpus).** The mk corpus is vendored: `scripts/test/fixtures/mk-codec-v0.1-vectors.json`,
  byte-identical to `crates/mk-codec/src/test_vectors/v0.1.json` at mnemonic-key `cf36ebc0` (last changed by
  `6faf8a60`), sha256 `7591561f61863f91a65f00f8b004d958ee03eb4a2272f593fb48c54c2e5c6caa`. The `--mk-vectors`
  default becomes that file, resolved from the script's own directory (`Path(__file__)`), so it works from any
  cwd and any clone. `--mk-vectors <file>` still selects another corpus (e.g. a live mk checkout).
- **D-I12b (missing corpus fails).** A missing corpus file prints the T7 line and the self-test fails (exit 1,
  the existing `SELF-TEST FAILED — emitting no vectors.` on stderr). An unreadable or non-JSON file keeps its
  current behaviour (a Python traceback, exit 1).
- **D-I12c (membership pin).** A new constant `MK_CORPUS_COUNTS = (19, 21)` beside `POLYMOD_INIT`. For each
  string the self-test evaluates both families; a string counts as regular when it verifies under the regular
  code only, as long when under the long code only. The corpus line is `ok` only when every string counts once
  (`r + l == len(strs)`) and `(r, l) == MK_CORPUS_COUNTS`; otherwise `FAIL` with the same line format. This is
  the docstring's own claim (40 vectors, 19 regular, 21 long) made executable.
- **D-I12d (HRP check must run).** When the corpus has no string that verifies under the regular code, the
  self-test prints T9 and fails, instead of skipping. The HRP check runs whenever strings were read, whether or
  not the corpus line passed (as today when the file exists).
- **D-I13 (generation needs its inputs).** Right after `parse_args()`, before the self-test and before any
  output is opened: when `--self-test` is not given and either `--tx` is absent or `--from-regtest` is absent,
  `ap.error(T10)` (argparse prints the usage and the error line on stderr, exit 2). Nothing is written and the
  self-test does not run. `--self-test` with other arguments is unchanged (it validates, writes nothing, and
  ignores them). The usage docstring line becomes T11. Generation itself is unchanged.
- **D-I14 (always build).** `demo/sh2/build.sh` always checks `go` and runs `(cd "$EMU" && ./build.sh)`,
  printing `== building emu.wasm`; Go's build cache tracks every input, embedded files included. The freshness
  test and the `== emu.wasm is current (…)` line are removed, and the comment above becomes T13.
- **D-I19a (runtime probe).** `sidecar_version`: a non-success exit status returns an error carrying T1 (exit
  code, or `signal`; the sidecar's stderr, `from_utf8_lossy` and `trim()`med, appended after `: ` only when
  non-empty). On success, stdout (`from_utf8_lossy`) with one trailing `\n` removed, if present, must contain no
  `\n` and must start with `me-preview ` (one space); the version is the rest, not trimmed. Anything else is the
  existing `unexpected --version output: {stdout:?}` error. The caller (`main.rs:1157-1183`) is unchanged, so
  both errors print as `me: cannot determine me-preview version: <error>`, exit 2, before any render. stderr of
  a successful probe stays ignored. An empty version (`me-preview \n`, a plain `go build`) still returns `""`
  and gets the existing mismatch refusal; the two existing unit tests keep passing.
- **D-I19b (`--check`).** Capture stdout only (`rc=0; got="$("$OUT" --version)" || rc=$?`); the probe's stderr
  goes to the terminal. A non-zero `rc` prints T4 and exits 1 before the comparison. The `ok` and `DRIFT` lines
  are unchanged, with `<got>` now stdout only.
- **D-I19c (build path).** The post-build probe at `:58` captures stdout only the same way; a non-zero `rc`
  prints T6 on stderr and exits 1. The existing mismatch `FATAL` line is unchanged.
- **D-CI (run the gates in CI).** The `test` job of `.github/workflows/release.yml` gains one step after
  `Go build + tests (ndef-roundtrip oracle)`, named `Dev gates (mt1 vector generator, SH2 demo build)`, with
  `if: steps.docs.outputs.docs_only != 'true'`, running `python3 -I scripts/test/gen_mt1_vectors_test.py` then
  `bash scripts/test/demo-sh2-embed-stale.test.sh` (Go is already set up earlier in that job; the Rust tests run
  in the existing nextest step). A comment above the step names bh-me-I12, I13, I14 and FIX_me_dev_gates.
  Because CI now runs the generator, its line in `ci/code-docs-exempt.txt` moves out of the "Not run by CI"
  group into a new group with this comment: `# Run by CI only through scripts/test/gen_mt1_vectors_test.py,
  which points` / `# --out at a temporary directory: the generator names mt1 docs files only as` /
  `# output names and in its docstring, and reads no docs file.` (measured: without the exemption,
  code-docs-check flags `SPEC_mt_v0_1.md` at `:13` and `mt1_v1_vectors.md` at `:371`). The Python test builds
  the two output names from a stem so it names no docs file itself.

## 3. Spec delta

None. SPEC_me_bundle_phaseB_preview.md §4.1 already states the single-line, exit-0 contract; D-I19a makes the
host enforce it.

## 4. Printed texts and exit codes

| id | where, trigger | text (exact) | exit |
| --- | --- | --- | --- |
| **T1** (new, PROPOSED) | `me bundle --preview`, probe exits non-zero | `me: cannot determine me-preview version: me-preview --version failed (exit <code>)`, plus `: <stderr trimmed>` when stderr is non-empty; `<code>` is `signal` when killed | 2 |
| T2 (existing) | probe exit 0, stdout not the one line | `me: cannot determine me-preview version: unexpected --version output: "<stdout, Rust Debug>"` | 2 |
| T3 (existing) | version differs | the existing `me: me-preview version mismatch: sidecar is "<v>", expected "<crate>"; refusing to render.` block | 2 |
| **T4** (new, PROPOSED) | `build-preview.sh --check`, probe exits non-zero; stdout | `FAILED: <OUT> --version exited <rc> — run without --check` | 1 |
| T5 (existing) | `--check` | `ok: <got> matches the crate` (0) and `DRIFT: sidecar reports '<got>', crate is <VERSION> — run without --check` (1), `<got>` stdout only | 0/1 |
| **T6** (new, PROPOSED) | `build-preview.sh` build path, probe exits non-zero; stderr | `FATAL: built sidecar's --version exited <rc>` | 1 |
| **T7** (changed, PROPOSED) | generator self-test, corpus file missing | `  FAIL mk corpus not found at <path>` (was `  skip  mk corpus not found at <path>`) | 1 |
| T8 (existing format) | corpus counts | `  ok   mk corpus: <n>/<total> verify (<r> regular, <l> long)`, or `  FAIL` in place of `ok  ` under D-I12c | 0/1 |
| **T9** (new, PROPOSED) | no regular-code string for the HRP check | `  FAIL HRP separation: the corpus has no regular-code mk1 string to test` | 1 |
| **T10** (new, PROPOSED) | generation without `--tx` or `--from-regtest`; stderr after argparse's usage line | `gen-mt1-vectors.py: error: generation needs --from-regtest DATADIR and at least one --tx LABEL=HEXFILE:note; --self-test alone validates without writing` | 2 |
| **T11** (changed, PROPOSED) | usage docstring `:29` | `    gen-mt1-vectors.py --from-regtest <datadir> --rpcport <port> --tx <label>=<hexfile>:<note> [--tx ...] --out design/vectors` | n/a |
| **T12** (changed, PROPOSED) | docstring `:17-20` | `VALIDATED BEFORE USE, and this is not optional: --self-test reproduces all 40 of mk-codec's committed vectors (19 regular-code, 21 long-code), from the copy in scripts/test/fixtures/mk-codec-v0.1-vectors.json (mnemonic-key 6faf8a60) or the file --mk-vectors names. A missing corpus, or one that does not verify as exactly 19 regular and 21 long strings, fails the self-test. If this implementation were wrong, every vector it emits would be wrong and the whole defence would rest on a broken generator. Run it first.` (backticks as in the current docstring around `--self-test`, `mk-codec`, the path and `--mk-vectors`) | n/a |
| **T13** (changed, PROPOSED) | `demo/sh2/build.sh` comment `:16-19` | `# emu.wasm is gitignored in the fork and is rebuilt on every run: Go's build` / `# cache tracks every input, //go:embed payloads included. A freshness test on` / `# *.go and go.mod/go.sum shipped a stale wasm after an embedded-payload change` / `# (bh-me-I14), and an earlier cmd/emu-only test missed a gui/ change (2026-09-23).` | n/a |
| **T14** (new, PROPOSED) | `crates/me-cli/CHANGELOG.md` `[Unreleased]` / `### Fixed` | `- **\`me bundle --preview\` trusts the sidecar's version probe only when it exits 0 and prints exactly \`me-preview <version>\`** (bughunt-1119 bh-me-I19, Important). A probe that exits non-zero is refused with exit 2 and the sidecar's stderr; any other output shape is refused as before for an unreadable version; no plate is rendered. \`scripts/build-preview.sh --check\` no longer approves such a sidecar. The developer gates bh-me-I12, I13 and I14 (mt1 vector generator, SH2 demo build) are fixed alongside.` | n/a |

On T1/T2 `me bundle --preview` behaves as on the existing T3 refusal: `wire_previews` returns before the
manifest is serialised or written (`main.rs:885-888`). The `demo/sh2/build.sh` output is otherwise unchanged
(`== building emu.wasm` now on every run). No toolkit-manual text changes: `mnemonic-toolkit` origin/master
`docs/manual/src/` does not mention `me-preview` or `build-preview` (`git grep`, 0 hits).

## 5. RED rows (run by the author on base e09d538d; the four new files placed at their final paths)

All rows: `bash scripts/fix-me-dev-gates-gate.sh` (needs `$HOME/.cargo/bin`, Go 1.26.x and python3 on PATH;
`GATE_LOG_DIR` optional). On base: **`gate: 6 FAIL`** (G1, G2, G4, G5, G6, G8 FAIL; G3 and G7 ok, they guard the
build rather than show a defect). GREEN target: **`gate: 0 FAIL`**.

| row | command | base (seen) | GREEN |
| --- | --- | --- | --- |
| G1 / I12, I13 | `python3 -I scripts/test/gen_mt1_vectors_test.py` | `Ran 7 tests`, `FAILED (failures=6)`; the vendored-corpus control passes on this host only because the developer path exists | `Ran 7 tests`, `OK` |
| G2 / I12 | `grep -n 'mnemonic-key/' scripts/gen-mt1-vectors.py` | match at `:315` | no match |
| G3 | sha256 of the vendored corpus | ok (fixture placed) | ok |
| G4 / I13 | `grep -F` of T11 | no match | match |
| G5 / I14 | `bash scripts/test/demo-sh2-embed-stale.test.sh` | `== emu.wasm is current (2190657 bytes)` / `FAIL: second dist wasm lacks fixture-bravo (fixture-alpha lines: 1)`, exit 1 | `== building emu.wasm` / `PASS: second dist wasm contains fixture-bravo` |
| G6 / I19 | `cargo nextest run --locked --build-jobs 2 -j 2 -p mnemonic-engrave --test preview_version_probe` | `11 tests run: 2 passed, 9 failed` | `11 tests run: 11 passed` |
| G7 | `bash ci/code-docs-check.sh` | ok | ok |
| G8 / D-CI | `grep` of both script commands in `release.yml` | absent | present |

Per-test RED evidence (base):

- Python: `test_selftest_missing_corpus_fails` `0 != 1 : self-test passed without its corpus`;
  `test_selftest_one_string_corpus_fails`, `test_selftest_regular_only_corpus_fails`,
  `test_selftest_extra_string_fails` each `0 != 1` (passed on 1, 19 regular-only, 41 strings);
  `test_generation_without_tx_refuses_before_writing` `outputs rewritten (exit 0)`;
  `test_generation_without_from_regtest_refuses_before_writing` `1 != 2` (the generator reached RPC and died on
  the missing `bitcoin-cli`; the test runs every generator with an empty PATH so no node is ever contacted);
  `test_selftest_vendored_corpus_passes` ok (control).
- Rust: `version_probe_nonzero_exit_is_refused_before_any_render`, `version_probe_failure_text_carries_its_stderr`,
  `version_probe_without_separator_is_refused`, `version_probe_with_leading_space_is_refused`,
  `version_probe_with_a_second_line_is_refused`: each `probe … must be refused` with exit 0 and a rendered
  manifest; `build_preview_check_refuses_failed_probe`: `--check approved a failed probe`, stdout
  `ok: me-preview 0.13.0 matches the crate`, exit 0; `build_preview_check_passes_probe_stderr_through` and
  `build_preview_check_ignores_stderr_on_a_good_probe`: stdout `DRIFT: sidecar reports 'me-preview 0.13.0\n<stderr>'…`;
  `build_preview_build_refuses_failed_probe`: exit 7 with empty stdout and stderr (want 1 and T6).
  Controls PASS on base: `version_probe_success_renders_and_mismatch_refuses`, `build_preview_check_reports_drift`.

**GREEN probe (author, uncommitted, reverted).** A minimal implementation of D-I12a-d, D-I13, D-I14, D-I19a-c and
D-CI reached `gate: 0 FAIL`; on it the whole workspace `cargo nextest run --locked` gave 804 run, 802 passed, 2
skipped, 0 failed once the `third_party/seedhammer` submodule was initialised (`cross_lang` and
`preview_cross_lang` need it; the first run without it failed exactly those two, environment only), `cargo
+1.85.0 fmt --check` and both CI clippy commands on 1.85.0 were clean, and `go test ./...` in `preview/` passed.
So no existing test changes. (Local nightly clippy flags five pre-existing lints in `seal/`; CI pins 1.85.0.)

## 6. Tests

No existing test changes (probe-measured, §5). New tests, each named exactly as here; the evidence copies in
`design/evidence/fix-me-dev-gates/` are the RED versions and go in unchanged:

- `crates/me-cli/tests/preview_version_probe.rs` (11, `#![cfg(unix)]`; stub sidecar via `ME_PREVIEW_BIN` that
  logs every call): `version_probe_nonzero_exit_is_refused_before_any_render` (T1, no render),
  `version_probe_failure_text_carries_its_stderr` (T1 with `: probe broke`),
  `version_probe_without_separator_is_refused`, `version_probe_with_leading_space_is_refused`,
  `version_probe_with_a_second_line_is_refused` (T2 each, no render),
  `version_probe_success_renders_and_mismatch_refuses` (control: good probe renders, good probe with stderr
  renders, 0.12.0 gets T3 with no render), `build_preview_check_refuses_failed_probe` (T4, exit 1),
  `build_preview_check_passes_probe_stderr_through` (T4 on stdout, the probe's stderr exactly on stderr),
  `build_preview_check_ignores_stderr_on_a_good_probe` (T5 ok), `build_preview_check_reports_drift` (T5 DRIFT,
  control), `build_preview_build_refuses_failed_probe` (fake `GO` writes a stub whose probe exits 7: T6, exit 1).
- `scripts/test/gen_mt1_vectors_test.py` (7, unittest): `test_selftest_vendored_corpus_passes`,
  `test_selftest_missing_corpus_fails`, `test_selftest_one_string_corpus_fails`,
  `test_selftest_regular_only_corpus_fails`, `test_selftest_extra_string_fails`,
  `test_generation_without_tx_refuses_before_writing` (the printed usage form minus `--tx`, against two sentinel
  files: exit 2, T10, both unchanged, no `SELF-TEST` on stdout),
  `test_generation_without_from_regtest_refuses_before_writing`.
- `scripts/test/demo-sh2-embed-stale.test.sh` (fixture fork with an embedded payload, two assemblies, asserts
  the second dist wasm carries the new bytes; `mktemp -d`, removed on exit).
- The consultant's two-transaction byte comparison against a regtest node is not automated (needs a node; the
  generation path is unchanged by D-I13). The gate script says so.

## 7. Points this document decides that the bug report and rulings do not literally state (PROPOSED, architect confirms)

- **P-1 (D-I12a) vendor versus require a path.** The consultant offers "make the corpus portable or require an
  explicit path". Chosen: vendor, because CI can then run the self-test with no sibling checkout, and `--self-test`
  stays a one-word command. Cost: a 35 KB copy that can lag mk-codec; that is harmless, since it validates the
  generator's BCH code, not mk.
- **P-2 (D-I12c) pin counts, not the file hash.** The consultant asks to "pin its identity or expected
  membership". Chosen: the counts in the script (any corpus passed with `--mk-vectors` must also be 19 + 21), and
  the vendored file's sha256 in the gate (G3). A content-hash pin inside the script would refuse every live mk
  corpus.
- **P-3 (D-I19a) strictness.** The spec says "a single line"; chosen: exactly one line, one optional trailing
  `\n`, no trimming, so a leading space, a missing separator and a second line are all refused (T2). Today's
  `trim()` is dropped; a genuine Go sidecar (`fmt.Fprintln(stdout, "me-preview", version)`) is unaffected.
- **P-4 (T1) stderr in the refusal.** "Preserve stderr as diagnostics" is read as: include it in the refusal
  text, mirroring the existing render-failure text (`me-preview render failed (exit <c>): <stderr>`), but without
  the trailing `: ` when stderr is empty.
- **P-5 (D-I19c) the build path is in scope.** The bug report names only `--check`; its fix says "before
  rendering or approving a build". The build path already fails closed, but silently and with stderr mixed in;
  T6 makes it name the cause.
- **P-6 (D-CI) CI wiring and the exemption move.** Allowed by the table; without it the two script tests never
  run. The exemption comment is reworded rather than the generator's docstring.
- **P-7 (D-I13) `--self-test` plus generation flags.** Left as today (validate only, ignore the rest) rather
  than adding a new refusal.
