# R0 GATE RE-REVIEW (round R1, post-fold) — SPEC_seedhammer_T5_bundle_sequencing.md

**Reviewer:** opus architect (R0 pre-implementation gate, R1 re-review). **Date:** 2026-06-19.
**Spec under review (folded):** `design/SPEC_seedhammer_T5_bundle_sequencing.md`
**Prior review re-checked:** `design/agent-reports/seedhammer-T5-bundle-sequencing-spec-review-R0.md` (NOT GREEN — 2C/3I/5 Minor).
**Recon-refresh consulted:** `design/agent-reports/seedhammer-T5-recon-refresh-postT10.md`.
**Authoritative sources verified directly (not trusted from prose):** GO fork `/scratch/code/shibboleth/seedhammer` @ `bb0e506` (confirmed HEAD: `git rev-parse HEAD` = `bb0e506fd71972718766561f5ed244d4c20fc618`, "Merge #10b"); HOST `crates/me-cli/src/bundle.rs`. Baseline `go test ./gui/ -run 'TestAllocs|TestEngraveXpubProgramNavigable' -count=1` = **GREEN** (`ok seedhammer.com/gui 1.445s`), unchanged from R0 (the fold is doc-only; no source touched).

---

## VERDICT: GREEN

0 Critical, 0 Important. The two blocking Criticals (C-1, C-2) and the three Importants (I-A, I-B, I-C) from R0 are all **CLOSED** with the fold faithfully matching authoritative source. The fold introduced **no drift** in the source-load-bearing claims, the Option-A faithfulness ruling still holds, and the new-csid-=-new-card inversion remains sound for chunked cards while single-string cards are now correctly routed off the csid map. Two **Minor** items remain (one new, surfaced by the fold; one carried) — both NON-blocking and may be folded opportunistically or pushed to the implementation plan. **The spec is cleared for the implementation plan.**

---

## CLOSED/OPEN STATUS OF THE FIVE R0 FINDINGS (each with source evidence)

### C-1 — single-string undefined / csid-0 collision / mk1 with no integrity — **CLOSED.**
The spec now defines all three single-string cases precisely and matches the host:

- **Single mk1 → REFUSED.** Spec §2 IN line 18, I-1 line 59, acceptance #5 line 51. Verified:
  - `mk.Decode([single])` takes the `!first.Chunked` path that returns the fragment with **NO cross-chunk hash** — `mk/mk.go:177-181` (`return frags[0].fragment, nil // single-string fragment IS the bytecode (no hash)`). So a single mk1 would "verify" on its BCH alone — exactly the integrity bypass the fold now refuses.
  - Host parity confirmed: `bundle.rs:128` — the non-`Chunked` arm returns `Err(BundleError::Mk1SingleString(...))`; display text `bundle.rs:56-58` ("unsupported for bundle (no chunk_set_id)").
  - Underlying fact (mk1 is always chunked): `xpubCompactBytes = 73` (`mk/mk.go:127`) and the mk1 bytecode layout `header(1)|stub_count(1)|stubs(4N)|[fp(4)]|path|compact73` (`mk/encode.go:48`) → minimum ≥75 B bytecode cannot fit a single string. The spec's "73B > 56B cap" phrasing is directionally correct; the load-bearing facts (no-hash single path + host refusal) are both confirmed, so the exact 56 B figure is not gate-relevant.
