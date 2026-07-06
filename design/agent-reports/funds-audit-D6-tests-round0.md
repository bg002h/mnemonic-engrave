# Funds-Safety Audit — Dimension D6: Test Adequacy & Testing-Hardening Plan

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (Rust CLI `me` in `crates/me-cli`;
Go preview sidecar in `preview/`; NDEF round-trip harness in `firmware/ndef-roundtrip`).
Audit date: 2026-07-06. Auditor: D6 finder (test adequacy).
Scope: `crates/me-cli/tests/`, `preview/*_test.go`, in-crate `#[cfg(test)]` modules,
CI config (`.github/workflows/`), Makefiles/justfiles. Read-only w.r.t. repo except this file.

This report seeds next cycle's SPEC, so the hardening proposal (§5) is written to be
implementation-ready: each proposed test names its layer, the exact property it pins,
a harness sketch, and the finding-class it guards.

---

## 0. Executive summary

The **implementation** code is small, disciplined, and well-covered *at the unit level*
for the happy path and the primary refusal/integrity invariants. The **test SYSTEM around
it, however, has a keystone hole and several funds-relevant thin spots**:

1. **No CI runs the tests at all.** The only workflow, `release.yml`, runs `cargo build`
   and `go build` — never `cargo test` or `go test`. Every funds-safety invariant
   (ms1 refusal, byte-exact NDEF, chunk-set integrity, preview-public-only) is guarded
   only by tests that a human must remember to run locally. A regression that breaks any
   of them merges and can be **tagged/released** with the suite red. **(D6-1, critical.)**
2. **The two differential (Rust↔Go) tests silently pass as green no-ops when the Go
   toolchain is absent** — verified live in this environment (see §2). The single strongest
   guard (does SeedHammer's real reader parse what `me` emits?) is vacuous unless Go is
   present *and someone runs it*. **(D6-2, important.)**
3. **One golden `.ndef` vector, one string (24-char md1).** No mk1 golden, no
   length/short-record boundary golden, no full-alphabet glyph-coverage golden, and the
   in-crate round-trip tests decode with `me`'s OWN decoder (blind to a symmetric
   encoder/decoder bug). **(D6-3 / D6-4.)**
4. **The preview SVG/PNG has no output-fidelity golden.** The only "params" golden measures
   geometric *bounds*, which are computed independently of the SVG/PNG walk — so a pen-up↔
   pen-down swap or a dropped-segment mutation in `render_svg.go` renders a wrong preview
   and every test stays green. **(D6-5, preview-divergence class.)**
5. **The md1 chunked/single discriminator + the md-codec-0.36 deviation workaround is
   pinned by only two fixtures and has no version-drift guard**; a misclassification admits
   one chunk of a set as a "complete" standalone plate → a silently incomplete backup.
   **(D6-6.)**
6. **No fuzz or property tests** on the parser/bundle pipeline. **(D6-7.)**

Suites currently PASS. Runtimes are trivially small (see §1), so cost is not an argument
against wiring them into CI or expanding them.

---

## 1. Test inventory (what each test actually pins) + run results

Ran `cargo test -p mnemonic-engrave` and (with `go` placed on PATH from
`/home/bcg/.local/go/bin`) `go test ./...` in `preview/`.

### Rust — result: **all pass**
- `cargo test -p mnemonic-engrave`: lib+unit **41 passed**; `tests/cli.rs` **18 passed**;
  `tests/cross_lang.rs` **1 passed** (SKIPPED when go absent — see §2); `tests/golden.rs`
  **1 passed**; `tests/preview_cross_lang.rs` **1 passed** (SKIPPED when go absent).
  Per-binary runtime 0.00–0.37 s; full run (after build) a few seconds.

