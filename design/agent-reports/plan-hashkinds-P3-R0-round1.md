# R0 round 1 — hashkinds phase 3 (`mnemonic-engrave`), adversarial

**Artifact:** branch `hashkinds-p3`, commits `73f3e7d8`, `f300545d`, `acbfcc93`
off `8a47bf43` (= `origin/master`). Reviewed at tip `acbfcc93`.
**Lens:** adversarial correctness + gate integrity. FIRST review of this branch.
**Reviewer:** independent context, opus. Worktree left clean and byte-identical
(`git diff HEAD | wc -c` = 0; `git status --porcelain` = 0 lines).

---

## 0. Gate re-run, pinned toolchain (author's claims re-measured, not trusted)

    export PATH=/home/bcg/.cargo/bin:$PATH
    cargo +1.85.0 nextest run --locked --all-targets
      → Summary [32.185s] 655 tests run: 655 passed, 2 skipped        NEXTEST_EXIT=0
    cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings
      → CLIPPY_EXIT=0, zero warning/error lines
    cargo +1.85.0 fmt --all -- --check                → FMT_EXIT=0
    cargo +1.85.0 metadata --locked --format-version 1 → META_EXIT=0

All three author claims hold exactly (655 passed / 2 skipped / clippy 0 / fmt
clean). The nightly-default trap was avoided as instructed.

---

## 1. Critical

**None.**

The §6 fail-closed guarantee — the funds-safety one — survived every attack I
could construct. Details in §4 below; I consider that question closed and it
should not be re-asked in a later round.

---

## 2. Important

### I-1 — §6's producer rule is implemented only in a function with zero production call sites; `me sysw pack` emits the form §6 says is "never emitted", and that form is inert on the shipped device

`crates/me-cli/src/sysw/composer_records.rs:317-327` (`hash_record`)

