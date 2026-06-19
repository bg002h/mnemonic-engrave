<!--
Persisted verbatim. opus-architect R0 gate of the T2a ms1-decode implementation plan
(IMPLEMENTATION_PLAN_seedhammer_T2a_ms1_decode.md @ 2789a8e). Reviewer agentId a553b3a268574bf68.
Verdict: NOT GREEN 1C/1I (3 minor). The reviewer materialized the plan's exact code in a throwaway
worktree off 68e6ead and RAN it: the decoder + display + all 5 parity vectors PASS (incl. the
mnem-Japanese vector, reproduced from the live Rust ms-codec encoder = genuinely Rust-sourced);
DecodeMS1 branches on Seed()[0], the {16..32} guard precedes bip39.New (panic unreachable), refusal
cases exercise the right branches, the pager is gap-free, scope/secrecy/reuse-not-port clean.
C-1: the plan's Button2-on-unshared change HANGS the existing TestConfirmCodex32UnsharedNoRecover —
its ms10tests… secret is plain BIP-93 (Seed()[0]=0x31, not m-format) → DecodeMS1 errors → showError
consumes the queued Button3 → engrave never fires → spin; the plan's "existing tests stay green" was
false. I-1: the Step-4 "Replace" blocks paraphrase rather than quote codex32_polish.go:108-111/116-122.
Minors: M-1 no 24-word paging test; M-2 dev-flavored error text; M-3 TinyGo CI-deferred.
DISPOSITION: folded — C-1 fixed by gating "Show secret" on f.Unshared AND DecodeMS1-success (a
non-m-format unshared secret → affordance hidden, Button2 inert → existing test preserved, no hang;
spec §2.7 refined); I-1 Step-4 now quotes exact anchors; M-1 24-word paging test added. Re-dispatching
R1. The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — T2a ms1 decode (plan)

