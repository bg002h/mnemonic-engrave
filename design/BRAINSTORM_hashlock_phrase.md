# Brainstorm — the HASHLOCK PHRASE (`ms hashlock` + on-device entry)

**STATUS: BRAINSTORM COMPLETE 2026-09-03; PAUSED BEFORE SPEC (L19).** Rulings
below are the operator's and are final; everything else is measured context,
agreed design (4.1-4.6), or a default taken for the operator's veto (section
5). Two review lenses ran on it (section 7), each fold sonnet-verified. This
is a brainstorm record, not a spec: nothing may be implemented from it, and
the specs start only at the operator's word. Written mid-session so the
rulings outlive the context ([[decisions-must-outlive-the-agent]]).

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
| L18 | **A second lens before ruling on 4.6.** On the 4.6 (testing) draft: "Ask security software expert for review" | Single opus agent, security-software-engineering lens, over the whole record with 4.6 as presented; brief `design/agent-briefs/hashlock-brainstorm-R0-r2-security-software-brief.md`, report `design/agent-reports/hashlock-brainstorm-R0-r2-security-software-expert.md`. 4.6 was presented, not agreed, until the operator ruled after it (L23). |
| L19 | **"Pause before spec."** | After the L18 review is persisted, folded and walked with the operator, NO spec is started until the operator says so. The controller stops there. |
| L20 | **`--in FILE` on `ms hashlock` means the preimage ms1**, like the six reading verbs (chosen over "phrase plus an ms1-shape refusal" and "no `--in`"), on review C-1. | The phrase has exactly two channels: `--hashlock-phrase` (argv-guarded) and `--hashlock-phrase-stdin` (redirect a file into it). An ms1-shaped phrase is refused on both, naming `--in`/`-`. |
| L21 | **`--random` refuses unless `--out FILE` or `--json`** (chosen over "allow, the card is the copy"), on review C-3. | A preimage that reaches no persistent channel is data loss, so it gates; `--random --no-engraving-card` without either exits 64 naming `--out`. `--out`'s overwrite semantics stay as ruled 2026-08-26. |
| L22 | **The classifier lands Rust-first as H1b; the fork mirrors it; no new class this cycle** (chosen over "the fork ships a new class, me catches up"), on review I-4 + C-2. | H1b (engrave, before H2): `me`'s ms1 classifier treats kind `0x03` as inert with a vector row, in me 0.8.1. H2: the fork's `isStrictMs1` gains the same prefix test; `DecodeMS1` keeps refusing `0x03` and a separate decoder serves the new consumer. |
| L23 | **Section 4.6 (testing) stands**, with the r2 review's additions. "Yes, 4.6 stands." | The brainstorm is complete: sections 4.1-4.6 agreed, two lenses run and folded, each fold sonnet-verified. L19 now binds: STOP before any spec. |

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
- **H1b mnemonic-engrave (Rust, before H2; L22).** `me`'s ms1 classifier
  (`crates/me-cli/src/seal/record.rs`, the strict-ms1 test the fork mirrors)
  gains the prefix test: a kind-`0x03` string is INERT -- not a seed, not a
  new class -- with a record-class vector row. Ships in me 0.8.1 (F-454).
- **H2 seedhammer fork (Go port).** `DecodeMS1` KEEPS refusing `0x03`
  (review C-2: its five callers all discard the prefix and would turn a
  preimage into seed entropy); a separate `DecodeMS1Preimage` serves only the
  new consumer, with the Go length rule restated (the shared `case 16, 20, 24,
  28, 32` switch cannot be inherited). `isStrictMs1` mirrors H1b's prefix test
  so a preimage string is inert and reaches no screen by the existing
  absent-is-false admission rule. The `Type a hashlock phrase` row in
  `Which hash?` with the method pick and the confirm copy, label-keyed (4.4).
  Lockstep test against the vendored Rust vectors.
- **H3 records.** Composer spec fold (§6c, §8 copy, §12 acceptance, §14 row,
  C25) and a new ms spec for the kind and verb, each under its own R0; the
  F-467 journey regeneration.
- **H4 device acceptance.** Emulator capture arm, then the live walk: the
  operator types a phrase on the SH2 and its H is compared with the host's.

