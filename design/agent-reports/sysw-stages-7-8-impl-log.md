# Implementation log — systemwide payloads, plan stages 7 and 8

**I implemented these two stages.** This project has nine implementation logs and
had none for systemwide payloads, so until now there was no record of who built
that feature. This is the record for stages 7 and 8 only; stages 1–6 and 9–13
have no such log, and that gap is still open.

- **Date:** 2026-08-12
- **Agent:** Claude Opus 5 (1M context), dispatched as the single implementer
- **Plan:** `design/IMPLEMENTATION_PLAN_systemwide_payloads.md`, stages 7 and 8
- **Normative rule:** `[mdmk-decode]` — `SPEC_systemwide_payloads.md` §12.6,
  ruled into existence 2026-08-12 as §13 D6; flag consequence in §3.3.3
- **Commits:**
  - `mnemonic-engrave` `2b570fc` — stage 7 (from `c49199b`)
  - `seedhammer` `0775e9e` — stage 8 (from `b14662a`, branch `sysw-port`)

Both trees end clean. Nothing was flashed; there was no hardware step in either
stage.

---

## What the rule is

A `ClassMDMK` record is DECODE-CONFIRMED when the payload's own `ClassMDMK`
records contain the complete card set it belongs to **and** that set reassembles
and decodes by the format's real decoder. Anything else — incomplete set,
reassembly failure, decode failure, unreadable card identity — leaves it
UNCONFIRMED, and for flag evaluation an unconfirmed record counts as SECRET.

Nothing is refused. §13 D6 demoted the refusal §5.3.2 used to carry, and a lone
card of a chunked set is exactly what `bundleFlow` legitimately seeds with.

---

## What I changed

### Stage 7 — Rust primary (`mnemonic-engrave`)

| file | change |
| --- | --- |
| `crates/me-cli/src/sysw/record.rs` | `mdmk_unconfirmed(&[String]) -> Vec<usize>`, plus a private `chunk_key` mirror; 9 new tests |
| `crates/me-cli/src/sysw/coverage.rs` | §8.3 test 14 re-pointed to `Vector("S-J")`; vector S-J added with its cards; the derived-vector test now pins that S-I is no longer *required* |
| `crates/me-cli/src/sysw/vectors.rs` | `Vector` gains `mdmk_unconfirmed`; a new test recomputes it from the BLOB and refuses a set that carries only one answer |
| `crates/me-cli/testdata/sysw_vectors.json` | regenerated — new field on all 8 vectors, new S-J |
| `crates/me-cli/src/main.rs` | `pack` warns once per unconfirmed record and proceeds; `show` prints `confirmed`/`unconfirmed` per `ClassMDMK` record |
| `crates/me-cli/tests/sysw_cli.rs` | 4 CLI tests; plus a pre-existing clippy fix (below) |

### Stage 8 — Go port (`seedhammer`)

| file | change |
| --- | --- |
| `sysw/confirm.go` | **new.** `MDMKUnconfirmed(records []string) []int` + `cardKeyOf` |
| `sysw/confirm_test.go` | **new.** 9 tests, mirroring the primary's |
| `sysw/conformance_test.go` | the vector struct gains `mdmk_unconfirmed`; `TestConformanceMDMKDecode` recomputes it from the blob for every vector |
| `gui/sysw_session.go` | `syswRecord.unconfirmed`, set once at `load` |
| `gui/sysw_admit.go` | `syswFlags(c, unconfirmed, src, sealed, weak)` — secrecy joined in one place |
| `gui/sysw_load.go` | `syswLoadWarnings` de-duplicates by `(flag, cause)` and names the unconfirmed case distinctly |
| `gui/sysw_admit_test.go`, `gui/sysw_confirm_test.go` | signature updates + 7 new tests |

Order was Rust-first, as required: stage 7 was committed with vector S-J before
any Go file was touched.

---

## Gates, run by me

