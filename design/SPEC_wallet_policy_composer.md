# SPEC — Wallet Policy COMPOSER: on-device authoring of arbitrary tr/wsh wallet policies (spend-path grammar)

**STATUS: DRAFT 2026-09-01, R0 NOT YET RUN.** Two sections are marked PENDING
RECON (C27) and are filled from the two recon reports before the R0 loop starts.
Every ruling cited as `Cn` is the operator's, recorded verbatim in
`design/BRAINSTORM_wallet_policy_composer.md` section 2; every `file:line` was measured
against the heads below on 2026-09-01. Nothing may be implemented from this
document until it is GREEN (0 Critical / 0 Important) under the R0 loop and its
build gates have RUN.

| repo | head |
| --- | --- |
| fork `bg002h/seedhammer` | `169073c` |
| `bg002h/mnemonic-engrave` | `9100230` (brainstorm record + F-448/F-449) |
| `descriptor-mnemonic` | `790fc224` (md 0.14.0; md-codec 0.42.0 pinned by the fork) |
| `mnemonic-toolkit` | `d8f06483` |

Companions: `BRAINSTORM_wallet_policy_composer.md` (rulings + measurements),
`HANDOFF_arbitrary_tr_wsh.md`, `STAGED_PLAN_tr_wsh_concrete_descriptors.md`
(Stages 0-5 DONE; this is its D1 "compose later"), reviews
`agent-reports/composer-lowering-rules-bitcoin-expert-review.md` (`e9f2ba0`) and
`agent-reports/composer-lowering-i1-single-key-head-miniscript-review.md`
(`e0375f1`).

---

## 1. The gap, in one sentence

The SH2 can CONSUME any host-built tr/wsh wallet policy (Wallet Policy program:
structural summary, receive+change addresses, named wallet id) and can AUTHOR
only a `sortedmulti` k-of-n under wsh / sh(wsh) / sh from a seed (Multisig Build);
it cannot author a taproot policy, a timelock, a hashlock, or more than one
spend path (F-150 items 3 and 4). Everything below it already exists: the Go
`md` package serialises any tree (`md/encode.go:159` `writeNode`, all seven body
kinds) and emits Script for every fragment but `andor` and `pk_h`
(`md/script_emit.go`); what is missing is a way for a person to BUILD the tree by
tapping, and the Rust-first definition of what that tapping produces.

## 2. Rulings this spec implements

| ruling | one line | spec section |
| --- | --- | --- |
| C1 | authoring model = ordered list of spend paths; fixed lowering, no compiler | §4, §5 |
| C2 | the toolkit's five archetypes are PRESETS over the grammar, not the menu | §4d |
| C3 | firmware-resident vocabulary, defined Rust-first | §5, §10 |
| C4 | template first: compose the keyless shape, then seat keys | §7 |
| C5 | one slot, one path; a person in two paths holds two keys from two hardened accounts | §4b, §7d |
| C6 | front door = "Build" inside Wallet Policy; the door becomes a choice in every state | §7a |
| C7 | Multisig Build deprecated by COMMENT only, no enforcement | §8e, §9 |
| C8 | seating is payload-first (NFC later): slot-directed pick list of the payload's keys | §7d |
| C9 | teach the mk1 stub after authoring, unconditionally; both stubs after seating | §7c |
| C10 | engraved FORM is the operator's choice: concrete policy (text / QR / keyed md1) or template + key cards | §7f |
| C11 | timelock UI respects the very different ranges of the lock kinds | §6b |
| C12 | seeds (BIP-39 words or ms1) are a key source, via the unsealed payload or the keyboard | §7d |
| C13 | seed-derived slots get Full / Watch-only modes; the secret's plate form is the operator's choice | §7f |
| C14 | no Sealed-Payload-grade memory treatment; scrub on exit as Multisig Build does | §9 |
| C15 | the lowering lives in md-codec as `compose`, surfaced as `md compose` | §10 |
| C16 | grammar bounds approved; keyless paths and unsorted `multi` behind EXPERIMENTAL | §4 |
| C17 | one key: `pkh` in wsh, `pk` in tr (F-448 filed) | §5 |
| C18 | NUMS spelled raw `H` this cycle; BIP-388 gap documented; F-449 filed | §5c, §8f |
| C19-C23 | review findings adopted: first-appearance numbering, `or_i(pkh)` head, `or_d` only for a bare multi head, keyless wsh-only, lock-only refused, ≥1 keyed path, first-listed single key as internal key, lock ranges sourced | §5 |
| C24 | pack time reaches the device as a payload item, a LOWER bound on now | §6a, §6b |
| C25 | entry UX adopted: kind → unit → digits → echo; digit pad; `hash:` record or typed hex | §6b, §6c |
| C26 | Build with no payload is allowed: the result is a keyless TEMPLATE | §7b, §7e |
| C27 | recon fan-out: origin convention (1) and same-fingerprint import (2) dispatched; unhardened child (3) deferred | §13 |

