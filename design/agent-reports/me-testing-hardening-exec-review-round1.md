# me funds-safety + testing-hardening — post-implementation adversarial execution review (round 1, scoped fold-verification)

**Reviewer:** independent opus execution reviewer (mandatory Step 12 gate, round 1).
**Scope:** SCOPED verification of the round-0 fold commit `7565f62` (L1/L2/N1
closure), confirming it closes the round-0 Low/Nit findings without regression and
that nothing else on the branch changed since round 0.
**Target branch:** `me-testing-hardening` in worktree
`/scratch/code/shibboleth/mnemonic-engrave-testing-hardening`, tip
`7565f62e6c2b645ad8a2b8e56ee52dd960cba9d7`.
**Date:** 2026-07-06.
**Sources read:** round-0 review (`…-exec-review-round0.md`, L1/L2/N1 defs), the
complete fold delta (`git show 7565f62`), the branch-tip `validate.rs` / `bundle.rs`,
the impl-log fold section, and the mk-codec 0.4.1 registry source
(`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mk-codec-0.4.1/`).

## Verdict: **GREEN (0 Critical / 0 Important)**

The fold commit `7565f62` closes round-0 L1, L2, and N1 correctly, fail-closed, and
secret-safe. No behavior regression: full suite **82 pass / 0 fail / 0 skip** under
`ME_REQUIRE_GO=1` with Go present; `go test ./...` (preview) green. Scope is exactly
the four intended files, and `7565f62` is the sole commit on top of the round-0 tip
`b586b29` (13 commits total on `master..`). One purely-informational observation on
the `.gitignore` pattern breadth — not a finding, and matches a round-0-blessed option.

---

## Commit-graph & scope discipline (task item 4) — CLEAN

- `git log --oneline master..` → **13 commits**, `7565f62` on top of `b586b29`
  (the round-0 target). Confirmed.
- `git show --stat 7565f62` and `git diff --name-only b586b29 7565f62` both list
  **exactly four** paths — no more, no less:
  - `.gitignore` (+3)
  - `crates/me-cli/src/bundle.rs` (+12/−3: Display comment reword + B8 canary variant)
  - `crates/me-cli/src/validate.rs` (+8: the redaction arm)
  - `design/agent-reports/me-testing-hardening-impl-log.md` (+19: fold record)
- `git status --porcelain` → **empty** (clean worktree, no stray/untracked drift).
- No source outside `validate.rs`/`bundle.rs` was touched; no Cargo.lock/Cargo.toml
  change; no new `.ndef`/fixture churn. Zero branch drift since round 0.

## L1/L2 closure (task item 1) — CLOSED, fail-closed, secret-safe

**The redaction arm redacts fully.** `validate.rs:36-38`:
```
ValidateError::Mk(mk_codec::Error::InvalidHrp(_)) => {
    write!(f, "invalid mk1 string: invalid or missing HRP")
}
```
- The payload `String` is bound to `_` and **discarded** — no `{e}`, no `{0}`, no
  `{:?}` Debug form, no fragment. The rendered text is a fixed static string. No
  input byte can reach stderr through this arm.
- Arm **order** is correct: the specific `InvalidHrp(_)` arm precedes the general
  `ValidateError::Mk(e) => "…: {e}"` fallthrough, so first-match-wins routes InvalidHrp
  to the redacted arm and every other variant to the (metadata-only) passthrough.
- `mk_codec::Error` is `#[non_exhaustive]`; matching one named variant with a trailing
  general `Mk(e)` arm compiles cleanly (whole suite compiles + passes, proving it).

**The B8 canary genuinely exercises the pass-through (not test theater).** The new
variant (`bundle.rs:368-371`)
`BundleError::Validate(CANARY, ValidateError::Mk(mk_codec::Error::InvalidHrp(CANARY)))`
routes through the *real* Display chain: `BundleError::Validate` arm
(`"invalid input string: {e}"`) → `ValidateError::Mk(InvalidHrp)` arm. Perturbation
proof (rsync'd scratch copy at `/scratch/code/shibboleth/me-review-scratch-r1`, the
worktree itself **never modified**; scratch deleted after use): with the redaction arm
**deleted**, `cargo test --lib no_bundle_error_display_leaks_the_input_body` goes
**RED** —
```
BundleError Display leaked the input body:
"invalid input string: invalid mk1 string: invalid HRP: CANARY_SECRET_BODY"
```
— confirming the canary catches the exact InvalidHrp pass-through, and that the
worktree's arm is what suppresses it. Restored (worktree unchanged), the test is green.

