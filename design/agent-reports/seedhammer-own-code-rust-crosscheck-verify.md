# Cross-implementation correctness check — VERIFY cluster (Go fork → primary Rust)

**Task:** A recent adversarial bug-hunt on the SeedHammer fork's OWN Go code found 8 defects (now fixed). Project rule: whenever a defect was found+fixed in a Go port, check whether the SAME defect-class exists in the PRIMARY Rust implementation; if so, fix Rust first (with a test). This report covers the **VERIFY-CORRECTNESS cluster** (H1, H2, M1, L2). READ-ONLY (plus this report).

**Source-of-truth for findings:** `design/agent-reports/seedhammer-fork-own-code-bughunt.md`.

**Rust sources checked (pinned):**
- `mnemonic-toolkit` @ `6de53879` — verify-bundle / restore engine, bundle chunk-reassembly, mk1↔md1 stub-binding, ms1/codex32 comparison.
- `descriptor-mnemonic` @ `54dd765` (md-codec) — md1 chunk reassembly (order determinism).
- `mnemonic-key` @ `1279ef9` (mk-codec).

> NB: `target/package/**` and `.spike-v0.*/**` are build/spike artifacts, NOT live source; live source is under `crates/`. Citations below are to live `crates/` paths.

---

## Findings being cross-checked

- **H1** — multisig mk1 verify SELF-COMPARED a card against itself instead of reading back the operator's engraved plate → silent false-PASS.
- **H2** — md1 gatherer `collected()` used Go map-iteration order (random) → false-FAIL of a multi-chunk md1. (Rust HashMap iteration is ALSO non-deterministic — confirm reassembly sorts/indexes explicitly.)
- **M1** — ms1 `Verify` compared ENTROPY only, ignoring codex32 language/HRP → false-PASS of a non-English readback.
- **L2** — tautological / uninformative multisig verify comparison.

---

## M1 — ms1 verify ignores codex32 prefix/language (entropy-only compare) → VERDICT: N/A (Rust not affected)

**Go defect:** `bundle/verify.go` decoded BOTH the derived and read-back ms1 to ENTROPY bytes via `ms1Entropy()` (which discarded the codex32 prefix + BIP-39 language) and compared only `bytes.Equal(dEnt, rEnt)`. A non-English `mnem`-prefix card with the SAME entropy but a different language byte falsely PASSED, even though it recovers a different wallet.

**Rust analog — the ms1 comparison is a FULL-STRING byte-identical compare, NOT entropy-only:**

Single-sig (`mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`):
```
2048    let supplied_ms1 = supplied.ms1.first().map(|s| s.as_str()).unwrap_or("");
2049    let expected_ms1 = expected.ms1.first().map(|s| s.as_str()).unwrap_or("");
2050    match ms_codec::decode(supplied_ms1) {
2051        Ok(_) => {
...
2058            if supplied_ms1 == expected_ms1 {                 // <-- full codex32 STRING compare
2059                ...
2062                    detail: "ms1 byte-identical".into(),
```
Multisig (same file, `emit_multisig_checks`):
```
2483    if s == exp_ms1 {                                          // <-- full codex32 STRING compare
...
2486        detail: format!("cosigner[{}] ms1 byte-identical", i),
```

The Rust path compares the WHOLE codex32 string verbatim (`supplied_ms1 == expected_ms1`), not a decoded entropy slice. The codex32 string fully encodes the HRP, the prefix (`entr`/`mnem`), and the language — so ANY HRP/prefix/language difference produces a different string and FAILS the comparison. This is in fact STRICTER than the Go fork's *fix* (which compares entropy + language only); Rust catches every string-level divergence.

**Independent sources confirmed:** `expected` is the bundle re-synthesized from the seed; `supplied` is `SuppliedCards { ms1: &args.ms1, mk1: &args.mk1, md1: &args.md1 }` — the operator's independently-supplied `--ms1/--mk1/--md1` artifacts. All four production callsites (`verify_bundle.rs:1045,1144,1252,1726`) wire `&args.ms1` (operator-supplied) as `supplied`. So the two sides are genuinely independent, and the compare is byte-exact over the full secret string. The Go entropy-only narrowing simply does not exist in Rust.