- **Single md1 → accepted as a standalone 1-plate card, validated by `md.Decode`, in a NON-csid bucket.** Spec §2 IN line 19, I-1 line 59, I-2 line 60, acceptance #5 line 51. Verified:
  - The "BCH + structural decode = its integrity" claim is **SOUND.** `md.Decode(s)` (`md/md.go:1216-1230`) calls `codex32.MDDataSymbols(s)` (BCH/checksum validation, `:1217`), rejects chunked (`:1221-1222`), then `decodePayloadValidated(b, 5*len(syms))` (full TLV structural decode + placeholder/policy validation, `:1225`) and `summarize(d)` (`:1229`). That is genuinely full validation of a single-string md1 — no chunk set to reassemble, so there is nothing more to integrity-check.
  - Host parity: single md1 = `Parsed::Md1Single` (`bundle.rs:75-76,150,160-162`), engraved as its own `Integrity::BchOnly` plate carried VERBATIM (`bundle.rs:228-240`, `string: Some(s.clone())`). The host additionally per-string PRISTINE-validates via `validate::validate` (`bundle.rs:101`) — the Go `md.Decode` is the analogue.
  - csid-0 collision is now closed: single md1 gets its own bucket/unique key (spec I-2 line 60, "Single-string md1 cards get a unique non-csid bucket"), NOT keyed by the zero-value csid. Verified the zero-value: `ParseChunkHeader` short-circuits a single (non-chunked) md1 to `ChunkHeader{Chunked: false}` with `ChunkSetID` unset (`md/chunk.go:193-196`); single mk1 likewise has `ChunkSetID` unset in the `typeSingle` arm (`mk/mk.go:74-76`). Routing singles off the csid map prevents two distinct singles from merging at bucket 0.
  - Also closed the latent `gatherIgnored` drop: a single md1 offered to a fresh `md1Gatherer` returns `gatherIgnored` (`md1_gather.go:38-40`), so the fold's choice to validate it via `md.Decode` OUTSIDE the sub-gatherer is the correct path (it would otherwise be silently dropped).
- **Routing key stated precisely:** §2 IN line 20 + I-2 — only CHUNKED cards keyed by `chunk_set_id`; a new csid starts a new card; singles take a separate non-csid path. This matches the host partition (`bundle.rs:198-294`: `md1_singles` Vec vs `md1_groups`/`mk1_groups` BTreeMaps by csid).
- Acceptance now covers all three: #5 line 51 ("single mk1 → REFUSED … single md1 → standalone 1-plate … two distinct single md1 cards do NOT collide"). **CLOSED.**

### C-2 — ms1 refusal won't fire (silently dropped, not refused) — **CLOSED.**
The spec now mandates an EXPLICIT `codex32.String`/HRP-`ms` refusal case. Spec §2 IN line 17, §3 line 36, I-3 line 61, acceptance #4 line 50. Verified:
- The scanner yields a `codex32.String` (NOT `mdmkText`) for an `ms` secret: `gui/scan.go:70-71` tries `codex32.New(string(buf))` BEFORE the md/mk check at `:72-73`. `codex32.New` accepts any HRP that passes `inputHRP` + checksum (`codex32/codex32.go:98-126`), so a valid `ms1` parses as a `codex32.String` and is returned as such.
- The existing single-card gather loops type-assert `mdmkText` ONLY (`gui/mk1_inspect.go:218`, `gui/md1_gather.go:135`: `if s, ok := scan.Object.(mdmkText); ok`); a `codex32.String` fails that assertion and is dropped silently — confirming the R0 defect. The fold's explicit `case codex32.String:` refusal is therefore necessary and correctly placed in the bundle gather loop (a sibling flow, not the single-card loops).
- The refusal-case placement is correct: the existing top-level scan dispatch (`engraveObjectFlow`, `gui/gui.go:1881-1882`) routes `codex32.String → engraveCodex32` (the secret-engrave path). The bundle loop must instead route it to a refusal + continue — the fold says exactly this.
- The secret is never *gathered* either way (dropped/refused, never accumulated) — the fold correctly frames this as a UX-correctness + acceptance-fidelity fix, not a leak fix. Acceptance #4 now asserts it is NOT silently dropped and that the refusal message + end-of-bundle reminder appear. **CLOSED.**

### I-A — enum insertion position + derived consts/wrap bounds — **CLOSED.**
Spec §2 IN line 15, §3 line 35, I-6 line 64, acceptance #9 line 55. Verified:
- The spec now mandates `engraveBundle` BEFORE `qaProgram` (so `qaProgram` stays last/out of the navigable range) and explicitly states `npage`/`npages` are *derived consts* that must be rewritten off `engraveBundle`, plus both wrap bounds.
- Source confirms the miscount risk: enum `{backupWallet=0, engraveXpub=1, qaProgram=2}` (`gui/gui.go:147-151`); `const npage = int(engraveXpub) + 1` (`:1834`) and `const npages = int(engraveXpub) + 1` (`:1853`) both evaluate to 2; left-wrap `m.prog = engraveXpub` (`:1629-1630`) and right-wrap `if m.prog > engraveXpub` (`:1636-1637`). If `engraveBundle` were appended after `qaProgram`, all four would still key off `engraveXpub`, leaving the new program unreachable and the pager dot-count wrong. The fold's "rewrite ALL of these off `engraveBundle`, the new navigable upper bound" is correct and complete. **CLOSED.**

