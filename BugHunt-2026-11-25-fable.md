# Bug hunt 2026-11-25 (Fable): mnemonic-engrave

Report-only review of `mnemonic-engrave` at `80dfaeb`, part of a constellation-wide hunt across all 11 bg002h repositories. Nothing in this repository was changed by the hunt; this file is the only addition.

**How it was done.** Eight Fable reviewers each audited an area (one of them the boundaries between repositories), reproducing what they could with locally built binaries or scratch tests that were not committed. Three further Fable verifiers then tried to disprove every High and Medium finding (and several Low ones) and re-graded them. Grades below are the verifiers' final grades where they ran.

**Grades.** Critical: can lose funds or leak a seed in ordinary use. High: wrong wallet, addresses or keys, or secret exposure, plausible in normal use, or recovery broken. Medium: needs an unusual but realistic input, or fails in a way a careful user might not notice. Low: fails closed, hardening, or a small exposure. Info: context, not a bug.

**Categories.** security (secret or key exposure, injection, integrity of what is trusted), funds (wrong address, amount, descriptor or recovery that could lose or strand coins), other.

**Status.** "Confirmed (reproduced)" means a command or test showed it. "Confirmed by reading" means the code path was traced but not executed. "Reviewer-confirmed, not re-verified" means the first reviewer reproduced or read it and no verifier re-checked it.

**Totals:** 2 Medium, 4 Low. No Critical findings.

## Summary

| ID | Grade | Category | Status | Finding |
|---|---|---|---|---|
| E1 | Medium | security | Confirmed (reproduced) | `me` and `me bundle` print secret material to stderr via the "unrecognized HRP" message |
| E2 | Medium | funds | Confirmed (reproduced) | Normative mt1 spec has a wrong BIP-68 fact; a relative-locked tx is engraved `NO TIMELOCK` |
| E3 | Low | funds | Reviewer-confirmed, not re-verified | `me sysw pack --as md1` accepts an origin path contradicting the xpub's depth/child number |
| E4 | Low | security | Reviewer-confirmed, not re-verified | `me sysw pack` says "the device will treat it as a SECRET" about records it packed as non-secret |
| E5 | Low | other | Confirmed (reproduced) | `me`'s mt-codec pin predates the balanced-split decode refusal |
| X7 | Low | other | Reviewer-confirmed, not re-verified | md-codec pin behind 0.48.6 can mint a card the decoder refuses |

IDs: `md#`, `mk#`, `ms#`, `toolkit#`, `G`, `refugium#`, `wallet#`, `E`, `T`, `SH#`, `M` are per-repository findings; `X` findings are at a boundary between repositories and appear in every report they touch.

## Findings

## E1: `me` and `me bundle` print secret material to stderr via the "unrecognized HRP" message

**Grade:** Medium · **Category:** security · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `engrave`)

### E1. `me` and `me bundle` print secret material to stderr through the "unrecognized HRP" message

- Severity: **Medium** (High wherever stderr is captured: CI logs, `script`, terminal scrollback, a wrapper GUI)
- Status: **CONFIRMED**
- Evidence:
  - `crates/me-cli/src/classify.rs:44-53` — the HRP is "everything before the FIRST `1`" of the whole trimmed input, with no length or charset bound, and is stored in `ClassifyError::UnknownHrp(String)`.
  - `crates/me-cli/src/classify.rs:32-34` — `Display` interpolates it.
  - `crates/me-cli/src/lib.rs:48` (`ConvertError::Classify(e) => write!(f, "{e}")`) and `crates/me-cli/src/bundle.rs:107` (`"cannot classify input: {e}"`) print it. `crates/me-cli/src/seal/record.rs` `RecordError::Unclassifiable(e.to_string())` does the same for `me seal`.
  - The crate states the opposite as an invariant at `crates/me-cli/src/bundle.rs:96-101` ("A1/F1: NEVER interpolate the raw input string here — a mangled-HRP ms1 (`msx1…`) would print its intact secret body to stderr"), and the test `no_bundle_error_display_leaks_the_input_body` (`bundle.rs:655-658`) only injects a canary into the *first* field, with `UnknownHrp("zz")` as the second, so it cannot see this.
