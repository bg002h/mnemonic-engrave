# Phase 2 — adversarial review of the RUST HOST SIDE (`me` CLI, seal seam)

**Scope:** `crates/me-cli/src/**` at `master` / `4d5ef3f`, reviewed against
`design/SPEC_encrypted_payload_delivery.md` and the Go consumer at
`/scratch/code/shibboleth/seedhammer/seal/` (`823499c`).
**Reviewer stance:** read-only. The only write is this file.

**Result: 1 Critical, 4 Important, 5 Minor, 2 Nits.**

Everything below was **measured**, not read off a doc comment. Every negative
check is paired with a positive control, and every control is reported. Two
harnesses were built in `/tmp` (never in a working tree):

- a Go tool that reverses `uf2.rs::to_uf2` and runs the device's real
  `seal.Opener{}.Open` over the result — so "the device refuses this" is the
  device's own code saying so, not my reading of it;
- a Rust `#[global_allocator]` that inspects every block **at `dealloc` time,
  before it is returned**, so "the secret is freed unscrubbed" is an
  observation, not an inference.

A first mutation harness of mine reported 7 false SURVIVEDs because it graded
`cargo test` by grepping stdout instead of checking exit status — the exact
mistake this project's discipline note warns about. It was caught by a
deliberately-broken-build control, rebuilt on exit status, validated in both
directions (`baseline exit=0`, `broken build exit=101`), and every mutation was
re-run. **Only the re-run numbers appear below.**

---

## Critical

### C1 — `me seal` seals a BIP-39 mnemonic whose word separators are not single ASCII spaces; the device then refuses the entire payload

`crates/me-cli/src/seal/mod.rs:249` (`record_or_mnemonic`), with
`crates/me-cli/src/seal/passphrase.rs:27` (`normalise`) and
`crates/me-cli/src/seal/container.rs:59` (`encode_section`).

**Mechanism.** For a secret-section record, `validate_record` fails on a
mnemonic (no bech32 `1` separator), so `record_or_mnemonic` falls through to
`passphrase::is_valid(s)`. That function calls `normalise`, which
`split_whitespace()`-es the input — collapsing *any* Unicode whitespace run to a
single space — and only then parses. So the acceptance decision is made on the
**normalised** form. `encode_section` then emits the record **as supplied**: it
strips only leading/trailing whitespace and rejects only `\n` and `\r`. Interior
tabs, double spaces, NBSP, vertical tabs and ideographic spaces all survive into
the ciphertext verbatim.

The device's `bip39.Parse` (`seedhammer/bip39/bip39.go:268`) splits on
`[]byte(" ")` — a single ASCII space, nothing else. Any other separator makes a
"word" that is not a word, so `Classify` returns `ClassUnknown`,
`permitted(SectionEncrypted, ClassUnknown)` is false, and `AdmitSection` rejects
**the whole payload**.

**Measured, end to end** (host `me seal` → real UF2 → device `seal.Opener.Open`),
using the repo's committed `bacon`×24 fixture — no real seed material:

| separator after word 1 | bytes | `me seal` | device `seal.Open` |
| --- | --- | --- | --- |
| `U+0020` (canonical) | `20` | SEALED(0) | **DEVICE_ACCEPTED: 0 public, 1 secret** ← control |
| `U+0020 U+0020` | `20 20` | SEALED(0) | DEVICE_REFUSED — classification not permitted |
| `U+0009` TAB | `09` | SEALED(0) | DEVICE_REFUSED — classification not permitted |
| `U+00A0` NBSP | `c2 a0` | SEALED(0) | DEVICE_REFUSED — classification not permitted |
| `U+000B` VTAB | `0b` | SEALED(0) | DEVICE_REFUSED — classification not permitted |
| `U+3000` IDEOGRAPHIC | `e3 80 80` | SEALED(0) | DEVICE_REFUSED — classification not permitted |

The canonical row is the positive control: the harness *can* report acceptance,
so the five refusals are real.

