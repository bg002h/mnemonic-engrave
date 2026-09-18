VERDICT: NO-GO

Counts: 1 Critical, 2 Important, 0 Minor, 0 Nit

Scope: tight review of `design/SPEC_sh2_demo_payload.md` against the four
assigned questions only. Settled items (seed choice, on-device delivery vs
NFC, the accepted §5 risk, non-funds-adjacent classification) not
re-litigated. Fork checked at working tree HEAD under
`/scratch/code/shibboleth/seedhammer`.

## Q1 — Is the ≤2-distinct-words invariant checkable against the shipped artifact?

**Yes, and it is already proven, not hypothetical.** `sysw.Open(blob, "")`
(`sysw/open.go`) decodes a raw `.bin` into `Payload{Public []string, Secret
[]string}`; `sysw.Classify` (`sysw/classify.go`) classifies each record, and a
`ClassMnemonic` record's string **is** the raw phrase text verbatim (not
entropy bytes) — confirmed both by `sysw_test_payload.go`'s own PROVENANCE
comment (the embedded record is literally `"abandon abandon … about"`) and by
existing tests. `cmd/emu/sysw_cards_payload_host_test.go` and
`sysw_test_payload_host_test.go` are **UNTAGGED** (no `//go:build js`), so
they build and run under plain `go test`, `os.ReadFile` the actual `.bin` off
disk, call `sysw.Open`, and assert against the decoded content — e.g.
`TestSyswCardsPayloadCoversEveryStagesWalk` counts `ClassMnemonic`/`ClassMDMK`
records straight from the opened blob. A demo-payload test that opens the
`.bin`, filters `ClassMnemonic` records, splits on whitespace and counts
distinct words is a direct extension of an already-working, already-host-run
pattern. No new decoding machinery is needed. §3's "run against the actual
embedded blob" instruction is achievable exactly as written.

## Q2 — Does the §4 narrowing of `TestEveryEmbeddedPayloadIsStructurallyConfined` work?

**No — as stated, it cannot.** Two independent problems in
`cmd/emu/embed_confinement_test.go`:

**(C-1) Directory scope is hardcoded to `cmd/emu`, which the device payload cannot live in.**
`findEmbedTokens(t, emuDir)` calls `os.ReadDir(dir)` — non-recursive — against
`emuDir := filepath.Join(root, "cmd", "emu")` only (lines 52-54, 153-156).
That is the *entire* discovery surface; nothing outside that one directory is
ever inspected for `//go:embed`. But a payload that must reach the real
device build has to live outside `cmd/emu` by construction: `cmd/controller`
(the firmware entrypoint) "does not import [cmd/emu], cannot import it" per
this repo's own doc comments in `sealed_test_payload.go`/`sysw_test_payload.go`
(two `package main`s can't coexist in one binary). So the file that would
carry the demo payload's `//go:embed` is structurally outside the one
directory this guard ever scans — it can neither forbid nor, via the
proposed §3 exception, permit it. Precedent for exactly this shape already
exists and is already invisible to the guard: `font/comfortaa/bold17.go`,
`font/poppins/regular16.go`, `font/constant/constant.go`, `font/sh/sh.go`,
and `gui/assets/embed.go` all carry untagged `//go:embed` directives that DO
ship into `cmd/controller` — confirmed no `//go:build` line on any of them —
and the guard has zero visibility into that whole class of file today.

**(I-1) The §3 exception needs content decoding the guard has never done.**
Today's discovery is purely lexical/structural: an AST walk for `//go:embed`
patterns and declared identifiers (`findEmbedTokens`), plus a second pass
matching identifier/string-literal text against Go source
(`referencedNames`). It never opens a `.bin`'s bytes. §4's "unless every seed
it carries passes §3" requires the guard to call `sysw.Open` + `Classify` and
count distinct words per phrase — real new logic, not a narrowing of an
existing per-file predicate. Buildable (Q1 shows the pieces exist), but the
spec's framing ("must be narrowed, not deleted") undersells this as a small
edit to an existing condition rather than new semantic-checking capability
bolted onto a guard that is otherwise pure syntax matching.

Related and worth surfacing here even though it's an open-question gap (see
Q3): it is not even settled that the payload reaches the device via a Go
`//go:embed` at all. The real device already reads its payload from a fixed
XIP flash region (`sysw.XIPReader`, `sysw.RegionAddr = 0x10D00000` in
`sysw/wire.go`/`sysw/read_tinygo.go`) — the same region an NFC/USB-loaded
payload occupies, matching the operator's own "they can live in payload."
If provisioning writes the demo bytes directly into that flash region at
manufacture time instead of compiling them into `cmd/controller`, no new
`//go:embed` exists anywhere and this guard is not the relevant mechanism at
all. (No tooling for that route exists yet either — `~/bin/sh/sh2-flash`
only builds/signs/flashes the firmware `.uf2`; it has no payload-region write
step.) Either way, §4's plan to narrow this specific test does not work as
written.

