<!--
Persisted verbatim. opus-architect R0 gate of the Cycle D D1 plan
(IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D1.md @ 7c93473). Reviewer agentId
a0956156e05b98b53. Verdict: NOT GREEN 0C/1I — the reviewer EXECUTED the ported crypto
against all official vectors (proved correct end-to-end); the lone blocker is a fabricated
idx-3 test mnemonic literal (I1) + 6 minors. Disposition: folded (testdata loader replaces
inline crypto literals). The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Cycle D D1 plan (SLIP-39 crypto port)

**Reviewer:** opus architect (adversarial execution R0 gate, read-only)
**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D1.md` @ `7c93473`
**Spec:** `SPEC_seedhammer_slip39_recovery.md` §0/§4/§7 (R1 GREEN)
**Base:** fork `main` `20fa4c4`; oracle `mnemonic-toolkit/.../slip39/*.rs`
**Date:** 2026-06-18

## Verification Results

This is a crypto port, so I did not merely read the inline Go — I **assembled the plan's exact ported code** (gf256, lagrange, feistel, combine, the widened `ParseShare`+`decodeValue`) into a standalone harness on Go 1.26, linked the real firmware wordlist/RS1024, and executed it against the genuine official SLIP-0039 vectors and the negative corpus. Every port claim below is execution-verified, not eyeballed.

**Positive vectors — exact recovered master matches the published hex (`go vet` + `gofmt` clean):**
| Vector | Topology | Recovered | Result |
|---|---|---|---|
| idx 0 | 1-of-1, T==1 no-digest | `bb54aac4b89dc868ba37d9cc21b2cece` | PASS |
| idx 3 | 2-of-3, 128-bit | `b43ceb7e57a0ea8766221624d01b0864` | PASS (matches plan's pinned value) |
| idx 17 | group-threshold, 128-bit | `7c3397a292a5941682d7a4ae2d898d11` | PASS |
| idx 35 | 256-bit, **33-word**, multi-group | `5385…123b` | PASS (exercises long byte-oriented unpack) |
| idx 42 | extendable ext=1 (empty salt) | `48b1a4b80b8c209ad42c33672bdaa428` | PASS |
| idx 3 passphrase | `"TREZOR"` vs `""` | distinct secrets | PASS |

**Negative vectors — sentinel mapping (exactly as plan/§7 specify):** idx 1 → `errBadChecksum` (parse-stage); idx 4 → `errInsufficientShares`; idx 5 → `errIdentifierMismatch`; **idx 9 → `errGroupThresholdExceedsCount` (parse-stage, the I2 fold)**; **idx 12 → `errDigestVerificationFailed` (the critical digest gate)**; idx 13 → `errInsufficientShares`. **Panic-safety:** duplicate `(group,member)` set with valid-length values → `errDuplicateMemberIndex`, **no panic**; empty → `errEmptyShares`; bad value length → `errInvalidShareValueLength`.

**Line-by-line port confirmations (Go vs Rust):**
- **gf256:** generator **3**, `0x11b`, table loop `x=(x<<1)^x` + reduce, `exp[255]=1`, `mul` `>=255?-255`, `inv` `(255-log)%255` — all match `gf256.rs:31-88`. Independently recomputed `gfMul(0x53,0xCA)==1`, `a*inv(a)==1 ∀a`, `exp[1]==3`. The recon's generator-2-vs-3 bug is **absent**.
- **lagrange:** XOR-subtraction basis, per-byte loop, `secretIndex=255`/`digestIndex=254`/`digestLen=4` — match `lagrange.rs:37-91`.
- **feistel:** decrypt round order `i=3,2,1,0`, body `l[j]^=f[j]` then swap, output `r||l`, password `[]byte{i}||passphrase`, salt `saltPrefix||r`, `itersPerRound=(10000<<e)/4`, saltPrefix `"shamir"||be16(id)` (non-ext) / `nil` (ext) — match `feistel.rs:101-212`. A forward-encrypt/decrypt round-trip confirms direction. Recon's `r||l`-vs-`l||r` and `"shamir_extendable"`-as-salt bugs are **absent** (`"shamir_extendable"` correctly used only as the RS1024 cs in `share.go`, never the salt). Append-aliasing: each round rebuilds salt via `append(append([]byte(nil),salt...),r...)` (fresh backing) — **no shared-backing mutation**.
- **combine/recoverSecret:** empty→err; per-share len ∈{16,20,24,28,32}; six cross-share sentinels vs `shares[0]`; group-by sorted keys; per-group uniform mt + distinct member idx + exactly-mt; exactly groupThreshold groups; `recoverSecret` T==1 no-digest / T≥2 `HMAC-SHA256(R,S)[:4]` via `subtle.ConstantTimeCompare`; feistelDecrypt — match `mod.rs:206-458`. `ConsistentShares` is original firmware (Cycle-B analogue), not a Rust port — its logic is self-consistent.
- **share value extraction:** `decodeValue` get_bit MSB-first, per-byte 8-bit accumulator (no value-wide accumulator), leading-pad-zero check — matches `share.rs:260-369`. Parse-error ordering (word→length/padBits→checksum→group-threshold→value-pad) matches `share.rs:199-262`. All five word counts {20,23,27,30,33} → padBits {2,0,8,6,4} (all ≤8; W=27 is the `==8` boundary as claimed) → valueBytes {16,20,24,28,32}.

**Integration / environment:**
- `golang.org/x/crypto v0.52.0` present in `go.mod`; `pbkdf2.Key(password,salt []byte,iter,keyLen int,h func()hash.Hash)[]byte` legacy signature preserved — the plan's call matches.
- No identifier or `init()` collisions in package `slip39`; `errUnsupportedSize` references confined to the two files the plan edits (no dangling refs elsewhere).
- GUI: `confirmSLIP39Flow`/`engraveSLIP39` read only header fields, never `Value`, never construct `Share` positionally — adding `Value []byte` is purely additive. Guard tests `TestConfirmSLIP39Render`/`TestEngraveSLIP39BackoutRecognized` exist. GUI has no premature `Combine`/`Value`/`recoverSLIP39` refs — D1 is genuinely GUI-free and mergeable.
- 32-bit/TinyGo: `(10000<<15)/4 = 81,920,000` fits int32; decode is byte-oriented. Confirmed.
- Test-RNG wedge (`MNEMONIC_SLIP39_TEST_RNG`/`_IDENTIFIER`) exists, wired into the **CLI** path (`src/cmd/slip39.rs:416-499`) — reachable for reproducible fixtures; note `extendable` is hardcoded `false` there (ext=1 coverage comes from official idx 42, not fixtures, which the plan does).

## Findings

### CRITICAL
None. The crypto port is faithful — proven by execution against all official positive vectors (incl. 33-word), the full negative corpus, and panic-safety.

### IMPORTANT

**I1 — The hard-coded idx-3 mnemonic in Task 3 Step 1 (`TestParseShareExtractsValue`) is wrong; it is not a valid share and does not exist in the official vectors.**
The plan embeds (lines 260-261) as a "known-valid 128-bit (20-word) official vector share":
`"shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustard faded picture sister enchant wisdom flavor brave little gather"`
The real official idx-3 first share is:
`"shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed"`
The two diverge at word 11 (`mustard…` vs `mustang…`). The plan's string appears **nowhere** in `slip39_vectors.json`. It is not a valid RS1024 share, so `ParseShare` returns `errBadChecksum` → the test hits `t.Fatalf("ParseShare: %v")` and **fails**, contradicting the plan's GREEN claim and blocking the implementer who pastes it verbatim. (The other pinned strings — idx-3 master `b43c…0864`, idx-17 `7c33…8d11`, the AES inverse pair `gfMul(0x53,0xCA)==1`, the salt/iters assertions — are all execution-verified correct; only this one mnemonic literal is wrong.)
**Required fix:** replace the embedded idx-3 mnemonic with the correct string above (or, preferably, drop the inline literal entirely and load via `vectorShare(t,3,0)` from `testdata/slip39_vectors.json`, which Task 6 establishes — eliminating the transcription-risk class). Re-verify the idx-3 assertion against the testdata after the fix.

### MINOR
- **M1 — Orphaned constants.** Replacing the `switch len(fields)` gate leaves `wordsShort`/`wordsLong` (`share.go:31-32`) unused. Go permits unused package-level consts (compiles + vets clean), but the self-review checklist's "no dangling refs" should explicitly remove them for tidiness.
- **M2 — `errorsIs` wrapper is gratuitous.** Task 3 tests call `errorsIs(err, sentinel)`; the firmware's existing tests use `errors.Is` directly. Either define the trivial wrapper in Task 6's helper bag or just use `errors.Is`. Cosmetic.
- **M3 — Step-1 header says "INVERT" but the body says "Remove" for the 33-word `errUnsupportedSize` assertion.** "Remove" is correct (the old test feeds junk `"duckling"×33`, which now fails RS1024, not parses clean); the implementer must delete, not flip-to-expect-nil. The NOTE is right; the header wording is loose.
- **M4 — Length-gate formulation (b) labeled "equivalently."** The general `len>=20 && padBits<=8 && valueBytes∈{…}` form reduces to the same five counts (valueBytes uniquely determines word count), so it is equivalent in effect, but the explicit `switch {20,23,27,30,33}` (formulation a) is unambiguous and tested; prefer it.
- **M5 — Plan's inline `combine.go` var block is not gofmt-aligned** (`errIterationExponentMismatch` breaks the `=` alignment). Self-heals under the per-task `gofmt -l` guard the plan already mandates; non-blocking.
- **M6 — Fixture reproducibility wording.** Task 6 says "a small `cargo test`/`examples/` harness calling `slip39_split`"; the deterministic wedge actually lives in the CLI layer. Either path is reproducible (a raw `slip39_split` call with a seeded `ChaCha20Rng` works identically); the committed JSON is what the test reads, so this is a regeneration-ergonomics note, not a correctness issue.

## Verdict

**NOT GREEN — 0 Critical / 1 Important.**

The cryptographic port itself is correct and faithful end-to-end — I executed the plan's exact Go against all official positive vectors (including the 256-bit/33-word and extendable cases), the full negative corpus with correct sentinel mapping, and panic-safety, all passing. No transcription error exists in the field arithmetic, interpolation, Feistel, digest, combine, or value-unpacking. The single blocking item is a test-fixture defect, not a crypto defect:

**Required fix before GREEN:**
1. **I1** — Correct the fabricated idx-3 mnemonic literal in `TestParseShareExtractsValue` (Task 3, Step 1). Use the verified official share `shadow pistol … cylinder mustang wrist rescue view short owner flip making coding armed`, or replace the inline literal with a `vectorShare(t,3,0)` testdata load. Re-verify the assertion.

Fold I1 (and ideally the MINORs), persist this review verbatim to `design/agent-reports/`, and re-dispatch for the GREEN re-confirmation.
