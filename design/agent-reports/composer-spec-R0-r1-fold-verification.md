# R1 — mechanical fold-verification of `bc1c07c` against the four R0 round-0 reports

**VERDICT: 46 FIXED / 1 PARTIAL / 1 DECLINED-BY-DEFAULT / 0 NOT FIXED, out of 48
Critical+Important findings. 1 regression found (a stale `§9 item N` cross-reference
introduced by the renumbering). 0 dropped normative content. 13/13 newly-added
citations checked and support their claims.**

Scope: did fold commit `bc1c07c` fix every C/I in the four persisted reports, and
did it introduce a new defect. Rulings (brainstorm §2, C1-C29) and the controller
defaults recorded at brainstorm §3.12 are treated as final and not re-litigated;
where a finding's suggested fix differs from what the fold did, and §3.12 records
that as a deliberate choice, the finding is graded DECLINED-BY-DEFAULT rather than
NOT FIXED.

---

## 1. Per-finding table — all 48 Critical/Important findings

### `composer-spec-R0-r0-correctness.md` (1C/11I — 12/12 FIXED)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C1 | date entry has no lower bound; pre-1985 dates encode as a block height | §4c: "the OPERAND floor is 1985-11-05 00:53:20 UTC, the DATE-ENTRY floor is 2009-01-03 (§6b)"; §6b: "the entry refuses every date before 2009-01-03 with 'Dates before 2009 cannot be written as a time lock.'"; §12 item 4 adds "a date before 2009-01-03" as a required negative vector | FIXED |
| I1 | tr spine undefined for 0/1 leaves; addresses diverge | §5 tr `paths combine` row rewritten over `L` (path list minus the extracted internal-key path): "m = 0: `tr(@0/<0;1>/*)`, no tree. m = 1: `tr(IK, P)`, the leaf written bare... m ≥ 2: one leaf per path on a right spine..., leaf j (1-based) at depth min(j, m−1)" | FIXED |
| I2 | §5 defines no lowering for `sh`/`sh(wsh)`; conflicts with §4a at n=1 | §4a: "n ≥ 2... n = 1 is refused at the picker"; §5 header: "wsh (and `sh`/`sh(wsh)` for their single path, wrapped)"; §4e: "`sh`/`sh(wsh)` with anything other than ONE unlocked, unhashed path whose key set has n ≥ 2 → REFUSE" | FIXED |
| I3 | `md compile` cross-check can't run for keyless-wsh, but §12 requires it | §5b: "The compile leg is carved out for keyless families: the compiler refuses any sigless spend path in both contexts...; those vectors keep the other legs and compare `lift()` against a hand-written `Semantic` policy" | FIXED |
| I4 | no refusal for two slots resolving to the same xpub | §7d: "Two slots resolving to the same xpub → REFUSE at the mapping review, naming both slots (BIP-388 l.193, pairwise distinct...)"; §12 item 4 negative vector "a same-xpub double seating" | FIXED |
| I5 | §6a/§9 name `seal.Classify`; should be `sysw.Classify` | §6a: "The classifier is `sysw.Classify` (`sysw/record.go:100`); `seal.Classify` belongs to the frozen Sealed Payload and is not touched."; §9 item 4 | FIXED |
| I6 | §6a edits a §3.3.2 row that does not exist | §6a: "section 3.3.2 has NO Wallet Policy row today (F-415...), so this cycle CREATES it with all ten cells: Mnem •, Cdx32 •, Passph •, FreeText blank, Descr •, MDMK •, Addr blank, Key •, Hash •, Now •" | FIXED |
| I7 | §4f cites the LOCKED shared origin (`:1359`), not the per-slot path | §4f now rows wsh/sh(wsh)/sh/tr separately: wsh cites `:594-601`, sh(wsh) cites `multisig_build_slots.go:111-130` (the S5 fix), sh cites `multisigScriptTypeComponent`; `:1359` citation removed entirely; `coin'` stated as `0'` and cited | FIXED |
| I8 | automatic `now:` contradicts C24; unscoped (when/position/reproducibility) | §6a: "`me sysw pack` appends `now:` as the LAST record by default; `--no-now` omits it so a fixture's pack output stays a pure function of its inputs (§10 item 2)" | FIXED |
| I9 | below-`now:` refusal undefined when `now:` has no height | §6b: "When `now:` is present its seconds field bounds dates and its height field bounds heights; a field that is absent bounds nothing." | FIXED |
| I10 | §12 vector families miss 5 normative branches (sh wrapper, internal-key state, or_d/or_i head, path count, 4 lock encodings) | §12 item 1 product now includes `wrapper ∈ {wsh,tr,sh,sh(wsh)}`, `path count ∈ {1,2,3,4,8}`, `head ∈ {bare multi, single key, locked}`, `internal key ∈ {extracted first-listed, extracted not first-listed with ≥4 paths, NUMS, none}`, `lock ∈ {none, blocks, 512-s units, height, time}` | FIXED |
| I11 | §8a/§8b copy never states unskippable or the trigger condition | §7b: "both are confirm-to-proceed, neither is dismissible (C16)"; §5a: "the §8b confirm fires ONLY where sorted was legal and declined, never on a lowering-forced `multi`"; §12 item 5 adds fires-on-condition tests for §8a/§8b | FIXED |

