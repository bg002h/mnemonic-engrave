# Lens 4 — which shape should this project recommend: `wsh(or_i(...))` or `tr(NUMS,{...})`?

Reviewer: independent lens agent, 2026-08-21.
Scope: the COMPARISON and recommendation only. Restore inventory, labour ledger and
document wording belong to other lenses and are touched only where they change the
comparison.

Grounding: `transcript_pathological.txt`, `transcript_tr_pathological.txt`,
`out/pathological/journey_pathological.html`, `out/tr-pathological/tr-pathological-journey.html`,
`inputs-pathological/wallet-policy.txt`, `inputs-pathological/wallet-policy-tr.txt`,
`design/FOLLOWUPS.md` (F-219/F-220/F-221), `mnemonic-toolkit/CHANGELOG.md` (NUMS provenance),
plus five read-only runs of `md 0.13.0` (version confirmed at run time; commands quoted below).

**New measurements made for this comparison** (none of these were in the settled facts):

1. `md encode --group-size 0 --force-chunked` over the **wsh** pathological policy with the
   eleven true origins written **inline** (`@0/48'/0'/0'/2'/<0;1>/*`, …) succeeds: **4 chunks**
   (chunk-set-id `0x66602`), and `md inspect` shows all eleven per-slot origins —
   `@0: m/48'/0'/0'/2'` … `@10: m/48'/0'/2'/2'` — with `wallet-descriptor-template-id`
   **identical** to the journey's flattened card (`5b48af35d4321a3ac18b43045e2523cc`).
2. The **keyed** wsh set (same 11 xpubs/fingerprints as the tr journey) encodes to
   **24 chunks** (chunk-set-id `0x504e1`) — exactly the tr keyed set's 24.
3. `md encode --path bip48` over the wsh template **with divergent inline origins** exits 0,
   silently discarding all eleven and reproducing the journey's flattened card byte-identically
   (chunk-set-id `0x829c6`). No warning on stderr.
4. The **same command over the tr policy** also exits 0 and silently flattens
   (chunk-set-id `0x1dd83`, 3 chunks; stderr carries only the keyless-template note). The tr
   journey's sentence that this "is a card `md encode` now refuses outright" is **not what
   md 0.13.0 does for the keyless form** — F-221 says the refusal fires only on the *keyed*
   form. The flatten footgun is symmetric across wrappers. (Wording is another lens; the
   *measured symmetry* is load-bearing here.)
5. `50929b74…803ac0` is the **BIP-341 reference NUMS H-point**, chosen deliberately —
   `mnemonic-toolkit/CHANGELOG.md:1005` and `:4580` (`--taproot-internal-key nums`).

---

## 1. COMPARISON TABLE

Axes are backup-and-restore for THIS wallet: a degrading 4-tier vault, 11 keys, 3 masters,
4 divergent account origins, two hashlocks, all four timelock kinds.

