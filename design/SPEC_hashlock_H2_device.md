# SPEC — Hashlock H2: the device leg (SeedHammer II fork)

**STATUS: DRAFT 2026-09-05 — citations measured at fork main `c4a64fc`, ms `cd0a60f`
(ms-codec 0.8.0 as published), engrave `02193cb`; R0 not yet dispatched.**

This is stage H2 of the hashlock-phrase cycle (`design/BRAINSTORM_hashlock_phrase.md`
§4.1; SPEC_ms_hashlock §9's sequence). H0 (reader guards) is merged in both repos;
H1 (`ms hashlock`, ms-codec 0.8.0) is released. H2 gives the device what the
brainstorm's §4.4 agreed (L7, L15, L16) and the r2 security review sharpened:
**type a hashlock phrase on the SH2, pick the method, and set a spend path's hash
to the SAME digest the host derives** — deriving, using and dropping the preimage,
never storing, showing or engraving it. The Go derivation is a strict port of
`ms_codec::hashlock` with the ms-codec 0.8.0 corpus vendored and pinned
(Rust-primary rule; nothing is decided in Go).

Rulings that bind (operator, verbatim in the brainstorm): L5 two methods, the
operator chooses; L7 the device derives H, uses it, scrubs X, never stores/shows/
engraves a preimage this cycle; L12 both warnings warn, never refuse; L15 no scrub
discipline beyond what the composer does by construction; L16 §4.4 agreed; L22
`0x03` inert, no new class (shipped in H0); L24 `TagKindMismatch` refused.

---

## §1. Scope

In:
1. `Which hash?` gains the row **`Type a hashlock phrase`** before `Type 64 hex`
   (§4), with the row switch re-keyed by LABEL (§5) — the shipped switch is
   index-keyed and its fallthrough clears the lock (r2 C-4).
2. The phrase screen (§4.2), the method pick (§4.3), the derivation (§3, §4.4),
   the confirm modal (§4.5). Back at any step returns to `Which hash?` with the
   path unchanged.
3. `hashlock` — a new fork package porting `ms_codec::hashlock` (§3) with the
   corpus `hashlock-v0.8.json` vendored under `hashlock/testdata/` and a
   provenance pin (§7.1).
4. `codex32.DecodeMS1Preimage` (§6): the `0x03` arm as its OWN function with its
   own length rule; `DecodeMS1` unchanged (r2 C-2). No screen calls it this cycle;
   it exists so the kind has one decoder and one test, Rust-first.

Out (§9): storing, displaying or engraving a preimage; reading a preimage plate
into any flow; a scrub discipline; `ms split` of preimages; salt/iteration
choices (F-469); the `me`/host side (H1b, separate plan); the operator's live
device walk (H4, §8).

---

## §2. The phrase rule — the host's, byte for byte

SPEC_ms_hashlock §4.3 is normative; the device applies the SAME predicate to the
typed bytes, in this order, and refuses with a modal (never silently):

1. **non-empty**;
2. **printable ASCII only**, every byte in `0x20..=0x7E` (the keyboard cannot
   produce anything else — the test still pins it);
3. **ms1-shaped is refused** — the string, trimmed and case-folded, starts
   `ms1` and is BCH-valid codex32 (`codex32.New` accepts it): *"That is an ms1
   string, not a phrase. Load it from the payload instead."* (a plate is not a
   phrase; there is no device route for a preimage plate this cycle — §6);
4. **at most 100 characters** — the counter reads `n/100`, is NOT clamped (so
   `101/100` is visible and the lockstep row is constructible), and OK is
   refused above 100: *"A hashlock phrase is at most 100 characters."*;
5. **exactly 64 hex characters is refused**, either case: *"That is a preimage
   in hex, not a phrase. Use the Type 64 hex row."*

The bytes are used VERBATIM: no trimming, no case folding, no normalisation
(rule 3 folds only to DETECT a plate; the phrase itself is never changed).
`Correct Horse` and `correct horse` are different phrases — the corpus row
`Correct Horse Battery Staple` pins it.

The limit is a named constant, `hashlock.PhraseMaxChars = 100`, the ONLY source
of the counter's denominator and the rule's bound; a test asserts both read it.

---

## §3. Derivation — the port

`hashlock` package, one file, provenance-pinned to ms `cd0a60f`
(`crates/ms-codec/src/hashlock.rs`, ms-codec 0.8.0):