| Test (file) | Funds-safety property actually pinned |
|---|---|
| `lib::converts_md1_to_ndef` | convert(md1) → NDEF → **me's own** decode == input. Self-round-trip (see D6-3 caveat). |
| `lib::refuses_ms1` | `convert(ms1)` → `RefusedSecret`. **Core (b) invariant.** ONE ms1 vector, lowercase. |
| `lib::rejects_unknown_hrp` | Unknown HRP → `Classify` err. |
| `lib::flags_plate_overflow_risk` | `exceeds_plate_budget` threshold at 300 chars (advisory only; not a hard limit). |
| `classify::*` (3) | HRP → Format; case-insensitive+trim; unknown/empty-HRP rejects. Good HRP coverage; **no uppercase MS1**. |
| `validate::accepts_valid_{md1,mk1}` | One known-good md1, one mk1 accepted. |
| `validate::rejects_corrupted_md1` | Last-char flip on md1 → BCH reject. **Guards (b): corrupt INPUT can't be engraved.** |
| `validate::rejects_corrupted_mk1` | Last-char flip on mk1 → `MkCorrected` (non-pristine refused). |
| `ndef::encodes_expected_bytes` | Byte-exact TLV for `"md1q"` (4-char). Hard byte anchor #1. |
| `ndef::round_trips` | encode→decode(me) for one 15-char mk1-ish string. Self-round-trip. |
| `ndef::rejects_oversize` | `text_record("a"*255)` → `TooLong`. Exercises the **payload_len** guard, NOT the tlv_wrap boundary (see D6-4). |
| `manifest::*` (4) | serde renames; unchunked-md1 omits chunk fields; preview Some/None; checklist text. Pure-data. |
| `bundle::*` (16) | The integrity core: unchunked vs chunked classification, mk1 set-id grouping, ms1-line refusal, ms1-anywhere early refusal, corrupted-mk1 reject, 2 distinct mk1 sets, mismatched set-ids fail, reordered chunks verify, dropped chunk fails, **duplicate chunk-index fails**, **cross-chunk-hash mismatch fails**, md1 chunked set verify + drop-fails. **Strong for mk1; see D6-6 for the md1-discriminator thinness.** |
| `preview::*` (7) | sidecar filename/discovery, `--version` parse (incl. empty), render writes file & returns path, PNG ext, non-zero exit → `Render`. **`render_plate_writes_file_and_returns_path` asserts the piped body == the exact input string (guards me→sidecar pass-through).** |
| `cli.rs` top-level (10) | md1→hex stdout shape; **ms1 exit 3 + "CODEX32" msg**; missing-mode exit 2; `--echo` to stderr + **stdout purity (no secret bleed)**; no-echo default; bundle manifest on stdout; checklist on stderr; **bundle ms1 exit 3**; **bundle dropped-chunk exit 4 + no stdout**; converter-without-subcommand. |
| `cli.rs preview::` (8) | render-fail→exit 4; matched-version renders + **ms1 never rendered** + exactly 3 svgs; `--png`; version-mismatch→exit 2; absent sidecar→exit 0 + "preview skipped" + no preview keys; no-`--preview`→byte-for-byte Phase-A golden; unwritable dir→exit 2. Hermetic (shell-script fake sidecar). |
| `golden.rs::md1_short_matches_golden` | **The only external byte-anchor** for the engraved payload: convert(24-char md1) == `vectors/md1-short.ndef`. ONE string. |
| `cross_lang.rs` | md1 NDEF parsed by SeedHammer's **real** `nfc/ndef` reader round-trips to input. ONE md1. **SKIPS silently if go absent.** |
| `preview_cross_lang.rs` | Real Go sidecar: each public plate yields non-empty SVG w/ `<svg`+`<path`; ms1 never rendered. Does **not** assert the SVG encodes the RIGHT string. **SKIPS silently if go absent.** |

### Go (`preview/`) — result: **ok, 0.058 s**
`ndefroundtrip` module: **no test files** (exercised only indirectly by `cross_lang.rs`).