- User impact: the two secret shapes the tool exists to refuse are echoed in full when classification fails:
  1. an ms1 whose separator was mistyped (`msl0…`, `ms 0…`, `ms0…`) followed by any later `1` (a second line holding an md1/mk1, or a note such as "plate 1");
  2. a BIP-39 mnemonic on a line (or, for the bare converter, anywhere in the file) that is followed by a `1`.
  Exit code is 4 ("invalid"), not 3 ("refused secret"), and nothing tells the operator a secret was just printed. Partial leaks of the same kind: a hex private key is echoed up to its first `1` (12 hex chars in the run below), a WIF likewise.
- Reproduction (the ms1 below is the public test vector from `seal/record.rs`, with `1` typed as `l`):

  ```
  $ printf 'msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw\nmd1yqpqqxqq8xtwhw4xwn4qh\n' > mangled.txt
  $ me --in mangled.txt --hex
  me: unrecognized HRP 'msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw
  md' (expected md, mk, ms, or mt)
  exit 4

  $ printf 'msl0entrs…q5f042qmrw90mw plate 1\nmd1yqpqqxqq8xtwhw4xwn4qh\n' | me bundle
  me: cannot classify input: unrecognized HRP 'msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw plate ' (expected md, mk, ms, or mt)
  exit 4

  $ printf 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\nmd1yqpqqxqq8xtwhw4xwn4qh\n' | me --hex
  me: unrecognized HRP 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
  md' (expected md, mk, ms, or mt)
  exit 4

  $ printf 'seed: abandon … about (12 words)\nmd1yqpqqxqq8xtwhw4xwn4qh\n' > notes.txt; me bundle --in notes.txt
  me: cannot classify input: unrecognized HRP 'seed: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about (' (expected md, mk, ms, or mt)
  exit 4

  $ printf 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n' | me seal --out x.uf2
  me: record 0 … in the secret section: unrecognised record: unrecognized HRP 'e3b0c44298fc' (expected md, mk, ms, or mt)
  ```
- Fix direction: `UnknownHrp` should carry no input text (or only an HRP that is short and all-lowercase-alphanumeric, e.g. ≤ 8 chars, otherwise "not shown"). Run a secret-shape check (BIP-39 word run, `ms`-prefixed codex32 body) before classification and answer with the exit-3 refusal. Extend the canary test to put the canary inside the `ClassifyError`. `mt` (`mt-cli/src/validate.rs` `secret_guard`) and `me sysw pack` (`unknown_reason`) do not echo and were checked.

**Verifier re-check** (source: `verify-2`)

### E1: `me` echoes secrets through the unrecognized-HRP message

**Verdict: CONFIRMED. Grade: Medium. Category: security.**

```
$ printf 'msl0entrsqqg5y2z9…q5f042qmrw90mw\nmd1yqpqqxqq8xtwhw4xwn4qh\n' > mangled.txt; me --in mangled.txt --hex
me: unrecognized HRP 'msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw
md' (expected md, mk, ms, or mt)                                   rc=4
$ printf 'abandon … about\nmd1yqpqqxqq8xtwhw4xwn4qh\n' | me --hex
me: unrecognized HRP 'abandon abandon … abandon about
md' (expected md, mk, ms, or mt)                                   rc=4
```

Bounds found while trying to break it: a well-formed `ms1` is refused without echo (rc=3, both `me` and
`me bundle`); a mangled ms1 or a mnemonic with no later `1` in the input gives "not a bech32 string" with no
echo; `me bundle` on mnemonic-then-md1 lines did not echo (per-line classification), so the bundle leak needs
a `1` on the same line. The leak therefore needs a secret followed by a `1`, which a file holding a seed and
an md1 card supplies on the bare converter. The secret was already in the user's file or stdin; the new
exposure is stderr, scrollback and any captured log, at exit 4 with no word that a secret was printed. It
contradicts the crate's own stated invariant (`bundle.rs:96-101`).

