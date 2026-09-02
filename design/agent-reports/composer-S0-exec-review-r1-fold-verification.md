# Composer S0 — mechanical fold-verification of the exec-review-r0 response (R1)

**Artifact under verification:** two commits on branch `composer-s0`, worktree
`/scratch/code/shibboleth/wt-composer-s0`, repository descriptor-mnemonic:
`7c9b4fd7` (code + CHANGELOG fold) and `66bdf2f4` (`design/FOLLOWUPS.md` entry).
**BEFORE** = `9820e618`, **AFTER** = `66bdf2f4`.
**Responds to:** mnemonic-engrave `design/agent-reports/composer-S0-exec-review-r0.md`
(opus, 0C/1I/2M/3N), persisted at mnemonic-engrave `976cc45` (verified: `git log`
on that path shows `976cc45` as the commit that added the file).
**Lens:** did the fold fix each finding as the finding stated the defect, did it
introduce a new defect, and are its five new claims true.
**Read-only** on both repos and the worktree except its own `target/`. No
`.jsonl` file read.

**What I ran (summary; full list at the end):** read the BEFORE/AFTER diff for
every touched file; grepped `crates/md-cli/src/parse/template.rs`,
`cmd/build.rs`, `cmd/vectors.rs`, `cmd/descriptor.rs`, `cmd/address.rs` to
confirm the caller graph the CHANGELOG now asserts; invoked the already-built
`target/debug/md` (built at `66bdf2f4`, confirmed via `strings` containing the
new hint text) directly — no private copy needed since read-only invocation
was explicitly permitted — to reproduce every constructed example the
CHANGELOG and the FOLLOWUPS.md entry cite; ran three narrowly-scoped
`cargo nextest` invocations (not the full suite) to independently confirm the
M-2 fix and the two message-matching test files still pass; ran
`cargo fmt --check` and `cargo clippy --all-targets --all-features -D warnings`
scoped to the two touched crates.

---

## VERDICT: 4 FIXED / 0 PARTIAL / 0 NOT FIXED / 1 DECLINED (reason holds) — 0 regressions, 0 new defects

I-1, M-1, M-2, N-2 and N-3 are each fixed exactly as the finding described the
defect. N-1 was declined with the same reason the finding itself gave
(cosmetic, zero behavior/test-power impact); that reason still holds — `git
diff 9820e618..66bdf2f4 -- crates/md-codec/src/compose/mod.rs` is empty. All
five of the fold's new claims are TRUE, each reproduced live against the
built binary rather than accepted on the fold's word. No regression and no
new defect found in the touched code.

---

## Per-finding table

| id | one-line title | fold's response (AFTER, quoted) | verdict |
| --- | --- | --- | --- |
| I-1 | `md descriptor/address --template` newly refuse a signature-free path with no `--experimental` opt-out; undisclosed | CHANGELOG "Blast radius" paragraph names both verbs plus `md vectors` explicitly and states "NO `--experimental` opt-out yet"; `design/FOLLOWUPS.md` gains `md-descriptor-address-template-lack-experimental` with the exact caller and remedy | **FIXED** — the finding's own remedy was "amend the CHANGELOG … OR file a follow-up"; the fold did both |
| M-1 | Malleability + mixed-timelock are newly enforced under `wsh`/`sh` for the first time, not just the signature rule; CHANGELOG read as "already enforced" | Same paragraph, new sentence: "two more classes become refusals at mint for the first time: MALLEABLE scripts … and MIXED heightlock/timelock … No flag relaxes those two — both describe defective wallets, and `tr` has refused them all along." | **FIXED** — matches the finding's one-sentence remedy verbatim in substance |
| M-2 | `if let Ok(s) = encode_md1_string(...) { assert!(...) }` is a false-PASS shape (guard hides a dead assertion if the shape ever needs multiple strings) | `let s = encode_md1_string(&c.descriptor).expect("a two-path wsh fits one md1 string"); assert!(s.starts_with("md1"));` — comment cites M-2 by name | **FIXED** — exactly the finding's prescribed remedy; branch proven live below |
| N-1 | `compose/mod.rs` module declarations relocated vs. the plan's text (cosmetic) | No change — `git diff` on that file between BEFORE and AFTER is empty | **DECLINED, reason holds** — the finding itself graded this "harmless: no behaviour or test-power change"; leaving it as-is is consistent with that grading, and nothing regressed |
| N-2 | `md compose --json` shows a blank `--help` description | `main.rs` gains a doc comment: "Emit JSON: the origin-less template, the inline-origin template, the slot map, the taproot internal-key path and the EXPERIMENTAL marks." | **FIXED** — reproduced live, see Claims §4 |
| N-3 | `md encode`'s signature-free refusal names no way forward, unlike `md compose`'s | `parse/template.rs`: when the miniscript error contains "require a signature", the message gains " -- a spend path that needs no key. `md compose` refuses the same shape without --experimental; pass --experimental here to encode it as EXPERIMENTAL (whoever holds the preimage can spend: bearer access)" | **FIXED** — reproduced live, see Claims §2 |