### I-B — title fail-open + nav-test must land ON engraveBundle — **CLOSED.**
Spec §2 IN line 15, §3 line 35, I-6 line 64, acceptance #9 line 55. Verified:
- The spec now requires BOTH arms: the `title` switch arm (`gui/gui.go:1655-1660`) AND the `layoutMainPlates` arm (`:1849`).
- Source confirms the asymmetry: `layoutMainPlates` has `panic("invalid page")` on a missing case (`:1849`) — a hard panic; the title switch (`:1655-1660`) has NO default, so a missing `engraveBundle` case yields `titleTxt = ""` (silent blank-title fail-open). The fold correctly flags the title arm as the silent one.
- The nav-test fix is correct: the current `TestEngraveXpubProgramNavigable` (`gui/derive_xpub_program_test.go`) navigates Right onto "Account Xpub" then wraps to "Backup Wallet" (`:21-39`), with the comment at `:30-31` pinning `engraveXpub` as the upper bound. After insertion, the second Right must land ON `engraveBundle`; the fold requires the test to land there and assert a non-blank title (catching the fail-open). **CLOSED.**

### I-C — multiPlateEngrave parameterization keeps deriveXpubFlow byte-unchanged — **CLOSED (with one Minor citation nit, M-2 below).**
Spec §2 IN line 23, I-7 line 65, risk #5 line 73, acceptance #10 line 56. Verified:
- The spec now pins the contract: add parameters (card label + plate-count context) with defaults / a thin bundle-specific sequencer that leaves `deriveXpubFlow`'s call site BYTE-UNCHANGED, and explicitly forbids mutating the mk1 copy/behavior `deriveXpubFlow` depends on.
- Source confirms the load-bearing copy that must be preserved: `multiPlateEngrave` (`gui/derive_xpub.go:263-293`) with mk1-specific `showError(ctx, th, "Account Xpub", "This key card doesn't fit a plate.")` (`:269`); `abortWarning` (`:298-302`) with "This key card set can't be restored…" (`:300-301`). Single live caller: `deriveXpubFlow` at `gui/derive_xpub.go:162` (`multiPlateEngrave(ctx, th, strs)`), confirming I-7's no-regression target.
- The generalization mechanism is genuinely format-agnostic (`validateMdmk(params, s)` per string, `:267`; `validateMdmk` at `gui/gui.go:1897`) and safe to reuse for md1 + mk1.
- Acceptance #10 line 56 asserts `deriveXpubFlow` byte-unchanged call site (I-C). **CLOSED.** (Citation nit → M-2.)

---

## RE-CONFIRMED ITEMS (folds can drift — all still hold)

