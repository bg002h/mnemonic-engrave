# R0 — architect review, round 1 (fold verification)

**Artifact:** `design/SPEC_constellation_cli_uniformity.md` as folded (commit `87d080a`,
merged at `08c803c`).
**Reports under review:** `design/agent-reports/R0-cli-uniformity-spec-round0.md`
(4C/11I/6M/2N) and `design/agent-reports/R0-cli-uniformity-fold-round0.md`.
**Reviewer:** independent context, worktree `/scratch/code/shibboleth/_work/r0r1/mnemonic-engrave`
on `review/r0-round1`.
**Scope, as briefed:** exactly two questions — did the fold fix each round-0 finding, and
did the fold introduce a new defect. No fresh audit; no re-opening of D1–D6, C-1's
resolution, or the 2026-08-26 argv ruling.

**Verdict up front: NOT GREEN. 0 Critical / 5 Important / 2 Minor / 2 Nit.**

Binaries executed (nothing below is described from help text alone):

```
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md
/scratch/code/shibboleth/mnemonic-key/target/debug/mk
/scratch/code/shibboleth/mnemonic-secret/target/debug/ms
/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/target/debug/mt
/scratch/code/shibboleth/mnemonic-engrave/target/debug/me
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic
```

---

# Part A — disposition of the 23 round-0 findings

