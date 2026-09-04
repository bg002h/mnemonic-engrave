# Brainstorm — the HASHLOCK PHRASE (`ms hashlock` + on-device entry)

**STATUS: BRAINSTORM IN PROGRESS, 2026-09-03.** Rulings below are the operator's
and are final; everything else is measured context, agreed design, or a default
taken for the operator's veto. No R0 review, no gates, nothing may be
implemented from it. Written mid-session so the rulings outlive the context
([[decisions-must-outlive-the-agent]]).

Companions: `SPEC_wallet_policy_composer.md` (§6c, §8h, §8i, §14 row
"on-device preimage derivation"; C25), `S4_journey_walk_2026-09-02.md` (W-5),
`FOLLOWUPS.md` F-132, F-465, F-466, F-467, F-468. Terminology is in memory
(`hashlock-phrase-terminology`) and repeated in section 3.1.

Heads measured against: mnemonic-secret `7fc1e58` (ms-cli 0.17.1, ms-codec
0.7.0), fork `70008da5`, mnemonic-engrave `51b7c69`, descriptor-mnemonic
`1dc8d409`, mnemonic-toolkit `d8f06483`.

---

## 1. What the operator asked for

At the S4 device walk (W-5, 2026-09-03), typing a hashlock's 64 hex on the SH2
was "very hard", and the operator asked (1) for a host method that takes a
memorable text, derives the preimage and packs the `hash:` record, and (2) to
type that text on the SH2 directly. Rulings that day: `ms` owns the host command
(F-465); on-device entry is REQUIRED, Rust first (F-466).

This session: "Do you remember our hashlock phrase discussion? I want to pivot
to that."

## 2. Rulings (operator, verbatim where quoted)

| # | Ruling | Consequence |
| --- | --- | --- |
| L1 | **Home and order.** 2026-09-03 (W-5): "We must allow user to type hashlock phrase on the SH2 also, but the code should be rust first." `ms` owns the command. | The phrase -> preimage -> digest rule lands in Rust with test vectors; the device screen is its behaviour-faithful Go port under the Rust-primary rule. |
| L2 | **Terminology** (2026-09-03, agreed): the memorable text is the **hashlock phrase** (two words; never "passphrase"); the 32-byte value the script demands is the **preimage** X; H = sha256(X) is the **digest** the policy carries. | Flags `--hashlock-phrase*`; screens say "Hashlock phrase". |
| L3 | **Derivation, first pass.** Asked "how should X be derived from the phrase": "Must be either sha256 or ripemd160 to work with bitcoin script." Then, on seeing that the phrase -> X step is off-chain: "I see I was misunderstanding your question ... that could in theory be quite different than sh256... I like your KDF idea, but can a bitcoin hardware wallet perform that operation? I think we can assume a bitcoin hardware wallet can do kdf iterations at 1/10th the performance of the sh2, but we should limit iterations to 100,000. But the the user should be encouraged to document or backup the method of hashlock phrase to pre-image as well but that backup will be external to our tool." | Iteration cap 100,000; a signer at 1/10th of the SH2's rate is the design margin; the tool must state the method so the operator can write it down outside the tool. ripemd160 cannot be X's derivation in any case: every miniscript hash fragment demands a 32-byte X (`OP_SIZE 32 EQUALVERIFY`), and ripemd160 yields 20. |
| L4 | **KDF parameters** (chosen from three options): **PBKDF2-HMAC-SHA256, 100,000 iterations**, fixed salt, dkLen 32. | Reuses the primitive the SH2 already runs for the sealed payload and measured (section 3.4). |
| L5 | **Two methods, the operator's choice.** "User should be allow to choose between hardened kdf as above or just sha256 to go from hashlock phrase to primate." | `--method hardened` (L4) or `--method sha256` (X = sha256(phrase bytes)); the device asks the same question as a two-row pick. The card and `--json` always name the method used. If the note is lost, try both: two derivations, two digests, one matches the policy. |
| L6 | **The preimage gets its own ms1 kind byte** (chosen over "plain entr-32 plus a note" and "no ms1 form"). | ms-codec 0.8.0 claims prefix `0x03` = hashlock preimage, 32 bytes only; `ms decode` never prints it as words; the device never offers it as a seed. |
| L7 | **Device scope this cycle = digest only** (chosen over "digest plus a preimage plate" and "digest plus preimage as a source"). | The SH2 derives H from a typed phrase, uses it, shows `first8..last8`, states §8i and F-132, and scrubs X. It never stores, shows, engraves or sources a preimage. |
| L8 | **32 bytes = 64 hex characters.** "Do we mean 64 hex chars or 32?" Both spellings name the same value. | Every refusal and help line says "32 bytes (64 hex characters)". |
| L9 | **The `ms hashlock` surface in section 4.2 is agreed as a form.** "Agree with form." | Section 4.2 is the spec's input; defaults in section 5 remain vetoable. |