| Axis | `wsh(or_i(...))` | `tr(NUMS,{...})` | Verdict |
| --- | --- | --- | --- |
| **Plates: descriptor card, like-for-like (true origins inline, keyless)** | **4 chunks** (measured, `0x66602`) | **4 chunks** (transcript, `0x3d896`) | Parity |
| **Plates: descriptor card as the journeys actually built them** | 3 chunks, origins flattened to one (`0x829c6`) | 4 chunks, all 11 true origins | tr — but only because wsh took `--path` |
| **Plates: keyed full-policy set** | **24 chunks** (measured, `0x504e1`) | **24 chunks** (transcript, `0x23401`) | Parity |
| **Plates: mk1 key cards** | 30 chunks for 11 cards (transcript) | mk1 is wrapper-agnostic (xpub + origin + stub); same 30 | Parity |
| **Plates: seeds** | 3, typed on device | 3, typed on device | Parity |
| **What a restorer needs** | seeds + per-slot origins **with fingerprints** + preimage for tiers 1–2 | identical list, plus the NUMS point (on the card) | Parity. Both keyless templates carry bare paths, **no master fingerprints** (both inspects); with 3 masters and order-significant `multi`/`multi_a`, template-only restore leaves master↔slot assignment ambiguous either way. Fingerprints ride the mk1 cards or the keyed set — in both shapes. Preimage: zero backup strings carry it in either (F-132). |
| **Privacy before spend** | P2WSH output — visibly a script wallet | P2TR output — indistinguishable from single-sig taproot | **tr, intrinsic** |
| **Privacy at spend** | **Entire script published on first spend from any tier**: all 11 pubkeys, all thresholds, all four timelocks, the hash — the complete weakening schedule of every still-locked UTXO | Only the exercised leaf + a Merkle path; unspent tiers stay hidden (settled). Costs: the standard NUMS in the control block reveals "no key path" and fingerprints the software; a hashlock spend publishes the preimage (both shapes) | **tr, intrinsic, decisive for this wallet class** |
| **Fee cost at spend** (script-size arithmetic, estimate ±15%, not run) | Witness ≈ 500-byte script + sigs: tier 1 ≈ 190 vB witness, tier 4 ≈ 145 vB | Tier-1 leaf ≈ 111 vB witness, tier-4 leaf ≈ 77 vB (depth-3 control block = 129 B) | **tr, intrinsic** — ~40% less at tier 1, ~half at tier 4, the tiers most likely spent under duress |
| **Host tool support today** | encode/decode/inspect/verify/address all shown (transcript) | same (transcript) | Parity |
| **Device support today** | Engrave leg proven for this wallet (emulator, seed plate). Wallet-Policy gather/consent for THIS wsh wallet **not demonstrated** in any journey (the wallet-policy journey walked a 5-of-12 `wsh(multi)`) — not proven absent, but not proven | **End-to-end proven**: 24 chunks gathered, 1 card, 4 consent pages, wallet id `590f3abc…` and 4 addresses matched, with a negative control — after the F-214 fix, 3 commits old | tr has the stronger evidence; wsh has the older, better-worn engrave path. Both rest on the same fresh emitter era |
| **Failure modes** | `--path` silently flattens divergent origins (measured, exit 0); F-220 (canonical wrapper never demands an origin); F-221 (contradiction check blind to keyless form); F-219 (decode stdout lossy) | `--path` silently flattens here too (measured, exit 0); non-canonical wrapper at least *warns* when no origin is given (F-129 via F-220); F-219 identical | Slight edge tr: the canonical-default rule means wsh fails silent where tr fails noisy. Same defect class, both fixable |
| **External ecosystem (background knowledge, not artifact-grounded)** | wsh miniscript: years of coordinator/descriptor support (Core, Liana-class tools) | tapscript miniscript leaves + `multi_a`: newer, thinner outside this constellation | **wsh** — the one axis it wins |

## 2. THE RECOMMENDATION

**Recommend `tr(NUMS,{...})` for a user backing up a degrading multi-tier vault — provided
the backup carries per-key origins with fingerprints (the keyed 24-chunk set, or template +
11 mk1 cards), which is the same condition either shape needs.**

Why, ranked:

1. **Unspent-tier secrecy is intrinsic and is exactly what this wallet class needs.** A
   degrading vault's later tiers exist for duress, death and loss, and sit unspent for
   years. Under `wsh`, the *first* spend from *any* tier publishes the full map — which
   keys, what thresholds, and precisely when each remaining tier of every still-locked
   UTXO becomes easier to take. Under `tr`, a routine tier-1 spend reveals tier 1 and
   nothing else. This is a funds-relevant property of the backup's end state, not taste.
2. **Deep-tier spends are roughly half the witness weight** (estimate above) — and the
   deep tiers are the fee-stressed, duress-shaped ones.
3. **Every reason this project's own artifacts gave to prefer wsh is measured to be
   accidental.** The 3-of-11 restore was `--path`, not the wrapper (the inline-origin wsh
   card carries all eleven, 4 chunks, same template-id). Plate counts are at parity at
   every layer (4v4, 24v24, mk1 identical, 3 seeds each). The device now walks the tr
   wallet end-to-end — currently the *stronger* on-device proof of the two.

**By how much: on backup mechanics, zero — the shapes are at measured parity. The entire
margin is on-chain (privacy of what remains, fees at the tiers that matter), where tr wins
intrinsically and wsh wins nothing.** The recommendation is therefore clear but not
overwhelming, and it rests on one assumption: **the restorer's toolchain understands
tapscript miniscript.** This constellation's does — both transcripts prove it. If the
user's threat model is "heirs restore in 15 years with whatever generic tools exist, no
`md` binary survives," the conservative pick flips (see §4.1).

Improvements that would strengthen the recommended shape (noted, not the answer): engrave
the keyed set or key cards, never template-only; a warn/refuse on `--path` over divergent
origins on the *keyless* form, symmetric across wrappers (the F-220/F-221 ruling);
fingerprints in the keyless template; a per-wallet blinded NUMS (`H + rG`) to remove the
software fingerprint, at the cost of backing up `r`.

