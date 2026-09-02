# R2 — mechanical fold-verification of `4724aea` against the three R0 round-1 reports

**VERDICT: 30 FIXED / 3 PARTIAL / 2 NOT FIXED / 1 DECLINED-BY-DEFAULT, out of 36
Critical+Important+regression/PARTIAL items reviewed; 27 round-0/round-1 Minors
carried forward, of which 23 FIXED / 4 NOT FIXED (2 of the 4 are the same
underlying gap cited twice, in `correctness` and `coverage`). 3 NEW defects found
in this fold's own additions (not claimed as fixed by any round-1 report): a stale
table-cell quote, a fires-on-condition-list omission for the two letters this fold
itself created, and two refusal strings left outside the glyph/fits gate the fold
otherwise closed for this exact class of defect.**

Scope: did fold commit `4724aea` fix every C/I/regression/PARTIAL in the three
persisted round-1 reports, and did it introduce a new defect. Brainstorm §2
rulings (C1-C29) and brainstorm §3.12 controller defaults (items 1-16, including
10-16 recorded for this round) are treated as final and not re-litigated. Round-0
Minors/Nits are graded only where the r1 fold-verification report (`aa022ae`)
listed them "still applicable"; its own regression and PARTIAL are graded fresh.

---

## 1. Per-finding table

### A. `composer-spec-R0-r1-fold-verification.md` — its own regression + PARTIAL

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| regression | §4d cited `§9 item 10` (should be `§9 item 5`) | §4d now reads "...the operator then edits (§9 item 5), plus a sixth, **plain k-of-n multisig**..." | FIXED |
| coverage I-7 (PARTIAL) | §12 item 5's fires-on-condition list omitted §8k | §12 item 5 now lists "§8a, §8b, §8f, §8g, §8h, §8j, **§8k**, §8l, §8o, §8p, §8q, §8r, §8s" | FIXED |

