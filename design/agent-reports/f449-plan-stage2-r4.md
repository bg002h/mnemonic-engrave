# R4 scoped re-review: the fold of `f449-plan-stage2-r3` into `IMPLEMENTATION_PLAN_f449_stage2_compose.md`

**Verdict: GREEN. 0 Critical / 0 Important / 3 Minor / 2 Nit. Both R3 Importants are closed.**

Scope: did the fold `11a14370..fab1c881` close R3-I-1 and R3-I-2, and did its own new text
introduce a Critical or Important? This is not a fresh audit. I did not re-derive anything the brief
settled.

**How the new text was executed.** I used two throwaway worktrees, both removed afterwards:
- descriptor-mnemonic `25acb33c` (toolchain 1.85.0);
- mnemonic-engrave `fab1c881`.

`git status --porcelain` in descriptor-mnemonic is empty afterwards. In mnemonic-engrave it shows only
the pre-existing `?? .claude/worktrees/`.

1. **Extraction (Task 2c Step 2).** Extracted `chunk.rs:547-631` verbatim into
   `pub fn correct_chunks(strings: &[&str]) -> Result<(Vec<String>, Vec<CorrectionDetail>), Error>`
   and re-exported it in `lib.rs`. `decode_with_correction` now keeps its `ChunkSetEmpty` guard, calls
   `correct_chunks(strings)?`, and in the single-string pre-pass re-derives the symbols with
   `parse_chunk_symbols(&corrected_strings[0], 0)?`, as the plan prescribes at `:565`.
   **`cargo nextest run --locked --all-features`: 1500/1500, 3 skipped**, the same as baseline.
2. **The `repair.rs` branch (Task 2c Step 3).** Written as prescribed, it produced:
   - the R3 v12 card `md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg` (1 error): stdout
     `md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg`, **exit 5**; `--json` gives the D27 `RepairJson`
     shape, also exit 5;
   - the clean v12 card `md1uzfdss…` (0 errors): **exit 2**, empty stdout;
   - baseline `md` on the damaged card: exit 2.
3. **Task 5 Step 2's order and proof.** I committed two probe JSONL records in the engrave worktree,
   then edited `NAMES`/`ACCEPTED_NAMES`, ran the regenerator and ran the three proof bullets. Results
   are in §1. The full suite with the 9-case corpus plus the new branch passed 1500/1500.

## 1. R3 findings

| R3 finding | verdict | evidence |
| --- | --- | --- |
| **R3-I-1** Task 2c's API change breaks the toolkit, and the D26 note denied a divergence | **ADDRESSED** | **(a)** The prescribed shape is the additive `pub fn correct_chunks`, with `Error` and `decode_with_correction` untouched. I implemented it and ran the whole suite: 1500/1500, the same as baseline. Every early return in the loop (`parse_chunk_symbols`, `ChunkSymbolCountOutOfRange`, the three `TooManyErrors`) moves into `correct_chunks` unchanged and propagates through `?` in the same order with the same `chunk_index`. The `ChunkSetEmpty` guard stays ahead of it. Re-parsing `corrected_strings[0]` cannot fail and returns the same symbols: a pass-through string was already parsed by the loop with the same function, and an `encode_chunk_string` output is `md1` + alphabet characters. The toolkit has no `use md_codec::*` and no `correct_chunks` of its own, so the new name cannot collide. **(b)** The v12 error is `WireVersionMismatch` on both paths. `decode_md1_string_with_opts` (`decode.rs:187-195`) unwraps first, and codex32 already passes after the correction. `Header::read` (`header.rs:50-55`) is the first read of the payload. On the chunked path `ChunkHeader::read` (`chunk.rs:69-73`) is the first read per chunk in `reassemble_with_opts` (`:351`). No earlier check can mask the version. `md repair` exits 5 on the corrected card and 2 on the clean one. `mnemonic repair` exits 2 on both: R3 measured the corrected card, and I measured the clean one (`post-correction decode failed: … got 12, expected 4`). So the D26 line, the CHANGELOG line (Task 8 Step 3) and the toolkit follow-up (Task 8 Step 5) all describe a real divergence and name its owner. **(c)** The "not in `repair.rs`" line is corrected: both files are in the Files list and in the text. |
| **R3-I-2** Task 5 Step 2's proof failed on a clean tree, and "same commit" spanned two repos | **ADDRESSED** | The order now spans the two repos explicitly (engrave commit, then descriptor-mnemonic). `git diff --exit-code` is retired, and re-stamping is declared expected. The recovery-path data is routed to the evidence directory. Executed: on a correct run all three bullets pass. Bullet 1 finds `probe-nested-b` with `accepted: true`. Bullet 2 prints only the new case's lines plus the `+  },` / `+  {` separator (Nit N-2). Bullet 3 finds 9/9 `source_commit` equal to the probe commit. A real change to an existing case (a moved address) would print `-`/`+` lines and fail bullet 2. Residue: bullet 3 cannot detect the exact order violation R3 described (M-3). The step order is now stated, so this is Minor. |

## 2. New Critical / Important

