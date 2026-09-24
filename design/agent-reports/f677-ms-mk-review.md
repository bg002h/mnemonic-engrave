# F-677 / F-670: independent review of the ms + mk branches

Date: 2026-09-24. Reviewer: a single agent, independent of the implementer, with no subagents.
Scope: `mnemonic-secret` `f845bac..75f2168` and `mnemonic-key` `9ccd549..1b8c524`, against F-677 and
F-670 in `mnemonic-engrave/design/FOLLOWUPS.md`, and the implementer's report
`design/agent-reports/f677-ms-mk-impl.md`. THE QUESTION: is each help text or hint now TRUE when run,
and did anything break? Everything below was executed, not read.

Toolchain: 1.85.0 for build/test/clippy (rustup toolchain bin prepended to PATH), 1.95.0 for `cargo
fmt` (both repos' CI toolchain), matching the implementer's report.
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f677-{ms,mk}-target` (real disk, not `/tmp`'s tmpfs).
Both worktrees are clean (`git status --porcelain` empty) at the end of this review.

## 1. ms: build + every EXAMPLES line against fixtures

`cargo build --release -p ms-cli` succeeded. `cargo nextest run --locked --workspace`: **613/613
passed, 11 skipped** — matches the implementer's claimed count exactly. `cargo clippy -p ms-cli -p
ms-codec --all-targets -D warnings`: clean. `cargo +1.95.0 fmt --all --check`: clean.

I independently re-extracted every verb's `EXAMPLES:` block from the built binary's `--help` output
(not from the source diff) and ran every line by hand against fresh fixtures in a scratch directory,
separately from the implementer's own `f677_help_matches_behaviour.rs` test. All 12 verbs with
examples (`derive` has none, as documented) ran to completion:

- **encode** (5 lines), **decode** (4), **inspect** (3), **verify** (3), **repair** (4), **split**
  (4), **combine** (5), **hashlock** (4), **vectors** (3), **gui-schema** (3), **gen-man** (2) — all
  exit 0 (repair's damaged-input examples exit 4 by design, matched `repair --in broken.txt` etc.),
  all `| jq FILTER` tails resolved to a non-null value, and none was refused by the argv guard.
- One run initially failed: `ms split --language japanese --in phrase-ja.txt -k 2 -n 3 --json | jq
  .shares` exited 1 with "`--in` reads a PHRASE and never sniffs the file's contents." This was **my
  own fixture bug** — I had populated `phrase-ja.txt` with `ms encode`'s ms1 *output* instead of an
  actual Japanese BIP-39 phrase. Regenerated via `ms decode --in card.ms1 --language japanese` (the
  same zero-entropy phrase the implementer's test derives via the `bip39` crate directly), the example
  ran clean, exit 0, `.shares` a 3-element array. Not a defect in the branch.
- Confirmed the argv guard is a live refusal, not decorative: `ms encode --phrase "abandon … about"`
  on real argv exits 1 with "argument 3 on ARGV … Refused BEFORE the command line was parsed."
- No example line places a secret literally on argv anywhere in the 12 verbs — every secret-bearing
  example uses `--in FILE`, `-`/stdin with a redirect, or `--out`.

## 2. F-670: the `for md compose:` hint, verbatim, against md 0.20.2, all four kinds

`which md` → `md 0.20.2` (installed, matches the brief). For each of `sha256`, `hash256`,
`ripemd160`, `hash160`, I ran `ms hashlock --kind <kind> --hashlock-phrase-stdin` to get the real
printed fragment, substituted `<your other paths>` with `2of3` (same substitution the implementer's
report used), and ran the resulting `md compose --wrapper wsh --path 2of3 --path
keyless,<kind>=<hex> --experimental --md-only` command **verbatim**:

- All four kinds: **exit 0**, prints `wsh(or_d(multi(2,@0/…,@1/…,@2/…),<kind>(<hex>)))` plus the
  expected EXPERIMENTAL warning and the "no coordinator imports this" note.
- Negative controls, also run for real: `--wrapper tr` → exit 1, "this build will not put a key-less
  path in taproot"; `--wrapper sh` → exit 1, "legacy wrappers hold one plain sorted multisig only";
  `wsh` without `--experimental` → exit 1, "this policy needs --experimental"; `wsh --experimental`
  without `--md-only` → exit 1, "Pass --md-only to compose it anyway." Every claim embedded in the
  code comment and CHANGELOG entry is true as measured.

## 3. `ms split --out` / `--json` advisory

Ran fresh (not reusing the implementer's test binary):
- `--out FILE`, text: stdout 0 bytes, stderr has **0** occurrences of "carries private key material".
- No `--out`: stdout 180 bytes (the shares), stderr has **1** occurrence of the warning.
- `--out FILE --json`: stdout contains `"shares"`, stderr has **1** occurrence of the warning.

Fires exactly when stdout carries share material, matching the F-589 condition ported from `encode`.

## 4. `ms encode --group-size` behaves as its new help says

Ran `--group-size 3` four ways: plain (stdout unbroken ms1, stderr card grouped in 3s), `--out FILE`
(file holds the unbroken ms1, stdout 0 bytes), `--json` (`.ms1` unbroken), `--no-engraving-card`
(no card line printed at all, group-size has no visible effect). All four match the help text's claim
word for word: "Shapes the stderr `engraving card:` line and nothing else."

## 5. mk: version, `version_is_honest`, and toolkit blast radius

`cargo build --release -p mk-cli` succeeded; binary prints exactly **`mk 0.14.0-dev`**.
`cargo nextest run --locked --workspace`: **391/391 passed, 0 skipped** — matches the implementer's
claim. `cargo clippy --workspace --all-targets -D warnings`: clean. `cargo +1.95.0 fmt --all --check`:
clean. `ci/repro/vendor-freshness.sh`: OK.

`version_is_honest.rs` is a real assertion, not vacuous (confirmed by mutation — see §6). Verified the
CHANGELOG state it depends on directly: no `## [0.14.0]` heading exists yet
(`grep -n '^## \[' CHANGELOG.md` lists `[Unreleased]` then `[0.13.0]`…), `[Unreleased]` names "Next
release: 0.14.0" verbatim, `Cargo.toml`/`Cargo.lock` both say `0.14.0-dev`. Also spot-checked the
three CHANGELOG entries the implementer says were missing and added: commits `37a9524`, `1711228`,
`c1b56b9` exist with exactly the described subjects, and their named test files
(`csid_verification.rs`, `encode_repair_chunk_set_id_p2.rs`, `decode_verify_correction_note.rs`) exist
in `crates/mk-cli/tests/` and are part of the 391 passing. `git rev-list --count
mk-cli-v0.13.0..9ccd549` = **83**, matching the CHANGELOG's stated gap exactly.

**Does `-dev` break anything that parses mk's version?** No. Grepped
`/scratch/code/shibboleth/mnemonic-toolkit` for every mk-version site:
- `scripts/install.sh` (`component_info` table) and `.github/workflows/{quickstart,manual,manual-gui,technical-manual}.yml` all pin mk by a **git tag** (`mk-cli-v0.13.0`, or `v0.11.0` for the GUI arm)
  via `cargo install --git … --tag …`, never by parsing `mk --version`'s output. This branch creates
  no tag, so every one of these pins is untouched.
- `.github/workflows/sibling-pin-check.yml` cross-checks those same tag strings against
  `install.sh`'s table — also tag-keyed, not runtime-version-keyed.
- `docs/manual/src/40-cli-reference/44-mk-cli.md` uses the prose convention "(unreleased: mk-cli
  after 0.13.0: --flag)"; no CI job parses or asserts this string against a live `mk --version`
  (confirmed no hit for `mk --version` output comparison anywhere in `.github/workflows/*.yml` or
  `scripts/*.{sh,py}`; the one `mk --version` occurrence in `docs/manual/src/20-quickstart/21-install.md`
  is a "run this to check your install" instruction, not a pinned expected-output transcript).