## Q3 — Are the five §6 open questions right? Missing anything?

The five (firmware size, digest pinning, record inventory, template-vs-concrete
selection, whether the demo engraves words) are all legitimate and each names
a concrete consequence of skipping it.

**(I-2) Missing: how the payload physically reaches the device.** Compiled
into the firmware image as a `//go:embed` (which would also require changing
the on-device read path — today `sysw.XIPReader.Read()` in
`sysw/read_tinygo.go` unconditionally reads the fixed flash region and has no
fallback to an embedded Go string, so `Platform.SyswReader()` in
`cmd/controller/platform_sh2.go` would need new logic) vs. written directly
into the existing payload region at `0x10D00000` at provisioning time
(matching "they can live in payload," but needs new flashing tooling that
does not exist in this repo today). This is concretely missing and
concretely important: left undecided, §4 can be (and was) written against a
mechanism — `//go:embed` — that may not be how the payload ships at all, and
either resolution requires real implementation work that neither §4 nor §6
currently names. This should be its own bullet in §6, resolved before R0.

## Q4 — Factual check

All claims checked out; nothing wrong found.

- **"every `//go:embed` under `cmd/emu` is `//go:build js`"** (the spec's
  actual, cmd/emu-scoped claim in §4): TRUE. All four embeds under `cmd/emu`
  — `sysw_test_payload.go`, `sysw_cards_payload.go`, `sealed_test_payload.go`,
  `sysw_composer_payload.go` — open with `//go:build js`. (If misread as a
  claim about the *whole repo*, it would be false — see C-1's font/gui-assets
  evidence — but the spec does not make that broader claim.)
- **"`beef ×12` is already a constellation fixture"**: TRUE.
  `mnemonic-toolkit/crates/mnemonic-toolkit/tests/lib_final_word.rs` defines
  `BEEF_11_PARTIAL` with the comment "full target phrase is `beef × 12`";
  `mnemonic-key/design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md` and
  `mnemonic-secret/design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` also
  reference it, satisfying "the final_word tests, and two implementation
  plans."
- **§2 entropy/fingerprint table**: TRUE, independently recomputed (BIP-39
  wordlist index → PBKDF2-HMAC-SHA512 seed → BIP-32 master key → hash160),
  not read off the spec:
  - `abandon×11+about` → entropy `00000000000000000000000000000000` ✓
  - `zoo×11+wrong` → entropy `ffffffffffffffffffffffffffffffff` ✓, master fp
    `3f635a63` ✓ (recomputed exactly)
  - `beef×12` → entropy `140280500a0140280500a0140280500a` ✓ (checksum
    valid), master fp `66d455ea` ✓ (recomputed exactly)
  - the beefbeef-as-entropy trap: `0xbeefbeef…` (16 bytes) decodes to `same
    law room lava winner jelly wing water use wash use teach` ✓ (recomputed
    exactly)
- **§6 firmware-size figures** (`1,506,884 B` flash / `62,592 B` RAM at
  `321acb56`): consistent with the figure already on record in this repo's
  own `CLAUDE.md` (measured 2026-09-02); not independently re-measured here
  as it is a settled prior measurement, not a new claim.

## Bottom line

Q1 is clean — the invariant is enforceable exactly as specified, using
machinery the codebase already has twice over. Q4's facts all hold. Q2 is
where this spec is not implementable as written: `TestEveryEmbeddedPayloadIsStructurallyConfined`'s
discovery is hardcoded to `cmd/emu`, which structurally cannot contain the
embed a device-shipped payload needs, and the guard has no content-decoding
capability to apply §3's exception even where it can see an embed. Q3 is
missing the one open question that would have surfaced this: how the payload
actually reaches the device. Recommend: add that question to §6, resolve it,
and rewrite §4 against whichever mechanism is chosen (a widened-scope guard
plus new content-check logic if `//go:embed`-based; a different guard
entirely, or none, if flash-write-based) before R0.