## 3. Measured inventory (what exists, so no section re-derives it)

| capability | state | evidence |
| --- | --- | --- |
| Wallet Policy program: payload cards → gather → consent (summary, named id, receive+change) → review → engrave | shipped | `gui/wallet_policy.go:35-142` |
| Seat mk1 cards into a keyless template by declaration match | shipped | `gui/key_card_seating.go:53` |
| Serialise any tree to md1 (keyed or keyless) | exists, not exposed as a builder | `md/encode.go:159`, `:374`, `:461` |
| Emit Script for fragments | all but `andor`, `pk_h` | `md/script_emit.go` cases; `grep -c tagAndOr` = 0, `grep -c tagPkH` = 0 |
| Tap leaves via the same emitter | shipped (F-214) | `md/tapleaves.go:188` |
| Device derives use-sites | `<a;a+1>/*` and bare `*` only | `gui/md1_expand.go:149` |
| Form-aware wallet id and stub | shipped | `md/template_id.go:122,163` |
| Re-mint an mk1 card | exists | `mk/encode.go:39` `Encode` (deterministic) |
| Seed entry: payload words / typed words | shipped; ms1 legs exist elsewhere | `gui/derive_xpub.go:104`, `gui/gui.go:1262`, `gui/gui.go:2856` |
| Multisig Build: per-slot accounts by ordinal, passphrase per seed, Full/Watch-only | shipped | `gui/multisig_build.go:594-601`, `:738`, `gui/multisig_build_census.go:475` |
| Text plates (greedy packing) and 16-symbol structured-append QR plates | shipped in Engrave Transaction | `gui/transaction.go:1145`, `:1369`; `txqr.MaxSymbols = 16` |
| Payload: prefixed records `text:`/`pass:` with hex bodies, matched before sniffers, reserved | shipped | `SPEC_systemwide_payloads.md` section 5.3 |
| Payload region | 64 KiB | `sysw/wire.go:28` |
| Wallet Policy admission row | Descriptor + MDMK only | `gui/sysw_admit.go` |
| Device clock | none; `time.Now` drives timeouts only | `gui/gui.go:128,139,442` |
| Numeric entry widget | none; passphrase keyboard has a digits page mixed with punctuation | `gui/passphrase_keyboard.go:21` |
| md admission: nested `sortedmulti`/`sortedmulti_a` | refused; whole script / whole leaf only | brainstorm record section 3.8 |
| md admission: keyless tap leaf | refused ("All spend paths must require a signature") | brainstorm record section 3.7, review I2 |
| md admission: `older(0x400000)` | ACCEPTED (defect, filed `md-older-zero-time-units-not-refused`) | brainstorm record, C20 |

## 4. The grammar — NORMATIVE

A **policy** is an ordered list of 1 to 8 **spend paths** under one **wrapper**.

### 4a. Wrapper

| wrapper | admitted when |
| --- | --- |
| `tr` | any path list satisfying §4e |
| `wsh` | any path list satisfying §4e |
| `sh(wsh)`, `sh` | ONLY a single path that is an unlocked, unhashed `sortedmulti` (the Multisig migration, C7); n ≤ 15 for `sh` (Core `MAX_P2SH_SIGOPS`, review point 7) |

### 4b. Path

A path is `KEYS` ∧ optional `HASH` ∧ optional `LOCK`, where:

| element | bound | source |
| --- | --- | --- |
| `KEYS` | k-of-n over FRESH slots, n in 1..=9, 1 ≤ k ≤ n; every slot appears in exactly one path (C5) | `multi` ≤ 20 and `multi_a` ≤ 999 (rust-miniscript-fork `src/miniscript/limits.rs` lines 35 and 38), md1 32 per fragment (`crates/md-codec/src/tree.rs:92-120`) |
| `HASH` | at most one `sha256(H)`, H = 32 bytes | both reference wallets use sha256 only; `hash256`/`ripemd160`/`hash160` stay decodable, not composable |
| `LOCK` | at most one of `older` or `after`, values per §4c | C11, C20 |
| keyless path | `HASH` with or without `LOCK`, no `KEYS`: **wsh only, EXPERIMENTAL** (C16/I2) | a lock-only path (no keys, no hash) is REFUSED: anyone can spend after N (I3) |
| policy | at least one path has `KEYS` (I4) | BIP-388 l.191 |

### 4c. Lock values — SOURCED (C20)

| lock | admitted operand | meaning | sources |
| --- | --- | --- | --- |
| `older(n)`, blocks | n in 1..=65535 | n blocks, ≤ 455.1 days | BIP-68 l.30-40, 74-83 (bit 31 disable, bit 22 type, mask `0x0000ffff`); BIP-112 l.28-33; BIP-379 l.135 (`1 <= n < 2^31`) |
| `older(n)`, time | n = 0x400000 + u, u in 1..=65535 | u × 512 s, ≤ 388.4 days | same; BIP-68 l.46 (zero units = no lock) |
| `after(n)`, height | n in 1..=499,999,999 | block height | BIP-65 l.27, 243-250; Core `script.h:48` `LOCKTIME_THRESHOLD` |
| `after(n)`, time | n in 500,000,000..=2,147,483,647 | Unix time, 1985-11-05 .. 2038-01-19 UTC | BIP-379 l.135; rust-miniscript-fork `src/primitives/absolute_locktime.rs` line 10 |

Every other operand miniscript would accept is either masked by consensus to a
different lock or to no lock (`older(0x400000)`), and the composer never emits
one. The device enforces these tables itself (§6b); it does not rely on md's
downstream guard, which today misses the zero-units case.

### 4d. Presets (C2)

The five toolkit archetypes — simple-timelocked-inheritance, kofn-recovery,
tiered-recovery, hashlock-gated, decaying-multisig — are offered as one-tap
presets that POPULATE a path list the operator then edits. They are the same
spend conditions as `mnemonic build-descriptor`'s goldens but NOT byte-identical
to them (brainstorm record section 3.7): the composer's lowering (§5) applies uniformly.

### 4e. Structural refusals (before lowering)

| condition | outcome |
| --- | --- |
| no path with keys | REFUSE: "Every wallet needs at least one path with a key." |
| a path with neither keys nor hash | REFUSE: "A path with only a time lock means anyone can spend after it. Add a key or a hash." |
| keyless path under `tr` | REFUSE: "Taproot cannot hold a key-less path. Use wsh, or add a key." |
| more than 8 paths, or n > 9 in a path | REFUSE at the picker (the picker does not offer the value) |
| `sh`/`sh(wsh)` with more than one path or any lock/hash | REFUSE: "Legacy wrappers hold one plain multisig only. Use wsh or tr." |
| a taproot key-path slot that is also in a leaf | cannot occur (C5) |

## 5. The lowering — NORMATIVE (brainstorm record section 3.10; C19-C23)

"Lowering" is the FIXED, search-free translation from the path list to a BIP-388
descriptor template. It is defined in Rust first (§10) and ported to Go; the two
must produce byte-identical templates for every composable list.