- **Option-A faithfulness (UPHELD prior — still accurate).** `bundle.rs` groups purely by `chunk_set_id` into per-format BTreeMaps (`:198-294`), proves each set independently complete/integral (`md_codec::chunk::reassemble` `:246`; `mk_codec::decode` `:273`), refuses ms1 up front (`:97-99,188-192`), and appends a trailing ms1 reminder unconditionally (`:296-306`). No expected-card-count, no md1↔mk1 cross-match anywhere. Spec §1/§4/I-8 remains accurate; the fold's added single-string handling actually *improves* the host-fidelity claim (it now matches `bundle.rs:128` + the `Md1Single` path, the very axis R0 flagged as under-stated).
- **New-csid=new-card inversion (chunked) — still sound.** Each sub-gatherer (`mk1Gatherer.offer` `gui/mk1_inspect.go:55-73`; `md1Gatherer.offer` `gui/md1_gather.go:30-53`) self-keys on its header and only ever sees its own csid's chunks, so the `gatherForeign` arm (`mk1_inspect.go:65-67`, `md1_gather.go:45-47`) is never hit inside a correctly-routed sub-gatherer. A chunk for no existing card primes a NEW sub-gatherer. I-7 (single-card foreign semantics untouched) preserved. SOUND.
- **Per-card integrity gates — still prevent a partial/tampered chunked card.** mk1: `mk.Decode` reassembles + enforces the cross-chunk `SHA-256[0:4]` gate (`mk/mk.go:214-222`), with completeness via `len(frags)==total` (`:184`) and per-chunk csid/index consistency (`:189-205`). md1: `Reassemble` enforces consistency (version/csid/count all-equal, `md/chunk.go:248-252`), completeness + gap-free indices (`:253-270`), and the cross-chunk csid re-derivation gate `deriveChunkSetID(id) != expCsid → ErrChunkSetIDMismatch` (`:284-291`). `complete()` is true only at `len(set)==total` (`mk1_inspect.go:75`, `md1_gather.go:55`). A partial chunked card cannot complete; a tampered one fails Decode. CONFIRMED.
- **I-4 verbatim engrave** — host carries the input string into every plate (`bundle.rs:234,260,287`); chunked sets reassembled only to PROVE integrity, never re-encoded. Spec I-4/§4 matches. CONFIRMED.
- **I-5 set-level abort** — `abortWarning` (`gui/derive_xpub.go:298-302`) records no completed state (dismiss-only `showError`); spec I-5/risk #4 extends this to the partial-bundle warning. CONFIRMED.
- **I-7 no-regression** — single-card flows and codecs untouched (the bundle loop is a sibling flow; gatherers reused unchanged). CONFIRMED via the single live caller of `multiPlateEngrave` (`derive_xpub.go:162`).
- **8-site lockstep list** — spec §3 line 35 lists `:1489-1502,1628-1631,1636-1639,1655-1660,1834,1842-1850,1852-1853` + enum `:147-151` + nav-test `derive_xpub_program_test.go:30-31`. Every citation verified exact against source (dispatch `:1489-1502`; left-wrap `:1628-1631`; right-wrap `:1636-1639`; title `:1655-1660`; npage `:1834`; layoutMainPlates `:1842-1850` panic at `:1849`; npages `:1852-1853`). MATCHES recon Job-1 §"GUI program + lockstep". CONFIRMED.
- **Acceptance gate now has 10 items** covering: multi-card gather (#1), new-csid=new-card (#2), per-card integrity (#3), ms1-not-dropped refusal (#4, C-2), single-string mk1-refuse / md1-standalone / no-collide (#5, C-1), guided engrave sequence (#6, I-4), set-level abort (#7, I-5), empty + half-scanned-on-done (#8, M-1/half-scan), program nav incl. fail-open title + panic + `TestAllocs` (#9, I-A/I-B), no-regression + fuzz (#10, I-C/I-7). All invariants and the new single-string + empty + ms1-not-dropped cases are covered. CONFIRMED.

---

## MINOR (non-blocking — fold opportunistically or carry to the implementation plan)

### M-1 (NEW, surfaced by checking acceptance #1's fixtures). Acceptance #1's "a 3rd card (different csid) adds a 3rd" cannot be satisfied with the cited goldens as-is.
Spec acceptance #1 line 47 + §3 line 38 cite `wshSortedmultiChunks` + `v1c0`/`v1c1`/`v3c1`. Verified: `wshSortedmultiChunks` is a complete 6-chunk md1 set (`gui/md1_gather_test.go:14-21`); `v1c0`+`v1c1` are a complete 2-chunk mk1 set (`gui/mk1_inspect_test.go:11-12`). BUT `v3c1` is ONLY chunk-index-1 of a *different* csid set — there is **no `v3c0`** in-tree (grep: `v3c1` appears only at `mk1_inspect_test.go:14,28`, used solely to assert `gatherForeign`). A sub-gatherer needs the COMPLETE set to verify+add, so `v3c1` alone stays pending and never becomes the "3rd verified card." **Fix (plan-level):** for the "3rd card adds a 3rd" assertion, the implementer must supply a *complete* distinct-csid set (a second full chunked mk1 or md1 at a new csid), not `v3c1` alone — or the plan should drop the "3rd card" sub-clause from #1 and test new-csid-=-new-card via #2 with a complete set. The spec's "embedded golden chunk sets" is achievable for the 2-card case verbatim; only the 3rd-card sub-clause over-claims `v3c1`. NON-blocking (spec logic is sound; this is a fixture-availability detail for the plan).

### M-2 (carried/refined). I-C citation points at the copy strings, not the literal call site.
Spec §2 IN line 23 says "leave `deriveXpubFlow`'s existing call site (`derive_xpub.go:269,300-301`) BYTE-UNCHANGED." Verified: the actual call site is `multiPlateEngrave(ctx, th, strs)` at `gui/derive_xpub.go:162`; lines `:269` and `:300-301` are the mk1-specific *copy strings INSIDE* `multiPlateEngrave`/`abortWarning` (the text that must be preserved), not the call site. The intent is correct and the cited lines ARE the load-bearing copy to preserve — but the label "call site" is imprecise. **Fix:** reword to "leave `deriveXpubFlow`'s call site (`derive_xpub.go:162`) and the mk1 copy it depends on (`:269`, `:300-301`) byte-unchanged." NON-blocking (cosmetic; the exec-review can still check both).

### Carried-over Minors that the fold already addressed (no longer open):
- R0-M-1 (empty bundle) → now acceptance #8 line 54 + risk #2; host parity `BundleError::Empty` (`bundle.rs:183-184`). Addressed.
- R0-M-2 (duplicate whole card) → §2 IN line 21 ("duplicate of an already-complete card → ignore with feedback"). Addressed.
- R0-M-3 (fixtures real, don't invent) → §3 line 38 cites them; see M-1 above for the one over-claim.
- R0-M-4 (non-md/mk scan objects, bip39 mnemonic dropped) → §2 IN line 17 ("drop+note any other non-md/mk scan object (e.g. a bip39 mnemonic, a bare address)"). Verified scanner emits `bip39.Mnemonic`/`*bip380.Descriptor`/`addressText`/`debugCommand` (`gui/scan.go:58-79`); the bundle loop drops all non-md/mk (the mnemonic is the only secret-ish one and is NOT routed/engraved). Addressed.
- R0-M-5 (alloc gate off-path; re-run `TestAllocs` after enum change) → acceptance #9/#10 line 55-56 re-run `TestAllocs`. Baseline GREEN confirmed. Addressed.

---

## DRIFT CHECK
No drift introduced by the fold. The fold is doc-only (no source changed; baseline tests remain GREEN). Every source citation the fold added or moved was re-verified against `bb0e506`: the single-string mk1 no-hash path, the single md1 `md.Decode` validator, the csid zero-values, the `codex32.String` scan classification, the `mdmkText`-only assert sites, the enum/consts/wrap-bounds/title/panic sites, and the `multiPlateEngrave`/`abortWarning` copy + single call site. The Option-A faithfulness ruling and the chunked-card integrity-gate ruling from R0 are unchanged and still hold. The only items not 100% pinned to source are the recon's "56 B single-string cap" figure (not load-bearing; the no-hash + host-refusal facts carry it) and the M-1 fixture gap (plan-level, not spec-logic).

---

## RESULT
**VERDICT: GREEN (0 Critical / 0 Important).** C-1, C-2, I-A, I-B, I-C are all CLOSED with source-verified evidence; the fold introduced no drift; the acceptance gate (10 items) and invariants (I-1..I-8) cover every flagged case including the new single-string, empty-bundle, and ms1-not-dropped scenarios. The two remaining Minors (M-1 fixture over-claim for the 3rd-card case; M-2 I-C citation wording) are NON-blocking and should be carried into the implementation plan.

**The spec is cleared for the implementation plan.** Per project policy: implementation plan → its own opus R0 to 0C/0I → single-implementer TDD in a worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed + DCO) → push bg002h. The ms1 secret-spine and the verbatim-engrave invariant carry through.
