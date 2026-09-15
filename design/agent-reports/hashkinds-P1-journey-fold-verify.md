# hashlock-kinds P1 journey-walk fold — verification

**Scope, per brief: two questions only.** (1) Did the fold close each finding?
(2) Did the fold introduce a new defect? Not a fresh audit. Verification
preferred executing over reading throughout.

**Fold commits verified:**
- `descriptor-mnemonic` `65eab419` (worktree
  `/scratch/code/shibboleth/dm-worktrees/hashkinds-p1`, branch `hashkinds-p1`) —
  F-A1.
- `mnemonic-engrave` `63d7d87c` — F-C1.

Both repos built clean (`cargo build`, no warnings) on their gated toolchains.

---

## CLOSED

### F-A1 — CLOSED, no defect found

Claim: a new `warning:` in `md compose`, printed when **every** path carries a
hashlock (the preimage is the only way to spend), naming the kind(s) and what is
lost; fires for all four kinds; does not fire when any path is keyed-only or
timelocked-without-a-hash; does not fire for `--preset hashlock-gated` (its
second path is keyed+timelocked, no hash).

Ran every shape named in the brief against `./target/debug/md` built from
`65eab419`:

| # | shape | expected | observed |
|---|---|---|---|
| 1 | `--path 2of3,ripemd160=HEX` (only path) | warn | **warned**, names `ripemd160` |
| 2 | `--path 2of3,ripemd160=HEX --path 1of1,sha256=HEX` (both hashlocked) | warn | **warned**, names `ripemd160, sha256` |
| 3 | `--path 2of3,ripemd160=HEX --path 1of1` (one keyed, no hash) | quiet | **quiet** |
| 4 | `--path 2of3,ripemd160=HEX --path 1of1,older=144` (keyed+timelock, no hash) | quiet | **quiet** |
| 5 | `--path 2of3 --path keyless,ripemd160=HEX --experimental` (keyless shape) | quiet (F-A1 warning); only the pre-existing EXPERIMENTAL warning | **quiet on F-A1**; `warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the preimage)` still fires as before |
| 6 | `--path keyless,ripemd160=HEX --experimental` (keyless-only) | refused before the warning code runs | `md: every wallet needs at least one path with a key`, exit=1 — confirmed by reading `compose()`: the `?` on `compose(&list)` returns before the new warning block |
| 7 | `--preset hashlock-gated,ripemd160=HEX,older=144` | quiet | **quiet** — confirmed in `presets::hashlock_gated` (`crates/md-codec/src/compose/presets.rs:92`): path 1 = 1of1+hash, path 2 = 1of1+timelock **without** hash, so `all(hash.is_some())` is false |
| 8 | `--wrapper tr --path 2of3,ripemd160=HEX` | warn | **warned**, identical wording |
| 9 | `--wrapper wsh ... --json`, stdout/stderr captured separately | warning on stderr only; stdout pure JSON | confirmed: warning + `note:` both on stderr; stdout parsed as valid JSON via `python3 -c "json.load(...)"`, prints `VALID JSON` |
| extra | keyed+hash path **plus** keyless+hash path (`--experimental`) — a case not in the brief's list, added because it's the one shape where "at least one keyed path" and "all paths need hash" can both hold simultaneously | warn | **warned**, correctly names both kinds (`hash256, ripemd160`), alongside the unrelated EXPERIMENTAL warning |
| extra | single path combining key + hash + timelock (`1of1,ripemd160=HEX,older=144`) | warn (timelock doesn't provide an escape — read the lowering) | **warned**; lowered to `and_v(v:pkh(...), and_v(v:ripemd160(...), older(144)))` — the timelock is ANDed in, not an alternative branch, so "no timelock matures" is literally true even here |
| extra | `--wrapper sh-wsh --path 2of3,ripemd160=HEX` | n/a — legacy wrapper refuses hash entirely, pre-existing and unrelated | `md: legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr`, exit=1 — confirms the new code path is never reached for `sh`/`sh-wsh`, no interaction |

**No shape found where the preimage is genuinely the only way to spend and the
tool stays silent, and no shape found where the warning fires on a wallet that
has a real keyed escape.** The condition (`!paths.is_empty() &&
paths.iter().all(|p| p.hash.is_some())`) is exactly right for what it needs to
decide: whether *every* spending branch is gated on the preimage, independent of
how many of those branches also carry keys.

**Mutation-proven, both directions, confirmed by direct mutation of
`crates/md-cli/src/cmd/compose.rs` (not just trusting the commit's claim):**

- Under-warning: `if false && !list.paths.is_empty() && ...` →
  `a_wallet_with_no_keys_only_path_warns_that_the_preimage_is_the_only_key`
  failed with "sha256: no warning on the shape where the preimage is the only
  way to spend". Mutation confirmed applied (grepped the literal `if false`
  before running).
- Over-warning: `.all(...)` → `.any(...)` →
  `a_wallet_with_a_keyed_escape_gets_no_preimage_warning` failed with "the
  second path spends with keys and a timelock, so the preimage is NOT the only
  way in" and the warning text present in the captured stderr.
- File restored and diffed byte-identical to the pre-mutation copy after each
  mutation (`diff` → `RESTORED`/`RESTORED_CLEAN`), `git status --porcelain`
  empty at the end.

### F-C1 — CLOSED, no defect found

Claim: `me sysw pack`'s advice beside a rejected `hash:` record now names the
tagged form as legitimate, says not to delete a kind tag, and states what
deleting it does.

Ran the built `./target/debug/me` (from `63d7d87c`) on tagged `hash256`,
`ripemd160` and `hash160` records — all three refused (40/64-hex-per-kind check
unchanged) and all three print the identical updated advice:

> `a hash record is `hash:` + a sha256 digest as 64 lowercase hex, or
> `hash:<kind>:` + that kind's digest for hash256, ripemd160 or hash160 — IF A
> RECORD ALREADY CARRIES A KIND TAG, DO NOT DELETE IT TO SATISFY THIS RULE: an
> untagged record is read as sha256, and a hash256 digest is also 64 hex, so
> stripping the tag is accepted and commits the payload to a different digest
> than the wallet`

Verified the two factual claims inside that sentence, not just its presence:

- **Untagged 64-hex is really accepted as sha256.** `echo
  'hash:98a20fc2...641cd488' | me sysw pack` → `exit=0`; `me sysw show` on the
  resulting file prints `public record 0: sha256 hashlock (hash:) —
  98a20fc2..641cd488` — the hash256 digest, now silently committed as a sha256
  hashlock. Exactly the trap the advice warns against.
- **Tagged ripemd160 (40 hex) is still refused after tag-stripping.** `echo
  'hash:09e7bb50...946b' | me sysw pack` → still `hash: must be exactly 64 hex
  characters`, exit=4. Confirms the hazard is hash256-specific, matching the
  journey walk's B.3 finding.
- **No contradiction found** against `ms hashlock`'s card text (which quotes
  this same error string verbatim, unchanged by this fold) or against `me sysw
  show`'s own classification. The only other places the superseded phrasing
  ("the 32-byte digest as 64 lowercase hex") appears are two historical design
  docs (`design/BRAINSTORM_hashlock_H6_preimage_plates.md`,
  `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`) — records of a
  past decision, not live advice text; out of the fold's scope.