| rule | wsh | tr |
| --- | --- | --- |
| paths combine | listed order, recursive, last path stands alone: `or_d(P, R)` iff `P` is a bare unlocked, unhashed `multi(k,…)` with n ≥ 2; otherwise `or_i(P, R)`. A bare single key is `or_i(pkh(K), R)`. Never `andor`, never `thresh` over paths | one leaf per path on a right spine in listed order `{P1,{P2,{P3,P4}}}`; path k at depth min(k, n−1) |
| inside a path | `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`, dropping absent parts | same |
| key set | unlocked single-path: `sortedmulti`; locked/hashed multi-key: `multi`; one key: `pkh` | unlocked whole leaf: `sortedmulti_a`; locked/hashed: `multi_a`; one key: `pk` |
| unsorted where sorted was legal | `multi` instead of `sortedmulti`, EXPERIMENTAL | `multi_a` instead of `sortedmulti_a`, EXPERIMENTAL |
| internal key | n/a | the FIRST-LISTED unlocked, unhashed one-key path (then not a leaf); otherwise NUMS |
| NUMS spelling | n/a | raw `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (C18); see §8f |
| placeholder numbering | `@i` by FIRST APPEARANCE in the emitted text; slot labels shown to the operator are these indices, computed after lowering | same; an extracted internal key is `@0` |
| keyless path | `and_v(v:sha256(H), LOCK)` or `sha256(H)` | refused (§4e) |
| use-site | `/<0;1>/*` on every slot | same |

### 5a. Why these and not others (measured, both reviews)

- `or_d(pkh(P1), R)` is dominated: +34 WU on every non-P1 spend AND it publishes
  P1's key. `or_i(pkh)` keeps the key hidden until P1 spends. The BIP-388/compiler
  form `or_d(pk(P1), R)` is 26 WU cheaper per P1 spend, key in cleartext; named
  as the drop-in if C17 is ever revisited (C21).
- `or_d` over a bare `multi(k)` head: −2 WU per head spend, +k WU per other
  spend, no key exposure (M3); kept (C23).
- Conjunct order is byte-identical while LOCK is last; LOCK anywhere else costs
  +1 byte (N1). Leaf text order at a node does not change the address (N2).

### 5b. Cross-check contract (for the Rust vectors, §10)

For every composable list the emitted template: parses in its context; passes
`sanity_check` (wsh keyless paths admitted via `ExtParams::top_unsafe()`, review
point 8); `lift()`s to the same semantic policy as `md compile` of the equivalent
Concrete policy; and survives `md encode` → `md decode` byte-identically (C1).

### 5c. NUMS and BIP-388 (C18)

The raw `H` spelling is valid in Bitcoin Core and imported by Nunchuk; it is NOT a
BIP-388 key placeholder, so BIP-388-strict registration and Liana refuse it. This
is a property of md's existing wire (`is_nums` flag, `crates/md-codec/src/tree.rs:51`) that
the composer inherits; the xpub form is F-449, its own constellation cycle. The
device says so in copy (§8f).

## 6. Inputs — NORMATIVE

### 6a. Three new payload record classes (host `me sysw pack` + device `seal.Classify`, lockstep)

All three follow `SPEC_systemwide_payloads.md` section 5.3: a reserved prefix, a
lowercase-hex body, matched BEFORE the sniffers, and a prefixed record whose body
is not valid hex is `ClassUnknown` and refused. None is secret.

| record | body | decoded meaning | class |
| --- | --- | --- | --- |
| `key:<hex>` | hex of the UTF-8 text `[fingerprint/path]xpub` (BIP-380 key-origin notation, the same line `mk encode --keys` reads) | one candidate key for seating; ORIGIN REQUIRED (an md1 slot carries a path; F-166 pathless is open); a bare xpub is refused naming the fix | `ClassKey` NEW |
| `hash:<hex>` | the 32-byte digest itself, 64 lowercase hex | one candidate `sha256` hashlock | `ClassHash` NEW |
| `now:<hex>` | hex of the UTF-8 text `<unix-seconds>[,<block-height>]` written by `me sysw pack` at pack time | the PACK time and optional height: a LOWER BOUND on the present (C24) | `ClassNow` NEW |

Why `key:` exists: today a bare xpub line packs as a `pkh(xpub)` single-sig
WALLET and a `[fp/path]xpub` line is refused (brainstorm record section 3.6, measured). Why
`now:` is a lower bound: the device has no clock; the record affects ONLY echoes
and refusals (§6b), never an encoded operand.

The admission table (`SPEC_systemwide_payloads.md` section 3.3.2) gains three columns
and Wallet Policy's row becomes: Mnem •, Cdx32 •, Passph •, Descr •, MDMK •,
Key •, Hash •, Now •. The enum comment at `gui/gui.go:191` ("came from OUTSIDE
this device", "needs neither a seed requirement") is rewritten (C12).

### 6b. Lock entry (C11, C24, C25)

Kind → unit → digits → echo, on a NEW digit-pad widget (digits, backspace,
done). The operator never types a raw operand.

| kind | unit | entry | encoding | echo |
| --- | --- | --- | --- | --- |
| relative | blocks | 1..65535 | `older(n)` | "N blocks (about D days)" |
| relative | days | 1..388 | `older(0x400000 + ceil(days*86400/512))` | "D days = U units of 512 s (D' days)" |
| absolute | height | 1..499,999,999 | `after(h)` | "block H" + lower-bound line if `now:` carries a height |
| absolute | date | `YYYY-MM-DD` | `after(unix at 00:00:00 UTC)`; ceiling 2038-01-19 | "DATE 00:00 UTC" + "at least N days after this payload was packed on <pack date>" when `now:` is present |

Refusals: blocks > 65535 or days > 388 → "Relative locks reach at most 455 days
in blocks or 388 days in time. Use an absolute date." A date or height BELOW the
`now:` value → "That is before this payload was packed. Choose a later date."
Without `now:` the echo shows the typed value alone; the copy never says "now".

### 6c. Hashlock entry (C25)

Primary: pick from the payload's `hash:` records. Fallback: type 64 hex on the
keyboard. The consent screen shows the digest. On-device preimage derivation is
DEFERRED (§14).

## 7. The flow — the C8 workflow, walked

### 7a. The door (C6)

Wallet Policy opens on a ChoiceScreen in EVERY state (today the no-payload path
drops straight into the NFC gather, `gui/wallet_policy.go:97`). Choices name the
route they take (F-437): "Scan cards", "From payload" (only when a payload is
loaded), "Build a new policy".

### 7b. Shape

Wrapper → preset or blank → paths. Per path: keys (n, k), lock (§6b), hash
(§6c). A path list screen shows each path as one line ("Path 2: 2-of-3 + 90
days"). Back preserves everything ("going back should lose nothing").

### 7c. Teach the stub (C9) — shown UNCONDITIONALLY after the shape is complete

```
Template-ID:  <32 hex>
mk1 stub (template):  <8 hex>
   mk encode ... --policy-id-stub <8 hex>
```

After seating (§7d) the keyed policy's id and stub are added, and the screen
recommends stamping BOTH stubs on each key card (`--policy-id-stub` is
repeatable). The id and the stub are labelled as different things; the shipped
`Template-ID:` label ambiguity (`gui/wallet_policy.go:194` prints 16 bytes,
`gui/template_engrave.go:70` prints 4) is fixed in the same change.

### 7d. Seating (C4, C8, C12, C26)

Offered only when the payload holds keys or seeds, or the operator chooses to
type a seed. Slot-directed: for each emitted slot index (§5, numbering), "Slot
@2, Path 1 key 2 of 3: choose a key" over a pick list of the REMAINING sources:

- `key:` records — label: fingerprint + origin path;
- mk1 cards — same label; their stubs are IGNORED at seating (the policy does
  not exist yet); a seated card is RE-MINTED for engraving with the new policy's
  stub appended to its existing stubs (`mk.Encode`);
- seeds — BIP-39 words or ms1, from the payload or typed; a seed may fill several
  slots, each at its own hardened account by ordinal among the slots that master
  fills (`gui/multisig_build.go:594-601`), per-seed passphrase as Multisig Build
  offers; scrubbed on exit (C14).

Each key is used at most once. A mapping-review screen (slot → fingerprint +
origin) precedes consent; Back keeps assignments. The F-216 rule "the operator
is never asked to assign a card to a slot" does not transfer: that rule seats a
template that ALREADY declares its origins; a composed template has none, so
the operator's choice IS the declaration (brainstorm record section 3.5(b)).

Seating is all-or-nothing: fewer sources than slots → REFUSE at the transition,
naming both counts, offering Back-to-edit or "engrave as a keyless template".

### 7e. Consent

The existing Wallet Policy consent surface (`walletPolicyConsentLines`): the
structural summary, the id NAMED by kind, receive and change addresses 0..1 when
seated, "Keyless template - no addresses" when not (D4). Plus the stub lines of
§7c.

### 7f. Engrave (C10, C13)

Form choice: **concrete policy** (plain-text plates, QR plates, or keyed md1
strings) or **template + keys** (keyless md1 + mk1 cards). For seed-derived
slots: **Full (seed + keys)** or **Watch-only (keys)**; in Full mode the secret is
cut as words, as a SeedQR, or as ms1 strings. Plate census before cutting, as
Multisig Build does. Read-back integrity differs by form and the census says so:
md1/mk1 carry BCH; a text or QR descriptor carries only its BIP-380 checksum.

### 7g. Divergences (refusal / warning / default / documentation)

| step | what else might they do | class |
| --- | --- | --- |
| door | choose Build with no payload | DEFAULT: compose a keyless template (C26) |
| pack | two keys share a fingerprint (two accounts) | DEFAULT: labels show fingerprint AND origin |
| shape | more slots than the payload holds keys | REFUSAL at seating transition, offers keyless template |
| shape | keyless hash path in tr | REFUSAL (§4e) |
| shape | lock-only path | REFUSAL (§4e) |
| lock | date before the pack date | REFUSAL (§6b) |
| lock | relative lock beyond 388/455 days | REFUSAL naming absolute date |
| seating | wrong key for a slot | WARNING surface: mapping review; Back keeps choices |
| seating | card origin script type disagrees with wrapper | DOCUMENTATION pending recon C27-1 |
| consent | compares the shown id with a coordinator | DEFAULT: id kind named (shipped) |
| engrave | concrete descriptor longer than the plate holds | REFUSAL by census with the measured ceiling (§13) |

## 8. Copy — operator-facing strings (ASCII only; `plan-glyph-check.sh` clean)

### 8a. EXPERIMENTAL, keyless path (wsh)

```
KEY-LESS PATH (EXPERIMENTAL)
This path needs no signature. Whoever knows the
preimage of its hash can spend it. If that preimage
is ever engraved, the plate is bearer access.
```

### 8b. EXPERIMENTAL, unsorted keys

```
UNSORTED KEYS (EXPERIMENTAL)
Key order is part of this wallet. Restoring a
key-less template of it needs the key order or a
permutation search. Prefer sorted keys.
```

### 8c. Lock echoes

```
90 days = 15188 units of 512 s (90.0 days)
Block 905000
2027-03-01 00:00 UTC
  at least 181 days after this payload was packed
  on 2026-09-01
```

### 8d. Stub teaching — §7c.

### 8e. Deprecation note on Multisig Build (C7) — a COMMENT in
`gui/multisig_build.go` and a FOLLOWUPS entry only: "Deprecated 2026-09-01 in
favour of Wallet Policy > Build a new policy. No enforcement by operator ruling."

### 8f. NUMS note (C18), shown when a tr policy falls back to NUMS

```
KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and
Nunchuk import this form. Liana and BIP-388 signers
need an unspendable xpub instead (see F-449).
```

## 9. Device work items (fork)

1. `md` tree BUILDER API: construct a `descriptor` from a path list and emit
   keyless and keyed md1 through the existing serialiser (`encodePayload`,
   `encodeMD1String`); byte-identical to the Rust `compose` vectors (§10).
2. `pk_h` emitter arm in both script contexts (`md/script_emit.go`), with a
   mutation check that the hash changes the address (C17).
3. Digit-pad widget (§6b).
4. Wallet Policy door ChoiceScreen in every state (§7a); admission row (§6a).
5. Seating pick list (§7d); ms1 legs wired into the seed source picker; per-slot
   accounts via the Multisig Build machinery; `mk.Encode` re-mint with appended
   stub.
6. Stub-teaching screen (§7c); id/stub label fix.
7. Engrave form choice (§7f): keyed md1 and keyless md1 via item 1; concrete
   descriptor text and QR plates via the transaction program's plate machinery,
   after §13's ceiling measurement.
8. Three payload classes in `seal.Classify` (§6a), lockstep with the host.
9. Deprecation comment (§8e). Scrub-on-exit through `buildMultisigSeedHook`'s
   seam (C14).

## 10. Host work items (Rust first)

1. md-codec `compose` module (C15): path list → template tree; `md compose`
   subcommand beside `md compile` with the opposite contract. Vectors per
   composable shape family, with the §5b cross-check, in the corpus the Go side
   consumes; a vector where the internal key is not path 1 (C1).
2. `me sysw pack`: `key:`, `hash:`, `now:` classes (§6a); `now:` written
   automatically at pack time.
3. The five presets as Concrete policies + expected templates (C2).
4. `md-older-zero-time-units-not-refused` patch (independent; filed).

## 11. Refusals — what the operator SEES

Collected from §4e, §6b, §7d, §7g. Every refusal names what to do instead;
none prints an encoding.

## 12. Acceptance — NORMATIVE

1. **Vectors.** Every composable shape family (single/multi keys × none/lock/hash
   × wsh/tr × sorted/unsorted × keyless-wsh) has a Rust vector: path list →
   template text → md1 chunks → addresses (receive 0..1, change 0..1) → both ids;
   the §5b cross-check holds; the Go builder reproduces every template and md1
   byte for byte and every address.
2. **Journey, EXECUTED.** The C8 workflow on the emulator: `me sysw pack` with
   `key:` records (and a seed), flash, Wallet Policy → Build → shape → stub
   screen → seating from the pick list → consent → engrave choice, with the
   consent's ids and addresses compared against `md` output; the capture refuses
   to finish on a mismatch and the negative control is run. A plan may not close
   while this gate has never run.
3. **Emulator walk with NO payload** ends in a keyless-template engrave (C26).
4. **Multisig Build parity (optional, C7 comment-only):** the `sortedmulti`
   preset with seed-derived slots reproduces `gui/testdata/t6b_multisig_full.md1.txt`.
5. **Copy gates:** `scripts/plan-glyph-check.sh` and the raster floor
   (`gui/raster_test.go`) on every new screen.
6. **Cite gate:** `scripts/plan-cite-check.sh` on this spec before each R0 round;
   `CITE_FORK_ROOT` set to the working tree under review.

## 13. What is NOT verified — PENDING RECON (C27)

1. **Origin convention for seed-derived slots under `tr`.** wsh keeps BIP-48
   script_type 2' (`gui/multisig_build.go:1359`). For tr the composer must
   declare SOMETHING; BIP-48 defines no taproot script type, the reference wallet
   uses a custom purpose (`270028'`). Filled from
   `agent-reports/composer-recon-taproot-multisig-origin-convention.md`.
2. **Coordinator import of same-fingerprint two-account cosigners** (C5's normal
   shape). Filled from
   `agent-reports/composer-recon-same-fingerprint-two-accounts-import.md`; decides
   §7g's "card origin disagrees with wrapper" class and any seating warning.
3. **Plate ceilings for a concrete descriptor.** 688 chars for the two-path
   wallet (brainstorm record, C10); text plates per default font and QR symbols per plate
   are measured on the emulator, not read off a constant (`gui/transaction.go:1369`).
4. **Nunchuk UI** treatment of `or_i` vs `or_d` (library is Core's verbatim).
5. **Import tests** of composed outputs into Core, Nunchuk, Liana are import
   tests, not emit tests, and belong to the journey (item 2) or to F-449.

## 14. Out of scope, with reasons

| item | why |
| --- | --- |
| a miniscript tree editor or an on-device compiler | C1 |
| a host-generated fragment menu | C3 |
| widening md1's use-site grammar (key reuse via disjoint multipath) | F-417; C5 |
| the unhardened-child route to one card per cosigner | deferred by the operator (C27-3) |
| unspendable-xpub NUMS form | F-449, its own cycle |
| quantum framing of pk vs pkh in tr | F-448 |
| NFC seating | C8: payload first; NFC hardware not yet in hand |
| on-device preimage derivation | C25 |
| removing or redirecting Multisig Build | C7: comment only |
| on-screen QR display of a descriptor | staged plan 6b, deferred |
| Sealed-Payload memory discipline for seeds | C14 |
| `andor` emitter arm | not in the grammar |

## 15. Process

Risk-set work on three counts (normative codec behaviour, funds/keys/addresses,
spans repos). R0 to 0C/0I before code; reports persisted by the agents to
`design/agent-reports/`; persist and fold are two commits; the build gate runs on
every fold (`scripts/plan-build-gate.sh` for Rust blocks, `plan-build-gate-go.sh`
for Go, `plan-cite-check.sh`, `plan-glyph-check.sh`). Rust first: `compose` and
its vectors land in descriptor-mnemonic before the Go builder. UC is OFF for
implementation. A plan's GREEN expires: re-validate the plan immediately before
dispatching its implementer.
