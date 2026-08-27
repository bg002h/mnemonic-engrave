# CLAIM-CHECK — `SPEC_constellation_cli_uniformity.md` fold `cca3b86`

**Not a review round.** The object is `git show cca3b86` and nothing else. It
answers `design/agent-reports/R0-cli-uniformity-spec-verify.md` (0C/3I/4M).
Every command below was run against the built debug binaries by absolute path,
with stdin from `/dev/null` where relevant, in this worktree on 2026-08-26.
No number below is transcribed from either document — each was re-executed.

**VERDICT: NOT GREEN — 0 Critical / 1 Important / 0 Minor.**

The 3 Importants and 4 Minors the fold answers are all CLOSED. The one
Important below is new: a self-contradiction the fold introduced across two
sites it edited in this same commit, while closing two of those seven items.

---

## Part A — the verify round's 7 findings

| # | sev | verdict | evidence |
| --- | --- | --- | --- |
| I-1 | I | **CLOSED** | §6b:369 now reads "the gap is `md`'s four other verbs PLUS `mt`'s `decode`, `verify` and `inspect`" with all three measured at exit 2 on `-` and `encode -` noted as F-250's sole fix. Matches measured: `mt decode -` / `verify -` / `inspect -` → exit 2 each; `mt encode -` → exit 1. §7 P1 already agreed; text now matches. |
| I-2 | I | **CLOSED** | §7 P0 (`:1045`) now reads "remedy text per §6h (**from `me` ALONE — `mt`'s zsh branch is superseded, §6d**) … Extracted FROM `me`; `mt`'s purge text is NOT a source." `grep -n '`mt`/`me`'` over the document now returns **zero** hits (previously 486 and 979). §6h (`:960`) already named `me` alone and is untouched. |
| I-3 | I | **CLOSED** | §6f's cell is now "1 or 2 — by input shape (see below)," derived from `mnemonic inspect`, not the nonexistent `mnemonic decode`. Re-measured this round: `mnemonic inspect notanartifact` → **2**, `mnemonic inspect md1nonsense` → **1**. The `mk` collision is ruled explicitly (see Part C, ruling 1). |
| M-1 | m | **CLOSED (as scoped)** | All three cited phrases gone: `grep -n "no earlier draft named"` → 0 hits; `grep -n "four consumers"` → 0 hits; `grep -n "must name which mechanism"` → 0 hits. §7:1093 now reads "all five consumers" with a parenthetical reconciling it to D5/§5a. **This exact fix is also the site of the new Important below.** |
| M-2 | m | **CLOSED** | Crate named: `**\`m-cli-io\`**` at §5a:278, on the `mt-codec`/`wc-codec` precedent, with a P0 availability-check gate. `me`'s consumer status stated: "All six binaries are consumers, `me` included (C-d)" at §5a:290, backed by a verified fact (`me-cli/src/main.rs:252` is an ordinary `#[arg(long)]` flag, not raw-argv parsing) and an owner ("P0 owns it"). |
| M-3 | m | **CLOSED** | §8a no longer states a live count. It states the historical "16 distinct sigils over 50 occurrences… when it was written," then: "**No current count is quoted here, deliberately**… it has been wrong three times, including once in the same fold that added the sentence warning about it." The command is left as the claim. |
| M-4 | m | **CLOSED** | `grep -n "must fill"` → 0 hits. §6f (`:838`) now reads "All four `mnemonic` cells are now measured… so nothing here is left for the plan to fill." §7 P0's gate (`:1045`) now reads "§6f's `mnemonic` invalid-artifact cell re-measured under a verb that EXISTS — `inspect`, not `decode` (I-3)" — the gate that would have caught I-3 is restored. |

**7 of 7 CLOSED as scoped.**

---

## Part B — every new factual claim, run

