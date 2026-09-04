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
| L5 | **Two methods, the operator's choice.** "User should be allow to choose between hardened kdf as above or just sha256 to go from hashlock phrase to primate." | `--method hardened` (L4) or `--method sha256` (X = sha256(phrase bytes)); the device asks the same question as a two-row pick. The card and `--json` always name the method used. If the method line is lost, try each method that shipped with the version named on the card: one derivation per method, one digest matches the policy (wording per review M-4). |
| L6 | **The preimage gets its own ms1 kind byte** (chosen over "plain entr-32 plus a note" and "no ms1 form"). | ms-codec 0.8.0 claims prefix `0x03` = hashlock preimage, 32 bytes only; `ms decode` never prints it as words; the device never offers it as a seed. |
| L7 | **Device scope this cycle = digest only** (chosen over "digest plus a preimage plate" and "digest plus preimage as a source"). | The SH2 derives H from a typed phrase, uses it, shows `first8..last8`, states §8i and F-132, and scrubs X. It never stores, shows, engraves or sources a preimage. |
| L8 | **32 bytes = 64 hex characters.** "Do we mean 64 hex chars or 32?" Both spellings name the same value. | Every refusal and help line says "32 bytes (64 hex characters)". |
| L9 | **The `ms hashlock` surface in section 4.2 is agreed as a form.** "Agree with form." | Section 4.2 is the spec's input; defaults in section 5 remain vetoable. |
| L10 | **The preimage kind and the codec placement (section 4.3) are agreed.** Asked "If in codec, can cli use it?": yes, `ms-cli` already depends on `ms-codec` by path + pin and every verb calls the codec's public API; the pin moves to `=0.8.0`. "Looks good." | The derivation (`ms_codec::hashlock`) and the kind live in ms-codec; `ms hashlock` is a thin verb over them. |
| L11 | **Review before anything else.** "Document brainstorm and send it to an opus cryptography bitcoin programmer expert for review -- single agent." | Brief: `design/agent-briefs/hashlock-brainstorm-R0-crypto-review-brief.md`; report: `design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md` (1C/6I/6M/2N; dispositions in section 7). |
| L12 | **`--method sha256` warns always, never refuses** (chosen over "warn always, refuse under 20" and "warn plus an acknowledgement flag"), on review C-1. | Every sha256 use prints the brainwallet line; the operator's choice stands at any length. The hardened method keeps its warning under 20 characters, with the corrected figure (section 3.4). |
| L13 | **No `--salt` flag this cycle** (chosen over an optional operator salt), on review I-1. | The fixed salt stays; the shared-table consequence is recorded (section 3.4) and carried in the copy ("choose the phrase from a generator"); `--salt` filed as F-469. |
| L14 | **Preimage singles carry their own id `hash`** (chosen over keeping `entr` and mitigating in copy), on review I-4. | The plate reads `ms10hash...`, not `ms10entrsq...`; readers still dispatch on the prefix byte; `RESERVED_ID_BLOCKLIST` gains `hash`. |
| L15 | **No scrub discipline for the phrase or X on the device.** On the 4.4 draft's scrub bullet: "No. We don't need to scrub like we would for a sealed payload." | Consistent with C14 (no Sealed-Payload memory treatment for the composer's seeds); a phrase is less sensitive than a seed. The device leg adds no wiping beyond what the composer already does by construction; secret-handling is non-gating anyway (2026-08-27 ruling). |
| L16 | **Section 4.4 (the device leg) is agreed** without the scrub bullet. "Yes." | 4.4 is the composer spec fold's input. |
| L17 | **Section 4.5 (process and homes) is agreed.** "Ok on 4.5" | Two specs, one plan per stage, the order in 4.5. |

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
and tiers 1-2 of that (never funded) policy can never be spent. Filed F-467;
confirmed by the review with digests (Q7). Two facts the review added: BIP-174
puts NO length bound on `PSBT_IN_SHA256`'s preimage, so a coordinator carries a
40-byte "preimage" in a well-formed PSBT and the failure surfaces only at script
execution -- the PSBT layer is not a guard; and rust-miniscript's satisfier type
is `Preimage32 = [u8; 32]`, so the wrong satisfaction cannot even be expressed.
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