## 3. Measured context (so no later section re-derives it)

### 3.1 The three names

| what | name |
| --- | --- |
| the memorable text | hashlock phrase |
| X, exactly 32 bytes, what the script demands | preimage |
| H = sha256(X), what the policy and the plate carry | digest |

### 3.2 Why X must be 32 bytes, and a shipped document that got it wrong

`sha256(H)` compiles to `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL`
(spec §6c, §8i). The hashvault journey (`design/journeys/derive-hashvault-keys.sh:61-76`)
derives its three hashlocks as sha256 of phrases of **40, 38 and 34 bytes**
hashed ONCE (measured with `printf '%s' | wc -c`), so no 32-byte preimage exists
and tiers 1-2 of that (never funded) policy can never be spent. Filed F-467.
This is the defect class `ms hashlock` exists to make impossible.

### 3.3 What `ms` is today (mnemonic-secret `7fc1e58`)

- Ten verbs: derive, encode, decode, inspect, verify, vectors, gen-man, repair,
  split, combine. Secret discipline: `SECRET_FLAGS = ["--phrase", "--hex",
  "--ms1", "--passphrase"]` in `crates/ms-cli/src/argv_guard.rs:86`, matched on
  raw argv BEFORE clap, refused without `--allow-argv-secret`; `-` and
  `--in FILE` are the private channels; `--passphrase-stdin` strips exactly one
  trailing LF or CRLF (`parse.rs:139-148`); `encode` prints the artifact on
  stdout, an engraving card on stderr, and the class advisory
  (`advisory.rs:53`: PrivateKeyMaterial / WatchOnly / Template).
- ms1 wire: prefix byte `0x00` = entr, `0x02` = mnem, `0x01` and `0x03..0xFF`
  UNALLOCATED, claim-via-PR (`design/SPEC_ms_v0_2_kofn.md:27`, `MIGRATION.md:23`).
  Single strings of every kind carry `Tag::ENTR` (`cmd/encode.rs:200`); the kind
  lives on the prefix byte alone. The prefix axis is orthogonal to the share
  axis. Dispatch sites: `envelope.rs:192` (`dispatch_payload`), `:231`
  (`payload_wire_bytes`), `inspect.rs:122`; shares go through the same two
  functions (`shares.rs:15`).
- A 32-byte payload under `0x00` is entr-32: `ms decode` prints it as 24 BIP-39
  words (`cmd/decode.rs:118-121`) and the device decodes it as a seed
  (`codex32/mspayload.go:34-60`, `DecodeMS1`, default arm `errMSBadPrefix` for
  an unknown byte). Same string length either way: 33 payload bytes -> 75 chars.
- `ms-codec` depends on `getrandom` (shares); `me` depends on `pbkdf2 = 0.12`
  + `sha2 = 0.10` (`crates/me-cli/Cargo.toml:45-46`, `seal/crypto.rs:35`
  `pbkdf2_hmac::<Sha256>`); the fork vendors `golang.org/x/crypto/pbkdf2`
  (`slip39/feistel.go:50`, `cmd/kdfbench`).
- `ms split` resolves `--phrase`/`--hex` only (`cmd/split.rs:108`
  `resolve_secret_payload`): no ms1 positional source. Filed F-468.
- The manual chapter that must move in lockstep:
  `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` (exists).

### 3.4 The KDF rate is measured, not estimated

`cmd/kdfbench` on real RP2350 silicon (Pico 2, 150 MHz, the firmware's own
flags): **9,715 PBKDF2-HMAC-SHA256 iterations/s at dkLen 32**
(`CONTINUITY_2026-08-07.md` §4). The sealed payload ships 300,000 iterations =
30.9 s and the operator accepted that unlock time.

| where | 100,000 iterations |
| --- | --- |
| SH2 | 10.3 s |
| a signer at 1/10th (L3) | 103 s |
| a laptop core | under 0.1 s |

Memory-hard KDFs are out: the RP2350 has 520 KB of RAM. SHA-512 as the HMAC
hash would be 2-3x slower on a 32-bit core for no gain at dkLen 32.