## E2: Normative mt1 spec has a wrong BIP-68 fact; a relative-locked tx is engraved `NO TIMELOCK`

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `engrave`)

### E2. Normative mt1 spec states a wrong BIP-68 fact; the engraved legend says `NO TIMELOCK` on a relative-locked transaction (cross-repo: mnemonic-transaction T1)

- Severity: **Medium**
- Status: **CONFIRMED** (behaviour reproduced in `mt`; the protocol fact is BIP-68's own text)
- Evidence: `design/SPEC_mt_v0_1.md:691` and `:2848-2866` ("A BIP-68 relative timelock lives in the witness script as `OP_CSV` … Reading it means evaluating the sending wallet's script, which is out of scope … The legend now reads `NO TIMELOCK`: precisely true about the fields `mt` read").
- Why it is wrong: BIP-68 is a consensus rule on the transaction's own fields. With `nVersion >= 2`, an input whose `nSequence` has bit 31 clear is relative-locked (bit 22 selects 512-second units, low 16 bits are the value). `OP_CSV` (BIP-112) only forces the spender to set that field. So the lock is a field read, the same class as the `nSequence` finality rule the spec already calls "in scope", and `mt` reads `nSequence` on every input already.
- User impact: a pre-signed recovery/inheritance spend through an `older(N)` path has `nLockTime = 0` and `nSequence = N`. `mt encode` prints `LOCKTIME NO TIMELOCK` and puts `NO TIMELOCK` in the legend that is cut into steel next to the transaction. The spec argues this only over-warns; in practice an heir reads "no timelock", broadcasts, gets `non-BIP68-final`, and has a plate that states something false about when the money moves. The spec's own RCW fixture has such a leaf (32,768 blocks).
- Reproduction: see mnemonic-transaction T1 (`mt inspect --in csv.hex`, version 2, `nSequence = 0x00008000`, `nLockTime = 0` → `LOCKTIME  NO TIMELOCK`).
- Fix direction: correct §8.4's text here first (this repo owns the spec), then have `locktime::read` report per-input BIP-68 locks (`RELATIVE LOCK: N BLOCKS` / `~N×512 s after the input confirms`) when version ≥ 2 and bit 31 is clear; keep `NO TIMELOCK` for the case where neither field locks.

**Verifier re-check** (source: `verify-3`)

### E2 / T1 — a BIP-68 relative-locked transaction is reported and engraved as `NO TIMELOCK`

- Verdict: **CONFIRMED**. Grade: **Medium**. Category: **funds**. Cross-repo: mnemonic-engrave (spec owner),
  mnemonic-transaction (code).
- Protocol fact: BIP-68 is a rule on the transaction's own fields. rust-bitcoin (vendored,
  `mnemonic-toolkit/vendor/bitcoin/src/blockdata/transaction.rs`): the `Sequence` doc lists "Indicating and encoding
  [BIP-68] relative lock-times" (:312); `is_relative_lock_time` is `self.0 & LOCK_TIME_DISABLE_FLAG_MASK == 0`
  (:393-395); bit 22 selects 512-second units (:399-407); "BIP-68 only uses the low 16 bits" (:499);
  `Version::TWO` is "post-BIP-68" (:1158). `OP_CSV` (BIP-112) only forces the spender to set that field. So the spec
  sentence "A BIP-68 relative timelock lives in the witness script as `OP_CSV` … Reading it means evaluating the
  sending wallet's script" (`mnemonic-engrave/design/SPEC_mt_v0_1.md:2848-2866`, also :691) is wrong, and its
  conclusion that `NO TIMELOCK` is "precisely true about the fields `mt` read" is false: `mt` reads `nSequence`.
