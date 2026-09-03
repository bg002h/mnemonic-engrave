# SPEC — Wallet Policy COMPOSER: on-device authoring of arbitrary tr/wsh wallet policies (spend-path grammar)

**STATUS: 2026-09-01, R0 CLOSED UNDER LENS-CLOSURE (0 Critical / 0 Important open) AFTER ROUNDS 0-3.** Lenses run: correctness, adversarial (twice), journey (twice), spec-coverage, implementation-feasibility, fold-verification (three times); a LIVE operator journey walk has not been done and stays open as a lens the operator may run at any time. Round 3: fold verification of the round-2 fold (12 FIXED / 2 PARTIAL / 0 NOT FIXED, `composer-spec-R0-r3-fold-verification.md`); its two propagation gaps (the `now:` auto-append rule restated unconditionally in 6a; the door's key-count dispatch rule in 7a) and one imprecise citation folded here. Round 2: fold verification (30 FIXED / 3 PARTIAL / 2 NOT FIXED, folded at `99463ac`) and an adversarial pass on the fold-added mechanisms (1C/7I/5M/1N, `composer-spec-R0-r2-adversarial.md`, folded here). Round 0: 4 lenses,
14C/34I (`composer-spec-R0-r0-*.md`), all folded at `bc1c07c`. Round 1: fold
verification (46 FIXED / 1 PARTIAL / 0 NOT FIXED), a second journey walk
(1C/9I/6M/2N) and an implementation-feasibility lens (1C/4I/6M/2N)
(`composer-spec-R0-r1-*.md`), all C/I and the applicable M/N folded here.
Controller defaults are listed in the brainstorm record section 3.12.
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

Measurements in this document ran on the INSTALLED binaries `~/.cargo/bin/md`
0.14.0, `~/.cargo/bin/mk` 0.13.0, `~/.cargo/bin/ms` 0.16.0 and this repo's
`target/release/me` 0.7.0 unless a repo build is named (the repo's unreleased mk
carries `--keys`; the installed one does not).

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
| C6 | front door = "Build" inside Wallet Policy; the door becomes a choice in every state | §6a, §7a |
| C7 | Multisig Build deprecated by COMMENT only, no enforcement | §8e, §9 |
| C8 | seating is payload-first (NFC later): slot-directed pick list of the payload's keys | §7d |
| C9 | teach the mk1 stub after authoring, unconditionally; both stubs after seating | §7c |
| C10 | engraved FORM is the operator's choice: concrete policy (text / QR / keyed md1) or template + key cards | §7f |
| C11 | timelock UI respects the very different ranges of the lock kinds | §6b |
| C12 | seeds (BIP-39 words or ms1) are a key source, via the unsealed payload or the keyboard | §6a, §7d |
| C13 | seed-derived slots get Full / Watch-only modes; the secret's plate form is the operator's choice | §7f |
| C14 | no Sealed-Payload-grade memory treatment; scrub on exit as Multisig Build does | §9 |
| C15 | the lowering lives in md-codec as `compose`, surfaced as `md compose` | §10 |
| C16 | grammar bounds approved; keyless paths and unsorted `multi` behind EXPERIMENTAL | §4 |
| C17 | one key: `pkh` in wsh, `pk` in tr (F-448 filed) | §5 |
| C18 | NUMS spelled raw `H` this cycle; BIP-388 gap documented; F-449 filed | §5c, §8f |
| C19-C23 | review findings adopted (C22 withdrawn by C23): first-appearance numbering, `or_i(pkh)` head, `or_d` only for a bare multi head, keyless wsh-only, lock-only refused, ≥1 keyed path, first-listed single key as internal key, lock ranges sourced | §5 |
| C24 | pack time reaches the device as a payload item, a LOWER bound on now | §6a, §6b |
| C25 | entry UX adopted: kind → unit → digits → echo; digit pad; `hash:` record or typed hex | §6b, §6c |
| C26 | Build with no payload is allowed: the result is a keyless TEMPLATE | §7b, §7e |
| C27 | recon fan-out: origin convention (1) and same-fingerprint import (2) done and verified; unhardened child (3) deferred | §4f, §7d, §13 |
| C28 | seed-derived TAPROOT slots derive at `m/48'/coin'/account'/3'`; wsh at `.../2'` | §4f |
| C29 | one seed at two slots INSIDE one path: warning; across paths: informational | §7d, §7g, §8g |

## 3. Measured inventory (what exists, so no section re-derives it)

| capability | state | evidence |
| --- | --- | --- |
| Wallet Policy program: payload cards → gather → consent (summary, named id, receive+change) → review → engrave | shipped | `gui/wallet_policy.go:35-300` |
| Seat mk1 cards into a keyless template by declaration match | shipped | `gui/key_card_seating.go:53` |
| Serialise any tree to md1 (keyed or keyless) | exists, not exposed as a builder | `md/encode.go:159`, `:374`, `:461` |
| Emit Script for fragments | all but `andor`, `pk_h` | `md/script_emit.go` cases; `grep -c tagAndOr` = 0, `grep -c tagPkH` = 0 |
| Tap leaves via the same emitter | shipped (F-214) | `md/tapleaves.go:188` |
| Device derives use-sites | `<a;a+1>/*` and bare `*` only | `gui/md1_expand.go:149` |
| Form-aware wallet id and stub | shipped | `md/template_id.go:122,163` |
| Re-mint an mk1 card | exists | `mk/encode.go:39` `Encode` (deterministic) |
| Seed entry: payload words / typed words | shipped; ms1 legs exist elsewhere | `gui/derive_xpub.go:104`, `gui/gui.go:1262`, `gui/gui.go:2856` |
| Multisig Build: per-slot accounts by ordinal, passphrase per seed, Full/Watch-only mode labels | shipped | `gui/multisig_build.go:594-601`, `:738`, `gui/multisig_build_census.go:475` (the Full label) |
| Text plates (greedy packing) and 16-symbol structured-append QR plates | shipped in Engrave Transaction | `gui/transaction.go:1145`, `:1369`; `txqr.MaxSymbols = 16` |
| Payload: prefixed records `text:`/`pass:` with hex bodies, matched before sniffers, reserved | shipped | `SPEC_systemwide_payloads.md` section 5.3 |
| Payload region | 64 KiB | `sysw/wire.go:28` |
| Wallet Policy admission row | Descriptor + MDMK only | `gui/sysw_admit.go` |
| Device clock | none; `time.Now` drives timeouts only | `gui/gui.go:128,139,442` |
| Numeric entry widget | none; passphrase keyboard has a digits page mixed with punctuation | `gui/passphrase_keyboard.go:21` |
| md admission: nested `sortedmulti`/`sortedmulti_a` | refused; whole script / whole leaf only | brainstorm record section 3.8 |
| md admission: keyless tap leaf | refused ("All spend paths must require a signature") | first review, I2 |
| md1 TOTAL slot count | 1..=32 per descriptor (5-bit `path_decl.n`) | `md/md.go:215-221`; `crates/md-codec/src/error.rs:57-59`; measured: 36 slots refused, 32 encode |
| Firmware headroom | 1,503,652 B flash / 62,592 B RAM (r1, TinyGo 0.41.1, pico2) | feasibility report point 6 |
| md admission: `older(0x400000)` | ACCEPTED (defect, filed `md-older-zero-time-units-not-refused`) | brainstorm record, C20 |

## 4. The grammar — NORMATIVE

A **policy** is an ordered list of 1 to 8 **spend paths** under one **wrapper**.
"Path" below always means a spend path; a derivation path is always called an
origin. One lock per spend path (§4b) is what discharges C11's mixing rule: a
height lock and a time lock can never meet inside one `and_v` chain.

### 4a. Wrapper

| wrapper | admitted when |
| --- | --- |
| `tr` | any path list satisfying §4e |
| `wsh` | any path list satisfying §4e |
| `sh(wsh)`, `sh` | ONLY a single path that is an unlocked, unhashed key set with n ≥ 2 (a `sortedmulti`; the Multisig migration, C7). n = 1 is refused at the picker |

### 4b. Path

A path is `KEYS` ∧ optional `HASH` ∧ optional `LOCK`, where:

| element | bound | source |
| --- | --- | --- |
| `KEYS` | k-of-n over FRESH slots, n in 1..=9, 1 ≤ k ≤ n; every slot appears in exactly one path (C5) | `multi` ≤ 20 and `multi_a` ≤ 999 (rust-miniscript-fork `src/miniscript/limits.rs` lines 35 and 38), md1 32 per fragment (`crates/md-codec/src/tree.rs:92-120`) |
| `HASH` | at most one `sha256(H)`, H = 32 bytes, the SHA-256 of a 32-byte preimage (§6c) | both reference wallets use sha256 only; `hash256`/`ripemd160`/`hash160` stay decodable, not composable |
| `LOCK` | at most one of `older` or `after`, values per §4c | C11, C20 |
| keyless path | `HASH` with or without `LOCK`, no `KEYS`: **wsh only, EXPERIMENTAL, confirm-to-proceed** (C16) | a lock-only path (no keys, no hash) is REFUSED: anyone can spend after N |
| policy | at least one path has `KEYS` (BIP-388 l.191); if EVERY path carries `HASH`, the §8h warning fires before consent; the policy's TOTAL slot count is 1..=32 (the wire's 5-bit `path_decl.n`, `md/md.go:215-221`, `crates/md-codec/src/error.rs:57-59`; measured r1) | reviews r0, r1 |

