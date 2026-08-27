# R0 VERIFICATION PASS — `SPEC_constellation_cli_uniformity.md` fold `d31beed`

**Not a design round.** The object is `git show d31beed` and nothing else. The
round it answers is `design/agent-reports/R0-cli-uniformity-spec-close.md`
(0C / 4I / 4M).

**VERDICT: NOT GREEN — 0 Critical / 3 Important / 4 Minor.**

Every command below was executed in this worktree on 2026-08-26 against the
built debug binaries. One correction to my own method, recorded because it
nearly produced a false finding: **the login shell aliases `md='mkdir -p'`**, so
`md decode notanartifact` returned exit 0 and `md decode --help` printed
`mkdir`'s usage. Every measurement in this report uses absolute binary paths.

---

# PART A — the close round's 8 findings

| # | sev | verdict | evidence |
| --- | --- | --- | --- |
| **B-1** propagation fold closed 1 of 5 sites | I | **PARTIAL** | 3 of the 4 survivors are closed (§2:61, §6d:542, §6d:545 all rewritten and correct). **§6b:351 is half-closed** — it drops `ms combine` but still reads *"the real `-` gap is `md`'s four other verbs — one targeted addition"*, omitting `mt`'s three verbs, which the same reviewer named in the same finding. Measured: `mt decode -` / `mt verify -` / `mt inspect -` each → `error: unexpected argument '-' found`, **exit 2**. §7 P1:980 says so too, so the document now contradicts itself. The gate still exits 1 on the reviewer's five phrasings. See I-1. |
| **C-a** `--out` clobber unruled | I | **CLOSED** | §6b now carries a five-line ruling naming the operator, the `.truncate(true)` mechanism, the rejected refusal, and the accepted consequence. Verified: `write_private` is `opts.write(true).create(true).truncate(true)` at `crates/me-cli/src/main.rs:859`; and empirically `me sysw pack --no-passphrase --out payload.bin` over a pre-existing 46 B 0644 file → exit 0, file becomes 102 B and 0600, `grep -c 'PRE-EXISTING' payload.bin` → **0**. A second run overwrites again at exit 0. |
| **C-b** which purge text, no gate | I | **NOT CLOSED** | The new §6d text is *factually* right in every part (all four claims re-measured, Part C rows 10–13) — but it lands in the wrong section and asserts a gate that does not exist. **§7 P0:979 still reads `Extracted FROM `mt`/`me`.`** — the exact site C-b named — untouched. §6d:486 attributes that phrasing to *§6h*, which does not contain it (§6h:913 already names `me` alone: *"The reference implementation is `me sysw pack`'s widened argv refusal"*). And C-b's requested P0 test on the zsh branch does not exist. See I-2. |
| **C-c** `mnemonic` exit cells falsify §6f | I | **NOT CLOSED — and the fold made the record worse** | The cell is now filled from **a subcommand that does not exist**: `mnemonic decode md1nonsense` → `error: unrecognized subcommand 'decode'`, exit 64. That 64 is clap's *unknown-subcommand* error, not an invalid artifact. Under verbs that exist the answer is **1 or 2**, and the 2 is exactly the collision C-c asked to be ruled in advance. See I-3. |
| **B-2** the C-3 grep prints 48 not 26 | m | **CLOSED** | Both replacement commands reproduce exactly, run from `mnemonic-toolkit`: files → **26**, references → **86** (cross-checked against the reviewer's `grep -o \| wc -l`, also 86). The retracted command reproduces its stated failure: plain `grep -rl … | wc -l` → **48**. |
| **B-3** §5a decided what §7 P0 still tells the plan to decide | m | **NOT CLOSED** | Untouched by the fold. `:1030` still *"P0 — the distribution mechanism, which no earlier draft named"*; `:1045` still *"**P0 must name which mechanism it uses.**"*; `:1044` still *"shipping a change to **four consumers**"* against D5:228 *"all five"* and §5a:275 *"the **fifth consumer**"*. |
| **B-4** §8a's sigil count is stale | m | **PARTIAL** | The occurrence count is now right and the distinct count is not. §8a's own command run verbatim prints **21** lines over **64** occurrences; §8a claims *"18 over 64"*. 18 is the count only after normalising three trailing sentence periods (`§6h.`, `§9a.`, `§2.`). Same class the fold set out to eliminate. See M-3. |
| **C-d** no crate name; consumer set stated three ways | m | **NOT CLOSED** | Untouched. No crate name anywhere in the document (grepped `crate name`, `name the crate`, `named crate` — zero hits). Consumer count still 4/5/fifth across three sections. `me`'s own consumer status still unstated. |

**Close-round tally carried forward: 1 Important CLOSED, 1 Minor CLOSED.
The other six are PARTIAL or NOT CLOSED.**

---

# PART B — defects in `d31beed`'s new text

## I-1 (Important) — §6b:351 still states an exhaustive `-` gap that omits `mt`×3, and the commit message reports the gate clean when it exits 1

Two halves, one site.

**The text.** §6b:351, as rewritten by this fold:

> **So the real `-` gap is `md`'s four other verbs** — one targeted addition plus a `combine` DOC fix, not a constellation rollout.

Measured this round:

```
$ echo "" | mt decode  -   ->  error: unexpected argument '-' found   exit 2
$ echo "" | mt verify  -   ->  error: unexpected argument '-' found   exit 2
$ echo "" | mt inspect -   ->  error: unexpected argument '-' found   exit 2
```

So the gap is `md`×4 **plus `mt`×3**, which is what §7 P1:980 already says
verbatim (*"`-` on `decode`, `verify` and `inspect` — F-250 fixed `encode`
ALONE, and the other three still exit 2 (I-3)"*) and what §7:1102 repeats. The
document now asserts both. The close round named this half explicitly — *"False
twice: `combine` is not a gap, and `mt`'s `decode`/`verify`/`inspect` are"* —
and gave the cost: an author scoping `-` from §6b builds `md`×4 instead of
`md`×4 + `mt`×3. The fold halved that cost and did not remove it.

**The record.** `d31beed`'s message ends:

```
   fold-propagation     -> the reviewer's five phrasings, all gone
```

Run in this worktree at `d31beed`, with the close round's five phrasings and no
substitutions:

```
$ ./scripts/fold-propagation-check.sh design/SPEC_constellation_cli_uniformity.md \
    'not .combine.' 'positionals ONLY' 'ordering constraint that makes P2 non-negotiable' \
    'real .-. gap' 'confined to the two rulings'
  gone   not .combine.
  gone   positionals ONLY
  gone   ordering constraint that makes P2 non-negotiable
  LEFT   real .-. gap
           351:  `md` documents it on `repair` alone. **So the real `-` gap is `md`'s four
  gone   confined to the two rulings
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.        (exit 1)
```

**Exit 1, not clean.** The script's own header says a surviving hit may be a
legitimate historical note and must be *read* — but this one is not a historical
note, it is the live claim, and the message did not report a judged hit. It
reported the gate clean. This is the second consecutive fold commit whose
message states a false result for this gate.

**What closes it.** §6b:351 reads *"the real `-` gap is `md`'s four other verbs
**and `mt`'s `decode`, `verify` and `inspect`** (F-250 fixed `encode` alone) —
seven targeted additions plus a `combine` DOC fix"*, or drops the exhaustive
framing. Then re-run the gate and read the surviving hit rather than reporting
it gone.

## I-2 (Important) — the purge ruling landed in §6d, asserting a §7 gate item that §7 does not contain, and citing a phrase to a section that does not hold it

`d31beed` added to §6d:477:

> Give the purge commands, per §6h. **The text comes from `me`, and `mt`'s is
> SUPERSEDED — P1 replaces it, and it is a gate item rather than a courtesy.**
> … **So §6h's "from `mt`/`me`" is wrong as written**: from `mt` it ships the trap.

Three problems, none of them with the facts:

1. **The phrase is not in §6h.** `grep -n 'mt./.me.'` over the document returns
   exactly two lines: **486** (this new sentence) and **979** (§7 P0). §6h is
   already correct — it names `me` alone as the reference implementation. The
   sentence corrects a section that did not need it and cites a section that
   does not contain the text.
2. **§7 P0:979 is untouched** and still reads `remedy text per §6h, … Extracted
   FROM `mt`/`me`.` That is the site C-b named, and it is the site a plan author
   reads to decide what to lift. C-b's stated fix — *"§7 P0 says 'remedy text
   from `me` — `mt`'s is superseded and P1 replaces it'"* — was not applied.
3. **The claimed P1 gate item does not exist.** §7's P1 row content is `--out`,
   `--allow-argv-secret`, and `-` on three verbs; its gate is the 237 tests plus
   the three `-` assertions. No purge-text work, no purge-text gate. §7:1102
   still says P1's diff is *"three items, not two"*, which this fold's own
   sentence would make four. C-b's second requested item — a test asserting the
   zsh branch does not advise `history -d` — exists in no phase.

The underlying facts all hold (Part C rows 10–13): `mt`'s `purge_command()` is
at `mt-cli/src/validate.rs:541`, its zsh branch is `history -d $HISTCMD && fc
-W`, its fish branch anchors on the material, `history -d` deletes nothing on
zsh 5.9.2, and `me`'s shipped text names that trap. **The defect is that a
correct ruling was written where nothing will execute it.**

**What closes it.** Fix §7 P0:979 to `Extracted FROM `me``; add the replacement
and its assertion to §7's P1 row (and update §7:1102's "three items"); correct
§6d:486 to cite §7 P0 rather than §6h.

## I-3 (Important) — §6f's new `mnemonic` invalid-artifact cell is measured with a subcommand that does not exist

`d31beed` filled the cell and added:

> `mnemonic decode md1nonsense` → **64** (invalid artifact) … **Neither needs
> changing.** The 64 does not collide with `mk`'s invalid-artifact 2 — it is
> clap's usage code…

Run:

```
$ mnemonic decode md1nonsense
error: unrecognized subcommand 'decode'

  tip: a similar subcommand exists: 'decode-address'
exit=64
```

**`mnemonic` has no `decode` subcommand.** Verified against both the debug build
and `~/.cargo/bin/mnemonic` (both 0.97.0), and against the full subcommand list:
the m-format reading verbs are `inspect`, `convert` and `repair`. The 64 is
clap's *unrecognised subcommand* error. The reasoning that follows it is
circular: the number *is* clap's usage code, because the command does not exist.

Under verbs that do exist, alias-free, absolute paths, stdin at `/dev/null`:

| input | `md decode` | `mk decode` | `ms decode` | `mnemonic inspect` |
| --- | --- | --- | --- | --- |
| `notanartifact` | 1 | 2 | 1 | **2** (unknown HRP) |
| `md1nonsense` | 1 | 2 | 1 | **1** (decode failure) |

So the cell's real value is **1 or 2 depending on the input shape**, and the 2 is
precisely the collision C-c asked to be ruled in advance: *"states in advance
what happens if `mnemonic`'s number matches `mk`'s"*. The fold's *"Neither needs
changing"* and *"`mk`'s 2 → 1 remains the only code this cycle changes"* rest on
a number no real invalid-artifact path produces.

The repair cell is sound: `mnemonic repair md1zzzzzzzz8xtwhw4xwn4qh` → **2**
(re-measured), matching `md`'s 2.

**What closes it.** Name a verb that exists — `mnemonic inspect` — state which
input shape the "invalid artifact" column means for a tool whose positional
self-identifies by HRP, and rule the `mk` collision the way C-c asked.

## M-1 (Minor) — B-3 unfolded

`:1030`, `:1044`, `:1045` unchanged; four-vs-five consumers still contradicts
D5:228 and §5a:275, and §7 still instructs P0 to decide a mechanism §5a decided.

## M-2 (Minor) — C-d unfolded

No crate name anywhere; `me`'s own consumer status still unstated.

## M-3 (Minor) — §8a's repaired sigil count still does not match §8a's command

The command as printed yields **21** distinct sigils over **64** occurrences;
the text says **18 over 64**. 18 requires normalising `§6h.` → `§6h`, `§9a.` →
`§9a`, `§2.` → `§2`, which the printed command does not do. For calibration, at
`ceec3b7` the same raw command gave **20 / 62** — exactly the reviewer's figure —
so the occurrence half was repaired against the command and the distinct half
was not. **The fold's own headline lesson, reproduced one line below where it is
stated.**

## M-4 (Minor) — the fold filled the `mnemonic` cells and left two sections ordering the plan to fill them

- §6f:797 — *"The plan **must fill the two `mnemonic` cells still marked 'not
  measured'**"*. The fold filled both, 100 lines above, in the same section.
- §7 P0:979 gate — *"the two `mnemonic` exit cells still marked 'not measured'
  filled"*.

`grep -n 'not measured'` returns no `mnemonic` exit-code cell. A plan author
looking for the cells the gate names finds none. This is the same
fold-falsifies-adjacent-text shape the cycle keeps hitting, and it is new in
`d31beed`. (It also removed the one gate that would have caught I-3 before P0.)

---

# PART C — every command in the fold's new text

| # | command (as the fold states it) | printed | matches? |
| --- | --- | --- | --- |
| 1 | `ms split --phrase <all-abandon> -k 2 -n 3 --group-size 0 \| head -2 > shares.txt` then `ms combine - < shares.txt` | `entropy: 000…0`, `phrase: abandon … about`, **exit 0** | **YES** — the fold's central new fact holds |
| 2 | `-` documented on 7 of 8 `ms` verbs | help mentions stdin on encode/decode/verify/inspect/repair/split/derive; **combine: 0 mentions** | **YES** |
| 3 | `-` implemented on all 8 | `decode -`, `verify -`, `inspect -`, `derive -`, `repair --ms1 -`, `split --phrase -`, `encode --phrase -`, `combine -` → **all exit 0** | **YES** |
| 4 | `ms combine` has no `--in` | `error: unexpected argument '--in' found` | **YES** |
| 5 | `mt encode --quiet --bitcoin-cli /nonexistent --in even.hex`, stderr piped → six strings, exit 0 | `grep -c '^mt1'` → **6**, exit **0** (stdout 0600; a 0644 stdout trips the F-252 refusal at exit 1) | **YES** |
| 6 | `--quiet` gives **70** stderr lines against **108** | `2>&1 >/dev/null \| wc -l` → **70** quiet, **108** plain, delta 38 | **YES** — reproduces only with the offline flag AND stderr piped, both of which the fold now states |
| 7 | none of `TX`/`CUT`/`PREFIX` appear under `--quiet` | **0** quiet, **3** plain | **YES** |
| 8 | `git ls-files '*.rs' \| xargs grep -l 'secret_in_argv_warning' \| wc -l` → 26 | **26** | **YES** |
| 9 | `git ls-files '*.rs' \| xargs grep -c … \| grep -v ':0' \| awk -F: '{s+=$2}'` → 86 | **86** (cross-checked `grep -o \| wc -l` → 86) | **YES** |
| 10 | plain `grep -rl …` prints 48 | **48** | **YES** |
| 11 | `mt`'s `purge_command()` at `mt-cli/src/validate.rs:541` | `541:fn purge_command() -> &'static str {` | **YES** — exact line |
| 12 | its zsh branch is `history -d $HISTCMD && fc -W`; fish branch `history delete --contains <tx>` | `:543` and `:544`, verbatim | **YES** |
| 13 | `history -d` does not delete on zsh 5.9.2 — reports success, purges nothing | zsh 5.9.2; `history -d $HISTCMD` → exit **0**, prints the history WITH timestamps, entry still present afterwards (`fc -l 1 \| grep -c SECRET…` → **1**). `history 1` vs `history -d 1` differ only by a timestamp column | **YES** — `-d` is a display flag, confirmed empirically |
| 14 | `me`'s text names the command and calls out the zsh trap | `me-cli/src/main.rs:2014` `sed -i '/me sysw pack/d' "$HISTFILE"`; `:2017` *"On zsh, `history -d` does NOT delete -- -d prints timestamps"* | **YES** |
| 15 | `write_private` is `.truncate(true)` | `me-cli/src/main.rs:859` | **YES** |
| 16 | `me sysw pack --out payload.bin` destroys the first artifact on a re-run | pre-existing 46 B 0644 file → 102 B 0600, old content gone, exit 0, twice | **YES** |
| 17 | `mnemonic decode md1nonsense` → 64 (invalid artifact) | `error: unrecognized subcommand 'decode'` — **the subcommand does not exist** | **NO** — I-3 |
| 18 | `mnemonic repair <an uncorrectable md1>` → 2 | `md1zzzzzzzz8xtwhw4xwn4qh` → **2** | **YES** |
| 19 | §8a sigil sweep → 18 distinct over 64 | **21** distinct over **64** | **PARTIAL** — M-3 |

## Gates, re-run in this worktree at `d31beed`

```
$ ./scripts/spec-structure-check.sh design/SPEC_constellation_cli_uniformity.md
  sections: 24 ; cross-refs checked: 18
  STRUCTURE OK                                                      (exit 0)

$ ./scripts/plan-table-check.sh design/SPEC_constellation_cli_uniformity.md
  table rows checked: 67 ; malformed: 0                             (exit 0)

$ ./scripts/fold-propagation-check.sh <spec> <the close round's FIVE phrasings>
  LEFT   real .-. gap    351                                        (exit 1)
```

Two of the three commit-message gate lines are true. The third is false.

---

# Observations, deliberately NOT counted as findings

- **§6h contradicts itself about zsh, and predates this fold.** §6h says both
  *"the builtin rejects the invocation"* and *"Advising it would report success
  while purging nothing."* Measured: `history -d $HISTCMD` **succeeds** and
  lists; only a bare `history -d` with no event errors. The fold's new §6d text
  picks the correct half. Pre-existing text, no fold touched it, no conclusion
  moves.
- **§2's `mt` cell** reads *"default, plus bare `-`"*, true for `encode` only.
  Not named by any round, not touched by this fold; it is the same fact as I-1
  and closing I-1 is the natural moment to look at it.
- **§7 P2 still spells its sequence FIRST/THEN** after §6d:578 demoted it to a
  preference. Not a contradiction — a preference may still be written as an
  order — and §7 never calls it non-negotiable.
- **`ms combine`'s `-` is undocumented, and the argv refusal names the channel.**
  §6d requires the refusal to *"Name the private channels: `--in FILE`, `-` for
  stdin"*, so refusing argv on `combine` before the `--help` line lands does not
  strand an operator. The fold's demotion of the ordering constraint is sound.
- **Method note.** The login shell aliases `md='mkdir -p'`. A first pass through
  the exit-code comparison returned `md decode notanartifact` → **0** and would
  have been a fabricated Critical. Absolute paths throughout.

---

# Counts

| severity | items |
| --- | --- |
| **Critical (0)** | — |
| **Important (3)** | I-1 §6b:351 omits `mt`×3 and the gate line in the commit message is false; I-2 the purge ruling asserts a P1 gate item §7 does not have and leaves §7 P0's `FROM mt/me` standing; I-3 `mnemonic decode` does not exist, so the invalid-artifact cell is unmeasured and the `mk` collision is unruled |
| **Minor (4)** | M-1 B-3 unfolded; M-2 C-d unfolded; M-3 §8a says 18 where its command prints 21; M-4 two sections still order the plan to fill cells this fold already filled |
| **Nit (0)** | — |

**VERDICT: NOT GREEN (0C / 3I / 4M).**

## What closes it — three edits, all mechanical

1. **§6b:351** — add `mt`'s `decode`/`verify`/`inspect` to the `-` gap, or drop
   the exhaustive framing. Verified: each exits **2** on `-`. Then re-run
   `fold-propagation-check.sh` with the close round's five phrasings, **read**
   the surviving hit, and report what it says rather than that it is gone.
2. **§7 P0:979** — `Extracted FROM `me``; move the purge replacement into §7's
   P1 row as a real work item with a real gate assertion (and reconcile
   §7:1102's *"three items"*); correct §6d:486 to cite §7 P0, not §6h.
3. **§6f** — re-measure the invalid-artifact cell with `mnemonic inspect`
   (**1** on `md1nonsense`, **2** on `notanartifact`), state which shape the
   column means, and rule the `mk` collision in advance as C-c asked. While
   there, clear the stale *"still marked 'not measured'"* orders at §6f:797 and
   §7:979.

The four Minors are one-line edits and do not gate.

**On whether a further round is warranted.** No. Every item above is a
transcription or propagation error in text this fold wrote, and each has its
value already measured and quoted here — none needs a reviewer to decide
anything. The design questions remain closed: I constructed no reading that
reopens C-1, C-2, C-3, §5a or D7, and the fold's central new fact (`ms combine -`
works) is true and was verified independently. **After these three edits, the
next pass should be a claim-check against this report's Part C, not a round.**
