# ms-cli-v0.16.0 pre-tag review — 2026-08-15

**This is an agent's advisory review (Fable 5, dispatched as the designated single highest-stakes pre-irreversible-action gate), not a human decision.**

Repo under review: `/scratch/code/shibboleth/mnemonic-secret` (`bg002h/mnemonic-secret`). Tag candidate: `d49d5c0` (local, unpushed). Last tag: `ms-cli-v0.14.1` at `ffc9d71`.

---

## VERDICT

**CUT AFTER: (1) `d49d5c0` earns its four required contexts green on a `ci/staging` push — g6 alone red is the expected state, not a failure — and (2) `master` is pushed to `d49d5c0` first, so the tag is cut on a commit that is on master and CI-proven. Tag LAST.**

No Critical or Important finding exists against the release *content*. The only unsafe element is the plan's *ordering*: it tags a SHA on which CI has never run (0 workflow runs exist at `d49d5c0` — verified via the workflow-runs API by `head_sha`) and omits pushing `master` entirely. An irreversible tag must not be the first thing that ever builds this SHA in CI.

---

## Q1 — Is it safe to cut `ms-cli-v0.16.0` now?

**RULING: Not *now*; safe after two ordering conditions, both cheap.**

Required order (replaces plan step 2):

```
git push origin master:refs/heads/ci/staging     # d49d5c0 earns its contexts
# wait; require green: test (ubuntu-latest), clippy, test (ms-codec), clippy (ms-codec)
#   — the rust workflow will show RED overall because g6 is red. That is the
#     plan's own predicted state. Only the four required contexts gate.
git push origin master                           # no bypass message = satisfied
git push origin ms-cli-v0.16.0                   # tag LAST (annotated, matching precedent)
git push origin --delete ci/staging
```

**WHY:**
- **CI has never executed at `d49d5c0`.** `actions/runs?head_sha=d49d5c0…` returns **0 runs**. The project's own rule: a gate that has never executed is a hypothesis. Local checks cover the Linux legs only — macOS, Miri, musl-cross, and FreeBSD legs exist only in CI.
- The risk is small but the cost asymmetry is total: the `de593ca` master run (run 31872122114) is a near-perfect rehearsal — its tree differs from `d49d5c0`'s **only** in `.github/workflows/rust.yml` + `CLAUDE.md` (verified: `git diff de593ca d49d5c0 --stat` = those 2 files), and it went **11/12 jobs green with only `g6 invariant` red** — exactly the red the plan predicts. So the staging run is expected green, but the tag is irreversible and the check costs minutes.
- The plan **never pushes master**. A tag alone leaves master at `2ebea45`; step 4 (drop the exemption) needs `d49d5c0` on master anyway, and after toolkit step 3 an unpushed master would make this repo's g6 red regardless. Master must move to `d49d5c0`, and per this repo's own CLAUDE.md ("Pushing master — agents stage"), via `ci/staging` so the required contexts are satisfied rather than bypassed.
- Tag **after** master, not before: if the staging battery surfaced a surprise, a pre-cut tag would already be public on a bad SHA.

**WHAT I VERIFIED:** `git status --porcelain` (clean but the 3 pre-existing untracked files → tag content is exactly `d49d5c0`); branch protection via `gh api …/branches/master/protection` → required contexts `["test (ubuntu-latest)","clippy","test (ms-codec)","clippy (ms-codec)"]`, `strict:false` (g6 and fmt NOT required); no tag rulesets; remote has no `v0.15`/`v0.16` tag or release (`git ls-remote --tags` grep exit 1; releases list ends at `ms-cli-v0.14.1`); previous tags are annotated tag objects (`git cat-file -t ms-cli-v0.14.1` → `tag`).

Local full gate at `d49d5c0` (all true exit codes, no pipes):
- `cargo test --workspace` → **exit 0, 409 passed / 0 failed** (summed from the log file by awk over 78 `test result:` lines; zero non-ok lines).
- `cargo clippy --all-targets -p ms-cli -- -D warnings` → exit 0; same for ms-codec → exit 0.
- CI fmt-gate logic replicated exactly → **PASS, zero diffs in any file** (mlock.rs included — the exemption is already unnecessary at this commit).
- `sha256sum -c design/display-grouping-vectors.tsv.sha256` → OK.
- Toolchain: `rust-toolchain.toml` pins 1.85.0 = MSRV, so local runs are MSRV runs.

---

## Q2 — What does the tag publish, and is any of it unintended?

**RULING: Fully enumerated; nothing sensitive; nothing unintended beyond one harmless extra workflow run.**

Workflows triggered by pushing tag `ms-cli-v0.16.0` (read from the workflow files, not assumed):

