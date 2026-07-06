# Verify D6-7 — "No fuzz or property targets on the parse/bundle pipeline"

Verifier #0 (adversarial). Verdict: **REFUTED** (as a funds-safety finding). Confidence: **high**.

## The finding
- id D6-7, severity moderate, location `crates/me-cli/src/bundle.rs:94` (`parse_line`).
- Claim: no cargo-fuzz targets, no proptest tests; `convert`/`run_bundle`/`classify`/`parse_line`
  consume arbitrary stdin but are only hit by fixed fixtures.
- Failure scenario: "a crafted or malformed multiline input hits an unhandled slice/bit-read
  path in `parse_line`'s md1 probe and panics (or, worse, silently misroutes), producing a
  crash or an inconsistent manifest instead of a clean exit-4 refusal."

## What is true
The *observation* is factually correct. Confirmed there are:
- no `fuzz*` directories outside `third_party/`,
- no `proptest` / `quickcheck` / `arbitrary` / `cargo-fuzz` / `libfuzzer` references anywhere in `crates/`,
- `me-cli` dev-deps are only `assert_cmd` + `predicates`.

So "there are no fuzz/property targets" is TRUE. That is a test-hygiene gap.

## Why the funds-safety failure scenario is NOT substantiated
The severity ("moderate", funds-relevant) rests entirely on the *failure scenario* — a reachable
panic or a fund-losing silent misroute in the md1 probe. Reading the actual code at the cited
location shows every leg of that scenario is already closed:

1. **The md1 probe never sees unvalidated input.** In `parse_line` (bundle.rs:94), the BitReader
   probe at lines 144-148 runs only *after* `validate::validate(fmt, s)` (line 101) succeeds.
   `validate::validate(Format::Md, …)` (validate.rs:43) *is* `md_codec::codex32::unwrap_string(s)`
   — the same call whose `(bytes, bit_count)` output feeds the probe (line 132). So the bytes the
   BitReader reads come from a string that already passed BCH/pristine verification. Arbitrary
   garbage fails `classify` (wrong HRP → `Classify` → exit 4) or `unwrap_string` (bad BCH →
   `Validate` → exit 4) long before the bit-read. The scenario's "malformed input hits the
   bit-read path" is not reachable.

2. **The bit-read is defensively coded — there is no "unhandled" read.**
   `probe.read_bits(5).map(|sym| sym & 0x01 != 0).unwrap_or(false)` — a read error yields `false`
   (treated as unchunked), never a panic. `ChunkHeader::read` is only called when the chunked flag
   is set, and its `Result` is matched *exhaustively* (Ok / `ChunkHeaderChunkedFlagMissing` /
   `WireVersionMismatch` / catch-all `Err(e) => Md1HeaderRead`). No `unwrap`/`expect`/indexing on it.

3. **No unhandled slice.** The only manual slice in the pipeline is `classify`'s `s[..sep]`
   (classify.rs:46) where `sep = s.find('1')` is the byte offset of an ASCII `'1'` — always a char
   boundary, so the slice cannot panic on multibyte UTF-8.

4. **`run_bundle` propagates, never panics.** It collects `parse_line` via
   `.collect::<Result<_,_>>()?` and maps every error to an exit code through `exit_code()`
   (2 usage / 3 refused / 4 invalid). A bad line yields a clean exit-4, exactly the "clean
   refusal" the finding worries is absent.

5. **The "silent misroute" alternative is D6-6, not a new defect.** A set-member-admitted-as-
   standalone risk is the md1 discriminator concern already filed as D6-6 (important). It is a
   correctness question about the `& 0x01` discriminator on *valid* chunked md1, not a
   panic/robustness gap, and is out of scope for D6-7's "arbitrary input → panic" claim.

Any residual panic would have to originate *inside* `md_codec`/`mk_codec` internals — which the
finder's own report explicitly scopes out ("crate INTERNALS audited separately").

## Empirical probe
Ran an adversarial fuzz against the prebuilt release binary
(`target/release/me --hex` and `me bundle`), harness at `/var/tmp/d6probe/fuzz.sh`:
- curated seeds (empty, `1`, `md1`, oversize 5000-char, uppercase, padded, `md1`+mk1 body, ms1, …);
- multi-line combos (ms1 anywhere, duplicate chunks, mixed md1/mk1);
- per-position byte mutation of the valid md1 and mk1 fixtures over a 40-char alphabet;
- ~1200 random-body md1/mk1/ms1/xx1 inputs incl. base64 noise.

Result: **6377 invocations, 0 panics** (no exit 101/134/≥132). Every input produced a clean
0/2/3/4 exit. The valid md1 fixture itself exercises the probe's chunked-flag branch; the existing
unit test `md1_chunked_set_verifies_and_drop_fails` exercises the real `ChunkHeader::read` arm on
genuine multi-chunk md1.

## Severity assessment
The failure scenario (panic / inconsistent manifest / crash instead of exit-4) is not reachable
and is empirically absent over 6377 hostile inputs. The finding therefore does not produce a
wrong-but-accepted plate or lost funds. What remains is a pure defense-in-depth / regression-lock
hygiene suggestion (a fuzz target would *lock in* the already-true no-panic property). As a
funds-safety finding at "moderate" it is refuted; if retained at all it is a "low" hardening
nice-to-have, not a moderate funds risk.

## Verdict
**refuted = true**, confidence **high**. The observation (no fuzz/proptest) is true but the
concrete harm asserted in the failure scenario cannot be substantiated: the probe path is gated by
prior validation, is defensively coded with `unwrap_or(false)` and an exhaustive `Result` match,
has no unhandled slice, and survived 6377 adversarial invocations without a single panic.