| # | claim | command | printed | matches? |
| --- | --- | --- | --- | --- |
| 1 | `mt decode -` → 2 | `mt decode -` < /dev/null | `error: unexpected argument '-' found`, exit **2** | YES |
| 2 | `mt verify -` → 2 | `mt verify -` < /dev/null | same, exit **2** | YES |
| 3 | `mt inspect -` → 2 | `mt inspect -` < /dev/null | same, exit **2** | YES |
| 4 | `mt encode -` → 1 | `mt encode -` < /dev/null | `REFUSED — §8.2e, input is not a PSBT…`, exit **1** | YES |
| 5 | `mnemonic` has no bare `decode` | `mnemonic decode md1nonsense` | `error: unrecognized subcommand 'decode'`, exit **64** | YES |
| 6 | `mnemonic inspect notanartifact` → 2 | ″ | `does not begin with a recognized HRP prefix`, exit **2** | YES |
| 7 | `mnemonic inspect md1nonsense` → 1 | ″ | `md1 codex32 decode: character 'o' not in codex32 alphabet`, exit **1** | YES |
| 8 | `mnemonic repair <uncorrectable>` → 2 | `mnemonic repair md1zzzzzzzz8xtwhw4xwn4qh` | `too many errors to correct uniquely`, exit **2** | YES |
| 9 | a clap usage error on `mnemonic` → 64 | `mnemonic this-verb-does-not-exist` | `unrecognized subcommand`, exit **64** | YES |
| 10 | `mk decode notanartifact` → 2 (the ruled collision) | `mk decode notanartifact` | `error: invalid HRP: notanartifact`, exit **2** | YES |
| 11 | `crates/me-cli/src/main.rs:252` is `allow_argv_secret: bool` under `#[arg(long)]` | `sed -n '250,254p'` | L250 doc comment, L251 `#[arg(long)]`, L252 `allow_argv_secret: bool,` | YES — exact line |
| 12 | §7 P0 no longer sources remedy text from `mt`; §6h names `me` alone | grep | `:1045` "Extracted FROM `me`; `mt`'s purge text is NOT a source"; `:960` "reference implementation is `me sysw pack`'s widened argv refusal" | YES |
| 13 | no surviving "four consumers" | `grep -n "four consumers"` | 0 hits | YES |
| 14 | no surviving "must name which mechanism" | `grep -n "must name which mechanism"` | 0 hits | YES |
| 15 | no surviving "no earlier draft named" | `grep -n "no earlier draft named"` | 0 hits | YES |
| 16 | no surviving literal "mnemonic decode" (the retracted re-minted string) | `grep -n "mnemonic decode"` | 0 hits | YES |

**16 of 16 factual claims true.**

---

## Part C — the three rulings the fold authored

### Ruling 1 — the `mk`/`mnemonic` invalid-artifact collision stands

§6f (`:743–753`) rules the collision explicit rather than deferred: `mk`'s 2 and
`mnemonic`'s 2 "do not mean the same thing" (artifact-invalid vs.
not-an-m-format-artifact-at-all), `mnemonic` sits in a different tier (§9a) the
shared crate cannot reach, and `mk`'s own 2→1 change is ruled for an unrelated
reason (disagreeing with `md`/`ms` on the same question).

Checked for contradiction:
- `D26` **does exist** in this document (`:823`, cited from
  `mnemonic-toolkit/design/SPEC_followup_toolkit_v0860_demote.md`), but it
  governs the **repair-uncorrectable** 4-vs-5 split, a different pair of table
  cells entirely — not the invalid-artifact column this ruling addresses. No
  contradiction.