**Rust** (`mnemonic-engrave`), whole run grepped for `FAILED` → 0 hits:

```
lib 204 ok | main 1 | cli 30 | cross_lang 1 | golden 3 | preview_cross_lang 1
prop 6 | seal_cli 14 | sysw_cli 28 | doc-tests 0     — 0 failed anywhere
cargo clippy -p mnemonic-engrave --all-targets -- -D warnings: Finished, clean
```

**Go** (`seedhammer`):

```
SYSW_REQUIRE_VECTORS=1 go test ./sysw/ ./gui/
  ok  seedhammer.com/sysw  0.040s
  ok  seedhammer.com/gui   56.629s
go build -tags tinygo ./gui/            exit 0, no output
GOOS=js GOARCH=wasm go build ./cmd/emu/ exit 0, no output
gofmt -l .                              no files listed
```

`go vet ./gui/` still fails on the PRE-EXISTING go1.26 issue in
`freetext_sizeproof_golden_test.go:111`. Not mine, not touched.

---

## Mutation testing, and the trap it caught

Every mutant below carried a print marker **on the mutated line**, run with
output uncaptured, so a surviving mutant could be told apart from an unexecuted
one. That distinction is the whole point and it earned its keep immediately.

**Rust (6):** group by HRP alone; collapse the non-chunked group key; report
filtered positions; non-chunked md1 always confirmed; mk1 always confirmed; drop
the fail-closed arm. All killed, all markers fired — M1's with real chunk-set ids
(398802, 841149, 74565, 153721).

**Go (7):** the same seven, killed, markers fired.

**GUI (4):** stop reading secrecy through §12.6; never mark the record; dedupe by
flag alone — all killed. The fourth, computing over `Public` only, **survives and
is a true equivalent mutant**: `ClassMDMK` is not a secret class, so every
`md1`/`mk1` is in the public section in both container variants. The code passes
both lists anyway so no index arithmetic is needed, and says so in a comment.

**The one that mattered: Go mutant G2 survived on the first run.** The
two-non-chunked-cards test used the `0xAB` smuggled-entropy record, which sets
the chunked flag with a wrong wire version. Rust treats *any* chunk-header read
failure as "not chunked", so that record does reach the grouping there — but
`md.ParseChunkHeader` consults the `syms[0]&1` discriminator first and then
errors, so in Go the record is reported from the fail-closed arm and **never
reaches the grouping at all**. The test read as coverage of a line it never
executed. Both sides now use a `0x01` fill for that test, which leaves the flag
clear so the two take the same path; `0xAB` is kept for the smuggling test, where
the divergent routes are the point.

---

## Things I got wrong

**I claimed the fail-closed arm was unreachable, in a code comment and in a
commit message.** It is reachable. `validate_record` trims before validating
while the decoders are handed the record as given, so ` md1…` (leading space)
classifies `MdMk` and then fails `unwrap_string`. I only found out because I sat
down to write the input I had just said could not exist. Now tested on both
sides, and the comment says so. `decode_public_set` has the same asymmetry and
gives the same answer, which is why the two walks still agree.

I amended the stage-7 commit rather than stacking a correction on top, because
nothing depended on it yet and the false claim would otherwise have been
permanent. The amended message states the correction plainly.

---

## Things I was unsure about — read this part

### 1. The plan's S-J, as literally written, is degenerate

The plan says: *"S-J: a single chunk of a declared multi-chunk `md1` set … Expected:
classifies `ClassMDMK`, unconfirmed."* The existing `MD1` fixture that every
other sysw vector already uses **is** chunk 0 of 3 of set 398802 — measured, not
assumed. So a vector with `records: [MD1]` would be **byte-identical to S-I**,
and useless: a vector in which every record answers "unconfirmed" cannot fail an
implementation that answers "unconfirmed" to everything.