### 4c. Lock values — SOURCED (C20)

| lock | admitted operand | meaning | sources |
| --- | --- | --- | --- |
| `older(n)`, blocks | n in 1..=65535 | n blocks, ≤ 455.1 days | BIP-68 l.30-40, 74-83 (bit 31 disable, bit 22 type, mask `0x0000ffff`); BIP-112 l.28-33; BIP-379 l.135 (`1 <= n < 2^31`) |
| `older(n)`, time | n = 0x400000 + u, u in 1..=65535 | u × 512 s, ≤ 388.4 days | same; BIP-68 l.46 (zero units = no lock) |
| `after(n)`, height | n in 1..=499,999,999 | block height | BIP-65 l.27, 243-250; Core `script.h:48` `LOCKTIME_THRESHOLD` |
| `after(n)`, time | n in 500,000,000..=2,147,483,647 | Unix time; the OPERAND floor is 1985-11-05 00:53:20 UTC, the DATE-ENTRY floor is 2009-01-03 (§6b) | BIP-379 l.135; rust-miniscript-fork `src/primitives/absolute_locktime.rs` line 10 |

Every other operand miniscript would accept is either masked by consensus to a
different lock or to no lock (`older(0x400000)`), and the composer never emits
one. **The DEVICE enforces these tables itself** (§6b, §9 item 3) and does not
rely on md's downstream guard, which today misses the zero-units case; §12 item 7
is the acceptance that fails if it does not.

### 4d. Presets (C2)

