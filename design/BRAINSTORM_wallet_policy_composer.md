# Brainstorm — on-device wallet-policy COMPOSER (arbitrary tr/wsh authoring on the SH2)

**STATUS: BRAINSTORM IN PROGRESS, 2026-09-01.** Rulings below are the operator's
and are final; everything else is measured context or an open question. No R0
review, no gates, nothing may be implemented from it. Written mid-session so the
rulings outlive the context ([[decisions-must-outlive-the-agent]]).

Companions: `HANDOFF_arbitrary_tr_wsh.md`, `Preliminary-Brainstorm-Arbitrary-Tr-Wsh-Wallet-Backup.md`,
`STAGED_PLAN_tr_wsh_concrete_descriptors.md` (Stages 0-5 DONE; this is the
"compose later" of its ruling D1, now arriving). F-150 items 3 and 4 are this
brainstorm.

Heads measured against: fork `169073c`, mnemonic-engrave `83aad9d`,
descriptor-mnemonic `2ca75116` (md 0.14.0), mnemonic-toolkit `d8f06483`.

---

## 1. What the operator asked for

"SH2 can consume host-created arbitrary policies but not yet create them. We need
to brainstorm how to get SH2 to allow arbitrary policy authoring."

## 2. Rulings (operator, this session, verbatim where quoted)