### `composer-spec-R0-r0-adversarial.md` (5C/5I — 10/10 FIXED)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C-1 | consent screen can't describe any composed policy; reports 2 paths as 1 | §7e rewritten: consent is "derived from the DECODED md1... through an extended `md.PolicyShape` (§9 item 1) and MUST name, per path in listed order: its k-of-n or single key, its lock kind and value..., its digest..."; device "asserts that the decoded shape equals the composed path list and REFUSES to continue on mismatch"; §9 item 1: "split `or_i`/`or_d` (and `andor`...) into separate `Branch`es... carry the lock operand and the digest" | FIXED |
| C-2 | date before 1985-11-06 silently becomes a block height | same fix as correctness C1 | FIXED |
| C-3 | composed keyless template never required to declare fingerprints | §5 "declarations" row: "EVERY slot declares an origin (§4f) and, when seated, the master fingerprint..."; §12 item 6: "a template with two same-origin slots and no fingerprints is never produced" | FIXED |
| C-4 | card re-mint appends the POLICY stub only; unseatable at restore | §7d: "every seated card is later cut as a RE-MINTED mk1 carrying BOTH the composed template's stub and the composed policy's stub APPENDED to its existing stubs"; §12 item 6 asserts round-trip through `seatKeyCards` | FIXED |
| C-5 | sole hash-gated path admitted, no preimage anywhere, no warning | §4b: "if EVERY path carries `HASH`, the §8h warning fires before consent"; §8h copy added; §7g row "every path hashed → WARNING (§8h)" | FIXED |
| I-1 | key-set table has no row for a non-sole bare unlocked multi-key path | §5 key-set row: "SOLE path, unlocked, unhashed, n ≥ 2: `sortedmulti`...; ANY other multi-key path: `multi`" | FIXED |
| I-2 | tr depth formula stated over paths, ambiguous once internal key is extracted | same fix as correctness I1 (restated over `L`, the leaf list) | FIXED |
| I-3 | §4e admits `sh(pkh(K))`, which §4a forbids | same fix as correctness I2 (§4a/§4e both now require n≥2) | FIXED |
| I-4 | nothing invalidates seating on a shape edit; slots renumber silently | §7d: "Any change to the path list after seating began discards ALL assignments; the operator is told so before the edit is accepted (§8j)"; §7g row added | FIXED (adopted verbatim as the reporter's own suggested minimal fix; brainstorm §3.12 item 5 records the discard-all-with-confirm design) |
| I-5 | §6a specifies only hex-validity; body malformations (hash length, key depth/origin, now: format/duplicates) undefined | §6a "Body validation" paragraph: `hash:` MUST be exactly 32 bytes; `key:` MUST have non-empty origin, xpub depth 3/4, component count = depth; `now:` MUST match `^[0-9]{1,10}(,[0-9]{1,9})?$`, seconds `1..=2147483647`, at most one `now:` record | FIXED |

### `composer-spec-R0-r0-coverage.md` (2C/8I — 9/10 FIXED, 1 PARTIAL)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C-1 | §4f wrong for `sh(wsh)` (says `2'`, shipped code uses `1'`); cites the wrong function | §4f now rows per-wrapper: sh(wsh) → `1'` cited to `multisig_build_slots.go:111-130`; sh → `2'`; wsh → `2'` (`:594-601`); tr → `3'`. `:1359` citation removed. §9 item 8 adds the taproot-arm work item (`multisigScriptTypeComponent`/`md.MultisigScript`) | FIXED |
| C-2 | no §12 item can fail when a refusal fails to refuse (incl. the `older(0x400000)` funds-safety guarantee) | §12 item 4 (whole new negative-vector family, one per §4e/§6a/§6b/§6c/§7d/§7g refusal, explicitly incl. `older(0x400000)` device-side); §12 item 7 (new): "Device-side lock range check is a unit gate on the emitter's input, not on md's acceptance"; §12 item 5 adds fires-on-condition tests for warnings | FIXED |
| I-1 | §6a understates payload-spec changes; 5 sub-gaps; no work item to edit the payload spec | §6a names §3.3.1 (3 new class rows), CREATES the §3.3.2 row with all 10 cells (incl. the previously-omitted FreeText/Addr), names §5.3's prefix list, and names the §3.3.3 flag-screen consequence (F1/F2 firing in the composer's seed step); §10 item 6 (new) owns editing `SPEC_systemwide_payloads.md` | FIXED |
| I-2 | contradicting comment misnamed/misquoted; `sysw_admit.go:47-51` unnamed; D5 unreconciled | §6a: "Two comments are rewritten...: `gui/gui.go:191-203` (full 3-clause quote) and `gui/sysw_admit.go:47-51` ('NO seed class ... least privilege'), which C12 deliberately reverses. Staged-plan D5 stands as history: Wallet Policy remains its own program..." | FIXED |
| I-3 | 9 sections requiring device code (a)-(i) have no §9 item | §9 expanded 9→11 items; every lettered gap now traces: (a)/(b)→item 5, (c)→item 3, (d)→item 9, (e)→item 5, (f)→item 7, (g)→item 9, (h)→items 7/9 (see note below), (i)→item 8 | FIXED — note: item 9's own enumeration names §8l/§8i/§8h/§8a/§8b explicitly but not §8f by number (substantively covered since item 9 says "Composer consent surface (§7e)" and §7e's own text mandates the NUMS line); flagged in §3 below, non-blocking |
| I-4 | C16's "unskippable" dropped everywhere | §7b: "neither is dismissible (C16)"; §4b: "confirm-to-proceed"; §12 item 5 fires-on-condition tests | FIXED |
| I-5 | §12.1 has no path-count axis; central §5 rules untested | §12 item 1: `path count ∈ {1, 2, 3, 4, 8}`; `internal key ∈ {..., extracted not first-listed with ≥ 4 paths, ...}` | FIXED |
| I-6 | 3 record classes have no host/device lockstep acceptance | §12 item 8 (new): "Record classes, lockstep. A cross-language vector set: each `key:`, `hash:`, `now:` record (valid and each §6a malformation) classifies identically on the host and on the device" | FIXED |
| I-7 | §7d seating rules have no acceptance; 3 diverge silently from shipped code | §7d states the composer does NOT call `seatKeyCards` and reconciles "used once" vs. "one card may fill several slots" as coexisting; both-stubs re-mint fixes the data-loss divergence; §12 item 6 gates the round-trip. **Gap:** §12 item 5's fires-on-condition list omits §8k (the C5 informational line) — only §8g (C29) is in the enumerated list | PARTIAL — data-loss and rule-coexistence fixed; the C5/§8k informational-line "fires" gate the finding also asked for is still absent from §12 item 5's list |
| I-8 | §7f engrave surface has no acceptance; unnamed 6d reversal; §14 omits D8 formats | §7f: "This re-opens staged-plan 6d, deferred 2026-08-20 for 'unmeasured sizing...': §13 item 1 measures the sizing and §5 has fixed the content rules; the named backup formats of D8 (BSMS, Nunchuk, Sparrow: staged-plan 6c) stay deferred (§14)"; §14 gains the explicit BSMS/Nunchuk/Sparrow row; §12 item 9 (new) gates the engrave surface | FIXED |