The five toolkit archetypes — simple-timelocked-inheritance, kofn-recovery,
tiered-recovery, hashlock-gated, decaying-multisig — are offered as one-tap
presets that POPULATE a path list the operator then edits (§9 item 5), plus a sixth, **plain k-of-n multisig** (one unlocked, unhashed `sortedmulti` path), which is the Multisig Build shape C7 migrates; presets are offered under `wsh` and `tr`; under `sh`/`sh(wsh)` only the plain k-of-n preset is offered. They are
the same spend conditions as `mnemonic build-descriptor`'s goldens but NOT
byte-identical to them (brainstorm record section 3.7). **This generalises:** the
lowering is ONE fixed spelling, so any policy the operator also holds elsewhere in
another spelling (the reference wallet's `tr.policy` spells tier 1 hash-first) is a
DIFFERENT wallet with a different id and different addresses. The stub screen says
so (§8d).

### 4e. Structural refusals (before lowering)

| condition | outcome |
| --- | --- |
| no path with keys | REFUSE, §8m line 1 |
| a path with neither keys nor hash | REFUSE, §8m line 2 |
| keyless path under `tr` | REFUSE, §8m line 3 (a policy choice of this build, not a taproot limit) |
| more than 8 paths, or n > 9 in a path, or n = 1 under `sh`/`sh(wsh)`, or a 33rd slot | REFUSE at the picker (the picker does not offer the value); the slot cap says §8m line 5 |
| `sh`/`sh(wsh)` with anything other than ONE unlocked, unhashed path whose key set has n ≥ 2 | REFUSE, §8m line 4; the legacy wrappers are sorted-only, so the §8b confirm is never offered under them (feasibility M-5) |
| a taproot key-path slot that is also in a leaf | cannot occur (C5) |

### 4f. Key origins of slots (C28; corrected r0)

| wrapper | origin of a slot derived from a seed on the device | account |
| --- | --- | --- |
| `wsh` | `m/48'/0'/account'/2'` | by ordinal among the slots that master fills, in ascending emitted slot index (`gui/multisig_build.go:594-601`, C5/C12) |
| `sh(wsh)` | `m/48'/0'/account'/1'` (the shipped device's own S5 fix, `gui/multisig_build_slots.go:111-130`) | same |
| `sh` | `m/48'/0'/account'/2'` (BIP-48 defines no legacy type; this device's convention, `multisigScriptTypeComponent`) | same |
| `tr` | `m/48'/0'/account'/3'` | same |

`coin'` is `0'`: complex-policy derivation is mainnet-only by construction
(`gui/policy_address.go:61`), §14. A slot seated from a `key:` record or an mk1
card carries the origin the record or card DECLARES, verbatim; nothing measured
refuses or warns on any origin (BIP-388, Ledger, Nunchuk, Liana, md; brainstorm
record section 3.11), so an origin/wrapper mismatch is documentation only.

**Unseated slots (C26, or the unfilled slots of a partially seated template)
declare the §4f origin for the wrapper with `account'` = the LOWEST account not
already declared by any slot of this template, assigned in ascending emitted
slot index, and no fingerprint.** **Invariant, enforced before any template is
emitted and re-checked by §7e's self-check on the decoded md1: no two slots of a
produced template declare the same origin unless BOTH declare a fingerprint and
those fingerprints differ.** Why: a pathless slot is refused by the fork's decoder
(F-166, open); two slots at one origin with no fingerprints are unseatable
(`errSeatSlotContested`); and two slots at one origin where only ONE declares a
fingerprint let a single card fill both silently, because `slotMatchesCard` skips
the fingerprint test when the template declares none
(`gui/key_card_seating.go:119-151`): a mis-seated key presented as reviewed. A
seated pair that violates the invariant (two privacy-preserving cards at one
origin) is REFUSED at the mapping review (§8v). The template screen (§7c) states
the expected origin per unseated slot. No standard exists for taproot multisig origins (BIP-48 registers `1'`/`2'`
only; bips PR #1473 proposing `3'` closed unmerged 2024-05-14); `3'` is what
Coldcard Edge exports (`shared/export.py:414`) and what `mnemonic-toolkit` sweeps
as `bip48-tr-multi-a`. `ms derive` must gain a `bip48-p2tr` template (§10 item 5).

## 5. The lowering — NORMATIVE (brainstorm section 3.10; C19-C23; r0 folds)

"Lowering" is the FIXED, search-free translation from the path list to a BIP-388
descriptor template. It is defined in Rust first (§10) and ported to Go; the two
must produce byte-identical templates for every composable list.

| rule | wsh (and `sh`/`sh(wsh)` for their single path, wrapped) | tr |
| --- | --- | --- |
| paths combine | listed order, recursive, last path stands alone: `or_d(P, R)` iff `P` is a bare unlocked, unhashed `multi(k,…)` with n ≥ 2; otherwise `or_i(P, R)`. A bare single key is `or_i(pkh(K), R)`. Never `andor`, never `thresh` over paths | let `L` be the path list with the extracted internal-key path removed and `m` the number of paths in `L`. `m = 0`: `tr(@0/<0;1>/*)`, no tree. `m = 1`: `tr(IK, P)`, the leaf written bare (braces spell a branch only; `{P}` is refused by md and BIP-386). `m ≥ 2`: one leaf per path on a right spine in listed order, leaf j (1-based) at depth min(j, m−1) |
| inside a path | `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`, dropping absent parts | same |
| key set | SOLE path, unlocked, unhashed, n ≥ 2: `sortedmulti` (BIP-383/388 sole-child rule); ANY other multi-key path: `multi`; one key: `pkh` | SOLE leaf, unlocked, unhashed, n ≥ 2: `sortedmulti_a` (whole leaf); any other multi-key path: `multi_a`; one key: `pk` |
| unsorted where sorted was legal | `multi` instead of `sortedmulti`, EXPERIMENTAL confirm (§8b) | `multi_a` instead of `sortedmulti_a`, EXPERIMENTAL confirm (§8b) |
| internal key | n/a | the FIRST-LISTED unlocked, unhashed one-key path (then not a leaf); otherwise NUMS |
| NUMS spelling | n/a | raw `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (C18); see §5c |
| placeholder numbering | `@i` by FIRST APPEARANCE in the emitted text; slot labels shown to the operator are these indices, computed after lowering and RECOMPUTED after any shape edit (§7d) | same; an extracted internal key is `@0` |
| declarations | EVERY slot declares an origin (§4f) and, when seated, the master fingerprint of the seated key; a keyless template is engraved WITH fingerprints for seated slots and with distinct-account origins for unseated ones | same |
| keyless path | `and_v(v:sha256(H), LOCK)` or `sha256(H)` | refused (§4e) |
| use-site | `/<0;1>/*` on every slot | same |

### 5a. Why these and not others (measured, both reviews)

- `or_d(pkh(P1), R)` is dominated: +34 WU on every non-P1 spend AND it publishes
  P1's key. `or_i(pkh)` keeps the key hidden until P1 spends. The BIP-388/compiler
  form `or_d(pk(P1), R)` is 26 WU cheaper per P1 spend, key in cleartext; named
  as the drop-in if C17 is ever revisited (C21).
- `or_d` over a bare `multi(k)` head: −2 WU per head spend, +k WU per other
  spend, no key exposure (M3); kept (C23).
- Conjunct order has identical witness-byte cost while LOCK is last; LOCK anywhere
  else costs +1 script byte (N1). "Byte-identical" elsewhere in this spec means the
  ENCODED artifact, chunk by chunk. Leaf text order at a node does not change the address (N2).
- Nested `sortedmulti`/`sortedmulti_a` are refused by md and by BIP-383/388, so a
  locked or non-sole multi-key path is necessarily unsorted; the §8b confirm
  fires ONLY where sorted was legal and declined, never on a lowering-forced
  `multi`.

### 5b. Cross-check contract (for the Rust vectors, §10)

For every composable list the emitted template: parses in its context; passes
`sanity_check` (keyless wsh paths via `ExtParams::top_unsafe()`, review point 8);
survives `md encode` → `md decode` byte-identically (C1); and, for every family
WITH a key in every path, `lift()`s to the same semantic policy as `md compile` of
the equivalent Concrete policy. **The compile leg is carved out for keyless
families**: the compiler refuses any sigless spend path in both contexts
(measured: "Top Level script is not safe on some spendpath"); those vectors keep
the other legs and compare `lift()` against a hand-written `Semantic` policy.

### 5c. NUMS and BIP-388 (C18)

The raw `H` spelling is valid in Bitcoin Core and imported by Nunchuk; it is NOT a
BIP-388 key placeholder, so BIP-388-strict registration and Liana refuse it. This
is a property of md's existing wire (`is_nums` flag, `crates/md-codec/src/tree.rs:51`)
that the composer inherits; the xpub form is F-449, its own constellation cycle.
The device says so in copy (§8f).

## 6. Inputs — NORMATIVE

### 6a. Three new payload record classes (host `me sysw pack` + device `sysw.Classify`, lockstep)

All three follow `SPEC_systemwide_payloads.md` section 5.3: a reserved prefix, a
lowercase-hex body, matched BEFORE the sniffers, and a prefixed record whose body
is not valid hex is `ClassUnknown` and refused. The classifier is
`sysw.Classify` (`sysw/record.go:100`); `seal.Classify` belongs to the frozen
Sealed Payload and is not touched. The reserved prefixes today are `text:`,
`pass:`, `tx:` (`sysw/record.go:15-21`); these three join that list. None is
secret.

| record | body | decoded meaning | class |
| --- | --- | --- | --- |
| `key:<hex>` | hex of the UTF-8 text `[fingerprint/path]xpub` (BIP-380 key-origin notation, the key form `md decompose` prints) | one candidate key for seating | `ClassKey` NEW |
| `hash:<hex>` | the 32-byte digest itself, 64 lowercase hex | one candidate `sha256` hashlock | `ClassHash` NEW |
| `now:<hex>` | hex of the UTF-8 text `<unix-seconds>[,<block-height>]` | the PACK time and optional height: a LOWER BOUND on the present (C24) | `ClassNow` NEW |

**Body validation after hex-decoding, each failure `ClassUnknown` and refused with
its own line (§11):** `key:` MUST parse as BIP-380 key-origin notation with a
NON-EMPTY origin (an md1 slot carries a path; F-166 pathless is open), an xpub at
depth 3 or 4 (md's own `--key` rule), and an origin whose component count equals
the xpub's depth; a bare xpub is refused naming the fix. `hash:` MUST decode to
exactly 32 bytes. `now:` MUST match `^[0-9]{1,10}(,[0-9]{1,9})?$` with seconds in
`1..=2147483647` and the height, when present, in `1..=499,999,999` (§4c's height
band). Uppercase hex anywhere is not valid hex (section 5.3). **Where the refusals
live:** a malformed record is refused on the HOST by `me sysw pack`, which already
walks the whole record vector and refuses an unclassifiable record by index
(`crates/me-cli/src/sysw/mod.rs:288` `pack_with`, whose per-index refusal is `admit_check` at `:416`); the per-failure lines of §8n
are its lines. On the DEVICE a record that fails classification goes INERT under
the shipped contract (`sysw/descriptor.go:46-48`: "stays in the session, is offered
to nobody, and reaches no screen"). The device-visible signal is per class: a
malformed `key:` reduces the door's "Keys loaded: N"; a malformed `hash:` or
`now:` changes no count, so the door also carries "N payload records were not
understood" from the session's inert count (§8r), the one line that covers all
three. The payload-wide rule "at most ONE `now:` record" is enforced at the two
sites that see the whole payload: host `pack_with` and device `syswSession.load`
(`gui/sysw_session.go:80`). `me sysw pack` auto-appends `now:` ONLY when the
operator's records contain none AND at least one of them is a `key:` or `hash:`
record — the two classes that, like `now:`, are admitted at Wallet Policy alone —
or when `--now` is passed (stand-in ruling 2026-09-02,
`design/agent-reports/composer-S1-decision-now-default.md`, revocable by the
operator); so an operator-supplied `now:` wins silently and pins a deliberate
bound, and a payload holding no composer-only record (a seed for Backup Wallet, a
`tx:`, a descriptor or a card) packs byte-identically to today and carries no
bound unless the operator asks; two OPERATOR-supplied `now:` records are a host
refusal with a remedy (§8n: "Remove one."), and on the device both go inert with
the door showing no bound. **What a `key:` record's origin proves:** the xpub's depth and
its last component are checked against the declared path; the account and every
interior component are declarations this device cannot verify (F-217; `mk`
cannot either), so the mapping review prints each slot's origin verbatim beside
its fingerprint with the note that the device cannot confirm the key was derived
there.

Why `key:` exists: today a bare xpub line packs as a `pkh(xpub)` single-sig
WALLET and a `[fp/path]xpub` line is refused (brainstorm record section 3.6).
Why `now:` is a lower bound: the device has no clock; the record affects ONLY
echoes and refusals (§6b), never an encoded operand. `me sysw pack` appends `now:`
as the LAST record ONLY when the operator's records hold none and include a `key:`
or `hash:` record, or when `--now` is passed (the rule above, one statement of
it); `--no-now` suppresses that auto-append so a fixture's pack output stays a
pure function of its inputs (§10 item 2). A seed-only or card-only payload
therefore carries no bound unless the operator asks; at lock entry the §6b
"cannot tell the time" line then stands alone.

**What the payload spec must receive (§10 item 6; its own R0-gated fold):**
section 3.3.1 gains three class rows with `secret? = no`; section 3.3.2 has NO
Wallet Policy row today (F-415, the fork is ahead of the document:
`gui/sysw_admit.go:26` carries `progWalletPolicy`), so this cycle CREATES it with
all ten cells: Mnem •, Cdx32 •, Passph •, FreeText blank, Descr •, MDMK •, Addr
blank, Key •, Hash •, Now •; section 5.3's reserved-prefix list gains the three
prefixes. Because Mnemonic, Cdx32 and Passphrase become admitted at Wallet Policy
for the first time, section 3.3.3's flag screens (F1 unencrypted-in-flash with its
erase offer; F2 weak seal) are the screens that cover them. MEASURED at fork
`169073c` and unchanged at `321acb56`: they fire at payload LOAD, not in a seed
step, and not per program. `syswLoadWarnings` (`gui/sysw_load.go:259`) is called
from exactly one place, `syswLoadFlow` (`gui/sysw_load.go:210`), whose only call
sites are `gui/gui.go:2074` (boot) and `gui/sysw_unload.go:36,75` (reload); it
walks `s.records` and consults NO admission table, so a mnemonic in a payload
already raises F1 whatever this row admits. Admitting the seed classes here
therefore adds no flag-screen wiring, and none was added -- a test asserts that
`syswLoadWarnings` gained no per-program caller. §7g's classification is
unchanged and still DEFAULT: the operator meets the flags at load, which is
before the composer consumes a seed. Two comments are rewritten with
the change: `gui/gui.go:191-203` (the program "came from OUTSIDE this device";
"would drag a seed requirement or a plate census into a flow that needs neither";
"not a rename of Multisig") and `gui/sysw_admit.go:47-51` ("NO seed class ... least
privilege"), which C12 deliberately reverses. Staged-plan D5 stands as history:
Wallet Policy remains its own program; Build is a door inside it, and the
`sh`/`sh(wsh)` admission is the migration C7 names.

### 6b. Lock entry (C11, C24, C25)

Kind → unit → digits → echo, on a NEW digit-pad widget: digits, backspace, done,
and for dates a fixed `YYYYMMDD` field of eight digits echoed as `YYYY-MM-DD`
(the pad types no separators). The operator never types a raw operand. Impossible
dates (2027-02-31) are refused at entry.

| kind | unit | entry | encoding | echo |
| --- | --- | --- | --- | --- |
| relative | blocks | 1..65535 | `older(n)` | "N blocks (about D days)" |
| relative | days | 1..388 | `older(0x400000 + ceil(days*86400/512))` | "D days = U units of 512 s (D' days)" |
| absolute | height | 1..499,999,999 | `after(h)` | "block H" + the §6b bound line |
| absolute | date | `YYYYMMDD`, 2009-01-03 ..= 2038-01-19 | `after(unix at 00:00:00 UTC)` | "DATE 00:00 UTC" + the §6b bound line |

**Date floor, independent of `now:`:** any date whose 00:00:00 UTC value is below
500,000,000 encodes as a block HEIGHT, not a time (1985-11-05 00:00 UTC is
499,996,800); the entry refuses every date before 2009-01-03 with §8t. The
date-entry band is therefore strictly inside §4c's time row.

**The bound line.** When `now:` is present its seconds field bounds dates and its
height field bounds heights; a field that is absent bounds nothing. A date or
height BELOW its bound → REFUSE with §8o (dates: "Choose a later date."; heights:
"Choose a later height."). Above it → the echo ADDS the pack date and never
withdraws the disclaimer: "This device cannot tell the time. The payload says it
was packed on <pack date>, which may be long ago. Nothing here has checked that
this is in the future." (§8c); heights read "the packed height was H". When the
relevant field is ABSENT the echo carries the bare disclaimer. The copy never
says "now"; a stale `now:` can only weaken the below-bound refusal, never invent
one, which is why the refusal stands and no reassurance is given.

Refusals: blocks > 65535 or days > 388 → §8u.

### 6c. Hashlock entry (C25; r0 C-4)

Primary: pick from the payload's `hash:` records, each row `hash <i>  <first 8>..<last 8>` in the host's pack order (28 characters, inside the 436 px label budget; a 64-hex row would be cut, not wrapped). Fallback: type 64 hex on the keyboard, accepted only when exactly 64 valid hex characters are present. At entry and at consent the device states the 32-byte rule:
`sha256(H)` compiles to `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL`, so
the preimage MUST be exactly 32 bytes; a digest of a passphrase directly can never
be spent (§8i; the reference wallet's own README records months of exactly that).
The composer never derives, stores or engraves a preimage this cycle (§14). When
every path of the policy carries a hash, the §8h warning fires before consent.

## 7. The flow — the C8 workflow, walked

### 7a. The door (C6; r0 I-3)

Wallet Policy opens on a ChoiceScreen in EVERY state (today the no-payload path
drops straight into the NFC gather, `gui/wallet_policy.go:97`). Choices name the
route they take (F-437, resolved): "Scan cards", "From payload" (only when the
loaded payload holds a Descriptor or md1/mk1 record), "Build a new policy". The
door states the key state WITH Build, in the screen's Lead rather than beneath
the row: `ChoiceScreen` draws its Lead through `widget.Labelw`, which WRAPS
(`gui/gui.go:1969`), and its choice rows through `widget.Label`, which does not
-- so a line as long as "A payload is in flash but not loaded. Load it from the
carousel first." would be cut off the panel if it were a row. The state:
"Keys loaded: N" when the payload holds N keys and no seed; "Keys loaded: N, plus
M seed." when it holds N keys and M seeds (the noun pluralises with M, as the
not-understood line's does); "A seed is loaded. It can fill any number of slots."
when it holds seeds and no key (a seed fills any number of slots, so no count is
printed for it); "No keys loaded. This builds a key-less template." when it holds
neither or none is loaded; and, when a payload
is present in flash but was skipped at boot, "A payload is in flash but not
loaded. Load it from the carousel first." (F-152's future default would make that
a button).

### 7b. Shape

Wrapper → preset or blank → paths. The preset screen's FIRST row is the blank route, "Build my own paths" (S4 walk W-1, 2026-09-02: when Back alone was the blank route, the operator saw six presets and no way forward); Back on that screen returns to the wrapper choice. Per path: keys (n, k), lock (§6b), hash
(§6c). A path list screen shows each path as one line ("Path 2: 2-of-3 + 90
days") and, whenever a payload is loaded, a live line "slots: N / keys available:
M". Back preserves everything ("going back should lose nothing"). The
EXPERIMENTAL confirms fire here: §8a once per keyless path when it is added, §8b
once per key set where sorted was legal and the operator chose unsorted; both are
confirm-to-proceed, neither is dismissible (C16).

### 7c. Teach the stub (C9) — shown UNCONDITIONALLY after the shape is complete, and RE-SHOWN after any shape edit

> Template-ID:  <32 hex>
> mk1 stub (template):  <8 hex>
>    mk encode --xpub <xpub> --origin-fingerprint <fp>
>      --origin-path <path> --policy-id-stub <8 hex>
> Slot @0 expects a key at m/48'/0'/0'/3'  (unseated slots only)

The per-slot "expects a key at" line is shown ONLY for slots that will stay
unseated (§4f's unseated rule); a slot seated from a record, card or seed
carries that source's declared origin instead, and the line for it reads "Slot
@i: <fingerprint> <origin>" once seated. The screen is a PAGED widget: the fixed
header (id, stub, command, §8d) on the first frame, then slot lines at a stated
per-frame budget with a pager, because the body grows one line per slot and the
grammar admits 32 (§9 item 6; the per-frame capacity is a plan-time render
measurement, §13). The template id is key-independent and origin-invariant but
NOT shape-invariant: the wrapper, the path list, every lock operand and every hash
digest all enter it (measured: changing an operand or a digest moves the id), so
the screen re-appears after EVERY edit made on the shape screen and says, per
§8s, that the id changed and that cards already minted with the old stub will
not seat into this template (`gui/key_card_seating.go:66-73`). After
seating (§7d) the keyed policy's id and stub are added and the screen recommends
stamping BOTH stubs on each key card (`--policy-id-stub` is repeatable). Labels,
literally: `Template-ID:` and `Policy-ID:` for the 32-hex ids; `mk1 stub
(template):` and `mk1 stub (policy):` for the 8-hex stubs; the shipped
template-engrave screen's 4-byte `Template-ID:` (`gui/template_engrave.go:70`) is
relabelled `mk1 stub (template):` in the same change (§9 item 6). The screen also
carries §8d's line: a wallet composed here is its own wallet.

### 7d. Seating (C4, C8, C12, C26; r0 folds)

Offered only when the payload holds keys or seeds, or the operator chooses to
type a seed. **The composer does NOT call `seatKeyCards`** — that function seats a
template that already declares its origins, by declaration match, for cards that
already carry the template's stub; a composed template has no declarations and no
card carries its stub yet. The composer's seating is slot-directed: for each
emitted slot index (§5 numbering), "Slot @2, Path 1 key 2 of 3: choose a key" over
a pick list of the REMAINING sources:

- `key:` records — label: fingerprint + origin path;
- mk1 cards — same label; their stubs are IGNORED here (the policy does not exist
  yet); every seated card is later cut as a RE-MINTED mk1 carrying BOTH the
  composed template's stub and the composed policy's stub APPENDED to its existing
  stubs (`mk.Encode`), so one card seats into either engraved form and stays
  indexed to the wallets it already belonged to;
- seeds — BIP-39 words or ms1, from the payload or typed; a seed may fill several
  slots, each at its own hardened account by ordinal among the slots that master
  fills, in ascending emitted slot index (§4f), per-seed passphrase as Multisig
  Build offers; scrubbed on exit (C14).

"Used at most once" governs `key:` records and mk1 cards (C8 "remaining"); a
SEED is a source of as many slots as the operator assigns to it (C12, §4f), so the
shortfall test below counts ASSIGNABLE slots, not sources, and "keys available"
(§7a, §7b) counts records plus cards plus, for each seed, "seed: any slots". The
consume path's "one card may fill several slots" rule
(`gui/key_card_seating.go:28-30`) governs restore, not composition, and the two
coexist. The prompt for an extracted internal-key slot reads "Slot @0, key path
(spends alone): choose a key". **Two slots resolving to
the same xpub → REFUSE at the mapping review**, naming both slots (BIP-388 l.193,
pairwise distinct; md refuses it only at encode). **Any change that moves slot NUMBERING (the
wrapper, the path count, or a path's key count) after at least one slot has been
assigned discards ALL assignments**; the operator is told so before the edit is
accepted (§8j), because §5 renumbers slots by first appearance in text that is a
function of the wrapper as well as the path list (tr extracts an internal key as
`@0`, wsh does not), and a carried assignment would seat keys silently into the
wrong slots. A lock or hash edit moves no slot, keeps assignments, and re-shows
the stub screen (§7c). With no slot yet assigned there is nothing to discard and
§8j does not fire. "Path N" in every seating and mapping prompt is the OPERATOR's
listed path index, never an emitted leaf index.

A mapping-review screen (slot → fingerprint + origin, the origin printed verbatim,
with the note that the device cannot confirm the key was derived there) precedes
consent; Back keeps assignments. Two slots that would declare the same origin with
fewer than two distinct fingerprints violate §4f's invariant and are REFUSED here
(§8v). When one seed or fingerprint fills two slots INSIDE ONE path the
review shows the C29 WARNING (§8g). The same fingerprint in two DIFFERENT paths is
C5's normal case and gets one informational line, plus the §8k line that a person
in two paths needs two keys from two accounts.

Seating is all-or-nothing: fewer assignable slots than slots → REFUSE at the
transition with §8p: the count line ("N slots, M keys available") and the unfilled
slots named; no cause is guessed (the C5 lesson is taught at the shape step by
§8k); then Back-to-edit or "engrave as a keyless template" (§7f, the partially
seated form).

### 7e. Consent — a NEW surface for composed policies (r0 C-1/C-2)

The shipped Wallet Policy consent (`walletPolicyConsentLines`) summarises from
`md1Summary`, which prints "Complex policy - cannot display safely." for every
shape the codec marks non-renderable — measured for every multi-path or taproot
shape this composer exists to author (`md/md_test.go:337,416`); and the one structural summary that
exists (`policySummaryLines`, one call site in `gui/template_engrave.go:86`)
counts a multi-path wsh script as ONE branch (`md/policy_shape.go:43`). Neither
may be the composer's consent.

The composer's consent is derived from the DECODED md1 the device is about to
engrave (never from UI state) through an extended `md.PolicyShape` (§9 item 1)
and MUST name, per path in listed order: its k-of-n or single key, its lock kind
and value in operator units (§6b echo form), its digest (first 8 and last 8 hex),
and the EXPERIMENTAL marks; then the key-path line ("Key-path: A KEY CAN SPEND
ALONE" for an extracted internal key, §8f's NUMS note otherwise); then the id
NAMED by kind with both stubs (§7c); then receive and change addresses 0..1 when
seated, or "Keyless template - no addresses" (D4). Before the screen is shown the
device asserts that the decoded shape, the slot assignment, every slot's origin
and fingerprint (against the mapping review), the fixed use-site, and §4f's
pairwise-distinguishability invariant all hold on the DECODED md1, and REFUSES to
continue on mismatch with §8q ("... Go back and check the path list, or start
again."), so a builder defect in the shape, the seating, the origins, the
fingerprints or the use-site cannot reach steel as a reviewed wallet (what stays
outside the check: the key bytes themselves, which the addresses cover); this
refusal is provoked by fault injection, not by an input (§12 item 4). The surface is `confirmReviewScreen`'s PAGED form
(`gui/multisig_build.go:1908-1931`), which draws its pager only when a second page
exists; eight paths plus four addresses do not fit one frame. Then the
"nothing outside this device has checked this policy" warning (§8l), Multisig
Build's, unskippable.

### 7f. Engrave (C10, C13; r0 I-1)

Form choice: **A, concrete policy** (plain-text plates, QR plates, or keyed md1
strings) or **B, template + keys** (keyless md1 WITH fingerprints + one mk1 card
per seated slot). Every seated slot yields a card in form B regardless of source:
a `key:` record is MINTED as an mk1 (fingerprint + origin + xpub + both stubs), a
payload mk1 is RE-MINTED with both stubs appended, a seed-derived slot is minted
likewise. A keyless composition (no seated slots) has no form A and no cards: the
choice collapses to "template only" and says so. A PARTIALLY seated composition
(the §8p fallback) offers no form A either; its form B is the keyless template,
whose unseated slots take §4f's lowest-free accounts, plus one card per SEATED
slot carrying the TEMPLATE stub only, and the screen says the policy id does not
exist until every slot is seated. For seed-derived slots: **Full
(seed + keys)** or **Watch-only (keys)**; in Full mode the secret is cut in one
of the TWO plate forms this device has. MEASURED: `engraveSeed`
(`gui/gui.go:839`) bakes the BIP-39 words AND a SeedQR onto ONE `backup.Seed`
plate, and `backup.SeedString` (`backup/backup.go:26`) is the string-only form
`engraveCodex32` (`gui/codex32_polish.go:218`) cuts for ms1. There is no
words-only and no QR-only plate for a mnemonic, so "words" and "a SeedQR" are
not separate choices -- they are one plate. Splitting them is a new plate layout
with its own sizing and its own goldens, and is filed as **F-455**. A seed that
filled several slots is cut ONCE.
Plate census before cutting, as Multisig Build does, and it counts CARD chunks
too: appending stubs can push a card into a third chunk (`mk/encode.go:26-29`);
the census WOULD refuse a
concrete descriptor longer than the plate holds, naming the measured ceiling
(§13 item 1) -- but S3 ships form A as the keyed md1 ONLY, so there is no
concrete-descriptor plate to refuse and the refusal is NOT IMPLEMENTED. It
belongs to **F-457** with the renderer it depends on: `md` deliberately emits
no descriptor text, so the plate needs a Rust-first renderer before the refusal
guarding it can exist. The ceiling itself IS measured (596 characters, §13
item 1) and `composerDescriptorCeilingChars` computes it by search, so the
number is ready when the plate is. Recovery-time error detection differs by form and the census says so: md1/mk1
carry BCH; a text or QR descriptor carries only its BIP-380 checksum. **This
re-opens staged-plan 6d**, deferred 2026-08-20 for "unmeasured sizing plus an
irreversible medium while content rules were still moving": §13 item 1 measures
the sizing and §5 has fixed the content rules; the named backup formats of D8
(BSMS, Nunchuk, Sparrow: staged-plan 6c) stay deferred (§14).

### 7g. Divergences (refusal / warning / default / documentation)

| step | what else might they do | class |
| --- | --- | --- |
| door | choose Build with no payload | DEFAULT: compose a keyless template (C26); the door SAYS so (§7a) |
| door | payload in flash but skipped at boot, then Build | DOCUMENTATION at the door naming the route (§7a; F-152) |
| pack | two keys share a fingerprint (two accounts) | DEFAULT: labels show fingerprint AND origin |
| shape | more slots than the payload holds keys | REFUSAL at seating transition, naming counts and cause, offers keyless template |
| shape | keyless hash path in tr | REFUSAL (§4e) |
| shape | lock-only path | REFUSAL (§4e) |
| shape | every path hashed | WARNING (§8h) |
| shape | keyless path added / unsorted chosen | EXPERIMENTAL confirm, unskippable (§8a/§8b) |
| shape | edits the wrapper, the path count or a path's key count after a slot was assigned | WARNING before the edit; assignments discarded (§8j) |
| shape | edits a lock or a hash after a slot was assigned | DEFAULT: assignments kept; stub screen re-shown (§7c) |
| shape | a 33rd slot | REFUSAL at the picker (§8m line 5) |
| pack | a malformed `key:`/`hash:`/`now:` record | REFUSAL on the host (§8n); INERT on the device, visible in the door's "not understood" count and, for `key:`, in "Keys loaded" (§6a) |
| pack | operator supplies their own `now:` | DEFAULT: it wins; `me sysw pack` appends none (§6a) |
| pack | seed-only or card-only payload for Build, absolute lock intended | DEFAULT: no bound appended (no `key:`/`hash:` record); the §6b bare disclaimer at lock entry; `--now` adds one (§6a) |
| lock | date before 2009-01-03 | REFUSAL (§8t) |
| lock | date or height before the pack bound | REFUSAL (§6b) |
| lock | no `now:` field for this lock kind | DEFAULT: the "cannot tell the time" line (§6b) |
| lock | relative lock beyond 388/455 days | REFUSAL (§8u) |
| lock | impossible date | REFUSAL at entry |
| seed | payload seed admitted for the first time at Wallet Policy | DEFAULT: the payload spec's F1/F2 flag screens fire before use (§6a) |
| seating | wrong key for a slot | WARNING surface: mapping review; Back keeps choices |
| seating | two slots resolve to the same xpub | REFUSAL at mapping review (§7d) |
| seating | two slots at one origin with fewer than two distinct fingerprints | REFUSAL at mapping review (§8v; §4f invariant) |
| seating | a `key:` record's account or interior origin components | DOCUMENTATION: unverifiable by the device (F-217), printed verbatim at the mapping review |
| seating | card origin script type disagrees with wrapper | DOCUMENTATION: the origin is declared as carried (§4f) |
| seating | one seed fills two slots in ONE path | WARNING (C29, §8g) |
| seating | one seed fills slots in two different paths | DEFAULT: informational line (C5, §8k) |
| stub screen | operator wrote the stub down, then edits the shape | DEFAULT: screen re-shown with §8s's changed-id body (§7c) |
| consent | compares the shown id with a coordinator's | DOCUMENTATION: §8d line; a composed wallet is its own wallet |
| consent | decoded shape or seating differs from the composed list | REFUSAL with an exit (§8q) |
| engrave | keyless composition | DEFAULT: form choice collapses to template only (§7f) |
| engrave | partially seated composition | DEFAULT: template plus cards for seated slots, template stub only, no form A (§7f) |
| engrave | concrete descriptor longer than the plate holds | REFUSAL by census with the measured ceiling (§13 item 1) -- NOT IMPLEMENTED in S3, which ships no concrete-descriptor plate; deferred with the plate to F-457 |

## 8. Copy — operator-facing strings (blockquoted so `plan-glyph-check.sh` scans them; ASCII only; every FIXED body passes the modal-fits assertion, §12 item 5; every confirm-to-proceed screen is dismissed only by a tap on CONTINUE, and Back returns to the shape)

### 8a. EXPERIMENTAL, keyless path (wsh) — confirm-to-proceed, fires once per keyless path

> KEY-LESS PATH (EXPERIMENTAL)
> This path needs no signature. Whoever knows the
> preimage of its hash can spend it. If that preimage
> is ever engraved, the plate is bearer access.

### 8b. EXPERIMENTAL, unsorted keys — confirm-to-proceed, fires once per key set where sorted was legal and declined

> UNSORTED KEYS (EXPERIMENTAL)
> You chose unsorted keys where sorted was possible.
> Key order is part of this wallet. Anyone restoring
> it must keep the same order. Sorted keys need none.

### 8c. Lock echoes (SEVEN separate bodies)

The blocks echo and the packed-HEIGHT bound were folded in at composer S3's
review r0 (M-3). They shipped in `gui/composer_copy.go` tagged §8c while §8c
blockquoted neither: the blocks echo is §6b's table row ("N blocks (about D
days)") and the height bound is this section's date body spliced with §6b's
"heights read `the packed height was H`". A copy row compared against text the
spec does not carry is a self-consistency check, not a spec check, so both are
here now and all of §8's bodies are diffable again.

> 90 days = 15188 units of 512 s (90.0 days)

> 1000 blocks (about 6.9 days)

> Block 905000

> 2027-03-01 00:00 UTC

> This device cannot tell the time. The payload
> says it was packed on 2026-09-01, which may be
> long ago. Nothing here has checked that this is
> in the future.

> This device cannot tell the time. The payload
> says the packed height was 905000, which may be
> long ago. Nothing here has checked that this is
> in the future.

> This device cannot tell the time. Nothing here has
> checked that this is in the future.

### 8d. Stub teaching — §7c, plus this line

> A wallet built here is its own wallet. The same
> rules written by another tool give a different id
> and different addresses.

### 8e. Deprecation note on Multisig Build (C7) — a COMMENT in
`gui/multisig_build.go` and a FOLLOWUPS entry only: "Deprecated 2026-09-01 in
favour of Wallet Policy > Build a new policy. No enforcement by operator ruling."

### 8f. NUMS note (C18), shown when a tr policy falls back to NUMS

> KEY PATH: NONE (NUMS)
> Spends use the script paths only. Bitcoin Core and
> Nunchuk import this form. Liana and BIP-388 signers
> need an unspendable xpub instead (see F-449).

### 8g. Same seed twice in one path (C29): the first body when the shared seed's slots in that path reach the threshold, the second otherwise

> SAME SEED, SAME PATH
> Slots @1 and @2 are the same seed. This path's
> 2-of-3 can be satisfied by one person.
> Liana will refuse it.

> SAME SEED, SAME PATH
> Slots @1 and @2 are the same seed. One person
> holds 2 of the 3 signatures this path needs.
> Liana will refuse it.

### 8h. Every path needs a preimage

> HASH ON EVERY PATH
> Every way to spend this wallet needs the preimage
> of a hash. It is not on this device and not on
> these plates. Back the preimage up separately.

### 8i. Hashlock entry rule (at entry and at consent)

> The hash must be SHA-256 of a 32-byte value. A
> passphrase must be hashed to 32 bytes first, then
> hashed again. A hash of the passphrase itself can
> never be spent.

### 8j. Shape edit after at least one slot was assigned

> EDITING THE SHAPE CLEARS THE KEYS
> Slot numbers change with the shape. Every key you
> seated will be cleared. Continue?

### 8k. A person in two paths (C5)

> One person in two paths needs two keys: a second
> account from the same seed, or a second card.

### 8l. Nothing outside this device has checked this policy (Multisig Build's warning, reused)

> Nothing outside this device has checked this
> policy. Before you fund it, restore these plates
> in your coordinator and compare your own first
> receive address.

### 8m. Structural refusals (§4e), one body each

> Every wallet needs at least one path with a key.

> A path with only a time lock means anyone can
> spend after it. Add a key or a hash.

> This build will not put a key-less path in
> taproot. Use wsh, or add a key.

> Legacy wrappers hold one plain multisig only.
> Use wsh or tr.

> This wallet already has 32 key slots.

### 8n. Host-side record refusals (`me sysw pack` stderr, §6a), one line each; host lines, not device modals, so the panel budget does not apply

> record N: key: needs [fingerprint/path]xpub with
> an origin; a bare xpub is not a key record

> record N: hash: must be exactly 64 hex characters

> record N: now: must be <seconds>[,<height>] in range

> record N: a second now: record; only one is
> allowed. Remove one.

### 8o. Below the pack bound (§6b)

> That is before this payload was packed.
> Choose a later date.

> That is before this payload was packed.
> Choose a later height.

### 8p. Seating shortfall (§7d)

> 4 slots, 3 keys available.
> Unfilled: slot @3.

### 8q. Consent self-check (§7e)

> The policy on this device does not match what
> you built. Go back and check the path list, or
> start again.

### 8r. Door key-state lines (§7a)

> Keys loaded: 4

> Keys loaded: 4, plus 1 seed.

> A seed is loaded. It can fill any number of slots.

> 3 payload records were not understood.

> No keys loaded. This builds a key-less template.

> A payload is in flash but not loaded.
> Load it from the carousel first.

### 8s. Seating and stub-screen lines (§7c, §7d)

> The shape changed, so this id changed. Cards
> minted with the old stub will not seat here.

> Slot @2, Path 1 key 2 of 3: choose a key

> Slot @0, key path (spends alone): choose a key

### 8t. Date floor and ceiling (§6b)

The CEILING body is F-456, folded in at composer S3's review r0 (M-3). §8t
covered only the floor, so a date past 2038-01-19 was refused as "that date
does not exist" -- false of 2045-06-01, and on the archetype §4d lists first a
twenty-year date is the ordinary case. F-458 is the separate defect that the
dispatch between the two could not tell them apart.

> This build will not write a date before 2009
> as a time lock.

> This build writes dates up to 2038-01-19. For a
> later time, use a block height instead.

### 8u. Relative lock ceiling (§6b)

> Relative locks reach at most 455 days in blocks
> or 388 days in time. Use an absolute date.

### 8v. Same origin, too few fingerprints (§4f invariant, at the mapping review)

> Two keys declare the same origin and not both
> carry a fingerprint. This template could not be
> restored. Use cards or records with fingerprints.

## 9. Device work items (fork)

1. `md` tree BUILDER API and an extended `md.PolicyShape`: construct a
   `descriptor` from a path list and emit keyless and keyed md1 in CHUNK form
   through the existing `split` (`md/chunk.go:121`), the artifact every consumer
   accepts (`encodeMD1String`'s single-string form is rejected downstream with a
   wire-version mismatch and is kept only for the corpus's single-string parity
   leg), byte-identical to the Rust `compose` vectors (§10); split `or_i`/`or_d` (and `andor`, for consumed
   cards) into separate `Branch`es in `md/policy_shape.go` and carry the lock
   operand and the digest on `Branch` so §7e can render them.
2. `pk_h` emitter arm in both script contexts (`md/script_emit.go`), with a
   mutation check that the hash changes the address (C17); a prerequisite for
   every §7e and §12 item 1 address of a policy with a single-key wsh path.
3. Lock entry (§6b): kind and unit pickers, the digit-pad widget with the
   eight-digit date field, days-to-units and date-to-Unix conversion, the
   floor/ceiling refusals, the bound line, and the device-side §4c range check
   independent of md.
4. Wallet Policy door ChoiceScreen in every state with its key-state lines (§7a);
   the admission row and flag-screen wiring (§6a); the three payload classes in
   `sysw.Classify` lockstep with the host.
5. Path-list screen with the slots/keys line and Back-preserves-everything (§7b);
   presets populating a path list (§4d); the §4e structural refusals and picker
   bounds.
6. Stub-teaching screen as a PAGED widget with a stated per-frame slot budget,
   re-show on edit, the conditional per-slot line and the §8d line (§7c); the
   literal id/stub relabelling on both shipped screens.
7. Seating pick list (§7d) as a PAGED widget with stated capacity (the shipped
   `ChoiceScreen` does not scroll, `gui/gui.go:1993-2026`; a payload may hold more
   rows than the 232 px content box shows), slot-directed assignment, the
   same-xpub refusal, the §4f invariant refusal (§8v), discard-on-numbering-change
   with the §8j confirm, the mapping-review screen with verbatim origins, the
   unverifiable-account note, the C29 warning and the §8k line, ms1 legs wired into the seed
   source picker, per-slot accounts via the Multisig Build machinery.
8. Taproot origin arm: `3'` in `multisigScriptTypeComponent`
   (`gui/multisig_build_slots.go:125-130`, "the ONE site that decides it") and a
   taproot member on `md.MultisigScript`, or the composer's own origin function
   with the same table (§4f).
9. Composer consent surface (§7e) on `confirmReviewScreen`'s paged form, with
   the decoded-shape-and-seating self-check and its §8q exit, and the §8l
   warning; hashlock entry (§6c) with the §8i line; the §8h all-hashed
   warning; the §8a/§8b confirms.
10. Engrave form choice (§7f): keyed md1 and keyless md1 with fingerprints via
    item 1; card minting for `key:`-sourced and seed-derived slots and re-minting
    with both stubs via `mk.Encode`; concrete descriptor text and QR plates via the
    transaction program's plate machinery after §13 item 1's ceiling measurement;
    the census refusal.
11. Deprecation comment (§8e); scrub-on-exit through `buildMultisigSeedHook`'s
    seam (C14); the two comment rewrites named in §6a.

## 10. Host work items (Rust first)

1. md-codec `compose` module (C15): path list → template tree; `md compose`
   subcommand beside `md compile` with the opposite contract. `compose` is
   UNCONDITIONAL (not behind `cli-compiler`; only §5b's compile leg needs that
   feature, which CI supplies with `--all-features`). Vectors per §12 item 1,
   with the §5b cross-check, in the corpus the Go side consumes; divergent
   per-slot origins are written INLINE in the vector templates, since the corpus's
   `path` field carries one shared path.
2. `me sysw pack`: `key:`, `hash:`, `now:` classes with the §6a body rules and
   the §8n refusal lines from `pack_with`; the payload-wide single-`now:` rule at
   the same site; `now:` appended last ONLY when the operator's records hold none
   and include a `key:` or `hash:` record, or when `--now` is passed (an
   operator-supplied `now:` wins; `--now` and `--no-now` conflict); `--no-now`
   suppresses the auto-append for deterministic fixtures; a payload with no
   composer-only record packs byte-identically to today.
3. The five presets as Concrete policies + expected templates (C2).
4. `md-older-zero-time-units-not-refused` patch (independent; filed).
5. mnemonic-secret: `ms derive --template bip48-p2tr` (= `m/48'/0'/account'/3'`)
   so host-derived taproot `key:` records match the device's C28 origin
   (`ms-derive-taproot-justifications-stale`, second half); this flips the shipped
   negative test asserting the template is refused
   (`crates/ms-cli/tests/cli_derive_bip48.rs:174-178`), which is renamed, not
   deleted.
6. `SPEC_systemwide_payloads.md`: the section 3.3.1 rows, the CREATED Wallet
   Policy row in 3.3.2 (and, while there, the missing `progTransaction` row is
   noted for its own owner), the section 5.3 prefixes — a normative artifact with
   its own R0 history, folded under its own gate, not inside this spec's prose.

## 11. Refusals — what the operator SEES

Every refusal in §4e, §6a, §6b, §6c, §7d and §7g names what to do instead and
prints no encoding. §12 item 4 is the family that fails if any of them fails to
refuse; the copy of each refusal is a blockquote in §8 or a quoted string in its
table, so the glyph and modal-fits gates cover it.

## 12. Acceptance — NORMATIVE

1. **Positive vectors, Rust first, Go chunk-identical — TAGGED COVERAGE, not a
   product.** Every vector names the §5 rows, §4c lock rows and §4f origin rows
   it exercises, and a script asserts each tag appears in at least TWO vectors,
   except a tag with exactly one legal shape, which is named as such in the
   test (m = 0 is one unlocked single key and nothing else) (the full product is 28,800 cells and is not an acceptance anyone can build;
   a pairwise covering array over the legal axes needs about 50-60 named
   vectors). Required tags include: all four wrappers; path counts 1, 2, 3, 4
   and the 32-slot maximum; taptree spine shapes m in {0, 1, 2, 3, 7}; the
   extracted internal key first-listed AND not first-listed with ≥ 4 paths;
   NUMS; the five lock encodings; hash present; sorted and unsorted; keyless
   wsh; the three fingerprint cases (declared, one seed at two slots in one
   path, one seed across two paths); unseated-slot origins per wrapper. Each
   vector: path list → template text → md1 CHUNKS (keyless and keyed) → addresses
   (receive 0..1, change 0..1) → both ids → the consent lines. The §5b
   cross-check holds; the Go builder reproduces every template, every CHUNK and
   every address byte for byte; a separate named leg compares the single-string
   payload bytes.
2. **Journey, EXECUTED.** The C8 workflow on the emulator: `me sysw pack` with
   `key:` records, a `hash:` record, a `now:` record and a seed; flash; Wallet
   Policy → Build → shape → stub screen → seating → mapping review → consent →
   engrave form → census, with the consent's ids and addresses compared against
   `md` output; the capture refuses to finish on a mismatch and the negative
   control is run. A plan may not close while this gate has never run.
   **EXECUTED 2026-09-03** on the emulator: `design/journeys/capture_composer.py
   --arm both` (fork `6fb90cb`, three legs, 50 shots) matched the payload
   digest, both ids, both stubs, all four addresses and every engraved string
   byte for byte; the negative control (`--prove-it-can-fail`) attributes its
   failure to the address comparison, and the unchunked-string substitution is
   caught (56 vs 47 chars). Records: `composer-S4-implementation-report.md`,
   `composer-S4-exec-review-r0.md`, the journey
   `SeedHammer-II-composer-journey.pdf`.
3. **Emulator walk with NO payload** (C26): door line present, shape, stub screen
   with per-slot expected origins, consent stating no addresses, form choice
   collapsed, keyless-template engrave whose md1 decodes on the device and whose
   slots carry distinct-account origins.
   **EXECUTED 2026-09-03**: the keyless arm of the same capture (`--arm
   keyless`): the C26 door line, tr 2-of-3, the stub screen with per-slot
   origins at accounts 0', 1', 2', consent "Keyless template - no addresses.",
   the form choice collapsed, one plate whose string is
   `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` (the device's
   chunk form; `md decode` prints the three distinct-account origins). The
   physical plate is the S4 device walk's (Task 4), pending.
4. **Negative vectors: every refusal refuses.** For each §4e, §6a, §6b, §6c, §7d
   and §7g refusal, an input that must be refused and the exact §8 line shown;
   including a lock operand outside §4c refused BY THE DEVICE on an md build that
   still accepts it (`older(0x400000)`), a date before 2009-01-03, a date and a
   height below the pack bound, a `hash:` of 31 and 33 bytes, a bare-xpub `key:`,
   a 33rd slot, two OPERATOR `now:`
   records (refused by host `pack_with`; inert on the device via `syswSession.load`), a same-xpub double seating, two slots at one
   origin with one fingerprint (§8v), a path-count edit AND a wrapper change after
   a slot was assigned (discard), and a lock edit after a slot was assigned (kept). The §7e self-check is exercised by
   FAULT INJECTION (flip one builder output, assert §8q fires), not by an input.
5. **Copy gates:** `scripts/plan-glyph-check.sh`, the raster floor
   (`gui/raster_test.go`), AND the modal-fits assertion (`gui/modal_fits_test.go`,
   `assertModalBodyFits`) on every §8 body and every new screen; plus a
   fires-on-condition test for each of §8a, §8b, §8f, §8g, §8h, §8j, §8k, §8l, §8m,
   §8n (host), §8o, §8p, §8q, §8r, §8s, §8t, §8u, §8v and the §6b bound and no-bound lines; the variable-length
   screens (§7c stub screen, §7e consent, §7d pick list) are asserted by PAGING
   capacity at the measured per-frame budget, since a fits assertion cannot pin a
   body with no single source string.
6. **Seating and cards.** For every keyed vector: the re-minted or minted cards
   seat into the engraved keyless template through the shipped `seatKeyCards`
   (both stubs when a keyed policy exists, the template stub otherwise; existing
   stubs preserved) and reproduce the keyed policy's addresses. §4f's invariant
   holds for EVERY emitted template: no two slots share an origin unless both
   declare distinct fingerprints; a named negative vector runs the asymmetric
   one-card case (one slot with a fingerprint, one without, at one origin)
   against `seatKeyCards` and asserts it is never produced; the partially seated
   artifact (template + template-stub cards for seated slots) is a named vector.
7. **Device-side lock range check** is a unit gate on the emitter's input, not on
   md's acceptance: every §4c boundary value in and out, per kind.
8. **Record classes, lockstep.** A cross-language vector set: each `key:`,
   `hash:`, `now:` record (valid and each §6a malformation) classifies identically
   on the host and on the device; for each malformation the host emits its §8n
   line and the device leaves the record inert, with "Keys loaded" reduced by one
   for a `key:` and the "not understood" count raised by one for every class.
9. **Engrave surface.** Per journey: the form choice offered (A and B, B only,
   template only), Full versus Watch-only, the three secret forms, the census
   lines, the read-back-integrity line; the census refusal on the measured ceiling
   is NOT asserted: §13 item 1 now carries the number (596 characters), but S3
   ships no concrete-descriptor plate for it to guard, so the refusal and its
   assertion both belong to F-457.
    **EXECUTED 2026-09-03** per journey on the emulator: form A / form B offered
    on the keyed arm, template-only on the keyless arm; Full versus Watch-only
    asked when a seed-seated slot exists (Watch-only taken); the census lines
    photographed (`shots/c12-census-A.png`, `c12-census-B.png`,
    `k04-census.png`).
10. **Multisig Build parity (optional, C7 comment-only):** the `sortedmulti`
    preset with seed-derived slots reproduces `gui/testdata/t6b_multisig_full.md1.txt`.
11. **Cite gate:** `scripts/plan-cite-check.sh` on this spec before each R0 round;
    `CITE_FORK_ROOT` set to the working tree under review.

## 13. What is NOT verified

1. **Plate ceilings for a concrete descriptor.** 688 chars for the two-path
   wallet (brainstorm record, C10); text plates per default font and QR symbols per
   plate are measured on the emulator, not read off a constant
   (`gui/transaction.go:1369`). The per-frame capacities of the three paged
   screens (§7c stub screen, §7d pick list, §7e consent) are the same kind of
   plan-time render measurement -- `Choice.Size` and `widget.Labelw`'s sizes
   exist only at render time, so no static read produces them. MEASURED at
   composer S3 on fork `composer-s3` (parent `321acb56`), re-measured at S4 on
   `composer-s4c` `0b49f66`, by
   `CGO_ENABLED=0 go test -run '^TestComposerMeasureSection13Numbers' -v ./gui/`
   (`gui/composer_measure_test.go`), at the SeedHammer II display size:

   ```text
   SPEC13 stub_screen    lines= 42 per_frame= 6 pages=7
   SPEC13 pick_list      lines= 36 per_frame= 7 pages=6
   SPEC13 consent        lines= 17 per_frame= 7 pages=3
   SPEC13 descriptor_plate ceiling_chars=596  c10_688_fits=false
   ```

   So the pick list and the consent hold **7 rows per frame** and the stub screen
   **6** (re-measured 2026-09-03 on fork `composer-s4c` `0b49f66`, after S4 walk
   W-3 made every paged line wrap inside the band left of the navigation column:
   `Template-ID: <32 hex>` now takes two lines instead of drawing its last digit
   under the Back button; before W-3 all three held 7), and the concrete
   descriptor plate ceiling is **596 characters** at this platform's params --
   measured by binary search against a real `backup.Text` plate
   (`composerDescriptorCeilingChars`), never written down as a constant. C10's
   688-character two-path wallet therefore does NOT fit one text plate, which
   is why §7f's form B (template plus key cards) is the form that carries it.
2. **Ledger registration of md's depth-0 xpubs.** Core, Liana and Sparrow accept
   them (measured); Ledger's whole-xpub `memcmp` likely does not: UNVERIFIED,
   filed descriptor-mnemonic `md-descriptor-depth0-xpub-ledger-registration`.
3. **Nunchuk UI** treatment of `or_i` vs `or_d` (library is Core's verbatim) and
   of custom-template miniscript imports.
4. **Import tests** of composed outputs into Core, Nunchuk, Liana are import
   tests, not emit tests, and belong to the journey (§12 item 2) or to F-449;
   Liana's import refuses any `after` or hashlock path regardless of head
   (second lowering review), so F-449's acceptance wallet must be `older`-only.
5. **Recon results folded here were verified against Core v25 (single-chain
   forms; the local build lacks BIP-389 multipath and tapscript miniscript),
   Liana master and drongo HEAD**: brainstorm record section 3.11. A newer Core
   re-run on the multipath forms is a plan-time gate, not a spec question.

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
| on-device preimage derivation, storage or engraving | C25; §6c |
| removing or redirecting Multisig Build | C7: comment only; its dead-end (F-150 item 1) stays as filed and is not fixed by this cycle |
| scrub timing at every abandon point of the composer's seed screens | secret-handling, non-gating by the 2026-08-27 ruling; filed as a follow-up for optimisation |
| on-screen QR display of a descriptor | staged plan 6b, deferred |
| BSMS / Nunchuk / Sparrow named backup formats | staged plan 6c and D8, deferred on-device |
| Sealed-Payload memory discipline for seeds | C14 |
| `andor` emitter arm | not in the grammar (consumed cards keep the summary split of §9 item 1) |
| networks other than mainnet | D1, `gui/policy_address.go:61` |

## 15. Process

Risk-set work on three counts (normative codec behaviour, funds/keys/addresses,
spans repos). R0 to 0C/0I before code; reports persisted by the agents to
`design/agent-reports/`; persist and fold are two commits; the build gate runs on
every fold (`scripts/plan-build-gate.sh` for Rust blocks, `plan-build-gate-go.sh`
for Go, `plan-cite-check.sh`, `plan-glyph-check.sh`, `spec-structure-check.sh`,
`plan-table-check.sh`, `plan-stepref-check.sh`, `plan-wiring-check.sh`,
`fold-propagation-check.sh` with each fold's superseded phrasings, and
`plan-staleness-check.sh` against the heads table at the top of this document as
its baseline).
Rust first: `compose` and its vectors land in descriptor-mnemonic before the Go
builder. UC is OFF for implementation. A plan's GREEN expires: re-validate the
plan immediately before dispatching its implementer.