None.

## 3. Minor

- **M-1: the prescribed stderr text misdirects every pre-v0.30 card, and pre-v0.30 cards are this
  branch's documented main trigger.** Plan `:604` prescribes
  `…cannot read wire version N (accepted: …); take the corrected card to a newer md`. The branch fires
  on any `WireVersionMismatch`, and the codec documents that error first as the **v0.x legacy
  rejection** (`header.rs:1-10`, `:95-121`; `chunk.rs:62-66`, `:141-166`). Legacy cards share the HRP
  and `MD_REGULAR_CONST` (the same `0x0815c07747a3392e7` at `md-codec-v0.16.2`), so they pass BCH and
  reach this branch.
  - **Measured.** I took four md1 strings from the `md-codec-v0.16.2` tree and changed one character at
    data position 7 of each:
    - `md1qppqqxzxpp29gtcfh4dhmh72l6atuttfxe3cw2xenm` (got 0);
    - `md1qppqqxzxpp29gz6vpgtqrc4medgary758` (got 0);
    - `md1qqqqqvcrqceqqzqvrveqzrqmxgpq047xqk42r234a` (got 0);
    - `md1zpfcaqqqqsgqps3ssjs54glaevx4atqmzd8` (got 2).

    With the prescribed branch, all four exit **5** with the correct corrected string and
    *"take the corrected card to a newer md"*. Baseline exits 2 on all four.
  - **Why it is wrong.** No newer md reads version 0 or 2, so the advice is a dead end. The exit code
    and the stdout are right.
  - **Why it does not block.** The error is only in the advice: no result is wrong and nothing is lost.
    A one-line fix still belongs in the plan before implementation:
    - word the advice on `got`, saying "newer md" only when `got` is even and greater than 8, and
      "a pre-v0.30 or misread card" otherwise;
    - add a legacy-card row to the five assertions (e.g. the first string above with one error).
- **M-2: Task 2c Step 4's prescribed mutation is inert against the prescribed structure.** Step 3
  enters the report branch only when the details are non-empty. Plan `:626` says to mutate the new
  branch to return `if details.is_empty() { 0 } else { 5 }` and to show that assertion 5 reddens.
  - **Measured.** Applied literally to the branch's `return Ok(5)`, the mutation leaves the clean v12
    card at **exit 2**, and assertion 5 stays green. Only after I also removed the non-empty guard did
    the card exit **0** and assertion 5 redden.
  - **Why it does not block.** It fails loudly: the proof cannot be shown, so nothing false passes.
  - **Fix.** Specify the mutation as "drop the non-empty guard and return
    `if details.is_empty() { 0 } else { 5 }`", which models the reuse that R3 M-6 describes.
- **M-3: bullet 3 of the Task 5 Step 2 proof cannot see the ordering violation it exists for.**
  - **Measured.** I regenerated over *uncommitted* JSONL records (engrave HEAD `fab1c881` does not
    contain `probe-nested-b`). Bullets 1 and 2 give identical output. Bullet 3 shows 9/9 uniform
    `source_commit` values, so it passes whenever the implementer compares against the current HEAD,
    which is the only SHA available if step 1 was skipped.
  - **Why it does not block.** The order is stated, and nothing reads `source_commit`.
  - **Fix.** Add a mechanical check that the stamped commit holds the evidence:
    `git -C <engrave> show <sha>:design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl | grep -c '"<new-name>"'`
    should print `1`, and likewise for the out-file. Alternatively, require
    `git -C <engrave> status --porcelain -- design/evidence/composer-fable-r0` to be empty before the
    run.

## 4. Nit

- **N-1.** Because only lines 547-631 are extracted, `correct_chunks(&[])` returns
  `Ok((vec![], vec![]))` rather than `ChunkSetEmpty`. That is harmless, since the only caller reaches
  it after a decode error on non-empty input. But `correct_chunks` is a new `pub` API, so give it the
  same guard, or document the behaviour.
- **N-2.** Bullet 2 (`:760`, *"prints nothing except the new case's own lines"*) also prints the
  structural `+  },` and `+  {` lines that git attributes to the insertion. The new case's closing
  `  }` is instead aligned with the old one. Say "the new case's lines and the separating `},`/`{`" so
  the implementer does not read those lines as a failure.

## 5. Checked and sound (new text)

- **Files list and version bump.** Task 8 Step 1 lists the additive `correct_chunks` under the minor
  bump. The Files list adds `Cargo.lock` (R3 M-3).
- **The M-5 example.** `md compose --wrapper tr --path 1of1 --path 1of1,sha256=<64hex>,older=100`
  reports `"internal_key_path": 0`. The Step 4 warning therefore fires, and so does the hashlock half,
  as the fold claims.
- **Recovery paths.** R3 noted that `cases.json` cannot hold them, and the fold routes them to the
  evidence directory. The out-JSONL record also carries them in its `recovery` field, which is
  consistent with that routing.
- **Adding a ninth ACCEPT.** It turns no existing test red: 1500/1500 on the 9-case corpus.

ready for implementation: yes
