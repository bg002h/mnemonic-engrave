# `me bundle --preview` Phase B — PLAN architect review (R0 gate)

- **Stage:** mandatory plan R0 gate before any code. Plan written via writing-plans, self-reviewed.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; all code verified against seedhammer v1.4.2 source + existing `me`).
- **Scope:** `design/IMPLEMENTATION_PLAN_me_bundle_phaseB_preview.md` vs `/scratch/code/shibboleth/seedhammer-ref-v1.4.2/` (the submodule's exact source), the GREEN spec, and `crates/me-cli/`.
- **Verdict:** **NOT-GREEN — 2 Critical / 2 Important** (+ 5 Minor). The Critical (SVG walk) would have shipped a broken renderer; caught before any code.

---

## VERBATIM REVIEW OUTPUT

### CRUX 1: The SVG walk pattern (Task 4 — the render crux) — REFUTED (Critical)
The plan emitted one `<path d="M C0.x C0.y C C1…C3"/>` per pen-down cubic. Wrong in two ways: (1) `C0` is the B-spline-derived segment start (= prior `seg.prev` = prior C3), NOT a user anchor — re-specifying it via `M C0` breaks G1 continuity; (2) `continue` on `!engr` discards the pen-UP `M C3` moves that position the cursor for the next pen-down run, breaking multi-run paths. (3) misses the `dt == 0` zero-duration skip.

**Correct pattern (from seedhammer's OWN `internal/golden/golden.go:175-194`, confirmed vs `engrave.go:timeConstantPath:1153-1168`):** a SINGLE `<path d="…">`; on pen-UP (`!line`) emit `M C3.x C3.y`; on pen-DOWN (`line`) emit `C C1 C2 C3` (NO `M`, NO `C0` — cursor is implicit C0); skip `dt == 0`.
- `bspline.Segment` zero-value: CONFIRMED safe (timeConstantPath/ProfileSpline use `var seg bspline.Segment` then `seg.Knot(k)` in-loop, no priming; early calls yield `dt==0` and are skipped).
- `seg.Knot(k) (bezier.Cubic, uint, bool)`: CONFIRMED `bspline.go:47`; 3rd = `s.Knots[1].Engrave` (pen-down). `bezier.Cubic.C0..C3` + `.X/.Y`: CONFIRMED `bezier.go:7-9`.

### CRUX 2: Seq re-range (Tasks 3,4) — CONFIRMED SAFE
`backup.EngraveText` (`backup.go:252`) returns a fresh closure `func(yield …)` — re-rangeable. `engrave.PlanEngraving` (`engrave.go:969`) returns `planEngraving(knotBuf, conf, e)` whose inner closure does `spline := knotBuf[:0]` on entry — re-rangeable. So `fits(eng)` then `renderSVG(eng)` (both re-range) is safe. (Plan's two-PlanEngraving note is valid but overcautious.)

### CRUX 3: go.mod require+replace (Task 1) — NEEDS-CHANGE (Important)
`require seedhammer.com v1.4.2` will fail: the proxy lacks v1.4.2 and `go mod tidy` resolves the require version against the proxy even with a `replace`. The precedent `firmware/ndef-roundtrip/go.mod` uses `require seedhammer.com v0.0.0` + the local replace — the `v0.0.0` sentinel satisfies the require syntax without a proxy fetch. Use `v0.0.0`. Also: seedhammer `go.mod` is `go 1.25.10`; preview should match (tidy may bump it).

### CRUX 4: params/layout/fit/types (Tasks 2,3) — CONFIRMED
`engrave.Params{StrokeWidth int, Millimeter int, StepperConfig}` (embedded) — `params.StepperConfig` works; `StepperConfig{Speed,EngravingSpeed,Acceleration,Jerk,TicksPerSecond uint}` all present; `backup.EngraveText(Params, Text) Engraving`; `backup.Text{Paragraphs []Paragraph, Font *vector.Face}`; `backup.Paragraph{Text string, QR *qr.Code, QRScale int}`; `qr.Encode(s, qr.L)`; `bspline.Measure(curve).Bounds.In(bspline.Bounds{Min:bezier.Pt(...),Max:bezier.Pt(...)})` (`bspline.go:160`, `bezier.go:319`). **`sh.Font` IS `*vector.Face`** (`sh.go:10` `var Font = vector.NewFace(...)`), matches `Text.Font` — no mismatch. Fit Min=`Pt(3*mm,3*mm)`, Max=`Pt(82*mm,82*mm)` matches upstream `gui.go:2476-2477`.

### CRUX 5: Rust integration (Tasks 7,8,9) — CONFIRMED, one constructor-count flag (Important)
`PlateEntry.preview: Option<String>` + `skip_serializing_if` is backward-compatible (None omits; Phase A golden unaffected). `std::env::current_exe()` discovery valid. Exit codes (2 mismatch / 0 degrade / 4 render-fail) consistent with `main.rs:51-54` (EXIT_USAGE=2/OK=0/INVALID=4). **BUT** adding the field breaks ALL `PlateEntry { … }` literals: 4 in `bundle.rs` (md1-singles `:230`, chunked-md1 `:254`, mk1 `:281`, ms1 `:294`) AND test literals in `manifest.rs` (`:116-130`, `:162-193`). Task 7 only lists bundle.rs → build fails to compile until the manifest.rs test literals also get `preview: None`.

### CRUX 6: CI + version baking — CONFIRMED, one Minor
`var version string` (Task 1 stub) is the `-X main.version=$VERSION` target ✓. Release-CI structure matches spec §10. `CGO_ENABLED=0` safe (only CGO is `driver/otp/otp_rp2350.go`, tinygo-gated). `checkout submodules: true` noted. Minor: `preview/go.mod` `go 1.25` vs submodule `go 1.25.10`.

## Issues
### Critical
**C-1 — Task 4: SVG walk fundamentally wrong.** Emits `M C0 C C1 C2 C3` per pen-down cubic; correct is a SINGLE `<path>` with pen-up→`M C3`, pen-down→`C C1 C2 C3` (C0 implicit), skip `dt==0` (per `internal/golden/golden.go:175-194`). Naive `M C0` breaks B-spline G1 continuity + multi-run paths. **Fix:** rewrite `render_svg.go` to the single-path accumulated pattern.
### Important
**I-1 — Task 1: `require seedhammer.com v1.4.2` fails `go mod tidy`** (proxy lacks v1.4.2). **Fix:** `require seedhammer.com v0.0.0` (the ndef-roundtrip precedent).
**I-2 — Task 7: `manifest.rs` test `PlateEntry` literals omitted** from the `preview: None` update list → won't compile. **Fix:** list all sites in bundle.rs (4) + manifest.rs tests.
### Minor
m-1 Task 2 test missing `seedhammer.com/engrave` import. m-2 `preview/go.mod` use `go 1.25.10`. m-3 double-`Measure` in renderSVG (safe, optional). m-4 ms1 guard (already correct, no action). m-5 document windows/arm64 omission in release README.

## Assessment
TDD/decomposition sound (red→green→commit, bite-sized; Task 11 CI correctly uses a PR build-only dry-run since CI can't be fully run locally). Spec coverage map holds. Generate-and-pin artifacts (geometry-golden consts, minisign.pub) correctly flagged, not placeholders. Cross-lang test mirrors `cross_lang.rs` auto-skip. No over-build (gonum trade-off acknowledged; sidecar Rust uses stdlib only).

## Verdict: NOT-GREEN — 2 Critical / 2 Important
(Counted C-1 once; the report body labeled it "2 Critical" reflecting its two structural sub-faults — treated as the single finding C-1.) Fix C-1 (SVG single-path), I-1 (go.mod v0.0.0), I-2 (manifest.rs test literals); fold the minors. Then R1.

---

## Fold plan (main session) — ALL FOLDED
- **C-1** → Task 4 `render_svg.go` rewritten to the single-`<path>` pattern (pen-up `M C3` / pen-down `C C1 C2 C3` / skip `dt==0`), citing `golden.go:175-194`.
- **I-1 / m-2** → Task 1 go.mod `require seedhammer.com v0.0.0` + `go 1.25.10`; Step 4 hedge hardened.
- **I-2** → Task 7 now updates ALL `PlateEntry{}` sites (bundle.rs 4 + manifest.rs test literals).
- **m-1** → Task 2 test adds `seedhammer.com/engrave` import.
- **m-3** (safe/optional) + **m-4** (already correct) + **m-5** (Task 11 already notes win/arm64) — no change needed.
Re-dispatch R1 to converge — confirm the C-1 rewrite matches `internal/golden/golden.go`.