### B. `composer-spec-R0-r1-journey2.md` (1C/9I)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C-1 | discard-on-edit scoped to "path list", not the wrapper; §4e's own refusal routes the operator into the uncovered edit | §7d: "Any change to the SHAPE, wrapper included, after at least one slot has been assigned discards ALL assignments"; §7g row "edits the shape (paths OR wrapper) after a slot was assigned"; §12 item 4 adds "a wrapper change after a slot was assigned" | FIXED |
| I-1 | §7c states a per-slot expected origin unconditionally; next step (seating) overwrites it | §7c: "The per-slot 'expects a key at' line is shown ONLY for slots that will stay unseated...; a slot seated from a record, card or seed carries that source's declared origin instead, and the line for it reads 'Slot @i: <fingerprint> <origin>' once seated" | FIXED |
| I-2 | §7c screen body grows with slot count; §12.5's fits assertion can't pin a variable body | §7c: "The screen is a PAGED widget: the fixed header...on the first frame, then slot lines at a stated per-frame budget with a pager"; §12 item 5: "the variable-length screens (§7c stub screen, §7e consent, §7d pick list) are asserted by PAGING capacity...since a fits assertion cannot pin a body with no single source string" | FIXED |
| I-3 | composer's consent surface has no stated widget | §7e: "The surface is `confirmReviewScreen`'s PAGED form (`gui/multisig_build.go:1908-1931`)...; eight paths plus four addresses do not fit one frame"; §9 item 9 updated to name the paged form | FIXED |
| I-4 | malformed record reaches no screen; §12.8 passes either way | §6a: new "Where the refusals live" paragraph — host `pack_with` refuses by index with §8n lines; device leaves it INERT per the shipped contract, "the door's...count...is the device's only signal and the spec says so rather than promising a screen"; §12 item 8: "for each malformation the host emits its §8n line and the device leaves the record inert with the door's count reduced by one" | FIXED |
| I-5 | seating-shortfall refusal names ONE cause for several conditions | §7d: count line always ("N slots, M keys available"), unfilled slots named, "the C5 cause line ONLY when a fingerprint the payload already holds appears in two paths of the composed shape" | FIXED |
| I-6 | "seed fills several slots" vs. "used at most once" contradiction; §12.10 needs the forbidden reading | §7d: "'Used at most once' governs `key:` records and mk1 cards...; a SEED is a source of as many slots as the operator assigns to it...so the shortfall test below counts ASSIGNABLE slots, not sources" | FIXED |
| I-7 | hashlock pick list has no specified row content; 64-hex can't be drawn in a row | §6c: "each row `hash <i>  <first 8>..<last 8>` in the host's pack order (28 characters, inside the 436 px label budget...)" | FIXED |
| I-8 | self-check refusal names no action; §12 item 4 can't construct it from an input | §7e: REFUSES with §8q ("...Go back and check the path list, or start again."); "this refusal is provoked by fault injection, not by an input (§12 item 4)"; §12 item 4: "exercised by FAULT INJECTION (flip one builder output, assert §8q fires), not by an input" | FIXED |
| I-9 | operator-facing refusal copy sits outside the gate §11 claims covers it (glyph gate's blind spot: prose-quoted strings) | §4e's 5 refusals → §8m lines 1-5; §7a's 3 door lines → §8r; §7c's re-show line, §7d's slot prompts → §8s; §7d's shortfall lines → §8p; §7e's self-check → §8q; host record refusals → §8n. **But** two §6b refusal strings remain bare prose quotes, uncited to any §8 letter and unlisted in §12 item 5's fires-on-condition enumeration: "Dates before 2009 cannot be written as a time lock." and "Relative locks reach at most 455 days in blocks or 388 days in time. Use an absolute date." (both verified present, verbatim, at `design/SPEC_wallet_policy_composer.md` §6b, lines ~322-323 and ~335-336 of the folded file). These are exactly the class `plan-glyph-check.sh` cannot see (it scans blockquotes and 40+-char backtick spans only) — the same blind spot I-9 named. | PARTIAL — the fix pattern (promote to §8, cite by letter) was applied to every refusal EXCEPT these two, which are in the same section (§6b) whose other refusals (the bound-line pair) were correctly promoted to §8o |

### C. `composer-spec-R0-r1-feasibility.md` (1C/4I)

| id | title | fold's response | verdict |
| --- | --- | --- | --- |
| C-1 | 32-slot cap is policy-wide, not per-fragment; §4b/§4e admit up to 72 slots | §4b: "the policy's TOTAL slot count is 1..=32 (the wire's 5-bit `path_decl.n`, `md/md.go:215-221`, `crates/md-codec/src/error.rs:57-59`)"; §4e: new row "...or a 33rd slot → REFUSE at the picker...; the slot cap says §8m line 5"; §8m line 5 "This wallet already has 32 key slots."; §12 item 1's path-count axis replaced with "1, 2, 3, 4 and the 32-slot maximum" | FIXED |
| I-1 | §9 item 1 names the single-string serialiser; every consumer rejects its output | §9 item 1: "emit keyless and keyed md1 in CHUNK form through the existing `split` (`md/chunk.go:121`), the artifact every consumer accepts (`encodeMD1String`'s single-string form is rejected downstream...)" | FIXED |
| I-2 | §12 item 1's "byte for byte" doesn't name the wire form; single vs. chunk differ | §12 item 1: "the Go builder reproduces every template, every CHUNK and every address byte for byte; a separate named leg compares the single-string payload bytes" | FIXED |
| I-3 | §6a assigns a payload-cardinality rule to a per-record classifier; no home; §12.8 can't cover it | §6a: "The payload-wide rule 'at most ONE `now:` record' is enforced at the two sites that see the whole payload: host `pack_with` and device `syswSession.load` (`gui/sysw_session.go:80`)"; §12 item 4 moved the "two `now:` records" vector out of item 8 into item 4 | FIXED |
| I-4 | §12 item 1's vector product is 28,800 cells (57,600 with origin axis); unbuildable | §12 item 1 rewritten: "TAGGED COVERAGE, not a product...a script asserts each tag appears in at least TWO vectors...a pairwise covering array...needs about 50-60 named vectors," with the required-tags list given | FIXED |

### D. Minors/Nits carried from `composer-spec-R0-r1-journey2.md` (still-applicable set, 8 items)

| id | summary | fold's response | verdict |
| --- | --- | --- | --- |
| M-1 | door offers "From payload" for a payload with no policy | §7a: "'From payload' (only when the loaded payload holds a Descriptor or md1/mk1 record)" | FIXED |
| M-2 | §8j's copy false when nothing was seated; "after seating began" undefined | §7d: "With no slot yet assigned there is nothing to discard and §8j does not fire"; §8j heading renamed "Shape edit after at least one slot was assigned" | FIXED |
| M-3 | preset availability per wrapper unspecified | §4d: "presets are offered under `wsh` and `tr`; under `sh`/`sh(wsh)` only the plain k-of-n preset is offered" | FIXED |
| M-4 | seating prompt undefined for the extracted internal-key slot | §7d: "The prompt for an extracted internal-key slot reads 'Slot @0, key path (spends alone): choose a key'" (§8s) | FIXED |
| M-5 | "This id changed with the shape." names the change, not the consequence — needs a clause that old-stub cards won't seat | No such clause added anywhere in §7c, §7d, §7g, or §8s. Searched the whole folded spec for "old stub" / "minted with" / "seat into this template" / "permutation" — zero hits outside the unrelated §8b rewrite. | **NOT FIXED** |
| M-6 | §8b warns about a cost §5's declarations removed; stays silent where key order really matters (tiers forced to `multi`, non-sole) | §8b reworded: "Key order is part of this wallet. Anyone restoring it must keep the same order. Sorted keys need none." — drops the false "permutation search" claim, but does not distinguish third-party/descriptor-text restore (where order still matters) from restore via this composer's own seated cards (where `slotMatchesCard` makes order irrelevant); §5a's "fires ONLY where sorted was legal and declined" sentence is byte-for-byte unchanged from before this fold, so the silent-non-sole-multi case the finding named is still silent | PARTIAL |
| M-7 | secret-handling scrub timing across composer screens, non-gating | §14: "scrub timing at every abandon point of the composer's seed screens \| secret-handling, non-gating by the 2026-08-27 ruling; filed as a follow-up for optimisation" | FIXED |
| M-8 | no total-slot bound stated (72-slot product) | Superseded by C-1's fix: the grammar itself now caps total slots at 32 (§4b, §4e, §8m line 5) | FIXED |
| N-1 | C11's mixing rule never connected in prose to why it holds | §4 opening: "One lock per spend path (§4b) is what discharges C11's mixing rule: a height lock and a time lock can never meet inside one `and_v` chain." | FIXED |
| N-2 | §7c's taught command elides `--keys`/`--from-md1` behind an ellipsis | §7c blockquote spells out `mk encode --xpub <xpub> --origin-fingerprint <fp> --origin-path <path> --policy-id-stub <8 hex>`, no ellipsis | FIXED |

*(9 rows above; M-3 of the fold-verification's own journey table — "'byte-identical' used for both a WU-cost and an actual-bytes claim" — is graded under §E below since it is identical to a correctness-report row already tracked there.)*

### E. Minors/Nits carried from `composer-spec-R0-r1-fold-verification.md` §3 (round-0 reports, "still applicable" rows)

| report.id | summary | verdict | evidence |
| --- | --- | --- | --- |
| correctness M1 | wrong citation for keyless-tap-leaf row | FIXED | §3: "md admission: keyless tap leaf \| refused (...) \| first review, I2" (brainstorm §3.7 citation dropped) |
| correctness M2 | §4e claims taproot "cannot" hold a keyless path (false) | FIXED | §8m: "This build will not put a key-less path in taproot." |
| correctness M4 | §12's sortedmulti-preset-parity item has no premise in §4d | FIXED | §4d now ships a 6th preset (plain k-of-n `sortedmulti`, controller default 10); §12 item 10 relabelled "Multisig Build parity (optional, C7 comment-only)" |
| correctness M5 | two §3 citations don't fully carry their claims | FIXED | `gui/wallet_policy.go:35-300` (was `:35-142`); `gui/multisig_build_census.go:475 (the Full label)` |
| correctness M7 | §6b's below-bound refusal names "date" even for a height violation | FIXED | §8o split into a date body and a height body |
| correctness M8 | §15's gate list omits 4 script names + a baseline rev | **NOT FIXED** | §15 (unchanged this fold) still lists only `plan-build-gate.sh`, `plan-build-gate-go.sh`, `plan-cite-check.sh`, `plan-glyph-check.sh`, `spec-structure-check.sh` — `plan-table-check.sh`, `plan-stepref-check.sh`, `plan-wiring-check.sh`, `plan-fold-sweep.sh`, `plan-staleness-check.sh` and a baseline rev are still absent |
| correctness N2 | head-pin drift: which binary built the measurements is unrecorded | **NOT FIXED** (low priority, Nit) | Header's repo/head table unchanged in kind; no note on binary provenance added |
| adversarial | (all previously "still applicable" rows were already superseded/fixed at r1; none carried) | — | — |
| coverage M-2 | §7c doesn't state the literal replacement labels | FIXED | §7c: "Labels, literally: `Template-ID:` and `Policy-ID:`...; `mk1 stub (template):` and `mk1 stub (policy):`..." |
| coverage M-3 | "path" carries two meanings, never disambiguated | FIXED | §4 opening: "'Path' below always means a spend path; a derivation path is always called an origin." |
| coverage M-6 | F-150 item 1 unreferenced near the deprecation note | FIXED | §14: "...its dead-end (F-150 item 1) stays as filed and is not fixed by this cycle" |
| coverage M-7 | §10 item 5 silently changes a shipped negative test | FIXED | §10 item 5 names `crates/ms-cli/tests/cli_derive_bip48.rs:174-178`, "renamed, not deleted" |
| coverage M-8 | (duplicate of correctness M8) | **NOT FIXED** | same gap |
| coverage M-9 | §7f's "ms1 strings" secret plate form has no citation | **NOT FIXED** | Searched folded spec for "codex32_polish" — zero hits; §7f's "ms1 strings" clause (line ~485) still uncited |
| coverage M-10 | C21's Liana-refuses-`after`/hashlock finding not carried into §13 | FIXED | §13 item 4: "Liana's import refuses any `after` or hashlock path regardless of head (second lowering review), so F-449's acceptance wallet must be `older`-only" — content verified verbatim against C21's own text in the brainstorm record |
| coverage M-11 | §9 item 2 (`pk_h` emitter) unstated prerequisite | FIXED | §9 item 2: "...a prerequisite for every §7e and §12 item 1 address of a policy with a single-key wsh path" |
| coverage N-1 | §2 labels C22 "adopted" though C23 withdrew it | FIXED | §2: "C19-C23 \| review findings adopted (C22 withdrawn by C23)..." |
| coverage N-2 | §2 omits §6a for C6/C12 | FIXED | §2: C6 → "§6a, §7a"; C12 → "§6a, §7d" |
| coverage N-4 | §8g's example (`@0`/`@2`) unreachable under `tr`'s numbering | FIXED | §8g: "Slots @1 and @2 are the same seed." |
| coverage N-5 | C11's mixing rule never connected in prose (dup of journey N-1) | FIXED | same §4-opening sentence as journey N-1 above |
| coverage N-6 | §7a cites F-437 as open though it's resolved | FIXED | §7a: "(F-437, resolved)" |
| journey M-3 | "byte-identical" conflates WU-cost and encoded-byte claims | FIXED | §5a: "'Byte-identical' elsewhere in this spec means the ENCODED artifact, chunk by chunk" |
| journey M-4 | (duplicate of coverage M-2) | FIXED | same fix |
| journey M-5 | (duplicate of coverage N-4) | FIXED | same fix |
| journey M-6 | typed-64-hex hashlock fallback has no length/case validation | FIXED | §6c: "accepted only when exactly 64 valid hex characters are present" |
| journey M-7 | secret-handling: composer holds seeds across more screens, non-gating | FIXED | §14 follow-up row (same as §D M-7 above, this is the same underlying finding) |
| journey M-8 | no total-slot bound stated | FIXED | superseded by the 32-slot cap (as §D M-8 above) |
| journey N-1 | (duplicate of coverage N-5) | FIXED | same fix |
| journey N-2 | §7c's taught command elides a flag | FIXED | as §D N-2 above |

**F. Feasibility M/N (repeated here for completeness of the three-report scope):** M-1
graded DECLINED-BY-DEFAULT per the brief (brainstorm §3.12 item 16: `md decompose`
cited in place of the nonexistent `--keys` flag). M-2 through M-6 all FIXED (verified
in §C's table context and individually: §10 item 1 states `compose` is
unconditional; divergent origins written inline; §7e's overbroad claim narrowed to
"multi-path or taproot"; §4e/§8m note that legacy wrappers are sorted-only so §8b
never fires there; §6a's `now:` height bound tightened to `1..=499,999,999`). N-1
and N-2 required no spec change (informational); N-2's flash/RAM figures were in
fact added to §3's inventory table as a new row, beyond what the finding asked.

---

## 2. Regressions

**Three new defects, all introduced by this fold's own additions — none claimed
fixed by any round-1 report, and none present before this fold.**

1. **Stale table-cell quote.** §7g's divergences table still reads: `| stub
   screen | operator wrote the stub down, then edits the shape | DEFAULT: screen
   re-shown, "This id changed with the shape." (§7c) |` — but this fold reworded
   the actual copy, in both §7c's prose and the new §8s blockquote, to "**The
   shape changed, so this id changed.**" The table cell was not updated in the
   same change and now quotes text that appears nowhere else in the document.
   This is exactly the "table cell disagrees with prose" class the brief asked
   for. Fix: update the §7g cell to match §8s, and cite it as `(§8s)` rather than
   `(§7c)` for consistency with the rest of the table's post-fold citation style.

2. **§12 item 5's fires-on-condition list omits the two letters this very fold
   created.** The list now reads "§8a, §8b, §8f, §8g, §8h, §8j, §8k, §8l, §8o,
   §8p, §8q, §8r, §8s" — every pre-existing letter, plus every NEW letter from
   this fold EXCEPT §8m (the five §4e structural refusals, load-bearing and
   cited by "§8m line N" five times in §4e's own table) and §8n (the four
   host-side record-refusal lines newly added to close journey2 I-4). Both are
   fixed, high-traffic strings that should have a fires-on-condition test exactly
   like their siblings §8o/§8p/§8q added in the same diff. Fix: add "§8m, §8n"
   to the list.

3. **Two §6b refusal strings remain outside the gate the fold otherwise closed
   for this class.** See journey2 I-9 in the table above (§1B) — "Dates before
   2009 cannot be written as a time lock." and "Relative locks reach at most 455
   days in blocks or 388 days in time. Use an absolute date." are both bare
   prose quotes in §6b, never promoted to a §8 letter, and absent from §12 item
   5's enumeration. Grouped here as well because it is the same defect shape as
   finding 2 above — an acceptance/gate list one step behind a copy change made
   in the same fold.

**No dropped normative content found.** Diffed §2-§15 old (`bc1c07c`) vs. new
(`4724aea`) section by section: every §9/§10/§12 item count is unchanged (§9
still 11 items, §10 still 6, §12 still 11); items 1, 2, 6, 9 of §9 and items 1,
4, 5, 8 of §12 were revised in place, none removed; §13's 5 items and §14's row
count are both unchanged in count (content extended, not cut); the new §8m-§8s
blockquotes are net additions. The one prior-round dead clause already removed
(the `sh` "n ≤ 15" text) was removed at `bc1c07c`, not touched again here.

**No stale cross-reference found.** Extracted every `§9 item N` / `§10 item N` /
`§12 item N` occurrence in the folded document (14 total) and checked each
against its target item's actual current content — all 14 resolve correctly,
including the four references to items whose CONTENT changed this round (§9
item 1, referenced at lines ~458 and ~864; §9 item 6, referenced at ~385 and
~394; §12 item 1, referenced at ~690 and ~731; §12 item 4, referenced at ~469
and ~755). The one stale reference from the prior round (§4d's `§9 item 10`) is
the regression fixed in §1A above.

**No new table/prose value contradiction found** among the four rule classes
the brief named: the 32-slot cap (32 stated identically in §3's inventory row,
§4b, §4e, §8m line 5, and §12 item 1); the date floor (2009-01-03 entry floor /
500,000,000 operand floor stated identically in §4c and §6b); the `now:` height
bound (`1..=499,999,999` stated identically in §4c, §6a and §6b's entry table);
and the discard-on-edit trigger ("any change to the SHAPE, wrapper included,
after at least one slot has been assigned" stated identically in §7d, §7g,
§8j's heading, and §12 item 4). The three regressions above are copy-string and
gate-list omissions, not numeric-rule contradictions.

---

## 3. Citation content — the 9 citations ADDED by this fold

Diffed the `file:line` set of `bc1c07c` vs. `4724aea` (regex over backticked
`path:line` spans): 9 added, 2 removed (`gui/wallet_policy.go:194`, superseded
by the widened `:35-300`; `gui/wallet_policy.go:35-142`, widened to `:35-300` —
both correctly retired, not orphaning any claim). All 9 new citations read
against the real source, not the report's transcription:

| citation | claim it supports | read at the source | supports? |
| --- | --- | --- | --- |
| `crates/md-codec/src/error.rs:57-59` (descriptor-mnemonic `3b0944fb`) | TOTAL slot count 1..=32 is a wire limit, not a per-fragment one | Lines 57-59: `/// Key count `n` out of range. Per SPEC v0.30 §4: `1 ≤ n ≤ 32`.` / `#[error(...)]` / `KeyCountOutOfRange { n: u8 }` | yes |
| `md/md.go:215-221` (fork `169073c`) | the wire's 5-bit `path_decl.n` field is the source of the 32-slot cap | Lines 215-221: comment "readPathDecl: n = read(5)+1..." through `func readPathDecl(...)` and `n := uint8(raw) + 1` | yes |
| `md/chunk.go:121` (fork) | `split` is the chunk-form emit path the composer should use | Line 121: `func split(d *descriptor) ([]string, error) {`, preceded by a comment describing exactly the chunk-header + payload-slice mechanism the spec attributes to it | yes |
| `sysw/descriptor.go:46-48` (fork) | shipped contract for an unclassifiable record: "stays in the session, is offered to nobody, and reaches no screen" | Lines 46-48, verbatim: "A record failing any of it is ClassUnknown and goes INERT -- the existing contract for an unclassifiable record (it stays in the session, is offered to nobody, and reaches no screen)." | yes, verbatim |
| `gui/sysw_session.go:80` (fork) | `syswSession.load` is the device-side site that sees the whole payload, where the single-`now:` rule is enforced | Line 80: `func (s *syswSession) load(p *sysw.Payload, identity [32]byte, sealed, cliffAbove, compared, digestShown bool) {` — takes the whole `*sysw.Payload`, confirming it is the whole-payload site; the rule itself is prospective device work (§9), not yet implemented at this line, consistent with how the spec cites other not-yet-built work-item sites | yes (structural-site claim; not a claim that the rule is already coded there) |
| `crates/me-cli/src/sysw/mod.rs:288` (this repo, `4c863a94`) | `pack_with` "already walks the whole record vector and refuses an unclassifiable record by index" | Line 288 is `pub fn pack_with(` — the function's signature. The actual per-index walk and refusal (`for (i, r) in records.iter().enumerate() { if ... Unknown { return Err(SyswError::Unclassifiable(i, ...)) } }`) lives in `admit_check` at line 416, reached transitively via `pack_with → pack_deterministic_with → split → admit_check`. The underlying claim is TRUE (traced the whole call chain), but the cited line is the entry point's signature, not the mechanism. | weak — supports the claim only transitively; a tighter citation would be `crates/me-cli/src/sysw/mod.rs:416-422` (`admit_check`) |
| `crates/ms-cli/tests/cli_derive_bip48.rs:174-178` (mnemonic-secret `5f37b43`) | this cycle "flips" a shipped negative test asserting `bip48-p2tr` is refused | Lines 174-178: `let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2tr"]); assert_ne!(code(&o), 0, "bip48-p2tr must be refused; stdout={}", out(&o));` inside `fn an_unregistered_script_type_is_refused()` | yes |
| `gui/multisig_build.go:1908-1931` (fork) | `confirmReviewScreen`'s pager draws only when a second page exists | Line 1877 `func confirmReviewScreen(...)`; line 1917 comment "The pager is drawn ONLY when there is a second page..."; line 1929 `if start > 0 \|\| shown < len(lines) {` (both inside the cited range) | yes |
| `gui/wallet_policy.go:35-300` (fork, widened from `:35-142`) | evidence for the shipped "gather → consent...→ review → engrave" capability row | File is 368 lines total; the widened range now spans `walletPolicyFlow` (35), `walletPolicyMd1` (143), `walletPolicyConsentLines` (163), `walletPolicyAddressLines` (250) and into `policyAddressAt` (291-300) — covers gather/consent/addresses, matching M5's ask to extend past the old `:35-142` cutoff | yes |

**8 of 9 solidly support their claim; 1 (`sysw/mod.rs:288`) supports only
transitively** — it names the right function but not the line where the claimed
behavior (per-index refusal) actually executes. Not gating (the underlying fact
is true and independently confirmed by tracing the call chain), but worth a
one-line tightening at the next fold.

---

## 4. Copy check — the new §8m-§8s blockquotes

Measured every body's line count and per-line non-whitespace character length
(the same unit the modal-fits gate uses), and scanned for non-ASCII bytes.

**Line-count and ASCII: clean.** All 31 blockquote bodies across §8m-§8s are
1-3 lines (never over the 4-line budget), and every line is pure ASCII.

**Character budget: 2 of 31 lines exceed 50 characters, both in §8n.**

| letter | line | length | over? |
| --- | --- | --- | --- |
| 8n | `record N: hash: must be exactly 64 hex characters` | 49 | no |
| 8n | `record N: now: must be <seconds>[,<height>] in range` | **52** | **yes, +2** |
| 8n | `record N: a second now: record; only one is allowed` | **51** | **yes, +1** |

All other lines across 8m/8o/8p/8q/8r/8s are ≤ 48 characters. The two
over-budget lines are both §8n — the HOST-side `me sysw pack` CLI refusal
lines, which §6a's own text says are never rendered on the device panel at
all ("on the DEVICE a record that fails classification goes INERT..."). So the
~50-char / panel-width budget that governs the on-device §8 bodies may not be
the right constraint for §8n's terminal-only strings in the first place; either
the two lines should be shortened to respect the same discipline as every other
§8 body (for consistency, since §8's own header claims "every FIXED body passes
the modal-fits assertion" with no carve-out for §8n), or §8n's header should
state that it is exempt from the panel-width budget because it never reaches
the panel. As written, the two lines are inconsistent with the header's blanket
claim. Not gating (these are Rust CLI stderr strings, not a device rendering
defect — no panel ever has to draw them) but worth a one-line fix or
clarification.

**Refusal-to-§8-letter citation check (§4e/§6a/§6b/§7d/§7e):**

- §4e — all 5 rows cite `§8m line N` (N=1..5, verified each points to the
  correct body). Complete.
- §6a — cites `§8n` for the host per-failure lines. Complete for the
  classification-failure family; the payload-wide "two `now:` records" case is
  described in prose and cross-referenced to §12 item 4, not quoted as a
  literal string, so there is nothing to promote there.
- §6b — **incomplete**: the bound-line pair correctly cites `§8o`, but two
  other refusals in the same section ("Dates before 2009...", "Relative locks
  reach at most 455 days...") remain bare prose quotes with no §8 letter. See
  Regression 3 / journey2 I-9 PARTIAL above.
- §7d — cites `§8p` (shortfall) and `§8s` (slot prompts). Complete.
- §7e — cites `§8q` (self-check) and `§8l` (the reused Multisig Build warning).
  Complete.

---

## 5. What I ran

- Read all three round-1 reports in full (`design/agent-reports/composer-spec-R0-r1-{fold-verification,journey2,feasibility}.md`).
- `git show bc1c07c:design/SPEC_wallet_policy_composer.md` (735 lines) and
  `git show 4724aea:design/SPEC_wallet_policy_composer.md` (876 lines, ==
  working tree) — read both in full; `git diff bc1c07c..4724aea --
  design/SPEC_wallet_policy_composer.md` (634 lines) read in full.
- Read `design/BRAINSTORM_wallet_policy_composer.md` §3.12 (items 1-16,
  including the round-1 controller defaults 10-16) to distinguish FIXED from
  DECLINED-BY-DEFAULT, and to verify C21's own text against the new §13 item 4
  citation.
- Regex-diffed the `` `path:line` `` citation set old vs. new (9 added, 2
  removed) and read every added citation's cited lines at the real source:
  fork `bg002h/seedhammer` at `169073c` (`/scratch/code/shibboleth/seedhammer`),
  `descriptor-mnemonic` at `3b0944fb`, `mnemonic-secret` at `5f37b43`, and this
  repo's `crates/me-cli/src/sysw/mod.rs` at `4c863a94` — including tracing the
  `pack_with → pack_deterministic_with → split → admit_check` call chain to
  check the one transitively-supported citation.
- Extracted and measured every §8m-§8s blockquote body: line count, per-line
  non-whitespace character length, and a non-ASCII byte scan (awk script over
  the section boundaries).
- Extracted all 14 in-document `§9 item N` / `§10 item N` / `§12 item N`
  cross-references and checked each against its target item's current content.
- Grepped the whole folded document for every literal quoted refusal string
  named in journey2's I-9 finding and in the brief's four named "stated twice"
  rules (32-slot cap, date floor, `now:` height bound, discard-on-edit
  trigger), and cross-checked table cells against prose for each.
- Grepped for the coverage M-9 (`codex32_polish`) and journey M-5 ("cards
  minted with the old stub") suggested fixes to confirm their absence.
- Did **not** re-run `spec-structure-check.sh`, `plan-glyph-check.sh`, or
  `plan-cite-check.sh` — clean per the fold commit message (87 strings / 0
  undrawable; 61/61 citations resolve) and taken as given per the brief.
- Did not re-derive the 29 operator rulings, brainstorm §3.12 controller
  defaults 1-16, or the correctness of any judgment call itself — only whether
  the spec text implements what round-1's reports asked for and what §3.12
  records as decided.