> Test-only nuance (NOT a production defect): several unit tests set `supplied_ms1 = expected.ms1.clone()` (`verify_bundle.rs:3409,3462,3605,3739`) — a tautology *in the test fixture*, mirroring the Go "tests pass derived as readback" masking pattern. Production code does NOT do this. Worth a note for the toolkit's own test-hardening backlog, not a correctness bug.

**Verdict: N/A.** The Rust ms1 leg never reduces to entropy-only; it byte-compares the full codex32 string, so prefix/HRP/language are all load-bearing. No Rust fix needed.

---

## H2 — md1 reassembly/compare order-sensitivity over an unordered collection → VERDICT: N/A (Rust not affected; codec-level confirmed, toolkit-level pending agent)

**Go defect:** `md1Gatherer.collected()` ranged a Go MAP (random iteration order) and `bundle.Verify` did a POSITIONAL `equalStrings(derived.MD1, readback.MD1)` with NO sort by chunk index → a correct multi-chunk md1 FALSE-FAILED.

**Rust analog — codec-level reassembly is deterministic-by-index (explicit Vec + sort + gap check):**

`descriptor-mnemonic/crates/md-codec/src/chunk.rs` `reassemble()`:
```
319    let mut parsed: Vec<(ChunkHeader, Vec<u8>)> = Vec::with_capacity(strings.len());   // ordered Vec, NOT a map
...
351    if parsed.len() != expected_count as usize {                                        // completeness check
352        return Err(Error::ChunkSetIncomplete { got: parsed.len(), expected: ... });
...
358    // Sort by index; verify 0..count-1 with no gaps.
359    parsed.sort_by_key(|(h, _)| h.index);                                               // EXPLICIT sort by chunk index
360    for (i, (h, _)) in parsed.iter().enumerate() {
361        if h.index as usize != i {                                                       // contiguity / no gaps / no dupes
362            return Err(Error::ChunkIndexGap { expected: i as u8, got: h.index });
...
379    let md1_id = compute_md1_encoding_id(&descriptor)?;                                   // cross-chunk integrity:
380    let derived_csid = derive_chunk_set_id(&md1_id);                                      // derived chunk-set-id must
381    if derived_csid != expected_csid { return Err(Error::ChunkSetIdMismatch{..}); }       // match every header's csid
```

No HashMap/HashSet iteration anywhere in the reassembly path. The input is a positional `&[&str]` slice; chunks are placed into a `Vec`, explicitly sorted by `h.index`, and contiguity + completeness + cross-chunk-id integrity are all verified. Out-of-order input is sorted (order-tolerant); missing/duplicate/gap chunks are rejected, not silently dropped. This is the correct pattern the Go fork ADOPTED in its fix.

The toolkit's multisig md1 path calls this exact function: `verify_bundle.rs:~2360` `let supplied_md_decoded = md_codec::chunk::reassemble(&supplied_md1_strs);` — so it inherits the codec's determinism. (Toolkit-side md1 *compare* semantics confirmed by the dispatched agent below.)

**Toolkit-side confirmation (agent-verified):** Every md1 verify/restore verdict either routes through the order-tolerant codec `reassemble()` or uses ORDERED collections with sort-then-compare:
- `verify_bundle.rs:2359` (supplied) and `:2698-2700` (expected) both reassemble via `md_codec::chunk::reassemble()`, then compare DECODED `Descriptor` semantic fields, not raw chunk strings.
- `md1_xpub_match` is an explicit sort-then-compare multiset: `verify_bundle.rs:2731-2735` `exp_sorted.sort(); act_sorted.sort(); let pubkeys_match = exp_sorted == act_sorted;`.
- mk1 grouping uses `BTreeMap<u32, Vec<&str>>` (`:2318`, csid-ascending `.into_values()`) and `check_mk1_template_stubs` uses `BTreeSet<u32>` set-equality (`:1002-1009`). `restore.rs` groups cosigners/chunks via `BTreeMap<u8, …>` (`:1452`, `:1474`, `:3091`) and tracks verified positions in `BTreeSet<u8>` (`:2992`). NO `HashMap`/`HashSet` in any md1/chunk/mk1 reassembly or compare path in either crate (repo-wide sweep clean; the only md-codec HashMaps are an unrelated label-renumbering lookup and a dedup set).