| name | value | Rust |
| --- | --- | --- |
| `Salt` | the 14 bytes `ms-hashlock-v1` | `HASHLOCK_SALT` |
| `Iterations` | 100000 | `HASHLOCK_ITERATIONS` |
| `PreimageLen` | 32 | `HASHLOCK_DKLEN` |
| `PhraseMaxChars` | 100 | ms-cli `HASHLOCK_PHRASE_MAX_CHARS` (`crates/ms-cli/src/hashlock_phrase.rs:24`; the codec carries no cap — the rule is the CLI's and the device's, §4.3) |
| `PreimageHardened(phrase []byte) [32]byte` | PBKDF2-HMAC-SHA256(phrase, Salt, Iterations, 32) | `preimage_hardened` |
| `PreimageSHA256(phrase []byte) [32]byte` | SHA-256(phrase) | `preimage_sha256` |
| `Digest(x *[32]byte) [32]byte` | SHA-256(x) | `digest` |

**The salt is passed as its own 14-byte slice through a NEW driver**, never
through `unlockDerive`'s `seal.Header` (its `Salt [16]byte` would zero-pad to 16
bytes and every device digest would silently diverge — r2 M-5). The only test
that can see that class is a comparison against the vendored corpus CONSTANT,
never against a value recomputed by the same Go function (§7.1).

Both methods run on the countdown screen the sealed payload already uses
(`gui/unlock_kdf.go:236`, "Unlocking. About N seconds left.", driven by the
measured rate), retitled `Deriving`; SHA-256 is instant and shows no countdown.
The measured device rate is 9,715 PBKDF2 iterations/s (brainstorm §3.4), so
hardened takes about 10 s — the method row says so.

**The preimage lives on the stack for the derivation and the confirm modal and
is dropped after CONTINUE or Back** (L7; L15: no scrub beyond that). The digest
is what the composer stores (`st.list.Paths[idx].Hash`, a `[32]byte`), exactly
as `Type 64 hex` stores one today.

---

## §4. Screens and copy

All copy ASCII, inside the modal-fits assertion the composer's copy tests use.

### §4.1 `Which hash?`

Rows, in order: the payload's `hash:` records (`hash <i>  <first8>..<last8>`,
unchanged), **`Type a hashlock phrase`**, `Type 64 hex`, `No hash lock`. With no
`hash:` record loaded, the screen's lead reads:

> No hash record in the payload. ms hashlock on the host makes one.

(the F-465 hint; second lead line only when `len(digests) == 0`).

### §4.2 The phrase screen

Title **`Hashlock phrase`**. The four-page printable-ASCII keyboard built with
`NewPassphraseKeyboard` (`gui/passphrase_keyboard.go:76`) — not `NewTextKeyboard`
(:92, settings gear + newline) nor `NewLineKeyboard` (:112). A NEW flow function
`hashlockPhraseFlow(ctx, th) ([]byte, bool)`, not `passphraseEntryFlow`
(`gui/passphrase_flow.go:74`: it hard-codes the "Passphrase" title, the
pass-proof trigger and an over-length message about plate legibility — r2 M-4).
Counter `n/100` from `hashlock.PhraseMaxChars`. OK applies §2; Back returns to
`Which hash?`.

### §4.3 The method pick

A two-row `composerPickScreen` titled `Hashlock method`, lead `Which method?`:

1. **`Hardened (about 10 s)`** — under 20 characters, a confirm modal first:
   > A 20-character phrase falls in about 72 days on one GPU. Choose it from a
   > generator. Continue?
2. **`SHA-256`** — always, a confirm modal first:
   > This is the brainwallet construction: anyone holding the digest tests
   > 10^10 phrases per second. A phrase a person chose is not safe here; use
   > six diceware words. Continue?

Both are confirm-to-proceed (L12: never refusals). The method is a permanent
property of the policy; the confirm modal (§4.5) prints it.

### §4.4 Deriving

Hardened: the countdown screen, title `Deriving`, body `About N seconds left.`
from the measured rate. Back during the derivation abandons it: nothing was
assigned (r2 Q3) and the composer state is untouched; power loss likewise.

### §4.5 The confirm modal

Title `Hash lock`. Lines, in order:

```
hash  <first8>..<last8>
method: hardened            (or: method: sha256)
```
then the §8i rule (composer spec §8i, unchanged), then:

> One phrase per policy. Spending any path of a wsh
> wallet publishes this digest. Never use this phrase
> as a passphrase or a password anywhere else.

then the F-132 line the composer already prints when every path is hashed
(§8h). **CONTINUE** sets `st.list.Paths[idx].Hash` to the digest and returns to
the path; **Back** discards (nothing was assigned before CONTINUE).

The 64 visible bits (`first8..last8`) are a transcription check for the operator,
adequate ONLY because the full-width lockstep vector runs in CI (§7.1); the H4
walk records both full digests (r2 N-2).