---

## Claims checked

**1. CHANGELOG "Blast radius" paragraph — every named verb, route and figure.**
**TRUE.**
- `md descriptor --template` and `md address --template` parse under the
  minting disposition: `crates/md-cli/src/cmd/build.rs:65` calls
  `parse_template`, which hard-wires `Disposition::Refuse` at
  `crates/md-cli/src/parse/template.rs:2613/2623`; `build_descriptor` is
  called from `crates/md-cli/src/cmd/descriptor.rs:118` and
  `crates/md-cli/src/cmd/address.rs:38`. `md vectors` calls the same
  `parse_template` directly at `crates/md-cli/src/cmd/vectors.rs:54`.
- Live reproduction with real xpubs (`crates/md-codec/tests/compose_support.rs`
  constants) and the shape `wsh(or_d(multi(2,@0..,@1..,@2..),and_v(v:sha256(H),after(1383520))))`:
  `md descriptor --template … --key …` and `md address --template … --key …`
  both refuse with the signature-rule message; `--experimental` on either
  verb is clap's `error: unexpected argument '--experimental' found`
  (exit 2) — confirming "have NO `--experimental` opt-out yet."
- Card-input routes unaffected: minted the same shape with
  `md encode --experimental`, then fed the resulting chunks to
  `md descriptor <chunks>`, `md address <chunks>` and `md decode <chunks>` —
  all exit 0, correct descriptor/address/template recovered.
- Both example shapes are refused by the AFTER binary exactly as claimed:
  `wsh(or_d(j:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))` → `Miniscript is malleable`
  (exit 1); `wsh(and_v(v:pk(@0/<0;1>/*),and_v(v:after(100),after(500000000))))`
  → `Contains a combination of heightlock and timelock` (exit 1). Both are the
  review's own constructed examples (r0 report, M-1 section).