`hash_record()` implements §6's producer rule correctly. It has **no production
call site**:

    $ grep -rn "hash_record" crates/*/src/ --include=*.rs
    crates/me-cli/src/sysw/composer_records.rs:317:pub fn hash_record(lock: &HashLock) -> String {
    crates/me-cli/src/sysw/composer_records.rs:333:/// `hash_record` and `now_record` build public data, so a plain `String` is

Every other reference is in `crates/me-cli/tests/sysw_composer_records.rs`. The
real producer of a payload is `me sysw pack`, which stores the operator's record
text **verbatim**. So the accepted-but-never-emitted spelling reaches the wire:

    $ printf 'hash:sha256:3cf5d421...a4c12\n' > explicit.txt
    $ printf 'hash:3cf5d421...a4c12\n'        > bare.txt
    $ me sysw pack --in explicit.txt --out explicit.bin --no-now   # exit 0
    $ me sysw pack --in bare.txt     --out bare.bin     --no-now   # exit 0
    $ ls -l explicit.bin bare.bin
    -rw------- 128 explicit.bin
    -rw------- 121 bare.bin
    $ strings -a explicit.bin | grep -o "hash:[a-z0-9:]*"
    hash:sha256:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12

**What that costs, measured rather than argued.** The shipped fork
(`/scratch/code/shibboleth/seedhammer`, `0562e81`) parses a hash record as:

    // sysw/composer_records.go:192
    func ParseHashRecord(record string) ([32]byte, error) {
        body, ok := strings.CutPrefix(record, HashPrefix)
        if !ok || len(body) != 64 {
            return [32]byte{}, ErrHashRecord

`len("sha256:"+64hex)` is 71, so the record is `ClassUnknown` and **inert** — it
reaches no screen and surfaces only in the door's not-understood count. This is
fail-closed, which is why it is Important and not Critical. But the journey is:
an operator uses a spelling §6 blesses on input, `me` accepts it without a word,
and the hashlock silently never reaches the composer. Nothing between the two
tells them. The wrong outcome here is worse than telling them nothing.

**Why no test catches it.** The new test
`the_producer_rule_is_bare_for_sha256_and_tagged_for_the_rest`
(`tests/sysw_composer_records.rs:632`) exercises the dead function, so it is
green while the shipped binary violates the rule. Mutation M3 confirms the test
is live *for that function* — it is testing the wrong producer, not failing to
test.

Remedy is a plan-level call (normalise on pack / warn / refuse the liberal
spelling at `pack`), and I deliberately do not prescribe one: `pack`'s
verbatim-passthrough is load-bearing for the payload digest the operator
verifies, so "just normalise" is not obviously right.

### I-2 — the §8n refusal line changed in code but its verbatim pin in `SPEC_wallet_policy_composer.md` was not re-pinned; spec §11 named this as two-place work and one place was done

`design/SPEC_wallet_policy_composer.md:814`

SPEC_hashlock_kinds §11 states: *"`composer_records.rs:144` is pinned verbatim by
`SPEC_wallet_policy_composer.md` §8n **and** by `host_line` rows in
`record_class_vectors.json`, so editing that string is re-pin work in two
places."* The branch re-pinned the vectors (13 new rows, fixture SHA moved) and
did not touch §8n, which still reads:

    ### 8n. Host-side record refusals (`me sysw pack` stderr, §6a), one line each
    > record N: key: needs [fingerprint/path]xpub with
    > an origin; a bare xpub is not a key record

    > record N: hash: must be exactly 64 hex characters      ← RETIRED STRING

    > record N: now: must be <seconds>[,<height>] in range

`me` no longer emits that line under any input — verified by exhaustive search of
the branch's refusal surface. §8n also omits both new lines
(`hash: <kind> needs exactly <n> lowercase hex characters`, and the
unknown-kind line). §11 separately lists *"§8 spec rows for every new string |
`SPEC_wallet_policy_composer.md`"* as a gate that must move.

This is the authority the phase-4 Go port reads for host lines, and the tests
that assert the new strings still declare themselves as testing §8n
(`tests/sysw_composer_cli.rs:2`, `:176`;
`tests/sysw_composer_records.rs:2`, `:346`, `:451`). No follow-up defers it
(`grep -n "§8n" design/FOLLOWUPS.md` → no match).

### I-3 — `me sysw pack --help` still documents one kind and one spelling; §11 named it, phase 3 owns it, it did not move

`crates/me-cli/src/main.rs:195,197` — the exact file:line §11 lists as
*"`me-cli`'s hashlock help text"*. Reachable, measured:

    $ me sysw pack --help | grep -n "hash:<"
    13:`key:<hex of "[fingerprint/path]xpub">`, `hash:<64 lowercase hex>` and
    `now:<hex of "<seconds>[,<height>]">` feed the SeedHammer II's Wallet Policy
    composer: a cosigner key for seating, a sha256 hashlock digest, and the pack
    time …

Two problems, both of them the ones this cycle exists to fix:

1. **The grammar is wrong.** `hash:<64 lowercase hex>` is now one of two
   spellings covering one of four kinds. There is no producer verb for a hash
   record (see I-1 — `hash_record` is dead), so this help **is** the producer
   documentation, exactly as the adjacent `phrase:` paragraph says of itself
   (*"No verb emits one, so this help is the producer"*).
2. **It violates §6's two-axis rule in the one document where both axes are
   read together.** The line says *"a sha256 hashlock digest"* (bare kind token)
   four lines above *"the method that derives its preimage — `hardened` or
   `sha256`"* (bare method token). §6: *"where both axes could be read, **both
   are named or neither is**."*

The operator is not stranded — the refusal at `main.rs:3226` does teach
`hash:<kind>:` — but they reach that only by first guessing wrong.

---

## 3. Minor

- **M-1 — `class_name(C::Hash)` still returns `"sha256 hashlock (hash:)"`**
  (`crates/me-cli/src/main.rs:2715`; §11's `main.rs:2685` row, unmoved). It is
  **unreachable today**: its only call site (`main.rs:2432`, `decide_sealing`)
  filters on `.is_secret()` and `Class::Hash` is not secret — confirmed by
  execution (a payload with a `ripemd160` record prints only
  `sealing:  NOT SEALED — no record in this payload is secret material`). Dead
  string, but it is a false label one predicate-change away from the screen.

- **M-2 — `crates/me-cli/src/sysw/record.rs:70`**: *"`hash:` — a 32-byte sha256
  digest for a hashlock"*. False for `ripemd160`/`hash160` (20 bytes) and for
  `hash256`.

- **M-3 — `HashLock::new`'s width guard is not independently pinned.** Mutation
  M11 replaced `(digest.len() == kind.digest_len())` with `(true)`; the mutation
  was verified applied (file sha `3299497a → ebbec680`) and
  `each_kind_takes_its_own_hex_width` +
  `every_case_classifies_as_its_row_says_and_refuses_with_its_line` both still
  **passed**. The rule itself is safe — M13 (both guards removed) is caught by 3
  tests — but `HashLock::new`'s documented contract (*"Returns `None` rather than
  truncating or padding"*) is redundant with `parse_hash`'s check and nothing
  pins it alone. It is a `pub` constructor and the only way to build a `HashLock`
  outside the module; phase 4 ports this type.

- **M-4 — the corpus coverage gate gained 13 rows and 0 required names.**
  `tests/sysw_composer_records.rs:493-533`'s `required` list (*"§6a rule without a
  fixture row"*) still names only the 7 pre-existing `hash-*` rows. None of the
  13 new ones is required, and the two new `hash160`/`hash256` width lines are
  not in the ≥2 `host_line` tally either. Deletion is still caught (M9: removing
  `hash-ripemd160-valid` fails
  `the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest`),
  which is why this is Minor and not Important — but the name gate is the one
  that survives a deliberate `regenerate`, and it did not grow.

- **M-5 — the phase-4 handoff record is stale in two ways.**
  `design/CONTINUITY_hashkinds_2026-09-15.md:24` still says phase 3 is *"not
  started"* at a tip where it is complete, and `:32` names one corpus SHA for
  phase 4 (`0a911f78…`, which is *mnemonic-secret's*, per
  `mnemonic-secret/CHANGELOG.md:20`). Phase 3 moved a **second** one: the fork's
  `sysw/testdata/record_class_vectors.provenance.json:18` still pins
  `3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf` and must go
  to `d6766fdd7308ce28a23ef954f227d8e4e8431f841b64ea336ea9ea9a09b928dd`
  (verified: `sha256sum crates/me-cli/testdata/record_class_vectors.json`). The
  fork's own gate fails loudly when phase 4 runs, so this is a record gap, not a
  latent defect.

- **M-6 — §13.5's `me bundle` half is not filed.** Spec §13.5 says *"Both are
  recorded as follow-ups owned by this cycle's plan, not as silent scope."*
  `me bundle` is in `me-cli` = phase 3. `grep -niE "six-plate|bundle.*byte-identical"
  design/FOLLOWUPS.md` finds no entry for it (F-534…F-538 cover other things).

- **M-7 (SECRET HANDLING — Minor by operator ruling 2026-08-27)** — phase 3
  introduces two unzeroized copies of the 32-byte preimage.
  `crates/me-cli/src/main.rs:2665` and `:2669`:

        Some(Ok(ComposerRecord::Phrase(p))) => Some(*p.method.preimage(p.phrase.as_bytes())),
        Ok((_, ms_codec::Payload::Preimage(x))) => Some(*x),

  `HashlockMethod::preimage` returns `zeroize::Zeroizing<[u8; 32]>`
  (`composer_records.rs:85`) and `ms_codec::Payload::Preimage` holds
  `Zeroizing<[u8; 32]>` (`ms-codec/src/payload.rs:46`). The `*` copies out of the
  wrapper, so `preimage_of`'s `Option<[u8; 32]>` return and the caller's `x`
  (`main.rs:2603`) are plain arrays that are never wiped. Before this branch the
  function returned a **digest** (public), so this is new. The preimage alone
  spends a key-less hashlock path — the warning `me` prints two lines above says
  exactly that. Logged, not gating.

- **M-8 — a test docstring narrates a retired behaviour as current.**
  `tests/sysw_composer_cli.rs:218-222` still states *"`me sysw pack` refuses
  `hash:hash256:<64 hex>` with 'hash: must be exactly 64 hex characters'"*. The
  body's inline comment corrects it, so a reader who stops at the doc comment is
  the only casualty.

---

## 4. What I attacked and could NOT break — closed questions

Recorded so a later round does not re-spend budget here.

### 4.1 §6 fail-closed: no input reads one kind as another

30 constructed inputs through the real binary, all `exit 4`, no panic, no
mis-read. The parser's last-colon split is **not** exploitable: hex contains no
colon, so a body with more than one colon always yields a token containing a
colon, which `from_token` refuses.

| input | verdict |
| --- | --- |
| `hash:sha256:sha256:<64hex>` | REFUSED — unknown hash kind |
| `hash::<64hex>` | REFUSED — unknown hash kind |
| `hash:<64hex>:` | REFUSED — unknown hash kind |
| `hash:<64hex>:<64hex>` | REFUSED — unknown hash kind |
| `hash: sha256:<64hex>` (leading space) | REFUSED |
| `hash:sha256 :<64hex>` / `hash:sha256\t:<64hex>` | REFUSED |
| `hash:sha256: <64hex>` (space in body) | REFUSED — sha256 needs 64 |
| `hash:<64hex> ` (trailing space) | REFUSED |
| `hash:ripemd160:<64hex>` | REFUSED — ripemd160 needs 40 |
| `hash:sha256:<40hex>` / `hash:hash256:<40hex>` | REFUSED — needs 64 |
| `hash:hash160:<64hex>` | REFUSED — needs 40 |
| `hash:SHA256:…` / `hash:RIPEMD160:…` / `hash:Hash160:…` | REFUSED — case not folded |
| `hash:shа256:…` (Cyrillic а) | REFUSED |
| `hash:<4096-char token>:<64hex>` | REFUSED, no DoS |
| `hash:<10000 colons><64hex>` | REFUSED, no DoS |
| `hash:hash160:0x<40hex>` | REFUSED |
| `hash:ripemd160:<38hex>１２` (fullwidth) | REFUSED |
| `hash:ripemd160:<39hex>` / `<41hex>` | REFUSED |

A 20-byte digest cannot be read as sha256 (bare form demands 64 hex) and a
sha256 digest cannot be read as a 20-byte kind (width refused). The same-width
`sha256`/`hash256` pair is separated only by the tag, which is exactly §6's
design, and `main.rs:3226`'s refusal warns against stripping it.

### 4.2 Byte-identity for existing payloads — proven by construction

Built `origin/master`'s `me` in a throwaway worktree at `8a47bf43` and packed an
identical `key:`/`hash:`/`now:` record file with both binaries:

    sha256  cead4d77d8c908036f1f313ae5dd19202cd6b3dc4af5a34bef233dcee51d5630  old.bin
    sha256  cead4d77d8c908036f1f313ae5dd19202cd6b3dc4af5a34bef233dcee51d5630  new.bin
    cmp → BYTE IDENTICAL   (433 bytes)

§6's byte-identity promise holds for the bare form. (The explicit form is I-1,
a different question — it is a *new* payload, not an existing one.)

### 4.3 The KAT constants are genuinely independent, and the dispatch is correctly wired

`tests/sysw_hashkinds.rs:18-21` recomputed from scratch with `python3 hashlib`,
never from the implementation:

    X = pbkdf2_hmac('sha256', b'correct horse battery staple', b'ms-hashlock-v1', 100000, 32)
      = c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
    sha256(X)    = 3cf5d421…b70a4c12   == D_SHA256     ✓
    sha256d(X)   = 98a20fc2…641cd488   == D_HASH256    ✓
    ripemd160(X) = 09e7bb50…bcc2946b   == D_RIPEMD160  ✓
    hash160(X)   = b5b72c0e…0214cd0bd  == D_HASH160    ✓

Cross-checked a second time through a different route: the preimage-plate arm
with `X = 0xab × 32` (derived by bech32-decoding `PLATE`) reports
`ripemd160 5786aabc..c98f43f4`, matching my independent
`ripemd160(0xab×32) = 5786aabcae0e6cd2dfaeca2767dc8996c98f43f4`.

`RecordHashKind::digest_of` (`composer_records.rs:162-170`) calls the four
`ms-codec` functions directly rather than `ms_codec::hashlock::HashKind::digest`.
That is §5-compliant (one local definition per crate, each caller maps its own
kind), and §10's *"each caller's map gets its own row"* is satisfied: the map is
one named function with no inline kind switches at call sites, and all four arms
are exercised end-to-end through the CLI.

### 4.4 The §8.2.3 orphan check — full enumeration, no wrong-direction silence

Run against the real binary, both carrier classes:

| payload | result |
| --- | --- |
| phrase + matching record, each of the 4 kinds | silent ✓ |
| plate + matching record, each of the 4 kinds | silent ✓ |
| phrase + non-matching sha256 | WARNS, `checked: sha256 3cf5d421..b70a4c12` ✓ |
| plate + non-matching ripemd160 | WARNS, `checked: ripemd160 5786aabc..c98f43f4` ✓ |
| phrase + matching sha256 **and** orphan ripemd160 | silent |
| phrase + orphan sha256 **and** matching ripemd160 | silent |
| phrase + two non-matching, different kinds | WARNS, names **both** kinds ✓ |
| phrase, no `hash:` record | the §8.2.3 note, once ✓ |
| two phrases + matching sha256 | silent ✓ |

The any-match semantics (rows 5-6) is **unchanged from `origin/master`**
(`hashes.contains(&digest)` → `.any(|hl| …)`) and is correct for a check whose
subject is the *carrier*: a payload may legitimately hold a co-party's hashlock
with no preimage present, and warning on that would fire on correct payloads. I
found no payload where the branch is silent and master warned.

### 4.5 Mutation-proof of the new and changed tests

Every mutation's application was verified by file sha256 **before** its result
was believed; each was reverted and the file hash checked back to
`3299497a…` / `2844e734…`.

| # | mutation | applied? | caught? |
| --- | --- | --- | --- |
| M1 | `digest_of` Ripemd160 arm → `digest_hash160` | ✓ | ✓ `a_phrase_matching_a_non_sha256_record_does_not_warn` |
| M2 | `digest_of` Hash256 arm → `digest_sha256` (the §3-F4 pair) | ✓ | ✓ same |
| M3 | `hash_record` emits the tagged form for sha256 | ✓ | ✓ 3 tests |
| M4 | unknown token falls back to sha256 | ✓ | ✓ 2 tests |
| M5 | kind token case-folded | ✓ | ✓ 2 tests |
| M6 | orphan check reverts to sha256-only | ✓ | ✓ |
| M7 | `show` label → hardcoded `"sha256"` | ✓ | ✓ `every_kind_packs_and_show_names_the_kind_it_packed` |
| M8 | `show` slice reverts to `&hx[56..]` | ✓ | ✓ (would panic on 40 hex) |
| M9 | delete corpus row `hash-ripemd160-valid` | ✓ | ✓ fixture-SHA test |
| M10 | corrupt a new `host_line` (40 → 64) | ✓ | ✓ 2 tests |
| M11 | `HashLock::new` stops enforcing width | ✓ | **✗ NOT CAUGHT** → M-3 |
| M12 | `parse_hash` width check dropped (new() still guards) | ✓ | ✗ not caught — correct, redundant guard |
| M13 | **both** width guards dropped | ✓ | ✓ 3 tests |

No silent no-op mutations: every "not caught" above is a *verified-applied*
mutation, and M12's pass is the intended defence-in-depth, not a gap.

### 4.6 The git-rev pin

    ms-codec 0.10.0 git+https://github.com/bg002h/mnemonic-secret
             ?rev=9dcd2e0dc0fef57ef94274359181a9f706a10fc6

- `cargo +1.85.0 metadata --locked` resolves (exit 0).
- The rev **is** `mnemonic-secret`'s `origin/master`:
  `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse origin/master`
  → `9dcd2e0dc0fef57ef94274359181a9f706a10fc6`. ✓
- No `ms-codec = "0.9"` remains in any build input (`Cargo.toml`, `Cargo.lock`);
  remaining hits are historical design docs only.
- A fresh clone builds exactly as well as it did before: `mt-codec` was already a
  git-rev dep on a public repo (`Cargo.toml:82`), CI uses `--locked` with no
  `--offline`, and `ripemd 0.1.3` entered the lockfile as expected.

### 4.7 Fixture rows

20 `hash-*` rows, 13 of them new. Every row's class and `host_line` is verified
mechanically by `every_case_classifies_as_its_row_says_and_refuses_with_its_line`
(green) and that test is mutation-proved live (M10). Fixture SHA re-pinned
correctly: `sha256sum crates/me-cli/testdata/record_class_vectors.json` =
`d6766fdd7308ce28a23ef954f227d8e4e8431f841b64ea336ea9ea9a09b928dd` =
`FIXTURE_SHA256`. ✓

### 4.8 Vacuity check on the re-pointed tests

`the_rejected_hash_record_advice_does_not_tell_you_to_strip_the_kind_tag` was
re-pointed because phase 3 made `hash:hash256:<64hex>` legal. The replacement
trigger (`hash:ripemd160:<64hex>`) still refuses, and the test still asserts all
three load-bearing substrings (`DO NOT DELETE IT`, `hash:<kind>:`,
`read as sha256`). **Not vacuous.** `show_prints_each_class_legibly`'s
`"public record 1: sha256 hashlock (hash:)"` assertion is on a *bare* record and
remains correct. No test in the 655 became vacuous that I could find.

---

## 5. Counts

| severity | count | ids |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 3 | I-1, I-2, I-3 |
| Minor | 8 | M-1 … M-8 |

`me sysw pack` emits a spelling that is inert on every shipped device (I-1), and
two of the cycle's own §11 "gates that must move deliberately" did not move
(I-2 §8n, I-3 the help text). None of the three is a wrong-digest path; the
funds-safety guarantee in §6 is sound and I consider §4.1-§4.4 closed.

# NOT GREEN — 0 Critical / 3 Important
