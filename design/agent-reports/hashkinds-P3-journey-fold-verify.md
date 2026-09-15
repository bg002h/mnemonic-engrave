# Fold verification — phase 3 journey walk (J-1, J-2), across two repos

**Scope, per brief:** did the fold close each finding, did the fold introduce a
new defect. NOT a fresh audit. Verification preferred execution over reading.

**Under review:**
- `mnemonic-engrave` branch `hashkinds-p3`, fold `35c14b8a` (J-1: `me sysw pack`
  note).
- `mnemonic-secret` branch `hashkinds-p3-caveat`, fold `26dd8d0` (J-2: `ms`
  card's third consumer line).
- Journey walk under response: `design/agent-reports/hashkinds-P3-journey-walk.md`
  (0C/2I/3M, NOT GREEN). Only J-1 and J-2 (Important) are in scope; the three
  Minors (J-3 folded, J-4/J-5 not) are not re-litigated.

Both worktrees are clean and byte-identical to their fold commits — confirmed
by `git status --short` / `git diff --stat` returning nothing, checked
immediately before writing this report.

---

## Build gate — reproduced independently, matches both commits' claims exactly

**mnemonic-engrave, `cargo +1.85.0` (CI's pin; the repo has no
`rust-toolchain.toml` and the rustup default 1.97.0-nightly clippy shows 3
errors that are neither new nor real — also present on `origin/master`, not
gated on here):**

```
cargo +1.85.0 build --locked            -> exit 0
cargo +1.85.0 nextest run --locked      -> 659 tests run: 659 passed, 2 skipped
cargo +1.85.0 clippy --all-targets --locked -- -D warnings  -> exit 0, no output
cargo +1.85.0 fmt --check                -> exit 0
```

Matches the fold commit's claim ("659 tests, clippy 0, fmt clean") exactly.

**mnemonic-secret, pinned toolchain (`rust-toolchain.toml` = 1.85.0,
`cargo --version` confirmed 1.85.0; fmt on `+1.95.0` per project convention):**

```
cargo build --locked                     -> exit 0
cargo nextest run --locked               -> 584 tests run: 584 passed, 11 skipped
cargo clippy --all-targets --locked -- -D warnings  -> exit 0, no output
cargo +1.95.0 fmt --check                -> exit 0
ci/repro/vendor-freshness.sh             -> "OK — vendor/ satisfies Cargo.lock."
```

Matches the fold commit's claim ("584 tests, clippy 0, fmt clean, vendor-freshness OK")
exactly.

---

## J-1 CLOSED — `me sysw pack` announces a kind-tagged hash record's inertness

Built `me` at `35c14b8a`/current tree and exercised `me sysw pack` directly
(not just the two new repo tests), enumerating the cases the brief named:

| case | result |
| --- | --- |
| `hash:hash256:<64hex>` alone | note fires, names `hash256` |
| `hash:ripemd160:<40hex>` alone | note fires, names `ripemd160` |
| `hash:hash160:<40hex>` alone | note fires, names `hash160` |
| bare `hash:<64hex>` (sha256) | **silent** |
| explicit `hash:sha256:<64hex>` | **silent** (normalises to bare on the wire, confirmed by `xxd`) |
| three mixed kinds in one payload (hash160, ripemd160, hash256, fed in that input order) | note fires **once**, `(hash256, ripemd160, hash160)` — deduplicated, ordered by `RecordHashKind`'s derived `Ord` (enum declaration order: Hash256 < Ripemd160 < Hash160), not alphabetically. Deterministic, not a defect. |
| two records of the **same** non-sha256 kind, different digests | note fires **once**, kind listed once — dedup confirmed |
| no `hash:` record at all (a `text:` record only) | **silent** |
| malformed hash record (unknown kind token, or right kind/wrong width) | payload **refused at exit 4 before the note runs** — the note only fires on a successfully-parsed payload, confirmed by its absence in both refusal transcripts |
| with `--pack-preimage` and a matching preimage plate | note still fires (unconditional on the flag, as claimed); preimage-admission warnings print after it |
| without `--pack-preimage`, same payload (record refused as an unadmitted preimage, exit 4) | note **still fires before the refusal** — see Observation below; not a defect |
| `me sysw pack --help` | present, contains no `--json` flag for `pack` (checked; none exists) |

**Actually stripped a tag end to end** (the sharper claim in the note): took
the `hash256` digest `98a20fc2…41cd488`, removed its `hash256:` tag, packed
the bare 64-hex body. Result: accepted silently (exit 0, no note — correctly,
since the bare form is unequivocally read as sha256), and `me sysw show`
reports `public record 0: sha256 hashlock (hash:) — 98a20fc2..641cd488` — i.e.
the payload now silently commits to sha256 semantics for a digest that was
derived as hash256. This is exactly the hazard both new texts describe.

**Claims checked against the fork's shipped parser**
(`/scratch/code/shibboleth/seedhammer` at `main` `0562e81`, same revision the
journey walk used — its correction that the parser lives in the fork checkout,
not the vendored `third_party/seedhammer` submodule, was re-verified: the
submodule pin `713aee2e`/`v1.4.2` has no `sysw` package):

- `sysw.ParseHashRecord` (`sysw/composer_records.go:192`) requires the body
  after `hash:` to be **exactly 64 characters**; `hash:ripemd160:<40hex>` and
  `hash:hash160:<40hex>` have bodies of length 50/49 and `hash:hash256:<64hex>`
  has length 72 — all fail the length check and return `ErrHashRecord`.
- `classifyComposer` (`sysw/composer_records.go:103`) falls through to
  `return ClassUnknown` on that error — read directly, not inferred.
- `composerDoorCounts` (`gui/composer_door.go:45`) counts `ClassUnknown` into
  `inert`, surfaced via `composerCopyNotUnderstood(inert)` — confirms "counts
  the record in the door's 'not understood' total."
- No hashlock path is built from an unclassified record (no `ClassHash` arm
  matched it) — confirms "builds NO hashlock path from it."

Both sentences in the note and the `--help` clause are true, verified against
source and by execution, not by re-reading the journey walk's claim.

**Mutation-proven, both directions, confirmed to apply before trusting the result:**

- *Silence*: commented out the `report_hash_kind_inertness(&recs);` call site
  (`crates/me-cli/src/main.rs:1545`). `cargo +1.85.0 nextest run --test
  sysw_hashkinds` → `a_kind_tagged_record_is_announced_as_inert_on_firmware_without_support`
  **FAILED** ("hash256: pack says nothing about the tag being inert"). Reverted,
  confirmed clean.
- *Over-firing*: removed the `if h.kind() != RecordHashKind::Sha256` guard from
  the filter (`main.rs:2559`), so the note would fire for sha256 too. Same test
  run → `a_bare_sha256_record_gets_no_inertness_note` **FAILED** ("the bare
  form is what every shipped device reads: me: note — …(sha256)…"). Reverted,
  confirmed clean, rebuilt.

**Observation, not a defect.** When a payload is refused for an unrelated
reason after successful parsing (here: a preimage plate present without
`--pack-preimage`, exit 4, no output file written), the J-1 note still prints
before the refusal. This matches the codebase's existing print-then-refuse
convention elsewhere in `pack` (warnings already print ahead of hard failures
on other paths, e.g. the passphrase-strength warning ahead of the sealing
refusal) and is not something this fold introduced or that the brief's
scepticism list treats as in-scope; noted for completeness only.

**stderr-only, and no existing pinned output silently broken (scepticism
point 2).** Grepped both `sysw_pack_preimage.rs` and `sysw_cli.rs` for any
test asserting an exact stderr line count or an indexed line
(`.lines().nth(...)`) — none exists; the only line-count assertion in
`sysw_cli.rs:910` is a wire-capacity byte count unrelated to stderr text.
Directly confirmed the note reaches stderr only, including the harder case
where the **container itself** is written to stdout (`me sysw pack … >
p.bin`, no `--out`): stdout is byte-clean (`MNEMSYSW` magic at offset 0,
`me sysw show` reads it back correctly), and the note appears only on the
stderr stream. No corruption, no accidental mixing.

---

## J-2 CLOSED — `ms hashlock --kind` names the SeedHammer II as the third, unready consumer

Ran `ms hashlock --hashlock-phrase-stdin --kind <k>` for all four kinds plus
no `--kind`, against the built `ms` at `26dd8d0`/current tree:

| kind | third line present? |
| --- | --- |
| `hash256` | yes — `"...and the SeedHammer II needs firmware with hashlock-kind support to read a \`hash256\` record at all. Firmware without it counts the record in the door's \"not understood\" total and builds NO hashlock path from it -- it does not misread the digest, it ignores it. Do not strip the kind tag to make it parse."` |
| `ripemd160` | yes, same shape, kind substituted |
| `hash160` | yes, same shape |
| `sha256` | **absent** |
| no `--kind` (default) | **absent** — grepped the card for `firmware`/`SeedHammer`, no match |

Text is consistent with the J-1 note and with the pre-existing refusal text at
`main.rs:3295` in the sibling repo (same "an untagged record is read as
sha256, and a hash256 digest is also 64 hex" reasoning) — same claim, same
wording pattern, no drift found between the four places it now appears
(`me sysw pack --help`, the new `me` note, the pre-existing `me` `Composer`
error text, and the new `ms` card line). No fifth, stale place found in either
repo's design docs (`SPEC_hashlock_kinds.md` and `mnemonic-secret/design/*.md`
grepped for the old two-consumer framing and for "hashlock-kind support" — no
contradicting text).

**Mutation-proven, both directions:**

- *Silence*: replaced the third `writeln!` block with a no-op comment
  (`crates/ms-cli/src/cmd/hashlock.rs:589-597`). `cargo nextest run --test
  hashlock_kind` → `a_non_sha256_kind_warns_that_md_may_not_accept_the_operand`
  **FAILED** ("hash256: the device is the consumer most likely to be
  unready…"). Reverted, confirmed clean.
- *Over-firing*: inserted a duplicate device-caveat `writeln!`, unconditioned
  on kind (fires for `sha256` too), immediately after the existing gated
  block. Same test **FAILED** on the sha256 negative assertion ("sha256 is the
  form EVERY shipped device reads; a firmware caveat here is the noise that
  teaches operators to skip the real one"). Reverted, confirmed clean.

---

## Process note (not a fold defect)

Mid-verification, a stale `target/debug/me` binary — left over from the
`nextest run` invoked during the me-cli over-firing mutation, which rebuilds
the binary under test but is not itself rebuilt on `git checkout --` — briefly
produced a false positive (the J-1 note appearing to fire on a stripped bare
sha256 record). `cargo +1.85.0 build --locked` after the revert reproduced the
correct, silent result shown above. All `pack()` scenarios reported in the J-1
table above were run against a binary built from clean, unmutated source
(confirmed by build ordering); only the one retracted false alarm used the
stale binary, and it was caught and corrected before being trusted.

---

## Counts and verdict

| severity | count |
| --- | --- |
| Critical | 0 |
| Important | 0 |
| Minor | 0 (J-3 was already folded; not re-litigated) |
| Nit | 0 |

**Both findings closed. No new defect found in either fold.**

**GREEN.**