Out of scope this cycle: hash160/ripemd160/hash256 on the host (the composer
composes sha256 only); a preimage plate; a preimage as a device source; any
non-ASCII phrase; K-of-N shares of a preimage from the CLI (F-468).

### 4.2 `ms hashlock` (L9: "Agree with form")

Sources, exactly one per invocation:

- `--hashlock-phrase TEXT` (joins `SECRET_FLAGS`; refused on argv without
  `--allow-argv-secret`) or `--hashlock-phrase-stdin` (one trailing LF/CRLF
  stripped; a phrase file is redirected into it). These are the ONLY phrase
  channels (L20). Joining the guard is a three-part edit the record names
  (review I-1, I-2, M-3): `SUBCOMMANDS` (`[&str; 12]` -> 13) so the refusal
  and the purge pattern name `hashlock`; `override_applies`'s verb match so
  `--allow-argv-secret` works; `flag_class` so the refusal says "a hashlock
  phrase", not "a BIP-39 passphrase"; and the verb's `Source` is built
  `.on("--hashlock-phrase")` so the admitted value arrives through the side
  channel instead of whatever stdin holds (the gate: the same invocation with
  stdin at `/dev/null` still derives from the flag's value). With stdin at a
  terminal, `--hashlock-phrase-stdin` prints one prompt line to stderr
  ("Type the hashlock phrase, then Enter.") rather than blocking silently
  (review M-7; the constellation's recorded `mt` finding).
- `--hex HEX` or `--hex -`: an existing X, exactly 32 bytes (64 hex characters);
  anything else is refused naming §8i.
- `<ms1>`, `-`, or `--in FILE`: a preimage-kind ms1, to re-derive H from a
  plate -- `--in` means the ms1 here as on the six reading verbs (L20; review
  C-1: the argv guard's remedy for a refused plate string prints
  `ms hashlock --in FILE`, so this channel MUST take the plate). An entr or
  mnem string is refused: "that is a seed backup, not a hashlock preimage".
- `--random`: 32 bytes from the OS CSPRNG (the one shares use; `getrandom`,
  failing closed). No phrase exists, so nothing can be guessed -- and nothing
  can be remembered: the card says both halves (review M-5): "No phrase exists,
  so nothing can be guessed, and nothing can be remembered. This plate is the
  only copy." **`--random` REFUSES (exit 64, naming `--out`) unless `--out
  FILE` or `--json` is given** (L21; r2 review C-3: with the card suppressed
  or redirected to `/dev/null` the digest would reach a payload while its
  preimage existed nowhere -- a data-loss rule, so it gates).

Method (L5), for the phrase sources only: `--method hardened` = L4;
`--method sha256` = X = sha256(phrase bytes). Default: `hardened`, announced on
the card and in `--json` (section 5). `--hex`, `--random` and `<ms1>` take no
method: X is given, so `--method` WITH any of them is refused at exit 64
(review r2 M-6: a flag the operator set that does nothing is a defect), the
card's method line reads "preimage supplied", and `--json` omits the `method`
key for those sources. **The two methods get different warnings (review C-1,
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
different X); the device's phrase screen applies the same check. **A phrase
that is ms1-shaped is refused on both phrase channels** naming `--in`/`-`,
reusing `argv_guard::is_ms1_shaped` so the two predicates cannot drift (r2
review C-1: a plate string pasted as a phrase derives a preimage on no plate).
**The phrase channels use a NEW byte-verbatim reader** -- bytes as given,
exactly one trailing `\r?\n` stripped, nothing else -- and never
`parse::read_input` (strips all whitespace plus `-` and `,`) or
`parse::read_phrase_input` (trims and collapses runs) (r2 review I-3: either
would silently change X and every codec vector would still pass). No entropy
is gained past 64 characters (section 3.4); the cap is a usability bound, not
a security one.

Outputs:

- stdout: one line, `hash:<64 hex>`, the record `me sysw pack --in -` consumes.
  Public, so no stdout advisory. **`--out` never suppresses it** (r2 review
  I-5: `encode` suppresses its stdout artifact when `--out` is given because
  both are the same secret; here the two channels carry different artifacts,
  and copying encode's shape would hand `me sysw pack` an empty stream).
- `--out FILE`: the preimage ms1, 0600, overwriting. `--out` is the preimage's
  channel; stdout is the digest's.
- stderr card (off with `--no-engraving-card`): its FIRST line names it as
  carrying the preimage (r2 review M-1: on this verb the polarity is inverted,
  stdout public and stderr secret, so `2>>log` or `2>&1 | tee` lands the
  preimage in a 0644 file and nothing else labels it); then the digest; the
  `sha256=` operand for `md compose --path`; the preimage as grouped ms1
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
  and `sha2`, both pure Rust and already trusted by `me`, spelled exactly as
  `me` spells them (`pbkdf2 = { version = "0.12", default-features = false,
  features = ["hmac"] }`, `sha2 = "0.10"`; no direct `hmac`, no
  `password-hash` -- r2 review N-3, measured: pbkdf2 0.12.2's default feature
  set is exactly `["hmac"]`, MSRV 1.60, pure Rust, no build script). `ms-cli`
  reaches it
  through its existing path + version dependency (pin `=0.8.0`); `ms hashlock`
  is a thin verb: flags, private channels, refusal text, output shape.
- **Vectors.** Encode/decode round trip, share round trip, inspect kind, and for
  each method phrase -> X -> H with the `python3` and `openssl kdf`
  reproductions RUN as a test, not quoted (measured 2026-09-03 for the phrase
  `correct horse battery staple`: hardened X
  `c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`
  byte-identical in both tools, hardened H
  `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`; sha256
  X `c4bbcb1f...d4e39a8a`, H `b867db87...edbc96cb` = the W-5 record; both
  methods pin BOTH X and H, r2 review N-1). Length rows for `0x03` payloads of
  16, 32, 34 and 46 bytes, each refused by name (review I-2; BIP-93's bracket
  reaches 16..46 payload bytes). Lockstep rows a 100-character phrase derives
  byte-identically on host and device and a 101-character one is refused on
  both (review M-6), plus the 64-hex refusal on both. The `python3` +
  `openssl kdf` reproduction test lives in ms-codec and runs in the
  `test (ms-codec)` job, which is Ubuntu-only; that job gains a preflight STEP
  (`openssl kdf --help`, `python3 -c 'import hashlib'`) so a missing tool
  fails the step, never a test someone can `#[ignore]` (r2 review I-6: the
  ms-cli matrix includes macOS, whose stock `openssl` is LibreSSL without
  `kdf`). The corpus SHA is re-pinned, which is what forces the minor bump.
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
  within the modal-fits assertion.) **The row switch becomes label-keyed**
  (named row indices computed once), the `default` arm is unreachable rather
  than "clear the lock", and the §8i modal's condition is restated as "the
  operator is taking a hash", not an index comparison (r2 review C-4: the
  shipped `composerHashEdit` keys on `len(digests)` arithmetic and its
  fallthrough clears the hash, so inserting a row makes `Type 64 hex` silently
  remove the lock). Tests cover every row by label, the displaced ones
  included.