**Failure scenario.** An operator pastes a 24-word seed phrase into a file — from
a PDF, a wrapped editor buffer, a phone note, anything that turns one space into
two. `me seal --seal-secret --in seed.txt --out backup.uf2` **exits 0**, prints a
freshly generated 12-word passphrase, and writes the UF2. The operator writes the
passphrase on paper, stores it apart, and destroys the plaintext source — which is
precisely the ceremony the tool is built to support. Later, at the machine, the
device parses the header, takes the twelve words, burns the **~31 s PBKDF2
derivation, and the GCM tag verifies** — the crypto is entirely correct — and then
the record classifier rejects the payload. Per §6.4 that surfaces to the operator
as "payload unreadable", which §2.2 item 4 has explicitly taught them to read as
*someone replaced my payload*. They have an intact backup, a correct passphrase, a
tamper warning, and no way to engrave.

**This is an unmet MUST, and the host is unambiguously the wrong side.** SPEC
§6.4 line 761: *"Every record MUST be the canonical, unbroken string — no interior
spaces, no hyphens, no grouping of any kind."* SPEC line 1215: *"`me seal` MUST
enforce **every** §6.4 constraint at seal time and refuse rather than emit … **A
bundle the device will reject must never leave the host.**"* That sentence
describes this defect exactly.

**Why the guard next door does not cover it.** `record_or_mnemonic` already
carries a hand-written uppercase check (`mod.rs:242-248`) whose comment states the
argument in full: *"without this check an UPPERCASE mnemonic validates and is then
emitted verbatim … the device's case-sensitive parse rejects it and the operator
gets 'payload unreadable' after a ~31 s KDF."* `normalise` lowercases **and**
collapses whitespace; only the first half of that observation was acted on. The
whitespace half is the surviving sibling of a bug already found and fixed.

**Fix shape (Rust-primary, no device change).** §6.4's literal "no interior
spaces" cannot apply to a mnemonic record — a 24-word mnemonic is interior spaces.
The device's operative rule is "exactly single ASCII spaces", so the host rule
should be: accept a mnemonic record only when it is **byte-identical to its own
normalised form**. That is one comparison at `mod.rs:249`, changes no normative
device behaviour, needs no Go convergence port, and is directly testable.

---

## Important

### I2 — the mk1 "pristine BCH" guard is the only thing preventing a second producer→consumer divergence, and no test covers it

`crates/me-cli/src/seal/record.rs:122`.

`validate_record` rejects an `mk1` that `mk_codec` had to BCH-error-correct
(`corrections_applied != 0`). Neutering that condition to `if false` leaves the
**entire suite green** (mutation M4, validated harness). The mutant is not
cosmetic: `mk-codec` corrects up to 4 symbols, while the device does no
correction at all — `seal/record.go:487-490` says so, and I confirmed it rather
than trusting the comment.

**Measured**, flipping the last symbol of the canonical 111-char `mk1` vector:

```
corrupted mk1 → device: REFUSE — classifies as unknown format
pristine  mk1 → device: ADMIT  — md1/mk1 card        ← positive control
```

So with the guard removed, `me seal` would accept a corrupted `mk1`, emit it, and
the device would reject the whole payload — the same class of failure as C1, on a
public wallet card. The guard is load-bearing for cross-implementation agreement
and is currently protected by nothing but the fact that nobody has touched it.
`validate.rs:94-97` has an equivalent rule that *is* tested
(`rejects_corrupted_mk1`), which is likely why the seal-path copy reads as
covered; it is not the same code path.

### I3 — the secret record reaches the allocator unscrubbed: `Payload.secret` is a plain `Vec<String>`

`crates/me-cli/src/seal/mod.rs:20-24`; fed from `main.rs:365`.