**One non-H2 positional raw-string compare (cosmetic, NOT a defect):** `verify_bundle.rs:645` `let md1_match = expected.md1 == args.md1;` (single-sig `md1_template_match`). This is NOT the H2 class: both sides are DETERMINISTICALLY ordered (expected from `chunk::split()`'s ascending-index loop; supplied is the user's literal CLI card order) — neither is map-iterated. It is a non-funds-bearing card-authenticity/template-binding check (the funds-path descriptor was already reassembled order-tolerantly at `:363` before this runs), and a single-sig keyless template is realistically single-chunk (≤320 payload bits). The multisig sibling (`:882`) is already immune (compares key-invariant `WalletDescriptorTemplateId` bytes, not strings). Residual: IF such a template were ever multi-chunk AND cards supplied out of order, this one redundant binding check could benign-false-FAIL. Worth a one-line symmetry hardening note (sort or reassemble-compare), NOT a correctness defect.

**Verdict: N/A.** Rust md1 reassembly is deterministic-by-index (explicit `sort_by_key` on `h.index` + gap/count/integrity enforcement) and every toolkit verdict routes through it or uses ordered `BTreeMap`/`BTreeSet` sort-then-compare. The Go H2 map-order false-FAIL does not exist in Rust. (Optional cosmetic symmetry note at `verify_bundle.rs:645`.)

---

## H1 — multisig mk1 verify self-compares re-derived value against itself (no independent readback) → VERDICT: N/A (Rust not affected)

**Go defect:** the multisig verify flow re-derived the operator mk1 from the re-typed seed and compared that re-derived value against ITSELF (both sides = `reDerived.MK1`); the flow structurally REFUSED to read back the engraved mk1 plate over NFC. The fingerprint/xpub/origin-path comparison was a tautology → silent false-PASS on a mis-engraved mk1. (The single-sig sibling correctly read BOTH plates back.)

**Rust analog — the verify-bundle compares a seed-DERIVED `expected` against INDEPENDENTLY-supplied artifacts:**

The whole engine has two genuinely independent sides:
- **EXPECTED** (re-derived from the seed): `synthesize_unified` (`synthesize.rs:994-1085`) builds `mk1`/`md1`/`ms1` purely from `ResolvedSlot`s produced by `resolve_slots(&args.slot, …)` — i.e. the seed/phrase/entropy in `--slot @N.<secret>=`. (`run_multisig` `verify_bundle.rs:1230-1249`; `run_full` `:1025-1044`.)
- **SUPPLIED** (independent user artifacts, the host-CLI analog of the NFC plate readback): `SuppliedCards { ms1: &args.ms1, mk1: &args.mk1, md1: &args.md1 }` (`verify_bundle.rs:1252-1256`), sourced from `--ms1/--mk1/--md1`, positional autodetect, or a `--bundle-json` envelope file (`:1887-1949`). These are NEVER re-derived from the seed.

Per-cosigner mk1 compare (`emit_multisig_checks`, `verify_bundle.rs:2561-2637`):
- `exp = expected_mk1_per_cos[i]` — decoded from the SEED-synthesized `expected.mk1` (built at `synthesize.rs:1043-1052`).
- `sup = card_for_cosigner[i]` — decoded from the user-SUPPLIED `--mk1` cards (grouped by chunk_set_id at `:2318-2354`).
- `mk1_xpub_match`: `if exp_x == act_x` (`:2571`), where `exp_x = exp.xpub.to_string()` (seed-derived) and `act_x = sup.xpub.to_string()` (supplied card). Likewise `mk1_fingerprint_match` (`:2598`) and `mk1_path_match` (`:2619`). This is derived-vs-supplied, NOT derived-vs-derived.

In a host CLI, the "engraved plate readback" is exactly the user-supplied `--mk1`/`--md1` argument; the code compares the seed-derived value against THAT — the structurally-correct analog of the Go single-sig sibling. The single-sig Rust path (`emit_verify_checks` `:2176-2248`) is identically structured.

