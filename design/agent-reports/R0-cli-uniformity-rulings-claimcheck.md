# Claim-check — fold `0645eae` against report `0093a77` (0C/4I/5M/1N)

**Method.** All binaries invoked by absolute path, stdin from `/dev/null`, exit
codes read directly (never through a pipe). `md`/`mk`/`ms` debug binaries and
`mnemonic` (mnemonic-toolkit) were rebuilt where missing. Gates already run
(spec-structure, plan-table, fold-propagation ×2) were **not** re-run, per
brief.

**VERDICT: NOT GREEN — 0 Critical / 1 Important still open / 1 Minor not
closed.**

---

## Part A — the ten findings

| # | verdict | evidence |
| --- | --- | --- |
| I-1 | **CLOSED** | `crates/me-cli/Cargo.toml` confirmed `[lib] name = "mnemonic_engrave"`. New text reads "No `mnemonic-*` package is library-ONLY" (true, provable) instead of the false negative; the `cargo install --list` block was moved off the negative claim. No re-mint of "there is no library" found anywhere in the doc. |
| I-2 | **CLOSED** | Re-ran live: no `-A` → `serde`=403, `mnemonic-io-lib`=403, `mnemonic_io_lib`=403 (indistinguishable, as the fold says). With `-A 'name-check'` → `serde`=200, both spellings=404. Spec now records exactly this control-then-gate shape. |
| I-3 | **PARTIAL — one site not fully closed** | Site B (§5a name, line ~284) fully fixed: reads "The name is RULED below." Site A (§5b) only half-fixed — see **Part C**, this is the same defect the new-defect hunt below found. |
| I-4 | **CLOSED** | §5c's new "DECIDED, NOT SCHEDULED" paragraph exists; §9a carries the matching qualifier (line 1560, "and that stays true after §5c…"). `grep -oiw` for `split`/`combine`/`compile`/`address`/`toolkit` inside §7's five phase rows (lines 1280-1284) = **0 each**, machine-verified. §6d's per-verb table (lines 828-835) does prescribe `--in`+argv-refusal on `ms`'s `split`/`derive` (and `combine`, same table), consistent with "P2 does schedule work on those verbs" via "all eight verbs." No contradiction found among §5c / §9a / §7 P2. |
| M-1 | **NOT CLOSED — new number is also false** | See **Part B**. `grep -ic repair` on the file at `0645eae` returns **41**, not the claimed **39**. |
| M-2 | **CLOSED** | Both sites now read "six manifests"; D5 (line 228) explicitly enumerates six consumers (`md`,`mk`,`ms`,`mt`,`me`,toolkit). |
| M-3 | **CLOSED** | Table now carries a strength column: `gen-man`/`gui-schema` = structural, `vectors` = locality. Verified `mnemonic-toolkit/vendor/` contains `md-codec`, `mk-codec`, `ms-codec`, and `mk-codec/src/bin/gen_mk_vectors.rs` exists outside any CLI. |
| M-4 | **CLOSED** | Now reads "4 of the 5 `-cli` crates are the reverse, and the fifth, `me-cli`, is both." Verified: `md-cli`/`mk-cli`/`ms-cli`/`mt-cli` have `main.rs` and no `lib.rs`; `me-cli` has both. |
| M-5 | **CLOSED** | New sentence added, verified by count: `md --help` = 12 subcommands excl. `help`, `mk` = 10, `ms` = 11, sum = **33**, matching "12 + 10 + 11 = 33" exactly. |
| N-1 | **CLOSED** | Section header now `### 5a-i. The name: \`mnemonic-io-lib\`…`, citable, sits between `5a` and `5b`. |

---

## Part B — factual claims in the fold's NEW text, re-run