| Test (file) | Property pinned |
|---|---|
| `params_test.go::TestParamsGeometryGolden` | bbox of MD1_REF == `wantDx/wantDy`. **Drift guard for replicated `engrave.Params`.** Measures *bounds*, independent of the SVG/PNG walk (see D6-5). ONE string. |
| `layout_test.go` (4) | engraveBest returns a valid mode + ≥1 command; oversize errors; engraveMode forces mode; unknown mode errors. |
| `render_test.go::TestRenderSVGContainsExpectedStructure` | SVG has `<svg`, `viewBox`, **exactly one** `<path`, ≥1 ` C `. Structural only — no path-content golden. |
| `render_test.go::TestRenderPNGValidHeader` | PNG magic + decodes + non-empty bounds. No pixel golden. |
| `version_test.go` (8) | `--version` string; render to file/stdout; PNG to file; forced mode line routing; oversize non-zero; unknown mode/bad format non-zero. |

### CI / build tooling
- `.github/workflows/release.yml` is the **only** workflow. No Makefile/justfile; no
  `Makefile`, `justfile`, or any second workflow (verified by `ls`/`find`).
- release.yml jobs: `go-build` (cross-compile all targets — build only), `rust-build`
  (5-target matrix — build only), `assemble` (tag-only: download, THIRD_PARTY_LICENSES,
  archive, SHA256SUMS, minisign-sign, attest, publish). **No `cargo test`, no `go test`,
  no clippy/vet/fmt anywhere.** The `git grep 'cargo test|go test'` hits are all in
  `design/` docs and one doc-comment in `preview.rs` — never in a CI file.

---

## 2. Live confirmation: the differential tests are green no-ops without Go

`go` is not on the default PATH in this environment. Running the cross-lang test with
`--nocapture`:

```
skipping cross-language round-trip: `go` is not on PATH
test result: ok. 1 passed; 0 failed; ...
```

Both `cross_lang.rs` and `preview_cross_lang.rs` `return` early on `go`-absent and are
**counted as passing**. After exporting `/home/bcg/.local/go/bin` onto PATH they run for
real and pass (0.24 s / 0.37 s). So the guard is real *when exercised* — but nothing forces
it to be exercised, and D6-1 means it never runs in CI regardless.

---

## 3. Gap analysis against funds-safety classes (a)–(d)

- **(a) emitted output differs from validated input** — *Rust NDEF layer:* well-guarded for
  the ONE golden string; **unguarded at the length/short-record boundary (D6-4)** and for
  **mk1 / full-alphabet glyphs (D6-3)**. *Preview layer:* **unguarded** — no path/pixel
  golden; a wrong-geometry render is invisible to the suite (D6-5). *Bundle pass-through:*
  guarded by the manifest golden for one fixture (the verbatim strings are pinned in
  `bundle-md1-mk1.json`).
- **(b) invalid/corrupted input admitted** — strong: per-string BCH validation is tested
  for both md1 (pure verify) and mk1 (pristine-only). ms1 refusal tested on convert + bundle
  + bundle-anywhere. **Thin:** ms1 refusal has one lowercase vector; **the md1
  chunked/single discriminator can misclassify (D6-6)** which is an admission-of-incomplete-
  set risk.
- **(c) secret exposure** — ms1 is refused by HRP *before any decode*, so no ms-codec is even
  a dependency and no secret is ever parsed/rendered. `--echo` stdout-purity is tested; the
  Zeroizing input buffer exists (not directly test-observable, acceptable). ms1-never-rendered
  is tested in both hermetic and real-sidecar preview paths. **No material gap here** beyond
  belt-and-suspenders (a proptest that no ms1-shaped input ever reaches validate/encode/render
  would harden it — folded into D6-7).
- **(d) silent partial failure** — bundle dropped/duplicate/reordered/cross-hash cases are
  tested and `bundle_dropped_chunk_exit_4_no_stdout` asserts **no manifest on failure**. Good.
  `me`'s file writes are non-atomic (`std::fs::write`) but a torn NDEF file is user-visible and
  re-runnable; low. Preview render failure → exit 4 tested.

---

## 4. Mutation-sensitivity spot check (described, NOT applied to the repo)

For each, I reason whether the CURRENT suite catches it. This is the evidence base for the
severities below.

**M1 — `ndef.rs:41` drop first text byte** (`text.as_bytes()` → `&text.as_bytes()[1..]`).
**CAUGHT** by `golden.rs` (byte mismatch), `ndef::encodes_expected_bytes`,
`lib::converts_md1_to_ndef`. Strong.