---

## §5. The row switch is label-keyed (r2 C-4)

`composerHashEdit` (`gui/composer_hash.go:140-172` at `c4a64fc`) builds `rows`
as payload digests + `Type 64 hex` + `No hash lock` and dispatches on
`sel < len(digests)` / `sel == len(digests)` / `default` — and the `default`
arm CLEARS the lock. Inserting a row before `Type 64 hex` under that switch
makes `Type 64 hex` fall into `default` and silently remove the hash.

Normative: the rows are built ONCE into a struct that records each named row's
index (`payloadRows`, `phraseRow`, `hexRow`, `noneRow`); the switch dispatches
on those names; there is NO `default` that assigns — an unknown index is a
programming error (`panic` naming the index), never "clear the lock". The §8i
modal fires when the operator is TAKING a hash (payload row, phrase row or hex
row), stated as that predicate, not as `sel <= len(digests)`. Tests cover every
row by label, with 0, 1 and 2 payload digests loaded (the displaced rows
included).

---

## §6. Preimage strings on the device — inert, one decoder

H0 (fork `c4a64fc`) made a kind-`0x03` single inert on every reader and door;
this stage adds the decoder and nothing that calls it from a screen:

`codex32.DecodeMS1Preimage(s String) (preimage [32]byte, err error)` — accepts
ONLY an unshared string whose data is exactly 33 bytes beginning `0x03` (the
shape `IsPreimage` already tests) and returns the 32 bytes; every other input
returns `errMSBadPrefix` or `errMSBadLength` (the same errors `DecodeMS1` uses).
`DecodeMS1` is UNCHANGED and keeps refusing `0x03` at all five callers
(`gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `gui/singlesig_verify.go:185`,
`gui/multisig_verify.go:1237`, `bundle/verify.go:138`) — r2 C-2 — and "Show
secret" stays gated on `err == nil`. A typed or scanned preimage string is
refused everywhere a seed is expected, as H0 measured.

The returned preimage is secret: the doc comment says the caller scrubs, exactly
as `DecodeMS1`'s does.

---

## §7. Tests

### §7.1 Lockstep against the vendored corpus (the gate that matters)

`hashlock/testdata/hashlock-v0.8.json` = ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json`
byte for byte, sha256 `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`,
with `hashlock-v0.8.provenance.json` in the shape of `sysw/testdata/record_class_vectors.provenance.json`
(repo, remote, path, commit `cd0a60f`, sha256, row count, recorded_at). Tests:

- the file hashes to the pinned literal (drift on either side reds one suite);
- every `derivation` row (11): `PreimageHardened(phrase)` == `hardened_x`,
  `Digest` == `hardened_h`, `PreimageSHA256(phrase)` == `sha256_x`, `Digest` ==
  `sha256_h` — compared against the JSON's CONSTANTS, never against a value the
  Go code recomputed (mutation: zero-pad the salt to 16 bytes → every hardened
  row fails; mutation: 99,999 iterations → every hardened row fails; mutation:
  lowercase the phrase → the `Correct Horse Battery Staple` row fails);
- the 100- and 101-character rows: 100 accepted by the rule, 101 refused with
  the §2 message (the lockstep row);
- the 64-hex rows: a 64-hex phrase refused naming `Type 64 hex`; 63 and 65 hex
  accepted as phrases;
- `PhraseMaxChars` is read by both the counter and the rule (mutation: change one
  → a test that types 100 characters and expects OK fails).

### §7.2 The screens, on the touch harness

Driven through the real flow (`runUITouch`, as `walk_h0_preimage.js`'s Go twin
tests do): tap `Type a hashlock phrase`, type the anchor phrase, pick each
method, confirm — the path's `Hash` equals the corpus's `hardened_h` /
`sha256_h`. Back at each step leaves `Hash` unchanged. The §2 refusals driven
through the screen with the counter at `101/100`, with a 64-hex phrase, with an
ms1-shaped phrase. The two method modals appear when their condition holds and
not otherwise (19 vs 20 characters; SHA-256 always). Geometry: the confirm
modal and the no-payload hint fit (the composer's modal-fits assertion).

### §7.3 The switch

Every `Which hash?` row by label with 0, 1, 2 payload digests: each row does
what its label says; `Type 64 hex` never clears the lock (the C-4 regression
test); the §8i modal fires for the three taking rows and not for `No hash lock`.

### §7.4 The decoder

`DecodeMS1Preimage`: the corpus plate (`ms10hashsq0p7jaf…` from the acceptance
record, or any row's plate produced by `ms hashlock --out`) → 32 bytes equal to
that row's `hardened_x`; an entr string → `errMSBadPrefix`; a share → error; the
`preimage-shape-entr-id` seam row → 32 bytes (the kind is the prefix byte; the
id is not consulted here either — the HOST refuses the mismatch, ruling L24).
`DecodeMS1` on the plate → `errMSBadPrefix` still (one test per caller site is
H0's; one here on the function).

### §7.5 The emulator arm

`cmd/emu/walk_hashlock_phrase.js`: from the composer, take a path to `Which
hash?`, tap the phrase row, type the anchor phrase, pick SHA-256 (instant),
confirm, and read the `hash first8..last8` line: it must equal
`b867db87..dbc96cb` (`sha256_h` of the anchor row); the negative control types a
different phrase and must NOT match. Hardened is walked once (about 10 s) and
compared to `3cf5d421..b70a4c12`.

### §7.6 Firmware size

Measured at the gate (`nix develop -c tinygo build -size short …`); PBKDF2 and
SHA-256 are already linked (`seal/pbkdf2.go`, `seal/crypto.go`), the keyboard
exists; expect a small delta over `c4a64fc`'s 1,583,132 / 62,800.

---

## §8. Acceptance (H4 — the operator's walk)

H2 is done when, on the flashed device: the operator types the anchor phrase
under each method and the `first8..last8` shown equals `ms hashlock`'s on the
host for the same phrase and method (`3cf5d421..b70a4c12` hardened,
`b867db87..dbc96cb` sha256), records both full 64-hex digests in the continuity;
a `hash:` record packed by `ms hashlock … | me sysw pack` is offered as a payload
row; and a preimage plate presented to the device is still refused (H0's walk).
Until the operator walks it, §7.5's emulator arm is the acceptance.

---

## §9. Out of scope (this cycle)

Storing, displaying or engraving a preimage on the device; reading a preimage
plate into any flow (§6 adds the decoder only); a scrub discipline (L15); the
salt/iteration parameters (F-469); `ms split` of a preimage (F-468); the host's
0.8 bump (H1b, `IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md`); the flash of H0
(the operator's).

---

## §10. Citations — measured at fork `c4a64fc`; re-grep at implementation time

| claim | where |
| --- | --- |
| `composerHashEdit` builds rows and dispatches by index; `default` clears | `gui/composer_hash.go:140-172` (`rows = append(rows, "Type 64 hex")` at :147; `composerPickScreen(ctx, th, title, "Which hash?", rows)` at :149; `default: st.list.Paths[idx].Hash = nil`) |
| `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)` | `gui/composer_paged.go:259` |
| `composerHexEntry`, `composerHashRow`, `composerPayloadDigests`, `composerCopyHashRule` | `gui/composer_hash.go:69, :38, :47`; `gui/composer_copy.go:175` |
| the three keyboards | `gui/passphrase_keyboard.go:76` (`NewPassphraseKeyboard`), `:92` (`NewTextKeyboard`), `:112` (`NewLineKeyboard`) |
| `passphraseEntryFlow` hard-codes its title and messages | `gui/passphrase_flow.go:74` |
| the countdown copy and `unlockDerive(ctx, th, h seal.Header, pass []byte)` | `gui/unlock_kdf.go:236`, `:242` |
| PBKDF2 and SHA-256 already linked | `seal/pbkdf2.go`, `seal/crypto.go`, `gui/unlock_kdf.go` |
| `DecodeMS1` and its five callers | `codex32/mspayload.go:35`; `gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`, `bundle/verify.go:138` |
| `IsPreimage` (H0) | `codex32/mspayload.go` (appended by H0; unshared, 33-byte payload, `0x03`) |
| vendored-corpus convention | `sysw/testdata/record_class_vectors.provenance.json`, `sysw/codex32_seam_test.go` (sha pinned as a literal) |
| the corpus and its sha | ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`, `a46c197a…1d30` (CHANGELOG ms-codec 0.8.0) |
| the derivation constants | ms `crates/ms-codec/src/hashlock.rs:27,30,32` at `cd0a60f` (`HASHLOCK_SALT`, `HASHLOCK_ITERATIONS`, `HASHLOCK_DKLEN`); the phrase cap ms-cli `crates/ms-cli/src/hashlock_phrase.rs:24` |
| the phrase rule and both warnings' copy | SPEC_ms_hashlock §4.3, §7 |
| measured KDF rate 9,715 it/s | brainstorm §3.4 |
| ruling L22 and H0's guards | `codex32.IsPreimage`, `sysw.isStrictMs1`, `seal.Classify`, `gui/scan.go`, `engraveCodex32`, `unlockEngraveCodex32` at `c4a64fc` |