| # | Ruling | Consequence |
| --- | --- | --- |
| C1 | **Authoring model = spend-path list.** "Option 1." The operator adds paths one at a time: which keys, threshold, optional timelock, optional hashlock. A fixed deterministic lowering turns the list into miniscript, defined Rust-first with vectors. No compiler on the device. | Not a miniscript tree editor, not an on-device policy compiler. |
| C2 | **Archetypes are presets over the spend-path grammar, not the menu.** "I accept your recommendation for the second." | The toolkit's five `build-descriptor` archetypes become one-tap presets that populate a path list; their byte-pinned goldens are free cross-implementation vectors. |
| C3 | **Firmware-resident vocabulary, defined Rust-first — NOT a host-generated fragment menu shipped to the device.** | The device already owns the serializer and emitter; a per-wallet menu over the payload is a template in pieces with an extra trust hop. |
| C4 | **Template first.** "Template first." Compose the keyless shape, then seat keys. | Seating is slot-directed (see open Q1); the output feeds Wallet Policy's existing consent/review/engrave tail. |
| C5 | **Key reuse: one slot, one path. Reuse the MASTER via distinct hardened accounts, never the key.** The wire stays narrow (F-417 reaffirmed). The unhardened-child route is a recon item behind an md admission change, not a wire change. | See §3.4; measured end to end. |
| C6 | **Front door = a "Build" choice inside the Wallet Policy program.** "Inside Wallet Policy" | The door must become a ChoiceScreen in EVERY state (today the no-payload path drops straight into the NFC gather with no screen). Enum comment at `gui/gui.go:191` ("came from OUTSIDE this device") is rewritten. |
| C7 | **Deprecate Engrave Multisig's "Build policy" path**, "with the goal of eventually migrating standard multisig wallet descriptor policy/template authoring in Wallet Policy." **Deprecation is a COMMENT, with no enforcement** — operator, verbatim: "For deprecation, a comment with no enforcement is a feature, not a bug." | No removal, no redirect, no migration gate this cycle. A deprecation note in the code and a FOLLOWUPS entry; nothing else. (The controller's proposed byte-identical migration gate against `gui/testdata/t6b_multisig_full.md1.txt` was NOT adopted — it stays available as an optional acceptance check if the composer ever ships a `sortedmulti` preset.) |
| C8 | **Seating is PAYLOAD-based first; NFC later.** Operator, verbatim: "we must first implement payload based gathering as I don't yet have nfc hardware. The workflow I foresee is: a user packs m keys into a payload on host and then flashes sh2, user opens wallet policy and selects build for wallet policy and then authors policy template and after than a screen prompts user to associate key slot in template with one of the remaining keys on the list of keys in the payload" | Slot-directed assignment from a pick list of the payload's mk1 cards, each key used at most once ("remaining"). Measured: `me sysw pack` admits mk1 records (`Class::MdMk`, `crates/me-cli/src/sysw/record.rs:264`); payload region 64 KiB (`sysw/wire.go:28`), ~200 B per two-chunk key card; `ctx.syswBundleSeeds` already carries payload cards into flows (`gui/multisig_build.go:183`). "My key from my seed" is deferred behind this. |
| C9 | **Teach the mk1 stub after authoring, unconditionally.** Operator, verbatim: "After authoring a wallet descriptor template, the user must be taught the stub required for encoding mk1 cards, whether or not they intend to import keys via mk1…they might later choose to use mk1 encoding." | The screen after authoring shows the keyless template's stub (WalletDescriptorTemplateId, first 4 bytes — key-independent and origin-invariant, so it is final before any seating) with the host spelling `mk encode … --policy-id-stub XXXXXXXX`; after seating it adds the keyed policy's stub (WalletPolicyId) and recommends stamping BOTH on a card (`--policy-id-stub` is repeatable), so one card matches either form of the wallet — the "one wallet has TWO stubs" trap of mk SPEC §3.3, answered on the device. Mechanism exists: `md.FormAwareStubChunks` / `md.FormAwareIdChunks` (`md/template_id.go:122,163`). Shipped-copy inconsistency to fix alongside: `gui/wallet_policy.go:194` prints the 16-byte id as `Template-ID:`/`Policy-ID:` while `gui/template_engrave.go:70,79` prints the 4-byte STUB under the same `Template-ID:` label — the composer's screen must label id and stub distinctly. |
| C10 | **What is engraved: the operator chooses the FORM.** Operator, verbatim: "user chooses a concrete wallet descriptor policy or a wallet descriptor template and keys (as both plain text qr or md1 strings for concrete policy or md1+mk1 strings for template)". | Two forms. (A) CONCRETE policy — as plain-text plates, as QR plates, or as keyed md1 strings. (B) TEMPLATE + keys — keyless md1 + mk1 key cards. (A) re-opens staged-plan **6d** (engraving a concrete descriptor, deferred 2026-08-20 for "unmeasured sizing"). Measured 2026-09-01 on the §3.4 wallet: concrete descriptor **688 chars**, keyless template 203 chars, keyed md1 8 chunks. Plate machinery exists in the transaction program: greedy text packing (`gui/transaction.go:1145`) and 16-symbol structured-append QR (`txqr.MaxSymbols = 16`, ceiling measured by search at `gui/transaction.go:1369`, no constant). Per-plate ceilings for a 688-byte descriptor are NOT yet measured — a spec task. Read-back integrity differs by form: md1/mk1 carry BCH; a text/QR descriptor carries only its BIP-380 checksum. "plain text qr" = plate kinds (text plate, QR plate); on-screen QR display (6b) stays deferred. |
| C11 | **Timelock UI must respect the very different ranges of the lock kinds.** Operator, verbatim: "we must be careful to get timing right for UI as some locks have a very different range than others". | The operator never types a raw `older()`/`after()` operand. UI asks kind + unit, computes the encoding, refuses out-of-range. Facts: `older` (BIP-68 CSV) value is 16 bits, 1..65535 blocks (≈455 d) or, with bit 22 set, 1..65535 × 512 s (≈388 d); consensus MASKS the operand to 0x0040FFFF, so a typed `older(105120)` ("2-year vault in blocks") silently becomes 39584 blocks ≈ 275 d — the exact hazard the toolkit's `SPEC_older_timelock_mask_gate.md` gates (accept 1..=65535 or 0x400001..=0x40FFFF; reject 0, 65536, 105120, 0x400000, bit 31). `after` (BIP-65 CLTV) is absolute: < 500,000,000 = block height, ≥ 500,000,000 = Unix time (e.g. 1893456000 = 2030-01-01 UTC). Worked decodings of all four kinds: `design/agent-reports/miniscript-nesting/timelocks.md` §0. Height-based and time-based locks may not mix within one spend path (miniscript timelock-mixing rule); the device shows each lock as a real duration or date, not a number. |
| C12 | **Seeds are a key source too: BIP-39 phrases or ms1 strings, via the standard (unsealed) payload or typed on the device.** Operator, verbatim: "user should also be able to supply seed phrases or ms1 strings via standard (unsealed) payload or typed on the device". Supersedes C8's "my key from my seed is deferred". | The seating pick list has three kinds of source: key records (§3.6), mk1 cards (stubs ignored), and SEEDS. A seed may fill several slots; each held slot gets its own hardened account by ordinal among the slots that master fills (Multisig Build's S5 rule, `gui/multisig_build.go:594-601`, `seedRegistry`) — C5 applied. Per-seed passphrase as Build offers (`buildSeedForSlot`). All four legs exist as primitives — payload words + payload ms1 (`gui/gui.go:2856`, Backup Wallet: `syswOffer(ClassCodex32Secret, "Seed from where?")`), typed words (`seedEntryFlowTitled`: FROM PAYLOAD / TYPE IT), typed ms1 (`inputCodex32Flow`, `gui/gui.go:1262`) — but Build's seed entry wires only the two WORD legs; the ms1 legs need wiring into the seating source picker. **Deliberate reversal:** Wallet Policy's payload admission row (`gui/sysw_admit.go`, today Descriptor + MDMK only, "no seed class" per Stage 4) widens to Mnemonic, Codex32Secret, Passphrase and the new Key class; the enum comment's "needs neither a seed requirement" rationale is rewritten. sysw §7.4 bars payload secrets from VERIFY flows only, so a composer may take them. |
| C13 | **Seed-derived slots get Build's two engrave modes, and the SECRET's plate form is the operator's choice.** Operator, verbatim: "Yes. User chooses to engrave plain text qr or ms1 strings via sealed" — then, asked about "via sealed": "If I typed via sealed I meant unsealed". | "Full (seed + keys)" vs "Watch-only (keys)" as Multisig Build offers (`gui/multisig_build_census.go:475`). In Full mode the secret is cut as words (plain text), as a SeedQR, or as ms1 strings — the three secret plate forms the device already has (`engraveSeed` primitive: words + standard SeedQR, `gui/bip85.go:131`; codex32 plates via the ms1 flows). The secret ARRIVES via the unsealed payload or the keyboard (C12); the Sealed Payload program stays frozen and untouched (`SPEC_systemwide_payloads.md:26`). |
| C14 | **No Sealed-Payload-grade memory treatment for the Build path.** Operator, verbatim: "This wallet policy menu option does not get the same careful memory treatment as sealed payload program". | Seeds held by the composer are handled as Multisig Build handles them — scrub on exit through the existing seam (`buildMultisigSeedHook`) — and NOT with Sealed Payload's wipes, idle timer, KDF and private session semantics (`SPEC_systemwide_payloads.md:26,266`). Consistent with the 2026-08-27 severity ruling: secret-handling defects are follow-ups, never gates. |
| C15 | **Rust home of the lowering = a `compose` module in md-codec, surfaced as `md compose`.** Operator: "I accept your recommendation" (2026-09-01). | One provenance: the fork's Go `md` package already pins `descriptor-mnemonic/crates/md-codec @ 0.42.0` (`md/bits.go:3`). Vectors join the corpus the Go side already consumes; rust-miniscript type + sanity checks run in the crate's tests; the toolkit's five archetype goldens become cross-check vectors (each archetype as a path list must lower to its byte-pinned descriptor). `md compose` sits beside `md compile` with the opposite contract — fixed rules, no search. Not chosen: a separate crate (second pin/publish); the toolkit builder (wsh-only, funds crate, second provenance). md-cli is binary-only (no lib target), so the library lives in md-codec. |
| C16 | **Grammar bounds APPROVED as proposed, with amendments.** Operator: "Otherwise proposed grammar bounds approved. Agree with recommendations for 1), 2) and 3) but for 3 you may permit multi under the same experimental warning". Asked: "Can we use pkh instead of pk for n=1?" | Grammar: wrapper `tr`/`wsh` for any list, `sh(wsh)`/`sh` only for a single unlocked `sortedmulti` path (Multisig migration); 1..8 paths; k-of-n over FRESH slots, n 1..9 (inside `multi` ≤ 20 / `multi_a` ≤ 999 / md1 32-per-fragment: `rust-miniscript limits.rs:35,38`, `md-codec tree.rs:92-120`); at most one timelock and one `sha256` per path; taproot key-path = the single unlocked one-key path if exactly one exists, else NUMS; one slot one path (C5). Rulings inside: (1) KEYLESS hashlock-only paths ADMITTED behind an unskippable EXPERIMENTAL screen naming bearer access; (2) legacy wrappers KEPT for the single-path case; (3) sorted thresholds by default, and unsorted `multi`/`multi_a` PERMITTED under the same EXPERIMENTAL warning (cost: template recovery of an unsorted path needs the permutation search the template-engrave estimate describes). **`pkh` for n = 1 — answer pending the operator's ruling; controller's recommendation: `pkh` in wsh, `pk` in tr.** Facts: in wsh the whole script is revealed on spend, so `pkh` hides the pubkeys of UNSPENT single-key branches behind HASH160 and saves 10 script bytes per such branch, at ~33 extra witness bytes when that branch spends — the toolkit's inheritance archetype spells its heir `pkh` for this reason; in tr unspent leaves are never revealed, so `pkh` buys nothing and costs witness bytes. Device cost: the fork's emitter has ZERO `pk_h` arms (`grep -c tagPkH md/script_emit.go` = 0; only `pk_k`), so `pkh` needs a new emitter arm in both contexts, Rust-first with vectors, before a pkh policy derives on-device. RCW spells its single-key tier `pk` in both forms. |
| C17 | **`pkh` for a single-key path in wsh, `pk` in taproot leaves.** Operator: "I accept your recommendation but file a followup 'investigate pkh vs. Pk in taproot wallets under the assumption that a cryptographically relevant quantum computer exists'". | Filed as **F-448** (`design/FOLLOWUPS.md`), recon item, not a composer gate. Work item for the cycle: a `pk_h` emitter arm on the device in both script contexts (today zero arms, `md/script_emit.go`), Rust-first with vectors and a mutation check that the hash changes the address. |
| C18 | **NUMS internal key stays the raw BIP-341 `H` this cycle; the Liana-import xpub form is filed, not built.** Operator: "File Liana wallet import followup" (2026-09-01), after the §3.9 survey and the controller's recommendation to keep raw `H` and document the BIP-388 gap. | Filed as **F-449** (`design/FOLLOWUPS.md`): a second internal-key KIND on the md1 wire, its own constellation cycle, not a composer gate. Composer copy and docs state that Core and Nunchuk import the raw form and that Liana and BIP-388-strict registration refuse it. Resolves the I5 tier question from the fable review (documentation now). |
| C19 | **Fable review findings, adopted/pending.** Operator (2026-09-01): "C1 agree / I1: ask miniscript expert agent / I2-4: agree / M3,n1-n3: agree". | ADOPTED into the lowering rules: **C1** placeholders numbered by first appearance in the EMITTED text, slot labels are those indices (internal key is `@0` when extracted); **I2** keyless paths wsh-only, refused in tr; **I3** lock-only paths refused ("anyone can spend after N"); **I4** at least one keyed path, checked before lowering; **M3/N1-N3** documented, no rule change. **I1** (`or_d` vs `or_i` for a single-key head) → second expert review dispatched 2026-09-01, report `design/agent-reports/composer-lowering-i1-single-key-head-miniscript-review.md`. **M1** (first-listed unlocked single key as internal key) and **M2** (lock ranges stated in the rule) NOT yet ruled. Review report persisted at `e9f2ba0`. |
| C20 | **M1 ADOPTED with the fallback answered; M2 is not a ruling but a SOURCED fact, found and cited.** Operator: "M1 agree, but what if there is no single key path?" and "M2: this isn't open to interpretation: there is only one correct answer and must be found and cited." | **M1:** internal key = the FIRST-LISTED unlocked, unhashed one-key path (not also a leaf); if NO such path exists → the NUMS point, spelled raw `H` (C18). BIP-341 l.157 names one other option for that case — an aggregate "everyone agrees" key — which needs MuSig2 and is outside this grammar. **M2 — the admitted lock values, each bound with its source (fetched 2026-09-01):** `older(n)` — consensus reads ONLY bit 31 (disable: no meaning), bit 22 (type: 512-s units), and the low 16 bits under mask `0x0000ffff` (BIP-68 l.30-40, l.74-83; BIP-112 l.28-33); "A relative time-based lock-time of zero indicates an input which can be included in any block" (BIP-68 l.46); miniscript admits `1 <= n < 2^31` (BIP-379 l.135; rust-miniscript `relative_locktime.rs:73`: `is_relative_lock_time() && seq != ZERO`). Therefore the ONLY values whose written form equals what consensus enforces are **n in 1..=65535 (blocks, ≤ 455.1 d) or n = 0x400000 + u with u in 1..=65535 (512-s units, ≤ 388.4 d)**; every other miniscript-admitted value is masked to a different or zero lock (toolkit `SPEC_older_timelock_mask_gate.md` l.19-24 has the exact predicate: reject iff `(n & !0x0040_FFFF) != 0 || (n & 0xFFFF) == 0`). `after(n)` — CLTV compares like with like: `n < 500,000,000` is a block height, `>= 500,000,000` a Unix time (BIP-65 l.27, l.243-250; Core `script.h:48` `LOCKTIME_THRESHOLD{500'000'000}`); nLockTime itself runs to `0xFFFFFFFF` (BIP-65 l.230-233; `script.h:54`) but miniscript caps at `1 <= n < 2^31` (BIP-379 l.135; rust-miniscript `absolute_locktime.rs:10,52` `MAX_ABSOLUTE_LOCKTIME = 0x7FFF_FFFF`). Therefore **height 1..=499,999,999; time 500,000,000..=2,147,483,647 (1985-11-05 00:53:20 UTC .. 2038-01-19 03:14:07 UTC)**. Measured on md 0.14.0: `after(0)`, `after(2^31)`, `after(2^32-1)` refused by rust-miniscript; `after(1)`, `after(499999999)`, `after(500000000)`, `after(2147483647)` encode; `older(0)`, `older(2^31)` refused by rust-miniscript; `older(65536)`, `older(0x40FFFF+1)`, `older(2^31-1)` refused by md's mask guard; `older(1)`, `older(65535)`, `older(0x400001)`, `older(0x40FFFF)` encode — **and `older(0x400000)` ENCODES** (`md1yqpqqxpye5kuqpqqqqqvkwqu7r50qu85`): a time-based lock of ZERO units, i.e. no lock, admitted by rust-miniscript (≠ ZERO) and missed by md-codec's guard, which checks only `v & !consensus_bits != 0` (`validate.rs:225`) and lacks the `(n & 0xFFFF) == 0` clause the toolkit gate has. Filed in descriptor-mnemonic FOLLOWUPS (see below). |
| C21 | **I1 ADOPTED: a single-key wsh head lowers to `or_i(pkh(P1), rest)`; `or_d` is reserved for a bare `multi` head.** Operator: "I accept your recommendation" (2026-09-01), after two independent reviews agreed (`design/agent-reports/composer-lowering-rules-bitcoin-expert-review.md` I1, persisted `e9f2ba0`; `design/agent-reports/composer-lowering-i1-single-key-head-miniscript-review.md`, persisted `e0375f1`). | Measured price of C17 at the head: `or_i(pkh)` costs 26 WU (6.5 vB) more than the BIP-388/compiler form `or_d(pk)` on every P1 spend and 10 WU less on every other path's spend; break-even 27.8 % P1 share. `or_d(pkh)` is dominated (+34 WU on every other spend AND publishes P1's key). Named drop-in if C17 is ever revisited for the head: `or_d(pk(P1), rest)`. Side finding for F-449: Liana's import refuses any `after` or hashlock path regardless of head, so its acceptance wallet must be `older`-only. |
| C22 | **No `or_d` anywhere: wsh paths combine as a uniform `or_i` chain, multi-key head included.** Operator (2026-09-01): "I don't think I want or_d for multi key head". Amends C21's multi-head clause. | Measured cost of dropping `or_d` for a bare `multi(k)` head: +2 WU on each head-path spend, −k WU on every other path's spend, no key exposure either way (first review M3; second review point 6, break-even head share k/(k+2)). One combinator for the whole wsh grammar; matches the combinator shape of the pathological fixture (`or_i` chain). `or_d` appears nowhere in the composer's output; the compiler's and BIP-388 l.249's `or_d` forms remain valid inputs the device can still CONSUME and derive. |
| C23 | **C22 WITHDRAWN: `or_d` for a bare multi head stands, as C21 ruled.** Operator (2026-09-01): "I take that back, I do want or_d for multi head. Proceed." | Table §3.10 restored to the C21 form: `or_d(P, R)` iff `P` is a bare unlocked, unhashed `multi(k,…)`, n ≥ 2; otherwise `or_i(P, R)`. C22's row is kept for the record only. |
| C24 | **Current time reaches the device as a PAYLOAD item.** Operator (2026-09-01): "Sh2 doesn't know current time but it could be provided as a payload item". | Proposed shape (pending the operator's adoption with the rest of entry UX): a `now:<unix-seconds>[,<block-height>]` record written by `me sysw pack` at pack time, a sibling of the `key:`/`hash:` composer-input records (host + device lockstep, sysw spec §3.1 row). SEMANTICS: it is the PACK time, hence a LOWER BOUND on the present — the SH2 has no RTC and `time.Now` is uptime (`gui/gui.go:128,139,442`). Uses: (1) an absolute `after` date or height BELOW the packed value is certainly already past → refusal naming the fact; (2) the echo for an absolute lock reads "at least N days after this payload was packed on <date>", never "in N days"; (3) relative locks need no time. The record is operator-authored and affects ONLY echoes and refusals, never the encoded operand. Without the record the echo falls back to the typed date/height alone. Copy must say "packed", not "now". |
| C25 | **Entry UX ADOPTED as a whole.** Operator: "Yes" (2026-09-01). | Locks: kind picker (relative / absolute) → unit picker (blocks / days; height / date) → digits on a NEW digit-pad widget (digits, backspace, done; not the passphrase keyboard's symbols page) → echo in real units. Days → 512-s units rounded up, echo shows units AND resulting days; refusals name the alternative (blocks > 65535 or days > 388 → "use an absolute date"). Dates `YYYY-MM-DD` → Unix time at 00:00:00 UTC (ceiling 2038-01-19); heights < 500,000,000. Pack-time lower bound from the `now:` record (C24): below-packed values refused, echoes say "at least N days after this payload was packed on <date>". Hashlocks: `hash:<64 hex>` payload record (primary) or typed 64 hex (fallback); on-device preimage derivation DEFERRED. All ranges per C20. |
| C26 | **Build with NO payload loaded is ALLOWED — the result is a keyless TEMPLATE.** Operator (2026-09-01): "Allow build with no payload…that's a template!" Reverses the controller's proposed refusal. | The composer runs with an empty key list: shape → (no seating possible) → the C9 stub-teaching screen (template id + `--policy-id-stub`) → keyless-template consent (D4; addresses stated absent, keyless-on-purpose) → engrave the template. Seating is offered only when the payload holds keys/seeds; "more slots than keys" (partial payload) keeps the stated default: refuse at the seating transition naming both counts, offering Back-to-edit or engrave-as-keyless-template. Stub handling default (§3.5a) stands: payload mk1 stubs ignored at seating, re-minted cards carry the new stub alongside existing ones. |
| C27 | **Recon fan-out approved for items 1 and 2; item 3 DEFERRED.** Operator (2026-09-01): "Recon Agent fan out 1) approved 2) approved 3) defer, we don't want to waste time on this niche area right now". UC ON for this recon step (consent given by this reply). | Dispatched 2026-09-01, opus tier, two disjoint briefs, each persisting its own report: (1) `design/agent-reports/composer-recon-taproot-multisig-origin-convention.md`; (2) `design/agent-reports/composer-recon-same-fingerprint-two-accounts-import.md`. Item 3 (unhardened-child route, one card per cosigner) stays a FOLLOWUPS-class note in §3.4 with no owner. |
| C28 | **Origin convention for seed-derived TAPROOT slots: `m/48'/coin'/account'/3'`; wsh stays `m/48'/coin'/account'/2'`.** Operator: "I accept recommendation" (2026-09-01), on the C27-1 recon (section 3.11). | Account by ordinal among the slots one master fills (C5/C12). No standard exists; `3'` is Coldcard Edge's export (`shared/export.py:414`), the toolkit's `bip48-tr-multi-a`, and structurally disjoint from the wsh key at the same account. Consequence: `ms derive` needs a `bip48-p2tr` template so host-derived `key:` records can match what the device derives (mnemonic-secret `ms-derive-taproot-justifications-stale`, second half). The operator's own `bg002h-tr` (`270028'/.../0'`) scheme is unaffected for the operator's wallets; the composer's DEFAULT is `3'`. |
| C29 | **Same seed twice INSIDE one spending path: WARNING, never refusal; across paths: informational line at most.** Controller's default from the C27-2 recon (section 3.11), standing unless the operator objects (presented 2026-09-01 with the origin ruling). | Copy: "Slots @0 and @2 are the same seed. This path's 2-of-3 can be satisfied by one person. Liana will refuse it." Grounds: Sparrow accepts the shape silently, Nunchuk's signing-progress view collapses it, Liana refuses it (`DuplicateOriginSamePath`), Core accepts. Origin/wrapper mismatch on a card (e.g. a `.../2'` key under tr): DOCUMENTATION only; nothing measured refuses or warns on any origin. |

## 3. Measured this session (the facts the rulings rest on)

### 3.1 The device is closer than the staged plan says

| capability | state | evidence |
| --- | --- | --- |
| Consume any host-built tr/wsh md1: structural summary, receive+change addresses, named id kind | shipped | `gui/wallet_policy.go:35` |
| Serialize an ARBITRARY tree to md1 | exists, all 7 body kinds | `md/encode.go:159` `writeNode` (keyArg, children, variable, multiKeys, tr, timelock, empty); only `EncodeSingleSig`/`EncodeMultisig` are public — no tree BUILDER API |
| Emit Script for fragments | whole set EXCEPT `andor` | `md/script_emit.go` cases: pk_k, c:, v:, and_v, or_i, and_b, or_b, or_c, or_d, thresh, s:, a:, d:, n:, j:, true/false, older, after, sha256/hash256, ripemd160/hash160, multi/sortedmulti, multi_a/sortedmulti_a. `grep -c tagAndOr md/script_emit.go` = **0**; `andor` IS decoded (`md/md.go:362`) and summarised (`md/policy_shape.go:235`). |
| Tap leaves | reuse the fragment emitter | `md/tapleaves.go:188` `EmitTapLeavesChunks` (F-214 closed 2026-08-21) |
| Seat mk1 cards into a keyless template | shipped, declaration match, all-or-nothing | `gui/key_card_seating.go:53` `seatKeyCards` (F-216 core `2f3d140`, wired `a18d19e`) |
| Device address derivation of use-sites | `<a;a+1>/*` and bare `*` only | `gui/md1_expand.go:149` `useSiteToChildren` |
| On-device authoring today | `sortedmulti` k-of-n, n 2..5, wsh / sh(wsh) / sh; seed-centric | `gui/multisig_build.go:48`, `:909` |

### 3.2 The toolkit descriptor-builder (the "Chinese menu" recon) — what it is and how it aged

`mnemonic build-descriptor` (toolkit v0.50-v0.52, June 2026): 17-kind fragment IR
(`crates/mnemonic-toolkit/src/descriptor_builder/schema.rs`), five archetypes
(`archetype.rs:456-532`), four-step gate, goldens in
`crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/`. **wsh only** — the
tr seam was deferred and never built. Never aimed at the device (no mention in
any FOLLOWUPS of mnemonic-engrave or descriptor-mnemonic).

| archetype | wsh lowering | derives on SH2 today |
| --- | --- | --- |
| simple-timelocked-inheritance | `or_d(pk(P), and_v(v:pkh(H), older(N)))` | yes |
| kofn-recovery | `or_d(multi(k,…), and_v(v:pk(R), older(N)))` | yes |
| tiered-recovery | `or_i(sortedmulti(k1,…), and_v(v:older(N), thresh(k2, pk, s:pk…)))` | yes |
| hashlock-gated | `andor(pk(A), sha256(H), and_v(v:pk(B), older(N)))` | **no** (`andor`) |
| decaying-multisig | nested `andor(multi, older, …)` | **no** (`andor`) |

Why the idea still holds: the fragment chosen per spend path (`or_d` vs `or_i` vs
`andor`) changes the script and the addresses, so it is a wallet-defining
decision that must be ONE pinned function shared by Rust and the device.
In tr the choice disappears: each path is a leaf; only tree shape remains.

### 3.3 BIP-388 on repeated keys (fetched from bitcoin/bips master 2026-09-01)

- l.193: "The public keys obtained by deserializing elements of the key
  information vector must be pairwise distinct" (footnote: miniscript pubkey-reuse
  insecurity).
- l.195: "If two KEY are KP/<M;N>/* and KP/<P;Q>/* for the same key placeholder
  KP, then the sets {M, N} and {P, Q} must be disjoint."
- l.308-309 forbidden examples: `sh(multi(1,@0/**,@0/**))`,
  `sh(multi(1,@0/<0;1>/*,@0/<1;2>/*))`.
- l.70: distinct public keys "can be guaranteed by using distinct hardened
  derivation paths".

Wrapper matters only at the SCRIPT layer: in wsh a repeated pubkey in one script
is witness-malleable and rust-miniscript refuses `RepeatedPubkeys`; in tr each
leaf is its own script, a tapscript signature commits to the leaf hash, and
rust-miniscript's `Tr::sanity_check` is per leaf (`rust-miniscript-fork
src/descriptor/tr/mod.rs:143-148`, no cross-leaf check). BIP-388 sits above both
and is wrapper-agnostic, so the composer never sees a split.

### 3.4 md admission, measured on md 0.14.0 built from `2ca75116`

| template | result |
| --- | --- |
| `@0/<0;1>/*` and `@0/<2;3>/*` (BIP-legal, one placeholder) | refused — md1 carries ONE use-site path per key slot (`keyArgBody{index}`; per-slot overrides `md/expand.go:168`); F-417 keeps it so; refusal names `--as descriptor` (`crates/md-cli/src/parse/reuse.rs:314`) |
| `@0/<0;1>/*` twice (BIP-forbidden) | refused since 2026-08-30 (P2/P3 of the mdcli mini-cycle; `md-repeated-placeholder-inverts-bip388` CLOSED) |
| same xpub at two placeholders, different origins | refused: "@0 and @1 carry the same key at the same use-site" |
| same fingerprint, two hardened accounts, two xpubs (wsh and tr) | **encodes; derives** |
| second slot = unhardened child of the account xpub (real child via `mk derive --path m/2`, depth 5) | **refused**: "expected an account-level xpub at depth 3 or 4 … got 5" — CLI admission at `crates/md-cli/src/parse/keys.rs:130`, no bypass; the wire carries 65 bytes and no depth |

Worked example (keys from `design/journeys/inputs-walletpolicy/`, all one master
`73c5da0a` at accounts 0..3; B/C fingerprints below are declared placeholders):

```
tr(NUMS,{multi_a(2,@0/48'/0'/1'/2'/<0;1>/*,@1/48'/0'/0'/2'/<0;1>/*,@2/48'/0'/3'/2'/<0;1>/*),
         and_v(v:pk(@3/48'/0'/2'/2'/<0;1>/*),older(26280))})
--key @0=key1 @1=key0 @2=key3 @3=key2  --fingerprint @0=73c5da0a @1=1b2c3d4e @2=5f6a7b8c @3=73c5da0a
→ 8 md1 chunks, chunk-set-id 0x9c09c, descriptor checksum #72a8pans
→ receive[0] bc1pweuk3648pdpwpvwng6j96tqe6r9wygrnpz5n03vy9hnd6q3rhzfqwnzt2k
```

Cost the rulings accept: a FOREIGN cosigner in two paths hands over two cards
(hardened accounts cannot be derived from an xpub); the operator's own second
slot is minted on-device from the seed, as Multisig Build's S5 already does.

### 3.5 Two findings from walking the C8 workflow (2026-09-01)

**(a) mk1 cards DECLARE at least one wallet, and the policy does not exist yet.**
Normative, not tooling: `SPEC_mk_v0_1.md` §3.2 `stub_count: 1 B; MUST be >= 1`;
§3.3 a stub is the first 4 bytes of the form-aware wallet identity (WalletPolicyId
for a keyed md1, WalletDescriptorTemplateId for a keyless template) and is "a
human-indexing aid, not a cryptographic primitive" — the real check is recomputing
the identity from the assembled descriptor at recovery. So "policy-bound" means
DECLARES, not cryptographically binds. **Already questioned for exactly this use:**
mnemonic-key `FOLLOWUPS.md` `mstar-prepolicy-key-backup` (surfaced 2026-06-20,
open, cross-repo, a BOUND PAIR with the toolkit entry of the same name): decision
needed between (a) an UNBOUND mk1 (no stubs — a wire change to mk-codec) and (b)
seed-only as the pre-wallet backup; analysis leans (a). The composer is that item's
second customer; the `key:` record of §3.6 is a third spelling of the same
artifact (policy-agnostic fp+origin+xpub) that needs no mk wire change.
Demonstrated 2026-09-01: for the §3.4 wallet, WalletPolicyId
`18d7ae384854be73c83b5952d6381d1a` → stub `18d7ae38`; `mk encode … --from-md1 <8
chunks>` and `mk encode … --policy-id-stub 18d7ae38` mint byte-identical cards;
`mk decode` shows `policy_id_stubs: 18d7ae38`. `mk encode`
REQUIRES `--policy-id-stub` or `--from-md1`; `mk.Card.Stubs` is `len >= 1`. A key
packed into the payload before composition carries a stub for some OTHER wallet
or a placeholder. Precedent: Multisig Build ignores cosigner-card stubs at gather
(the policy is assembled afterwards) and binds stubs on the OUTPUT cards
(`gui/multisig_build.go:464`). Controller's default, pending ruling: the composer
ignores payload-card stubs at seating; after seating it RE-MINTS each seated key
as a fresh mk1 carrying the new policy's stub via Go `mk.Encode`
(`mk/encode.go:39`, deterministic), keeping the card's existing stubs — stubs are
repeatable, so a key card lists every wallet it belongs to.

**(b) The shipped seating code forbids operator assignment — and the reason does
not transfer.** `gui/key_card_seating.go:23-31`: "GATHER ORDER IS NEVER AN INPUT,
and the operator is never asked to assign a card to a slot. Both were rejected
for the same reason: they are silent when wrong." That rule seats a template that
ALREADY DECLARES its origins, where an operator's assignment is a second source
that can silently disagree with the truth. A composed template has no
declarations: the operator's choice IS the truth. The residual hazard is a mistap,
which no derivation can detect; mitigations are a mapping-review screen before
consent (per slot: fingerprint + origin), Back preserving assignments, and the
consent screen's per-key lines. Record this so a reviewer does not read C8 as a
contradiction of F-216.

### Journey walk of the C8 workflow — divergences (controller's proposed class)

| step | what else might they do | class / proposal |
| --- | --- | --- |
| pack keys on host | payload also holds a seed record | not our concern this cycle (seed-derived slots deferred) |
| pack keys | two keys share a fingerprint (two accounts of one master) | default: pick-list labels show fingerprint AND origin path |
| boot | operator SKIPs the boot load offer, then chooses Build | refusal naming the route: "No payload loaded. Load it from the carousel" (F-152's fix would make it a default later) |
| author template | more slots than payload keys (n > m) | refusal at the transition into seating, naming both counts; offer Back-to-edit or engrave as a keyless template (D4) |
| seating | key list exhausted / a slot left unseated | refusal: all-or-nothing seating (a partial keyed md1 does not exist); keyless template is the only partial form |
| seating | picks the wrong key for a slot | warning surface: mapping review before consent; Back keeps choices |
| seating | a card's origin script type disagrees with the wrapper (BIP-48 `/2'` key in a tr policy) | documentation / warning — recon item (no standard for tr multisig origins) |
| consent | compares the shown id with a coordinator | default: id kind is named (D2+D4, already shipped) |

### 3.6 Plain-text keys as a payload source (operator, 2026-09-01)

Operator, verbatim: "Payload must contain keys, which I suppose shouldn't be
limited to mk1 strings as input…plain test would be a source of keys too."

Measured on `me` 0.7.0 (`me sysw pack --no-passphrase --in FILE`):

| line | today |
| --- | --- |
| `[73c5da0a/48'/0'/0'/2']xpub…` | REFUSED: "is a single extended key. `me` can infer a whole wallet from one only when its origin is m/44h/0h/0h (-> pkh), m/84h/0h/0h (-> wpkh) or m/49…" |
| bare `xpub…` | ACCEPTED — as a `pkh(xpub…)` single-sig WALLET (`Class::Descriptor`), not as a key |

So a plain-text key needs its own record class, or a bare xpub silently becomes a
wallet. Host classes today (`crates/me-cli/src/sysw/record.rs:45`): Mnemonic,
Codex32Secret, Passphrase, FreeText, Descriptor, MdMk, Mt, Tx, Address, Unknown.
Device classes (`seal/record.go:103`): Unknown, DebugCommand, Mnemonic,
Descriptor, Codex32Secret, MDMK, Address. Adding `Key` is a lockstep change on
both sides plus a sysw spec §3.1 row and admission rule.

Controller's proposal, pending ruling (C9): a **prefixed** record, e.g.
`key:[fp/path]xpub`, in BIP-380 key-origin notation — the same line format
`mk encode --keys FILE` already reads, so host tooling has a parser and the
device has `bip380.ParseKey` (`bip380/bip380.go:366`). **Origin REQUIRED**: BIP-388
makes key origin optional per KEY_INFO (l.180-182) but an md1 slot carries a path,
and F-166 (pathless origin in the Go decoder) is still open; a bare xpub is refused
naming the fix. Both sources — key records and mk1 cards — normalise to
(fingerprint, origin path, xpub) and feed ONE pick list; mk1 stubs are ignored at
seating (§3.5a).

### 3.7 `md compile` is a validity oracle, not a byte oracle (measured 2026-09-01, md 0.14.0 with `cli-compiler`)

Five path-list policies compiled in both contexts:

| policy (Concrete) | segwitv0 | tap |
| --- | --- | --- |
| 2-of-3 now, or @3 after 26280 | `andor(pk(@3),older(26280),multi(2,@0,@1,@2))` — `andor`, recovery FIRST | `tr(NUMS,{multi_a(2,@0,@1,@2),and_v(v:pk(@3),older(26280))})` |
| @0 now, or @1 after 65535 | `or_d(pk(@0),and_v(v:pk(@1),older(65535)))` — `pk` heir | `tr(@0,and_v(v:pk(@1),older(65535)))` — single unlocked key EXTRACTED as internal key |
| 2-of-3 AND sha256 AND older(1000) | `and_v(v:and_v(v:multi(...),sha256(H)),older(1000))` — keys, hash, lock | `and_v(v:and_v(v:sha256(H),multi_a(...)),older(1000))` — HASH, keys, lock |
| @0 now, or @1 after older(100), or @2 after after(1e6) | `or_d(pk(@0),or_i(and_v(v:pkh(@1),older(100)),and_v(v:pkh(@2),after(1000000))))` — `pkh` here | `tr(@0,{and_v(v:pk(@2),after(1000000)),and_v(v:pk(@1),older(100))})` — leaves REORDERED |
| 2-of-3 now, or @3 older, or @4 after | `or_d(multi(2,...),or_i(and_v(v:pkh(@3),…),and_v(v:pkh(@4),…)))` | `tr(NUMS,{{and_v(v:pk(@4),…),and_v(v:pk(@3),…)},multi_a(2,...)})` — primary path at depth 1 RIGHT |

So the compiler chooses `andor` (which the device cannot emit), flips `pk`/`pkh` by
cost, orders conjuncts differently per context, and reorders taproot leaves by
its own weights. It cannot be the normative reference for the lowering; it IS a
sound cross-check of validity and of MEANING: both our lowering and the
compiler's output must parse in-context, pass `sanity_check` (modulo the
EXPERIMENTAL allowances), and `lift()` to the same semantic policy.

**Correction to C2's consequence.** The toolkit's five archetype goldens are NOT
byte-identical to any uniform rule set: inheritance spells the primary `pk` and
the heir `pkh`; kofn-recovery spells its locked key `pk`; tiered-recovery uses
`or_i` where `or_d` is available and `thresh(2,pk,s:pk,s:pk)` where `sortedmulti`
would do; two use `andor`. They carry over as PRESETS (the same spend
conditions) and as validity/lift vectors, not as byte goldens. The composer's
byte goldens are new, Rust-first, in md-codec's corpus.

### 3.8 Sorted key sets nest nowhere (measured 2026-09-01, md 0.14.0) — corrects the proposed key-set rule

| template | md encode |
| --- | --- |
| `wsh(or_d(sortedmulti(2,…),and_v(v:pkh(@3),older(100))))` | REFUSED: "sortedmulti() is valid only as the sole child of sh() or wsh() (BIP-388 §Descriptor Templates, BIP-383); it cannot be nested inside a miniscript fragment" |
| `wsh(or_i(and_v(v:sortedmulti(2,…),older(100)),pkh(@2)))` | REFUSED, same rule |
| `wsh(or_d(multi(2,…),and_v(v:pkh(@3),older(100))))` | encodes |
| `tr(NUMS,{sortedmulti_a(2,…),and_v(v:pk(@3),older(100))})` | encodes — sorted as a WHOLE leaf |
| `tr(NUMS,{and_v(v:sortedmulti_a(2,…),older(100)),pk(@3)})` | REFUSED: "sortedmulti_a() is valid only as a taproot leaf (BIP-386/387); it cannot be nested inside a miniscript fragment" |
| `tr(NUMS,{and_v(v:multi_a(2,…),older(100)),pk(@3)})` | encodes |

So the proposed rule "sorted by default" holds ONLY for an UNLOCKED, UNHASHED
multi-key path (a whole script in single-path wsh, a whole leaf in tr). Every
locked or hashed multi-key path is necessarily `multi`/`multi_a`, and the C16
EXPERIMENTAL warning for "unsorted" applies only where sorted was legal and the
operator declined it. Template recovery of unsorted paths needs no permutation
search when slots declare fingerprints/origins (seating by declaration, F-216),
which the composer's output does (C9/D4). Sent to the fable review as a settled
fact (2026-09-01).

### 3.9 How the NUMS internal key is represented — survey (2026-09-01, sources fetched today)

| layer | rule / practice | source |
| --- | --- | --- |
| consensus / BIP-341 | any point with unknown dlog; example `H = lift_x(0x50929b74…e803ac0)`; recommends `H + rG` with fresh `r` "in order to avoid leaking the information that key path spending is not possible" | bip-0341.mediawiki l.157 |
| descriptors / BIP-386, 387, Core | raw x-only hex accepted as the `tr()` internal key; BIP-387's own examples use `tr(50929b74…,…)` | bip-0387.mediawiki l.69-77; Core `descriptor.cpp` |
| wallet policies / BIP-388 | internal key MUST be a placeholder `@i` backed by an xpub; a non-KP key is an invalid policy. No unspendable form is defined. | bip-0388.mediawiki l.139, 150-153, 310 (`grep -i unspendable` → none) |
| proposed standard | `unspendable()` key expression: xpub with pubkey `H`, chaincode = SHA256 of the DEDUPLICATED, SORTED, concatenated compressed pubkeys of all xpubs in the descriptor; always followed by `/NUM/…/*` | bitcoin/bips PR #1746 (draft `bip-xxxx.mediawiki` l.34-47, 95), opened 2025-01-17, **CLOSED UNMERGED 2025-09-17**: author "I am no longer working on this BIP draft". Grew out of delvingbitcoin thread 304 (s4, post #21 Liana variant = left-to-right order, posts #25-30 = sort+dedupe to fix `sortedmulti_a` order dependence) |
| Liana | emits an unspendable xpub: pubkey `H`, chaincode = sha256 of the leaf xpubs' pubkeys in **left-to-right (DFS) order, not sorted, not deduplicated**; `/<0;1>/*`. Import REQUIRES an xpub internal key (raw hex → `IncompatibleDesc`) | `liana/src/descriptors/analysis.rs:404-445, 596-599` |
| Nunchuk (libnunchuk) | EMITS the raw `H_POINT` for `WalletTemplate::DISABLE_KEY_PATH` (`tr(H_POINT,…)`); IMPORT accepts either the raw `H_POINT` or ANY xpub whose pubkey is `02‖H` (`IsUnspendableXpub`, chaincode ignored); also ships `GetUnspendableXpub(signers)` implementing the PR-#1746 recipe (sort + dedupe + sha256) with no in-library caller | `libnunchuk src/descriptor.cpp:38,193,299,459,487,517,529,724-753`; `descriptor.h:28,93` |
| Ledger | documents `tr(KP)`/`tr(KP,TREE)` only; no unspendable rule in `doc/wallet.md` | LedgerHQ/app-bitcoin-new `doc/wallet.md` l.30-38 |
| md-codec (ours) | the WIRE carries `is_nums: bool` on the tr body — an abstract flag, not key bytes; `render.rs` and `to_miniscript.rs` SPELL it as the raw `H` hex | `crates/md-codec/src/tree.rs:51`, `render.rs:87-88`, `to_miniscript.rs:319-343` |

Consequences: (1) there is NO standard spelling for a BIP-388 unspendable
internal key — one proposed BIP, abandoned; two shipping conventions (Liana's
ordered hash, the PR's sorted hash) that produce DIFFERENT xpubs for the same
wallet; Nunchuk emits raw H and imports either. (2) Because the derived internal
key differs between raw `H` and any `H`-xpub form (the xpub's per-index child is
`H + tweak`), changing md's spelling changes every NUMS wallet's ADDRESSES: it is
a new internal-key kind on the wire (a second flag value or TLV), not a re-render,
and existing NUMS plates keep meaning raw `H`. (3) Raw `H` is what Core and
Nunchuk accept and what both reference wallets already engrave; only Liana and
BIP-388-strict registration refuse it.
### 3.10 LOWERING RULES — RULED 2026-09-01 (C1/I1-I4/M1-M3 folded; C22 withdrawn by C23; supersedes the §4 item-4 draft)

| rule | wsh | tr |
| --- | --- | --- |
| precondition | at least one keyed path (I4); a lock-only path is refused as anyone-can-spend (I3) | same; keyless paths refused entirely (I2) |
| paths combine | listed order, recursive, last path stands alone: `or_d(P, R)` iff `P` is a bare unlocked, unhashed `multi(k,…)` with n ≥ 2; otherwise `or_i(P, R)` — a bare single key is `or_i(pkh(K), R)` (I1/C21; C22 withdrawn by C23). Never `andor`, never `thresh` over paths | one leaf per path on a right spine in listed order `{P1,{P2,{P3,P4}}}`; path k at depth min(k, n−1); text order at a node does not change the root (BIP-341 l.74), the spine is the canonical TEXT form |
| inside a path | `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`, dropping absent parts; keys, hash, lock last (byte-optimal, N1) | same |
| key set | unlocked single-path: `sortedmulti`; locked/hashed multi-key: `multi`; one key: `pkh` (C17) | unlocked whole leaf: `sortedmulti_a`; locked/hashed: `multi_a`; one key: `pk` (C17) |
| lock values (M2, C20) | `older`: 1..=65535 blocks, or 0x400000+u for u in 1..=65535 (512-s units); `after`: 1..=499,999,999 height, 500,000,000..=2,147,483,647 time | same |
| internal key (M1, C20) | n/a | the FIRST-LISTED unlocked, unhashed one-key path, which is then not a leaf; otherwise the NUMS point spelled raw `H` (C18) |
| placeholder numbering (C1) | `@i` by first appearance in the EMITTED text; slot labels shown to the operator are those indices | same; an extracted internal key is `@0` |
| keyless path (EXPERIMENTAL) | `and_v(v:sha256(H), LOCK)` or `sha256(H)` alone | refused |
| documented, no rule | `or_d` over a bare `multi(k)` head costs k+1 empty pushes on non-head spends: −2 WU per head spend, +k WU per other spend, no key exposure (M3); conjunct order is byte-identical while LOCK is last (N1); `pkh` at an `or_i` head costs 26 WU per P1 spend vs `or_d(pk)` (C21) | leaf text order irrelevant to the address (N2) |

Cross-check contract for the Rust vectors: every composable list lowers to text that
parses in its context, passes `sanity_check` (modulo the EXPERIMENTAL allowance for
keyless wsh paths via `ExtParams::top_unsafe()`), `lift()`s to the same semantic
policy as `md compile` of the equivalent Concrete policy, and re-encodes through
`md encode` → `md decode` byte-identically (C1).

### 3.11 Recon results (C27), controller-verified 2026-09-01

**(1) Taproot multisig origin convention** — report `agent-reports/composer-recon-taproot-multisig-origin-convention.md` (`9f55eb6`). No standard exists: BIP-48 registers `1'`/`2'` only; BIP-86 is single-key; bips PR #1473 (`3'` for taproot) closed unmerged 2024-05-14. Field split: Coldcard Edge exports `m/48h/…/3h` (`shared/export.py:414`, verified); Nunchuk derives `m/87h/coin/acct` (`libnunchuk src/utils/bip32.hpp:104-105`, verified); Liana hard-codes `48'/…/2'`; Sparrow has no taproot multisig. Nothing refuses any candidate; md round-trips all four (verified). Recommendation: `m/48'/coin'/account'/3'`; second choice BIP-87. Side finding filed: mnemonic-secret `ms-derive-taproot-justifications-stale`. **Ruling PENDING.**

**(2) Same fingerprint at two accounts** — reports `composer-recon-same-fingerprint-two-accounts-import.md` + `-core-` + `-sparrow-` (`f7f0b27`). Controller re-ran: Core v25 (peerless mainnet, watch-only) imports W2 and W3 single-chain with `success: true` (checksums `g7tqcjhe`, `y6wdsq0v`) and refuses the same-KEY-twice control (miniscript sanity); Liana `LianaDescriptor::from_str`: W2 ACCEPT with `73c5da0a` in both paths, W1/W3 REFUSE (shape / same fp twice in ONE path = `DuplicateOriginSamePath`); Sparrow drongo HEAD via compiled Java: two keystores with one fingerprint accepted as VALID with no warning, `tr(NUMS,{multi_a…})` refused ("Cannot determine the multisig threshold"), miniscript wsh mis-parsed as a flat `sortedmulti` and failing at address derivation. Nobody dedupes on fingerprint. Beyond the brief: Nunchuk's per-tx `get_signers()` is a fingerprint-keyed `std::map<std::string,bool>` (`nunchuk.h:1174`, verified type) so two keys of one seed collapse to one signing-progress row; md's depth-0 xpub re-serialisation draws no objection from Core/Liana/Sparrow but is a live UNVERIFIED Ledger risk (`register_wallet.c` whole-xpub memcmp) — filed descriptor-mnemonic `md-descriptor-depth0-xpub-ledger-registration`. **Consequence for the composer (proposed default): WARN, never refuse, when one seed/fingerprint fills two slots INSIDE ONE spending path** ("Slots @0 and @2 are the same seed. This path's 2-of-3 can be satisfied by one person. Liana will refuse it."); the cross-path case (C5's normal shape) gets at most an informational line.

## 4. Open questions, in the order they will be asked

1. **Seating design.** RULED: C8 (payload pick list, slot-directed), C12 (seeds
   as a source, per-slot accounts), C13 (Full vs Watch-only; secret plate forms),
   C10 (engraved form is the operator's choice), C14 (no Sealed-Payload memory
   treatment). Remaining details for the spec: mk1 stub handling at seating
   (§3.5a default), the two divergence rulings (n > m; Build with no payload
   loaded), wiring the ms1 legs into the seating source picker, and the
   per-plate ceilings for a concrete descriptor (C10).
2. **Rust home of the lowering.** md-codec / md-cli (the device's Go `md` pins
   there) vs the toolkit builder (where the archetypes live). One normative
   function; two candidate repos.
3. **Grammar bounds.** Per path: k-of-n over FRESH slots; at most one timelock
   (`older` | `after`) and at most one `sha256`; keyless (hashlock-only) paths —
   allow under EXPERIMENTAL (RCW tier 4, bearer) or refuse? Wrappers: tr, wsh;
   sh(wsh)/sh only for a single bare `sortedmulti` path (the Multisig migration).
   tr key-path: a single slot spends alone, else NUMS.
4. **Lowering rules.** RULED — see §3.10 (two independent reviews, C19/C20/C21).
   Remaining implementation consequences: a `pk_h` emitter arm on the device (C17);
   `andor` is out of the grammar so no emitter arm is needed.
5. **Entry UX.** RULED — C24/C25. New widget: digit pad. New payload classes:
   `key:`, `hash:`, `now:` (host + device lockstep, sysw spec rows).
6. **Recon items:** (1) RULED C28; (2) RULED C29 (default); both verified in
   section 3.11; (3) unhardened-child route: DEFERRED by the operator.
7. **Journey.** A composer journey, EXECUTED on the emulator, is a plan gate.

## 5. Process

Risk-set on three counts (normative codec behaviour, funds/keys/addresses,
spans repos). R0 to 0C/0I before code; agents persist their own reports to
`design/agent-reports/`; Rust-first for the lowering with vectors, then the Go
port. UC: propose at the recon step (external BIP facts); not yet asked.
Deprecation of Multisig Build (C7) lands as a FOLLOWUPS entry with the migration
gate above, not as code in this cycle.