| ID | Result | Evidence |
| --- | --- | --- |
| **C-1** | **CLOSED** | §6g carries the ruling (opt-in, keyed on kinds, incomplete set REFUSES); D6 added to §5; §2's retracted sentence removed. Mechanism corrected to the HRP discriminant and verified in source: `mdmk_unconfirmed` (`sysw/record.rs:168`) matches `('d', Some(_)) => md_codec::reassemble`, `('d', None) => md_codec::decode_md1_string`, `('k', _) => mk_codec::decode`. Reproduced: 1 of a 2-chunk `mk1` set → warning, payload written, **exit 0**. (Two numbers inside §6g are false — see B2.) |
| **C-2** | **CLOSED** | §10 rewritten from what the tools consume. Reproduced verbatim: `md encode "wsh(multi(2,[aabbccdd/…]xpub6Den8…/<0;1>/*))"` → `md: template parse error: template contains no @i placeholders`, **exit 1**. Two-stage form, `--from-md1-set FILE` named as the one piece of new surface with P3 owning it, negative half added. |
| **C-3** | **CLOSED** | §6e retracts the terminal-gate lift and scopes it to `me`'s binary container; `mt`'s print-to-terminal recorded as deliberate. Reproduced: `script -qec "mt encode --quiet --in tx.hex" /dev/null` → **exit 0**, strings printed. (The string *count* in that sentence is wrong — B6.) |
| **C-4** | **CLOSED** | §6d makes the pre-parser ordering normative, extends it to the override's own parse, and rules admitted material never returns to clap as a positional. Clap's echo reproduced: `mt encode --qr deadbeefcafe` → `error: invalid value 'deadbeefcafe' for '[-]'`, **exit 2**. |
| **I-1** | **CLOSED** | §2 cells corrected and both re-measured: `mk` documents `-`/stdin on all 5 artifact verbs and emits no header; `md` documents `-` on `repair` alone. §8 bullet 1 struck. P3's gate pinned to a chunking policy. |
| **I-2** | **CLOSED** | P1's gate is now *enumerate the diff and justify each edit*. 236 re-verified (`grep -ro '#\[test\]' --include='*.rs' crates \| wc -l` = 236, three consecutive runs). §6b rules `mt` gains `--out`, and the citation check holds up — see I-2's companion, the §4 provenance fix, below. |
| **I-3** | **CLOSED** | §6d carries a per-verb channel table for all eight `ms` verbs; P2 reordered `-`/`--in` first. Narrowing verified: stdin/`-` is documented on 7 of 8 (`encode decode verify inspect repair split derive`); **`combine` alone has neither** (0 stdin mentions in `ms combine --help`). |
| **I-4** | **PARTIAL** | The table exists and every `md`/`mk`/`ms`/`mt` cell reproduces (clap 2/64/64/2; invalid artifact 1/2/1/1; repair-applied 5/5/4; repair-uncorrectable 2/2). **But the `mnemonic` repair-applied cell says "5 per D26" in a table headed "Measured, every cell run during the fold", and it is not 5 for a non-chunked `md1`.** See **B3**. |
| **I-5** | **CLOSED** | §7 P0 gains the distribution sub-step. Verified in `me-cli/Cargo.toml`: `md-codec = "0.42"`, `mk-codec = "0.4"`, `ms-codec = "0.7"`, and `mt-codec = { git = …, rev = "72b79b87…" }`. Versions re-measured: md-cli 0.13.0, mk-cli 0.13.0, ms-cli 0.16.0, mt-cli 0.1.0, me-cli 0.7.0. Mixed states declared acceptable. (Line citation now stale — B7.) |
| **I-6** | **CLOSED** | §4's rule carries D3's qualifier; the absolute form is named as what would delete `md`'s and `mk`'s positionals, with the watch-only reasoning quoted. |
| **I-7** | **CLOSED** | §6a scopes the rule to `encode`; per-verb stdout table added; `decode` explicitly out of scope; `verify`/`inspect` declared report verbs. Spot-verified: `ms inspect <ms1>` prints an `OK:` line plus **8** labelled fields, matching the cell. Added to §9. |
| **I-8** | **CLOSED** | §6b retracts "uniform" and replaces it with a measured three-row table; the exclusion now stands on cost and is filed to a later cycle; §9 updated. |
| **I-9** | **CLOSED** | §1's cell reads **exit 1**, with the source confirmation. `grep -n 'exit 3' spec` returns one hit, and it is the correction sentence itself. |
| **I-10** | **CLOSED** | §2a names `mnemonic-engrave`; P2 carries the migration. Re-measured: `grep -rn -- '--phrase\|--hex\|--ms1' design/journeys/*.sh \| wc -l` = **18** across **7** files. Golden correction holds: `git ls-files design/journeys/out` returns nothing; **7** tracked files under `design/journeys/` carry `chunk-set-id:` (5 `.txt`, 2 `.sh`). (The all-of-`design/` figure is now off by one — B8.) |
| **I-11** | **CLOSED, and the widening verified** | `const SEPARATORS` in **4** files at exactly `md.rs:24`, `mk.rs:15`, `ms.rs:33`, `mnemonic.rs:47`; `default_value: Some("5")` at exactly **8** sites (`md.rs:77`, `mk.rs:71`, `ms.rs:78`, `ms.rs:414`, `mnemonic.rs:332/1281/1960/2050`). Drift-gate scope quote matches `schema_mirror_defaults_drift.rs`. Premise correction verified: `mt --help \| grep -c gui-schema` = **0**. |
| **M-1** | **CLOSED** | §6c's stderr table added; D4's consequence (inventing a card for `md` and `mk`) stated with P3 owning the contents; the `2>/dev/null` consequence recorded. |
| **M-2** | **CLOSED** | Re-measured, each fed **its own** hyphen output: `md decode md1yq-pqqxq-q8xtw-hw4xw-n4qh` → exit 0; `ms decode ms10e-ntrsq-…` → exit 0; `mk decode` on its two hyphen-grouped chunks → exit 0. All three offer `comma` (`space\|hyphen\|comma`), as does `mnemonic bundle`. |
| **M-3** | **CLOSED** | F-245's interaction recorded in §6c as a plan reconciliation item, with the reproduction. |
| **M-4** | **REJECTED — and the rejection is sound** | Declined in §6d with the reason. Confirmed against the settled ruling: `crates/me-cli/src/sysw/record.rs:105` defines `is_argv_forbidden`, and the shipped refusal names `--allow-argv-secret`. A rename would be a rename of shipped surface. Correctly not re-filed. |
| **M-5** | **CLOSED** | §6b now reads *"stdout is used when `--out` is not given"*, with the ambiguity named. |
| **M-6** | **CLOSED** | §8 bullet 2 struck with the reproduction and the diagnosis (a space-joined single value is a usage error). |
| **N-1** | **CLOSED** | §7 P0's distribution list carries both paths; re-verified that `_work/p3b/mnemonic-transaction` is a worktree of `/scratch/code/shibboleth/mnemonic-transaction`. |
| **N-2** | **CLOSED** | §8 gains the record-hygiene item naming F-246/250/251/252/253. |