**M2 — `ndef.rs:48` weaken the TLV length guard** (`message.len() >= 0xFF` → `> 0xFF`).
This permits a 255-byte NDEF message to be wrapped with a **1-byte TLV length of `0xFF`**.
The real SeedHammer reader (`third_party/seedhammer/nfc/ndef/ndef.go:73`:
`if length8 == 0xff { … 2-byte length … }`) then treats `0xFF` as the **2-byte-length
escape** and reads the next two bytes (the record header `D1 01`) as a big-endian length
(0xD101 = 53505) → total misparse of the engraved string. **NOT CAUGHT** — no test constructs
a message anywhere near 255 bytes. Boundary today is safe (max text 249 chars; 250+ →
`TooLong` → exit 4), but the boundary is entirely unpinned. → **D6-4.**

**M3 — `ndef.rs:38` wrong payload length** (`payload_len as u8` → `(payload_len+1) as u8`).
**CAUGHT** by `golden.rs` + `encodes_expected_bytes` (plen byte differs); also breaks
`round_trips`.

**M4 — `bundle.rs:148` flip the chunked-flag bit** (`sym & 0x01` → `sym & 0x02`, or `& 0x03`).
`sym & 0x02`: likely flips at least one of `parses_unchunked_md1_as_bch_only` /
`md1_chunked_set_verifies_and_drop_fails`, so **probably caught**. But `& 0x03` (bit0|bit1)
stays 0 for the unchunked fixture iff its bit1 is also 0, and stays non-zero for chunked
fixtures — such a mask can **escape** because coverage is exactly two hand-built fixtures with
fixed symbol bits. → **D6-6 (thin discriminator coverage).**

**M5 — `layout.go:52` reorder mode preference** (`{"text+qr","text","qr"}` → `{"qr",…}`).
For MD1_REF this changes the chosen mode → `TestParamsGeometryGolden` bbox changes → **CAUGHT
for that one string**. For strings where the reorder picks a *different* fitting mode than the
device would, the preview silently diverges and is **NOT caught** (one-string coverage). → D6-5.

**M6 — `render_svg.go:32` swap pen state** (`if line {` → `if !line {`), or `:29` drop the
`dt==0` skip. Pen-down cubics would emit `M` (jumps) and pen-up would emit `C` (strokes) →
the rendered path is garbage, yet it still contains exactly one `<path`, a ` C `, `<svg`,
`viewBox`. `TestParamsGeometryGolden` measures `bspline.Measure(PlanEngraving(...)).Bounds`,
which is computed from the plan **independent of `renderSVG`'s walk**, so it does not change.
**NOT CAUGHT by any test.** → **D6-5 (no preview output-fidelity golden).**

---

## 5. Prioritized test-hardening proposal (implementation-ready)

Priority order = fix the keystone first, then close the highest funds-relevance gaps.
Each item lists **layer**, **property pinned**, **harness sketch**, **guards**.

### P1 — Wire the suites into CI (guards D6-1; *enables every other test to matter*)
- **Layer:** CI. **Property:** every push/PR runs the full Rust + Go suites; a red suite
  blocks merge and blocks the tag→release path.
- **Harness sketch:** add `.github/workflows/test.yml` (or a `test` job set gating
  `assemble`):
  - `test-rust`: `actions/setup-go@v5` (Go 1.25.10, **so the differential tests are NOT
    skipped**) + `dtolnay/rust-toolchain@1.85.0` + `cargo test -p mnemonic-engrave --locked`.
    Put Go and Rust in the SAME job so `cross_lang.rs`/`preview_cross_lang.rs` build the real
    sidecar. Optionally `cargo clippy -- -D warnings` + `cargo fmt --check`.
  - `test-go`: `go test ./...` in `preview/` and `go vet ./...`.
  - Gate: `assemble.needs` += the test jobs; keep `if: startsWith(ref,'refs/tags/v')` so a
    tag with a red suite never publishes.
- **Guards:** D6-1 (and makes D6-2..D6-7 enforceable).

