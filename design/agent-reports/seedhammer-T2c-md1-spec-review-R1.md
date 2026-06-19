<!--
Persisted verbatim. opus-architect R1 GATE re-review of SPEC_seedhammer_T2c_md1_decode.md (folded,
commit 7663318). Reviewer agentId a75e2fd7ddf413c6f. Verdict: GREEN 0C/0I. All three R0 Importants
(I1 validator suite + canonical-origin table; I2 Template-vs-decode-error contract; I3 non-canonical Sh
shapes) and all 5 MINORs (M1-M5) verified CLOSED byte-for-byte against md-codec 0.36.0 source. Two new
MINORs surfaced in the §2.5 reject prose: M6 (InvalidPresenceByte is an identity.rs WalletPolicyId-layer
reject, not a decode validator — out of T2c scope) and M7 (truncation variant is BitStreamTruncated, not
UnexpectedEnd); both non-blocking, both FIXED in the GREEN commit. M8 (non-monotonic §2.x label order
from inserting 2.12/2.13 as 4c/4d) is cosmetic, all cross-refs resolve, left as-is. Disposition: GREEN —
cleared to implementation-plan authoring. The text below is the agent's report verbatim (HTML entities
un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R1 GATE REVIEW — SPEC_seedhammer_T2c_md1_decode.md (folded, commit 7663318)

Re-verified against authoritative source `descriptor-mnemonic/crates/md-codec` **@ 0.36.0** (`Cargo.toml` version line confirmed = `0.36.0`). Per the R1 charter I did NOT re-verify R0-cleared facts (BCH, header layouts, kiw formula, tag table, paths, TLV byte layout, corpus, representation gap, feasibility); I verified only that each fold closed against source and introduced no drift, plus a whole-spec internal-consistency skim.

## Fold verification