**Also checked, because the fold's most consequential correction rests on it.** The
fold's "found in the fold" item 2 replaced an unsourced block quote in §4 with a sentence
it claims is verbatim from `SPEC_mt_v0_1` section 3b. **It is.** A line-based `grep`
returns 0 because the sentence wraps, but whitespace-normalised it appears exactly once:

```
$ python3 -c "import re,sys; t=open(sys.argv[1]).read(); print(re.sub(r'\s+',' ',t).count('stdout carries the artifact, stderr carries everything the human must see'))" \
    /scratch/code/shibboleth/_work/p3b/mnemonic-transaction/design/SPEC_mt_v0_1.md
1
```

At lines 1660–1661, and section 3b runs 1490–1672, so the attribution is correct.
The superseded sentence it replaced is absent (0 hits). **The re-citation holds.**

**Propagation sweep — clean.** Every superseded phrasing appears only inside an explicit
retraction frame, and no old `§6.N` subsection reference survives the relettering:

```
$ grep -c '§6\.[0-9]'                   -> 0
$ grep -n 'exit 3'                      -> 1 hit, the correction sentence (line 41)
$ grep -n 'already uniform'             -> 3 hits: 2 unrelated, 1 the retraction (line 303)
$ grep -n 'suite unchanged'             -> 1 hit, the correction (line 779)
$ grep -c 'every tool, every verb'      -> 0
$ grep -n 'neither is given'            -> 1 hit, the M-5 correction (line 297)
```

---

# Part B — defects introduced by the fold

## B1 — IMPORTANT — §6h bullet 5 / §7 P2 gate: the fold asserts a shipped defect that the same merge fixed

`87d080a` wrote a true sentence; `956eea3`, its sibling in merge `08c803c`, falsified it;
the merge reconciled nothing. On the branch this spec ships on, §6h states:

> **Measured:** `me`'s shipped refusal advises `ms encode --in seed.txt` as the private
> channel for a secret class, and **`ms encode` has no `--in`** … **P2 owes this text, not
> merely the feature**, and until P2 lands, `me` is advising a command that fails.

The tree says otherwise:

```
$ git -C /scratch/code/shibboleth/_work/r0r1/mnemonic-engrave log --oneline -1
08c803c merge: R0 round-0 fold of SPEC_constellation_cli_uniformity

$ grep -n 'ms encode' crates/me-cli/src/main.rs
1983:  // `ms encode --in` DOES NOT EXIST (exit 64) -- caught by the
1987:  "    ms encode --phrase - < seed.txt | me sysw pack --out p.bin"
```

`me` advises `ms encode --phrase - < seed.txt`, which runs. **The spec's only "Measured:"
claim about `me`'s current remedy text is false in the tree that carries it**, and §7 P2's
gate item *"`me`'s shipped remedy text made true"* is already satisfied on that branch.