`Payload { public: Vec<String>, secret: Vec<String> }` has no `Zeroize`, no
`ZeroizeOnDrop`, and no `Drop`. F-102's fix reads `--in`/stdin into a
`Zeroizing<String>` (`main.rs:336`) — correct — but `main.rs:362-366` then does
`input.split('\n') … .map(str::to_owned)`, minting a **fresh, unwrapped heap
`String` per record**. Those are the buffers that hold the seed. The `Zeroizing`
input is scrubbed; its per-record copies are not. `encode_section` correctly
returns `Zeroizing<String>`, so the *section* is scrubbed — which makes the
remaining gap easy to miss.

**Measured** with a probing global allocator over the real
`mnemonic_engrave::seal::seal()` path, marker = `"bacon bacon bacon bacon"`:

```
CONTROL   (Zeroizing<String>, same marker) : dirty=0   ← probe reports clean correctly
NEG CTRL  (plain String, same marker)      : dirty=1   ← probe can see the marker
REAL      seal() path                      : dirty=1   ← SECRET FREED UNSCRUBBED
```

Both controls behave, so the real-path result is trustworthy. This is
defence-in-depth rather than a direct compromise, but it is the exact property
F-102 was closed on, on the one tool in the constellation whose job is handling
seeds.

### I4 — `passphrase::normalise` returns `Zeroizing<String>` but leaks its intermediates on every call

`crates/me-cli/src/seal/passphrase.rs:27-34`.

```rust
s.split_whitespace().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ")
```

`str::to_lowercase` allocates a fresh `String` **per word** unconditionally — even
when the input is already lowercase — and the `Vec<String>` holding them is
dropped unwrapped. Only the joined result is `Zeroizing`, which makes the
signature read as safe.

**Measured** (same probe, marker `zebrafish`, three-word input):

```
NEG CONTROL (plain String)       : dirty=1     ← probe works
POS CONTROL (Zeroizing<String>)  : dirty=0     ← probe distinguishes correctly
normalise() intermediates        : dirty=3     ← one block per word, LEAKED
```

Re-measured with an **already-lowercase** input — the real passphrase shape —
still `dirty=3`. This is on the passphrase path: `mod.rs:217` calls
`crypto::derive_key(&passphrase::normalise(passphrase), …)` on every seal, and
`is_valid` calls it again for every record checked. Fix is `Zeroizing` on the
per-word buffers, or building the output in one pass.

### I5 — `bip39`'s `zeroize` feature is OFF, so `Mnemonic` is not `ZeroizeOnDrop`

`crates/me-cli/Cargo.toml:` `bip39 = "2.2"`.

`bip39` v2.2.2 derives `Zeroize, ZeroizeOnDrop` on `Mnemonic` **only under
`#[cfg_attr(feature = "zeroize", …)]`** (`bip39-2.2.2/src/lib.rs:178`). Resolved
mechanically:

```
$ cargo tree -p bip39 -e features --offline
bip39 v2.2.2
├── bitcoin_hashes …
├── unicode-normalization …
└── serde …          # no zeroize dependency: the feature is OFF
```

So every `Mnemonic` this crate builds leaves its `words: [u16; MAX_NB_WORDS]` —
the complete word list — on the stack/heap unscrubbed. Reachable with **seed
material**, not just the passphrase: `record_or_mnemonic` (`mod.rs:249`) and
`check_public` (`mod.rs:107`) call `passphrase::is_valid` on every record, and
`generate()` (`passphrase.rs:19`) builds the passphrase mnemonic the same way.
The fix is `bip39 = { version = "2.2", features = ["zeroize"] }` — one line, and
it makes the `Zeroizing` wrappers around it actually mean something.