- §6f's own surrounding text (`:810–825`) discusses two *other* collisions
  (`md encode`/clap's 2, `ms repair`'s 4 vs `me sysw pack`'s 4) and explicitly
  defers the general "2 vs 64 clap-usage split" to an owning phase — neither
  requires `mnemonic`'s invalid-artifact code to change.
- No other site in the document asks for `mnemonic`'s invalid-artifact code to
  converge (`grep -n -i "converg\|unify"` returns only the `mk`→1 line, `:831`).

**CONSISTENT.**

### Ruling 2 — the crate name `m-cli-io`

`grep -n "m-cli-io"` → exactly one hit, `:278`. No other candidate name for the
same crate appears anywhere (`cli-io`, `io crate` search turns up only generic
references to "the shared crate," never a second proposed name). `mt-codec` and
`wc-codec` are cited only as **naming precedent**, not as alternate names for
this crate.

**CONSISTENT.**

### Ruling 3 — "`me` is a consumer, not only a donor" — **NOT CONSISTENT**

This is the fold's own new authorship, and it contradicts a second piece of the
fold's own new authorship, four sections away, in the same commit.

**Count, done directly:** the constellation has six binaries — `md`, `mk`, `ms`,
`mt`, `me`, `mnemonic` — all six built and exercised in Part B above.

- §5a:290 (**new in this fold**): *"**All six binaries are consumers, `me`
  included** (C-d)."* — asserts the total is **6**.
- §7:1093–1095 (**also new in this fold**, the very edit that closed M-1):
  *"…shipping a change to **all five consumers**… (**Five**, matching D5 and
  **§5a, which counts the toolkit as the fifth**; an earlier revision said four
  here.)"* — asserts the total is **5**, and explicitly claims that count is
  what §5a says.
- D5 (`:228`, untouched): *"depended on by all five."* — a third site still at
  5.
- §5a:275 (untouched, two paragraphs above the new "all six" sentence):
  *"…the toolkit becomes the **fifth consumer**"* — the same section states
  both 5 (at line 275) and 6 (at line 290) fifteen lines apart.

§7:1093–1095's parenthetical is not merely stale — it is a **false description
of what §5a currently says**, written in the same commit that rewrote §5a to
say the opposite. This is the fold-falsifies-adjacent-text shape the cycle has
repeatedly named (I-2 and M-4 above are both this shape), and here it sits
inside the very edit that closed M-1.

What is **not** ambiguous: the actionable item survives regardless of the count
— §5a:296 says outright "**P0 owns it**" for migrating `me`'s
`allow_argv_secret` from an ordinary clap flag to raw-argv parsing. A plan
author following that sentence builds the right thing. But the document has no
single stated answer to "how many consumers does D5's crate have," which is
exactly the kind of number this cycle has twice now (B-3, M-1) had to walk back
from a stale draft.

**NOT CONSISTENT. Contradicting sites: §5a:275, §5a:290, §7:1093–1095, D5:228.**

---

## Counts

| severity | items |
| --- | --- |
| Critical (0) | — |
| **Important (1)** | The consumer-count self-contradiction: §5a (`:290`, new) says six binaries are consumers, `me` included; §7 (`:1093–1095`, also new, same commit) says five and explicitly attributes that count to §5a; D5 (`:228`) and §5a's own untouched "fifth consumer" (`:275`) still say five. Two of the four sites are this fold's own new text, contradicting each other. |
| Minor (0) | — |
| Nit (0) | — |

**VERDICT: NOT GREEN (0C / 1I / 0M).**

## What closes it

One edit: pick five or six and say it once. Either (a) update §7:1093–1095's
parenthetical and D5:228 and §5a:275 to six, since §5a:290 makes `me` a
consumer too, or (b) if `me`'s status is meant to be tracked separately from
the "five" the crate is *distributed to* (i.e., `me` hosts and uses its own
donated code without counting as a distribution target), say that distinction
explicitly at §5a:290 rather than flatly asserting "all six binaries are
consumers." No re-measurement needed — every number in this finding is a grep,
not a build.

**On whether a further round is warranted.** No design question reopens: both
rulings 1 and 2 hold, all 7 named findings are genuinely closed, and 16 of 16
factual claims in the fold's new text are true. The one Important is a
same-commit propagation miss of the exact class this document keeps
re-discovering (four sites now disagree, two of them written together) — a
one-line arithmetic reconciliation, not a design defect. The next pass, after
that one edit, should confirm the edit and stop.
