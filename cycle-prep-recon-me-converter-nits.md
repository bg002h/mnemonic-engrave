# cycle-prep recon — 2026-06-16 — me-converter nits (5 slugs)

**Origin/master SHA at recon time:** `1012332`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** (none)

Slug(s) verified: `me-in-stdin-intermediate-zeroize`, `me-validate-ms-unreachable`, `me-decode-text-tlv-comment`, `me-canonical-string-stderr`, `me-go-harness-shortread-loop`. Expectation: near-zero drift — these were filed today against code merged today; the only post-filing change (`0cd48fd`) touched `cross_lang.rs`/`go.mod`, none of the cited files.

---

## Per-slug verification

### me-in-stdin-intermediate-zeroize
- **WHAT:** `read_to_string` (stdin and `--in`) may leave un-zeroized intermediate heap copies; add a clarifying comment / consider a `Zeroizing` read buffer.
- **Citations:**
  - `crates/me-cli/src/main.rs:46-47` "read_to_string (stdin and --in)" — **DRIFTED/IMPRECISE.** `:46` is the `--in` read (`std::fs::read_to_string(path)`) ✓; but the **stdin** read is at `:53` (`std::io::stdin().read_to_string(&mut input)`), not 47. The primary `input` buffer IS zeroized at `:62`.
- **Action for brainstorm spec:** cite both reads (`:46` --in, `:53` stdin) and the zeroize at `:62`; decide between a one-line comment vs reading into `Zeroizing<String>`. Source SHA `1012332`.

### me-validate-ms-unreachable
- **WHAT:** replace `panic!` on `Format::Ms` with `unreachable!(...)` (the invariant: `convert()` filters `Ms` first).
- **Citations:**
  - `crates/me-cli/src/validate.rs:53` `Format::Ms => panic!("validate() called on ms1 …")` — **ACCURATE** (verbatim at :53; doc-comment invariant at :40).
- **Action:** swap `panic!` → `unreachable!`. Trivial. Source SHA `1012332`.

### me-decode-text-tlv-comment
- **WHAT:** `decode_text_tlv` intentionally handles only the 1-byte TLV length form + skips the `0xFE` terminator check; add a comment.
- **Citations:**
  - `crates/me-cli/src/ndef.rs:67-74` `pub fn decode_text_tlv` — **ACCURATE** (function at `:67`; still no explanatory comment; `TLV_TERMINATOR` defined `:9`, used only on the encode side `:55`).
- **Action:** add the scoping comment. Source SHA `1012332`.

### me-canonical-string-stderr
- **WHAT:** spec §5 lists "the canonical validated string" among stderr outputs; the impl doesn't emit it. Reconcile.
- **Citations:**
  - `design/SPEC_seedhammer_engrave.md:78` (§5) — **ACCURATE**: still says the canonical validated string goes to stderr.
  - `crates/me-cli/src/main.rs` — **ACCURATE** (mismatch stands): `grep canonical` → no match; main.rs prints only a byte-count line for `--out`, nothing for stdout/hex/base64. The spec↔impl divergence is real.
- **Action (DECISION ITEM):** either (a) implement — echo the validated string to stderr on success, or (b) amend spec §5 to drop it (not re-emitting input is arguably better hygiene). Pick one in the brainstorm. Source SHA `1012332`.

### me-go-harness-shortread-loop
- **WHAT:** the Go round-trip harness does a single `rr.Read` into a 4096-byte buffer; a short-read loop would be more robust.
- **Citations:**
  - `firmware/ndef-roundtrip/main.go:21-27` single `rr.Read` — **ACCURATE**: `:21` `buf := make([]byte, 4096)`, `:22` `n, err := rr.Read(buf)`. (`io.ReadAll(os.Stdin)` at `:14` reads input fully; the short-read concern is the record read at `:22`.)
- **Action:** wrap `rr.Read` in a loop until EOF/full. Source SHA `1012332`.

---

## Cross-cutting observations
1. **No real drift.** 4/5 citations ACCURATE; 1 (me-in-stdin) is an imprecise line range (stdin read is `:53`, not 47) — content correct, not structural.
2. **All 5 are converter-only** (this repo); none touch the `seedhammer` fork or the upstream PRs (#34/#35).
3. **No lockstep triggers:** no clap flag-NAME changes (me-canonical-string-stderr is additive stderr output / a doc decision, not a new flag), so **no GUI `schema_mirror` and no toolkit manual-mirror** needed; `me` isn't in the toolkit manual yet.
4. **One decision item** (me-canonical-string-stderr: implement vs amend spec) is the only non-mechanical piece.
5. Sync is clean; no cross-pin/version staleness surfaced.

---

## Recommended brainstorm-session scope
- **One small cycle** ("converter polish", ~40–60 LOC total). All 5 slugs are independent and trivially co-shippable.
- **SemVer: PATCH** (`me` is unreleased at `0.1.0`; internal hygiene/comments + one additive-stderr-or-spec decision; no surface break).
- **Ordering:** mechanical four first (validate-ms-unreachable, decode-text-tlv-comment, go-harness-shortread-loop, in-stdin-zeroize), then resolve the canonical-string-stderr decision (the only one needing a call). If (b) chosen there, it's a spec edit — re-run the spec self-review.
- **Mandatory R0 gate** still applies: the brainstorm spec / plan-doc for this cycle must pass an opus architect R0 to 0C/0I before any code.