- "30 of 7,980" is quoted verbatim from the r0 report's M-1 measurement
  ("**30 templates flip from OLD exit 0 to NEW exit 1**" over "a generated
  corpus of 7,980 templates").
- `design/FOLLOWUPS.md` at AFTER contains a
  `### \`md-descriptor-address-template-lack-experimental\`` entry (confirmed
  by grep; added in commit `66bdf2f4`).

**2. The refusal hint fires ONLY on "require a signature"; malleability keeps the plain message.**
**TRUE.**
- Sigless `wsh` path, no `--experimental`:
  `md: template parse error: miniscript parse failed: All spend paths must
  require a signature -- a spend path that needs no key. \`md compose\`
  refuses the same shape without --experimental; pass --experimental here to
  encode it as EXPERIMENTAL (whoever holds the preimage can spend: bearer
  access)` — hint present, and "must require a signature" is still an intact
  substring.
- Malleable `wsh(or_d(j:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))`, with or without
  `--experimental`: plain `Miniscript is malleable` (or, with the flag, the
  existing "failed even with --experimental … relaxes ONLY the signature
  rule" message) — no hint text appended either way.
- Mixed timelock: plain `Contains a combination of heightlock and timelock` —
  no hint.
- Checked the vendored `rust-miniscript`
  (`vendor/miniscript/src/miniscript/analyzable.rs:152`): "All spend paths
  must require a signature" is the ONLY occurrence of "require a signature"
  in the crate, so the `contains("require a signature")` match cannot
  misfire onto a different error class.
- Both message-matching test files still pass against the AFTER binary,
  independently re-run (not accepted from the given whole-suite result):
  `cargo nextest run -p md-cli -E 'binary(cli_compose_encode_gate)'` → 3/3
  PASS; `cargo nextest run -p md-cli -E 'test(experimental_admits_a_keyless_spend_path)'`
  → 1/1 PASS. Both assert `contains("must require a signature")`, which
  still holds because the hint is appended, not substituted.

**3. M-2 branch is live (the two-path wsh really fits one string).**
**TRUE, confirmed by the passing test, not by reasoning.**
`cargo nextest run -p md-codec --test compose_lowering -E 'test(composed_templates_encode_and_round_trip_through_the_wire)'`
→ 1/1 PASS with the guard removed (now an unconditional `.expect(...)`). A
panic on `.expect` would have failed the run; it did not, so
`encode_md1_string` returns `Ok` for this exact fixture
(`keys(2,3)` + `with_lock(keys(1,1), Lock::OlderBlocks(26280))`, a 2-of-3 plus
a 1-of-1-with-older-lock wsh) and the assertion actually executes.

**4. `md compose --help` shows a description for `--json`.**
**TRUE.** Live `--help` output: `--json  Emit JSON: the origin-less
template, the inline-origin template, the slot map, the taproot internal-key
path and the EXPERIMENTAL marks` — matches `main.rs`'s new doc comment
verbatim, and no longer blank.

**5. Whole-workspace gate at AFTER (fmt/clippy/nextest 1318/1318/doctests).**
Taken as given per the brief; not re-run in full. Independently corroborated
on the touched crates only: `cargo fmt --check -p md-cli -p md-codec` → clean
exit 0; `cargo clippy -p md-cli -p md-codec --all-targets --all-features -D
warnings` → clean, no warnings.

---

## New defects introduced

None found. Specifically checked and ruled out:
- The substring match for the hint (`contains("require a signature")`) cannot
  false-fire on malleability/timelock/resource-limit/repeated-key errors —
  the string is unique to the signature-rule `Display` impl in the vendored
  miniscript crate.
- The hint is appended after the original library message rather than
  replacing it, so every existing test asserting on the original substring
  (`cmd_encode.rs::experimental_admits_a_keyless_spend_path`,
  `cli_compose_encode_gate.rs`'s two tests) still matches — confirmed by
  running all three, not by inspection alone.
- `crates/md-codec/tests/compose_lowering.rs`'s newly-unguarded `.expect`
  does not turn a previously-tolerated failure mode into a panic anywhere
  else in the suite: it is the only site the fold touched in that file, and
  the fold's own nextest run (given) plus my independent single-test rerun
  both show it green.
- No file outside the 5 touched by the two commits changed
  (`git diff --name-status 9820e618..66bdf2f4`: exactly `CHANGELOG.md`,
  `crates/md-cli/src/main.rs`, `crates/md-cli/src/parse/template.rs`,
  `crates/md-codec/tests/compose_lowering.rs`, `design/FOLLOWUPS.md`).

---

## What I ran

```sh
# diff inspection
git -C wt-composer-s0 diff --stat 9820e618..66bdf2f4
git -C wt-composer-s0 show 7c9b4fd7 --stat; git -C wt-composer-s0 show 66bdf2f4 --stat
git -C wt-composer-s0 diff 9820e618..66bdf2f4 -- CHANGELOG.md crates/md-cli/src/parse/template.rs \
    crates/md-cli/src/main.rs crates/md-codec/tests/compose_lowering.rs design/FOLLOWUPS.md
git -C wt-composer-s0 diff 9820e618..66bdf2f4 -- crates/md-codec/src/compose/mod.rs   # empty: N-1 untouched

# caller-graph confirmation
grep -n "parse_template(" crates/md-cli/src/cmd/build.rs crates/md-cli/src/cmd/vectors.rs
grep -n "fn parse_template\b|fn parse_template_ext|Disposition::Refuse" crates/md-cli/src/parse/template.rs
grep -n "build_descriptor(" crates/md-cli/src/cmd/descriptor.rs crates/md-cli/src/cmd/address.rs
grep -n "require a signature" vendor/miniscript/src/miniscript/analyzable.rs   # exactly one hit

# binary already built at AFTER (66bdf2f4) -- confirmed via `strings` for the new hint text,
# and worktree `git status --short` clean at 66bdf2f4
strings target/debug/md | grep -F "a spend path that needs no key"

# live reproduction (read-only invocations of the pre-built binary)
md encode "<sigless wsh, no --experimental>"                 # hint present, "must require a signature" intact
md encode --experimental "<same>"                             # succeeds, bearer-access warning
md encode "wsh(or_d(j:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))"        # malleable, no hint, with/without --experimental
md encode "wsh(and_v(v:pk(@0/<0;1>/*),and_v(v:after(100),after(500000000))))"   # mixed timelock, no hint
md descriptor/address --template <sigless shape> --key ...    # refused, no --experimental flag exists (exit 2 for the flag)
md encode --experimental ... | md descriptor/address/decode <chunks>            # card-input routes: exit 0
md compose --help                                              # --json now documented

# independent narrow test reruns (not the full suite)
cargo nextest run --locked -p md-codec --test compose_lowering \
    -E 'test(composed_templates_encode_and_round_trip_through_the_wire)'        # 1/1 PASS
cargo nextest run --locked -p md-cli -E 'binary(cli_compose_encode_gate)'       # 3/3 PASS
cargo nextest run --locked -p md-cli -E 'test(experimental_admits_a_keyless_spend_path)'  # 1/1 PASS

# sanity on touched crates only (whole-workspace gate taken as given per the brief)
cargo fmt --check -p md-cli -p md-codec                        # clean
cargo clippy -p md-cli -p md-codec --all-targets --all-features -- -D warnings  # clean

# provenance
git log --oneline -1 -- design/agent-reports/composer-S0-exec-review-r0.md    # 976cc45, confirmed
```

Did not read any `.jsonl` file. Nothing in either repository or the worktree
(beyond its own `target/`) was modified.