- **Phrase screen.** The existing four-page printable-ASCII keyboard, built
  with `NewPassphraseKeyboard` (not `NewTextKeyboard`, which carries a settings
  gear and a newline key, nor `NewLineKeyboard`), driven by a NEW flow
  function rather than `passphraseEntryFlow` (r2 review M-4: that flow
  hard-codes the "Passphrase" title, the pass-proof trigger and an over-length
  message about plate legibility). Titled `Hashlock phrase`, counter `n/100`
  against the dedicated `HASHLOCK_PHRASE_MAX_CHARS`, non-clamping as the
  passphrase flow is (so `101/100` is visible and the lockstep row is
  constructible). OK applies the host's rule byte for byte: non-empty, ASCII
  only, at most 100, the 64-hex refusal naming the `Type 64 hex` row, and the
  ms1-shape refusal. Back returns to `Which hash?`.
- **Method pick.** A two-row pick after the phrase: `Hardened (about 10 s)`
  and `SHA-256`. SHA-256 shows the brainwallet modal before deriving; Hardened
  under 20 characters shows the 72-days modal. Both confirm-to-proceed, never
  refusals (L12).
- **Derivation.** Hardened runs PBKDF2 on the countdown screen the sealed
  payload already uses (`gui/unlock_kdf.go:221-236`: "Unlocking. About N
  seconds left.", driven by the measured rate; retitled) -- through a NEW
  driver taking `salt []byte` and the iteration count, NOT `unlockDerive`'s
  `seal.Header` (r2 review M-5: its `Salt [16]byte` would zero-pad the
  14-byte salt and every device digest would silently diverge; only a
  comparison against the vendored corpus CONSTANT catches it, never a value
  recomputed by the same Go function). SHA-256 is instant.
- **Confirm modal.** `hash  first8..last8`, the §8i line, the F-132 line, and
  the section 3.7 lines. CONTINUE sets the path's hash; Back discards (safe by
  construction: nothing is assigned before CONTINUE, and Back or power loss
  during the derivation leaves the composer state untouched -- r2 Q3). The
  digest shown must equal the host's for the same phrase and method: the
  lockstep vector. The 64 visible bits are a transcription check, adequate
  ONLY because the full-width lockstep vector runs in CI; the H4 walk records
  both full digests, so nobody later drops the vector and keeps the walk (r2
  review N-2).
