# R0 gate, lens 3/4 — external-fact verification

Artifact: `design/SPEC_mt_v0_1.md` at commit `099a516e7b73f4f851afc56a3057bf4f2bfc1330` (534→538 lines).
Scope: mechanical fact-checking only, per dispatch brief. No architecture/threat-model/completeness commentary.

## Verdict

**1 Critical / 1 Important / 1 Minor / 0 Nit**

- **F-1 (Critical)** — §3/§2's choice of UR type `ur:bytes` as the production wire envelope
  contradicts BCR-2020-005 itself, which states the `bytes` type "exists only for testing
  and validation of UR implementations and MUST NOT be used for any other purpose." No
  other registered UR type (BCR-2020-006) covers a finalized/raw signed Bitcoin
  transaction — only `psbt`/`crypto-psbt`, which must be a valid BIP-174 PSBT.
- **F-2 (Important)** — §1a/§3's claim that Sparrow, Keystone, Passport and Specter
  "already read"/"already consume" UR is true only for `ur:psbt`/`crypto-psbt`. No
  positive evidence found that any of the four decode `ur:bytes` as a legitimate
  production payload; one concrete report (a Sparrow GitHub issue) shows a `ur:bytes`
  animated QR from a DIY signer failing to scan into Sparrow at all.
- **F-3 (Minor)** — §6d's "network-adjusted time" is stale terminology for current
  Bitcoin Core, which compares block timestamps to the node's own local clock
  (`NodeClock::now()`), not a peer-derived network-adjusted value (that mechanism was
  removed). The 2-hour bound itself, and every other timestamp claim, is exactly correct.

## Claim-by-claim table

| # | Claim | Authority | Verdict |
|---|---|---|---|
| 1 | BIP-341 `sha_amounts`/`sha_scriptpubkeys` quotes, ANYONECANPAY gating, "lie to offline signing devices" rationale | BIP-341 (bitcoin/bips, `bip-0341.mediawiki`) | **TRUE** |
| 2 | BIP-143: segwit v0 commits to the signed input's *own* amount | BIP-143 (bitcoin/bips, `bip-0143.mediawiki`) | **TRUE** |
| 3 | PSBT `non_witness_utxo` vs `witness_utxo` distinction; wallets hardened toward the former after the segwit fee-lying attack | BIP-174 + secondary historical sources (Unchained, WalletWasabi/BTCPayServer issues) | **TRUE** (history corroborated by secondary sources only, no single primary disclosure found — see Could-not-verify) |
| 4 | `gettxout <txid> <vout> false` semantics: returns `value`+`scriptPubKey`, queries UTXO set (null=spent/nonexistent), `include_mempool` defaults `true`, needs no `-txindex` | live Bitcoin Core v25.0.0 RPC (`bitcoin-cli help gettxout`, `help getrawtransaction`) | **TRUE** |
| 5 | `gettxoutproof`/`verifytxoutproof` semantics, incl. that `verifytxoutproof` checks against the node's own best chain | live Bitcoin Core v25.0.0 RPC help | **TRUE** |
| 6 | Block timestamp rules: (a) nTime > MTP of previous 11 blocks; (b) nTime ≤ network-adjusted time + 2h; (c) nTime not monotonic; (d) MTP is monotonic & consensus-enforced | Bitcoin Core `master`: `src/validation.cpp` (`ContextualCheckBlockHeader`), `src/chain.h` (`GetMedianTimePast`, `nMedianTimeSpan=11`), `src/consensus/consensus.h` (`MAX_FUTURE_BLOCK_TIME=2*60*60`) | **TRUE**, with Minor nuance on "network-adjusted" (F-3) |
| 7 | `target = mantissa × 2^(8×(exponent−3))`; expected hashes `2^256/(target+1)`; and the spec's own numbers for `nBits=17023cc1` (difficulty 125,807,076,547,198; 5.403×10²³ expected hashes) | Direct computation (Python), cross-checked against a live block sharing the same `nBits`/difficulty | **TRUE** — recomputed independently, exact match |
| 8 | Subsidy 3.1250 BTC for the block in question | Live `getblockstats` RPC + halving arithmetic (height in the 840,000–1,050,000 era) | **TRUE** |
| 9 | QR alphanumeric mode: `:` and `/` both present in the 45-char set; 2 chars pack into 11 bits; 11 bits/payload-byte given 2 bytewords-chars/byte; 37.5% expansion | ISO/IEC 18004 alphanumeric table (cross-checked via Wikipedia "QR code" + thonky.com QR tutorial) | **TRUE** |
| 10 | `qrcode` Rust crate "knows [Structured Append] only as a mode indicator with no encoder" | `qrcode` crate v0.14.1 source, read directly from `~/.cargo/registry/src/.../qrcode-0.14.1/src/bits.rs` | **TRUE** |
| 11 | Sparrow, Keystone, Passport, Specter "already read"/"already consume" UR — applied to `ur:bytes` (§3, the engraving envelope) | Wallet docs/blogs, Sparrow's own `hummingbird` (Java UR lib) source, a Sparrow GitHub issue | **FALSE / OVERSTATED as applied to `ur:bytes`** — see F-2 |
| 12 | "UR is BCR-2020-005" and "`ur:bytes` is a registered type" | BCR-2020-005 (`bcr-2020-005-ur.md`) and BCR-2020-006 (`bcr-2020-006-urtypes.md`), both read directly from source | **PARTIALLY TRUE** — see F-1 |
| 13 | `bitcoin` 0.32.101 ships `bitcoinconsensus` feature + `consensus/validation.rs` | — | **SKIPPED** (pre-verified per dispatch brief) |

