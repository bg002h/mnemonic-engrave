# F-449 spec r5 — final mechanical check (before implementation)

**Verdict: 0 unaddressed / 0 residue defects / 1 imprecise citation (non-blocking).**

Purely mechanical. Diff reviewed: `git diff 874317ae..5ba10a48 -- design/SPEC_liana_unspendable_internal_key.md`
(54 insertions, 20 deletions, single commit `5ba10a48`). Artifact read in full at
r5 (845 lines). Prior report: `design/agent-reports/f449-spec-r4-closing.md`
(0C/3I/5M — I-A, I-B, I-C, M-a..M-e).

---

## Q1 — did the fold land?

| id | verdict | note |
| --- | --- | --- |
| I-A (§8.9's §0b rows stated retired rules) | **ADDRESSED** | All three sub-rows rewritten: PLACEMENT now adds "and before `composerTemplateChunksFor`" (line 719); RESET now states the predicate-reevaluation rule plus the converse assertion that catches an over-eager reset (line 720); DEFAULT ROW now has the two-assertion form covering first-entry AND re-entry (line 721). |
| I-B (§8's renumbering broke stage 1b's gate, orphaned mutation testing) | **ADDRESSED** | Stage 1b's gate now reads "§8 vectors 1, 2, 5, 6, 7, **10**" with an explanatory parenthetical (line 779); the owning-stage table now reads "…**10 mutation**" and adds a new row "**9 operator-facing gate table** \| each row carries its own owning stage (1b, 2, 3, 4, 4a)" (lines 794-795). |
| I-C (RESET hook sits after the template is built) | **ADDRESSED** | §0b's RESET paragraph now hooks "immediately before `composerTemplateChunksFor` (`gui/composer_flow.go:98`)" (line 99-103), with a new paragraph explaining how the predicate gets a shape before the chunks exist (lines 105-113) and an explicit note that r4's `composerStubFlow` hook was wrong (lines 115-119). §8.9's RESET row matches (line 720). |
| M-a (PLACEMENT ordering inverted, not answered) | **ADDRESSED** | New paragraph (lines 105-113) explains `composerTemplateChunksFor` is pure and the screen can call it itself to get a `PolicyShape` before the chunks are built. |
| M-b (RESET rationale wrong for conjunct 2; drop is silent) | **ADDRESSED** | New paragraph (lines 124-133) distinguishes conjunct-1 unrepresentability from conjunct-2 usefulness, and states the drop "must be **signalled**, not silent," naming `composerStubDelta`'s insufficiency. (Scheduling a signalling mechanism, if any, is a design question, out of scope for this mechanical check — the finding text itself is now present.) |
| M-c (§6a row 1: "hence stage 3, not a `gui/` one" inaccurate) | **ADDRESSED** | Row rewritten: "the sentinel is a `md/` change (stage 3); `gatherIgnored` itself is `gui/` (…), so BOTH packages have work at that stage" (line 585). |
| M-d ("too damaged" survives in §8.9's `md repair` row) | **ADDRESSED** | Row rewritten to "KEEPS the correction and reports an unsupported wire version, distinctly from the atomic-fail exit" (line 724) — no "too damaged" phrase. |
| M-e (§9a undercounts; stage 4a content column drops `bundle.rs:371`) | **ADDRESSED** | Heading now "Stage 4a is **four** pieces of work" (line 809); stage 4a's content column now lists "§9a's four pieces … and **§8b's fail-open fix at `bundle.rs:371`**" (line 783); §9a's list item 3 explicitly adds "§8b's fail-open at `bundle.rs:371` … Filed as F-635" (lines 825-826). |

**0 of 8 unaddressed.**

---

## Q2 — remaining propagation residue

Grepped the whole file for each named phrase/pattern:

- `"too damaged"` — **0 hits.** Not present anywhere in the spec.
- r3's blanket RESET rule (`"resets on any shape, wrapper or path-list edit"`) — **1 hit**, at line 94, inside the sentence *"r3 said the kind 'resets on any shape, wrapper or path-list edit that re-enters `composerShapeFlow`', which is a granularity the composer cannot observe"* — explicitly attributed to r3 and immediately refuted in the same paragraph. Not residue.
- r3 DEFAULT ROW rule without the re-entry clause — the live DEFAULT ROW ruling (line 139) reads *"`Initial` is the NUMS row **on first entry only**; on any re-entry it is the kind currently set"* — the re-entry clause is present. Not residue.
- `"one choice screen is the minimum"` — **1 hit**, line 663: *"r1 asserted one choice screen was 'the minimum that makes §0 true'. It was not…"* — attributed to r1, refuted. Not residue.
- The old inverted firing predicate — the only occurrences (lines 47, 51, 504) are the historical table that *measures* r1's rule as inverted, or an unrelated reference to "r1's rule" re: libnunchuk (line 504, about the PR-1746 recipe, not the firing predicate). Not residue.
- `"three pieces of work"` — **0 hits** as a live claim; only the corrected heading "**four** pieces of work" (line 809) and a past-tense reference "r3 described stage 4a as … Measured it is four — three invisible from that description" (lines 811-813). Not residue.
- The claim that only `mnemonic-toolkit` is downstream — line 837 states *"r2's downstream list named only `mnemonic-toolkit`"* (historical, refuted by the surrounding paragraph establishing `me` is also downstream); line 840 independently states `mnemonic-toolkit` **is** downstream (true, not exclusive). Not residue.

**§8 item numbering.** Enumerated §8's ten items directly from the numbered list (lines 677-732): 1 recipe vectors, 2 descriptor equality, 3 address equality, 4 dispatch round trip, 5 identity distinctness, 6 structure-independence pin, 7 render-reparse fixpoint, 8 live Liana install, 9 operator-facing gate table, 10 mutation testing. Checked every `§8.N` reference against this list:
- Stage 1b gate (line 779): "1, 2, 5, 6, 7, 10" — all six correctly attributed to stage 1b per the owning-stage table.
- Stage 2 gate (line 780): "§8.2 and §8.8" — items 2, 8 — correct.
- Stage 3 gate (line 781): "§8.4's … leg and §8.5's Go identity leg" — items 4, 5 — correct.
- Stage 4 gate (line 782): "§8.3's device leg" — item 3 — correct.
- Stage 4a gate (line 783): "§8.9's `me` rows" — item 9's per-stage table has stage-4a rows — correct.
- Owning-stage table (lines 794-800): all seven rows resolve to real items and match the items' own text.
No dangling or mismatched `§8.N` reference found.

**Cross-reference integrity.** Extracted every `§N[a-z]?(.N)?` token (33 distinct forms) and every markdown heading. All resolve: §0, §0a, §0b, §2, §3a, §3c, §3d, §3e, §3f, §4, §4a, §5, §6, §6a, §7, §7a (+ .1/.2/.3, which map to §7a's own three-item numbered list), §8 (+ .2/.3/.4/.5/.8/.9/.10, all within §8's ten-item list), §8b, §9, §9a all exist as either section headings or list items inside an existing section. One reference, `§8f` (lines 161, 624), does not resolve to any heading or list item in *this* file — it is a pre-existing (untouched by this diff) reference to a section in the separate F-633 spec/copy, contextually clear from "F-633 becomes gating for this cycle, and its copy fix must also say that §8f's…" This is not new residue and not a broken internal cross-reference; flagged for completeness only.

**§0b internal agreement.** PLACEMENT, RESET, DEFAULT ROW and COPY (lines 79-154) checked word-for-word against §8.9's four gate rows (lines 719-722) and §9 stage 4's content/gate columns (line 782):
- PLACEMENT: §0b says "before `composerTemplateChunksFor`" (line 82-83); §8.9's row says the same (line 719). Agree.
- RESET: §0b hooks at `gui/composer_flow.go:98` (line 101); §8.9's row states the predicate-reevaluation test plus the converse (line 720), consistent with §0b's rule. The hook-point line number (`:98`) is identical everywhere it appears — lines 101 and 116 both cite `:98` (verified by grep, no other line number given for this hook anywhere in the file).
- DEFAULT ROW: §0b's "on first entry only… on any re-entry" (line 139-140) matches §8.9's "on FIRST entry… on RE-ENTRY" (line 721) exactly in structure.
- COPY: §0b (line 150-152) and §8.9's row (line 722) both state "coordinators" + "different wallets." Agree.
- §9 stage 4's content column (line 782) references "§0b's choice screen — predicate, placement, reset, default row and copy" (generic, non-contradictory) and its gate column references "§0b's firing predicate exercised on all six `tr` presets" (also non-contradictory, doesn't restate a stale rule).

No disagreement found among PLACEMENT/RESET/DEFAULT ROW/COPY, §8.9, and §9 stage 4.

---

## Q3 — citations

**New citations in this diff** (9 distinct `file:line` forms, extracted via `git diff … | grep '^+' | grep -oE` for qualified paths, plus bare `:NNN` forms in the same added lines): `gui/composer_flow.go:98`, `md/policy_shape.go:107`, `gui/composer_flow.go:267-273`, `gui/mk1_inspect.go:36`, `gui/md1_gather.go:33,39`, `:121` (same row, `gui/md1_gather.go`), `composer_flow.go:96-128`, plus re-cited (already-verified-in-r4) `bundle.rs:371`, `md/md.go:22`, `sysw/record.rs:251-252`, `validate.rs:95-100`.

Resolved against pinned baselines (`descriptor-mnemonic` not applicable here — these are all Go — `seedhammer` at `7b2fb`/`7b6f2fb`):

| citation | claim | result |
| --- | --- | --- |
| `gui/composer_flow.go:98` | `composerTemplateChunksFor(st)` call site, the RESET hook point | **exact** — `template, err := composerTemplateChunksFor(st)` is literally line 98 |
| `gui/composer_flow.go:95` (re-verified) | closing brace of `composerShapeFlow`'s `if` block | **exact** — line 95 is the lone `}` closing that block |
| `gui/composer_flow.go:96` (window) | `composerSizeAssignments(st)` | **exact** |
| `gui/composer_flow.go:128-129` (window) | `composerStubDelta` / first `composerStubFlow` | **exact** |
| `md/policy_shape.go:107` | `func PolicyShapeChunks(strs []string) (PolicyShape, error)` | **exact** |
| `gui/composer_flow.go:267-273` | `composerTemplateChunksFor` is pure, `md.ComposeWith(st.list, …).Chunks()` | **exact** — function body matches verbatim within the cited range |
| `gui/mk1_inspect.go:36` | `gatherIgnored` declared | **exact** — `gatherIgnored gatherStatus = iota // not an mk1 chunk / parse failed` |
| `gui/md1_gather.go:33,39` | both `return gatherIgnored` sites | **exact** — both lines are `return gatherIgnored` |
| `gui/md1_gather.go:121` ("printed at") | the `"Not an md1 descriptor chunk."` message | **off by one** — line 121 is `case gatherIgnored:`; the string is assigned one line later, at `:122` (`msg = "Not an md1 descriptor chunk."`). Same 2-line switch-case block, so an implementer lands in the right place regardless; flagged as an imprecise citation, not a wrong one. |

**1 imprecise citation, 8 exact, out of 9 new citations checked** (the four re-cited facts from r4 were already verified accurate in that report and were not re-derived here).

**10 older citations spot-checked at random** against `descriptor-mnemonic` `6cbd49d8` and `seedhammer` `7b6f2fb`:

| citation | claim | result |
| --- | --- | --- |
| `compose/mod.rs:292` (md-codec) | `is_bare_single` | exact |
| `compose/tr.rs:45` (md-codec) | `is_nums: ik.is_none(),` | exact |
| `header.rs:4` (md-codec) | usable WF-redesign set `{4, 8, 12}` | exact |
| `decode.rs:191-193` (md-codec) | `decode_md1_string_with_opts` entry, before header parse | resolves to the function's exact start; accurate anchor |
| `identity.rs:120-127` (md-codec) | "two engravings of the same logical wallet produce identical IDs" | exact phrase present in that doc comment window |
| `gui/multisig_verify.go:834` (fork) | device read-back path routes through the dispatch | exact — `md.ExpandWalletPolicyChunks(readbackMd1)` at line 834 |
| `md/compose.go:961` (fork) | `isNums: ik < 0` unconditional | exact — literal text at line 961 |
| `address/taproot_script_path.go:261-270` (fork) | `MultiALeafScript` sorts derived x-only keys | exact — function starts at 261, `sort.Slice` at 269 |
| `skeleton.rs:313-320` (md-codec) | `skeleton_key` appends `key_path_kind_label` | function starts :312, the named call is at :319, within range |
| `tree.rs:148` (md-codec) | the `debug_assert!` this cycle retires | exact — line 148 is inside that `debug_assert!` call |

**10 of 10 accurate.**

---

## ready for implementation: yes

0 of 8 findings unaddressed, 0 residue defects from the propagation class this check targeted, and 17 of 18 checked citations (9 new + 10 spot-checked, one overlap) resolve exactly — the sole miss is a one-line-off pointer inside a 2-line switch-case block that does not mislead an implementer. Nothing found here blocks implementation.