**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_T2a_ms1_decode.md` (`2789a8e`)
**Spec:** `design/SPEC_seedhammer_T2a_ms1_decode.md` (GREEN R1)
**Method:** Materialized the plan's exact code in a throwaway detached worktree off fork `68e6ead`, built and ran every command the plan specifies (Go 1.26.4 at `/home/bcg/.local/go/bin/go`). Cross-checked the m-format layout and all five parity vectors against authoritative source (ms-codec `consts.rs`, `tests/vectors/v0.1.json`, `tests/mnem.rs:144`), and reproduced the non-pinned Japanese vector from the live Rust ms-codec encoder. Worktree + temp artifacts cleaned up; fork untouched at `68e6ead`.

## Verification Results

**Baseline (`68e6ead`, before changes):** `go test ./codex32/ ./gui/ ./bip39/` → all `ok`.

**Compile:** `go build ./...` → BUILD OK. `assets.IconInfo` exists (`gui/assets/embed.go:98`). All signatures used by `ms1_decode.go` confirmed: `widget.Labelw(*op.Buffer, text.Style, int, color.RGBA, string) (op.Op, image.Point)` (`gui/widget/label.go:16`), `layoutTitle` (`gui/gui.go:1695`), `layoutNavigation` (`gui/gui.go:1724`), `showError(ctx,th,title,msg)` (`gui/slip39_polish.go:22`), `wipeBytes` (`gui/slip39_polish.go:330`), `leadingSize=44` (`gui/theme.go:43`).

**Task 1 — `go test ./codex32/ -run TestDecodeMS1 -v`:** ALL PASS.
- `TestDecodeMS1Parity` (all 5 subcases incl. `mnem-english16`, `mnem-japanese16`) PASS — proves `codex32.New(ms1).Seed()` → `DecodeMS1` yields the asserted prefix/lang/entropy byte-for-byte.
- `TestDecodeMS1Refusal` PASS.

**Task 2 — new GUI tests in isolation (`TestMS1Decode*`, `TestConfirmShowSecretGate`):** ALL PASS (EXIT=0). English words shown; non-English shows "Japanese" name + entropy hex and NOT English words; Button2 on the unshared secret opens the decode view.

**Regression check — `TestConfirmCodex32UnsharedNoRecover` (existing): HANGS → panic after 45s timeout.** Goroutine dump shows the spin in `confirmCodex32Flow` → `layoutNavigation` (`gui/codex32_polish.go:127`), driven from `gui/codex32_polish_test.go:228`. This is the single failure.

**Other regression tests:** `TestConfirmCodex32Unshared`, `TestConfirmCodex32Share`, `TestConfirmCodex32ShareOffersRecover`, `TestRecoverCodex32`, `TestRecoverCodex32Mismatch`, `TestEngraveCodex32BackoutNotUnknown` → all PASS. Full `gui` suite with only `TestConfirmCodex32UnsharedNoRecover` skipped → EXIT=0, no FAIL/panic. `TestAllocs` PASS (1.25s — confirms `confirmCodex32Flow` is not alloc-gated; the append-nav is fine). `go vet ./codex32/ ./gui/` clean. `gofmt -l codex32/ gui/` silent. codex32 + bip39 full suites `ok`.

**Vector provenance (load-bearing):** The five vectors are the exact authoritative values:
- entr16/entr32 + entr20-nonzero match `tests/vectors/v0.1.json` lines 6, 30, 36.
- mnem-English (`ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz`, lang 0, entropy `0c1e24…da1b`) matches `tests/mnem.rs:144` golden (wire `[0x02][0x00][16B]`).
- **mnem-Japanese**: NOT pinned in the corpus, so I ran the live Rust encoder `encode(Tag::ENTR, &Payload::Mnem{language:1, entropy: 0c1e24…da1b})` → output `ms10entrsqgqsc83yukgh23xkvmp59xf2eldpkpefrcjje3drdq`, byte-identical to the Go test's embedded string, and `decode` returns lang=1 + that entropy. Genuinely Rust-encoder-sourced, not Go-self-generated. ✔

**Prefix bytes / language table:** `RESERVED_PREFIX=0x00`, `MNEM_PREFIX=0x02` (`consts.rs:17,39`); `MNEM_LANGUAGE_NAMES` order matches `MSLanguageNames` (`consts.rs:47-58`). ✔

**`DecodeMS1` correctness:** branches on `Seed()[0]` (verified: the `ms10tests…` secret's `Seed()[0]`=0x31 → `errMSBadPrefix`, never the id); entr→`data[1:]`, mnem→lang=`data[1]`, entropy=`data[2:]`; {16,20,24,28,32} guard runs BEFORE any `bip39.New` (diagnostic confirmed 17/18/19/33/64-byte and 18-byte-mnem inputs all return errors, no panic — `bip39.New`'s `len%4!=0`/range panic is unreachable). `NewSeed`→`Seed()` round-trips byte-aligned payloads exactly (diagnostic: `[0x01]+16`→Seed[0]=0x01; `[0x00]+15`→16B; `[0x02,0x0a]+16`→lang byte 0x0a), so the refusal cases exercise the intended branches. ✔

**Pager (`ms1DecodeFlow`):** measure-and-advance mirrors `descriptorAddressFlow`; first line at `i==start` always shown (guarantees ≥1 + forward progress); `start += shown` is gap-free, wraps at end. Crucially it uses the SAME `widget.Labelw` call for measuring and rendering, so no Measure-vs-Labelw height mismatch (safer than the template, which uses `Measure` to size and `Labelw` to render). `lines` is never empty (≥12 words / 4 lines). No skip, no hang in the display flow itself. ✔ (24-word paging not separately exercised by a test, but the logic is sound; see MINOR-1.)

**Scope/secrecy:** ms1 secret is display-only — no engrave/NFC/mutation in `ms1DecodeFlow`; `defer wipeBytes(entropy)` scrubs. md1/mk1 (`mdmkText`) and the engrave path untouched. `git status`: exactly the 5 manifest files (1 modified, 4 new). Reuse-not-port holds (`DecodeMS1` = `Seed()`+slice+length-check; flow reuses `bip39.New`/`LabelFor`). ✔

## Findings

### CRITICAL

**C-1 — The plan breaks (HANGS) the existing `TestConfirmCodex32UnsharedNoRecover`; the plan's claim that "Existing `TestConfirmCodex32*` stay green" is false.**
*Location:* Plan Task 2 Step 4 (the `confirmCodex32Flow` Button2 edit) + Step 5 ("Existing `TestConfirmCodex32*`/`TestRecoverCodex32*` stay green") + File-manifest note "the engrave/back paths… return contract unchanged." The unaddressed test is `gui/codex32_polish_test.go:221-231`.
*Mechanism (verified by running it — panics after a 45s timeout, goroutine spinning in `confirmCodex32Flow`):* That test does `click(Button2, Button3)` on the unshared secret `ms10tests…` and asserts the result is `codex32Engrave` ("Button2 must be inert for an unshared secret"). The plan's edit makes Button2 NO LONGER inert: it calls `ms1DecodeFlow(ctx, th, scan)`. For `ms10tests…`, `Seed()[0]=0x31` → `DecodeMS1` returns `errMSBadPrefix` → `showError(…)` runs its own loop and consumes the queued **Button3** to dismiss (`ErrorScreen.Layout` dismisses on Button3, `gui/gui.go:208-209`) → `continue`. Now no events remain, `engraveBtn` never fires, and in the direct-call (non-`runUI`) context `FrameCallback` is nil so `ctx.Frame` never sets `ctx.Done` → the `for !ctx.Done` loop spins forever. The plan changes the observable Button2 contract for the unshared secret but neither updates nor even mentions this test — it is a hard, non-optional regression (a hang, not just an assertion failure), and it directly contradicts a load-bearing plan claim.
*Fix:* The plan MUST, in Task 2, update `TestConfirmCodex32UnsharedNoRecover` to reflect the new contract (Button2 on the unshared secret now opens the Show-secret sub-flow, it is no longer inert). Options: (a) retarget the test to assert that a lone Button3 (without a preceding Button2) still returns `codex32Engrave`, and add a separate assertion that Button2 enters the decode sub-flow (the new `TestConfirmShowSecretGate` partly covers the latter); or (b) rename/repurpose it. Either way the plan's "existing tests stay green" claim must be corrected, and the test must be edited in the same task as the `confirmCodex32Flow` change. As written, `go test ./gui/` will hang and the build can never reach GREEN.

### IMPORTANT

**I-1 — The plan's Step-4 "Replace" blocks do not textually match the code at `68e6ead`; a literal application is impossible and the intended mapping is under-specified.**
*Location:* Plan Task 2 Step 4 ("Replace the recover-click handling + the nav-append") with its two code blocks.
*Detail:* The plan's first block shows a starting point of a bare `recoverClicked := recoverBtn.Clicked(ctx); if recoverClicked { … }`, but the actual code (`gui/codex32_polish.go:108-111`) is `recoverClicked := recoverBtn.Clicked(ctx)` followed by `if !f.Unshared && recoverClicked { return codex32Recover }`. The plan's second block presents a single-line `navBtns := []NavButton{…}` base, but the actual nav (`:116-122`) is a multi-line literal with an `if !f.Unshared { … }` append. I was able to map the intent unambiguously (and the result compiles, passes the new tests, and keeps the share path green), but the plan gives no exact `old_string` anchors, so a single-implementer executing it literally will not find the quoted text. This is the kind of citation-decay the project's strict-gate guards against. *Fix:* Quote the exact current lines (108-111 and 116-122) as the replacement targets, or phrase Step 4 as precise edits against those line ranges.

### MINOR

**M-1 — No test exercises multi-page word paging (the 24-word case).** The English test uses a 12-word (16-byte-entropy) secret, which fits without paging; `TestConfirmShowSecretGate` likewise. The pager logic is sound by inspection (mirrors `descriptorAddressFlow`, same-call measure/render, first-line-always-shown), but the spec's "24 words page correctly without skipping" property is not directly asserted. Consider a 32-byte-entropy (24-word) display test that pages with Button3 and asserts a later word appears after paging. Non-blocking.

**M-2 — `showError` title "Secret" + body "Can't decode this secret — <err>" leaks the Go sentinel text** (e.g. "codex32: not an m-format secret payload"). Harmless for the unshared-secret happy path (which always decodes), and only reachable for a malformed/unexpected payload, but the message is developer-flavored. Cosmetic; non-blocking.

**M-3 — TinyGo not present locally**, so the `tinygo-device-build` job (compiling `codex32`+`gui` with the new symbols) is deferred to CI, as the plan allows. Regular `go build ./...` passes; no obvious portability hazards introduced (pure byte-slice decode + existing render primitives). Flagging only so the device build is confirmed in CI before merge.

## Verdict

The decoder and display flow are correct, the vectors are authoritative (Japanese vector reproduced from the live Rust encoder), the panic-guard and unshared-only gate hold, the share Recover path is intact, and scope/secrecy/reuse-not-port are clean. But the plan ships a hard regression it explicitly claims it does not: applying it makes `go test ./gui/` HANG on the existing `TestConfirmCodex32UnsharedNoRecover`. That blocks GREEN.

**NOT GREEN — 1 Critical / 1 Important.**
