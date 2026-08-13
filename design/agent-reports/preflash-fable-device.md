# Pre-flash gate review — device-side Go diff (sysw-port)

- **Reviewer:** fable (final gate, device lens only)
- **Repo/branch:** `/scratch/code/shibboleth/seedhammer` @ `sysw-port`
- **Diff:** `345d79ca..HEAD` (6 commits, 34 files, +1697/−26)
- **Taken as given (machine-verified):** host + tinygo + wasm builds; `go test ./gui/ ./sysw/`; Rust vector conformance.

## Verdict

**0 Critical for this flash.** The controlling fact, verified three independent
ways (grep for writers of `ctx.sysw`, grep for callers of `sysw.Open` /
`(*syswSession).load` / `Platform.SyswReader`, and reading `NewContext` at
gui/gui.go): **no production code ever assigns `ctx.sysw` or calls
`SyswReader()`/`sysw.Open`** — only tests do. Every new payload branch guards on
`ctx.sysw == nil` / `syswBundleSeed == ""` and is therefore dormant on the
machine. The firmware this flash produces never reads a byte of `0x10D00000`.
The load/compare/unlock flow (§10.1/§12.2) is not in this diff; until it lands,
the hostile-region attack surface is not reachable.

The device-reachable behavioral delta of this diff is exactly one thing: the
`inputWordsFlow` refactor — and it is behavior-preserving (see Q4).

Three Important findings below **gate the NEXT phase** (the wiring/load-flow
commit), not this flash. They are filed here so they cannot be forgotten; none
can fire while `ctx.sysw` is nil.

## The four questions

### Q1 — Brick / soft-brick: NO (this flash); one panic path found for the wiring phase

