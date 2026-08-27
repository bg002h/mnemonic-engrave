> # RETIRED 2026-08-26 — operator ruling. DO NOT BUILD FROM THIS DOCUMENT.
>
> *"I don't think encrypted_payload_deviceB is relevant any more."*
>
> **The post-wipe hang is known and deliberately accepted, not overlooked.**
> `full unlock → wipe → re-enter Sealed Payload` hangs deterministically on real
> hardware (`HARDWARE_RESULT_2026-08-09_phaseB2b.md`), and the phase's commits
> ARE merged into the fork's `main`, so the behaviour is present in flashed
> firmware. It was raised on 2026-08-26 and the operator ruled it closed the same
> day, on the reasoning that **if it is real it will happen again** — a
> deterministic hang does not hide, and it will return with a live
> reproduction attached rather than as an entry in a retired plan.
>
> **Do not re-open it as a new discovery** — the point of this note is that
> the next reader finds a decision rather than a defect.
>
> `DESIGN_b2b_payload_read_allocation.md` remains as the diagnosis
> (`XIPReader.Read`'s 64 KiB allocation, `sysw/read_tinygo.go:31`) should the
> feature ever be revived.

# Encrypted Payload Delivery — Plan B Phase A (device, headless core) — Implementation Plan

> **For agentic workers:** one implementer, own worktree, TDD, tasks in order.

**Goal:** A Go package in the SeedHammer fork that reads a sealed payload, parses
and bound-checks its header, splits and bounds its sections, allow-lists and
card-set-decodes its records, computes the §6.6 public-data hash, and decrypts the
encrypted section — all **headless and host-testable**, binding byte-for-byte to
the vectors Plan A emits.

**Non-goal:** every pixel. Menu entry, unlock UI, passphrase entry, progress
indicator, plate list, session lifecycle (§10.2.2) and idle wipe (§10.2.4) are
**Phase B**. This plan deliberately stops at the seam where a `go test` on the
host stops being able to prove anything.

**Repo:** `/scratch/code/shibboleth/seedhammer` (the fork). **Not** upstream.

**Spec:** `design/SPEC_encrypted_payload_delivery.md` in `mnemonic-engrave`.
Read **§6, §7, §10.1, §10.2 steps 1–2 and 8–9, §10.2.1 and §11.2** before starting.

**Rust-primary rule.** `mnemonic-engrave`'s `crates/me-cli/src/seal/` is
normative. This is a **behaviour-faithful port**, not a transliteration. Any
disagreement is a Go bug until proven otherwise, and a change to normative
behaviour lands in Rust first, with a vector. Never the reverse.

---

## Global Constraints

- **All Go work runs under `nix develop --command …`.** `nix` is NOT on `PATH` —
  use `/nix/var/nix/profiles/default/bin/nix`.
- **`go.mod` says `go 1.25.10`; TinyGo is 0.41.1** (measured, not assumed). The
  host `go` inside the dev shell is 1.26.3 and is NOT the firmware compiler — a
  feature that builds on the host can still fail under TinyGo.
- **Never a bare `go test ./... -update`.** Scope with `-run`, then `git status`.
- **Two commands do not build under host `go`** — `cmd/kdfbench` and
  `cmd/sealread` (Task 7 Step 4's on-target harness). Both import `machine`,
  which is not in the host standard library, so `go build`/`go vet`/`go test`
  report `package machine is not in std [setup failed]` for each. A
  `go test ./...` reporting exactly **those two** failures is GREEN; anything
  else is a regression. This baseline was ONE failure until `cmd/sealread`
  landed — if you add another TinyGo-only command, update this line, or the
  next person treats a real regression as expected noise.
- **`crypto/aes` and `crypto/cipher` are ABSENT from today's firmware build** —
  measured. Importing them is what makes AES-GCM callable and costs ~1.6 KB.
  `crypto/pbkdf2` and `crypto/sha256` are already linked *and already called*.
- **Never widen `seal_deterministic`'s visibility in the Rust crate.** It is
  `pub(crate)` deliberately (`crates/me-cli/src/seal/mod.rs:145-150`): a public
  version destroys the one-key-one-message property the moment a caller reuses a
  salt. If Task 1 seems to need it public, Task 1 is being done wrong.
- Stage paths explicitly. Never `git add -A`.
- `gofmt` clean before every commit; `go vet ./<touched>/...` clean.

---

## The vector contract — Task 1 exists because of this

Phase A's whole claim to correctness is that it reproduces Plan A's bytes. That
requires the vectors to cross the language boundary **as data**, not as numbers
retyped into a Go test. Retyped constants are how a port silently forks.

**What Plan A actually pins today**, measured rather than assumed — the exporter
emits more than this, and the difference matters:

| Field | Pinned in Rust? |
| --- | --- |
| blob length | A, C, D, E, F, G (**not** B) |
| blob sha256 | all seven |
| §6.6 hash | D, E (`pubhash.rs:69-80`) and G (`mod.rs:414-418`) only |
| derived key | salt `beef` at 100 000 and 100 001 only (`crypto.rs:86-102`) |
| header hex | **nothing** |
| GCM tag | **nothing** |

So Task 1 Step 2's cross-check can only confirm `blob_sha256` and the D/E/G
hashes. Every other emitted field is exporter output that nothing independently
verifies — which is exactly why the exporter must run the normative code rather
than reconstruct anything.

---

### Task 1: Export the canonical vectors from Rust, import them into Go

**Files:** modify `crates/me-cli/src/seal/mod.rs` (in `mnemonic-engrave`);
create `seal/testdata/vectors.json` + `seal/testdata/README.md` (in the fork)

**Why an exporter and not a hand-copied file:** the vectors must be regenerable
when the spec moves, and the regeneration must run the *normative* code. A JSON
blob nobody can rebuild is a fossil, and the first time it disagrees with Rust the
temptation is to edit the JSON.

- [ ] **Step 1: Write the exporter INSIDE the existing test module**

**Not** in `tests/`. An integration test under `tests/` is a separate crate and
sees only `pub` items — `seal_deterministic` is `pub(crate)` (`mod.rs:150`) and
every fixture (`PASS`, `bacon24()`, `bip84()`, `two_of_three()`, `salt()`, `iv()`)
lives inside `#[cfg(test)] mod tests` (`mod.rs:241`, `:244`, `:253`, `:262`,
`:808`). An exporter in `tests/emit_vectors.rs` **does not compile**, and the
shortest fix — making the seam `pub` — is the security regression the Global
Constraints forbid.

Add `#[test] fn emit_vectors()` to `src/seal/mod.rs`'s existing `mod tests`. It
therefore reuses `bip84()` and `two_of_three()` directly, so the *inputs* cannot
drift from the vectors either.

Behaviour: when `ME_EMIT_VECTORS=1`, write the JSON; otherwise **assert the
committed file still matches**, so drift in either direction fails CI.

Regenerate with:
```
ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors
```

Shape (one object per vector):

```json
{
  "note": "Generated by `ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors`. Normative source: crates/me-cli/src/seal/. Do not hand-edit.",
  "spec": "SPEC_encrypted_payload_delivery.md",
  "vectors": [
    {
      "name": "A",
      "passphrase": "beef beef … beef",
      "iterations": 100000,
      "salt_hex": "beefbeef…",
      "iv_hex": "bac0bac0…",
      "public": [],
      "secret": ["bacon bacon … bacon"],
      "pub_len": 0,
      "ct_len": 227,
      "blob_hex": "…",
      "blob_sha256": "6707c20e…",
      "header_hex": "4d4e454d424c4f42…",
      "derived_key_hex": "615ad9b7…",
      "tag_hex": "…",
      "pubhash_sealed": null,
      "pubhash_unsealed": null
    }
  ]
}
```

- **`pub_len` and `ct_len` are REQUIRED fields** and MUST be emitted from the
  `Header` struct the exporter constructed — **never** re-read from `header_hex`
  or `blob_hex`. Task 2's binding test asserts the parser against them, and a
  value recovered from the same bytes the parser just read cannot fail.
- `pubhash_*` are `null` when `pub_len == 0` (§10.2 step 3 displays nothing then).
- **Vector E is emitted via `seal_public_only`, not `seal_deterministic`**
  (`mod.rs:363-384` calls `seal_public_only(all[1..].to_vec())`, public API at
  `mod.rs:122`). Its `passphrase` is `null` and its `iterations`/`salt_hex`/
  `iv_hex` are zero — it is the unsealed shape and there is no key.

- [ ] **Step 2: Verify the exporter against what is already pinned**

Run `cargo test -p mnemonic-engrave`, confirm still green, then diff the emitted
`blob_sha256` fields against the literals in `src/seal/mod.rs` and the D/E/G
`pubhash_*` against `pubhash.rs`/`mod.rs`. **They must match without editing
either.** Per the table above, that is the whole of what this step can check.

- [ ] **Step 3: Commit the JSON into the fork with a provenance line**

Copy to `seal/testdata/vectors.json`. `seal/testdata/README.md` names the
generating command, the `mnemonic-engrave` commit it came from, and the rule that
it is **never hand-edited**.

**Do not update `md/bits.go`'s `0.36.0` provenance pin** while doing this. That
pin covers the whole `md` package and nothing here verifies the package
converged; bumping it would assert a sync that did not happen.

---

### Task 2: `seal/wire.go` — header parse and every §6.2 bound

**Files:** create `seal/wire.go`, `seal/wire_test.go`

Port of `crates/me-cli/src/seal/wire.rs`. 52 bytes, big-endian, both shapes.

**This is hostile input by construction** — it is parsed *before* anything
authenticates it, and it carries the iteration count. §6.2's bounds are what stop
an unbounded KDF, and the firmware has **no active watchdog**: an unchecked
iteration count is a hang, not a slow screen.

- [ ] **Step 1: Write the failing test first**

Mirror `wire.rs`'s eight tests — both shapes round-trip; sealed shape sets the
algorithm ids and unsealed zeroes them; bad magic/version/reserved/kdf/aead
refused; short buffer refused; out-of-range iterations refused; out-of-range
lengths refused; empty payload refused; **non-zero crypto fields when unsealed
refused**.

Plus the test that actually binds the port: **decode every `header_hex` in
`vectors.json` and assert the parsed fields equal that vector's declared
`iterations` / `salt_hex` / `iv_hex` / `pub_len` / `ct_len`** — reading those
expectations from the **JSON's own fields**, never from the header bytes.

- [ ] **Step 2: Run it, confirm it fails for the right reason**

Expect `undefined: ParseHeader`. **A run reporting `no test files` or `0 tests`
is a FALSE RED** — the package is not wired up. Fix that before proceeding.

- [ ] **Step 3: Implement**

Constants exactly as §6.2 and `wire.rs`: `MAGIC = "MNEMBLOB"`, `VERSION = 0x01`,
`KDFPBKDF2SHA256 = 0x01`, `AEADAES256GCM = 0x01` (`wire.rs:9-10`),
`HeaderLen = 52`, `SaltLen = 16`, `IVLen = 12`, `TagLen = 16`,
`MinIterations = 100_000`, `MaxIterations = 2_000_000`, `MaxSectionLen = 8191`,
`RegionLen = 65_536`, plus `MaxRecords = 24` and `MaxRecordLen = 512`
(`container.rs:10-11`) which Task 5 consumes.

Order of checks is normative and must match Rust: length → magic → version →
reserved → read `iterations`/`pub_len`/`ct_len` → section caps → empty → then the
sealed/unsealed split. **Every bound is checked before any KDF work** — the test
proving that lives in Task 8, because nothing at this layer can observe it.

Widen to `uint64` for the total-size check, as Rust does — 32-bit arithmetic
wraps for lengths near 2^32 and would pass a `<= 65536` test. **Note this check is
unreachable** behind the section caps (max total = 52+8191+8191+16 = 16 450 <
65 536), Rust has no test for it either (`wire.rs:161-165`), and it is
deliberately excluded from the mutation table. Keep it as defence in depth
against a future implementation that drops the caps; do not go looking for the
test that kills it.

- [ ] **Step 4: Commit** — `gofmt`, `go vet ./seal/...`, `go test ./seal/`

---

### Task 3: `seal/pubhash.go` — the §6.6 fixed public-data hash

**Files:** create `seal/pubhash.go`, `seal/pubhash_test.go`

`SHA-256(LABEL ‖ 0x00 ‖ sealed ‖ public_record_count ‖ records joined by LF)[:16]`,
`LABEL = "MNEMBLOB/pub/v1"`.

- [ ] **Step 1: Failing test, driven by the JSON**

Assert `pubhash_sealed` and `pubhash_unsealed` from `vectors.json` for every
vector that has them (D, E, G). Then assert the two **differ** — that inequality
is the downgrade detector, and an earlier spec draft required them to AGREE,
which was exactly the blindness a ciphertext-strip needs.

Also port Rust's `every_byte_of_the_section_affects_the_hash`: mutate the
section's true first and last byte and assert the digest moves. The D≠E
inequality alone does not kill a subset-of-the-input mutant.

- [ ] **Step 2: Run, confirm the right failure. Step 3: Implement.**

`public_record_count` is the count of records in the **public section only** —
not §6.4's `1..24` cap, which counts both. Vector D is 5 public of 6 total and
the two produce different digests.

- [ ] **Step 4: Commit**

---

### Task 4: `seal/crypto.go` — PBKDF2 and AES-256-GCM open

**Files:** create `seal/crypto.go`, `seal/crypto_test.go`

The device only ever **opens**. Do not port `seal_bytes` — a device that can
seal is a device that can be made to emit a payload, and nothing in §10 needs it.

- [ ] **Step 1: Failing test**

- `DeriveKey(passphrase, salt, iterations)` equals `derived_key_hex` for every
  vector. Vector **B differs from A only in iteration count** — it is the only
  test that catches a hardcoded one.
- `Open` round-trips every encrypted vector to its **exact records**, split on LF.
  Pinning the blob hash alone proves the bytes are stable, not parseable.
- `Open` **fails** on a flipped ciphertext byte, on a tampered AAD, and — the one
  that proves §6.1a — on **a flipped byte of the public section**, since the
  public section is inside the AAD.
- `Open` fails when `iterations` is altered in the header from 100000 to
  **100002** (not 50000 — that is refused by §6.2's floor before any tag work and
  proves nothing about the AAD).

- [ ] **Step 2: Run. Step 3: Implement.**

`crypto/pbkdf2` (stdlib, Go 1.24+) — already linked and already called by
bip39/slip39. `crypto/aes` + `crypto/cipher` for GCM; **these are new imports to
the firmware** and are what the measured ~1.6 KB buys.

`AAD = header ‖ public section` = `blob[:52+pub_len]`, and the ciphertext is
`blob[52+pub_len:]` including the trailing 16-byte tag. Go's
`gcm.Open(dst, nonce, ciphertext, aad)` expects the tag appended to the
ciphertext, which is exactly the wire layout — do not slice it off.

**Fail closed.** Return no plaintext on tag mismatch, ever. Go's `Open` already
does; do not add a path that inspects a partial result.

- [ ] **Step 4: Commit**

---

### Task 5: `seal/container.go` — the §6.4 section decode

**Files:** create `seal/container.go`, `seal/container_test.go`

Port of `crates/me-cli/src/seal/container.rs`, plus the device-only pre-split
rule. **§6.4 is NORMATIVE and this task is where it lives** — an earlier draft of
this plan had Task 8 say only "split on LF", which is precisely the naive
implementation §6.4 forbids.

- [ ] **Step 1: Failing test**

`SplitSection(b []byte) (recs []string, n int, err error)` — **three returns,
deliberately.** The record count comes back *out of band* so the error itself can
be a package-level preallocated sentinel; see the allocation note below for why
that is forced rather than chosen.

- **Count the `0x0A` separators and reject `> MaxRecords-1` BEFORE splitting.**
  A plaintext of 8191 LF bytes satisfies `ct_len <= 8191`; an implementation that
  splits first materialises ~8192 slice headers (~98 KB on a 32-bit target), a
  fifth of the free heap, transiently. With `ct_len == 0` this is reachable with
  **no passphrase and no KDF at all**.
- Per record: `1..=MaxRecordLen` (512) bytes, non-empty.
- Reject any `0x0D` anywhere — CRLF is **rejected, not tolerated** (§6.4).
- Reject a leading or trailing LF.
- The `1..=24` cap is over the **TOTAL across both sections**, so the caller
  checks `len(public)+len(secret)`; `SplitSection` cannot see both.
- The too-many-records error MUST be **distinguishable from "payload
  unreadable"**. `SplitSection` returns the preallocated sentinel
  `ErrTooManyRecords` plus `n`; **the caller (Task 8) composes the message that
  names the count and the cap**, which is also the only place the cross-section
  total is known.

**`TestOverlongSectionRejectsBeforeSplitting` must assert
`testing.AllocsPerRun(...) == 0`** — the number, not a comment saying "O(1)". A
return-value assertion here is a **guaranteed false PASS**: `bytes.Split`
performs exactly one allocation and a correct `bytes.Count`-style scan performs
zero, and both return the same error. Both are "O(1)".

**This is why the error must be a preallocated sentinel, and why the count is a
separate return value.** Measured, Go 1.26.3 in the fork's dev shell, 8191 LF
bytes, `testing.AllocsPerRun(100, ...)`:

| error shape | correct | split-first mutant |
| --- | --- | --- |
| preallocated sentinel | **0** | **1** |
| `&CountError{n, max}` | 1 | 2 |
| `fmt.Errorf("… %d … %d", n, cap)` | 3 | 4 |

Only the sentinel makes `== 0` a true discriminator. Under the struct shape a
**correct** implementation reports 1 — the same value the sentinel **mutant**
produces — so the assertion would fail on correct code and the two available
exits are both wrong: drop the count (violates §11.2's naming rule) or relax the
threshold, which is exactly the false PASS above. **If you change the error
shape you MUST re-measure both columns and confirm the mutant is exactly +1.**

- [ ] **Step 2: Run. Step 3: Implement. Step 4: Commit.**

---

### Task 6: `seal/record.go` — the §10.2.1 allow-list and the §6.3 card-set decode

**Files:** create `seal/record.go`, `seal/record_test.go`

This is the task that stops a seed reaching steel in the clear. It is the one to
slow down on.

**Three passes, distinct, none substituting for another:**
1. **case check**, once per record (§6.4)
2. **allow-list**, once per record (§10.2.1)
3. **decode**, once per card group (§6.3)

- [ ] **Step 1: Failing test**

**The lowercase check runs BEFORE classification and binds BOTH sections.**
Mirrors `record.rs:64-70`. This is not cosmetic: measured against the fork,
`ValidMD` returns **true** for a fully uppercased md1 (`lower=true, upper=true,
mixed=false`) — by design, and the device's own keyboard-entry path emits
uppercase (`gui/codex32_input_test.go:62`). Without this check an uppercase
record is admitted, engraved verbatim, and displayed with a §6.6 hash the
operator's recorded value **cannot** match on an untampered payload — teaching
them that mismatches are normal, which disarms the single control §6.6 provides.

The allow-list must be an **allow-list, not a deny-list** — a deny-list silently
admits whatever branch `gui/scan.go`'s classifier grows next. Verified branches
of `Scan` (`gui/scan.go:28-81`), in order: `debugCommand`, `bip39.Mnemonic`,
`*bip380.Descriptor`, `codex32.String`, `mdmkText`, `addressText`, error.

| Section | Permitted |
| --- | --- |
| public | `mdmkText` only, **and** every card group must reassemble and decode |
| encrypted | `mdmkText`, a codex32 secret (`ms1`), or a parsed BIP-39 mnemonic |

Tests that must exist, each named for what it kills:

- **`TestRefusesAnUppercaseRecord`** — `strings.ToUpper` of a vector record, in
  each section. Kills "lowercase check removed".
- **`TestPublicSectionRefusesDebugCommand`** — `command: lock-boot` at **index 2
  of a 6-record section**, not index 0. At index 0 the test passes under a loop
  that validates `records[0]` and trusts the rest. The test MUST **also assert
  the returned record slice is empty/nil** — Phase A's stand-in for §11.2's
  "nothing was engraved". This is the irreversible branch:
  `Platform.LockBoot()` (`cmd/controller/platform_sh2.go:545`) does
  `writeOTPValues()` → `otp.EnableSecureBoot()` → `machine.CPUReset()`, reached
  from `gui/gui.go:1672`, with the `command: ` prefix (`gui/scan.go:57`) as the
  only gate.
- **`TestPublicSectionRefusesAddressAndDescriptor`** — the other two extra
  classifier branches.
- **`TestPublicSectionRefusesASecret`** — an `ms1` in the public section rejects
  the whole payload (§6.3).
- **`TestDecodesACompleteCardSet`** — the full md1+mk1 set decodes. A per-record
  decode would reject every legitimate payload.
- **`TestRefusesAnIncompleteCardSet`** — one md1 chunk of three; one mk1 of two;
  two md1 of three.
- **`TestRefusesBCHValidButUndecodable`** — the smuggling case. `ValidMD`/`ValidMK`
  are **pure BCH verifiers that never decode**, and the fork ships the checksum
  generator, so arbitrary bytes wrap into a record that classifies as `mdmkText`.
  Without the decode step, seed entropy rides in the cleartext section where
  `picotool save` reaches it with no passphrase.
- **`TestGroupsByHRPAndChunkSetID`** — use **vector G**, whose public section is
  four cards: one `md1` chunked six ways plus three `mk1` cards of two chunks
  each. Grouping by HRP alone gives
  `received 6 chunks, header declares total_chunks = 2` and rejects **every
  multisig wallet**. D and E carry one card per HRP and F is `pub_len = 0`, so
  **nothing but G catches this.**
- **`TestDecodesTwoDistinctNonChunkedCards`** — two single-string md1 cards must
  NOT collide into one group, and each must route to `md.Decode`. Vectors D, E
  and G are all chunked, so without this the mutant "route every group to
  `md.Reassemble`, drop the `md.Decode` arm" survives the entire suite.
- **`TestRefusesAnOverlongRecord`** — a >93-symbol md1 codeword, so F-67's cap is
  load-bearing at this layer and not only inside `codex32`.
- **`TestLeftoverRecordRejects`** — a record belonging to no complete group.

- [ ] **Step 2: Run. Step 3: Implement.**

**The grouping key is `(hrp, chunked, csid, uniq)` — `chunked` comes from
`h.Chunked`, NOT from `ChunkSetID`.** `ChunkSetID == 0` is returned for a
non-chunked record *and* is a legal value for a chunked one
(`md/chunk.go:195` and `md/chunk.go:66`; `mk/mk.go:75-76`), so keying on csid
conflates the two in both directions: unrelated single-string cards collide, and
a legitimate chunked card with csid 0 is split into failing singletons. Rust
avoids this with `Option<u32>` (`record.rs:113`, `:165`). Set `uniq = i+1` when
`!h.Chunked`, else 0.

Both parsers return **two values** — `h, err := md.ParseChunkHeader(s)` and
`h, err := mk.ParseHeader(s)` (`md/chunk.go:185`, `mk/mk.go:56`). The live device
sites branch on `.Chunked` exactly as specified here: `gui/md1_gather.go:38`
`if !h.Chunked {`, and `gui/mk1_inspect.go:65`.

Dispatch per group, mirroring `record.rs::decode_public_set`:

| group | chunked | call |
| --- | --- | --- |
| `md` | yes | `md.Reassemble(set)` (`md/chunk.go:207`) |
| `md` | no | `md.Decode(set[0])` (`md/md.go:1231`) — refuses chunked input by design |
| `mk` | either | `mk.Decode(set)` (`mk/mk.go:148`) |

**Two Rust checks are deliberately not ported, because Go satisfies them by
construction** — verified, so the omission reads as intentional rather than
forgotten. `record.rs:78-84` rejects BCH-*corrected* mk1; `codex32.ValidMK` and
`mk.Decode` do no correction at all. `record.rs`'s `first_noncanonical` rejects
interior spaces and hyphens; the codex32 engine's `inputChar` has no mapping for
`0x20` or `-`, so `ValidMD`/`ValidMK` already return false.

**F-67 is already fixed** (`codex32/mdmk.go`, commit `4192458`). Do not
re-litigate it.

- [ ] **Step 4: Commit**

---

### Task 7: `seal/read.go` — the XIP read at `0x10E00000`

**Files:** create `seal/read.go` (untagged), `seal/read_host.go`
(`//go:build !tinygo`), `seal/read_tinygo.go` (`//go:build tinygo`),
`seal/read_test.go`

**This is the riskiest task in Phase A and the one with no precedent.** Verified:
there is **no existing "read N bytes from a fixed XIP address" anywhere in this
repo.** Every `unsafe.Pointer`/`unsafe.Slice` site takes the address of an
already-typed peripheral register struct field (`driver/dma/dma_rp2.go:70` is the
nearest analogue); `driver/otp/otp_rp2350.go` is a cgo bootrom *call*. The spec's
own citation of `otp_rp2350.go:13` is **stale** — that line is a `#define`.

Which form TinyGo 0.41.1 compiles correctly is an **implementation-time question
settled by a test on hardware**, not a design question (§10.1 says so).

- [ ] **Step 1: Split host from target, and keep the bound testable**

Interface `Reader` with `Read() ([]byte, error)`, plus an exported sentinel
`ErrNoPayload = errors.New("seal: no payload present")` that Phase B matches with
`errors.Is`. **Not `(nil, nil)`** — that is the shape callers forget to check.
Host implementation reads a file (so `go test` drives the whole pipeline from
`vectors.json`); the `//go:build tinygo` implementation does the real XIP read.
**`read_host.go` must carry `//go:build !tinygo` explicitly** — Go derives
implicit constraints only from `_GOOS`/`_GOARCH` filename suffixes, and `_host`
is neither, so without the tag the firmware build compiles both bodies and fails
on redeclaration. Host `go test` would stay green and the break would surface
only at Step 4.

**The `RegionLen` bound lives in an UNTAGGED helper** — `func clampRegion(n int)
int` in `seal/read.go`, called by both implementations — so a host test can kill
the unbounded-read mutant. A bound placed only inside `read_tinygo.go` is never
compiled by `go test` and no automated test can reach it.

**No package outside these files may reference an absolute address.**

- [ ] **Step 2: Detection** — read 8 bytes at `0x10E00000`, compare to
  `MNEMBLOB`. Absent → `ErrNoPayload`, which Phase B must distinguish from
  "present but corrupt".

- [ ] **Step 3: Bound the read** — never more than `RegionLen` (65 536), via
  `clampRegion`. The header's own lengths are attacker-controlled and are checked
  by Task 2, but this read happens **before** that, so it is bounded by the
  constant alone.

- [ ] **Step 4: Verify on the Pico 2, NOT the SeedHammer II**

⚠ **`tinygo flash` targets whatever RP2350 is in BOOTSEL.**

1. **Physically disconnect the SeedHammer II before starting.**
2. Enumerate and confirm **exactly one** RP2350 is present: `picotool info -a`,
   and match the chipid — SH2 is `0x77c483b745abf55c`, Pico 2 is
   `0x66d3d60ff20abf2f`. Cross-check with `lsblk -o NAME,MODEL` (`SHII` → the
   engraver, **STOP**; `RP2350` → the Pico 2, safe).
3. Sign Pico images with `rehearsal-work/my-key.pem` — the Pico's OTP does not
   trust the real key.
4. This kernel has no `cdc_acm`, so read serial with `scripts/cdcread.py`. A
   one-shot print is lost — print in a loop.
5. **Print the byte count actually read and confirm it against 65 536**, so
   Step 3's bound is checked on-target as well as by the host test.
6. While a board is flashed, **confirm the PBKDF2 rate on this silicon** — §7.1's
   9 715 it/s was measured on an RP2350**A** and the SH2 is a **B**.

- [ ] **Step 5: Commit**

---

### Task 8: `seal/open.go` — the end-to-end headless pipeline

**Files:** create `seal/open.go`, `seal/open_test.go`

One entry point Phase B calls: bytes in → parsed header, public records,
`pubhash`, and (given a passphrase) the decrypted records. No UI, no globals.

**The KDF is an injectable seam** — a `func(passphrase string, salt []byte, iters
int) []byte` field, or a package-level call counter. Task 8's most important test
cannot be written without it.

- [ ] **Step 1: Failing test — drive every vector end to end from `vectors.json`**

For each: parse, split, allow-list, group-decode, hash, and where `ct_len > 0`,
open with the vector's passphrase and assert the exact records come back.

**Assert the per-record classification sequence for vector C** — `ms1`, `mk1`,
`mk1`, `md1`, `md1`, `md1`. A bundle that split correctly but classified nothing
would otherwise pass (§11.2).

Then the negatives:

- **`TestBadHeaderNeverReachesTheKDF`** — for each §6.2 violation, assert the KDF
  call count is **0**. Without this the mutant "move the bound checks after
  `DeriveKey`" passes every other test in the plan, just ~31 s slower — and on a
  watchdog-less firmware that is the difference between a refusal and a hang.
- **`TestTotalRecordCapSpansBothSections`** — 20 public + 5 secret is 25 total and
  must be refused with the count-naming error. Task 5's 25-record test exercises
  only the single-section path, so with the caller-side check deleted this split
  passes everything else (§6.4's cap is normative "across both sections
  together"; `container.rs:1-3` assigns it to the caller).
- `ct_len == 0` → **no passphrase is asked for and none is needed** (§10.2 step 4).
  Conditional on `ct_len > 0` and nothing else.
- `pub_len == 0` → **no hash is produced** (§10.2 step 3). The digest of an empty
  record set is a constant, and showing it on every fully-encrypted payload would
  teach the operator it is furniture.
- **A tampered public payload fails at the TAG — and the tamper must be one that
  survives the allow-list.** **REORDER vector D's public records.** Do not flip a
  byte: a flipped byte breaks BCH, so the record is refused at the allow-list
  before any key is derived, and a test asserting only "an error came back" then
  passes under the very mutant it exists to kill (AAD dropping the public
  section). Reordering keeps every record BCH-valid and every group decodable
  (`mk.reassemble` slots by `ChunkIndex`), so the pipeline reaches step 8.
  **Assert the error is the authentication error specifically**, not merely
  non-nil. Phase A must return an error that lets Phase B say *"wrong passphrase,
  **or** this payload has been altered"* — reporting only "wrong passphrase"
  loses the one signal §2.2 item 4 exists to raise.

- [ ] **Step 2: Run. Step 3: Implement. Step 4: Commit.**

---

## Mutation testing (required before Phase A is done)

Every mutant must name the test that fails, and the test must be **watched
failing**. Copy the file and restore from the copy — never `git checkout`, which
has destroyed uncommitted work here. Assert the substitution matched before
running. `touch` after restoring and confirm a rebuild, or the "restored" run is
still the mutant.

| Mutant | Killed by |
| --- | --- |
| `DeriveKey` ignores `iterations` | vector B |
| §6.2 bound checks moved after the KDF | `TestBadHeaderNeverReachesTheKDF` |
| unsealed-shape zero checks removed | the unsealed-fields test |
| `sealed` byte dropped from the hash | pubhash D≠E **and** the JSON literals |
| hash over a subset of the section | `every_byte_…` + the literals |
| `public_record_count` dropped | the JSON literals (LF-joined records are already injective, so a record-removal test passes under this mutant) |
| AAD = header only, public section dropped | Task 4's flipped-public-byte test **and** Task 8's reorder test |
| split before counting separators | `TestOverlongSectionRejectsBeforeSplitting` (**the `AllocsPerRun == 0` assertion**, not the returned error — both paths return the same error) |
| record-count cap dropped (single section) | Task 5's 25-record test |
| cross-section total cap dropped | `TestTotalRecordCapSpansBothSections` — Task 5's test is single-section and does **not** kill this |
| CR tolerated instead of refused | the container CR test |
| lowercase check removed | `TestRefusesAnUppercaseRecord` |
| allow-list → deny-list | `TestPublicSectionRefusesDebugCommand` |
| allow-list applied to `records[0]` only | `TestPublicSectionRefusesDebugCommand` (**because the command sits at index 2**) |
| grouping by HRP alone | **vector G, and nothing else** |
| non-chunked dispatch arm removed | `TestDecodesTwoDistinctNonChunkedCards` |
| decode step removed entirely | `TestRefusesBCHValidButUndecodable` |
| `md1` codeword cap removed | `TestRefusesAnOverlongRecord` |
| XIP read unbounded | the `clampRegion` host test — **and only because the bound is in an untagged file**; a bound inside `read_tinygo.go` is unreachable from `go test` |

The `RegionLen` total-size check in `ParseHeader` is **deliberately absent** from
this table: it is unreachable behind the section caps (Task 2 Step 3).

Record results in `design/agent-reports/`.

---

## What Phase A does NOT cover

- **All UI**: menu entry, unlock flow, 12/24-word entry, KDF progress, the
  §10.2.3 warning screen, the plate list. Phase B.
- **Session lifecycle (§10.2.2)** — secrets offered first, each wiped as its plate
  leaves by any route, including a cancelled engrave. Phase B, and it is the
  hardest thing in the whole feature.
- **Idle wipe (§10.2.4)**, keyed on residency rather than last keypress. Phase B.
- **Wiping.** Phase A holds decrypted records in memory and hands them back;
  Phase B owns the lifetime. Note `seedEntryFlow` returns `bip39.Mnemonic`
  = `[]Word` = `[]int`, scrubbed by manual zeroing — **not** `wipeBytes`, which
  takes `[]byte` and belongs to a different flow.

## Phase B constraints to carry forward (do not rediscover)

- **`layoutNavigation` indexes a fixed `[3]int`** (`gui/gui.go:1857`) — a fourth
  nav affordance **panics**. Back / Lock / OK is the entire budget.
- **`ChoiceScreen` does not scroll** and draws over its own title past ~7 entries.
  The plate list must use `bundleReviewFlow`'s paged shape
  (`gui/bundle_flow.go:224`).
- **Adding a menu program touches four sites** — the `program` enum (inserted
  *before* `bip85Derive`, not appended), the title switch (`gui/gui.go:1709`),
  the dispatch switch (`gui/gui.go:1537`), and `layoutMainPlates`
  (`gui/gui.go:1927`) — guarded by a compile-time lockstep assertion
  `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`gui/gui.go:172`).
- **There is no precedent for a conditionally-shown menu entry.** §10.1 requires
  one ("Absent → the feature is invisible"). The only existing conditional is
  `Features().Has(FeatureSecureBoot)`, which toggles a string suffix, not
  visibility. This is new machinery — budget for it.