- **No scrub discipline** for the phrase or X beyond what the composer already
  does by construction (L15).
- **Payload preimage strings.** An ms1 of kind `0x03` in a payload is INERT
  (L22): `isStrictMs1` gains the prefix test, mirroring H1b's Rust change, so
  the string is never `ClassCodex32Secret` and reaches no screen by the
  existing absent-is-false admission rule; no new class this cycle.
  `DecodeMS1` keeps refusing `0x03` (r2 review C-2: all five callers discard
  the prefix, and "Show secret" is gated on `err == nil`), so a typed or
  scanned preimage string is refused everywhere a seed is expected, as today.
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
  lockstep -> H1b (engrave: `me`'s classifier treats kind `0x03` as inert,
  vector row, me 0.8.1 via its staging ritual; L22) -> composer spec fold R0
  -> H2 plan -> one H2 implementer in a fork worktree -> review -> merge ->
  flash at the operator's word -> the H4 walk with the operator -> H3 closes
  the records (the F-467 journey regeneration).
- **Tiers and reports.** Opus for spec and plan lenses and whole-diff reviews,
  sonnet for fold verification and pushes, fable not a tier. Every agent
  writes its own report into the reviewed repo's `design/agent-reports/`.
- **Rust-primary pins.** The fork's `0x03` arm and derivation carry a
  provenance pin to the ms-codec 0.8.0 commit; the hashlock vector corpus is
  vendored into the fork with a pin test, as the compose vectors are.

### 4.6 Testing (L23: "Yes, 4.6 stands", after the L18 review's additions)

- **Codec vectors (H1), the corpus with its SHA pin.** Kind `0x03`
  encode/decode/inspect; the share round trip through the codec API; length
  rows 16, 32, 34 and 46 payload bytes each refused by name; id `hash` on
  singles and its blocklist entry. Derivation rows for both methods: the W-5
  phrase, a 1-character phrase, 20 characters, 64 and 65 (the HMAC block
  boundary), 100 and 101, a phrase with leading, trailing and double spaces,
  the 64-hex refusal, a non-ASCII refusal, empty. A test executes the
  `python3` and `openssl kdf` reproductions and FAILS if either tool is
  absent, so a skip can never print ok.
- **CLI (H1).** Argv guard for `--hashlock-phrase` (refused without the allow
  flag, the value never echoed); stdin stripping of exactly one LF or CRLF;
  `--in FILE`; `--hex` at 63, 64 and 65 characters; entr and mnem strings
  refused, kind 3 accepted; `--random` twice gives two records; two sources
  exit 64; stdout is exactly the record line; `--out` is 0600 and overwrites;
  the card's contents per method including both `--random` halves and the
  section 3.7 lines; `--json` schema and advisory; `decode`, `inspect` and
  `combine` on the kind; `derive` and `verify` refuse with the remedy; one
  test per `unreachable!` site that panics on 0.17.x; MSRV, clippy, fmt; the
  man page carries the verb; the toolkit manual's flag-coverage lint passes.