- The `sysw` parser itself is sound against hostile bytes. Order of operations
  is correct: `boundBlob` (sysw/read.go:45) hands `ParseHeader` a
  constant-bounded 52-byte slice; both section lengths are checked ≤ 8191
  before any arithmetic; `TotalLen` max is 52+8191+8191+16 = 16450 < 65536, no
  32-bit wrap possible. Iterations bounded 100k–2M **before** any KDF (no
  watchdog hang). `clampRegion` kills negative/oversize counts before
  `unsafe.Slice`. XIP read of 64 KiB at 0x10D00000 (13 MiB) is strictly below
  the already-shipping seal region at 0x10E00000 (14 MiB), so the mapping is
  proven readable by hardware that already boots. `splitRecords` is
  UTF-8-checked and bounded; worst-case classify work is ~8192 empty records —
  bounded. No panic, no unbounded loop, no nil deref found in sysw/*.
- The one real crash path is **I-1 below** (freetext prefill → font panic),
  unreachable this flash.

### Q2 — Seed/passphrase leak or persistence: NO new live exposure

- No new storage is populated in this firmware (session nil; `syswBundleSeed`
  only ever written by `syswOffer`, which returns false).
- `inputWordsFlow`'s new `entered()` only counts slots; no new copy of the
  mnemonic is made.
- When wired later: the session holding secrets as Go strings for process
  lifetime is the operator's explicit non-wiping ruling (decision 2, EPD §2.2
  item 12) and is documented as such at gui/sysw_session.go:12-18.
  `syswBundleSeed` holds only ClassMDMK (non-secret md1/mk1) and is cleared at
  gather entry (bundle_flow.go:108-110). The passphrase prefill copies into the
  `wipeBytes`-deferred buffer, never a new string (passphrase_flow.go:620) —
  correct shape.

### Q3 — Wrong-but-plausible value: none live; one for the wiring phase (I-2)

- `PublicDataHash` (sysw/pubhash.go) is byte-for-byte the Rust primary's
  construction — I diffed it against
  `mnemonic-engrave/crates/me-cli/src/sysw/pubhash.rs` directly: same label,
  same 0x00, same sealed byte, same `count as u8`, same `join("\n")`.
  `FormatHash` grouping (8×4 hex) matches Rust's `format_hash`. The count byte
  wraps at 256 records **in both implementations identically** — not a port
  divergence, and not exploitable (records cannot contain `\n`, so the join is
  injective and binds content). Spec-level note only.
- Identity over region bytes (not the §6.6 digest) is the right equality key
  and covers ciphertext; the secrets-only-payload aliasing trap is correctly
  avoided.
- Classify-once-at-load (sysw_session.go:39-42) correctly prevents
  admit-as-one-class/consume-as-another.

### Q4 — `inputWordsFlow` callers: NONE broken; frozen path byte-identical

All six call sites updated, all pass `wordEntryOpts{checksumGate: true}`,
which reproduces the old unconditional behavior exactly (the diff only added
`opt.checksumGate &&` to `onLastWord` and an early return to `refreshCands`;
with `true` both are identity). All callers use it in statement position, so
the new return value changes nothing. **unlock_kdf.go:160 (frozen Sealed
Payload path): the sealed-payload passphrase is REQUIRED checksum-valid by §8**
— `errUnlockChecksum`, and unlock_kdf.go:173's own comment relies on
`LastWordCandidates` restricting the final slot — so `checksumGate: true` is
not merely compatible, it is the frozen behavior. Verified against the pre-diff
text of the function.

### F-126 device-side impact: NOT worse than recorded — effectively nil on the device

The wasm freeze does **not** translate to hardware. On the device the reader is
`poller.New(p.nfc)` (cmd/controller/platform_sh2.go:566), and every iteration
of the gui scan goroutine parks in a blocking channel select:

- No tag/field: `Detect()` blocks in `waitForInterrupt(0)`
  (driver/st25r3916/st25r3916.go:238, :394) — a `select` on the interrupt /
  cancel channels with **no** timer arm. Parked indefinitely; zero CPU.
- External field held on: `Device.Read` waits with 1–10 s interrupt timeouts
  (st25r3916.go:562-597); a timeout surfaces as an error → `scanFailed` → the
  gui goroutine's 1 s sleep.
- The EOF fast path (tag read to completion, `scanFailed` never set, so no
  sleep) loops straight back into the blocking `Detect()`.

There is no state in which the goroutine can iterate without passing through a
channel wait or a sleep, and TinyGo's scheduler yields on both. The busy-spin
is a property of readers that return EOF immediately and forever — the emulator
and tests — not of the poller. Note in passing: this diff makes F-126 actually
*fire* in the emulator for the first time (cmd/emu NFCReader was `nil` before,
now returns the one-shot `nfcSource`, which sits at EOF after its single
record) — that is the already-filed wasm freeze, unchanged in severity.

## Findings

### Critical — none.

### Important (all gate the WIRING phase, none reachable in this flash)

- **I-1 — Freetext payload prefill can panic the firmware on an unengraveable
  rune.** gui/freetext_flow.go:1480-1484 sets `text = string(raw)` where `raw`
  is arbitrary hex-decoded bytes from a hostile `text:` record (DecodeBody
  constrains encoding, not content — `text:00` is valid). The freetext fit path
  measures by fixed char width only (backup/fit.go wrapBlocks; no per-rune
  `Decode`), so a rune absent from the vector face sails through admission and
  reaches `panic(fmt.Errorf("unsupported rune: …"))` at engrave/engrave.go:1561
  when the plate is rendered/engraved. On TinyGo that is a device panic →
  reboot. The typed path is safe only because the keyboard IS the charset
  filter; the prefill bypasses it. Fix before wiring: filter through
  `face.Decode` (the `backup.TitleString` pattern, backup/backup.go:102) or
  refuse the record. Contrast: the passphrase prefill is already protected —
  `ValidatePassphrase` rejects non-printable-ASCII at the entry step it still
  walks through.
- **I-2 — Passphrase payload prefill silently truncates at 100 bytes.**
  gui/passphrase_flow.go:620 `n = copy(secret, raw)` with
  `len(secret) == passphrase.MaxLen == 100`, while sysw deliberately permits
  passphrases up to `PassphraseMax = 215` (sysw/wire.go:52, whose comment
  explicitly rejects the 100 limit for entry). A 120-char payload password
  engraves as its first 100 chars; `ErrTooLong` is unreachable because the copy
  pre-truncates to exactly the limit, and the only tell is a full "100/100"
  counter. A truncated password on steel is precisely the wrong-but-plausible
  artifact class. Fix before wiring: refuse over-length records ("too long for
  a plate") instead of truncating.
- **I-3 — `admits()` and `syswFlags()` are production-dead.** Both are defined
  (gui/sysw_admit.go:47, :80) and tested, but no non-test code calls either;
  admission today is per-call-site hardcoded class constants, and **no §3.3.3
  flag (F1 secret-in-plaintext, F2 weak-cliff, F3 source, F4 NFC-no-integrity)
  is raised anywhere at the point of use.** Harmless while the session cannot
  load, but the moment the load flow lands, a plaintext secret would be
  consumable with no warning. The wiring phase must connect the flag surface to
  the consumption screens — filing this so the tested-but-unwired table cannot
  read as done.

### Minor

- **M-1** — Bundle gather discards the payload card's offer status
  (bundle_flow.go:110: `scr.g.offer(...)` return dropped, unlike the scan path
  which routes through `scr.feedback`). A refused card (e.g. a lone mk1, which
  ClassMDMK admits) vanishes silently; the tally is honest but nothing says
  why. Also, Back from review re-enters a fresh gatherer and the already-consumed
  payload card cannot be re-offered without leaving the program.
- **M-2** — Classification/consumption parser asymmetry: records are classified
  with `bip39.Parse` (accepts ≥3-char word prefixes) but consumed with
  `bip39.ParseMnemonic` (exact words) at gui/gui.go:2396 and
  gui/derive_xpub.go:140. A prefix-form mnemonic record classifies as
  ClassMnemonic, is offered, then silently falls through to typed entry.
  Fail-closed, but the operator gets no explanation.
- **M-3** — `syswSeedPicker` (derive_xpub.go:135) silently drops into typed
  entry when `take` refuses (compared==false); the operator explicitly chose
  "FROM PAYLOAD". Fail-closed; needs a message when wired.
- **M-4** — `wordEntryOpts.terminator` claims "draws a `done` affordance"
  (gui/gui.go:744) but nothing draws it — only the `Clicked` check exists
  (gui/gui.go:837), plus a leftover `_ = doneBtn`. Dead until a caller passes
  `terminator: true`, at which point the button would work invisibly. The §8c
  stub should not land with an overclaiming comment.
- **M-5** — `PublicDataHash` count byte wraps at 256 records — in the Rust
  primary identically (`records.len() as u8`, pubhash.rs:26), non-exploitable
  (join binds content), but worth a one-line spec note in the primary.

## Scope note

Reviewed for the device lens: all of sysw/*, gui/sysw_session.go,
gui/sysw_admit.go, gui/gui.go, gui/derive_xpub.go, the four wired programs
(newInputFlow, bundle_flow, freetext_flow, passphrase_flow), the two verify
flows (typed-only switch is correct and structurally tested via the AST scan in
gui/sysw_verify_test.go), unlock_kdf.go, cmd/controller/platform_sh2.go, and
the driver/poller stack for the F-126 question. cmd/emu and cmd/journeykeys are
not firmware-reachable and were checked only for linkage (they are not linked
into cmd/controller).

**Gate result: CLEAR TO FLASH. 0C/0I against this firmware. I-1/I-2/I-3 own the
wiring phase and must burn down before the load flow ships.**