### 3.5 Who derives X at spend time

No hardware signer sees a hashlock phrase today: the coordinator (Liana,
Sparrow) places the 32-byte X in the PSBT's sha256-preimage field and the signer
only signs. So the phrase -> X derivation runs on whatever host tool the
operator has at spend time, which is why the method must be in the external
backup (L3). The 1/10th budget is margin for a future signer port.

### 3.6 The device today (fork `70008da5`)

- Hash entry (`gui/composer_hash.go`): `Which hash?` rows = the payload's
  `hash:` records as `hash <i>  <first 8>..<last 8>`, then `Type 64 hex`, then
  `No hash lock`; the §8i modal fires once a hash is actually taken; the
  fallback pad is hex-only, accepted at exactly 64 characters.
- A printable-ASCII keyboard with four pages, a reveal toggle and an n/100
  counter exists (`gui/passphrase_keyboard.go`; `passphrase.ValidatePassphrase`
  = non-empty, 0x20..0x7E only, at most 100 runes). `crypto/sha256` is already
  linked (`bip39/bip39.go`). The composer's scrub is one `defer st.reg.scrub()`
  at the flow top (`gui/composer_sources.go:222-230`).
- `sysw.ParseHashRecord` (`sysw/composer_records.go:100`): `hash:` + exactly 64
  lowercase hex. The Rust primary is `crates/me-cli/src/sysw/composer_records.rs`.
- `me`'s own ms1 classifier (`seal/record.rs`, `MAX_ENGRAVEABLE_MS1_LEN` 90,
  mirrored at `sysw/classify.go:13-16`) treats every checksum-valid `ms1` as a
  secret seed; a preimage-kind string would be offered at seed entry and fail
  at decode. Both sides need the kind, Rust first.

## 4. Design agreed so far

### 4.1 Scope and stages (presented; not objected to)

- **H1 mnemonic-secret (Rust).** ms-codec 0.8.0 adds the kind; ms-cli 0.18.0
  adds `ms hashlock` plus the kind's arms in decode, inspect, split, combine and
  refusals in derive and verify. Vectors pin phrase -> X -> H for BOTH methods.
  Manual chapter in lockstep.
- **H2 seedhammer fork (Go port).** The `0x03` arm in `DecodeMS1`, every seed
  call site refusing it by name, a payload class for it that reaches no screen,
  and the `Type a hashlock phrase` row in `Which hash?` with the method pick and
  the confirm copy. Lockstep test against the vendored Rust vectors.
- **H3 records.** Composer spec fold (§6c, §8 copy, §12 acceptance, §14 row,
  C25) and a new ms spec for the kind and verb, each under its own R0. `me`'s
  classifier learns the kind on the Rust side (rides the owed me 0.8.1, F-454).
- **H4 device acceptance.** Emulator capture arm, then the live walk: the
  operator types a phrase on the SH2 and its H is compared with the host's.

Out of scope this cycle: hash160/ripemd160/hash256 on the host (the composer
composes sha256 only); a preimage plate; a preimage as a device source; any
non-ASCII phrase; K-of-N shares of a preimage from the CLI (F-468).

### 4.2 `ms hashlock` (L9: "Agree with form")

Sources, exactly one per invocation:

- `--hashlock-phrase TEXT` (joins `SECRET_FLAGS`; refused on argv without
  `--allow-argv-secret`), `--hashlock-phrase-stdin` (one trailing LF/CRLF
  stripped), or `--in FILE` (the phrase from a file, same newline rule).
- `--hex HEX` or `--hex -`: an existing X, exactly 32 bytes (64 hex characters);
  anything else is refused naming §8i.
- `<ms1>` or `-`: a preimage-kind ms1, to re-derive H from a plate. An entr or
  mnem string is refused: "that is a seed backup, not a hashlock preimage".
- `--random`: 32 bytes from the OS CSPRNG (the one shares use). No phrase
  exists, so nothing can be guessed; the card says so.

Method (L5), for the phrase sources only: `--method hardened` = L4;
`--method sha256` = X = sha256(phrase bytes). Default: `hardened`, announced on
the card and in `--json` (section 5). `--hex`, `--random` and `<ms1>` take no
method: X is given.

Phrase rule, identical on host and device: non-empty, printable ASCII only, at
most 100 characters, bytes used exactly as typed (no trimming, case folding or
normalisation). Refusals name the rule and never echo the phrase. Under 20
characters the tool warns on stderr that anyone holding the template can guess
it, and proceeds.