**Tests pinning non-vacuity:** `verify_bundle.rs:3459-3505` `helper_singlesig_tampered_mk1_populates_forensics` tampers the SUPPLIED mk1 and asserts the mk1 leg FAILS; `cli_verify_bundle_md1_template_multisig.rs:509-545` asserts a wrong cosigner mk1 produces exit≠0 / no `result:ok`. If the compare were a self-tautology these could never fail.

**Verdict: N/A.** The Rust mk1 comparison is genuinely derived-vs-supplied across independent sources. The Go derive-and-self-compare defect does not exist.

---

## L2 — multisig md1/mk1 legs tautological (clone-vs-original / stub-vs-self) + success copy over-claims → VERDICT: N/A (Rust not affected)

**Go defect:** the readback bundle's md1 was a verbatim CLONE of the supplied/derived md1, and its mk1 was a STUB computed from that same md1, so the md1/mk1 legs could never fail; `checkStubBinding` passed by construction; yet the UI over-claimed "the engraved bundle matches the seed."

**Rust analog — no clone-vs-original, no stub-vs-self, no over-claiming message:**

- **md1 leg is a real cross-source compare:** `emit_md1_checks` (`:2838-2854`) and the multisig md1 block (`:2698-2764`) reassemble the SUPPLIED md1 (`supplied_md_decoded`, from `--md1`) and the seed-synthesized EXPECTED md1 (`expected_md_decoded`) independently, then compare pubkey TLVs (`exp_sorted == act_sorted` `:2735`; `exp_xpub == act_xpub` `:2854`). Supplied md1 is decoded from the user artifact, not cloned from expected.
- **mk1↔md1 binding is a real cross-source check:** supplied mk1 cards are mapped to cosigner slots by matching the supplied-card xpub against the supplied **md1**'s `tlv.pubkeys` (`:2392-2424`) — supplied-mk1 ↔ supplied-md1 cross-bind — then each mapped card is compared against the seed-derived `expected`. Never self-referential.
- **Template-id stub binding is two independently-sourced ids:** `verify_multisig_template` (`:808-983`): `completed_template_id = compute_wallet_descriptor_template_id(&outcome.completed)` (from the seed `--from` + cosigner keys via the shared completion search, `:878-879`) vs `supplied_template_id = compute_wallet_descriptor_template_id(d)` (from supplied `--md1`, `:880-881`); `md1_match = completed_template_id.as_bytes() == supplied_template_id.as_bytes()` (`:882`). `check_mk1_template_stubs` (`:992-1010`) checks the SUPPLIED `--mk1` stubs' chunk_set_ids against `derive_mk1_chunk_set_id_for_slot(completed_template_id[0..4], slot)` — supplied field vs a value derived from the COMPLETED (seed-side) wallet.
- **No over-claiming success string.** The verdict is the literal `result: ok` / `result: mismatch`, computed from `checks.iter().any(|c| !c.passed)` (`:499-522`). A repo-wide grep found NO "the engraved bundle matches the seed"-class guarantee string. Watch-only paths emit explicit `warning:` stderr lines naming what is NOT verified (`:1086-1100`, `:1188-1202`) — the opposite of over-claiming.

**Honest documented limit (mirrors the Go L2 "inherent" note, but already handled correctly):** `cli_verify_bundle_md1_template_multisig.rs:551-564` documents that for a *completed* wallet `md1_template_match` cannot fail by construction — and explicitly names the **completion search** (`--expect-wallet-id`/`--search-address` consumed by `complete_multisig_template`) as the real funds-safety gate, returning exit 4 / non-ok on NO-MATCH/AMBIGUOUS. Negative tests `:509-545` (wrong cosigner) and `:567-605` (cross-shape mix) pin that foreign/mismatched supplied cards yield non-zero exit and no `result:ok`. So the Rust design routes the "can't-fail-by-construction" leg's responsibility onto a genuine search gate AND tells the truth about it — it does not over-claim.

**Verdict: N/A.** No clone-vs-original or stub-vs-self tautology; binding checks are real cross-source comparisons; no over-claiming message.

---

## Verdict table

