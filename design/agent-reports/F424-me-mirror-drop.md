# F-424 close attempt — HELD, gate never cleared — 2026-08-31

**Outcome: no work performed. Tree left exactly as found.** `HEAD` unchanged
at `81afebdf90523caa73478f55c7d7afe874a8aede`, `git status --short` empty
before this report was written. No dependency bump, no code edit, no other
commit.

## Task

Close F-424 (`design/FOLLOWUPS.md`): once `md-codec` ≥0.43.0 publishes
carrying the F-217/F-218 encode validators (`validate_origin_key_consistency`,
`validate_no_duplicate_key_slots`), bump `me`'s `md-codec` dependency in
`crates/me-cli/Cargo.toml` (currently `"0.42"`) and drop the host-side mirror
of those two checks — `conjunct_8_key_identity` in
`crates/me-cli/src/descriptor/admit.rs:281-301` (called from `admit()` at
`:77`; helpers `same_origin`/`origin_text` at `:306-316` are private to it).
**Hard gate before any of that: md-codec 0.43.0 must be live on crates.io.**

## Gate attempts

Checked crates.io's registry API directly (`crates.io/api/v1/crates/md-codec`
requires a `User-Agent` header or it 403s):

1. One-shot check before starting work: `max_version: 0.42.0`, `0.43.0` not
   in the versions list.
2. A background poll (task `biiytu56p`) then ran every 3 minutes for the
   session's duration. Five attempts logged, all identical:
   `attempt 1..5: max_version=0.42.0`. No sixth attempt ran — the poll was
   stopped on the coordinator's stand-down instruction (below) rather than
   reaching its own 30-minute budget or finding 0.43.0.

## Why the gate is unreachable this session

The publish itself was halted, in the primary repo, before it happened. Full
record: `descriptor-mnemonic/design/agent-reports/publish-2026-08-31-md-codec-0.43.0.md`
(that repo's publish agent, same day). Summary, verified by reading that
report in full:

- `cargo publish -p md-codec --dry-run` (default features) fails to
  **compile** the packaged tarball: 3× `E0599` —
  `Descriptor::derive_at_index` and `Descriptor::into_definite`
  (`derive.rs:147,149`) and `Terminal::SortedMultiA`
  (`to_miniscript.rs:459`) — none of which exist on `miniscript` 13.1.0, the
  version the registry resolves to. All three exist only under this
  workspace's `[patch.crates-io]` git-fork pin (`ff4732e5`, for upstream
  PR #953/#915), which is a workspace-local override and correctly does
  **not** travel into a published crate's dependency graph.
- The default `derive` feature (`default = ["derive"]`) is what pulls in
  `miniscript`, so a published 0.43.0 as currently written would fail to
  build for every downstream consumer using default features.
  `--no-default-features` dry-run succeeds — only `derive` is broken.
- Confirmed a new regression this cycle, not a pre-existing published
  defect: the `md-codec-v0.42.0` tag's source uses only crates.io-compatible
  API names and has no `[patch.crates-io]` block at all.
- The publish agent stopped at the dry-run per its own brief ("if the
  dry-run flags anything ... STOP and report rather than publishing a crate
  that won't build for consumers"), reverted the version bump / lockfile /
  changelog draft it had prepared back to the 0.42.0 baseline, and pushed
  one independent, unrelated CI fix (`man-pages.yml`) that had been bundled
  in the same cycle. No tag was created; `cargo publish` (non-dry-run) was
  never invoked. `md-codec` remains `0.42.0` on crates.io.

F-424 stays **open**, now blocked on that new defect (gating `derive` behind
a feature the registry build can satisfy, or waiting for upstream PR
#953/#915 to land on crates.io, or vendoring the needed surface — an
operator decision, per that report).

## Recon performed before the gate check, kept for the next attempt

Before polling, the mirror and its call sites were read in full (no edits).
Recorded here so a future attempt does not re-derive it:

- The mirror is `conjunct_8_key_identity` (`admit.rs:281-301`), reached from
  `admit()` (`:47-80`) as the last of eight conjuncts. Tests exercising it by
  name: `row_key_identity` / `row_key_identity_duplicate`
  (`tests/descriptor_refusals.rs:536,554`) and
  `as_descriptor_on_multi_still_reports_key_identity`
  (`tests/descriptor_as.rs:734`) — all CLI-level, asserting exit code 3 and
  §6's verbatim refusal text.
- `admit(d, Path::…)` is called from **three** sites, and in at least two of
  them the call sequence never reaches `md_codec` at all, even after the
  dependency bump: `identify::block` (`identify.rs:46`, decides the
  full-vs-partial identification tier before any encode), `gate::carriage`
  (`gate.rs:234-235`, decides `AsDecides`/refuse before either `--as` path
  runs, and `main.rs:1536`'s `AsDecides` branch prints the *choice block* at
  a different exit code (`EXIT_USAGE`) if `admit()` does not refuse first),
  and `as_flag::descriptor_follower` (`as_flag.rs:132`, gates directly ahead
  of `Decision::Pack(vec![d.encode()])`).
- `--as descriptor`'s pack call, `Parsed::encode()`
  (`cascade.rs:375`/`encode_no_checksum`), is a local string formatter with
  **no call into `md_codec`** — confirmed by reading it in full. So even
  once 0.43.0 does publish and link, a literal unconditional drop of
  `conjunct_8_key_identity` would leave `--as descriptor` (and the
  no-`--as`-flag classification path, which decides via `admit()` before
  either follower runs) with **no enforcement at all** of the F-217/F-218
  checks, not merely a differently-worded one. Only `--as md1`
  (`md1::build` → `md_codec::split` → `encode_payload`, confirmed in
  `descriptor-mnemonic/crates/md-codec/src/{chunk,encode}.rs`) actually
  reaches the codec's validators — and even there, `as_flag.rs:118-125`'s
  catch-all currently wraps any `md1::build`/`strings` error as
  `Row::UseSiteOutOfSet` with a generic message, not the specific
  `key-identity`/`key-identity-duplicate` row text the tests assert
  verbatim.
- Net: this is independent, additional reason to expect the literal "drop
  the mirror, verify tests still pass" plan to fail its own safety check
  once the gate does clear — not just a bet that the published crate might
  not enforce the checks. The next attempt's build-gate should treat this as
  a near-certain finding to confirm empirically (per standing convention:
  recompute, do not assume), not a fresh question.

## What was NOT done

No `Cargo.toml` edit, no `cargo build`, no code removed, no spec text
touched, no test run, nothing pushed. The background poll (task `biiytu56p`)
was stopped via `TaskStop` on the coordinator's instruction before it could
reach its own budget.