- Code: `mnemonic-transaction/crates/mt-cli/src/locktime.rs:89-101` returns `Lock::None` whenever `nLockTime == 0`;
  `legend()` (:155-169) prints `NO TIMELOCK`.
- Reproduced: `work-memt/csv.hex` is version 2, one input with `nSequence = 0x00008000` (32,768 blocks),
  `nLockTime = 0`. `mt inspect --in csv.hex --bitcoin-cli /nonexistent` prints
  `LOCKTIME  NO TIMELOCK                      current height unknown (no node)`, exit 0.
- Grade: the steel legend states something false about when the money can move; the typical pre-signed recovery
  spend (`older(N)`) is exactly this shape. No coin moves wrongly (an early broadcast is refused `non-BIP68-final`),
  so not High.

## E3: `me sysw pack --as md1` accepts an origin path contradicting the xpub's depth/child number

**Grade:** Low · **Category:** funds · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `engrave`)

### E3. `me sysw pack --as md1` accepts a key whose origin path contradicts the xpub's own depth and child number

- Severity: **Low**
- Status: acceptance **CONFIRMED**; consequence on restore **SUSPECTED** (not decoded back — no `md` binary built)
- Evidence: `crates/me-cli/src/descriptor/` cascade/admit; `design/FIX_sec_canonical_keeps_xpub_depth.md:72` rules "No warning is added when a key's depth or child number contradicts its origin" for the `--as descriptor` form. The md1 route has no such excuse: mk-codec's hazard note (mnemonic-key CLAUDE.md, "xpub/origin consistency") says the compact form drops depth and child number and rebuilds them from the origin, so a mismatch must be refused.
- Reproduction (xpub is depth 3, child `0h`):

  ```
  $ echo "wpkh([73c5da0a/84h/0h/7h]xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz/<0;1>/*)" > d.txt
  $ me sysw pack --in d.txt --as md1 --out p.bin      # exit 0, 3 md1 records, no warning
  $ # same for origin 84h/0h/0h/5h (4 levels) and for 84h (1 level)
  ```
- User impact: the plate records an origin the key was not derived at. Watch-only use is unaffected (same key material), but a recoverer who derives from the seed at the engraved origin gets a different key and an apparently empty wallet, and the rebuilt xpub string differs from the one the wallet exported. The mismatch is detectable from the 78 bytes `me` already parsed.
- Fix direction: on the md1 route, refuse (or at least warn) when `origin.len() != depth` or the last origin element ≠ the key's child number, for depth ≥ 1.

## E4: `me sysw pack` says "the device will treat it as a SECRET" about records it packed as non-secret

**Grade:** Low · **Category:** security · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `engrave`)

### E4. `me sysw pack` says "the device will treat it as a SECRET" about records it has just declared non-secret and packed in cleartext

- Severity: **Low**
- Status: **SUSPECTED** (message contradiction confirmed; device behaviour not exercised)
- Evidence: `crates/me-cli/src/main.rs` `decide_sealing` (3007-3046, uses `sysw::classify`, where an undecodable md1/mk1 is `Class::MdMk`, not secret) versus `report_unconfirmed` (2554-).
- Reproduction: `printf '%s\n%s\n' <chunk 1 of 3> <chunk 2 of 3> | me sysw pack --out p.bin` prints `sealing:  NOT SEALED — no record in this payload is secret material` and, three lines later, `record 0 …: an md1/mk1 this tool could not decode; the device will treat it as a SECRET`; exit 0.
- User impact: for the case the unconfirmed check was written for (entropy wrapped as a BCH-valid md1, `record.rs` test `entropy_smuggled_as_md1`), the host identifies the record as secret-on-device and still writes it to flash unencrypted without asking for a passphrase.
- Fix direction: either seal when any md1/mk1 is unconfirmed, or refuse; at minimum make the two lines agree.