The risk is not cosmetic: an implementer reading §6h as a statement of intent has a
documented reason to write `ms encode --in seed.txt` back into `me`, re-opening exactly
what `956eea3` closed. The rule in the bullet ("the remedy must not forward-reference a
channel that does not exist") is sound and should stay; its evidence sentence and the P2
gate item must be restated in the forward tense — *when `--in` lands in P2, this line
becomes `ms encode --in seed.txt`* — which is what `956eea3`'s own commit message already
says.

## B2 — IMPORTANT — §6g's two "Measured:" numbers are both false, and one of them states a capability `md` does not have

§6g, the response to C-1:

> **Keyed on KINDS, not counts, because a chunked set is N records and N is unpredictable.**
> Measured: a trivial `pk(@0)` descriptor produces 2 `md1` strings; the reference
> transaction produces 22 `mt1` strings.

Neither number reproduces, and the first is not a miscount but a wrong claim about what
`md` accepts:

```
$ md encode 'pk(@0)' --group-size 0 ; echo "exit=$?"
md: template parse error: unsupported descriptor wrapper: pk(xpub6DXuQW1FgeHbfmexToxaz2g…)#n8k0r596
exit=1
$ md encode 'pk(@0/<0;1>/*)' --group-size 0 ; echo "exit=$?"    # same refusal
exit=1
$ md encode 'wpkh(@0/<0;1>/*)' --group-size 0 | grep -c '^md1'
1
```

`md` refuses `pk(…)` outright — it produces **no** `md1` strings, and the nearest template
that works produces **one**, not two.

The transaction number is off by more than a factor of two. The repo's only reference
transaction is `crates/mt-cli/tests/fixtures/p5_base.json` (`raw_hex`, prefix
`0200000000010293` — the same transaction round 0 used):

```
$ (umask 077; mt encode --in tx.hex > c2.out) 2> c2.err ; echo "exit=$?"
exit=0
$ grep -c '^mt1' c2.out
9
$ grep -E '^CUT' c2.err
CUT       9 strings, 787 characters
```

`mt`'s own report line says nine. The finalized PSBT from the same fixture also yields 9.
Nothing in the repo yields 22.

**Why it gates.** The ruling (kinds, not counts) is settled and is *unaffected* — 1 versus
9 already proves the point. What is affected is P0: §6g instructs the plan to fix and
enumerate a kind vocabulary, and a plan author who takes `pk(@0)` from this bullet as a
worked `md` input writes a fixture that cannot exist. The document's second paragraph
promises *"Every measurement in this document was re-run against the built binaries during
the fold"*; this bullet is labelled `Measured:` and was not.

## B3 — IMPORTANT — §6f's `mnemonic` repair-applied cell is inferred, not measured, and is wrong for a real input class

§6f's table is headed **"Measured, every cell run during the fold:"** and §8 lists only
**two** open `mnemonic` cells. The repair-applied cell reads `5 per D26` — an inference
from a ruling, in a table that declares itself measured. Measured, on identical input:

```
$ GOOD=md1yqpqqxqq8xtwhw4xwn4qh ; BAD=md1yqpqqxqq8xtwhw4xwn4qz
$ md       repair "$BAD" >/dev/null 2>&1 ; echo "md=$?"
md=5
$ mnemonic repair "$BAD" >/dev/null 2>&1 ; echo "mnemonic=$?"
mnemonic=4
```

The divergence is deliberate and `mnemonic` says so on stderr:

```
repair: correction UNVERIFIED — a non-chunked single-string md1 has no cross-chunk/content-id
oracle (the v0.35.0 single-string decode path skips it); … re-derive the wallet/address to
confirm before trusting this correction
```

It is 5 only when a cross-chunk oracle exists (a 2-chunk `mk1` set → 5, verified at four
damage positions), and 4 on a single-string `md1` and on `ms1`.

**Why it gates.** This is precisely the defect I-4 was filed to prevent — *"otherwise two
implementers build two different tables, and one of them silently changes what `mnemonic
repair`'s callers read."* §6f then rules **"The repair codes are FROZEN"** over that table,
so the plan freezes a value that is wrong for one of the three card types, and a P0
conformance assertion of `mnemonic repair → 5` on a single `md1` fails. The fix is the
shape §6f already uses for `ms`: record `mnemonic`'s reasoned 4/5 split with its condition,
and drop the cell out of the "every cell run" claim or measure it.

**I-4 is therefore PARTIAL, not closed.**

## B4 — IMPORTANT — the fold put `mnemonic` inside D3 and gave it no phase; §6d nonetheless says the flag ships "in all five CLIs"

The fold's fourth "found in the fold" item widened the spec to five CLIs. §2a:

> **D3 (argv).** `mnemonic bundle` takes `--passphrase <PASSPHRASE>` on argv.

and §6d closes with:

> **Declined. The name is `--allow-argv-secret` in all five CLIs.**

§7 has no phase that gives `mnemonic` the argv guard, `--in`, `-` or `--out`. P0 is the
crate, P1 is `mt`, P2 is `ms`, P4 is the journey, and P3's only mention is *"Plus
`mnemonic`'s grouping surface and the GUI mirror"* — D4 work, not D3 work. `mnemonic`'s
exit cells are gated (P0), and its grouping is gated (P3); **its argv exposure is owned by
nobody.**

The exposure is not one flag. Measured:

```
$ for c in bundle convert derive-child restore; do
    printf '%-14s ' $c; mnemonic $c --help | grep -oE '\-\-passphrase <PASSPHRASE>' | head -1; echo; done
bundle         --passphrase <PASSPHRASE>
convert        --passphrase <PASSPHRASE>
derive-child   --passphrase <PASSPHRASE>
restore        --passphrase <PASSPHRASE>

$ mnemonic electrum-decrypt --help | grep -m1 '^Usage:'
Usage: mnemonic electrum-decrypt [OPTIONS] --ciphertext <VALUE|-> <--decrypt-password <VALUE>|…>
```

Five argv channels for BIP-39 passphrases and an Electrum password, on the CLI §1's whole
argument is about — and `mnemonic` also ships `convert`, `seed-xor`, `slip39` and
`ms-shares`, the tools that move seed material between formats.

**This is the same defect shape as I-10**, which round 0 filed against the *previous*
revision: an affected surface named in the prose and absent from §7, producing a later gate
that cannot be satisfied. Here §6d's "all five CLIs" is the unsatisfiable claim. Either
`mnemonic`'s D3 work gets an owning phase (its `--passphrase-stdin` already exists on
`bundle`/`convert`/`derive-child`/`restore`, so the private channel is largely there), or
§6d and §2a must carve it out by name and say the constellation ships mixed on D3 — which
§7's own "mixed states are ACCEPTABLE" paragraph would then have to cover permanently
rather than as an intermediate.