- **Review gates.** The plan's tests lens mutates them before any code; the
  whole-diff review's mutation pass proves each guard fails on its named
  mutation with the output pasted.
- **Fork (H2).** `DecodeMS1` unchanged (a `0x03` string still refused at all
  five callers, one test each); the new `DecodeMS1Preimage` and its own
  length rule; `isStrictMs1`'s prefix test with the record-class vector row
  asserting an INERT classification (never `ClassCodex32Secret`, no new
  class; L22); derivation lockstep against the vendored corpus for both
  methods including the 100/101 and 64-hex rows; the phrase screen driven through the real flow
  on the touch harness (tap the new row, type, pick the method, confirm; the
  path's hash equals the vector's); geometry tests for the confirm modal and
  the no-payload hint; firmware size at the gate; an emulator capture arm
  that types a phrase and compares the digest row, with a negative control
  that must fail on a different phrase.
- **Device (H4).** The live walk with the operator: type the phrase on the SH2
  under each method and compare `first8..last8` with `ms hashlock` on the
  host; pack a record made by `ms hashlock` and see it offered as a row.
- **Records (H3).** The regenerated hashvault journey's digests equal
  `ms hashlock`'s; the plan cite, glyph and step-reference checks; the
  composer spec's new §12 rows.
- **Added by the r2 security-software review (each names the mutation it
  catches).** H1: an ms1-shaped phrase on both phrase channels is refused
  naming the ms1 route (mutation: delete the shape check on one channel);
  `--random --no-engraving-card` without `--out`/`--json` exits 64 naming
  `--out`, and `--random 2>/dev/null` likewise (mutation: drop the
  persistent-channel rule); `--hashlock-phrase X --allow-argv-secret` derives
  from X, and the same invocation with stdin at `/dev/null` still derives from
  X (mutations: `hashlock` missing from `override_applies`; `.on()` missing);
  `... --allow-argv-secret < other.txt` never derives from `other.txt`; the
  guard's refusal text for `ms hashlock` says "hashlock" and "a hashlock
  phrase" (mutations: `SUBCOMMANDS` not extended; `flag_class` arm missing);
  byte-exact rows `"  a  b "` and `"a-b,c"` through BOTH phrase channels --
  `--hashlock-phrase-stdin`, and `--hashlock-phrase` under
  `--allow-argv-secret` via the admitted side channel -- equal the codec
  vector (mutation: swap in `read_phrase_input` or `read_input` on either
  channel -- no codec vector can catch it; r3 verification finding 3); the stdin newline rows
  `"p\n"`, `"p\r\n"`, `"p"`, `"p \n"` strip one `\r?\n` and nothing else,
  `"p\n\n"` refused; NEGATIVE-CONTENT rows, one per refusal (empty,
  non-ASCII, over 100, 64-hex, ms1-shaped, `--hex` wrong length, wrong ms1
  kind, two sources, `--method` with a given X): the phrase and the preimage
  appear in neither stdout, stderr, nor the `--json` error envelope on stdout
  (mutation: a refusal built with `format!("... {phrase}")`); "stdout is
  exactly the record line" runs WITH `--out` as well (mutation: copy encode's
  suppression); the `--json` `method` shape per source; the downgrade row (a
  `0x03` string on the 0.17.x-equivalent codec refuses, never panics); the CI
  preflight step in `test (ms-codec)`. H1b: the record-class vector row for an
  inert `0x03` string in `me`. H2: `DecodeMS1Preimage` length rows for `0x03`
  at 16, 20, 24, 28, 34 and 46 payload bytes each refused (mutation: reuse
  `DecodeMS1`'s shared length switch instead of the 32-only rule; `DecodeMS1`
  itself stays untouched and refuses every `0x03` string, r3 verification
  finding 2); one test per `DecodeMS1` caller
  (`gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`,
  `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`,
  `bundle/verify.go:138`) that a `0x03` string is refused and "Show secret"
  is not offered; `sysw.Classify` on a `0x03` ms1 is never
  `ClassCodex32Secret` and `admits` is false for all ten programs (mutation:
  leave `isStrictMs1` unchanged); every `Which hash?` row by label reaches its
  own screen, `Type 64 hex` sets and never clears, the §8i modal fires for
  the three take-a-hash rows and not for `No hash lock` (mutation: insert the
  row without re-keying the switch); the device derivation test compares
  against the vendored corpus CONSTANT (mutation: zero-pad the salt into a
  16-byte header); widget identity (no gear, no newline key, title `Hashlock
  phrase`, no "plate" in the over-length text); the 101-character and 64-hex
  refusals driven through the real screen with the counter at `101/100`. H4:
  the walk records both methods' full 64-hex digests, not only
  `first8..last8`.

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
| the stderr card's first line names it as carrying the preimage (r2 M-1) | this verb inverts `ms`'s polarity (stdout public, stderr secret) and nothing else labels the stream; secret-handling class | drop |
| `--hashlock-phrase-stdin` on a terminal prints one prompt line to stderr (r2 M-7) | the first `ms` input a human is meant to type must not look like a hang; pre-existing shape on `--passphrase-stdin` is left as is | drop, or refuse a tty |
| `--method` with `--hex`/`--random`/`<ms1>` is refused at exit 64 (r2 M-6) | a flag the operator set that does nothing is a defect elsewhere in this codebase | ignore it silently |
| `DecodeMS1` unchanged; `DecodeMS1Preimage` for the new consumer (r2 C-2) | five callers discard the prefix; one added function keeps all five fail-closed with no audit | an arm in `DecodeMS1` plus a five-site checklist |
| `--out` carries the preimage ms1; stdout carries the digest | the secret goes to the private channel, the public record to the pipe | `--preimage-out` as a separate flag |
| `--json` reuses the PrivateKeyMaterial advisory | byte-parity with the toolkit; a preimage on a keyless path can spend alone | a new class (toolkit change) |
| `--random` included, its card saying both halves ("nothing can be guessed, and nothing can be remembered; this plate is the only copy"; review M-5) | one line over `getrandom`, failing closed; the strongest form and the worst loss profile | drop it, or drop the second half |
| the device asks the method AFTER the phrase, as a two-row pick `Hardened (about 10 s)` / `SHA-256` | mirrors the host flag; the wait is stated before it starts | ask before the phrase |
| the section 3.7 lines on the host card and in the device's confirm modal: "One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself." (review I-5 sharpened the controller's addition) | the tool cannot detect reuse (it never sees other policies or passwords), so the copy is the whole defence | drop or reword |