**Guessing rates (review I-1/C-1, from hashcat v6.2.6 on one RTX 4090:
PBKDF2-HMAC-SHA256 8,865.7 kH/s at 999 iterations; SHA-256 21,975.5 MH/s).**
At 100,000 iterations the hardened method is 8.9e4 guesses/s; the sha256 method
is 1.1e10 guesses/s (two hashes per guess): a ratio of 124,060. Assuming 2 bits
of entropy per character of a chosen English phrase:

| phrase | bits | hardened, expected | sha256, expected |
| --- | --- | --- | --- |
| 20 characters | ~40 | 72 days on one GPU | 50 seconds |
| 28 characters | ~56 | 12,900 years | 38 days |

The controller's earlier "years per GPU at 40 bits" was wrong by an order of
magnitude and is withdrawn. **The fixed salt (L4, L13) makes the grind
shareable:** one GPU precomputes the top 2^32 candidates in 13.5 hours, and
that table then breaks every ms1 hashlock ever made by lookup; a targeted
attacker pays the same 13.5 hours per target either way. RFC 8018 section 4.1
names precomputation as what a salt exists to prevent. The record carries this
so the copy can say the only real defence: choose the phrase from a generator
(six diceware words is ~77 bits) or use `--random`. Corollary from the review:
HMAC-SHA256 keys a phrase longer than 64 bytes by its SHA-256, so no entropy is
gained past 64 characters.

### 3.5 Who derives X at spend time

No hardware signer sees a hashlock phrase today: a coordinator able to populate
the PSBT's sha256-preimage field (BIP-174 `PSBT_IN_SHA256`, key type 0x0b: key =
the hash, value = the preimage) places the 32-byte X there and the signer only
signs. Ledger's app compiles the `sha256` fragment and has no preimage
derivation surface; Liana's own wallet model admits no hash fragment and Sparrow
has no documented preimage-entry UI (review N-2), so no coordinator is NAMED here
until one is verified. So the phrase -> X derivation runs on whatever host tool
the operator has at spend time, which is why the method must be in the external
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

### 3.7 A spent preimage is public (controller's addition after L10; confirmed and sharpened by review I-5)

Spending through a hash path reveals X in the witness, on-chain, forever.
Confirmed and sharpened by the review (I-5); four consequences the record
carries:

1. **One phrase per policy, never reused.** A phrase used for two policies
   gives both the same X, and the first spend hands the second policy's hash
   path to everyone.
2. **After a spend, that hash path is open to anyone** until the funds move
   (for a keyed path, still gated by the keys; for a keyless wsh path, C22, by
   nothing).
3. **H is public at the FIRST spend of ANY wsh path, not the hash path**
   (BIP-141: the whole witnessScript, every branch, is published). A wsh policy
   with a 2-of-3 key path and a keyless hash path publishes H on its first
   routine spend, and the grind against the phrase starts then. A `tr` leaf
   reveals nothing until that leaf is used (BIP-341). `sh(wsh)` is moot: the
   composer admits it only for a single unlocked, unhashed key set (spec §4a).
4. **A spent X is a permanent public oracle for the PHRASE.** Anyone can test
   candidate phrases against X directly, and a hit yields the text. So the
   phrase must never double as a BIP-39 passphrase, a sealed-payload
   passphrase or a password anywhere: the reverse direction is safe (the review
   confirmed no pre-spend leak in either direction), this is the only leak
   path. Copy: "Spending any path of a wsh wallet publishes this digest. Never
   use this phrase as a passphrase or a password anywhere else -- a spend
   publishes the preimage, and anyone can then test guesses at the phrase
   itself."

None is a codec question; all four are copy: card lines on the host and lines
in the device's confirm modal (section 5).

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
- `--random`: 32 bytes from the OS CSPRNG (the one shares use; `getrandom`,
  failing closed). No phrase exists, so nothing can be guessed -- and nothing
  can be remembered: the card says both halves (review M-5): "No phrase exists,
  so nothing can be guessed, and nothing can be remembered. This plate is the
  only copy."