Related, same file: `generate()` at `passphrase.rs:21` produces the passphrase via
`Mnemonic::to_string()`, whose `Display` impl grows a `String` by doubling —
orphaning unscrubbed partial copies of the passphrase before the final buffer is
moved into `Zeroizing`. This is the same reallocation-orphan defect the Go side
found and fixed in `bip39.Parse` (`bip39.go:269-273`, "measured, a 12-word parse
orphaned copies holding 1, 2, 4 and 8 words"). The Rust side has not had that
pass.

---

## Minor

### m1 — `check_public`'s "a mnemonic is in your public section" diagnosis is untested

`crates/me-cli/src/seal/mod.rs:107`. Deleting the guard leaves the suite green
(mutation M1, validated harness). **Not an admission hole** — the record is still
refused, by `validate_record`'s `NonCanonical` on the first space. But the
operator is then told *"re-run with --group-size 0"* instead of *"record N is
secret material and cannot ride in the public section"*, which is the message the
guard exists to produce and which its own comment (`mod.rs:104-106`) argues for at
length. Diagnosis-only, so Minor — but it is a guard about nearly publishing a
seed in the clear, and nothing holds it in place.

### m2 — a test's mutation note names a guard that no longer exists

`crates/me-cli/src/seal/mod.rs:1005` — the doc comment on
`an_overlong_ms1_in_public_is_reported_as_a_secret_not_as_too_long` says
*"Mutation this pins: delete the `classify(r) == Format::Ms` guard from
`check_public`."* That guard was removed by a later fold (the comment at
`mod.rs:117-124` describes replacing it with error-matching) and the code now
matches on `Err(RecordError::MsTooLong(_))`. The test still kills the *real*
mutant — verified, M2 KILLED — so this is a record defect, not a coverage gap.
Textbook `comments-outlive-their-conditions`.

### m3 — the nonce-reuse guard omits vector G

`crates/me-cli/src/seal/mod.rs:944` (`no_two_vectors_share_a_key_iv_pair`) builds
five `(key, iv)` pairs — A, B, C, D, F — and asserts they are distinct. **Vector G
(salt `abcd…`, iv `1234…`) is not in the list.** No collision exists today (G's
salt is unique), so this is a coverage gap rather than a live defect — but the
test's name claims a property over "no two vectors" that it does not check, and
this is the guard against GCM nonce reuse in our own published test data.

### m4 — `me hash` enforces neither the 1..24 record cap nor a count bound

`crates/me-cli/src/main.rs:471` (`run_hash_cli`). `me seal` enforces §6.4's
`1..=24` total (`mod.rs:199-203`) and the device enforces it in two places
(`container.go:58`, `unlock_key.go:99`). `me hash` enforces neither, so it will
print a confident public-data hash for a record list that can never be sealed or
engraved — for a tool whose whole purpose is producing a number the operator
compares against the device months later.

Coupled to this: `pubhash.rs:31` binds the record count as `records.len() as u8`,
which truncates above 255. Unreachable through `me seal` (capped at 24); reachable
in principle through `me hash`, which has no cap. I could not construct a
256-record input that also passes the card-set decode (see "could not determine"),
so the truncation is theoretical — but the missing cap is not.

### m5 — the AES round-key schedule is never scrubbed, and unlike the Go side this is not written down

`crates/me-cli/src/seal/crypto.rs:46` and `:69`. `derive_key` correctly returns
`Zeroizing<[u8; 32]>`, but `Aes256Gcm::new(key.into())` expands that key into a
~240-byte round-key schedule. Resolved mechanically: **`aes`'s `zeroize` feature
is OFF** (`cargo tree -p aes -e features` → 0 matches for `aes feature "zeroize"`),
so `Aes256` has no scrubbing `Drop` and the key-equivalent schedule outlives the
`Zeroizing` original.

Flagged Minor because it is in-process derived-key material and is RustCrypto's
default. Flagged *at all* because the Go side treats exactly this class as worth a
40-line "HONEST CAVEAT" — `seal/pbkdf2.go:60-84` documents that HMAC's ipad/opad
stay key-equivalent for the life of the `Deriver` and cites FIPS 198-1 §6. The
Rust side has the same residue and no equivalent note, so a future reader will
reasonably assume `Zeroizing<[u8;32]>` is the whole story.

---

## Nits

- **n1 — the public-only path still blocks on stdin.** `me seal --plaintext … --out …`
  with no records to encrypt reads stdin anyway (`main.rs:353`). Measured:
  `sleep 10 | me seal --plaintext … --out …` → **exit 124 (blocked)**; the same
  command with `</dev/null` → exit 0 (control). On a terminal the tool appears to
  hang. Guard the stdin read on `plaintext.is_empty()`.