I built S-J as eight real cards carrying **both** answers — a complete 3-chunk
`md1` set, a lone chunk of a different `md1` set, a non-chunked `md1`, a complete
2-chunk `mk1` card, and a lone chunk of a different `mk1` card; expected
`[3, 7]`. That satisfies the plan's sentence (record 3 is exactly the card it
describes) and is able to fail. **But it is more than the plan asked for, and if
the reviewer wanted the minimal vector, this is where to push back.**

### 2. "both directions" in stage 8 is ambiguous

Stage 8 says *"conformance: S-J through the shared vectors file, both
directions"*. I read that as *confirmed and unconfirmed are both pinned*. It
could instead mean *host→device and device→host*, which for a read-only
conformance harness has no obvious second direction. Flagging rather than
resolving.

### 3. The grouping is DUPLICATED, and I chose that

`seal::record::chunk_key` is private and `seal/` is frozen and explicitly out of
scope, so I mirrored the grouping into `sysw/record.rs` rather than widening the
frozen module's visibility by one word. The plan's file table lists only
`sysw/record.rs`, which supports that reading. Two implementations of one rule is
the exact defect shape this cycle has been fighting, so the copy is **watched**:
`the_two_walks_agree_wherever_both_have_an_answer` runs eleven record sets
through both and asserts `decode_public_set(..).is_ok() ==
mdmk_unconfirmed(..).is_empty()`, refusing to pass unless both answers occur.
**If a reviewer prefers `pub(crate) fn chunk_key`, the duplication goes away and
that test becomes unnecessary. I did not make that call unilaterally.**

### 4. The vector's index basis is my decision, not the plan's

`mdmk_unconfirmed` in the JSON is **indices into the PUBLIC SECTION**, not into
`records`. The public section is the one list both implementations reconstruct
identically from `blob`; `records` is the primary's packing order, which a sealed
payload never reveals. It costs nothing today because `ClassMDMK` is never
secret. The plan did not say which, and both sides now depend on this choice.

### 5. Two commands, two index bases

`me sysw pack` reports **argv** indices (the list the operator can act on);
`me sysw show` reports **public-section** indices (the only list it can see).
For an unsealed payload with no secret records these coincide; for a sealed one
they do not. Deliberate, documented at both sites, and a plausible source of
operator confusion I could not design away.

### 6. The vectors JSON gained a field

That is a fixture format change. Rust regenerated it; the Go struct was updated
in lockstep; `#[serde(default)]` and Go's tolerance of unknown fields mean a
stale consumer degrades to "nothing unconfirmed" rather than failing loudly.
**That is the wrong direction for a safety field**, and I left it that way
because a hard failure would break any consumer that has not been updated. Worth
a second opinion.

### 7. Clippy was already red at `c49199b`

`manual_repeat_n` in `pack_enforces_the_passphrase_length_bound` — a newer clippy
on code I did not write. Verified pre-existing by stashing my work and re-running.
I fixed it inside the stage-7 commit because the named gate could not otherwise be
run at all. That breaks "unrelated changes go in a third commit"; it is named in
the commit message and here rather than left to be discovered.

### 8. F3 and F4 are still unfired in production

`syswFlags`' only production caller is `syswLoadWarnings`, which renders F1 and
F2. So the new `unconfirmed` input to F4 is exercised only by tests. Stage 10
owns the sites that construct `srcNFC`. This is the plan's sequencing, not a gap
I introduced — but it means the F4 path has never run on a device.

### 9. What I did not measure

- **Firmware size.** `sysw` now imports `md` and `mk`. I checked with
  `go list -deps -tags tinygo ./cmd/controller/` that **both were already
  firmware dependencies before this change**, so no new package reaches the
  binary — but I did not run a real TinyGo compile or compare image sizes. The
  gate is `go build -tags tinygo`, which is a type-check, not the firmware build.
- **Cost at load.** The confirmation walk runs the real decoders once per card
  set at load time, on the device. I did not time it. A payload with many card
  sets pays for all of them before the load screen appears.
- **Anything on hardware.** Neither stage was flashed.
