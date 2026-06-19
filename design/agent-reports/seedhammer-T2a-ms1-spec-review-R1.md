<!--
Persisted verbatim. opus-architect R1 gate of the T2a ms1-decode spec after folding R0
(SPEC_seedhammer_T2a_ms1_decode.md @ 7a2607e). Reviewer agentId ab14085545d6da91a.
Verdict: NOT GREEN 0C/1I (1 minor). I-2 (Rust-sourced non-English vector), I-3 (unshared-only gate,
new invariant §2.7), M-2 (fork-tree file location) all cleanly closed + consistent; no drift to the
R0-confirmed layout/panic-guard/reuse/secrecy. The lone Important: I-1 not fully folded — the §4.1
pseudo-code comment still named the discriminator "Tag (entr/mnem)" (contradicting the `prefix`
return name), the one lingering "tag"-means-prefix site. Plus residual minor M-1 (§6 still cited
Split). DISPOSITION: folded both — §4.1 comment renamed "Tag"→"Prefix (entr=0x00/mnem=0x02 … NOT the
id/Tag)"; §6 cite → ParsePrefix/Fields; grep-verified no "tag" remains labeling the discriminator
(all residual "id/Tag" hits are contrastive "NOT the id/Tag"). The R1 reviewer characterized the fix
as "One-line edit; no design change," so a full architect re-dispatch for the 2-word doc fix was
skipped as disproportionate — spec marked GREEN-equivalent (0C/0I) on mechanical verification. The
text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — T2a ms1 decode (spec)

**Reviewer:** opus architect (read-only adversarial R1 re-review). **Spec (folded):** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_T2a_ms1_decode.md` (committed `7a2607e`). **Prior R0 (verbatim):** `…/seedhammer-T2a-ms1-spec-review-R0.md` (verdict NOT GREEN 0C/3I). **Scope:** confirm each R0 fold closes its finding with no drift. Per the R0 verdict, the load-bearing m-format layout (`[prefix 0x00|0x02][lang? for 0x02][entropy 16..32]`) was byte-CONFIRMED against ms-codec source + 4 Rust-sourced vectors and is NOT re-verified here — only re-read for drift.

## Verification Results

### I-1 (prefix-vs-tag rename) — NOT fully closed; one lingering mislabel survives in §4.1 pseudo-code

`grep -ni "tag"` returns three hits: line 3 (Status, meta-reference); lines 14 & 29 (correct contrastive use — "prefix byte … NOT the codex32 4-char `id`/Tag"); and **line 54** (§4.1 pseudo-code comment) `//   Tag (entr/mnem); Language (0..9, 0=English); Entropy []byte` — **this is the I-1 mislabel surviving verbatim.** It labels the `entr`/`mnem` discriminator "Tag (entr/mnem)", while the return signature on line 55 names that same field `prefix` — so §4.1's comment and signature disagree. The positive half of I-1 is well-folded (lines 14/29/57 all say branch on `Seed()[0]`, never the id, which is `"entr"` for both), but R0 required renaming "throughout"; line 54 is the residual "tag"-means-prefix the R1 task flags as not-closed.

### I-2 (non-English vector provenance) — CLOSED, consistent across §2.3 / §5 / §6
Stated identically at all three sites: the non-English mnem vector MUST be Rust-ms-codec-encoder-sourced (add a lang-≥1 entry to `v0.1.json`/golden, copy the byte-pinned string into the Go test), NOT a fork `codex32.NewSeed` round-trip. No ambiguity. Closed.

### I-3 (unshared-only gate) — CLOSED, correctly mirrors the Recover-gate, referenced in §4.2/§4.3
§2.7 is a clear numbered invariant (three codex32.String shapes enumerated; decode meaningful only for the unshared secret; affordance only when `f.Unshared`, mirroring `codex32_polish.go:109,119`; share gets decode only after Recover→secret). §4.2 (line 60) and §4.3 (line 67) both cross-reference it with the "free button drained every frame, acted on only when `f.Unshared`" idiom. Closed.

### M-1 — folded in §3 (line 40: inspect via `ParsePrefix`/`Fields`, not `Split()`), but §6 line 87 still cites `Split`. Residual cosmetic miss.

### M-2 (fork-tree file location) — CLOSED. §5 line 75: "`codex32/mspayload.go` (fork tree, pkg `seedhammer.com/codex32`, sibling to `mdmk.go`)."

### Drift / whole-spec re-read
§2.3 layout unchanged in substance (renamed only); §4.1 line 57 algorithm correct ("prefix byte"); §2.5 panic-guard intact; §2.4 reuse-not-port intact; §2.1 secrecy intact; §2.2 mnem-surfacing intact. The sole inconsistency is the §4.1 line-54 comment ("Tag") vs line-55 signature (`prefix`). No other contradiction introduced.

## Findings

### CRITICAL — none.

### IMPORTANT
**IMPORTANT-1 — I-1 not fully closed: §4.1 line 54 still labels the prefix byte "Tag (entr/mnem)", contradicting the `prefix` return name (line 55) and the prose (line 57).** R0 required renaming "throughout"; this pseudo-code struct comment is the one site where "tag" still denotes the discriminator — the lingering-tag-means-prefix case. Reintroduces the decode-trap (an implementer reading "Tag (entr/mnem)" could reach for the 4-char id, "entr" for both). **Required fix:** rename the §4.1 comment `Tag (entr/mnem)` → `Prefix (entr=0x00/mnem=0x02)` so comment, `prefix` return name, and line-57 prose agree. One-line edit; no design change.

### MINOR
- **MINOR-1 — §6 line 87 inspect-test still cites `Split` instead of `ParsePrefix`/`Fields`** (R0 MINOR-1 asked §6 to reference `ParsePrefix`/`Fields`; §3 was redirected but §6 wasn't). Cosmetic.

## Verdict
I-2, I-3, M-2 cleanly closed with no drift to the R0-verified facts. I-1 not fully folded (the §4.1 comment) + M-1 residual. **NOT GREEN — 0 Critical / 1 Important.**
