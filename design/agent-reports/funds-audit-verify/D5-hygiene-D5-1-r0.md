# Adversarial verification — D5-1 (verifier #0)

**Finding:** `me bundle` echoes the full input line to stderr — leaks an ms1 secret body
on the mangled-HRP path and systematically echoes privacy-critical xpubs.
**Claimed severity:** important. **Location:** `crates/me-cli/src/bundle.rs:54`.

## Verdict: CONFIRMED (not refuted). Severity important is honest.

I tried to refute on four axes (cited code, reachability, upstream/other-layer guard,
severity honesty). All four support the finding.

## 1. Cited code behaves exactly as claimed

`crates/me-cli/src/bundle.rs` `impl Display for BundleError` interpolates the full input
string `s` in three arms:
- `:54` `BundleError::Classify(s, e) => write!(f, "cannot classify '{s}': {e}")`
- `:55` `BundleError::Validate(s, e) => write!(f, "invalid string '{s}': {e}")`
- `:60` `BundleError::Md1HeaderRead(s, e) => write!(f, "cannot read md1 chunk header for '{s}': {e}")`

The wrapped values are the *full* line: `parse_line` builds `BundleError::Classify(s.to_string(), e)`
(`bundle.rs:96`) and `BundleError::Validate(s.to_string(), e)` (`:101`). The CLI emits these
verbatim: `run_bundle_cli` does `eprintln!("me: {e}")` on any `run_bundle` error
(`crates/me-cli/src/main.rs:184`). So the whole line reaches stderr.

## 2. The ms1-leak failure scenario is reachable — nothing upstream stops it

- The ms1 refusal is a classify-only pre-scan that fires only on an **exact** `ms` HRP:
  `run_bundle` loops `if classify::classify(line) == Ok(Format::Ms) { return RefusedSecret }`
  (`bundle.rs:188-192`), and `classify` matches the HRP substring before the first `1`
  literally against `"md"/"mk"/"ms"` (`classify.rs:42-51`).
- A mangled HRP such as `msx1…` → `classify` returns `Err(UnknownHrp("msx"))`, **not**
  `Ok(Ms)`, so the pre-scan is dodged. `parse_line` then re-runs `classify`, gets the same
  `Err`, and wraps it as `BundleError::Classify(full_line, …)`. The intact codex32 secret
  body after the mangled HRP is printed to stderr.
- Reachability requires no upstream validation to pass — on the contrary, the string reaches
  the echo *because* classify rejects it. No other layer redacts `{s}`.

## 3. The asymmetry claim (converter hardened, bundle regressed) is real

The converter's `ConvertError::Classify(e) => write!(f, "{e}")` (`lib.rs:29`) prints only the
inner `ClassifyError`, whose `UnknownHrp(h)` Display carries just the HRP substring
(`classify.rs:30-32`), never the body. So `me` (converter) feeding the same `msx1…` prints
only `me: unrecognized HRP 'msx' (...)` — 3 chars, no secret. The `bundle` first-party
wrappers embed the whole line. Confirmed asymmetry.

## 4. Empirical probe (standalone, outside the repo)

The full `me` binary would not build in this environment (pre-existing secp256k1-sys vendored
`depend/secp256k1/src/precomputed_ecmult.c` missing — unrelated to the finding). I instead
compiled a standalone program with **verbatim copies** of `classify()` (classify.rs:40-52),
the `BundleError::Classify` Display arm (bundle.rs:54), the pre-scan gate (bundle.rs:188-192),
and the `eprintln!("me: {e}")` emit (main.rs:184). Output:

```
--- correctly-typed ms1 into bundle:
me: REFUSED (no echo)
--- mangled-HRP ms1 (typo) into bundle:
me: cannot classify 'msx10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f': unrecognized HRP 'msx' (expected md, mk, or ms)
--- leak assertion: stderr contains secret body? true
```

This matches the finder's demonstrated output line-for-line: the correctly-typed ms1 is
refused with no echo, but the typo variant emits the intact secret body
`0entrsqqqq…cj9sxraq34v7f` to stderr. (Program: scratchpad `/repro/repro.rs`.)

The codex32 `ms` HRP is a fixed constant, so an intact data body with a corrupted HRP is
trivially recoverable as the secret (strip the stray char, prepend `ms`). The body carries the
entropy; the leak is a genuine seed-material disclosure, not just metadata.

Claim #2 (systematic xpub echo) is likewise confirmed: any invalid md1/mk1 → `BundleError::Validate(full_line, …)`
→ `invalid string '<full xpub-bearing string>': …`. This is the common failure case, not a corner.

## Severity assessment

`important` is honest and I do not adjust it.
- It leaks the **actual ms1 secret** (not just xpubs) into stderr / scrollback / `2>logfile` /
  CI logs — a direct violation of the tool's single most important invariant ("secret entropy
  must never reach a log/RF"), which the converter path was explicitly hardened to uphold.
- Not `critical`: it does not produce a wrong-but-accepted plate or a direct spend; realizing
  funds loss needs a chain (secret pasted into a public-only tool + a specific HRP typo that
  preserves the body + persistent stderr capture + adversary access to that capture). This is a
  defense-in-depth gap, not a primary-control failure — the correctly-typed ms1 is still refused
  without echo.
- Not merely `moderate`: the item at stake is the seed itself, the fix is trivial (redact `{s}`
  to HRP+length as the converter effectively already does), and the finder's severity model
  ("any path leaking the actual ms1 secret is important; md1/mk1-only echo is moderate/low") is
  internally consistent and defensible. A reviewer could argue moderate on double-fault
  reachability grounds, but for seed material in a hardware-wallet-adjacent tool, important is
  the safer and defensible calibration.

## Conclusion

refuted = false; confidence = high; severity unchanged (important).