### P2 — Force-run the differential tests in CI + broaden them (guards D6-2)
- **Layer:** differential/CI. **Property:** the Rust↔Go NDEF round-trip and the real-sidecar
  render are never a silent no-op in CI, and cover more than one md1.
- **Harness sketch:**
  1. Env opt-in: read `ME_REQUIRE_GO=1` in `cross_lang.rs`/`preview_cross_lang.rs`; when set,
     a missing `go` is a **hard `panic!`/assert failure**, not a `return`. CI sets it.
  2. Table-drive `rust_ndef_parses_in_seedhammer_go_reader` over `{short md1, long md1 (~245
     char single descriptor), a valid mk1, a max-length 249-char string, a min-length string}`
     — each must round-trip byte-exact through the real reader.
  3. In `preview_cross_lang.rs`, add an assertion that ties preview to content: render
     `--mode text+qr`, and (cheapest robust option) assert the sidecar's reported mode line
     and re-run the Go engrave for the same string to confirm identical bounds; stronger option
     — decode the QR module matrix the sidecar used and assert it equals the input (requires
     exposing the `qr.Code` matrix via a test hook in the sidecar).
- **Guards:** D6-2, and part of D6-3/D6-5.

### P3 — Expand the golden `.ndef` corpus incl. the short-record boundary (guards D6-3, D6-4)
- **Layer:** golden (byte-exact) + unit. **Property:** byte-exact NDEF for a representative
  spread, and the exact TLV/SR length boundary.
- **Harness sketch:** add `tests/vectors/*.ndef` + a table test:
  - `md1-short.ndef` (exists), `md1-long.ndef` (a ~245-char single md1), `mk1-chunk.ndef`
    (a real mk1 string), `mk1-short.ndef`.
  - **Boundary goldens (XS, highest value):** `boundary-249.ndef` — `encode_text_tlv` over a
    valid-charset 249-char string, byte-pinned; and an assert that a **250-char** string →
    `Err(NdefError::TooLong)`. Explicitly assert the emitted TLV length byte of the 249-case
    is `0xFE`/valid (< 0xFF) so a `>=`→`>` mutation (M2) fails.
  - Decode each golden with an **independent** decoder — reuse the Go harness
    (`firmware/ndef-roundtrip`) rather than `me`'s `decode_text_tlv`, so a symmetric
    encoder/decoder bug can't hide (D6-3 caveat).
- **Guards:** D6-3, D6-4.

### P4 — Glyph-coverage test over the full valid alphabet (guards D6-3, class (a))
- **Layer:** property/unit. **Property:** no character in the bech32 data alphabet
  (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) — nor any HRP char — is dropped, duplicated, reordered,
  or substituted between the validated string and the NDEF text bytes.
- **Harness sketch:** since `encode_text_tlv` is charset-agnostic (raw `str` byte copy), a
  cheap, strong test is: for a string containing **every** bech32 data char plus each HRP
  (`md1…`, `mk1…`), assert `decode_text_tlv(encode_text_tlv(s)) == s` AND that the text-byte
  slice of the encoded output equals `s.as_bytes()` exactly (positional, catches reorder/dup).
  Add a proptest generating random valid-charset strings up to 249 chars asserting the same.
  For the **engraved-glyph** side (device/preview), add a Go test that `sh.Font` has a glyph
  for every bech32 char and that `backup.EngraveText` yields ≥1 pen-down command per glyph
  (catches a silently-skipped glyph in the render path).
- **Guards:** D6-3 (charset), D6-5 (render-side glyph coverage).

### P5 — Preview output-fidelity golden (guards D6-5, preview-divergence class)
- **Layer:** golden + differential. **Property:** the SVG path `d` and the PNG raster for a
  fixed string are byte/pixel-stable, so a walk mutation (M6) is caught.
