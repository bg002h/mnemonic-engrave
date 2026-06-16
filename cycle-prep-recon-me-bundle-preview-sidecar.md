# cycle-prep recon — 2026-06-16 — me-bundle-preview-sidecar (Phase B)

**Origin/master SHA at recon time:** `8993579`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `(none)`
**Cross-repo refs:** seedhammer upstream ref `/scratch/code/shibboleth/seedhammer-ref-v1.4.2` (v1.4.2, module `seedhammer.com`); fork `/scratch/code/shibboleth/seedhammer` @ `6ab12c0` (branch `feat/engrave-mdmk`, PR #35).

Slug verified: `me-bundle-preview-sidecar`. The Phase-B FOLLOWUP carries `DESIGN_me_bundle_preview.md` §B (B1/B2/B3), which already passed the combined design R0→R1. This recon re-confirms its citations against current source and pins SHAs, and surfaces the one load-bearing spec decision (host-side `engrave.Params`).

---

## Per-slug verification
### me-bundle-preview-sidecar
- **WHAT (from FOLLOWUPS.md):** A `me-preview` (Go) sidecar renders ONLY a validated public md1/mk1 string + plate mode → `engrave.Engraving` → image (SVG primary). `me` (Rust) does all validation. Sidecar has no secrets, no network. Pins UPSTREAM v1.4.2. Delivered in a bundled signed per-platform release archive; `me` checks `me-preview --version` before invoking. (B1 trust/upstream-pin; B2 faithfulness/SVG; B3 delivery/version-check.)

- **Citations:**
  - `firmware/ndef-roundtrip/go.mod` replace pattern — **ACCURATE.** `replace seedhammer.com => ../../../seedhammer-ref-v1.4.2`; the v1.4.2 ref is PRESENT. The sidecar's go.mod reuses this pattern.
  - **Upstream v1.4.2 rendering primitives (what the sidecar pins)** — **ALL ACCURATE** (`seedhammer-ref-v1.4.2/`):
    - `backup.EngraveText(params engrave.Params, plate Text) engrave.Engraving` — `backup/backup.go:252`; `backup.Text` `:32`; `backup.Paragraph` `:37` (fields incl. `Text`, `QR *qr.Code`, `QRScale int`).
    - `engrave.Params` `engrave/engrave.go:38` (fields incl. `StrokeWidth`, `Millimeter`, `StepperConfig`; scaling via `F`/`I`); `engrave.Engraving = iter.Seq[Command]` `:55`; `engrave.Command` `:57`; `Command.AsDelay()` `:78`; `Command.AsKnot()` `:87` (returns unexported `splineKnot` — usable structurally from an external pkg); `engrave.QR(strokeWidth, scale, *qr.Code) Engraving` `:277`.
    - `font/sh` PRESENT (`sh.go`, `sh.bin`, `gen.go`) — `sh.Font` is the face `validateMdmk` uses.
  - **QR package** — **ACCURATE (external dep, pinned):** `qr "github.com/seedhammer/kortschak-qr"` (`backup/backup.go:10`), pinned `github.com/seedhammer/kortschak-qr v0.3.2` in `seedhammer-ref-v1.4.2/go.mod:13`. `qr.Encode(s, qr.L) -> *qr.Code`; the sidecar requires this dep at the same version.
  - **Fork `validateMdmk` layout PARAMS (the sidecar's reference)** — **ACCURATE** (`seedhammer/gui/gui.go` @ `6ab12c0`): `func validateMdmk(params engrave.Params, s string)` `:1746`; `qr.Encode(s, qr.L)` `:1747`; `const qrScale = 3` `:1751`; three `textEngraving` modes `:1756-1759` — `"TEXT + QR"` {Text, QR, QRScale}, `"TEXT"`, `"QR ONLY"` {QR, QRScale}; `backup.EngraveText(params, plate)` `:1769` with a `toPlate` fit check.

- **Action for brainstorm/spec:** Citations accurate; cite **me-repo `8993579`**, **v1.4.2 ref**, and **fork `6ab12c0`**. The sidecar **pins UPSTREAM v1.4.2** (`backup`+`engrave`+`font/sh`+`kortschak-qr`, NOT `gui`) and **reimplements `validateMdmk`'s logic** (qr.L, scale 3, 3 modes, `toPlate` fit) against those upstream libs — it does NOT import the fork/gui. So it is decoupled from PR #35's merge status (B1 claim confirmed).

---

## Cross-cutting observations
1. **★ LOAD-BEARING SPEC DECISION — host-side `engrave.Params` source.** `validateMdmk` takes `params` from `ctx.Platform.EngraverParams()` (a `gui.Platform` method, `gui/gui.go:2388`). The CANONICAL device values are `engraverParams = engrave.Params{ StrokeWidth: strokeWidth, Millimeter: mm, StepperConfig: engraverConf }` defined in **`cmd/controller/platform_sh2.go:402` — which is `//go:build tinygo && rp` gated, NOT host-compilable.** A host copy also exists in `gui/gui_test.go:355` (a `_test.go`, not importable). ⇒ The sidecar (host Go, no gui/driver) **cannot import the params**; it must **replicate the exact `strokeWidth` / `mm` / `engraverConf` constant values** from `platform_sh2.go` so the preview matches what the device engraves. The Phase-B spec MUST: (a) pin those exact values (read from `platform_sh2.go` @ v1.4.2) into the sidecar, (b) state the fidelity contract ("matches the SH2 device given these replicated params"), and (c) add a guard/test flagging drift if a future seedhammer ref changes them. This is the #1 fidelity risk.
2. **SVG fidelity (design m-5).** Walking the `Command` stream: `AsKnot()` returns `splineKnot{ Engrave, Knot(bezier.Point X/Y), Multiplicity }`. B-spline `ControlPoint` knots (multiplicity ≠ clamping) must be **interpolated**, not drawn as straight segments, or fonts mis-render. Spec must declare the fidelity target: exact B-spline evaluation vs documented-approximate (line-segments). (Design §B2 already flags this; it is a spec decision, not a blocker.)
3. **`--version` staleness mechanism (design I-4 / B3).** Spec must define what `me-preview --version` emits (a version/commit string) and how `me` compares it to an expected pin (compiled into `me`?) before invoking — mismatch → warn/refuse (never silent stale-layout render); absent + `--preview` → graceful degrade (manifest+checklist still emitted, per Phase A).
4. **Delivery / supply chain (design B3).** Bundled per-platform signed release archive (`me` + `me-preview` + `SHA256SUMS` + signature); cross-platform CI matrix; NO runtime network. The Go sidecar is a release-only artifact (cannot ship via crates.io). Spec must define the archive layout, the CI matrix (linux/macos/windows × amd64/arm64), and the signing approach.
5. **`me`-side integration.** Phase A's `me bundle` currently has no `--preview`. Phase B adds `--preview <dir>` (+ optional `--png`) to the `bundle` subcommand: for each public plate, invoke `me-preview` to render `out/plate-N.svg`; `ms1` not rendered. The manifest's `plates[]` already carries enough (string + kind + index) for `me` to drive the sidecar — confirmed forward-compatible in the Phase-A spec-R1 review.

---

## Recommended brainstorm-session scope
- **Brainstorm is largely SETTLED** by the Phase-A Q&A (full scope, prebuilt sidecar, bundled signed archive, no network) + the design §B R0→R1 GREEN. Remaining genuinely-open decisions for the spec: (i) SVG exact-vs-approximate fidelity (obs 2); (ii) the `--version` pin mechanism (obs 3); (iii) whether `--png` is in v0.3.0 or deferred; (iv) the exact `engrave.Params` replication + a drift guard (obs 1). Resolve (i)–(iv) in a short brainstorm confirmation, then write the spec.
- **One Phase-B cycle**, but the spec spans three sub-systems with different risk: **(B-core)** the Go sidecar rendering (params replication + EngraveText + SVG walker) — the fidelity crux; **(B-integ)** `me bundle --preview` wiring + `--version` check + graceful degrade (Rust); **(B-rel)** signed cross-platform release CI. Sizing: B-core ~150–300 LOC Go + a cross-lang fidelity test; B-integ ~60–100 LOC Rust + tests; B-rel = CI/release config (no app LOC). If the spec feels too broad at R0, split B-rel into its own follow-on (the sidecar + `me` integration is the shippable unit; signed-release automation can land second).
- **SemVer:** new `me-preview` binary + `me bundle --preview` flag = **MINOR**, `me` → **v0.3.0**. The Go sidecar is versioned in lockstep with `me` (the `--version` pin).
- **Lockstep flags:** no GUI `schema_mirror` (no SeedHammer change — sidecar only READS upstream libs). No toolkit-manual mirror yet (`me` undocumented there). If a manual is added, `me bundle --preview` mirrors. The sidecar's seedhammer pin (v1.4.2) is a new cross-pin to record; bump deliberately (re-verify the replicated params on any bump — obs 1).
- **Mandatory next gate:** the Phase-B spec AND plan each MUST pass an opus-architect R0 → 0C/0I before any code (project standard, [[iterative-architect-review-standard]]). The fidelity claims (params replication, SVG B-spline, qr params) are the things R0 must verify against source.

---

### Summary
All Phase-B FOLLOWUP citations verify ACCURATE against me-repo `8993579` / v1.4.2 ref / fork `6ab12c0`: the sidecar's upstream rendering primitives (`backup.EngraveText`, `engrave.{Params,Command,AsKnot,QR}`, `font/sh`, external `kortschak-qr v0.3.2`) all exist in v1.4.2, and the fork's `validateMdmk` confirms the exact layout params (`qr.L`, `qrScale=3`, 3 modes) the sidecar replicates. **No drift, no structural errors.** The one load-bearing spec decision the recon surfaces: the canonical `engrave.Params` values live in a **TinyGo-gated** `cmd/controller/platform_sh2.go` (not host-importable), so the sidecar must **replicate** them — this is the central fidelity risk the Phase-B spec must pin (exact values + drift guard + fidelity contract). Brainstorm is mostly settled; resolve SVG fidelity / `--version` mechanism / `--png` scope / params-replication in a short confirmation, then spec → R0 → plan → R0 → implement.