## E5: `me`'s mt-codec pin predates the balanced-split decode refusal

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `engrave`)

### E5. `me`'s mt-codec pin predates the balanced-split decode fix (cross-repo; partly tracked)

- Severity: **Low**
- Status: **SUSPECTED** (pin gap confirmed; no non-conforming set crafted)
- Evidence: `crates/me-cli/Cargo.toml:104` pins mt-codec `eec7910b`; mnemonic-transaction `95c39a3` ("decode refuses a set that is not §3b's balanced split", bh-mt-I18) is after it. `crates/me-cli/src/sysw/mt.rs` `diagnose` calls `mt_codec::pipeline::decode`.
- User impact: `me sysw pack` can report "confirmed" for an mt1 set that current `mt decode` refuses (a set no conforming encoder writes). mnemonic-transaction FOLLOWUPS `mt-codec-error-variant-semver` (open) already expects the pin to move; listed here only so the pin bump is not forgotten.

**Verifier re-check** (source: `verify-3`)

### E5 / X5 — engrave's mt-codec pin misses the balanced-split refusal

- Verdict: **CONFIRMED** (pin gap; behaviour not executed). Grade: **Low**. Category: **other**. Already tracked.
- `mnemonic-engrave/crates/me-cli/Cargo.toml:104` and `Cargo.lock:631-633` pin mt-codec `eec7910b`.
  `git -C mnemonic-transaction log eec7910..HEAD -- crates/mt-codec` lists `3923ad6`, `1b336c2`, `95c39a3`
  ("decode refuses a set that is not §3b's balanced split", new `Error::ChunkLengthMismatch`).
- Reach: `me` calls `mt_codec::pipeline::decode` at `crates/me-cli/src/sysw/mt.rs:163,240`. After it, `me` still
  requires zero corrections, a parse as a transaction and the 20-bit txid binding (`mt.rs:163-195`), so a
  non-conforming set is admitted only if its order-dependent bytes are a transaction with a matching txid prefix.
- Tracked as mnemonic-transaction FOLLOWUPS `mt-codec-error-variant-semver` (open, names `mt.rs:163,:240`).
  Both builds report mt-codec 0.2.0 (same version-reuse pattern as X2).

## X7: md-codec pin behind 0.48.6 can mint a card the decoder refuses

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X7. md-codec pins behind 0.48.6/0.48.7: a template numbered out of first-occurrence order can still mint a card the decoder refuses

- **Severity:** Low
- **Status:** SUSPECTED (0.48.5 read from `mnemonic-refugium/vendor/md-codec/src/canonicalize.rs:217-251`; not executed)
- **Repos:** mnemonic-refugium and mnemonic-engrave (0.48.5), refugium-wallet (0.48.4), seedhammer Go port
  (`md/bits.go:6`, tracks 0.47.0), descriptor-mnemonic
- **Evidence:** in 0.48.5 `canonicalize_placeholder_indices` remaps use-site overrides but keeps the old `@0`
  baseline, so `wsh(multi(2,@1/<2;3>/*,@0/<0;1>/*))` mints a card with an override keyed on `@0`, which decode
  refuses (0.48.6 CHANGELOG, bughunt-1119 C2). 0.48.7 makes `encode_payload` decode its own output. The failure
  is fail-closed at decode, so the risk is an engraved card that cannot be read back, not a wrong wallet.
  mr-codec mints through `encode_payload_unadmitted` and its own templates (`render.rs`), which set one use-site
  for all keys, so `mr` is unlikely to reach it; `me` does not mint md1. The Go port's `md/canonicalize.go`
  (364 lines) should be checked against the 0.48.6 rule under the Rust-primary rule.
- **Fix direction:** move the pins to the next md-codec release (see X2); port the 0.48.6 vector to Go.

---

_Generated by [Claude Code](https://claude.ai/code) on 2026-11-25. Findings were not fixed; each fix needs its own review under this repository's gates._