- **Harness sketch:** in `preview/`, add `render_golden_test.go`:
  - Pin the full SVG string (or a hash of the `d` attribute) for MD1_REF under each mode
    (`text+qr`, `text`, `qr`); regenerate-on-purpose via an `-update` flag.
  - Pin a hash of the PNG bytes (deterministic: fixed canvas, no timestamps) for MD1_REF.
  - Add a pen-state assertion: count `M` vs `C` tokens and assert the ratio matches the
    known pen-up/pen-down structure (a swap flips it).
  - Cross-check: assert `renderSVG` and `renderPNG` walk the SAME pen-down set (e.g. both
    derive from one shared segment iterator helper, tested once).
- **Guards:** D6-5, M5/M6.

### P6 — md1 discriminator + md-codec-pin drift guard (guards D6-6)
- **Layer:** unit + differential. **Property:** the chunked/single discriminator is exercised
  across every branch, and the md-codec-0.36 deviation the workaround depends on still holds.
- **Harness sketch:**
  - Add fixtures hitting all four `parse_line` md1 arms: unchunked (bit0=0), chunked-ok,
    `ChunkHeaderChunkedFlagMissing` branch, and `WireVersionMismatch` → `Md1WireVersion`.
    Vary the first-symbol bits so a `& 0x03`-style mask (M4) fails.
  - **Drift guard:** a test that directly asserts the documented md-codec-0.36 behavior the
    workaround relies on (single md1 → `ChunkHeader::read` returns `WireVersionMismatch{got:2}`,
    and `symbols.first() & 0x01` is the true discriminator). If a patched `0.36.x` changes this,
    the guard fails loudly instead of silently misclassifying. Consider pinning `md-codec =
    "=0.36.0"` (exact) with a comment, or a `Cargo.lock`-checked assertion.
  - **Funds-relevant assertion:** a chunk that is one member of a set must NEVER be admitted as
    a standalone `bch-only` complete plate — add a test feeding a single chunk of a known
    multi-chunk md1 and asserting `SetIncompleteMd` (not a lone `Md1Single`).
- **Guards:** D6-6 (dropped-set-member admission).

### P7 — Fuzz + property targets on the parse/bundle pipeline (guards D6-7)
- **Layer:** fuzz + property. **Property:** no panic on arbitrary input; invariants hold on
  random valid input.