---

## F-1 — `ur:bytes` is explicitly forbidden for production use by its own defining spec (Critical)

**Spec's words** (§3, lines 89 and 122–123):

> Fragmentation uses **UR (Uniform Resources, BCR-2020-005)**, type `ur:bytes`.
>
> ...Uppercased, `ur:bytes/N-M/…` is fully QR-alphanumeric...

Decision #3 in §1 ("The QR carries the standard form, never a codex32 string (F-234)") and all of §2/§3 rest on `ur:bytes` being a legitimate, standards-compliant, widely-supported envelope for the plate's payload — the spec calls UR "positively good here, not merely available" specifically because it is "already vendored and device-tested in the fork" and "what Sparrow, Keystone, Passport and Specter already read."

**The authority's words**, verbatim, from BCR-2020-005 itself (`https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md`, the "Types" section, fetched and grepped directly — not a summary):

> **⚠️ NOTE:** The only type this document specifies is `bytes` which represents an
> undifferentiated string of bytes of any length. The `bytes` type exists only for
> testing and validation of UR implementations and **MUST NOT be used for any other
> purpose.** It also has no corresponding CBOR tag (described below). Other
> specifications register and document types that specify forms of structured content
> intended to address various application domains.

`MUST NOT` is a normative RFC-2119 keyword in this document (the "Requirements" section three paragraphs above quotes RFC 2119 explicitly).

I then checked whether the companion registry (BCR-2020-006, `bcr-2020-006-urtypes.md`) defines *any* type suited to a finalized, network-serialized Bitcoin transaction — since if one exists, `mt` should presumably use it instead of `bytes`. It does not: the only Bitcoin-transaction-shaped entry is

> `psbt`/`crypto-psbt` — "Partially Signed Bitcoin Transaction (PSBT)" — "the type `psbt`
> contains a single, deterministic length byte string of variable length up to
> 2^32-1 bytes. Semantically, this byte string MUST be a valid Partially Signed Bitcoin
> Transaction encoded in the binary format specified by [BIP174]."

That type is defined to require a *PSBT*, not a finalized/broadcastable transaction — which is exactly what `mt` engraves (§8 refuses anything else). There is no registered UR type in the Blockchain Commons ecosystem for the payload `mt` actually wants to carry.

**Severity reasoning.** This is Critical, not Important: it is a false/materially-overstated fact that a *normative wire-format decision* rests on. §2 says explicitly "It is a real format... which is why it has no bech32 HRP and no BCH checksum" — the entire justification for building `mt-codec` on UR rather than something else cites BCR-2020-005 as the format's pedigree, while the same document forbids the specific use `mt` makes of it.