### `composer-spec-R0-r0-journey.md` (6C/10I — 15/16 FIXED, 1 DECLINED-BY-DEFAULT)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C-1 | re-mint appends the POLICY stub only; template refuses cards at restore | same fix as adversarial C-4 | FIXED |
| C-2 | consent surface can't state the shape of anything the composer authors | same fix as adversarial C-1 | FIXED |
| C-3 | per-slot fingerprint declaration unspecified; J2's shared-origin shape unseatable | same fix as adversarial C-3 | FIXED |
| C-4 | hashlock entry never states the 32-byte-preimage rule | §6c: "At entry and at consent the device states the 32-byte rule: `sha256(H)` compiles to `OP_SIZE <32>...`, so the preimage MUST be exactly 32 bytes; a digest of a passphrase directly can never be spent (§8i...)"; §8i copy added | FIXED |
| C-5 | "Back keeps assignments" undefined for a shape-edit renumber; proposed a finer per-(path,key-index) survival rule | §7d: "Any change to the path list after seating began discards ALL assignments... (§8j)" — brainstorm §3.12 item 5 records this as the controller's deliberate, simpler alternative to the finer survival rule this finding proposed | DECLINED-BY-DEFAULT — the underlying wrong-result risk (silent misassignment) is closed by the discard-all default, but not via this finding's specific proposed mechanism |
| C-6 | J3 has no defined slot origin; pathless is refused by the fork's decoder; §12.3 may be unsatisfiable | §4f: "Unseated slots... declare the §4f origin for the wrapper with `account' = the slot's emitted index`, and no fingerprint. A pathless slot is refused... so distinct accounts by slot index are the one form that both decodes and seats"; §12 item 3 updated to require this | FIXED |
| I-1 | engrave-form choice undefined for J3 (no keys/cards) and J1 (`key:`-sourced slots, no card minted) | §7f: "Every seated slot yields a card in form B regardless of source: a `key:` record is MINTED as an mk1...; A keyless composition (no seated slots)... the choice collapses to 'template only'" | FIXED |
| I-2 | device silent when no pack-time bound exists; absence of a line is the only signal | §6b: "When the relevant field is ABSENT the echo carries instead: 'This device cannot tell the time. Nothing here has checked that this is in the future.'" | FIXED |
| I-3 | Build after SKIPping the boot payload silently becomes the keyless journey | §7a: door states key state — "Keys loaded: N" / "No keys loaded..." / "A payload is in flash but not loaded. Load it from the carousel first."; §7g row added | FIXED |
| I-4 | §12.5 omits the modal-fits check; every §8 body is a new modal | §12 item 5: "...AND the modal-fits assertion (`gui/modal_fits_test.go`, `assertModalBodyFits`) on every §8 body and every new screen" | FIXED |
| I-5 | seating pick list has no widget/scroll spec; shipped `ChoiceScreen` doesn't scroll | §9 item 7: "Seating pick list (§7d) as a PAGED widget with stated capacity (the shipped `ChoiceScreen` does not scroll...)" | FIXED |
| I-6 | digit pad can't type `YYYY-MM-DD`; no impossible-date refusal | §6b: "a fixed `YYYYMMDD` field of eight digits echoed as `YYYY-MM-DD`... Impossible dates (2027-02-31) are refused at entry." | FIXED |
| I-7 | no "nothing outside this device has checked this policy" warning on the composer's own authored policy | §7e: "the 'nothing outside this device has checked this policy' warning (§8l), Multisig Build's, unskippable"; §8l copy added | FIXED |
| I-8 | §7c teaches an id the operator can invalidate by Back-editing the shape | §7c: "NOT shape-invariant: a path added or removed changes it, so the screen re-appears after every shape edit and says 'This id changed with the shape.'" | FIXED |
| I-9 | C5's two-keys-two-accounts consequence never reaches the operator during authoring | §7b: live line "slots: N / keys available: M"; §8k copy: "One person in two paths needs two keys: a second account..." | FIXED |
| I-10 | composing a wallet the host already holds yields a different id AND a different address | §4d: "This generalises: the lowering is ONE fixed spelling, so any policy the operator also holds elsewhere in another spelling... is a DIFFERENT wallet with a different id and different addresses. The stub screen says so (§8d)."; §8d copy added | FIXED |

