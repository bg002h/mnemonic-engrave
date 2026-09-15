# R0 round 2 — hashkinds phase 3 (`mnemonic-engrave`), FOLD VERIFICATION

**Artifact:** branch `hashkinds-p3`, fold commit `c43abb1a` responding to R0
round 1 (`design/agent-reports/plan-hashkinds-P3-R0-round1.md`, 0C/3I/8M).
**Scope, per dispatch:** did the fold close I-1/I-2/I-3/M11, and did it
introduce a new defect. NOT a fresh audit — §4.1-§4.4 of round 1 (fail-closed
parser attacks, byte-identity, KAT constants, orphan check) taken as settled
and not re-derived. Reviewer: independent context, sonnet, verification pass —
prefer executing over reading. Worktree left clean and byte-identical
(`git diff | wc -c` = 0; `git status --porcelain` = 0 lines).

---

## 0. Gate re-run, pinned toolchain (author's claims re-measured, not trusted)

    export PATH=/home/bcg/.cargo/bin:$PATH
    cargo +1.85.0 nextest run --locked --all-targets
      → Summary [32.201s] 657 tests run: 657 passed, 2 skipped
    cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings
      → CLIPPY_EXIT=0, zero warning/error lines
    cargo +1.85.0 fmt --all -- --check
      → FMT_EXIT=0

All three author claims hold exactly (657 passed / 2 skipped / clippy 0 / fmt
clean). Nightly-default trap avoided (`+1.85.0` pinned on every invocation).

---

## 1. Findings CLOSED

### I-1 — producer rule now runs at the wire — CLOSED, verified beyond round 1's own reproduction

Built `origin/master` (`8a47bf43`) in a throwaway worktree and the branch tip
in a separate `CARGO_TARGET_DIR`, then packed **16 payloads** covering every
record class the branch and master share (`key:`, `now:` with and without
height, `phrase:` with both methods, a preimage plate, and mixed multi-record
payloads combining them with a bare `hash:sha256:` record), unsealed via
`--no-passphrase --no-now` for determinism:

    IDENTICAL key_hashbare_now (419 bytes)      IDENTICAL key_hash_plate (470 bytes)
    IDENTICAL key_now (349 bytes)               IDENTICAL key_only (324 bytes)
    IDENTICAL mixed_all_no_hash (427 bytes)     IDENTICAL mixed_all_with_hash (497 bytes)
    IDENTICAL multi_key (597 bytes)             IDENTICAL no_now_flag_key_only (324 bytes)
    IDENTICAL now_height (90 bytes)             IDENTICAL now_only (76 bytes)
    IDENTICAL phrase_and_plate (201 bytes)      IDENTICAL phrase_only (129 bytes)
    IDENTICAL plate_only (127 bytes)

(My first pass, without `--no-passphrase`, showed same-size byte differences
on the phrase/plate cases — that was `pack`'s own auto-generated random
passphrase salt/IV, unrelated to this fold; re-run with `--no-passphrase`
eliminated it. Noted here so a later round doesn't rediscover the same
non-finding.)

This extends round 1's §4.2 (which checked only `key:`/`hash:`/`now:`) to
`phrase:` and preimage-plate records and to mixed multi-record payloads —
confirming the fold's own claim ("only `hash:` records are touched") across
every class that can coexist with a hash record, not just the three checked
before.

`hash256`/`ripemd160`/`hash160` tagged records correctly `exit 4` on master
(unknown record — that grammar doesn't exist pre-phase-3) and `exit 0` on the
branch; this is the feature working as designed, not a mismatch.

**Round-trip, all four kinds plus the explicit→bare case** (`me sysw pack` +
`me sysw show`, branch binary):

    explicit_sha256  input 3cf5d421..b70a4c12 → identity 04dc688a…, digest 3cf5d421..b70a4c12
    bare_sha256      input 3cf5d421..b70a4c12 → identity 04dc688a…, digest 3cf5d421..b70a4c12   (BYTE-IDENTICAL identity to explicit)
    hash256          input 98a20fc2..641cd488 → digest 98a20fc2..641cd488
    ripemd160        input 09e7bb50..bcc2946b → digest 09e7bb50..bcc2946b
    hash160          input b5b72c0e..214cd0bd → digest b5b72c0e..214cd0bd

Every digest survives round-trip intact; nothing displays a digest different
from what the operator gave; explicit and bare sha256 produce the identical
`identity` hash, confirming normalisation collapses them to the same bytes as
claimed.

**Ordering, per the brief's §2.** `split` calls `admit_check` (on the
un-normalised `records`) and computes `now_indices` (also on un-normalised
`records`) *before* the per-record normalisation loop runs — read directly at
`crates/me-cli/src/sysw/mod.rs:549-586`. `classify_with` (used by both
`admit_check` and the loop's secret/public partition) calls
`composer_records::parse` directly, the same function the new normalisation
line calls — confirmed by reading both call sites
(`crates/me-cli/src/sysw/mod.rs:305,341` and `:566`) — so a record `admit_check`
classified `Class::Hash` is exactly the set the loop's `match` arm fires on;
there is no window where the two functions could disagree. A malformed
`hash:` record (wrong width, unknown kind, non-hex) is refused by
`admit_check` before the loop runs at all — confirmed by execution: a 3-record
payload with a malformed `hash:deadbeef` at index 1 exits 4 with `record 1:
hash: sha256 needs exactly 64 lowercase hex characters` and writes **no output
file** (fail-closed, no partial write). Records that fail to parse (any
non-`Hash`, non-parseable body) fall through the match's `_ => r` arm
untouched, which is exactly what the byte-identity results above confirm for
`key:`/`now:`/`phrase:`/plate records.

### I-2 — SPEC §8n re-pinned — CLOSED, verified byte-for-byte against the running binary

`design/SPEC_wallet_policy_composer.md:814-820` now carries:

    > record N: hash: <kind> needs exactly <N> lowercase hex characters
    > record N: hash: unknown hash kind; expected `hash:<hex>` (sha256) or
    > `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase

Ran both refusal paths against the branch binary directly:

    $ echo hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc294 | ...pack...
    record 0: hash: ripemd160 needs exactly 40 lowercase hex characters

    $ echo hash:bogus:3cf5...4c12 | ...pack...
    record 0: hash: unknown hash kind; expected `hash:<hex>` (sha256) or
    `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase

Both match the spec's new pin exactly (kind/width interpolated correctly:
`ripemd160`/`40`). The retired string (`hash: must be exactly 64 hex
characters`) is gone from `--in`-driven pack output.