**I1 — CLOSED.** The five post-decode validators exist exactly as the spec names them, called in `decode_payload` at `decode.rs:56-69`: `validate_placeholder_usage` (:56), `validate_multipath_consistency` (:58), `validate_tap_script_tree` (:62), `validate_explicit_origin_required` (:68), `validate_xpub_bytes` (:69); definitions in `validate.rs:17/117/141/182/216`. The 5-shape canonical-origin table is at `canonical_origin.rs:45-79` and the values match the spec's new §2.12 byte-for-byte: pkh→`m/44'/0'/0'` (:48), wpkh→`m/84'/0'/0'` (:50), tr-keyonly (`tree:None`)→`m/86'/0'/0'` (:52-54), wsh(multi/sortedmulti)→`m/48'/0'/0'/2'` (:58-62), sh(wsh(multi/sortedmulti))→`m/48'/0'/0'/1'` (:65-73), everything else `None` (:74,:77). Every validator-raised variant the spec adds to §2.5 verified present in `error.rs`: `MissingExplicitOrigin` (:299), `PlaceholderNotReferenced` (:162), `PlaceholderFirstOccurrenceOutOfOrder` (:173), `MultipathAltCountMismatch` (:182), `ForbiddenTapTreeLeaf` (:191), `NUMSSentinelConflict` (:377), `InvalidXpubBytes` (:322). The validators that raise each match: placeholder_usage→{NotReferenced, FirstOccurrenceOutOfOrder, IndexOutOfRange, NUMSSentinelConflict}, multipath→AltCountMismatch, tap_script→ForbiddenTapTreeLeaf, explicit_origin→MissingExplicitOrigin, xpub→InvalidXpubBytes. §2.12, §3 (line 44), and §4.1 carry an identical five-validator list. Closed correctly. (One drift on a co-listed name — see Drift / M6 below.)

**I2 — CLOSED.** §2.4 now states a string is either (a) a decode error (BCH / bit-cursor / reassembly / any §2.12 validator → no Template) or (b) a fully-decoded-AND-validated Template, with `Renderable=false` "reserved for valid-but-complex wires, NOT for decode failures." §4.1 states `Decode` returns `(Template, error)`; ANY md-codec reject → non-nil error + zero Template; a Template is returned ONLY when the wire fully decodes AND passes all five validators. §2.12 reinforces "a card md-codec rejects MUST NOT decode on-device to a displayable Template." The three sections are mutually consistent and match source: `decode_payload` (`decode.rs:15-72`) returns `Result<Descriptor, Error>` and runs the five validators last, so a complex elided-origin wire returns `Err(MissingExplicitOrigin)` not an Ok descriptor. The decode order in §4.1 (header→PathDecl→UseSitePath→kiw→read_node→root-tag allow-list→TLV→five validators) matches `decode.rs:18-69` exactly.

**I3 — CLOSED.** §4.2's added "Canonical-origin interaction (R0-I3)" paragraph correctly states that of the renderable shapes only single-key (pkh/wpkh/tr-keyonly), `wsh(<multi>)`, and `sh(wsh(<multi>))` are canonical and may arrive elided, whereas `Sh(<multi>)`, `Sh(SortedMulti)`, and `Sh(Wpkh(@k))` are non-canonical and reach the renderer only with explicit per-`@N` origins (else `MissingExplicitOrigin`). Verified against `canonical_origin.rs`: `sh(multi)`/`sh(sortedmulti)` → `None` (tests `sh_multi_legacy_returns_none` :228, `sh_sortedmulti_legacy_returns_none` :222; catch-all :74), and `sh(wpkh)` → `None` (Sh body inner.tag==Wpkh≠Wsh, falls through :67-74). The §4.2 *membership* is unchanged and remains correct — all six listed shapes are structurally decodable (`Sh` reads one child `tree.rs:216-227`; `Wpkh` is a valid key-arg child :212). The renderer contract ("display each key's actual decoded OriginPath/Fingerprint ... never assume a canonical path the wire didn't carry") is sound and matches the `KeyOrigin.OriginPath` field already in the type.

**M1 — CLOSED.** §3 now says `Body` is **9-variant** and enumerates all nine (Children, Variable, MultiKeys, Tr, KeyArg, Hash256Body, Hash160Body, Timelock, Empty), matching `tree.rs:18-73` exactly.

**M2 — CLOSED.** §2.3 now names both budgets at their layers: codex32-symbol ≤4 (`codex32.rs:111-112`, doc text "≤4 bits of trailing zero-padding") and TLV-rollback ≤7 (`tlv.rs:296`, `if remaining_at_entry_start <= 7`); cited range `tlv.rs:215-303` and threshold `:296` are accurate.

**M3 — CLOSED (plan-targeted).** §7 line 109 mandates that all embedded test byte-strings/phrases be copied verbatim from live `tests/vectors/*.{bytes.hex,phrase.txt,descriptor.json}`, never the recon, citing the R0-M3 transposition. Carried as required.

**M4 — CLOSED.** §4.1's `ParseHeader` doc-comment now specifies single-string md1 → `{Chunked:false, TotalChunks:1, ChunkIndex:0, ChunkSetID:0}`. Matches source: the single header (`header.rs:30-49`) is 5 bits = `divergent_paths(1)|version(4)` with no csid/count/index fields.

**M5 — CLOSED.** New §2.13 calls out the Tr `is_nums` variable-width branch: read `is_nums(1)` then conditionally `key_index(kiw)`. Citation `tree.rs:271-276` is exact (`is_nums = read_bits(1)` :271; `key_index` read only when `!is_nums` :272-276). The mandated constructed `tr(NUMS,…)` parity vector is reflected in §6's renderable/negative test plan (line 103). Correct.

## Drift check

One name-level drift introduced/retained during the I1 fold:

- **§2.5 lists `InvalidPresenceByte` as one of "the §2.12 post-decode validator rejects."** It is not. `InvalidPresenceByte` (`error.rs:312`) is raised only by `identity::validate_presence_byte` (`identity.rs:253-256`), part of the `WalletPolicyId` / `compute_wallet_policy_id` canonical-record surface (the deferred T2c-b layer). `grep` confirms it is referenced nowhere in `decode.rs` or `tlv.rs` and is raised by none of the five `decode_payload` validators. The R0-I1 recommendation itself suggested adding it, but that recommendation was mistaken. §2.12 and §3 correctly omit it (they list only the five validators + `InvalidXpubBytes`), so the drift is confined to the single parenthetical in §2.5. The variant name is real, so it cannot cause an implementer to invent a non-existent error; the harm is bounded to a misattribution in the reject-set list and a potential spurious "must replicate `InvalidPresenceByte` in decode" reading. MINOR, not blocking.

No other drift. Cross-references resolve: §2.4→§2.12, §2.5→§2.12, §4.1→§2.12, §3↔§2.12↔§4.1 validator lists are identical; §4.2's canonical claims agree with §2.12 and source. The I2/I3 framing is internally consistent across §2.4/§2.5/§2.12/§4.1/§4.2.

## Findings

### CRITICAL
*(none)*

### IMPORTANT
*(none)*

### MINOR
- **M6 (new, drift) — `InvalidPresenceByte` misattributed in §2.5.** Drop it from the §2.5 "§2.12 post-decode validator rejects" parenthetical (leave `InvalidXpubBytes`). It is an `identity.rs` WalletPolicyId-layer reject (`identity.rs:253`), not a `decode_payload` validator, and is out of T2c's decode scope. Cosmetic-to-minor; does not block.
- **M7 (pre-existing, out of strict R1 scope) — phantom variant name `UnexpectedEnd` in §2.5.** §2.5 writes "truncation/`UnexpectedEnd`"; the actual truncation variant is `BitStreamTruncated` (`error.rs:23`); `UnexpectedEnd` does not exist in `error.rs`. This name predates the fold (the R1 charter excludes re-verifying the pre-existing §2.5 structural names), and §6 correctly tests rejection-by-category not error-string equality, so it cannot mislead the implementation. Noted for accuracy; non-blocking. Fix opportunistically alongside M6.
- **M8 (cosmetic) — non-monotonic §2.x label order.** The folds inserted §2.12/§2.13 as list items `4c`/`4d`, so the §2.x labels read 2.1…2.5, 2.12, 2.13, 2.6…2.11. All labels are unique and every cross-reference resolves correctly; only the visual scan order is non-monotonic. Harmless.

## Verdict

**GREEN — 0 Critical / 0 Important**

All three R0 Importants (I1 validator suite + canonical-origin table; I2 Template-vs-decode-error contract; I3 non-canonical Sh shapes) and all five MINORs (M1–M5) folded correctly and verify byte-for-byte against md-codec 0.36.0 source. The fold introduced one minor misattribution (M6: `InvalidPresenceByte` is not a decode validator) and left one pre-existing phantom name (M7: `UnexpectedEnd`); both are MINOR, confined to the §2.5 reject-set prose, cannot affect the by-category rejection tests, and do not block. Recommend fixing M6/M7 opportunistically in the implementation-plan doc or a follow-up touch, but they are not gate-blocking. Spec is cleared for implementation-plan authoring.
