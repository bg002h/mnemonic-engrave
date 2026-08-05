# Feature map — `Gangleri42/seedhammer` vs our fork

Read-only comparison, 2026-08-04. Ours = `bg002h/seedhammer` `main`; theirs = `Gangleri42/seedhammer` `main`.

## Headline

**The two forks diverged from upstream in almost entirely disjoint directions.**
Theirs added `curves/`, `richtext/`, `svgpath/`, `nip19/` (NFC drawings, rich text, Nostr keys)
plus `cmd/sh2key`, `cmd/svgplate`, `cmd/textplate`, `cmd/nfc-bridge`, `cmd/emufixtures`.
Ours added `md/`, `mk/`, `seedxor/`, `bundle/`, `passphrase/` plus a much larger `gui/`.

**Dependencies are effectively identical** (`go.mod` differs only in `golang.org/x/sys`
direct-vs-indirect), so importing anything below drags **no new third-party dependency**.

**No licensing catch.** Both trees carry the same three LICENSE files — root (Unlicense),
`font/comfortaa/`, `font/poppins/`. Everything theirs added is under the root Unlicense.

## Boot-key tooling — the substantive comparison

`cmd/sh2key` (32 files, ~253KB) mints/backs up/restores the boot key, derives a Nostr identity,
classifies board OTP state, and runs provision/sign/flash/revoke as one automated ceremony.

| Aspect | Theirs | Ours | Assessment |
|---|---|---|---|
| Automation | One binary chains mint → fuse → sign → flash, resumable, TUI | Manual, individually gated `picotool`/`openssl` steps + assertion scripts that verify but never chain writes | **Ours is more conservative by design.** Not automating the fuse step is a feature. |
| **Hardware rehearsal** | **Absent** | **Required** — `pico2-bootkey-rehearsal.sh` proves the mechanism A/B on a $5 Pico 2 (image REJECTED before the burn, ACCEPTED after) | **A safety edge we have and they don't.** |
| **Key backup as 24 BIP-39 words** | **Yes** — `backup`/`restore`, plus single/double-word repair search by fingerprint, plus a "plate 2" explaining the words are NOT a wallet seed | **Absent.** We say "back it up offline, twice" with no engraving path | **The one genuine gap.** |
| Revoke / KEY_INVALID | Automated, heavily gated | We deliberately offer none | Ours is the safer default; omission over automation for the one unrecoverable op. |
| Byte-level hash gotchas (uncompressed X‖Y, `-s` vs `otp load` majority-vote) | Documented | Documented **and asserted in scripts** | Converged independently — mutual validation. |

**Net: neither procedure is safer overall.** Theirs optimises for a smooth one-tool experience;
ours for forcing a rehearsed, unautomatable sequence of typed, individually verified steps.

## Firmware programs

Both have Backup Wallet, BIP-39 Password (theirs folded into the seed flow, ours a top-level
program), and Engrave Text (parallel invention). **We are ahead on** Account Xpub, Engrave Bundle,
separate Single-Sig/Multisig state machines, and BIP-85 wired into the GUI (theirs is host-only).
**They have one thing we lack:** multisig **cosigner-share splitting** — restoring the original
SeedHammer partition scheme SH2 dropped, so an n-1-of-n / n-of-n / 2-of-4 / 3-of-5 quorum gets one
fragment per cosigner plate instead of a full descriptor copy.

**Their engraving-quality claims are SCOPED, not general.** The README's "egg-shape fix, periodic
spline, serpentine rows" work lives in the new `svgpath`/`font/glyph` packages feeding the NFC
drawing pipeline — **not** in `bspline`/`bezier`/`font/sh`, which are structurally identical
file-for-file to ours. So there is no core motion-planner improvement to take.

## Verdict

**Worth taking:**
1. **Boot-key backup as 24 BIP-39 words**, plus the "this is not a seed" restore-instructions
   plate (~800–1000 lines carved out of `backup.go`/`key.go`/`restore.go`/`repair.go`; no NFC, no
   new deps). Today a lost boot-key PEM permanently costs an OTP slot with no recovery path — and
   **we already have the typed-word entry UI to engrave the words**.
2. **Single/double-word repair-by-fingerprint search** — small, generalisable to any typed BIP-39
   input checked against a known-good fingerprint.
3. *(intentionally thin — the forks solved different problems; there is no deep well of easy wins.)*

**Explicitly NOT worth taking:**
1. **The whole curves/richtext/Nostr/Studio family** — every path needs NFC (unavailable to this
   user) and a hosted browser companion.
2. **`sh2key`'s one-shot `provision` ceremony** — collapsing fuse+sign+flash into one command works
   against our deliberate choice to keep the one irreversible action separate and rehearsed.
3. **Multisig cosigner-share splitting** — a real, well-designed feature, but a new plate-content
   format on funds-critical ground (wrong-quorum reconstruction = lost access). Deserves its own
   from-scratch spec/R0 cycle, not a port.

## Not fully determined
`gui/gui.go` (4000 lines) was not read line-by-line, and four `howto-*.md` files were greped rather
than read in full. The "Account Xpub has no equivalent in theirs" claim rests on string search.