### I-3 — `--help` grammar and two-axis language — CLOSED, every claim checked against source

`me sysw pack --help` now reads (line-wrapped): *"A `hash:` record's KIND is
which hash the SCRIPT commits to: omit it for `sha256` (64 hex, the form
every shipped device reads), or write `hash:hash256:` (64 hex),
`hash:ripemd160:` or `hash:hash160:` (40 hex). An explicit `hash:sha256:` is
accepted and normalised to the bare form."* plus the two-axis paragraph.

Checked each factual claim against the code, not the prose:

- Widths: `RecordHashKind::digest_len()` (`composer_records.rs:129-134`) —
  `Sha256`/`Hash256` → 32 bytes (64 hex); `Ripemd160`/`Hash160` → 20 bytes (40
  hex). Matches the help text exactly.
- "normalised to the bare form": confirmed by the round-trip test above —
  `explicit_sha256` and `bare_sha256` pack to byte-identical output.
- "the form every shipped device reads": the fork's
  `/scratch/code/shibboleth/seedhammer/sysw/composer_records.go:192-194`
  (`ParseHashRecord`) requires `len(body) == 64` with no colon-tag handling at
  all — so today only the bare sha256 form is understood by any shipped
  device, and the help text does not claim otherwise for the other three
  kinds. Not an overclaim.
- Two-axis claim ("They share the token `sha256`"): `HashlockMethod::as_str`
  (`composer_records.rs:76-81`) returns `"sha256"` for the phrase method, and
  `RecordHashKind::token` (`:139`) returns `"sha256"` for the hash kind — same
  string, different axis, exactly as stated.

**Out of scope, noted so a later round doesn't re-raise it as new:** packing a
`hash256`/`ripemd160`/`hash160` record today produces a payload that is
**also** `ClassUnknown`/inert on every currently shipped device (the fork's
parser only ever accepted 64-char bodies with no tag at all) — the same
failure shape I-1 was about, for the other three kinds. This is not something
the fold introduced or could fix: `design/SPEC_hashlock_kinds.md:188` already
documents *"an old device treats the record as `ClassUnknown` and inert"* as
the deliberate, known state, and the Go device-side port is explicitly phase 4
(`design/SPEC_hashlock_kinds.md:396,416`). The help text does not claim
otherwise. Not a finding.

### M11 — `HashLock::new`'s width guard is now independently tested — CLOSED, mutation-proved

New test `hash_lock_new_refuses_a_digest_of_the_wrong_width`
(`crates/me-cli/tests/sysw_composer_records.rs:730-757`). Mutated
`HashLock::new`'s guard from `(digest.len() == kind.digest_len())` to `(true)`
— file sha confirmed applied (`3299497a… → ebbec680…`, matching round 1's own
M11 hash) — and reran just this test:

    FAIL sysw_composer_records::hash_lock_new_refuses_a_digest_of_the_wrong_width
      panicked: "Sha256: the other width must be refused, not reshaped"

Reverted; file sha back to `3299497a…`.

---

## 2. New defect introduced by the fold

**None found.**

Mutation-proved the fold's other new test the same way. Removed the entire
normalisation block from `split` (`crates/me-cli/src/sysw/mod.rs:566-579`) —
file sha confirmed applied (`4eb4be0d… → 561d75da…`) — and reran
`sysw_hashkinds`:

    FAIL the_producer_rule_reaches_the_wire_not_just_the_helper
      "an explicit sha256 record must normalise to the bare form on the wire"
    PASS every_kind_packs_and_show_names_the_kind_it_packed
    PASS a_genuinely_orphaned_phrase_warns_and_names_what_it_checked
    PASS a_phrase_matching_a_non_sha256_record_does_not_warn

Exactly the claimed pattern: only the new wire-level test catches the
regression, the other three (including the unit-level producer test from
round 1) stay green. Reverted; file sha back to `4eb4be0d…`.

No byte-identity break found for any record class (§1 above, 16 payloads plus
5 round-trip cases). No ordering break (admission and `now:`-count run on
un-normalised text; normalisation and classification agree because both call
`composer_records::parse` on the same string). No malformed-input regression
(fail-closed refusal, unchanged wording modulo the I-2 re-pin, no partial
file). `clippy`/`fmt`/full suite clean at the CI-pinned toolchain.

---

## 3. Counts

| severity | count | ids |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 0 | — |
| Minor | 0 (new) | — |

I-1, I-2, I-3 and M11 are all CLOSED. No new Critical, Important or Minor
defect found in the fold. Round 1's remaining Minors (M-1 through M-8, except
M-3 which was I-1's M11 and is now closed) were not in this fold's claimed
scope and are correspondingly untouched — confirmed no regression in any of
them incidentally (M-8's stale docstring in `sysw_composer_cli.rs` is
unmodified by this commit, as expected; not re-litigated here).

# GREEN — 0 Critical / 0 Important