| Finding | Go defect | Rust analog | Verdict | Key Rust evidence |
|---|---|---|---|---|
| **H1** | multisig mk1 verify self-compares re-derived value vs itself; never reads back the plate | seed-derived `expected.mk1` compared against independently-supplied `--mk1` artifact | **N/A** | `verify_bundle.rs:1252-1256` (SuppliedCards), `:2561-2637` (exp-vs-sup mk1 compare); test `:3459-3505` |
| **H2** | md1 gatherer ranges a Go map (random order); positional compare → false-FAIL | deterministic `Vec` + `sort_by_key(h.index)` + gap/count/integrity checks; toolkit uses `BTreeMap`/`BTreeSet` + sort-then-compare | **N/A** | `chunk.rs:319,351-356,359-367,379-386`; `verify_bundle.rs:2359,2698-2700,2731-2735` |
| **M1** | ms1 verify compares entropy-only, ignoring codex32 prefix/HRP/language | full codex32 STRING byte-identical compare (catches prefix/HRP/language) | **N/A** | `verify_bundle.rs:2058` (single-sig), `:2483` (multisig) |
| **L2** | multisig md1/mk1 legs tautological (clone/stub-self) + over-claiming copy | real cross-source compares; two independent template-ids; no over-claiming string; honest watch-only warnings | **N/A** | `verify_bundle.rs:2698-2764,2838-2854,880-882,992-1010,499-522` |

## Overall conclusion

**No Rust-side correctness fixes are needed for any of the four VERIFY-cluster findings (H1, H2, M1, L2).** All four are **GO-ONLY**; the primary Rust `mnemonic-toolkit` verify-bundle / restore engine and the `md-codec` chunk reassembly do NOT carry the analogous defect class:

1. **Architectural root cause of immunity (H1/L2):** the Rust verify-bundle is a host CLI that re-synthesizes an `expected` bundle from the seed and compares it against *independently-supplied* `--ms1/--mk1/--md1` artifacts (the host-CLI analog of the engraved-plate readback). There is no re-derive-and-self-compare and no clone/stub-self tautology; the stub/template-id bindings are genuine cross-source checks; and no success message over-claims (the verdict is a plain `result: ok|mismatch`, with explicit watch-only `warning:` disclaimers). Negative tests pin that tampered/foreign supplied cards produce `mismatch`/exit-4.
2. **Codec determinism (H2):** `md-codec::chunk::reassemble` collects into an ordered `Vec`, explicitly `sort_by_key(h.index)`, and enforces completeness + `0..count-1` contiguity + cross-chunk-id integrity. No `HashMap`/`HashSet` iteration exists in any md1/chunk/mk1 reassembly or compare path in either crate.
3. **String-level secret compare (M1):** the ms1 leg is a full codex32 byte-identical string compare, never reduced to entropy-only — strictly STRONGER than even the Go fork's *fix*; prefix, HRP, and language are all load-bearing.

**The defects were genuine Go-port regressions**, introduced by the firmware's NFC-gather/re-derive flow layer (Go map iteration; a flow that re-derives instead of reading back; an `ms1Entropy()` helper that discards language), none of which have a counterpart in the Rust pipeline. The Rust primary implementation is the more conservative reference here.

### Optional (NON-blocking) hardening notes — quality only, NOT correctness defects
- **(H2 symmetry)** `verify_bundle.rs:645` `md1_match = expected.md1 == args.md1` is a positional raw-string compare in the single-sig keyless-template authenticity check. Both sides are deterministically ordered today (encoder `split()` ascending-index vs user CLI order) and the template is realistically single-chunk, so it is NOT the H2 defect — but for symmetry with the multisig sibling (which compares template-id bytes) it could sort or reassemble-compare to be future-proof if such a template ever became multi-chunk. Benign-false-FAIL at worst; funds path unaffected.
- **(M1 test hygiene)** several verify-bundle UNIT tests set `supplied_ms1 = expected.ms1.clone()` (`verify_bundle.rs:3409,3462,3605,3739`) — a fixture-level tautology mirroring the Go "tests pass derived as readback" masking pattern. Production code does not do this (it wires `&args.ms1`), and integration tests (`cli_verify_bundle_*`) use independent fixtures, so coverage is real — but pinning at least one unit test with a genuinely-independent supplied ms1 would harden against future drift. Test-only; not a code defect.