- **n2 — no `Debug`/error path leaks a secret.** Checked rather than assumed:
  `SealError`, `RecordError`, `ContainerError`, `CryptoError` and `WireError` carry
  only positions, lengths, counts and codec messages — never record bodies. The one
  place a codec error could carry input, `mk_codec::Error::InvalidHrp`, is already
  redacted at `validate.rs:36-38`. `Payload` does **not** derive `Debug`. This is
  clean; recorded so the next reviewer does not re-derive it.

---

## F-120 — the divergence, characterised

F-120 asks for the two accept sets. Measured on both sides with real BIP-93
codex32 secrets generated by the fork's own `biptool` (`head -c N /dev/zero |
biptool seed -seedlen N -id entr`, N = 16..64), plus `mnem`-tagged strings
generated through `ms-codec` 0.7 directly. Host verdict = `me seal --seal-secret`
exit status. Device verdict = `seal.AdmitSection(…, SectionEncrypted)`.

### The headline F-120 understates

**The ledger entry frames this as a boundary case at 90.** It is not a boundary
case. The device admits **27** constructible codex32 lengths in 48–90; `me`
admits **10**, and only with the matching tag. **22 of the 27 diverge.** Verified
in both directions:

```
lengths device ADMITs, host REFUSEs : 48 51 53 54 58 59 61 64 66 67 70
                                      72 74 77 78 80 82 83 85 86 88 90   (22)