**Single-variant redaction is sufficient — NOT whack-a-mole.** Enumerated every
`mk_codec::Error` variant reachable from `decode_string` (mk-codec-0.4.1
`string_layer/bch.rs:658` — the sole constructor of `ValidateError::Mk`, via
`validate.rs:83`) against the registry source:
| Variant (reachable from `decode_string`) | Payload | Leak? |
|---|---|---|
| `MixedCase` | none (static) | no |
| `InvalidHrp(String)` | input substring (entire lowercased input on the no-`1` branch, `bch.rs:668`; HRP-prefix on `bch.rs:673`) | **YES → REDACTED** |
| `InvalidStringLength(usize)` | data-part length | no (metadata) |
| `InvalidChar { ch, position }` | one offending char + index | no (single-char diagnostic; same class round-0 accepted for `MdNonCanonical{ch}`) |
| `BchUncorrectable(String)` | fixed diagnostic text (`"regular code: more than 4 substitutions…"` / position+count, `bch.rs:434,450,488,503`) — never input | no |

`InvalidHrp` is the **only** `decode_string` variant carrying an input substring, so
one redaction point closes the surface. All bytecode-layer variants
(`UnsupportedVersion`, `PathTooDeep`, `InvalidPathComponent(String)`,
`InvalidXpubPublicKey(String)`, `ChunkedHeaderMalformed(String)`, …) are unreachable
via `decode_string` and, where they carry a `String`, it is fixed diagnostic text, not
input (verified in `error.rs` + the `parameterized_variants_render` test).

**Second raw-`mk_codec::Error` surface checked and cleared (beyond the task ask).**
`BundleError::SetIncompleteMk(String, mk_codec::Error)` (`bundle.rs:30`, Display
`"…is incomplete/inconsistent: {e}"`, `bundle.rs:67-69`) carries a **raw** codec error
(NOT wrapped in `ValidateError`), so the `validate.rs` redaction does not cover it.
Adversarial trace: its error comes from `mk_codec::decode(&refs)` (`bundle.rs:279`) at
reassembly, over strings in `mk1_groups`. Those strings **already** passed
`parse_line` → `validate::validate(Format::Mk)` → `decode_string` **Ok** (and a second
`decode_string` + Chunked-header parse in `parse_line`, `bundle.rs:111-116`) before
they could be grouped (`bundle.rs:200-227`, collected with `?`). `decode_string` is a
deterministic pure function: a string that returned Ok (HRP confirmed `mk`, separator
present) cannot subsequently yield `InvalidHrp`. Therefore `SetIncompleteMk`'s `e` can
only be a reassembly/bytecode error (`ChunkSetIdMismatch`, `CrossChunkHashMismatch`,
`MalformedPayloadPadding`, `ChunkedHeaderMalformed`, `MixedHeaderTypes`, bytecode
variants) — all metadata-only. `InvalidHrp` is **unreachable** on this path. The B8
test's documented exclusion of `SetIncomplete*` (`bundle.rs:349-351`) is therefore
sound. No residual leak; the single redaction genuinely covers **every** me-reachable
`InvalidHrp`:
- convert path — `ConvertError::Validate(ValidateError)` (`lib.rs:36` `"{e}"`) delegates
  to the redacted `ValidateError` Display ✓
- bundle validate path — `BundleError::Validate(_, ValidateError::Mk(InvalidHrp))`
  (`bundle.rs:107,112,116`) delegates to the redacted arm ✓ (exactly the canary)
- bundle reassembly path — `SetIncompleteMk` raw error: `InvalidHrp` unreachable ✓

**L2 (canary coverage) closed:** the B8 test now includes a variant that flows through
the codec-wrapped `ValidateError::Mk` pass-through — the actual residual surface
round-0 flagged as unexercised.