1. **`man-release.yml`** (`on: push: tags: ms-cli-v*`) — three jobs:
   - `man-tarball`: builds `ms`, self-emits man pages (`ms gen-man`, with a negative canary against `*-help*.1` shadow pages), `gh release create "$REF_NAME" --generate-notes` if absent, uploads **`ms-man.tar.gz`** (contains only generated roff pages).
   - `repro`: calls the toolkit-homed reusable `reproducible-musl-build.yml` @ pinned SHA `6e37b18e…`; `packages: write` — refreshes the **GHCR image `repro-musl-mnemonic-secret`** (private-by-default; not a release asset).
   - `musl-binaries` (needs `repro`): uploads **`ms-0.16.0-x86_64-linux-musl.tar.gz`** (built inside the digest-pinned repro container with `--network=none`), **`ms-0.16.0-aarch64-linux-musl.tar.gz`** (cross/QEMU, Cross.toml digest-pinned), **`SHA256SUMS.x86_64`**, **`SHA256SUMS.aarch64`**, **`PROVENANCE.x86_64.txt`**, **`PROVENANCE.aarch64.txt`**. Each binary tarball contains exactly one file: the `ms` binary (`tar -C target/…/release ms`). This matches byte-for-name the asset list I pulled from the live v0.14.1 release.
2. **`fuzz-smoke.yml`** — its `on: push:` has `paths:` but **no `branches:` filter**, and GitHub does not evaluate path filters for tag pushes, so the tag will trigger its `build` (compile-gate) job. The `smoke` job is `if: schedule || workflow_dispatch` — verified — so nothing runs long and **nothing uploads**. Harmless, just expect the extra run.
3. **NOT triggered:** `rust.yml` (branch-filtered: `[main, master, ci/**]`) and `vendor-freshness.yml` (branch-filtered: `[main, master]`).
4. **GitHub auto-attachments:** `Source code (zip/tar.gz)` = the tree at `d49d5c0` — all already-public content; the three untracked WIP files are not committed and therefore **not** in the archive. Auto-generated release notes render the 0.14.1..0.16.0 commit messages — all already public on pushed master.

**Sensitive-sweep check:** every `gh release upload` names files the job itself just created in a fresh CI checkout; no wildcard sweeps of the workspace. Nothing secret can be swept in.

**Release-completeness risk (not safety):** `musl-binaries` depends on `repro`. Preflighted its input: `cargo build --release --locked --offline -p ms-cli --bin ms` with the two-block vendored-sources config → **exit 0** ("Finished `release`"), so the committed `vendor/` satisfies `Cargo.lock`. `vendor-freshness` was also green on the pushed batch (`de593ca` head: success). If a leg still fails, assets are re-uploadable via re-run + `--clobber`; the tag itself stays sound.

---

## Q3 — The two never-tagged feature commits (`ddfa497` 0.15.0, `98e1f6a` 0.16.0)

**RULING: Sound, well tested, and correctly described by the CHANGELOG. 0 Critical / 0 Important. The funds-relevant facts are now machine-verified by two independent means.**

**WHAT I VERIFIED:**

1. **BIP-48 protocol facts against the authoritative BIP text** (fetched `bitcoin/bips/bip-0048.mediawiki`, not from memory):
   - Path `m / purpose' / coin_type' / account' / script_type' / change / address_index` — line 58 of the BIP. Matches `Template::account_path`.
   - Exactly two registered script types: `1'` Nested Segwit (p2sh-p2wsh), `2'` Native Segwit (p2wsh) — BIP lines 103–110. Matches `script_type()`; no `0'`, no Taproot — the code's refusal of `bip48-p2tr` is correct (and tested).
   - *"The recommended default for wallets is pay to witness script hash `m/48'/0'/0'/2'`"* — BIP line 112, **verbatim**. The CHANGELOG's and stderr note's "BIP-48 recommends it" claim is true, not marketing.