So nothing in the toolkit's install.sh pin checks, sibling-pin-check, or the manual's version-gate
prose is machine-checked against `mk`'s actual `--version` string, and none of them reads
`Cargo.toml`/`Cargo.lock` directly — this branch changes nothing they consume. (The toolkit's own
"installed mk still says 0.13.0" F-677 sub-item is a *separate*, still-open piece of work in the
toolkit repo itself — this branch doesn't touch that repo and isn't claimed to close it.)

## 6. Mutation re-application (one per repo)

**mk:** reverted `crates/mk-cli/Cargo.toml` version to `0.13.0` (Cargo.lock left mutated too, since
`cargo test --offline` needs it consistent). `cargo test -p mk-cli --offline --test
version_is_honest` reddened: `the_version_is_either_a_released_heading_or_a_named_dev_build` panics
with "mk-cli 0.13.0 is a release version, but `## [Unreleased]` lists changes it does not contain --
bump to the next `-dev` version" — the exact message the implementer's report quotes. Restored via
`git checkout -- crates/mk-cli/Cargo.toml Cargo.lock`; worktree clean.

**ms:** reverted `crates/ms-cli/src/cmd/encode.rs`'s `--group-size` doc comment to the pre-fix wording
("Insert a separator every N characters in the emitted ms1 string…"). `cargo test -p ms-cli --locked
--test f677_help_matches_behaviour encode_group_size_help_describes_the_card_not_stdout` reddened:
panics with "the old wording is back: …", matching M1 in the implementer's report. Restored via `git
checkout -- crates/ms-cli/src/cmd/encode.rs`; worktree clean.

Both mutations killed for the stated reason, on the first try, with no other test collaterally
failing.

## Findings

**Critical: 0. Important: 0.**

**Minor (non-blocking, informational):** `ms repair --in FILE --out repaired.ms1`'s new example
comment reads "# the repaired ms1 alone, to an owner-only file." Measured: the *file* does hold only
the bare ms1 (confirmed byte-for-byte), but **stdout still carries the full report text, including the
corrected ms1 in plaintext**, exactly as before — `--out` does not silence repair's stdout the way it
does for `encode`/`split`. The comment is technically accurate (it describes the file's content, not
a claim that stdout goes quiet), and the implementer's own report flags exactly this ("repair --out
still prints the report … so its unconditional advisory stays true"), so it is not a
help-text-disagrees-with-behaviour defect — but a reader skimming the example could reasonably infer
stdout is now clean when it is not. Filing as a documentation-clarity Minor, not blocking.

No other divergence found between any help text, hint, or CHANGELOG claim and the branches' measured
behavior. Both repos' full workspace suites, clippy, fmt, and (for mk) vendor-freshness are green at
the reviewed commits, matching the implementer's report exactly (613/613 and 391/391). Both
worktrees are clean; nothing was committed by this review.

ready to ship: yes