**Doc-accuracy folded:** the `bundle.rs:54-59` Display comment no longer overclaims
"codec text is metadata-only (verified)"; it now states the InvalidHrp exception and
where it is redacted. The impl-log fold section (`…-impl-log.md`, "Fold: exec-review
round 0") records L1+L2+N1 accurately and explicitly retracts the earlier Step-3
"metadata-only" overstatement (round-0 L1). Accurate.

## No behavior regression (task item 2) — CONFIRMED

- `env PATH="/home/bcg/.local/go/bin:$PATH" ME_REQUIRE_GO=1 cargo test --locked` in the
  worktree → **82 pass, 0 fail, 0 skip**: 54 lib + 23 cli + 1 cross_lang +
  3 golden + 1 preview_cross_lang (+0-test main.rs/doc harnesses). Matches the
  round-0 baseline exactly — the fold added test *assertions* (the B8 variant) without
  changing the test count (B8 is one `#[test]` iterating a `Vec`).
- `go test ./...` in `preview/` → `ok mnemonic-engrave/preview`.
- The only Display behavior that changed is the InvalidHrp sub-case (previously would
  have printed the substring; now prints a fixed message). Valid mk1 admission
  (`accepts_valid_mk1`), the `MkCorrected` non-pristine path (`rejects_corrupted_mk1`),
  the md1 paths, and every other error arm are byte-for-byte unchanged and green. No
  accepted input's engraved bytes change (goldens green). No funds-safety regression.

## N1 (.gitignore) — CLOSED, correctly scoped

`.gitignore` gained (line 7) a bare `ndefroundtrip` under a `# Go build artifact
(firmware/ndef-roundtrip)` comment — the "bin-glob" option round-0 explicitly blessed.
Verified:
- `git check-ignore -v firmware/ndef-roundtrip/ndefroundtrip` →
  `.gitignore:7:ndefroundtrip` (the build binary IS ignored).
- `git check-ignore firmware/ndef-roundtrip/` → **not ignored** (the hyphenated source
  dir and its tracked `go.mod`/`main.go` are untouched — the pattern `ndefroundtrip`
  does not match the dir `ndef-roundtrip`).
- `git ls-files | grep ndefroundtrip` → **none** (no tracked path is named
  `ndefroundtrip`, so nothing tracked is accidentally ignored).
- `git status` clean. N1's "a stray `git add -A` could stage the artifact" hazard is
  now closed.

**Informational (not a finding):** the pattern is un-anchored (no leading `/`), so it
would ignore any *future* file/dir named exactly `ndefroundtrip` anywhere in the repo.
Harmless — no such path exists, the real dir is hyphenated, and this is precisely the
round-0-offered "bin-glob" form. Path-anchoring (`/firmware/ndef-roundtrip/ndefroundtrip`)
would be marginally tighter but is not warranted.

## Test-theater / methodology notes

- The RED perturbation was performed in an rsync'd scratch copy only; the worktree was
  read but never written. Scratch (and its `target/`) removed after use — verified gone.
- The scratch build compiled `md-codec v0.40.0` + `mk-codec v0.4.1` (the pinned
  versions), so the RED result reflects the real dependency set.

---

## Findings summary

| # | Sev | Status |
|---|-----|--------|
| L1 (round-0) | Low | **CLOSED** — `ValidateError::Mk(InvalidHrp(_))` redacts fully (static text, payload discarded); sole input-carrying `decode_string` variant; `SetIncompleteMk` raw-error path proven InvalidHrp-unreachable. |
| L2 (round-0) | Low | **CLOSED** — B8 canary now exercises the codec-wrapped pass-through; perturbation confirms genuine RED. |
| N1 (round-0) | Nit | **CLOSED** — `ndefroundtrip` gitignored; ignores only the binary, no tracked path affected. |

**Round-1 new findings: 0 Critical / 0 Important / 0 Low / 0 Nit.**

**0 Critical, 0 Important → GREEN.** The fold commit `7565f62` closes all three
round-0 items correctly and fail-closed, introduces no regression (82/82 green, Go
green), and confines itself to exactly the intended four files with no other branch
drift. Merge-eligible.