## 6. What remains before the spec

- Nothing in this record: 4.1-4.6 are agreed (L9, L10, L16, L17, L23), both
  lenses are folded and verified (section 7).
- STOP (L19: "Pause before spec"). `SPEC_ms_hashlock.md` and the composer
  spec fold start only at the operator's word.

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

### 7.1 Round 2: security software engineering lens (report `hashlock-brainstorm-R0-r2-security-software-expert.md`, persisted e9d7895; 4C/6I/7M/3N)

Controller machine-checks before folding: `override_applies` matches a fixed
list of eight shipped verbs and `SUBCOMMANDS` is `[&str; 12]`; `is_ms1_shaped`
= `starts_with("ms1")` + bech32 charset after separator stripping; the guard's
remedy prints `ms {verb} --in FILE`; `flag_class` falls through to "a BIP-39
passphrase"; `parse.rs` has `read_input` (strips all whitespace, `-`, `,`),
`read_phrase_input` (trims, collapses), `read_in_file` (strips nothing) and
`Source` defaults `channel: ""`; `rust.yml`'s `test (ms-codec)` job is
Ubuntu-only and the ms-cli matrix is ubuntu + macos; the fork's five non-test
`DecodeMS1` callers all bind `_` for the prefix; `showSecret := f.Unshared &&
msErr == nil`; `sysw/classify.go:48` tests `isStrictMs1` first; six
`ClassCodex32Secret` admissions; `admits` is `admitted[p][c]` (absent = false);
`unlockDerive(ctx, th, h seal.Header, pass)` with `SaltLen = 16`; the hardened
H for the W-5 phrase is `3cf5d421...4c12`.

