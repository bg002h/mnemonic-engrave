# R1 (scoped re-review of the fold) — `IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md`

**1 Critical / 5 Important / 7 Minor / 3 Nit — NOT ready for implementation.**

Scope: did the fold address r0's findings, and are the three new things in it sound.
No fresh audit; stage 1b not reviewed.

**Tree note — the plan moved under me.** The brief names r1 at `4b2b720c`. While this
review was running, `f77b1324` ("plan: stage 1a — remove a stale paragraph that
contradicted its own fold") landed, deleting the six-line paragraph in Task 1 Step 2
that told the implementer to reuse `keyed_phrase_files()`. **This report is against
`f77b1324`**, and r0's I-2 is marked ADDRESSED on the strength of that commit alone —
at `4b2b720c` it was still open. Line numbers below are `f77b1324` (521 lines).

Everything below was reproduced in `/scratch/code/shibboleth/dm-worktrees/f449-stage1`
@ `6cbd49d8`. The probe example used for Q2(a) was deleted; the worktree is clean.

---

## Q1 — did the fold address each finding?

### Critical

| # | verdict | detail |
| --- | --- | --- |
| **C-1** (`cases.json` inside the generated corpus) | **PARTIAL** | The fixture is gone from `tests/vectors/` and the reason is stated correctly (Task 0 preamble, ll. 127–141). But **Task 0 Step 4 still stages the two artifacts the fold deleted**: `git add crates/md-codec/tests/vectors/liana … scripts/vendor-liana-evidence.sh` (ll. 243–244). Neither exists and neither is created by any step. Reproduced: `fatal: pathspec 'crates/md-codec/tests/vectors/liana' did not match any files`, **exit 128** — the Task 0 commit never happens. See **NEW-1**. |
| **C-2** (the "v0.14-era wire version 2" premise) | **PARTIAL — still Critical** | Task 0 Step 2 retracts the premise correctly and `decode_vendored` is the right fix (Q2a: verified, 65/65). But **Task 1 — which is where the gate actually runs — was not folded**: Step 1's test still calls `reassemble(&refs)` on every name in the golden (l. 286), and Step 1a's doc comment still says *"Those 13 are the v0.14-era wire-version-2 vectors -- Task 0 skips them by name"* (l. 304), which Task 0 explicitly no longer does. Reproduced: plain `reassemble` fails on exactly those 13 files, so Step 3's "Expected: **PASS**" is unreachable. The plan now states the fact and its retraction 130 lines apart. |

### Important

| # | verdict | detail |
| --- | --- | --- |
| **I-1** (golden specified three ways) | **NOT ADDRESSED** | Task 0 Step 2 emits an **object** (`{count, tr_count, vectors}`, l. 204). Task 1 Step 1's reader is still `Vec<(String, String)>` + `.expect("golden parses")` (ll. 279–281) — that `from_str` fails on an object, so the gate panics before reaching the wire comparison. Task 1 Step 2's check is still inert *and* now prints a wrong number: `print(len(d),'vectors')` on the object prints **3**. |
| **I-2** (reuse `keyed_phrase_files()`, 46 files) | **ADDRESSED** | Task 0 Step 1 forbids it by name with the measured 46 (l. 151), and `f77b1324` deleted the contradicting instruction from Task 1 Step 2. |
| **I-3** (`dump_encodings` run before defined; `dump_ids` never defined) | **PARTIAL** | Task 0 Step 1 now creates both and names the three real identity APIs. But Step 1 tells the examples to "read each with `load_vendored_phrase` (Task 1 Step 1a)" and "decode with `decode_vendored`" — `load_vendored_phrase` is defined inside `crates/md-codec/tests/internal_key_refactor.rs`, a **separate crate root** an example cannot import from, and `decode_vendored` has no stated home at all. Same non-executable-reuse class I-2 just closed. See **NEW-2**. |
| **I-4** (Step 8 acceptance count wrong) | **ADDRESSED** | Now "1400 + the new tests … Assert the shape, not a stale number: 3 skipped, 0 failed" (ll. 462–465). Correct and better than a number. |
| **I-5** (2 of 6 gate steps; `cargo build` misses examples) | **PARTIAL** | Widened to seven commands with the pinned-toolchain `PATH` export and `--all-targets` — the substance of the finding. But it hand-rolls the list instead of calling `scripts/phase-gate.sh`, and **drops `RUSTDOCFLAGS="-D warnings"` from the `cargo doc` line**. CI sets it at job level (`ci.yml:82–84`); without it `cargo doc` warns and exits 0, so the doc leg of the "full gate" cannot fail — on a repo whose current HEAD commit is *"md-codec: docs on public items must not LINK to private ones (CI cargo doc)"*, and in a task whose Step 4 adds an intra-doc link. See **NEW-3**. |
| **I-6** (no rule for `LianaUnspendable` at the inverse sites) | **ADDRESSED** | A ruling table for all four sites plus the reasoning. Sound — see Q2(b). |
| **I-7** (blast radius omits `crates/*/tests`) | **ADDRESSED** | Third table row (38) and the `tr_node` / 29-call-site note added (ll. 102–115). |
| **I-8** (Step 9 re-stages the golden unchecked) | **PARTIAL** | The guard is the right check and is placed correctly, but it is anchored to `HEAD` while Task 1 Step 2 still says *"Generate and commit it as its own commit"* (ll. 331–333). See Q2(c) — there is a passing-while-stale ordering, and it is the one the plan itself prescribes. |

### Minor / Nit

| # | verdict | detail |
| --- | --- | --- |
| **M-1** `fuzz/tests/gen_corpus.rs:182` | **NOT ADDRESSED** | `grep -c fuzz` on the plan = **0**. The file still builds `Body::Tr { is_nums: false, key_index: 0, tree: None }` and nothing in the workspace gate compiles it. |
| **M-2** the existing whole-corpus gate | **PARTIAL** | `vector_corpus.rs` is now cited in Task 0, but Task 1 Step 1's heading still claims "it is the whole gate for this task" (l. 261). |
| **M-3** the `&mut` rewrite | **NOT ADDRESSED** | Step 7 still gives three patterns; `canonicalize.rs:115` (`*key_index = perm[*key_index as usize]`) needs a fourth. |
| **M-4** golden covers `key_index == 0` only | **NOT ADDRESSED** | Re-measured over all 65 with the fold's own reader: 23 `Body::Tr`, 10 NUMS / 13 Slot, and **0 of the 13 Slot vectors carry a non-zero index**. Widening to 65 did not widen this. `tree.rs:559` remains the only `key_index = 2` coverage. |
| **M-5** prose sites; stale `Tr` variant doc | **NOT ADDRESSED** | Step 4's snippet shows the variant without its outer doc comment (`tree.rs:41–48`), which describes `is_nums`/`key_index` in five lines. |
| **M-6** dangling task references | **PARTIAL** | "Task 8" at l. 224 is now a retrospective reference and is fine. "Task 3" (l. 382) and "Task 6" (l. 454) still point at tasks this plan does not contain. |
| **M-7** "the assertion block below" | **ADDRESSED** | Removed. |
| **M-8** document still presents itself as 1a+1b | **NOT ADDRESSED** | Title, the File Structure table (seven 1b-only files) and the first four Global Constraints bullets are unchanged. |
| **N-1** `Copy`/`Hash` on `InternalKey` | **NOT ADDRESSED** | Unchanged. |
| **N-2** Status line | **NOT ADDRESSED — now garbled** | ll. 11–13 read: `r1, folded from the stage-1a R0 (…f449-plan-stage1a-r0.md). Awaiting re-review.` / `(0C/6I/16M). Reports: design/agent-reports/f449-plan-stage1-{r0,r1}.md.` / `Awaiting re-review.` — a dangling count from the *combined* plan's r1, a second report path that contradicts the one directly above it, and "Awaiting re-review" twice. |
| **N-3** two citation slips | **DECLINED, correctly on one** | `header.rs:26` **is** the `{4, 8, 12}` doc line and `:27` is the const — the plan's citation is right and **r0's nit was off by one**. The `split` slip stands: `chunk.rs:240` is `pub fn split(d: &Descriptor) -> Result<Vec<String>, Error>`, not `-> Vec<String>` (l. 72, in the 1b carried table). |

### New defects the fold introduced

- **NEW-1 (Important).** Task 0 Step 4's `git add` (ll. 243–244) names `crates/md-codec/tests/vectors/liana` and `scripts/vendor-liana-evidence.sh`, both removed by this fold. Reproduced: exit **128**, nothing staged, the task cannot close. Fix: `git add crates/md-codec/tests/golden crates/md-codec/examples`.
- **NEW-2 (Important).** Task 0 Step 1 delegates both examples' file reading to `load_vendored_phrase` in a test crate and their dispatch to `decode_vendored`, which no step assigns to a file. Self-Review §2 then asserts *"One helper is used and not defined by any step … It is defined in Task 1 Step 1a"* — the reference does not resolve across crate roots. Say plainly that each example carries its own copy of both helpers (or put them in `examples/common/` with `#[path]`).
- **NEW-3 (Important).** Step 8's `cargo doc` line omits `RUSTDOCFLAGS="-D warnings"`; the doc gate cannot fail. It also drops `phase-gate.sh`'s `design/display-grouping-vectors.tsv.sha256` step (harmless for this change — it touches no `design/` file — but it falsifies "the full six-command gate"). The other five commands and the freebsd cross-check are correct: `x86_64-unknown-freebsd` std **is** installed for the pinned 1.85.0 toolchain, so that line runs here.
- **NEW-4 (Minor).** Step 9's `git add` was broadened to `crates/md-codec/tests` and in doing so **dropped `crates/md-codec/examples`**. Only bites if an example is touched during Task 1, which Task 0 now owns — but the narrowing was not deliberate.

---

## Q2(a) — is `chunks.len() == 1` the right discriminator?

**Yes. Verified by execution, not reasoning.** I built an example in the worktree
carrying the plan's `load_vendored_phrase` and `decode_vendored` verbatim and ran it
over all 65 `tests/vectors/*.phrase.txt`:

```
files=65 ok=65 fails=[]
tr_vectors=23 tr_nodes=23 nums=10 slot=13 slot_with_nonzero_key_index=0
plain-reassemble would FAIL on 13 files: [nums_taproot, pkh_basic, sh_wpkh, sh_wsh_multi,
 single_string_boundary, tr_keyonly, tr_with_leaf, wpkh_basic, wsh_divergent_paths,
 wsh_multi_2of2, wsh_multi_2of3, wsh_sortedmulti, wsh_with_fingerprints]
```

- **All 65 decode**, and `encode_payload` then succeeds on all 65 (no encode failure).
- **`tr_count` is 23** under both readings — 23 files carry a `Body::Tr` *and* there are
  23 `Body::Tr` nodes in total (one per file). The plan never says which it means; here
  it does not matter. Worth one word in `dump_encodings.rs` anyway.
- **The `len() == 1` / chunk-set collision the brief asks about is real and is handled.**
  `wsh_multi_chunked` is `force_chunked: true` and splits into exactly **one** chunk, so
  its file has a `chunk-set-id:` header and one payload line: 14 files reduce to one
  line, not 13. It takes the `len() == 1` branch into `decode_md1_string`, whose
  auto-dispatch (`decode.rs:191–193`) reads the chunked flag, sees 1, and forwards to
  `reassemble_with_opts(&[s])`. Correct by construction, not by luck — `decode_md1_string`
  is a superset of the single-payload case, so the dispatch is safe in that direction.
- **The other direction cannot arise.** A single-payload vector with two lines would
  misroute, but `cmd/vectors.rs:70–87` writes exactly one line for `force_chunked: false`,
  and the helper filters blanks. No such file exists.
- Signatures check out (my probe compiled): `decode_md1_string(s: &str) -> Result<Descriptor, Error>`
  (`decode.rs:178`) takes `&chunks[0]` by deref coercion; `reassemble(strings: &[&str])`
  (`chunk.rs:311`, **not** `:240` — that is `split`) takes `&refs`. `pub mod decode` /
  `pub mod chunk` are both public (`lib.rs:21,24`). `serde_json` and `hex` are
  dev-dependencies, so they are available to `tests/` and `examples/` alike.

**But the discriminator is only in the generator.** Task 1 Step 1 — the gate — still
calls `reassemble` directly and will panic on those 13 names. That is C-2's residue, and
it is the finding that matters most here: the fold fixed the reader in the half of the
plan that captures the golden and not in the half that checks it.

## Q2(b) — the I-6 ruling

**Sound. Behaviour-preserving at all four, clean for 1b, and I found no fifth site.**

- **Behaviour-preserving in 1a: yes, trivially and verifiably.** I enumerated every
  non-test construction of `Body::Tr` in both crates (brace-matched against `#[cfg(test)]`):
  `compose/tr.rs:44`, `parse/template.rs:1604` and `:1629`, and the decode path
  `tree.rs:287`. After Step 7 rewrites them, none can yield `LianaUnspendable`, so every
  arm mapping it to NUMS is dead code in 1a. The choice cannot change behaviour.
- **The four sites are correctly characterised.** `render.rs:194` (`NUMS_H_POINT_X_ONLY_HEX`,
  `pub(crate)` in `nums.rs:12`, imported at `render.rs:21`), `to_miniscript.rs:341`
  (`build_nums_internal_key()` at `:363`), `policy_shape.rs:250`, `json.rs:353`. All four
  quotes match the source.
- **No fifth site.** Every other non-test consumer of `key_index` has the *forward* shape
  `if !*is_nums { … }`, for which Step 7's `if let InternalKey::Slot(i)` works and no rule
  is needed: `validate.rs:94`, `canonicalize.rs:63` / `:114` / `:283`, `policy_shape.rs:267`,
  `reuse.rs:520`. `canonical_origin.rs:110`/`:124` look like a fifth in a plain grep but sit
  inside `#[cfg(test)]` (opens at `:87`). `seat/compose.rs:148`, `encode.rs:152`,
  `decode.rs:130`, `validate.rs:245` and `canonical_origin.rs:53`/`:57` absorb the fields
  with `..`. (`canonicalize.rs:114` is M-3's `&mut` shape — a missing *pattern*, not a
  missing rule.)
- **1b's diff stays clean**, and the ruling hands 1b the list rather than a search.
- **One residual, recorded not blocking.** `to_miniscript.rs` is the address-derivation
  path: mapping `LianaUnspendable` to NUMS there means a 1b that forgets this site derives
  *NUMS-key addresses for a Liana descriptor, silently*, where `todo!()` would have
  panicked. The plan's counter-argument (a latent panic in funds-path code is worse) is
  defensible and, decisively, SPEC §8's vectors carry `liana_receive[3]` / `liana_change[3]`,
  so 1b has an address gate that catches the omission. Keep the ruling; make 1b's plan
  carry the four-site list as a checklist item rather than "note them in the stage's
  completion".

## Q2(c) — can the I-8 guard fail, and can it pass while the golden is stale?

**It can fail, and it can also pass while stale — via the ordering the plan itself prescribes.**

- **It can fail.** `git diff --quiet HEAD -- <path>` reports both staged and unstaged
  changes to tracked files, so the scenario I-8 named — implementer hits a red Step 8,
  regenerates the golden, `git add`s it — trips the guard at Step 9 whether or not it was
  already staged. That is the right check, placed at the right moment.
- **It can pass while stale.** The guard is anchored to `HEAD`, not to the commit Task 0
  made. **Task 1 Step 2 still exists and still says "Generate and commit it as its own
  commit"** (ll. 322–333) — it regenerates `pre_refactor_encodings.json` into the same
  path Task 0 already committed. An implementer who reaches a red Step 8, goes back and
  re-runs Step 2 as written, and commits it, has moved `HEAD` to contain the post-change
  golden; Step 9's guard then compares that golden against itself and passes. The gate
  reports green having pinned the refactor's output to itself — exactly the unintended
  green I-8 was raised to close.
  Two fixes, either sufficient: **delete Task 1 Step 2** (Task 0 Step 2 now owns the
  capture entirely, and the duplicate is what creates the hole), or anchor the guard to
  the Task 0 commit — `git diff --quiet $(git rev-parse HEAD~1) -- crates/md-codec/tests/golden`,
  or record the SHA in Task 0 Step 4 and diff against it.
- **Two smaller notes on the guard.** It sees tracked files only, so a golden written
  under a *new* filename passes unnoticed; and `|| { …; exit 1; }` will close an
  interactive shell if pasted rather than scripted.
- **A determinism question the plan should settle.** Because Step 2 regenerates the file,
  `dump_encodings.rs` must emit a **stable order** or the guard fires spuriously.
  `dump_skeleton_keys.rs` — the stated model — sorts (`out.sort()` at l. 34); the plan
  does not say the new examples must. One sentence closes it.

---

## Verdict

The fold's *substance* is good: the C-2 diagnosis is now correct and `decode_vendored`
is the right mechanism (proved by execution, 65/65, `tr_count = 23`); the I-6 ruling is
the right call and covers every site there is; I-4, I-7, I-2 and M-7 are cleanly closed;
I-5 and I-8 are 90% closed. What is not done is **propagation**. The fold rewrote Task 0
and left Task 1 Steps 1 and 1a on the retracted premise and the old reader, so the plan's
one gate cannot parse its golden and cannot decode 13 of the vectors the fold exists to
rescue — and Task 0's own commit step stages two files the fold deleted. Four of the six
blocking items are single-paragraph edits inside Task 1.

**ready for implementation: no**