Method (L5), for the phrase sources only: `--method hardened` = L4;
`--method sha256` = X = sha256(phrase bytes). Default: `hardened`, announced on
the card and in `--json` (section 5). `--hex`, `--random` and `<ms1>` take no
method: X is given. **The two methods get different warnings (review C-1,
L12):** under `sha256` the card ALWAYS carries the brainwallet line -- "This is
the brainwallet construction: anyone holding the digest tests 10^10 phrases per
second. A phrase a person chose is not safe here; use six diceware words or
--random" -- and never refuses. Under `hardened`, a phrase under 20 characters
gets the warning "a 20-character phrase falls in about 72 days on one GPU;
choose it from a generator", and the tool proceeds. Neither floor can see a
dictionary phrase, so the copy carries the weight and names the generator.

Phrase rule, identical on host and device: non-empty, printable ASCII only, at
most 100 characters (a dedicated `HASHLOCK_PHRASE_MAX_CHARS` on each side, not
the device's plate-legibility `passphrase.MaxLen`; review M-6), bytes used
exactly as typed (no trimming, case folding or normalisation). Refusals name the
rule and never echo the phrase. **A phrase that is exactly 64 characters, all
hex digits, is refused** naming `--hex` as the remedy (review I-6: it is a
preimage pasted into the wrong slot, and deriving from it silently yields a
different X); the device's phrase screen applies the same check. No entropy is
gained past 64 characters (section 3.4); the cap is a usability bound, not a
security one.

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
  on no plate; if the method line is lost, try each method that shipped with
  the version named on this card" (review M-4: "try both" would outlive its
  own precondition); the phrase's character count next to the method line
  (review M-2: the one signal that makes a stray space visible on the host;
  the device already shows n/100); the §8i and F-132 lines; the section 3.7
  lines (one phrase per policy; any wsh spend publishes the digest; never a
  passphrase or password elsewhere); the method's warning (sha256: always;
  hardened: under 20 characters); the source kind without its value.
- `--json`: digest, hash_record, sha256_operand, preimage_hex, preimage_ms1,
  source, method {kdf, hash, salt, iterations, dklen} or {hash}, phrase_chars.
  Carries the secret, so the private-key-material advisory fires, as
  `encode --json` does.