**What the spec may honestly say instead.** Either (a) note that `mt` is using `ur:bytes` off-label relative to its own defining spec, and argue explicitly why that's acceptable here (e.g., "no compliant alternative type exists for a finalized transaction, so we knowingly violate the letter of BCR-2020-005's test-only restriction"), or (b) reconsider the envelope — e.g. a private/unregistered type name (`ur:mt-tx` or similar) that is honest about not being an existing standard, which is closer to what real UR-based ecosystem tooling would expect for an unrecognized payload anyway. The current text implies compliance and ecosystem support that the primary source does not extend.

## F-2 — Wallet "already reads UR" claim does not establish `ur:bytes` support (Important)

**Spec's words** (§3, lines 98–100 and §1a, lines 65–68):

> It is already vendored and device-tested in the fork (`bc/ur`, `bc/bytewords`,
> `bc/fountain`), and it is what Sparrow, Keystone, Passport and Specter already read.
>
> ...The medium is `ur:psbt`, the same UR machinery §3 specifies, which is what Sparrow,
> Keystone, Passport and Specter already consume as an animated QR.

§1a's claim is scoped to `ur:psbt` and is well supported (see below). §3's claim is *not* scoped — it reads as a claim about the same four wallets reading UR *in general*, immediately following a sentence that names `ur:bytes` as §3's chosen type. A reader following the citation from §3 into §1a would reasonably conclude the same four wallets already read the plate's actual envelope, which is the premise F-234 depends on (a recoverer with no `mt`-aware software can still decode the QR with off-the-shelf wallet software).

**What I found per wallet/type:**

- **`crypto-psbt`/`ur:psbt` support is solidly confirmed for all four.** Sparrow's own `hummingbird` UR library (`sparrowwallet/hummingbird`, `RegistryType.java`) defines `CRYPTO_PSBT → "crypto-psbt"` and `PSBT → "psbt"`; Keystone's own integration guide (`KeystoneHQ/Keystone-developer-hub/Integration_guide.md`) states "We use crypto-psbt to encode the psbt data"; Specter and Passport's documented air-gap workflows are PSBT/animated-QR based throughout.
- **No positive evidence any of the four treat `ur:bytes` as a supported production payload.** Keystone's integration guide, read directly, documents `crypto-account`, `crypto-psbt`, and `bc-ur` (for a multisig config file) — it does not mention `bytes`/`ur:bytes` for transaction data anywhere I found.
- **Negative evidence for Sparrow specifically.** `hummingbird`'s `RegistryType` enum *does* recognize `BYTES → "bytes"` at the transport/CBOR-decoding layer — so the library can parse the UR envelope. But a live Sparrow GitHub issue (`sparrowwallet/sparrow#78`, "QR Scanning Legacy QR code") shows a user whose DIY signing device emits a signed-PSBT payload as `UR:BYTES/1OF4/[HASH]/[URFRAGMENT1]`: *"No matter what I try when scanning the QR when signing a transaction. It does not work."* — while the same user's device scans fine using the older Cobo-Vault legacy encoding. The issue is tagged `enhancement`/`low priority`; nothing in the visible thread confirms `ur:bytes` scan-back was ever fixed. That is a concrete, sourced counter-example to "already read," not merely an absence of evidence.
- **Passport and Specter**: found no source, positive or negative, addressing `ur:bytes` specifically — their public documentation only describes PSBT-based flows.

