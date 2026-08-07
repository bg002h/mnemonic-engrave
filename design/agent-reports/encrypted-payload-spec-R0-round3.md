# R0 round 3 — `SPEC_encrypted_payload_delivery.md` (bundle + session + ms1 additions)

Reviewer: opus, design-level adversarial review scoped to the material added
after round 2 (bundle container, session model, `ms1` admission).
Dispatched 2026-08-07.
Verdict: **1 Critical / 4 Important / 3 Minor / 1 Nit — GATE BLOCKED.**

Model tiering: fable was spent on the core crypto construction at round 1 and
that construction did not change. The new material is a container format, a
session model and a policy change — design-level, hence opus.

Persisted verbatim. Full finding text is reproduced in the controller's fold
notes below; the reviewer's own numbering of Important counts (4) differs by one
from the findings as listed (3) — immaterial, every finding was folded.

## Controller-verified before folding

- **CRITICAL (vector C display form)** — CONFIRMED by execution against the real
  `seedhammer.com/codex32` package:
  `SPACED len=80 New_err=codex32: invalid character ValidMD=false ValidMK=false`
  vs `CANON len=67 ValidMD=true`. Vector C as shipped would have been rejected
  on every record. Canonical rebuild independently recomputed and matches the
  reviewer's figures on all four values (plaintext sha256, tag, blob sha256,
  header hex).
- **IMPORTANT (record_count cap)** — CONFIRMED by measurement:
  2-of-2 `wsh-sortedmulti` = 10 records, 2-of-3 = 15. A cap of 7 rejected every
  multisig wallet. Also confirmed `bundleReviewFlow` (`gui/bundle_flow.go:224`)
  ships a paged arbitrary-length list via `pageBtn := &Clickable{Button: Button2}`,
  falsifying §12 item 7's claim that no such widget existed.
- **MINOR (idle timeout)** — CONFIRMED `idleTimeout = 3 * time.Minute` exists at
  `gui/gui.go:2801`; the "timer source must be identified" deferral was wrong.