---

## 2. Regressions and dropped content

**One regression: a stale cross-reference introduced by the renumbering.**
§4d (line 139): "are offered as one-tap presets that POPULATE a path list the operator then edits (**§9 item 10**)." §9 item 10 is "Engrave form choice (§7f)..." — presets are actually **§9 item 5** ("Path-list screen with the slots/keys line...; presets populating a path list (§4d); the §4e structural refusals and picker bounds"). This is exactly the "a §N cross-reference pointing at the wrong section now that items were renumbered" class the brief named. Fix: change `§9 item 10` to `§9 item 5` in §4d.

**All other `§9 item N` / `§10 item N` / `§12 item N` cross-references checked and resolve correctly** (13 occurrences total: `§9 item 1` ×3, `§9 item 3`, `§9 item 6`, `§9 item 7`, `§9 item 10` [the one bad one], `§10 item 2`, `§10 item 5`, `§10 item 6`, `§12 item 2`, `§12 item 4`, `§12 item 5`, `§12 item 7` — each checked against the target item's actual text).

**No dropped normative content found.** Diffed §4-§14 old vs. new section by section:
- §4a: the dead "`n ≤ 15` for `sh`" clause was removed — correctly, since three independent Minors (correctness M3, adversarial M-2, coverage N-3) all flagged it as unreachable/misleading dead text superseded by the new n≥2 rule; this is a deliberate response to those findings, not an unexplained drop.
- §9: old items 1-9 all map onto new items (1→1, 2→2, 3→3, 4→4, 5→7, 6→6, 7→10, 8→4 [merged], 9→11); nothing lost, all expanded.
- §10: old items 1-5 map onto new 1-5 unchanged in substance; new item 6 added.
- §14: all 12 old rows present (2 with expanded wording); 2 new rows added (BSMS/Nunchuk/Sparrow; networks other than mainnet). Nothing removed.
- §11, §12, §13: every old sentence/item has a corresponding (usually expanded) new one; no old acceptance item, refusal, or "not verified" entry disappeared.

**No new internal contradiction found** beyond the one cross-reference above. Checked in particular: §4c's "§9 item 3" / "§12 item 7" cross-references (both correct); §7c's "§9 item 6" (correct); §7e's "§9 item 1" (correct); §14's "§9 item 1" andor note (correct); the twelve §7g table rows against their cited sections (all consistent); the §2 ruling→section map (unchanged from before the fold, since §2 was not part of the regenerated range — this means correctness's own N1/N2, about §2 mislabelling C22 and omitting C6/C12→§6a, remain live; see §3 below).

**One completeness nuance, not a regression:** §9 item 9's own enumeration ("...the §8l warning...the §8i line; the §8h all-hashed warning; the §8a/§8b confirms") names five of the twelve §8 copy blocks by letter but not §8f (NUMS note) or §8k (C5 two-accounts line) individually, even though §7e's and §7d's own normative text require rendering them and item 9/7 cover the surfaces that host them. Not a dropped requirement — the underlying copy and its home section are both specified — but the itemization is inconsistent in which callouts it names explicitly. Worth a one-clause tidy at the next fold; not gating.

---

## 3. Minors and Nits, all four reports (not gating; controller folds next)

### correctness

| id | summary | still applicable? | fold suggestion |
| --- | --- | --- | --- |
| M1 | §3's keyless-tap-leaf row cites brainstorm §3.7 (wrong) instead of the review | yes | cite "review I2" only; add "(admitted by `md encode --experimental`)" |
| M2 | §4e's refusal copy claims taproot "cannot" hold a keyless path (false; it's a policy choice) | yes | reword to "This build will not put a key-less path in taproot." |
| M3 | §4a's `n ≤ 15` for `sh` was dead text | no — superseded | none; clause removed in the fold |
| M4 | §12's `sortedmulti`-preset parity item has no premise in §4d | yes | restate C7's condition ("if the composer ever ships a `sortedmulti` preset") since §4d still has none |
| M5 | two §3 citations (`multisig_build_census.go:475`; `wallet_policy.go:35-142`) don't fully carry their claims | yes | narrow the census citation to the Full-label half; extend the wallet_policy.go range to include the consent functions (~148-232) |
| M6 | §5 tr column self-contradicted ("one leaf per path" vs. "then not a leaf") | no — superseded | none; the m=0/1/≥2 rewrite over `L` removed the contradiction |
| M7 | §6b's below-bound refusal names "date" even for a height violation | yes | split the refusal copy by kind ("...Choose a later date/height.") |
| M8 | §7f's "Read-back integrity" implies a device read-back capability that never exists (no camera) | yes | rename, e.g. "recovery-time error detection" |
| M9 | `hash:`'s body is hex-of-bytes while `key:`/`now:` are hex-of-text; lead sentence blurs this | no — fixed | none; body-validation paragraph and per-row wording now state it explicitly |
| M10 | §6a named §3.3.2 but not §3.3.1 | no — fixed | none |
| N1 | `n` overloaded for leaf count vs. key count in the depth formula | no — fixed | none; renamed to `m`/`j` |
| N2 | head-pin drift: descriptor-mnemonic pin vs. locally-installed `md` binary provenance | yes (process note, not spec content) | low priority; note which binary built measurements in a future header |

### adversarial

| id | summary | still applicable? | fold suggestion |
| --- | --- | --- | --- |
| M-1 | "BELOW the `now:` value" didn't say which component a height compares against | no — fixed | none; the bound-line rewrite states it explicitly |
| M-2 | §4a's `n ≤ 15` for `sh` was dead text | no — superseded | none; clause removed |
| N-1 | §4c's `after`-time row range label read "1985-11-05..2038-01-19" (imprecise; floor is 00:53:20) | no — fixed | none; now states "1985-11-05 00:53:20 UTC" |

### coverage

| id | summary | still applicable? | fold suggestion |
| --- | --- | --- | --- |
| M-1 | `coin'` is a variable the device doesn't have; mainnet-only undocumented | no — fixed | none; §4f states `coin' is 0'` and §14 gains a mainnet-only row |
| M-2 | §7c doesn't say what the replacement labels actually become | yes | state the literal new label strings, not just "labelled as different things" |
| M-3 | "path" carries two meanings (spend path / derivation path), never disambiguated | yes | adopt "spend path" consistently per §4's own opening usage |
| M-4 | §12.3 doesn't assert the stub screen fires on the no-payload walk | no — fixed | none; item 3 now names "stub screen with per-slot expected origins" |
| M-5 | `hash:` is a different kind of hex from the other records (duplicate of correctness M9) | no — fixed | none |
| M-6 | F-150 item 1 (Multisig Build's dead-end bug) unreferenced near the deprecation note | yes | one line in §8e/§14 noting the deprecated path stays live and broken |
| M-7 | §10 item 5 silently changes a shipped negative test | yes | name `mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs:174-178` in the item |
| M-8 | §15's gate list omits `plan-table-check.sh`/`plan-stepref-check.sh`/`plan-wiring-check.sh`/`plan-fold-sweep.sh`/`plan-staleness-check.sh` and a baseline rev | partially — `spec-structure-check.sh` was added, the other four gates and the baseline rev are still unnamed | add the remaining gate names and a baseline revision for `plan-staleness-check.sh` |
| M-9 | §7f's "ms1 strings" secret plate form has no citation | yes | cite `gui/codex32_polish.go:218` |
| M-10 | C21's Liana-refuses-`after`/hashlock side finding isn't carried into §13 | yes | one line in §13 constraining F-449's eventual acceptance wallet to `older`-only for Liana |
| M-11 | §9 item 2 (`pk_h` emitter) is an unstated prerequisite for §7e/§12.1 addresses | yes | add "(prerequisite for §7e/§12.1 address derivation)" to item 2 |
| N-1 | §2 labels C22 "adopted" though C23 withdrew it | yes (§2 untouched by this fold) | split the row or add "(withdrawn by C23)" |
| N-2 | §2's mapping omits §6a for C6 and C12 | yes (§2 untouched) | add §6a to both rows |
| N-3 | §4a's `n ≤ 15` inert (duplicate framing of M-2/M3) | no — superseded | none |
| N-4 | §8g's example (`@0`/`@2` in one path) is unreachable under `tr`'s numbering | yes | pick indices that can co-occur, or note it's illustrative only |
| N-5 | C11's timelock-mixing clause is satisfied structurally but never connected in prose | yes | one sentence noting §4b's one-lock-per-path rule is why C11 is discharged |
| N-6 | §7a cites F-437 as though still open; it's RESOLVED | yes | drop the "(F-437)" citation or mark it resolved |

### journey

| id | summary | still applicable? | fold suggestion |
| --- | --- | --- | --- |
| M-1 | `hash:` doesn't follow the §5.3 UTF-8 pattern §6a claims all three follow (duplicate of correctness M9) | no — fixed | none |
| M-2 | §6a named the wrong comment site (`gui.go:191` only) | no — fixed | none; both sites now named |
| M-3 | "byte-identical" used for both a WU-cost claim and an actual-bytes claim | yes | distinguish "witness-byte cost" from "encoded byte identity" in §5a |
| M-4 | §7c's id/stub replacement label unspecified (duplicate of coverage M-2) | yes | same as coverage M-2 |
| M-5 | §8g's example unreachable in `tr` (duplicate of coverage N-4) | yes | same as coverage N-4 |
| M-6 | the typed-64-hex hashlock fallback has no stated length/case validation | yes | state that the typed entry is checked for exactly 64 valid hex chars before accept |
| M-7 | secret-handling: composer holds seeds across more screens than Multisig Build; scrub timing at abandon points unspecified | yes, but **non-gating per the 2026-08-27 severity ruling** | log as a follow-up for future optimization, as the reporter itself proposed |
| M-8 | no total-slot bound stated (72-slot product); pick-list scale unaddressed | partially — §9 item 7 now requires a PAGED widget with stated capacity, but no explicit numeric total-slot bound is stated | state the bound explicitly, or note it's implied by §4b's per-path/path-count product |
| N-1 | §4b's one-lock-per-path rule is *why* C11's mixing rule holds; never connected (duplicate of coverage N-5) | yes | same as coverage N-5 |
| N-2 | §7c's taught command elides the required `--keys`/`--from-md1` flag behind an ellipsis | yes | spell out the flag in the example |

---

## 4. Citation content check — the 13 citations ADDED by the fold

All 13 resolve (per the already-run `plan-cite-check.sh`, 49/49). Content re-checked against the fork at `/scratch/code/shibboleth/seedhammer` (`169073c`):

| citation | claim it supports | cited line, verbatim (trimmed) | supports? |
| --- | --- | --- | --- |
| `gui/gui.go:191-203` | full, unspliced 3-clause quote about the program's origin | "came from OUTSIDE this device"; "...drag a seed requirement or a plate census into a flow that needs neither"; "It is not a rename of Multisig and not an extension of Bundle." | yes — all three clauses present, unlike the old spec's 2-fragment splice |
| `gui/gui.go:1993-2026` | `ChoiceScreen` lays out choices in one stack with no scroll | draw loop accumulates `h += c.Size.Y` over all children, centers with `content.Center(image.Pt(maxW, h))`; no scroll offset or paging logic | yes |
| `gui/key_card_seating.go:28-30` | the consume path's "one card may fill several slots" rule, cited to justify coexistence with the composer's "used once" rule | "ONE CARD MAY FILL SEVERAL SLOTS. A policy can legitimately seat one master at several accounts... What is refused is the reverse: two DIFFERENT cards claiming one slot." | yes, verbatim |
| `gui/multisig_build_slots.go:111-130` | `sh(wsh)` derives at `1'` (the S5 fix), `wsh`/`sh` at `2'` | `multisigScriptTypeComponent`: "if script == md.MultisigShWsh { return 1 } return 2"; comment: "wsh -> 2'... sh(wsh) -> 1'... sh -> 2', because NO BIP assigns legacy P2SH... this device's convention" | yes |
| `gui/multisig_build_slots.go:125-130` | the taproot-arm work item targets "the ONE site that decides" the script-type component | same function; its own comment: "the ONE site that decides it" | yes |
| `gui/policy_address.go:61` | `coin'` is `0'`: mainnet-only by construction | `network := &chaincfg.MainNetParams // D1: mainnet-only.` | yes |
| `gui/sysw_admit.go:26` | `progWalletPolicy` exists, unaccounted for in the payload spec table | line 26 is exactly `progWalletPolicy` | yes |
| `gui/sysw_admit.go:47-51` | the "NO seed class... least privilege" comment C12 reverses | lines 47-51: "NO seed class. The Wallet Policy program never derives from a secret:...Least privilege, and it is enforced here rather than by the flow declining to ask." | yes, verbatim |
| `gui/template_engrave.go:86` | `policySummaryLines` has exactly one call site | line 86: `if summary := policySummaryLines(shape); len(summary) > 0 {` | yes |
| `md/md_test.go:337,416` | the composer's shapes measure `Renderable=false` | line 337: "...the wsh body is and_v(...), outside §4.2 → Renderable=false."; line 416: "...(FOLD A / §4.2) → Renderable=false." | yes, both |
| `md/policy_shape.go:43` | a multi-path wsh script is counted as ONE branch | line 43 `type Branch struct {`, preceding comment: "Branch is one independently satisfiable spend path: a tapscript leaf, or **the whole script for wsh/sh**" | yes |
| `sysw/record.go:100` | `sysw.Classify`'s signature/location | line 100: `func Classify(record string) Class {` | yes |
| `sysw/record.go:15-21` | the three existing reserved prefixes (`text:`, `pass:`, `tx:`) | lines 15-21: `const ( TextPrefix = "text:" ; PassPrefix = "pass:" ; ... TxPrefix = "tx:" )` | yes |

Two citations from the pre-fold spec were **removed**, both correctly: `gui/gui.go:191` (superseded by the fuller `:191-203` range) and `gui/multisig_build.go:1359` (the wrong-function citation identified by correctness I7 / coverage C-1; not replaced in kind — its role is now split across the three new per-wrapper citations above).

---

## 5. What I ran

- Read all four persisted reports in full (`design/agent-reports/composer-spec-R0-r0-{correctness,adversarial,journey,coverage}.md`).
- `git show 80e6a72:design/SPEC_wallet_policy_composer.md` and `git show bc1c07c:design/SPEC_wallet_policy_composer.md` — read both in full (497 and 735 lines).
- Read `design/BRAINSTORM_wallet_policy_composer.md` §3.12 (the controller-defaults record) to distinguish FIXED from DECLINED-BY-DEFAULT.
- `grep`/`sed` diffing of `§4`-`§14` old vs. new, section by section, for dropped normative content (§4a/e/f, §5/5a/5b, §6a/b/c, §7a-g, §8a-l, §9 items 1-11, §10 items 1-6, §11, §12 items 1-11, §13 items 1-5, §14 rows).
- Extracted every `§9 item N` / `§10 item N` / `§12 item N` in-document cross-reference (13 occurrences) and checked each against the target item's actual text — found the one stale reference reported in §2 above.
- Diffed the `file:line` citation set old vs. new (13 added, 2 removed) and read every added citation's cited lines against the fork at `/scratch/code/shibboleth/seedhammer` (`169073c`) to confirm content, not just existence (the existence gate, `plan-cite-check.sh` 49/49, was already run and is not re-run here).
- Did **not** re-run `spec-structure-check.sh`, `plan-glyph-check.sh`, or `plan-cite-check.sh` (already run and clean per the fold commit message; taken as given per the brief).
- Did not re-derive the 29 operator rulings, the §5 lowering rules' correctness, or any brainstorm §3.12 controller-default judgment call itself — only whether the spec text implements what §3.12 says was decided.