lengths host ADMITs, device REFUSEs : (empty)
```

The empty second line is the load-bearing one: **every `ms1` `me seal` will emit
is admitted by the device.** The divergence is strictly one-directional (host
stricter), so F-120 cannot produce an unopenable backup. That is the opposite of
C1, and it is why F-120 is correctly a polish item while C1 is not.

### F-120's stated Rust accept set is misleading

The entry writes the host set as `[50, 56, 62, 69, 75] ∪ [51, 58, 64, 70, 77]`.
Those are **two disjoint tag families, not a union of admissible lengths**:
`VALID_STR_LENGTHS` belongs to the v0.1 `entr` payload and `VALID_MNEM_STR_LENGTHS`
to the v0.2 `mnem` payload (`ms-codec-0.7.0/src/consts.rs:33,43`). A given string
is one or the other. Measured: an **`entr`-tagged** string of length 51/58/64/70/77
is refused by `me` — *"tag "entr" payload length 17 not in expected set
[16, 20, 24, 28, 32]"* — while a **`mnem`-tagged** string of the same length is
admitted. Writing the set as a union invites a future reader to conclude that any
77-character `ms1` seals, which is false.

### Concrete strings and each side's verdict

| len | representative string (real, generated) | tag | `me seal` | device |
| --- | --- | --- | --- | --- |
| 48 | `ms10entrsqqqq…w75sjdmwpnq89` | entr | **REFUSE** — length outside v0.1 set | **ADMIT** |
| 50 | `ms10entrsqqqq…cj9sxraq34v7f` | entr | ADMIT | ADMIT |
| 53 | `ms10entrsqqqq…e0t4k7x3hu9vj` | entr | **REFUSE** — length outside v0.1 set | **ADMIT** |
| 75 | `ms10entrsqqqq…cwugpdxtfme2w` | entr | ADMIT | ADMIT |
| 77 | `ms10entrsqqqq…nhj9cqnp68su2` | entr | **REFUSE** — `entr` payload length 33 | **ADMIT** |
| 77 | `ms10entrsqgqq…scll92pg4ndkd` | mnem | ADMIT | ADMIT |
| 90 | `ms10entrsqqqq…utd7mdh2lc8h2` | entr | **REFUSE** — length outside v0.1 set | **ADMIT** (last engraveable) |
| 91 | `ms10entrsqqqq…2uk6ly9a0dmw4` | entr | REFUSE — §10.2.1a, *too long to engrave* | REFUSE — `ErrCodex32TooLong` |
| 93 | `ms10entrsqqqq…mtf88e60hz9eu` | entr | REFUSE — §10.2.1a | REFUSE — `ErrCodex32TooLong` |
| 125 | `ms10entrsqqqq…t042k235w95p5rd` | entr | REFUSE — §10.2.1a | REFUSE — `ErrCodex32TooLong` |

The 91/93/125 rows are the two sides **agreeing**, each with its own dedicated
error — which is F-113/§10.2.1a working as designed on both halves.

### What F-120's "design call nobody has made" actually costs

The entry lists three options: narrow the device, widen `me`, or document. The
measurement changes the weighting:

- **Narrowing the device to the constellation set is the wrong move** and would be
  a normative regression. §10.2.1 explicitly forbids assuming a conforming `me`
  produced the blob, and the device's `codex32.New` band is BIP-93's, not ours.
  It would also break `-seedlen`-generated fixtures the fork's own tests need.
- **Widening `me` to BIP-93's band** is a real option but is a change to
  **`ms-codec`**, not to `mnemonic-engrave` — the narrowing is `ms_codec::decode`'s
  discrete length set, which `me` merely calls (`record.rs:156`). Under the
  Rust-primary rule that is a constellation-codec decision with its own test
  vectors, well outside this repo.
- **Documenting is therefore the only option that lives here**, and the honest
  documentation is the 22-length table above, not "90 is the boundary".

`record.rs:319-344`'s `does_not_fire_at_the_ninety_character_boundary` already
states this gap in its own text and asserts *not-`MsTooLong`* rather than `Ok`.
That is exactly right and should not be "fixed" into an `Ok` assertion.

---

## Suggested closures / ledger amendments (proposed only — nothing edited)

- **F-120 — do NOT close; amend.** The divergence is real and still open, but the
  entry's characterisation is materially understated (a boundary case at 90 vs. 22
  diverging lengths) and its `∪` notation for the host accept set is wrong (two
  disjoint tag families). Propose replacing the characterisation with the measured
  table above, and adding the newly-verified fact that the divergence is
  **strictly one-directional** — no `ms1` that `me` emits is refused by the device
  — since that is what justifies its "post-merge polish" owning phase rather than
  something more urgent.
- **F-102 — confirm CLOSED, with a sequel.** Verified by reading the shipped code,
  not the entry: `me seal` takes `--in`/stdin into a `Zeroizing` buffer
  (`main.rs:336-356`), argv survives for fixtures, and the seed-on-argv warning
  fires (`main.rs:391-398`). The closure is sound. **However**, finding I3 shows
  the `Zeroizing` buffer's contents are immediately re-copied into unwrapped
  `String`s at `main.rs:365`, so the at-rest exposure F-102 closed is partly
  reintroduced one line later. Propose filing I3 as F-102's sequel rather than
  reopening F-102.
- **No other follow-up in scope was found satisfied.**

---

## What I could not determine, and why

1. **Whether the md1/mk1 codec ports agree across their full input domains.**
   `md-codec` 0.42 / `mk-codec` 0.4 versus the fork's Go `md`/`mk` packages is a
   two-codec differential-fuzz problem. I exercised the committed vectors (which
   pass both sides) and one deliberately corrupted `mk1` (I2). A systematic
   differential over the chunk-header and reassembly space was out of budget, and
   it is the most likely place another C1-shaped divergence is hiding.
2. **Whether the device GUI renders `ErrRecordNotPermitted` to the operator as
   "Payload unreadable".** C1's operator-facing consequence depends on that
   mapping. §6.4 and the `seal` package comments say it does, but the rendering is
   device-side and explicitly out of my scope, so I verified only as far as the
   `seal` package's returned error.
3. **Whether `me hash`'s `as u8` count truncation is reachable.** It needs 256+
   records that each validate *and* form decodable card sets. My 75-record attempt
   died in the card-set decode (`chunk set incomplete: got 75 chunks`) — the
   correct refusal, but it means the truncation path stayed untested. Constructing
   256 genuinely distinct decodable cards was not worth the budget.
4. **Real-world memory residency.** The allocator probe observes buffers **at
   `dealloc`**, which proves the program frees secrets unscrubbed. It says nothing
   about what the OS, the allocator's own reuse, swap, or core dumps subsequently
   do with those pages. Findings I3/I4/I5 are therefore "secret material is
   released unscrubbed", not "secret material was recovered from a running system".
5. **The `pbkdf2`/`sha2` crates' own internal key residue.** I resolved the
   `zeroize` feature state for `bip39` and `aes` because both are directly
   key-bearing. I did not audit `pbkdf2`'s HMAC intermediates for the ipad/opad
   residue the Go side documents at `seal/pbkdf2.go:60-84`; the Rust side almost
   certainly has the same residue, but I did not measure it and am not reporting
   it as a finding.
6. **`bundle.rs`, `preview.rs`, `ndef.rs`, `manifest.rs`.** Outside the seal seam
   and outside my brief. I read them only where the seal path touches them
   (`classify`, `validate`, `write_private`).

---

## Areas checked and found sound (recorded so they are not re-derived)

- **Wire header.** `wire.rs` and `seal/wire.go` agree field-for-field: 52 bytes,
  big-endian, same magic/version/KDF/AEAD ids, same `[100_000, 2_000_000]`
  iteration bounds, same 8191 section cap and its `gui/scan.go` justification, same
  §6.2 anti-downgrade zero-field rule, same `u64` overflow-safe region check, and
  the same normative check *order*. Mutation M10 (little-endian iterations) is
  killed.
- **AEAD construction.** AAD = `header ‖ public section`, taken from the blob's own
  bytes on the device and built once and reused as the output prefix on the host
  (`mod.rs:212-222`) — so the host cannot desynchronise the AAD from what it
  writes. Mutation M6 (AAD = header only) is killed by
  `flipping_a_public_section_byte_fails_the_tag`.
- **Nonce discipline.** Salt and IV are freshly generated per call with no public
  seam to supply them (`mod.rs:70-96`); `seal_deterministic` is `pub(crate)`.
  One fresh salt per message makes the one-key-one-message property structural.
  Mutation-checked by `two_seals_of_the_same_payload_differ_everywhere`.
- **KDF parameterisation.** `iterations` always comes from the header on both
  sides. Vector B (identical to A but for the count) exists solely to catch a
  hardcoded value; mutation M5 confirms it works.
- **§6.6 public-data hash.** Byte-identical construction in `pubhash.rs:26` and
  `pubhash.go:51`, including the `sealed` downgrade byte and the public-only record
  count. Mutation M7 (drop the count byte) is killed. Host and device both hash the
  **trimmed** form (`main.rs:440`, `run_hash_cli:477`).
- **Container encoding.** LF-joined, no trailing LF, CR rejected before trimming
  (so CRLF is refused rather than normalised), 1..512 bytes per record, 1..24
  across both sections. Mutations M11 (cap) and M13 (CR) are both killed.
- **Test suite quality.** 11 of 13 targeted mutations killed by the existing tests,
  including every one on the crypto seam. Several tests carry accurate, specific
  notes about the mutant they exist to kill and the weaker assertion they replaced
  (`passphrase_line`'s vacuity note in `seal_cli.rs:13-22` is a good example). The
  two survivors are reported above as I2 and m1.
- **Error surfaces.** No secret reaches any `Display`/`Debug` impl; `Payload` has
  no `Debug`; `me seal` writes nothing to stdout at all, and that is asserted
  rather than assumed (`seal_cli.rs:44`).

---

*Harnesses used for this review live under the session scratchpad and touched no
working tree: a Go UF2→`seal.Open` bridge, a Go single-record `AdmitSection`
probe, a `ms-codec` `mnem`-tag generator, a Rust probing global allocator, and an
exit-status-graded mutation runner restoring from the pristine source each round.*