## B5 — IMPORTANT — a second false-clean cross-reference, in text the fold wrote, and §8a's claim that this class is gone is false

§8a states a document-wide property:

> **Naming the file is not enough**: the gate matches the section sigil wherever it
> appears … **External references here therefore drop the sigil entirely and read as
> *"section 3b"*.**

§6i, new in this fold, does not:

```
$ grep -n '§8' design/SPEC_constellation_cli_uniformity.md
714:distinct `§8` sections. Exactly **two** of the twelve are posture — the argv
```

That `§8` is `SPEC_mt_v0_1`'s section 8 (the refusal sections — `§8.2f`, `§8.2h`, …). It is
the document's *only* `§8` reference, and the gate resolves it against this spec's own §8:

```
$ ./scripts/spec-structure-check.sh design/SPEC_constellation_cli_uniformity.md | grep '§8'
     §8        What is NOT verified, and must be before the plan closes
  STRUCTURE OK
```

The counts themselves are right — I reproduced both:

```
$ grep -rc 'Refusal::new(' --include='*.rs' crates | awk -F: '{s+=$2} END {print s}'
56
$ # distinct §8.x within 900 chars of each Refusal::new( call site
12   ['§8.1','§8.2b','§8.2c','§8.2d','§8.2e','§8.2f','§8.2h','§8.3','§8.5','§8.6','§8.7b','§8.9']
```

so the finding is the citation, not the arithmetic. The behavioural cost is nil — §6i
opens *"Recorded as a taxonomy. No behaviour follows from it."* **The cost is to the next
reviewer:** §8a tells them this class has been eliminated document-wide, and the machine
gate agrees, so nobody looks again. That is the trap the gate's own comments describe
(*"measured: four to §10.20 meaning §1.1"*). Fix: drop the sigil, as §6f already does two
sections earlier — `12 distinct section-8 refusal sections` — and soften §8a's claim to
what is true after the edit.

## B6 — MINOR — §6e says `mt` prints "all six strings"; it prints nine, and the disagreement with round 0 is unnamed

The spec's preamble sets a rule for itself: *"Where round 0 and the fold disagree, the
fold's number is the one written down **and the disagreement is named at the point it
occurs**."* Round 0's C-3 says nine; §6e says six; nothing is named. Measured, nine:

```
$ script -qec "mt encode --quiet --in tx.hex" /dev/null | grep -c '^mt1'
9
```