2. **All five derivation pins reproduced by a from-scratch independent implementation.** I wrote a pure-Python (stdlib-only) BIP-39/BIP-32/secp256k1 oracle — zero shared code with rust-bitcoin, the `bip39` crate, or the SeedHammer Go implementation the commit used — and derived the abandon×11-about wallet:
   - master fingerprint `73c5da0a` — **MATCH**
   - `m/48'/0'/0'/2'` → `xpub6DkFAXWQ2dHxq2vatrt…` — **MATCH**
   - `m/48'/0'/0'/1'` → `xpub6DkFAXWQ2dHxnMKoSBo…` — **MATCH**
   - `m/48'/0'/1'/2'` → `xpub6DzhyrnFFYQ1HimDiM3…` — **MATCH** (pins that `--account` moves the account level, not script_type)
   - `m/48'/1'/0'/2'` testnet → `tpubDFH9dgzveyD8zTbPUFu…` — **MATCH** (coin_type `1'` + tpub version bytes)
   Oracle exit 0. A wrong derivation path placing a cosigner key where the operator did not intend is now excluded by three mutually independent implementations agreeing (rust-bitcoin, SeedHammer Go, this review's Python).
3. **Tests** (`crates/ms-cli/tests/cli_derive_bip48.rs`, 13 tests, all green in the 409): oracle matches for both script types; non-collapse of the two script types; `--account` lands on the account level; testnet coin type; phrase≡hex; `bip48-p2tr` refused; single-sig names unchanged; bare `bip48` accepted → `2'`; assumption announced on stdout (`(DEFAULT)`), stderr (names the alternative), and `--json` (`script_type_defaulted` true/false/false-for-single-sig). The failure modes that matter for funds (silent wrong path, silent wrong level, silent default) each have a dedicated test.
4. **Executed the built binary** (`ms 0.16.0`): bare `--template bip48` produces exactly the documented announced-default behavior (stdout `script_type: 2' p2wsh (native segwit) (DEFAULT)`, loud stderr note naming `bip48-p2sh-p2wsh`, watch-only advisory), and the emitted xpub matches my oracle.
5. **Code review:** the path string is built once and both derived-from and printed-from (`account_path`); `ScrubbedXpriv` confinement (move-only, no Debug, best-effort scrub on drop) is preserved unchanged; the JSON change is purely additive (`script_type_defaulted` mirrors `language_defaulted`). The 0.15.0→0.16.0 reversal story in the CHANGELOG is accurate and the 0.16.0 entry candidly documents it.

**Process note, discharged:** no reviewer-loop record existed for this phase in `design/agent-reports/` (verified: `grep -rl bip48` → exit 1; newest report commit 2026-06-23). Release-checklist item 4 was therefore *unexecuted*, not "passed". **This review is that gate**, run at 0C/0I with the math independently verified. Recommend copying this report into `mnemonic-secret/design/agent-reports/` in its own commit so the record lives in the repo it gates.

---

## Q4 — Is `v0.16.0` the right version at this commit?

**RULING: Yes. No `v0.16.1`, no CHANGELOG amendment required.**

**WHY:** Everything in `ms-cli-v0.14.1..d49d5c0` beyond the two feature commits is non-semantic for the shipped crates: `9a24999`/`d476b77` (CI + CLAUDE.md), `430008b`/`ef57a51` (test-only; ef57a51 turned the pre-existing red parity test into a documented skip), `bf77f89`/`27a8f64` (docs), and `de593ca`+`2ebea45`+`d49d5c0` (net effect: mlock.rs 1.95.0-formatted — two line-wraps, no semantic change; verified `git diff de593ca d49d5c0 -- crates/ms-cli/src/mlock.rs` = 0 lines, so the tag pins exactly the shape already CI-rehearsed at `de593ca`). Repo precedent explicitly ships test/CI-only content without a bump (`0.14.1` "test-only PATCH", `0.13.2` "binary-asset-only PATCH"). `Cargo.toml` = `0.16.0`, CHANGELOG `## ms-cli [0.16.0] — 2026-08-15` present and dated today, binary reports `ms 0.16.0`, and 0.15.0 is deliberately never tagged (both entries live in the CHANGELOG; nothing anywhere references a `ms-cli-v0.15.0` tag). One nit, not blocking: previous releases used a dedicated `release:` commit; here the tag lands on a fmt commit — put the release summary (incl. the mlock fmt/g6-lockstep transition) in the **annotated tag message**, which `d49d5c0`'s commit message already drafts well.

---

## Q5 — Does the release checklist pass?

**RULING: All items that bind an ms-cli tag are met (item 3 conditionally — it is Q1's condition; item 4 met by this review). The document itself is stale for ms-cli tags and should be refreshed in a follow-up.**

| # | Item | Binds ms-cli tag? | Status |
|---|------|-------------------|--------|
| 1 | Wire-format SHA pin | Only if the ms-codec vector corpus changed | **N/A-met** — corpus last touched at `a5a9091` (v0.1.1 era); ms-codec stays 0.7.0, untouched by this release |
| 2 | CHANGELOG entry | Yes | **Met** — 0.15.0 + 0.16.0 entries, correct crate prefix, accurate |
| 3 | CI gate green | Yes (as the *actual* surface: rust.yml's 12 jobs, not the checklist's fictional "stable+beta+MSRV three-row matrix") | **Met locally** (test 409/0 exit 0, clippy ×2 exit 0, fmt PASS, vector pin OK, at MSRV 1.85.0) + rehearsed in CI at the tree-identical-modulo-CI-config `de593ca` (11/12 green, g6 the expected red). **In-CI execution at `d49d5c0` itself is Q1's condition.** |
| 4 | No open Critical/Important per-phase findings | Yes | **Met** — `design/FOLLOWUPS.md` swept: open items are `test-infra` residual (parity guard dormant), cross-repo tracking, note-only, or upstream-blocked; none Critical/Important. The BIP-48 phase's missing review record is discharged by this report (0C/0I, math independently verified). |
| 5 | MIGRATION.md | Only for wire/API changes | **N/A-met** — additive CLI values only; no wire, codec, or API change |
| 6 | Cross-repo notification | Yes | **Met by the plan itself** (step 3 is the toolkit commit). Note: honor the standing `manual-cli-surface-mirror` invariant — the new `--template` values belong in the toolkit-side user manual with/after step 3. |
| 7 | `cargo publish --dry-run` for ms-codec | ms-codec releases only | **N/A** — no crates.io publish is triggered by an `ms-cli-v*` tag (no workflow does `cargo publish`). Stale parenthetical: neither Cargo.toml actually carries `publish = false` (grep: no match) — harmless today, worth a doc fix. |
| 8 | Tag and push | Yes (adapted) | Becomes Q1's sequence: staging → four contexts → master → annotated tag |

---

## Q6 — Is the four-step sequencing correct, and is the g6 red window unavoidable?

**RULING: The direction is correct and forced; the window is real, unavoidable in this direction, minimizable, and confined to a non-required context. The plan has one genuine hole — it never pushes this repo's master — fixed by Q1's ordering.**

**WHY the direction is forced (verified against both repos' workflow files):**
- The toolkit's g6 resolves the ms-cli ref **dynamically from `scripts/install.sh`** (toolkit `rust.yml` "Resolve pinned ms-cli tag from install.sh", step `g6-sibling-master-not-pin`; pin currently `ms-cli-v0.14.1`, install.sh line 38). A formatted toolkit copy can only be green once a **formatted tag exists to pin** — so the tag must be cut first, and toolkit-first ordering is impossible without a toolkit-side red window *plus* a dangling state.
- Given the tag, the toolkit converges in **one commit** (pin→v0.16.0 + format its mlock.rs + drop its exemption): its g6 flips green atomically. No toolkit-side red at all.
- The residual red is exactly this repo's g6 (compares against toolkit **master**) from the first CI build containing `d49d5c0` until toolkit step 3 merges. g6 is not a required context (verified in branch protection), so nothing is blocked; the `de593ca` incident already demonstrated this exact red is the only casualty.

**Plan corrections/improvements:**
1. **Insert the missing master push** (via `ci/staging`), and cut the tag last — Q1.
2. **Pre-stage the toolkit step-3 commit before pushing the tag** so the red window is minutes, not hours.
3. Step 4 (drop this repo's exemption) is safe immediately after step 3: at `d49d5c0` the replicated fmt gate already passes with **zero** diffs, mlock.rs included — the exemption is dead code the moment both copies are formatted. Also update the stale in-file comments that still say "g6 pins the FROZEN ms-cli-v0.7.0 tag" (rust.yml exemption note) when dropping it.
4. Expectation-setting for the operator: the staging run's **rust workflow will conclude `failure`** (g6) while all four required contexts pass — that is the planned state, and pushing master will report satisfied, not bypassed.

---

## Residual observations (Minor/Nit — none gate)

- `design/RELEASE_PROCESS.md` is written for `ms-codec-v*` and describes a CI matrix that does not exist; refresh it for ms-cli tags (owning phase: post-release docs).
- The dormant-parity residual (`parity-smoke-toolkit-version-drift`) is honestly filed; option (c) (third-party codex32 cross-check) is the only one restoring real independence — unchanged by this release.
- `fuzz-smoke` will run its compile gate on the tag push because its `on: push:` lacks a `branches:` filter (tag pushes skip path filters). Harmless; add `branches:` if the noise bothers.
- Consider whether `DeriveJson.schema_version` should bump past `"1"` on additive fields someday; today's addition mirrors the `language_defaulted` precedent and is safe.

## Post-review tree state

`git status --porcelain` after all checks: only the three pre-existing untracked files (`.claude/`, `cycle-prep-recon-codex32-vendor-fork-cluster.md`, `design/SPEC_codex32_vendor_fork_cluster.md`). No tracked file was modified; no tag, push, or release action was taken.