**Severity reasoning.** Important, not Critical on its own: I cannot prove the claim is flatly false for all four wallets (Sparrow's library does at least parse the `bytes` UR type at the transport layer, and I could not rule out that some newer firmware revision handles it end-to-end). But "already read" is stated as settled fact and is not — it's backed by real evidence only for a *different* UR type than the one §3 chose, plus one concrete negative data point. Combined with F-1 (the same type is explicitly off-label per its own spec), the compounding effect on F-234's premise is what pushes F-1 to Critical; I keep this one at Important because its evidentiary status is "overstated/unverified," not "proven false."

**What the spec may honestly say instead.** Narrow the claim to what's actually shown: "Sparrow, Keystone, Passport and Specter already read `ur:psbt`/`crypto-psbt` for the *presenting* use case (§1a). Whether the same software reads `ur:bytes` for the *engraved* payload (§3) is unconfirmed and should be tested against real wallet software before F-234's recoverability premise is relied on."

## F-3 — "network-adjusted time" is legacy terminology in current Bitcoin Core (Minor)

**Spec's words** (§6d, lines 396–397):

> ...must not exceed network-adjusted time by more than two hours.

**The authority's words.** Current Bitcoin Core (`master`, `src/validation.cpp`, function `ContextualCheckBlockHeader`, fetched and grepped directly from `raw.githubusercontent.com/bitcoin/bitcoin/master/src/validation.cpp`):

```cpp
// Check timestamp
if (block.Time() > NodeClock::now() + std::chrono::seconds{MAX_FUTURE_BLOCK_TIME}) {
    return state.Invalid(BlockValidationResult::BLOCK_TIME_FUTURE, "time-too-new", "block timestamp too far in the future");
}
```
`src/consensus/consensus.h`: `inline constexpr int64_t MAX_FUTURE_BLOCK_TIME = 2 * 60 * 60;` (exactly 2 hours — the number is correct).

`src/util/time.h` defines `NodeClock::now()` as "Return current system time or mocked time" — a local clock read, with no peer/network adjustment. A web search corroborates this is deliberate: a Bitcoin Core PR titled "Nuke adjusted time (attempt 2)" (discussed at `bitcoincore.reviews/28956`) removed the historical peer-time-averaging mechanism that "network-adjusted time" originally referred to (Satoshi-era terminology, still common in secondary literature).

**Severity reasoning.** Minor: the numeric bound (2 hours) is exactly right, and `mt` never implements or relies on the future-time check at all (only MTP, via `SPENDABLE AFTER BLOCK <n>` and the legend's MTP bound) — so this has zero effect on any `mt` behavior. It's purely a terminology precision issue in explanatory prose.

**What the spec may honestly say instead.** "...must not exceed the node's own clock, plus a fixed two-hour allowance" — or keep "network-adjusted time" but note it's the traditional/colloquial name for what current Core implements as a local-clock comparison.

---

## Could not verify

- **The specific block cited in §6c/§6d** (nBits `17023cc1`, a 4,886-transaction block, a 538-byte Merkle proof, header timestamp `2026-08-23T00:56:49Z`, MTP `2026-08-22T23:22:33Z`). I searched `design/measurements/` for any persisted raw output (`grep -rl "17023cc1\|4,886\|4886\|538-byte" design/`) and found only the spec file itself — unlike every other measured number in the spec (§4, §5, §3's overhead table), there is no `RESULTS_*.txt` backing this instance, so I could not re-run the exact query the operator ran. I instead verified the **formulas and consensus rules** against a live, fully-synced local Core v25.0.0 node (chain tip at the time of testing happened to share the same `nBits`/difficulty, being within the same ~2-week retarget epoch) and got an **exact** numeric match to the spec's stated difficulty, expected-hashes and chainwork figures, which confirms the arithmetic is correct even though I can't independently confirm this exact historical block's identity/timestamp. This gap in `design/measurements/` is also in slight tension with §11's claim that "everything measured is in `design/measurements/`" — flagging for the controller's awareness, not scoring it as a graded external-fact item since it wasn't one of the 13.
- **Exact history of the segwit fee-lying/non_witness_utxo hardening** (item 3). I found consistent secondary corroboration (an Unchained Capital blog post citing a "vulnerability... found in mid-2020" prompting hardware-wallet vendors to reintroduce input-amount checks; independent GitHub issues from WalletWasabi and BTCPayServer describing the same requirement) but no single primary disclosure document (CVE, official advisory, or the original researcher's writeup) in the time available. I rate the claim TRUE on the strength of multiple independent secondary sources agreeing, but flag the sourcing as secondary-only.
- **Whether Foundation Passport or Specter Desktop decode `ur:bytes` specifically.** No source, positive or negative, found for either. Searched their official docs and several web searches combining wallet name + "ur:bytes"; results consistently surfaced only PSBT/`crypto-psbt` workflow documentation, which is an absence of evidence rather than evidence of absence — folded into F-2 as "unverified," not asserted false for these two.