- **Harness sketch:**
  - `cargo fuzz` targets: `fuzz_convert` (arbitrary bytes → `convert` never panics, and any
    `ms`-HRP input is always `RefusedSecret`), `fuzz_run_bundle` (arbitrary multiline → never
    panics; if any line classifies as `ms` the whole run is `RefusedSecret`; a produced
    manifest's plate `string`s are all substrings of the input).
  - `proptest`: for random valid-charset md1/mk1 strings, `decode(encode(s)) == s`; oversize →
    `TooLong`; classify/validate never panic on random UTF-8.
  - Go: `go test -fuzz` on `engraveBest`/`renderSVG` with random ASCII (never panics; oversize
    errors cleanly).
- **Guards:** D6-7 (parser robustness), reinforces (b)/(c).

### P8 — ms1-refusal breadth (guards D6-8, low)
- **Layer:** unit. **Property:** refusal is charset/format-robust.
- **Harness sketch:** table test over `{"MS1…" uppercase, "  ms1…  " padded, mixed-case
  "Ms1…", ms1 as any line position in a bundle, an ms1 with a bad checksum}` — all →
  `RefusedSecret` / exit 3, and none is ever validated/decoded (assert no ms-codec on the path,
  which is structurally true today — a regression test locks it).
- **Guards:** D6-8.

---

## 6. Findings (severity = severity of the GAP)

- **D6-1 (critical):** No CI executes `cargo test` or `go test`; `release.yml` is build-only
  and the tag→release path can ship with a red suite. → P1.
- **D6-2 (important):** `cross_lang.rs` / `preview_cross_lang.rs` pass as green no-ops when Go
  is absent (verified live); the single strongest differential guard is not enforced and covers
  only one md1. → P2.
- **D6-3 (important):** One golden `.ndef` (24-char md1); no mk1/long/boundary/glyph golden;
  in-crate round-trips use `me`'s own decoder (symmetric-bug blind). → P3/P4.
- **D6-4 (moderate):** The NDEF TLV/short-record length boundary (`ndef.rs:48` `>=0xFF`,
  `:38` `payload_len as u8`) is entirely untested; a boundary mutation emits a `0xFF` TLV
  length byte that the real device reader misparses (grounded in `nfc/ndef/ndef.go:73`). → P3.
- **D6-5 (moderate):** No preview output-fidelity golden; the params golden measures bounds
  independently of the SVG/PNG walk, so pen-state/dropped-segment/coordinate mutations render a
  wrong preview with the suite green (preview-vs-device divergence class). → P5.
- **D6-6 (important):** The md1 chunked/single discriminator + md-codec-0.36 deviation
  workaround has two-fixture coverage and no version-drift guard; a misclassification admits one
  chunk of a set as a "complete" standalone plate → silently incomplete backup. → P6.
- **D6-7 (moderate):** No fuzz/property targets on `convert` / `run_bundle` / `classify` /
  `parse_line`. → P7.
- **D6-8 (low):** ms1-refusal coverage is one lowercase vector; uppercase/padded/mixed-case and
  bad-checksum ms1 refusal not table-tested. → P8.

---

## 7. Sound / negative results (checked and found adequate — for the next auditor)

- **ms1 secret handling:** ms1 is refused by HRP **before any decode** (`classify` →
  `RefusedSecret`); `ms-codec` is not even a dependency (`Cargo.toml` deps = md-codec 0.36,
  mk-codec 0.4 only). ms1 is never validated, encoded, or rendered. Tested on convert + bundle
  + bundle-anywhere + hermetic-preview + real-sidecar-preview (ms1 never rendered). No gap
  beyond breadth (D6-8) and a belt-and-suspenders fuzz (P7).
- **Bundle chunk-set integrity (mk1):** dropped, duplicate-index, reordered, mismatched-set-id,
  and cross-chunk-hash-mismatch cases are all tested through `run_bundle` (not just
  `parse_line`), and `bundle_dropped_chunk_exit_4_no_stdout` proves no manifest is emitted on
  failure. This is the strongest part of the suite. (md1-side is thinner — D6-6.)
- **Pristine-input enforcement:** md1 (`unwrap_string`, pure verify) and mk1 (`decode_string`
  with `corrections_applied != 0` → `MkCorrected`) both refuse corrupted/repaired strings, so a
  string that needed BCH repair is never engraved. Tested both directly and through the bundle.
- **me→sidecar pass-through:** `preview::render_plate_writes_file_and_returns_path` uses a
  `cat > out` fake and asserts the piped body equals the exact input string, so `me` piping the
  wrong/altered string to the sidecar would be caught. (The RENDERED-geometry fidelity is the
  residual — D6-5.)
- **`--echo` secret hygiene:** stdout-purity asserted (echo goes to stderr, input never bleeds
  to stdout); the echo line is built only on the success path (post-`convert`), so an ms1 secret
  never reaches the echo allocation.
- **Manifest determinism:** `BTreeMap` grouping + `sort_by_key(index)` give deterministic
  ordering; the manifest golden pins the full structure incl. verbatim strings for one fixture.
- **Exit-code contract:** 0/2/3/4 mapping is unit-tested (`exit_codes_match_spec`) and
  end-to-end-tested across converter and bundle (usage/refused/invalid), incl. preview render
  fail → 4 vs spawn/usage → 2.
- **Suites are fast** (Rust binaries 0.00–0.37 s; Go 0.058 s), so runtime is no obstacle to
  P1/P2 CI wiring or to the expanded corpora.

### Not in scope / not re-reported
- md-codec / mk-codec crate INTERNALS (audited separately) — only this repo's USE is in scope;
  the pin-drift concern is captured as D6-6.
- FOLLOWUPS.md items reviewed; none duplicates a D6 finding. The closest, the fork-side
  `seedhammer-own-code-fix-followups` residual ("`extractSuppliedMd1AndMk1` lacks a fuzz
  target"), is fork code, not this repo. No `knownFollowup=true` items apply to `me`/`me-preview`
  test adequacy.