**Mutation-proven, all three assertions independently, each mutation confirmed
applied before running:**

1. `DO NOT DELETE IT TO SATISFY THIS RULE` → `please delete it to satisfy this
   rule` — test failed on the "must warn against stripping" assertion.
2. `hash:<kind>:` → `hashKIND:` — test failed (checked separately, not shown
   verbatim above for brevity, same pass/fail shape).
3. `read as sha256` → `parsed as SHA-256` — test failed on the "must say WHAT
   stripping it does" assertion.

File restored to the pre-mutation copy after each mutation; final `diff`
against the backup was empty (`RESTORED CLEAN`).

---

## NEW DEFECTS FROM THE FOLD

None found. Both diffs are exactly what their commit messages claim:
`descriptor-mnemonic` `65eab419` touches only
`crates/md-cli/src/cmd/compose.rs` (+30) and adds
`crates/md-cli/tests/cli_compose_hashkinds.rs` (+61); `mnemonic-engrave`
`63d7d87c` touches only `crates/me-cli/src/main.rs` (+6/-1) and
`crates/me-cli/tests/sysw_composer_cli.rs` (+46) — matching `git show --stat`
exactly, nothing beyond the claimed scope in either fold.

---

## Full-suite re-measurement

**descriptor-mnemonic**, `cargo --version` = `cargo 1.85.0 (d73d2caf9
2024-12-31)` (repo's own `rust-toolchain.toml`):

```
cargo nextest run --locked --all-targets
Summary [22.271s] 1316 tests run: 1316 passed, 3 skipped
cargo clippy --workspace --all-targets --locked -- -D warnings   -> 0 errors
cargo fmt --all -- --check                                       -> clean
```

Matches the fold commit's claim (1316 tests, was 1314) exactly.

**mnemonic-engrave**, gated on CI's pinned toolchain (`cargo +1.85.0`; this repo
has no `rust-toolchain.toml`, so the bare `cargo clippy` here uses the
1.97.0-nightly default):

```
cargo +1.85.0 nextest run --locked --all-targets
Summary [32.184s] 647 tests run: 647 passed, 2 skipped
cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings   -> 0 errors
cargo +1.85.0 fmt --all -- --check                                      -> clean
```

Matches the fold commit's claim (647 tests, clippy 0, fmt clean) exactly.
Re-verified the toolchain-trap claim itself: bare `cargo clippy` (default
nightly, no `+1.85.0`) on this same tree reproduces **exactly 3** errors
(`manual implementation of .is_multiple_of()`, plus the two "could not compile"
cascades from it) — matching the commit message's count. Not re-diffed against
master in this pass (the commit message already records that check); the count
and error text match what the commit describes.

---

## Worktree cleanliness

`git status --porcelain` empty in both repos after verification (all mutation
edits reverted and diffed byte-identical to backups before restoring):

- `mnemonic-engrave`: `ME_CLEAN`
- `descriptor-mnemonic` worktree `hashkinds-p1`: `DM_CLEAN`

---

## Counts and verdict

- Findings closed: **2 of 2** (F-A1, F-C1).
- New defects introduced by the fold: **0**.
- Critical: 0. Important: 0. Minor: 0. Nit: 0.

**GREEN.**