| claim | command | result | matches? |
| --- | --- | --- | --- |
| `me-cli` declares `[lib] name = "mnemonic_engrave"` | `grep -A2 '^\[lib\]' crates/me-cli/Cargo.toml` | `name = "mnemonic_engrave"`, `path = "src/lib.rs"` | yes |
| publish gate, no `-A` | `curl -s -o /dev/null -w '%{http_code}' .../serde` etc. | `serde`=403, `mnemonic-io-lib`=403, `mnemonic_io_lib`=403 | yes |
| publish gate, `-A 'name-check'` | same + `-A 'name-check'` | `serde`=200, `mnemonic-io-lib`=404, `mnemonic_io_lib`=404 | yes |
| `repair` on **39 lines** | `grep -ic repair design/SPEC_constellation_cli_uniformity.md` | **41** (verified 3 ways: working tree, `/usr/bin/grep` direct, `git show 0645eae:<path> \| grep -ic repair`) | **NO — false, off by 2** |
| subcommand counts: `md` 12, `mk` 10, `ms` 11, sum 33 | `<abs-path> --help </dev/null`, count rows excl. `help` | 12 / 10 / 11 / 33 | yes |
| `-cli` partition: 5 `-cli` crates, 4 main-only, `me-cli` both | `ls src/{lib,main}.rs` per crate | `md-cli`,`mk-cli`,`ms-cli`,`mt-cli` main-only; `me-cli` both | yes |
| §7 phase rows: `split`/`combine`/`compile`/`address`/`toolkit` = 0 each | `sed -n '1280,1284p' \| grep -oiw <term> \| wc -l` per term | 0 / 0 / 0 / 0 / 0 | yes |
| `vectors` not structurally unmovable, `gen-man`/`gui-schema` are | inspected `mk-codec/src/bin/gen_mk_vectors.rs`, `md-cli/src/cmd/vectors.rs` (`use md_codec::test_vectors::{MANIFEST, Vector}`), `mnemonic-toolkit/vendor/{md,mk,ms}-codec` | vectors' generator/source lives in the codec crate outside any CLI; toolkit already vendors all three codecs | yes |
| controller-find: all four `mnemonic` exit-code cells (usage 64, invalid-artifact 1-or-2, repair-applied 4, repair-uncorrectable 2) | ran `mnemonic` (rebuilt) with no args/bad flag, `inspect notanartifact`, `inspect md1nonsense`, `repair md1yqpqqzqq8xtwhw4xwn4qh`, `repair` on two mangled strings; also `md repair` on the same corrected string for the cross-check | 64, 2, 1, `md repair`→5 / `mnemonic repair`→4, uncorrectable→2 | yes |

**One claim is false: M-1's replacement number.** `grep -ic repair` at the
exact committed SHA `0645eae` returns 41, not 39. This is the document's
named signature defect (a true-sounding number beside a command that does not
produce it) recurring **inside the very fold that fixed a different instance
of it** — the method (line count, case-insensitive, via `grep -ic`) is now
correctly described, but the number was not re-measured at the final text, so
it drifted low by 2 (plausibly because the paragraph's own sentence stating
the count is itself a "repair"-bearing line added after the count was taken,
and further text was added afterward). Low stakes on the merits (48 > 31 or
41 > 31 both support "load-bearing"), but it is exactly the class Question 2
was dispatched to hunt, and it is false as written.

---

## Part C — new defect found

**Site:** `design/SPEC_constellation_cli_uniformity.md` lines 478 and 484,
same paragraph, both inside this fold's diff hunk.

The fold reworded the paragraph's **opening** (I-3, site A) from "NOT
settled… whether the verbs… belong where they are" to:

> **SETTLED BELOW IN §5c** — the measurement here is the data §5c reasons
> from, and was written before it. The question it framed was whether the
> verbs in the right column belong where they are.

...but left the **same paragraph's closing sentence, seven lines later,
verbatim and untouched**:

> `repair` is duplicated in the toolkit as well. **Whether those move is the
> tier cycle's question, not this spec's**, and it is recorded here so that
> cycle starts from a measurement instead of an impression.

These now directly contradict each other within one paragraph: the head says
the question is settled below (in §5c), the tail — using the same word,
"whether" — says the question belongs to another cycle, not this spec. §5c
itself resolves the ambiguity in favor of the head: its ruling table gives a
per-verb yes/no ("→ toolkit" or "STAYS") for exactly this question, and the
later "DECIDED, NOT SCHEDULED" paragraph is explicit that this spec decides
**whether** verbs move (yes, five of six) — only **when/how** they move is the
tier cycle's question. The surviving tail sentence restates the pre-fold
position on the exact axis §5c settled.

This is not a fresh audit finding — it is the same site the report's own I-3
quote already spanned (the report quoted the paragraph's opening **and**
closing sentence together, with an ellipsis between them, as the problem
text). The fold's fix touched only the first half of the quoted text.

**What closes it:** delete or reword the closing sentence, e.g. "— the tier
cycle's remaining question is *when and how* the move executes, not whether;
§5c already answers that," and drop "not this spec's" (now false).

**Net effect:** I-3 should be treated as PARTIAL, not CLOSED.

---

## Counts and verdict

- Findings CLOSED: I-1, I-2, I-4, M-2, M-3, M-4, M-5, N-1 (8 of 10)
- Findings PARTIAL: I-3 (1 of 10) — one site fixed, one site (same paragraph) left contradicting the fix
- Findings NOT CLOSED: M-1 (1 of 10) — replacement number is also false (41 vs claimed 39)
- New defect: 1 — the §5b line-478/line-484 self-contradiction (same underlying text as the I-3 partial)
- False factual claim in fold's new text: 1 of 10 checked (`repair`-line count)

**VERDICT: NOT GREEN (0 Critical / 1 Important open [I-3] / 1 Minor not closed [M-1]).**

What closes it: (1) reword or delete design/SPEC_constellation_cli_uniformity.md:484's "Whether those move is the tier cycle's question, not this spec's" so it does not contradict line 478's "SETTLED BELOW IN §5c"; (2) fix the repair line-count to the correct measured value (41, or re-measure at whatever the final text becomes, since adding text shifts the count).