Outputs:

- stdout: one line, `hash:<64 hex>`, the record `me sysw pack --in -` consumes.
  Public, so no stdout advisory.
- `--out FILE`: the preimage ms1, 0600, overwriting. `--out` is the preimage's
  channel; stdout is the digest's.
- stderr card (off with `--no-engraving-card`): the digest; the `sha256=`
  operand for `md compose --path`; the preimage as grouped ms1
  (`--group-size`/`--separator` apply) and as hex; the METHOD LINE to write
  down, verbatim, e.g.
  `preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)`
  or `preimage = SHA-256(phrase)`, with "write this next to your phrase; it is
  on no plate; if unsure later, try both"; the §8i and F-132 lines; the source
  kind without its value; the short-phrase warning when it applies.
- `--json`: digest, hash_record, sha256_operand, preimage_hex, preimage_ms1,
  source, method {kdf, hash, salt, iterations, dklen} or {hash}, phrase_chars.
  Carries the secret, so the private-key-material advisory fires, as
  `encode --json` does.

The other verbs on the new kind: `decode` prints kind, preimage hex and digest,
never words; `inspect` reports the kind; `derive` and `verify` refuse it with the
executable remedy `ms hashlock <ms1>`; `encode --hex` stays entr, so
`ms hashlock` is the only door into the kind; the codec supports shares of the
kind and a test pins it (CLI source deferred, F-468).

Versions: ms-codec 0.8.0 (new kind, corpus SHA re-pinned, MIGRATION section),
ms-cli 0.18.0.

## 5. Defaults taken for the operator's veto

| default | why | veto changes |
| --- | --- | --- |
| `--method` defaults to `hardened`, announced loudly (the bip48 precedent: permissive on input, expressive on output) | an unstated default is one a later reader "fixes"; requiring the flag adds friction to the safer route | a required flag, or `sha256` as default |
| salt `"ms-hashlock-v1"` | ASCII, short, domain-separated from BIP-39's `"mnemonic"`; must be copyable by hand | any fixed ASCII string; changing it after any vector ships is a new method |
| prefix byte `0x03` | the next unallocated value after `0x02`; `0x01` stays unallocated as MIGRATION.md records | `0x01` or higher |
| 20-character warning floor | ~40 bits of English is years per GPU at 100,000 iterations; below it a warning, never a refusal (the operator's choice) | a different floor, or none |
| `--out` carries the preimage ms1; stdout carries the digest | the secret goes to the private channel, the public record to the pipe | `--preimage-out` as a separate flag |
| `--json` reuses the PrivateKeyMaterial advisory | byte-parity with the toolkit; a preimage on a keyless path can spend alone | a new class (toolkit change) |
| `--random` included | one line over `getrandom`; the strongest form | drop it |
| the device asks the method AFTER the phrase, as a two-row pick `Hardened (about 10 s)` / `SHA-256` | mirrors the host flag; the wait is stated before it starts | ask before the phrase |

## 6. Sections still to walk with the operator

- 4.3 the kind byte in detail (codec API, vectors, MIGRATION, versions).
- 4.4 the device leg (rows, copy, the 10 s wait, scrub, the payload-ms1 class,
  §14 narrowing, firmware delta).
- 4.5 process and homes (brainstorm here; `mnemonic-secret/design/SPEC_ms_hashlock.md`
  for the kind and verb; the composer spec fold; plans and gates per stage; one
  implementer per stage, UC off; whole-diff review; staging pushes; releases;
  flash at the operator's word).
- 4.6 testing (vectors for both methods incl. `python3`/`openssl kdf`
  reproductions machine-checked; argv-guard and newline tests; refusals; the
  kind through decode/inspect/derive/verify/split/combine; Go lockstep; the
  touch-harness screen test; capture arm; the live walk).

## 7. Follow-ups filed from this brainstorm

- F-467 `hashvault-journey-hashlocks-unspendable` (section 3.2).
- F-468 `ms-split-no-preimage-source` (section 3.3).

## 8. Lessons

- A question about a layer the operator does not usually touch needs the layer
  named in the question: "how is X derived" read as "which script hash" until
  the off-chain step was spelled out. The confusion was the finding
  ([[walk-journeys-with-the-user]]).
- A number the project already measured on silicon (9,715 it/s) is worth more
  than any estimate; look for the benchmark before estimating
  ([[records-are-the-weak-half]]).