## 3. INTRINSIC VS ACCIDENTAL

| Difference | Class | Evidence |
| --- | --- | --- |
| Unspent tiers hidden at script-path spend | **Intrinsic** (BIP-341) | Settled; structure of the control block |
| P2TR outputs uniform before spend; P2WSH visibly script | **Intrinsic** | Address formats in both transcripts (`bc1q…` 62-char vs `bc1p…`) |
| Smaller witnesses, especially deep tiers | **Intrinsic** | Script-size arithmetic (estimate, flagged) |
| Schnorr 64/65 B sigs vs ECDSA ~72 B; `multi_a` vs `multi` | **Intrinsic**, minor | BIP-341/342 |
| "The tr card carries all 11 true origins; the wsh card restores 3 of 11" | **Accidental** — the headline finding of this lens | Measured: inline-origin wsh encode = 4 chunks, all 11 origins, template-id unchanged (`5b48af35…`). The 3/11 card is the `--path` card |
| `--path` silently flattening divergent origins | **Accidental, symmetric** | Measured both wrappers: exit 0, no warning (`0x829c6`, `0x1dd83`) |
| wsh never *demands* an origin (silent), tr warns when none given | **Accidental** (codec canonical-wrapper rule) | F-220 / F-129 |
| Keyless template carries no master fingerprints (either shape) | **Accidental, symmetric** | Both `md inspect` outputs: bare `m/48'/…` paths |
| 3-chunk vs 4-chunk descriptor card | **Accidental** | Like-for-like is 4 v 4 (measured); 3 is the flattened card |
| 24 v 24 keyed, 30 mk1 chunks, 3 seed plates | Parity | Measured / transcripts |
| `mk --from-md1` rejects any chunked set (wire v9) | **Accidental**, shape-neutral | wsh transcript; vendored md-codec 0.34 vs 0.42 |
| Device tapscript emitter is 3 commits old; tr proof exists, this-wsh consent walk not demonstrated | **Accidental** (state of these tools, both directions) | tr journey §"Until three commits ago"; wsh journey device leg |
| External ecosystem maturity favors wsh | **Accidental in principle** (time closes it), real today | Background knowledge, hedged |
| Standard-NUMS software fingerprint at spend | **Accidental to the NUMS choice**, not to taproot | `mnemonic-toolkit/CHANGELOG.md:4580`; blinded NUMS possible |
| Preimage published on first hashlock spend; tier order ≠ reading order | Policy design, **equal both shapes** | F-132 / F-133 |

The accidental column is the actionable one: every backup-side difference between the two
shapes is in it, and every entry in it is fixable by ruling or convergence — none requires
choosing a wrapper.

## 4. WHAT WOULD FLIP IT

1. **The restorer-tooling assumption fails.** If restore must succeed on third-party tools
   only — no constellation binary survives — flip to `wsh` with inline origins: wsh
   miniscript descriptors have years of external support; tapscript `multi_a` leaves are
   newer and thinner outside. This is the single realistic flip, and it is a judgment about
   the user's heirs, not about these artifacts.
2. **The F-214 emitter regresses or the fork drifts.** The tr device proof is one journey
   on a 3-commit-old fix. A regression without a re-run journey returns the on-device
   evidence advantage to the wsh engrave path until re-proven.
3. **Structure disclosure stops mattering.** A vault whose tiers are deliberately public
   (audited corporate policy, court-visible inheritance) forfeits tr's headline advantage;
   the residual fee edge alone would not outweigh ecosystem maturity — wsh wins that wallet.
4. **The wallet class changes.** For a single-master wallet whose slots genuinely share one
   origin, the flattened 3-chunk wsh card is *honest*, one plate cheaper, and the privacy
   argument shrinks with the tier count. This lens's answer is for the degrading multi-tier,
   multi-origin vault specifically.
5. **Directional, not a flip: a real internal key instead of NUMS** (a cooperative key-path
   tier) widens tr's advantage — cheapest possible spend, zero structure revealed. If the
   user can name a cooperative quorum, the answer moves further toward tr, not back.

Not flips: fixing the accidental gaps (inline origins in wsh journeys, keyed-set engraving,
`--path` guard, fingerprints on the template) brings wsh to backup-parity — which is where
this comparison already placed it. Parity on the plates does not touch the intrinsic
on-chain margin, so the recommendation stands on that margin alone.
