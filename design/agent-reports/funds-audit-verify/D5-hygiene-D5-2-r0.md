# Adversarial verification — D5-2 (verifier #0)

**Finding:** Output artifacts (NDEF `--out`, manifest, preview SVG/PNG) written
world-readable 0o644, containing/depicting privacy-critical md1/mk1 material; no
0o600 tightening, no runtime warning.
**Finder severity:** moderate
**Verdict:** CONFIRMED (refuted = false). Severity adjusted down to **low** for the
funds-safety framing (see below). Confidence: high on the facts.

## What I verified

### 1. The cited write sites behave exactly as claimed
- `crates/me-cli/src/main.rs:140` — `std::fs::write(path, &bytes)` (NDEF `--out`).
- `crates/me-cli/src/main.rs:205` — `std::fs::write(path, json.as_bytes())` (manifest).
- `preview/main.go:128` — `return os.WriteFile(path, payload, 0o644)` (SVG/PNG `--out`).

These are the only production write sinks (`grep fs::write|WriteFile|create(`):
the other `fs::write`/`set_permissions` hits are all in tests / the fake-sidecar
harness (`crates/me-cli/tests/cli.rs:169-234`, `preview.rs:250-256,350-354` — those
`set_permissions` calls set the **exec** bit on a stub `me-preview` script for
tests; they do NOT tighten output artifacts). No `OpenOptions().mode(0o600)`, no
`Chmod`, no `umask` manipulation anywhere in production.

### 2. Probe — actual mode under the environment umask
Environment umask = `022`. Compiled and ran a scratch Rust probe OUTSIDE the repo:
```
std::fs::write mode = 644
group/other bits (m & 0o077) = 44        # world+group READ set
```
So `std::fs::write` (main.rs:140/205) yields `-rw-r--r--` under the default umask —
matching the finding. Go was not installed, but `os.WriteFile(path, payload, 0o644)`
passes the literal mode `0o644`; with umask 022 that is `0o644 & ~022 = 0o644`
(well-documented POSIX behavior). Claim holds for the SVG/PNG too.

### 3. The artifacts really contain / depict the wallet's public material
- Manifest embeds every raw md1/mk1 string: `bundle.rs:234,260,287` set
  `string: Some(s.clone())`. The preview SVG/PNG is a faithful, scannable render of
  that same payload (QR + text). Confirmed.
- ms1 is excluded and never rendered: `bundle.rs:301` sets `string: None`, and
  `main.rs:271-277` `continue`s on `PlateKind::Ms1`. So **no seed / spendable key**
  ever reaches these files. (This is exactly what caps the impact.)

### 4. Failure scenario reachability
A valid md1/mk1 bundle passes upstream validation, and `--manifest` / `--preview`
then write these files at 0o644 with no warning printed (`main.rs:209` prints only
"wrote manifest to …"; `main.rs:280` only "rendered plate N → path"). No layer
tightens perms or warns. The scenario is reachable.

## Severity assessment (why I adjust moderate → low)

The claim is factually accurate — this is a genuine hygiene gap worth fixing
(0o600 + a one-line "these files depict your wallet's public keys" warning). But in
a **funds-safety** audit the honest severity is low, because every impact-limiting
condition applies at once:

- **No funds / no seed / no wrong-plate.** The finder itself concedes "privacy, not
  spend"; ms1 is `string:None` and never rendered (double-guarded). This produces
  neither a wrong-but-accepted plate nor any spendable-key exposure.
- **Public-by-design data.** README:3,17 documents md1/mk1 as the **public** backup
  strings. Disclosure of xpubs+descriptor is a real privacy loss (full watch-only
  wallet history + forward addresses), but it is public-tier material, not secret.
- **Doubly conditional.** Harm requires (a) a shared/multi-user host — not the
  typical single-user desktop backup-engraving workflow — AND (b) umask 022. Hardened
  multi-user hosts (where the threat is real) frequently run umask 077, which already
  yields 0o600 for these same `fs::write`/`0o644` calls. The world-readable outcome
  is an environment property, not an intrinsic one.
- **User-chosen paths.** All three artifacts go to explicit `--out` / `--manifest` /
  `--preview DIR` locations the user picks; default-umask perms on user-directed
  output is conventional CLI behavior.

Moderate (the finder's rating) is internally coherent with their own scale
(xpub-disclosure > stale-image D5-3), so it is not dishonest — but against the
brief's yardstick ("would it really produce a wrong-but-accepted plate / lost
funds?" → no) and given the umask/shared-host conditionality plus the public-by-
design nature, **low** is the more honest funds-safety severity.

## Bottom line
- refuted = **false** (the code demonstrably writes world-readable artifacts of the
  wallet's public material; no mitigating layer; probe-confirmed).
- adjustedSeverity = **low** (real privacy hygiene gap; zero funds/seed/plate-
  correctness nexus; conditioned on shared host + default umask).
- confidence = **high** on the facts; the moderate→low move is a calibration judgment.