The retraction C-3 asked for is unaffected — `mt` does print to a pty at exit 0, which is
the load-bearing fact. Same paragraph, same class: §6e reports the `--quiet` stderr as
*"82 lines"*; I measure 47 with a 0600 destination and 78 with a 0644 one. The substantive
claim there (with `--quiet` there is no `TX`/`CUT`/`PREFIX` report) does reproduce — 0 hits
with `--quiet`, present without it — so only the count is loose.

## B7 — MINOR — two branch-relative statements were true at `87d080a` and are false at the merge tip

The fold branch did not contain `8295c60`; the merge does. Both statements below moved
under it:

```
$ git merge-base --is-ancestor 8295c60 87d080a && echo YES || echo NO
NO
$ git show 87d080a:crates/me-cli/src/main.rs | grep -n 'fn write_private'
844:fn write_private(…)
$ grep -n 'fn write_private' crates/me-cli/src/main.rs
856:fn write_private(…)
```

- §7 cites `write_private` at `crates/me-cli/src/main.rs:844`. It is at **856**.
- §5 says the argv work *"is newer than the branch this spec is being folded on, so the
  shipped `me` binary is ahead of the fold branch's source — a plan reading only this
  branch would conclude, wrongly, that the gate is still bearer-only."* On this branch the
  code is present (`sysw/record.rs:105`, `is_argv_forbidden`), so the warning now points a
  reader at a hazard that no longer exists.

Neither misleads an implementer into wrong behaviour, but §7's citation is the kind that
decays into a fold two cycles from now.

## B8 — NIT — "28 tracked files carry it across all of `design/`" is 29, and the fold's own commit is the 29th

```
$ git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l
29
```

The list includes `design/agent-reports/R0-cli-uniformity-fold-round0.md`, which did not
exist when the number was measured. The load-bearing figure — **7** tracked files under
`design/journeys/`, which is what P3 owns — is correct and unaffected. (Minor wording
alongside it: P3's gate calls them *"the 7 goldens"* while §7's own prose says 5
transcripts and **2 drivers**, and those 2 drivers are also in P2's list of 7 scripts, for
a different edit.)

## B9 — NIT — `design/SUPERSEDED_TERMS.txt` gained no entries, so the retraction-leakage check is a no-op for this spec

The fold made six retractions (`the one thing that did not need fixing`, `--json` is
`already uniform`, `mt`'s `suite unchanged`, `exit 3`, `neither is given`, the terminal-gate
lift). `SUPERSEDED_TERMS.txt` opens *"Terms that must NOT appear as LIVE text in
SPEC_mt_v0_1.md"* and lists only `mt`'s six. My manual sweep (Part A, above) found the
propagation clean — but it was manual, and §8a's *"Both gates are now CLEAN on this file"*
reads as more coverage than exists. Adding the six lines makes the next fold's sweep a
command rather than a discipline, which is the standing pattern for exactly this.

---

# Counts

**0 Critical / 5 Important / 2 Minor / 2 Nit**

Round-0 dispositions: **21 CLOSED, 1 PARTIAL (I-4), 1 REJECTED-and-sound (M-4),
0 NOT CLOSED, 0 WRONGLY CLOSED.**

# Verdict

**NOT GREEN — do not proceed to implementation.**

The fold is substantially good work: all four Criticals are genuinely closed, the two
retractions (C-3's terminal gate, I-8's "uniform") are complete, the §4 provenance
re-citation is verbatim and correctly attributed, the propagation sweep is clean, and every
count I could re-run in `mnemonic-gui`, `mnemonic-secret`, `mnemonic-transaction` and this
repo reproduced exactly. The five blocking findings are all in text the fold itself wrote.

**The single most important finding is B4.** The fold widened the spec to five CLIs and
ruled that `--allow-argv-secret` lands *"in all five"*, but §7 gives `mnemonic` only its
grouping surface — so the constellation's largest argv exposure for secret material (five
channels across `bundle`, `convert`, `derive-child`, `restore` and `electrum-decrypt`) has
no owning phase. That is the identical shape round 0 caught as I-10, re-introduced for the
tool the fold had just brought into scope: a surface named in the prose, absent from the
phasing, and a §6 ruling that no phase can satisfy.