| finding | disposition |
| --- | --- |
| C-1 `--in` ambiguity; the guard's own remedy routes a plate string into the phrase channel | RULED L20: `--in`/`-`/positional = the ms1; phrase via `--hashlock-phrase` and `--hashlock-phrase-stdin` only; ms1-shaped phrases refused on both (4.2) |
| C-2 a `0x03` arm in `DecodeMS1` fails open at five prefix-discarding callers; `isStrictMs1` classifies a preimage as a seed | RULED L22 + default: `DecodeMS1` unchanged, `DecodeMS1Preimage` for the new consumer; `isStrictMs1` gains the prefix test mirroring H1b; Go length rows (4.1, 4.4, 5) |
| C-3 `--random` can emit a digest whose preimage exists nowhere | RULED L21: refuse unless `--out` or `--json` (4.2) |
| C-4 the new row shifts an index-keyed switch whose default clears the lock | label-keyed rows, unreachable default, §8i condition restated, tests for every row by label (4.4, 4.6) |
| I-1 `--allow-argv-secret` inert on an unlisted verb; refusal names `encode` | the three `argv_guard.rs` edits named (4.2) |
| I-2 the side channel must be opted into or stdin is read | `.on("--hashlock-phrase")` named; the `/dev/null` gate (4.2, 4.6) |
| I-3 both shipped readers normalise; no test would notice | a new byte-verbatim reader; `read_input`/`read_phrase_input` forbidden; byte-exact CLI rows (4.2, 4.6) |
| I-4 the fork's classifier would lead the Rust primary | RULED L22: H1b in engrave before H2; no new class (4.1) |
| I-5 `--out` vs stdout unstated; encode's precedent inverts it | `--out` never suppresses the stdout digest (4.2, 4.6) |
| I-6 the reproduction test's job unnamed; macOS lacks `openssl kdf` | ms-codec's Ubuntu job with a preflight step (4.3, 4.6) |
| M-1 the card is unlabelled on a polarity-inverted verb | first line names the preimage (4.2, 5) |
| M-2 JSON errors go to stdout | negative-content rows cover the envelope (4.6) |
| M-3 `flag_class` misnames the material | with I-1 (4.2) |
| M-4 keyboard constructor and flow unnamed | `NewPassphraseKeyboard` + a new flow (4.4) |
| M-5 `unlockDerive` would zero-pad the salt | a new driver taking salt bytes; the constant-comparison rule (4.4, 4.6) |
| M-6 `--method` with a given X unspecified | refused at exit 64; `--json` omits `method` (4.2, 5) |
| M-7 a tty stdin blocks with no prompt | one stderr prompt line (4.2, 5) |
| N-1 hardened H unpinned | pinned (4.3) |
| N-2 64 visible bits | adequacy rests on the CI vector; the walk records full digests (4.4, 4.6) |
| N-3 `hmac` not a direct dep | `me`'s spelling copied verbatim (4.3) |
| reviewer question 5 (card first line) | taken (M-1) |

Lenses run on this record: cryptography + Bitcoin programmer (opus, r0);
fold verification (sonnet, r1: `hashlock-brainstorm-R0-r1-fold-verification.md`,
persisted 95e7423 -- FIXED 16 / PARTIAL 1 / NOT 0, every recomputed number
matched, one new Important: the first fold said all four `unreachable!` sites
become refusals while 4.2 says `decode` prints a preimage; folded above; the
r1 fold was wording only, no round on it); security software engineering
(opus, r2, above; folded with three rulings); fold verification (sonnet, r3:
`hashlock-brainstorm-R0-r3-fold-verification.md`, persisted e06dd15 -- FIXED
20 / PARTIAL 1 / NOT 0, every citation and number matched; the PARTIAL and
two Importants were one stale 4.6 fork bullet still describing the
pre-review H2, a length-row test naming `DecodeMS1` instead of
`DecodeMS1Preimage`, and the byte-exact row naming one phrase channel; all
three folded above, wording only, no further round). The controller's
propagation grep after the r2 fold had scoped itself to one section; this
fold's grep ran over the whole record. The operator ruled 4.6 stands (L23).
Lens-closure holds; the journey lens belongs to the spec, which will carry
the walks. STOP (L19).

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