The other verbs on the new kind: `decode` prints kind, preimage hex and digest,
never words; `inspect` reports the kind; `derive` and `verify` refuse it with the
executable remedy `ms hashlock <ms1>`, **and the refusal sits on the
`Ok((tag, payload))` arm BEFORE the shared `payload_entropy_and_language`
helper** (review I-3: today that helper's `_ => unreachable!` would panic first);
`combine` gains its own FUNCTIONAL arm: a recovered preimage share set prints
as `decode` does (review N-1; its `_ => unreachable!` is the fourth site);
`encode --hex` stays entr, so `ms hashlock` is the only door that CREATES the
kind; the codec supports shares of the kind and a test pins it (CLI source
deferred, F-468).

Versions: ms-codec 0.8.0 (new kind, corpus SHA re-pinned, MIGRATION section),
ms-cli 0.18.0.

### 4.3 The preimage kind in ms1 (L10: "Looks good")

- **Wire.** Payload `[0x03][X:32]`, 33 bytes, so the string is 75 characters,
  the same as entr-32. **Length no longer implies kind** (review I-4): entr and
  mnem never share a length (50/56/62/69/75 vs 51/58/64/70/77), the preimage is
  the first kind that collides with entr on length and on the first payload
  character (`q`, since 0x00/0x02/0x03 share their top five bits). So preimage
  SINGLES carry their own id `hash` (L14): the plate reads `ms10hash...`, a
  seed plate `ms10entr...`, and the eye can tell a bearer instrument from a
  share. Readers still dispatch on the prefix byte; `RESERVED_ID_BLOCKLIST`
  gains `hash`. No misread converts one into the other: the codewords are at
  least nine characters apart and BIP-93 corrects at most four. A `0x03`
  payload of any length other than 33 bytes is refused by an explicit check
  BEFORE the variant is built (review I-2), with a new
  `Error::PreimageLengthMismatch { got }`; the entr length error would name a
  legal entr length as illegal, and the obvious `data[1..33]` indexing panics.
  The share axis is untouched: a K-of-N set of a preimage recovers to a `0x03`
  payload.
- **Codec API (ms-codec 0.8.0).** `Payload::Preimage(Zeroizing<[u8; 32]>)`
  (review M-1: a bare array has no `Drop` and is memcpy'd on every move, so the
  crate's caller-wrap recipe cannot scrub it; the wrapper keeps the length rule
  structural and scrubs on drop), built with `<[u8; 32]>::try_from(&data[1..])`
  after the length check, never slice indexing. Matching `PayloadKind` and
  `InspectKind` variants and arms in `dispatch_payload`, `payload_wire_bytes`
  and `validate`. `ReservedPrefixViolation` stops firing for `0x03`; any test
  that pinned it as reserved flips and is machine-checked at plan time.
  **`non_exhaustive` is a hazard here, not a help** (review I-3): ms-cli's
  four `_ => unreachable!` arms over `Payload` (`cmd/combine.rs:166`,
  `cmd/decode.rs:107` and `:112`, `cmd/payload_lang.rs:61`, measured
  `grep -rn '_ => unreachable' crates/ms-cli/src` = 4) absorb the new variant
  silently and panic at runtime, and `verify.rs:99` / `derive.rs:434` reach
  the last one before any refusal could run. The H1 plan carries the four as
  an explicit checklist with one test per site, split by what the verb is FOR
  (r1 verification, new finding 1): verbs that READ a secret print a preimage
  as a preimage -- `decode.rs:107`/`:112` gain a `Payload::Preimage` arm that
  prints kind, hex and digest, and `combine.rs:166` gains the same for a
  recovered preimage share set -- while the verbs that need a SEED refuse:
  `payload_lang.rs:61`, reached only from `verify` and `derive`, becomes the
  typed refusal with the `ms hashlock <ms1>` remedy.
- **Derivation lives in the codec.** `ms_codec::hashlock`: `preimage_hardened`,
  `preimage_sha256`, `digest`, with the salt and iteration count as named
  constants. The kind and its derivation share one corpus and one SHA pin, and
  the Go port pins its provenance against one crate. ms-codec gains `pbkdf2`
  and `sha2`, both pure Rust and already trusted by `me`. `ms-cli` reaches it
  through its existing path + version dependency (pin `=0.8.0`); `ms hashlock`
  is a thin verb: flags, private channels, refusal text, output shape.
- **Vectors.** Encode/decode round trip, share round trip, inspect kind, and for
  each method phrase -> X -> H with the `python3` and `openssl kdf`
  reproductions RUN as a test, not quoted (measured 2026-09-03 for the phrase
  `correct horse battery staple`: hardened X
  `c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`
  byte-identical in both tools; sha256 X `c4bbcb1f...d4e39a8a`, H
  `b867db87...edbc96cb` = the W-5 record). Length rows for `0x03` payloads of
  16, 32, 34 and 46 bytes, each refused by name (review I-2; BIP-93's bracket
  reaches 16..46 payload bytes). Lockstep rows a 100-character phrase derives
  byte-identically on host and device and a 101-character one is refused on
  both (review M-6), plus the 64-hex refusal on both. The corpus SHA is
  re-pinned, which is what forces the minor bump.
- **MIGRATION.md.** A 0.7 -> 0.8 section: readers that dispatch on the prefix
  byte MUST treat `0x03` as a 32-byte preimage and never as entropy; length no
  longer implies kind (I-4) and singles of the kind carry id `hash` (L14);
  every downstream crate MUST sweep its `_ => unreachable!` arms over `Payload`
  because `non_exhaustive` means the compiler will not (I-3); and the pre-tool
  recipe this project documented everywhere (spec §8i, W-5, F-465: "hash the
  passphrase to 32 bytes, then hash again") is `--method sha256`, NOT the
  default, so a digest made by hand before 0.18.0 reproduces only with that
  flag (review M-3; the same note goes in the manual chapter and F-465's
  `Which hash?` hint). Older readers, including every flashed SH2 before H2,
  reject the string as a bad prefix (`ReservedPrefixViolation` /
  `errMSBadPrefix`, both traced by the review), so the failure is a refusal
  and never a seed.

### 4.4 The device leg (F-466; L7 digest only; L15 no scrub; L16: "Yes")

- **Entry point.** `Which hash?` gains one row, `Type a hashlock phrase`,
  placed before `Type 64 hex`; the payload rows stay first. With no `hash:`
  record loaded, the screen's lead line names the host route: "No hash record
  in the payload. ms hashlock on the host makes one." (F-465's hint; ASCII,
  within the modal-fits assertion.)
- **Phrase screen.** The existing four-page printable-ASCII keyboard with
  reveal toggle and counter, titled `Hashlock phrase`, counter `n/100`
  against the dedicated `HASHLOCK_PHRASE_MAX_CHARS`. OK applies the host's
  rule byte for byte: non-empty, ASCII only, at most 100, and the 64-hex
  refusal naming the `Type 64 hex` row. Back returns to `Which hash?`.
- **Method pick.** A two-row pick after the phrase: `Hardened (about 10 s)`
  and `SHA-256`. SHA-256 shows the brainwallet modal before deriving; Hardened
  under 20 characters shows the 72-days modal. Both confirm-to-proceed, never
  refusals (L12).
- **Derivation.** Hardened runs PBKDF2 on the countdown screen the sealed
  payload already uses (`gui/unlock_kdf.go:221-236`: "Unlocking. About N
  seconds left.", driven by the measured rate; retitled). SHA-256 is instant.
- **Confirm modal.** `hash  first8..last8`, the §8i line, the F-132 line, and
  the section 3.7 lines. CONTINUE sets the path's hash; Back discards. The
  digest shown must equal the host's for the same phrase and method: the
  lockstep vector.
- **No scrub discipline** for the phrase or X beyond what the composer already
  does by construction (L15).
- **Payload preimage strings.** An ms1 of kind `0x03` in a payload classifies
  as a new secret class that reaches no screen; seed entry refuses it by name
  instead of failing at decode.
- **Cost.** SHA-256 and PBKDF2 are already linked; the keyboard exists. A
  small firmware delta, measured at the gate.

### 4.5 Process and homes (L17: "Ok on 4.5")

- **Records.** This brainstorm stays the rulings ledger. Two specs follow it:
  `mnemonic-secret/design/SPEC_ms_hashlock.md` (the `0x03` kind with id
  `hash`, `ms_codec::hashlock`, the verb, the vector corpus) under its own R0
  with correctness, adversarial and tests-vector lenses; and the composer spec
  fold in this repo (§6c third row and method pick, §8 copy for the four new
  modals, §12 acceptance rows, §14 row narrowed to storage, engraving and
  sourcing, C25 updated) under its own R0 with a journey lens that re-walks
  W-5 against the folded text.
- **Plans, one per stage, each build-gated and re-validated immediately before
  its implementer.** H1 gets a `plan-build-gate-ms.sh` sibling of the me and md
  gates on the pinned toolchain. H2 uses `plan-build-gate-go.sh` with its
  hand-wire script committed to the repo, not the scratchpad.
- **Order.** ms spec R0 -> H1 plan R0 -> one H1 implementer (opus, UC off, a
  mnemonic-secret worktree) -> whole-diff opus review -> fold -> sonnet
  verification -> merge through the ms staging ritual -> release ms-codec
  0.8.0 and ms-cli 0.18.0 per `design/RELEASE_PROCESS.md` (corpus SHA pin,
  CHANGELOG, MIGRATION, publish dry run, both tags), manual chapter in
  lockstep -> composer spec fold R0 -> H2 plan -> one H2 implementer in a fork
  worktree -> review -> merge -> flash at the operator's word -> the H4 walk
  with the operator -> H3 closes the records (me 0.8.1 with the classifier
  learning the kind; the F-467 journey regeneration).
- **Tiers and reports.** Opus for spec and plan lenses and whole-diff reviews,
  sonnet for fold verification and pushes, fable not a tier. Every agent
  writes its own report into the reviewed repo's `design/agent-reports/`.
- **Rust-primary pins.** The fork's `0x03` arm and derivation carry a
  provenance pin to the ms-codec 0.8.0 commit; the hashlock vector corpus is
  vendored into the fork with a pin test, as the compose vectors are.

## 5. Defaults taken for the operator's veto

| default | why | veto changes |
| --- | --- | --- |
| `--method` defaults to `hardened`, announced loudly (the bip48 precedent: permissive on input, expressive on output) | an unstated default is one a later reader "fixes"; requiring the flag adds friction to the safer route | a required flag, or `sha256` as default |
| salt `"ms-hashlock-v1"`, fixed, no `--salt` flag (RULED L13 after review I-1) | ASCII, short, domain-separated from BIP-39's `"mnemonic"` (different PRF, count, dkLen and role) and from `me`'s 16-byte random seal salt (different length, so `S || INT(i)` never coincides); must be copyable by hand; the shared-table cost is recorded in section 3.4 | the spelling only; changing it after any vector ships is a new method |
| prefix byte `0x03` | the next unallocated value after `0x02`; `0x01` stays unallocated as MIGRATION.md records | `0x01` or higher |
| 20-character warning floor for the HARDENED method only; the sha256 method warns at every length (L12) | ~40 bits of chosen English is 72 days on one GPU at 100,000 iterations (section 3.4; the earlier "years" was wrong); below it a warning, never a refusal (the operator's choice) | a different floor, or none |
| the brainwallet line under sha256 names the rate (10^10 per second) and the generator (six diceware words, or `--random`) | a floor cannot see a dictionary phrase; the copy is the defence | reword |
| a phrase of exactly 64 hex characters is REFUSED naming `--hex`, on host and device (review I-6) | deriving from a pasted preimage silently yields a different X and a valid-looking record | warn instead |
| the card prints the phrase's character count beside the method line (review M-2) | the one signal that shows a stray space on the host; the device shows n/100 | drop |
| `HASHLOCK_PHRASE_MAX_CHARS = 100` as its own constant on each side, lockstep-pinned (review M-6) | the device's `passphrase.MaxLen` is a plate-legibility number and can move for its own reasons | bind to `passphrase.MaxLen` |
| `--out` carries the preimage ms1; stdout carries the digest | the secret goes to the private channel, the public record to the pipe | `--preimage-out` as a separate flag |
| `--json` reuses the PrivateKeyMaterial advisory | byte-parity with the toolkit; a preimage on a keyless path can spend alone | a new class (toolkit change) |
| `--random` included, its card saying both halves ("nothing can be guessed, and nothing can be remembered; this plate is the only copy"; review M-5) | one line over `getrandom`, failing closed; the strongest form and the worst loss profile | drop it, or drop the second half |
| the device asks the method AFTER the phrase, as a two-row pick `Hardened (about 10 s)` / `SHA-256` | mirrors the host flag; the wait is stated before it starts | ask before the phrase |
| the section 3.7 lines on the host card and in the device's confirm modal: "One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself." (review I-5 sharpened the controller's addition) | the tool cannot detect reuse (it never sees other policies or passwords), so the copy is the whole defence | drop or reword |

## 6. Sections still to walk with the operator (after the fold verification of section 7)

- 4.6 testing (vectors for both methods incl. `python3`/`openssl kdf`
  reproductions machine-checked; argv-guard and newline tests; refusals; the
  kind through decode/inspect/derive/verify/split/combine; Go lockstep; the
  touch-harness screen test; capture arm; the live walk).

## 7. R0 round 0 dispositions (report `hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`, persisted d13819e)

Controller machine-checks before folding: the W-5 digest `b867db87..edbc96cb`
IS sha256(sha256("correct horse battery staple")); the hardened X for that
phrase reproduces byte-identically in `python3 hashlib.pbkdf2_hmac` and
`openssl kdf`; sha256 of that phrase is the private key of
`1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T` (pure-Python secp256k1, reproduced); the
four `_ => unreachable!` sites and the two callers exist at the cited lines;
`seal/wire.rs` has `SALT_LEN = 16`, `MIN_ITERATIONS = 100_000`; the device
keyboard's four pages plus space are exactly the 95 printable ASCII characters.
The hashcat figures were not fetched independently; they match public
benchmarks and the ratio arithmetic reproduces.

| finding | disposition |
| --- | --- |
| C-1 shared floor; sha256 is 124,060x cheaper to grind; the W-5 example is the brainwallet key | RULED L12: sha256 warns always, never refuses; hardened keeps the 20-character warning; copy names the rate and the generator (4.2, 5) |
| I-1 "years per GPU" wrong (72 days); fixed salt makes the grind a shared table | number corrected (3.4); RULED L13: no `--salt` this cycle, consequence recorded and in copy; F-469 |
| I-2 wrong-length `0x03` has no refusal path; obvious code panics | explicit length check before construction, `PreimageLengthMismatch`, `try_from`, rows 16/32/34/46 (4.3) |
| I-3 `non_exhaustive` hides four `unreachable!` sites; verify/combine panic first | the four sites are an H1 checklist with one test each: functional arms in `decode` and `combine` (a preimage prints as a preimage), a typed refusal in `payload_lang` behind `verify`/`derive`, placed on the `Ok` arm; MIGRATION tells downstreams to sweep (4.2, 4.3; the r1 verification caught the first fold saying "refusal" for all four) |
| I-4 75-character/`entr`/`q` collision with a seed plate | RULED L14: id `hash` for preimage singles; "length no longer implies kind" stated here for MIGRATION (4.3); the ms spec and the manual chapter carry it when H3 writes them (r1: PARTIAL against the first fold's wording, which claimed all three) |
| I-5 H public at the first wsh spend of any path; a spent X is a phrase oracle | 3.7 extended to four consequences; card and modal copy (5) |
| I-6 64 hex pasted into the phrase slot derives a different X silently | refusal naming `--hex`, host and device (4.2, 5) |
| M-1 `[u8; 32]` unscrubbable under the caller-wrap contract | `Zeroizing<[u8; 32]>` (4.3); secret-handling class, taken anyway |
| M-2 no character count on the card | printed beside the method line (4.2, 5) |
| M-3 the default diverges from the only documented recipe | MIGRATION + manual + `Which hash?` hint name `--method sha256` as the pre-tool recipe (4.3) |
| M-4 "try both" outlives its precondition | reworded to "each method that shipped with the version named on this card" (4.2) |
| M-5 `--random`'s card states half | both halves (4.2, 5) |
| M-6 the cap is a plate-legibility constant, independently editable | `HASHLOCK_PHRASE_MAX_CHARS` on each side, lockstep rows (4.2, 4.3, 5) |
| N-1 4.2's verb list omits split/combine | `combine` named with its arm (4.2) |
| N-2 Liana/Sparrow unverified | 3.5 names no coordinator; the PSBT field cited |
| Q3(c) PSBT bounds no preimage length | 3.2 |
| Q3(d) `sh(wsh)` moot | 3.7 |
| reviewer question 2 (floor in bits + generator) | taken: the copy names the generator; the count stays in characters because that is what the device counts (5) |
| reviewer question 5 (device 64-hex check) | taken: both sides (4.2) |

Lenses run on this record: cryptography + Bitcoin programmer (opus, r0);
fold verification (sonnet, r1: `hashlock-brainstorm-R0-r1-fold-verification.md`,
persisted 95e7423 -- FIXED 16 / PARTIAL 1 / NOT 0, every recomputed number
matched, one new Important: the first fold said all four `unreachable!` sites
become refusals while 4.2 says `decode` prints a preimage; folded above). The
r1 fold is wording only, so no r2 round (proportional re-review rule). The
brainstorm's R0 closes under lens-closure; the journey lens belongs to the
spec, which will carry the walks.

## 8. Follow-ups filed from this brainstorm

- F-467 `hashvault-journey-hashlocks-unspendable` (section 3.2).
- F-468 `ms-split-no-preimage-source` (section 3.3).
- F-469 `ms-hashlock-optional-salt` (section 3.4; L13).

## 9. Lessons

- A question about a layer the operator does not usually touch needs the layer
  named in the question: "how is X derived" read as "which script hash" until
  the off-chain step was spelled out. The confusion was the finding
  ([[walk-journeys-with-the-user]]).
- A number the project already measured on silicon (9,715 it/s) is worth more
  than any estimate; look for the benchmark before estimating
  ([[records-are-the-weak-half]]).
- The controller's own guessing-cost estimate ("years per GPU") was off by an
  order of magnitude because it assumed a GPU rate from memory; the reviewer
  cited a published benchmark. A security floor gets a cited rate and a stated
  entropy-per-character, or it is not a floor.
- The operator's own hand measurement (W-5) used the canonical brainwallet
  phrase without anyone noticing, which is the whole argument for the tool
  carrying the warning rather than the operator carrying the knowledge.
