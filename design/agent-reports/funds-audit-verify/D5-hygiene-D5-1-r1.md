# Adversarial verification — D5-1 (D5-hygiene)

- Finding: `me bundle` echoes the full input line to stderr; leaks an `ms1` secret body
  on the mangled-HRP path and systematically echoes privacy-critical xpubs.
- Claimed severity: **important**
- Verifier verdict: **CONFIRMED** (refuted = false), severity **important** (unchanged),
  confidence **high**.

## What I checked

### 1. Does the cited code behave as claimed at that location?
Yes. `crates/me-cli/src/bundle.rs` `Display for BundleError` interpolates the full input
string `s` at exactly the cited lines:
- `:54` `BundleError::Classify(s, e) => write!(f, "cannot classify '{s}': {e}")`
- `:55` `BundleError::Validate(s, e) => write!(f, "invalid string '{s}': {e}")`
- `:60` `BundleError::Md1HeaderRead(s, e) => write!(f, "cannot read md1 chunk header for '{s}': {e}")`

The wrappers are populated with `s.to_string()` in `parse_line` (`:96`, `:101`, `:166`),
and `run_bundle_cli` prints them to stderr unredacted at `main.rs:184` (`eprintln!("me: {e}")`).

### 2. Is the ms1-leak failure scenario reachable / does another layer prevent it?
Reachable; no upstream layer prevents it. `classify::classify` (`classify.rs:40-51`)
lowercases the HRP and matches **exactly** `md`/`mk`/`ms`; any other HRP → `UnknownHrp`.
The bundle ms1 refusal pre-scan (`bundle.rs:188-192`) fires **only** on
`classify(line) == Ok(Format::Ms)`. A mangled 2-char HRP (`msx1…`, or a spurious early
`1`) classifies as `Err(UnknownHrp)`, so:
  - it dodges the `RefusedSecret` pre-scan, then
  - `parse_line` (`:96`) returns `BundleError::Classify(s.to_string(), _)`, whose Display
    echoes the entire input line — the intact codex32 secret data body — to stderr.

The data part after the (mistyped) HRP is byte-identical to the real secret, and the true
HRP of a codex32 secret is always `ms`, so the echoed line trivially reconstructs the full
`ms1` seed share.

### 3. Concrete probe (built release binary, out-of-repo)
Built `me` 0.3.0 to `/var/tmp/d5probe` (repo `/tmp` tmpfs was full at 100%; built with
`CARGO_TARGET_DIR`/`TMPDIR` on a disk-backed fs — no repo files touched). Results:

- **Mangled-HRP ms1** — `printf 'msx10entrsqqqq…cj9sxraq34v7f\n' | me bundle` → exit 4, stderr:
  `me: cannot classify 'msx10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f': unrecognized HRP 'msx' (expected md, mk, or ms)`.
  The full secret body `0entrsqqqq…cj9sxraq34v7f` is present in stderr. **LEAK CONFIRMED.**
- **Control, correct ms1** — `ms10entrs…` | `me bundle` → exit 3, refusal message only;
  secret body **ABSENT**. Confirms the leak is specific to the mangled-HRP `Classify` path.
- **Corrupted mk1 xpub** — full `mk1qpzg69pp…tfel6q` string echoed verbatim via
  `BundleError::Validate` (`me: invalid string '<full mk1>': mk1 string is not pristine …`).
  **XPUB ECHO CONFIRMED.**
- **Asymmetry confirmed** — the converter path (`me --hex`) does **not** echo: corrupted mk1
  → `me: mk1 string is not pristine: …` (no string); mangled-HRP ms1 → `me: unrecognized HRP
  'msx' …` (no body). So the bundle path is a genuine, asymmetric regression of the "never
  echo the input" discipline the converter deliberately enforces.

### 4. Is the severity honest?
Yes, and it is not overstated.
- This is not a wrong-but-accepted-plate / mis-engraving defect; it is a secret-hygiene
  defect (audit brief class (c), error-text leakage).
- Impact is genuinely high on the ms1 path: the leaked string is the complete codex32 secret
  share; if it reaches a `2>logfile`, shell scrollback captured by tooling, or CI logs, it is
  a direct path to seed recovery / fund loss. This is exactly the invariant the ms1 refusal
  exists to enforce, defeated on a plausible single-character HRP typo.
- Likelihood is discounted (compound: user must feed ms1 to `bundle` at all — a discouraged
  action — AND mistype the HRP). The finder already reflected this by rating it **important**
  rather than **critical** (which they reserve for direct wrong-plate/funds), and by rating the
  md1/mk1-only echo lower. The one-notch gap over the moderate D5-2 (public-xpub) findings is
  principled: this path can leak actual spendable-seed material, strictly worse than any xpub
  disclosure.

## Conclusion
The claim is accurate at the cited location, the failure scenario is reachable and was
demonstrated end-to-end with the built binary, no other layer prevents it, and the "important"
severity is internally consistent and defensible for a funds-safety audit. Confirmed as stated.

Verifier: adversarial verifier #1. Date: 2026-07-06.
