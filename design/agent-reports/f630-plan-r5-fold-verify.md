# F-630 r5 fold verification (sonnet, mechanical, closing)

**Reviewer:** independent of the fold's author; verifying the fold only, not re-litigating r5.
**Input report:** `design/agent-reports/f630-plan-r5-close.md` (0C/2I/5M/1N).
**Plan before:** `git show 7b032747:design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`.
**Plan after:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at `d9499e7b`.
**Fold diff:** `git diff eecb1bf7..d9499e7b` — confirmed `eecb1bf7` and `7b032747` are byte-identical
for this file (0-line diff), so the brief's premise holds: the range is exactly the r5 fold.
Fold touches two files: the plan doc (+72/-8) and `design/FOLLOWUPS.md` (+18, new F-632 entry).
No other tracked file changed.

**Trees:** fork `/scratch/code/shibboleth/seedhammer` `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` `b2c5d693` — both match the brief's pinned
baselines exactly. No tracked file modified by this review; both trees confirmed clean
(`git status --porcelain` empty in both) and in mnemonic-engrave (only the pre-existing untracked
`.claude/worktrees/` noted at session start, not touched here). No scratch file was created in
either repo; a scratch diff file was written under the scratchpad only.

**Verdict: 7 FIXED outright, 1 FIXED-with-a-note (M-1's selector sub-part), 0 PARTIAL, 0
NOT-FIXED. 0 new Critical / 0 new Important / 0 new Minor / 0 new Nit introduced by the fold.**

---

## Per-finding verdicts

| Finding | Verdict | Plan text that settles it |
| --- | --- | --- |
| **I-1** D1′'s false justification | **FIXED** | Lines 118-123 name and retract the false claim verbatim ("An earlier draft justified D1′ by claiming `template` is 'bound to the card by the existing `wallet_descriptor_template_id` assertion'. **That is false**..."), then lines 125-134 substitute the measured-true input↔output rationale, citing `test_vectors.rs:108`, `vectors.rs:54`, `vectors.rs:142`, all three verified below. |
| **I-2** pinned tier omits D1′ | **FIXED** | Line 331-332 adds "**D1′ and D1″**" to the pinned-tier clause list and states "Only the *header* arm of D1 relaxes; nothing else does (r5 I-2)." Lines 333-340 reproduce the M19 SILENT-under-plan-as-written measurement and the 46/46-pass-with-D1′-applied-everywhere measurement, matching the r5 report's own numbers exactly. |
| **M-1** membership-assertion severity | **FIXED** | Line 360-361: "`t.Errorf`, never a `t.Fatalf` (r5 M-1): at T1 the pinned set is legitimately empty, and a fatal assertion aborts before any vector is examined, hiding T1's own 44/2 acceptance behind a single unrelated line." Matches T1's stated "44 of 46 fail, 2 pass" (line 404). **Note, not a defect:** the report's M-1 had a second sub-part — whether the tier *selector* stays file-existence. The fold's new text doesn't add a dedicated sentence for it, but the surrounding D5g text is unchanged and already states the selector explicitly: "the gate enumerates `md/testdata/forkbuilt/*.conformance.json`" (line 359, pre-existing from r4) — a directory glob, i.e. file-existence — with the name-list as the assertion layered on top, which is exactly the reading that reproduces T1's "2 pass/44 fail." So the ambiguity the report flagged is resolved by pre-existing text once read in full context; nothing here is now wrong or newly ambiguous. |
| **M-2** strict vs normalising | **FIXED** | Lines 136-139: "**D1′ compares STRICTLY — no hardening normalisation (r5 M-2).** D3 normalises `48h`↔`48'` because it compares *paths*; D1′ compares whole strings, and both variants are exact on 46 of 46, so strict is free and additionally catches a bracket spelling flip." Explicitly distinguishes D1′ from D3's normalisation (line 199-201, unchanged) — no contradiction. |
| **M-3** xprv, non-gating residue | **FIXED** | Plan's "not cover" section, lines 514-519, states it is a secret-handling defect under the 2026-08-27 ruling, non-gating, "**Filed as a follow-up, not scheduled here.**" Verified an actual follow-up exists: `design/FOLLOWUPS.md` **F-632** (lines 19064-19080 at current HEAD), Status OPEN, Owning phase "none (opportunistic; secret-handling, non-gating)", Tier `secret-handling`, content matches the plan text (same header-byte forgery, same "unreachable from the primary" reasoning). Filed in the same fold commit (`d9499e7b`), which is correct — filing residue is part of the fold's authorship response, not a second copy of a report. |
| **M-4** T4 rationale vs D1″'s export | **FIXED** | Lines 489-495: "**one non-test Go file**, and only one: `bip380/checksum.go`, where D1″ exports `validChecksum` (r5 M-4)... If the size moves at all, something beyond that export was edited..." This also *repairs a pre-existing internal inconsistency*: the "not cover" section already said (unchanged by this fold) "one identifier exported from `bip380` for D1″" (line 501), which directly contradicted T4's old "zero non-test Go files" claim — that contradiction is exactly what M-4 flagged, and it is now resolved: both sections agree. |
| **M-5** ambiguous slot lookup | **FIXED** | Lines 141-146: "**A slot lookup that is not single-valued must fail loudly (r5 M-5).**... the gate reports 'ambiguous slot material' and stops, rather than guessing." The report's suggested mechanism (match on `(65 bytes, origin path)`) wasn't adopted verbatim, but the core ask — "an ambiguity announces itself instead of resolving arbitrarily" — is satisfied by the explicit fail-loud behaviour, which is a valid (and simpler) way to close the false-RED risk the finding was about. |
| **N-1** vendored copies under pinned names | **FIXED** | Lines 520-527, new "not cover" bullet: "**The three VENDORED records under pinned names get no descriptor coverage** (r5 N-1)... Recorded because it is the one place where 'every keyed record's descriptor is checked' is not literally true." |

## The four newly-added citations — all verified against the pinned trees

1. **`crates/md-codec/src/test_vectors.rs:108`** — is that a `Vector { … template: … }` literal?
   **Confirmed.** `sed -n '108p'` at `b2c5d693`:
   `Vector { name: "keyed_wsh_multi_2of3", template: "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,…))", …`

2. **`crates/md-cli/src/cmd/vectors.rs:54`** — does it derive the descriptor from `v.template`?
   **Confirmed.** `sed -n '54p'`: `let mut descriptor = parse_template(v.template, &parsed_keys, &fps)?;`

3. **`crates/md-cli/src/cmd/vectors.rs:142`** — does it emit `template` into the record?
   **Confirmed.** `sed -n '142p'`: `root.insert("template".into(), json!(v.template));` — inside
   `conformance_json`, the function that builds each `.conformance.json` record.

4. **`rec.template` is asserted nowhere in the fork.**
   **Confirmed by exhaustive grep.** The Go struct field exists in exactly two places:
   `md/conformance_keyed_test.go:16` (`Template string \`json:"template"\`` — declared, then
   **zero** further `.Template` occurrences anywhere in that file) and
   `gui/policy_address_test.go:20` (same declaration), where the only two uses of `vec.Template`
   in the whole file are at lines 224-225, both **inside a `t.Fatalf` message string**, never
   compared or asserted against anything. Full-repo grep for `\.Template\b` (excluding the
   unrelated `md.Template` decoded-summary type) surfaced no third site. The claim holds.

## Did the fold introduce a defect? — No.

- **No over-reach.** I-2's "only the header arm of D1 relaxes; nothing else does" was checked
  against both named risks in the brief: (a) **D3's allowlist** — a wholly separate mechanism for
  two *different* vectors (`keyed_wpkh`, `keyed_tr_keyonly`, elided-origin), disjoint from the
  three F-529 pinned-tier names, so no contradiction is possible; (b) **D2c** — the pinned-tier
  clause list explicitly says "full D2a, D2b, D2c" (unchanged wording extended, not narrowed), so
  D2c is not weakened for the pinned tier. Confirmed no tension.
- **No orphaned labels or dangling counts.** All 8 `r5 …` tags (I-1, I-2, M-1 through M-5, N-1)
  appear exactly once each, each attached to the passage that resolves it. `grep -c "D1′|D1″"` = 15
  occurrences, all in defined, cross-referenced contexts. T1's "44/2" and "5 pass/41 fail" numbers
  (pre-existing, unchanged) still agree with the new M-1 text that cites "44/2" directly.
- **Scope matches the diff.** `git diff --stat eecb1bf7..d9499e7b` touches only the plan doc and
  `FOLLOWUPS.md`; no code file, no other design doc.
- **Trees clean.** No tracked file modified by this review in mnemonic-engrave, the fork, or the
  primary. No scratch file left in either repo.

## Out of scope, per the brief

r5's verdict (B = YES) was not re-litigated. Earlier rounds' findings were not re-checked. No
style/wording review beyond what bears on the 8 findings. No new features proposed.
