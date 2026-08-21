# R3 — Independent adversarial review: `card-index.txt` attribution

**Question asked:** Can the rewritten `card-index.txt` derivation attribute an mk1 string
to the WRONG cosigner key or the WRONG origin — and would anything catch it if it did?

**Reviewer:** independent (did not author the change).
**Range:** `6e6753c..master` (4 commits), `design/journeys/transcript_pathological.sh`.
**Method:** ran the transcript; decoded all 30 cards; re-ran the *replaced* per-key loop
on the journey's own keys; perturbation-tested the awk join and the PDF builder's caption
lookup; machine-counted `mk` invocations with a wrapper shim; read `mk`'s `--keys`
implementation and its pinning tests.

---

## Headline answer, in two parts

**Part 1 — is the attribution correct today? YES.** Verified three independent ways, none
of which relies on any ordering assumption:

1. **Decode every card, compare to the source key file.** 30/30 rows in
   `card-index.txt` carry the fingerprint, path AND xpub that the card itself decodes to.
2. **Re-ran the per-key loop this change deleted**, on this journey's real 11 keys.
   Byte-identical to the batch output.
3. **Section 5's single-key card == batch block 1**, byte-identical.

**Part 2 — would anything catch it if it were wrong? NO.** The join is positional and the
only guard is a **count**, which is permutation-blind. A two-record permutation produces a
fully self-consistent 30-line `card-index.txt`, awk exit 0, and **5 silently wrong plate
captions** that `build_pdf_pathological.py`'s own guard does not fire on. That is exactly
the failure class documented in this file's own comment at line 241-246.

No Critical. One Important (the missing guard), three Minor, three Nit.

---

## IMPORTANT — the attribution has NO content-based check; the count guard cannot see a permutation

`design/journeys/transcript_pathological.sh:257-276` (the `awk -v RS=` join), guard at `:275`

**Failure scenario.** The change replaced a *causal* binding — each mk1 string paired, in
the same loop iteration, with the key whose own `mk encode` invocation produced it — with a
*positional* one: card block N ↔ `keys-meta.txt` line N. The only thing standing between a
correct index and 30 wrong engraved-plate captions is `END { if (NR != n) ... }`, which
compares **counts**. Any permutation preserves the count.

Worse, `tot` is recomputed from the block (`:272`), not read from the meta record, so a
permutation *self-heals* into a plausible-looking chunk number: key @1 (really 3 chunks)
gets captioned "chunk 1/2" and nothing anywhere notices the contradiction.

Downstream does not close it either. `build_pdf_pathological.py:446` raises only when a
plate's mk1 string is **absent** from the index. Under a permutation every string is
present, so the guard never fires and the committed PDF carries wrong captions at exit 0.

The comment at `:254-256` states the coupling as a fact ("`--keys` emits cards in FILE
order ... so block N belongs to meta line N"). It is a true fact — but it is a **cross-repo
contract enforced only in another repo's test suite**. `mnemonic-key` pins it with
`batch_matches_per_key_loop` and `blank_line_separates_cards_only_between`
(`crates/mk-cli/tests/keys_batch.rs:82,140`). `mnemonic-engrave`'s CI (`release.yml` is the
only workflow) runs neither those tests nor this journey, and `$MK` is an unversioned local
build at `$C/mnemonic-key/target/release/mk`. So the mechanism by which a silent regression
arrives is: someone changes `--keys` emission order upstream, `mnemonic-key`'s tests are
updated with it, and this journey silently re-captions every plate.

Note the obvious remedy does **not** work: `mk encode --keys --json` emits
`{mk1_strings, chunk_count, code_variant}` per card
(`mnemonic-key/crates/mk-cli/src/cmd/encode.rs:247-257`) — **no origin**. Switching to JSON
would trade blank-line parsing for array-index parsing and stay just as positional. The only
ordering-independent check is decode-and-compare, and `mk decode` is already invoked two
lines below at `:280`.

**Evidence.**

Correctness today (all three checks):

```
$ python3 verify_attr.py    # decodes each key's chunks, compares to key-NN.xpub
card-index rows: 30
@ 0 claim=[73c5da0a/48'/0'/0'/2'] decoded=[73c5da0a/48'/0'/0'/2'] keyfile=[73c5da0a/48'/0'/0'/2'] xpub_match=True chunks=2 (long) -> OK
...
@10 claim=[28645006/48'/0'/2'/2'] decoded=[28645006/48'/0'/2'/2'] keyfile=[28645006/48'/0'/2'/2'] xpub_match=True chunks=3 (long) -> OK
MISMATCHES: 0

$ diff loop-cards.txt out/pathological/mk-encode-raw.txt   # replayed per-key loop
BYTE-IDENTICAL: batch == per-key loop on this journey's real keys

$ diff sec5.txt <(sed -n '1,2p' out/pathological/mk-encode-raw.txt)
section-5 card == batch block 1
```

The guard is permutation-blind (swap meta lines 1 and 2, feed the real card blocks):

```
$ head -3 meta-perm.txt
1 73c5da0a 48'/0'/1'/2'
0 73c5da0a 48'/0'/0'/2'
2 73c5da0a 48'/0'/2'/2'

$ awk -v RS= ... cards.txt ; echo "AWK EXIT=$?"
AWK EXIT=0

$ awk '{print substr($1,1,24)"... "$2" "$3"/"$4" "$5" "$6}' out-perm.txt | head -6
mk1qpd8cwpqqsq4kj90x4eut... 1 1/2 73c5da0a 48'/0'/1'/2'     <-- really @0
mk1qpd8cwpp806lhaeh6rekn... 1 2/2 73c5da0a 48'/0'/1'/2'     <-- really @0
mk1qp4dj9zqqsq4kj90x4eut... 0 1/3 73c5da0a 48'/0'/0'/2'     <-- really @1
mk1qp4dj9zp68w6hzragnj3g... 0 2/3 73c5da0a 48'/0'/0'/2'     <-- really @1
mk1qp4dj9zzv308jhm5uzl5t... 0 3/3 73c5da0a 48'/0'/0'/2'     <-- really @1
mk1qp8lruzqqsq4kj90x4eut... 2 1/3 73c5da0a 48'/0'/2'/2'
$ wc -l < out-perm.txt
30
```

Note "@1 chunk 1/2" — key @1 has three chunks. The falsehood is internally consistent.

The PDF builder's own guard does not fire (exact copy of its `_CARD_OWNER` / `_caption`
lookup, fed the permuted index and the real `manifest.json`):

```
WRONG CAPTION  got: plate 14 — @1 [73c5da0a/48'/0'/1'/2'] chunk 1/2   truth: plate 14 — @0 [73c5da0a/48'/0'/0'/2'] chunk 1/2
WRONG CAPTION  got: plate 15 — @1 [73c5da0a/48'/0'/1'/2'] chunk 2/2   truth: plate 15 — @0 [73c5da0a/48'/0'/0'/2'] chunk 2/2
WRONG CAPTION  got: plate 21 — @0 [73c5da0a/48'/0'/0'/2'] chunk 1/3   truth: plate 21 — @1 [73c5da0a/48'/0'/1'/2'] chunk 1/3
WRONG CAPTION  got: plate 22 — @0 [73c5da0a/48'/0'/0'/2'] chunk 2/3   truth: plate 22 — @1 [73c5da0a/48'/0'/1'/2'] chunk 2/3
WRONG CAPTION  got: plate 23 — @0 [73c5da0a/48'/0'/0'/2'] chunk 3/3   truth: plate 23 — @1 [73c5da0a/48'/0'/1'/2'] chunk 3/3

plates raising SystemExit: 0
plates silently mis-captioned: 5
```

A full reversal mis-captions all 30. No CI, no journey step, and no builder guard sees it.

**Verdict: CONFIRMED** (as a missing guard for a demonstrated silent-wrong-answer class;
the artifact produced *today* is correct).

---

## What I attacked and could NOT break

Recorded so a later reviewer does not re-derive it.

- **Blank-line contract — SOUND.** `encode.rs:188-201` prints a blank line only when
  `i > 0`: never leading, never trailing. Pinned by
  `blank_line_separates_cards_only_between`. An mk1 string can never *contain* a blank
  line: `render_grouped` (`format.rs:16-28`) only ever inserts `separator`, and
  `parse_separator` (`format.rs:40-49`) restricts that to `space|hyphen|comma` — a newline
  is unrepresentable. So `--group-size` being nonzero could not break the delimiter even if
  it were not pinned at `0` on the command line (`:212`), and `^mk1` still matches a grouped
  string.
- **stderr never merges into stdout.** `:212-213` redirects them to separate files; the
  watch-only advisory goes to stderr (`encode.rs:203-206`). Confirmed: `mk-encode-stdout.txt`
  is 40 lines = 30 mk1 + 10 separators, no advisory line.
- **Glob order — SOUND.** All eleven files are zero-padded two digits (`key-00`..`key-10`),
  so lexicographic == numeric, in any locale (pure ASCII). `read_key_records`
  (`keyfile.rs:99-108`) pushes in line order; `minted` and the emit loop both preserve it.
  And because `$KEYFILE` and `$KEYMETA` are written in the *same* loop iteration
  (`:193,:198`), an unpadded filename would change display order but **not** attribution.
- **`m[NR-1]` indexing — CORRECT.** Paragraph-mode NR counts blocks; block 1 → `m[0]`.
- **`getline < meta` under `RS=""` — handled.** The BEGIN loop re-splits on `\n`
  (`:263-266`), giving n=11. (This was the bug already found and fixed in development.)
- **Trailing blank lines / missing final newline — HARMLESS.** Both produce 30 rows, exit 0.
- **Merged blocks (a delimiter lost) — CAUGHT.** `FATAL: 10 card blocks for 11 key records`,
  exit 1, propagated through `|| exit 1` to the script.
- **Extra blocks — CAUGHT.** `FATAL: more card blocks than key records`, exit 1.
- **Duplicate mk1 strings** (which would silently overwrite in the builder's
  `_CARD_OWNER` dict): none. `cut -d' ' -f1 card-index.txt | sort | uniq -d` → 0.
- **Downstream ordering assumption — GONE.** `build_pdf_pathological.py:423-449` keys on the
  plate's own string via `manifest.json`; it never walks key order. The historical
  `me bundle` chunk_set_id-vs-key-order bug is genuinely fixed.
- **Whole transcript** runs to exit 0 with zero FATAL / CAPTURE FAILED lines (re-confirmed
  twice, including after restoring `out/`).

---

## MINOR — the awk records EVERY line of a block, not just `^mk1`, and nothing compares the two files

`design/journeys/transcript_pathological.sh:272-273` (no `^mk1` filter), `:225`, `:278-279`

**Failure scenario.** `mk-encode-raw.txt` — the file that is actually engraved — is
`grep '^mk1'`-filtered (`:225`). `card-index.txt` is not: the awk emits `L[j]` for every
line in the block. If `mk` ever printed a non-mk1 line to **stdout** inside a card block, it
would (a) become a bogus index row and (b) **shift `nth`/`tot` for that key's real chunks**,
silently. The two line counts are printed adjacently by `run wc -l` at `:278-279` — where a
reader will assume they are being compared — but nothing compares them.

**Evidence.** Inserting one `note: stdout is watch-only` line inside block 2:

```
### D: a non-mk1 note line inserted INSIDE block 2
  exit=0 lines=31
4:note: stdout is watch-only 1 2 4 73c5da0a 48'/0'/1'/2'
```

31 rows vs 30 engraved strings, exit 0, and @1's real chunks are now numbered 3/4 and 4/4.
Today the counts agree (`diff <(cut -d' ' -f1 card-index.txt) mk-encode-raw.txt` → identical),
so this is latent, not live — it needs an upstream stdout change to trigger.

**Verdict: CONFIRMED** (latent).

---

## MINOR — `:211` prints a command that was never run and cannot be run, in the format reserved for real ones

`design/journeys/transcript_pathological.sh:211`

**Failure scenario.** Every other command in this transcript is echoed by `run()`/`runcap()`
with its **real argv**. Line 211 hand-writes a `$ ...` line, and `:220` follows it with a
hand-written `[exit 0]`, so it is typographically indistinguishable from a captured command
in the operator-facing PDF. It is not executable, and its argument count is misleading:
`${#FROM_MD1[@]}` is the **array length (8)**, but the text reads "8 `--from-md1` args" when
there are **4** `--from-md1` flags (4 md1 chunks × 2 array elements each).

**Evidence.**

```
$ grep -n 'mk encode --keys' run1.txt
88:$ /scratch/code/shibboleth/.../mk encode --keys /scratch/.../keys.txt 8 --from-md1 args --group-size 0
$ grep -c '^md1' out/pathological/md1.txt
4
```

**Verdict: CONFIRMED.**

---

## MINOR — both committed invocation counts are wrong (machine-counted)

`design/journeys/transcript_pathological.sh:248-252`; commit `01697a1` subject

The comment says the transcript "now runs it **twice**" and previously ran `mk encode`
"**33** times". Measured with a counting shim wrapping `mk`:

| | claimed | measured |
|---|---|---|
| new (`master`) | 2 | **3** |
| old (`6e6753c`) | 33 | **34** |

The third call is `:154` (`MK_STUB`, the section-6 cross-check) — added by `85cf6c7` in this
same range, so the claim was already false when written. The old 34 = 1 demo + 11 cards + 22
index; the "22 more" figure at `:248-249` is correct.

**Evidence.**

```
$ export MKLOG=...; bash t.sh >/dev/null 2>&1
=== total mk invocations ===   6
=== mk encode invocations ===  3
=== mk decode invocations ===  2
encode --xpub xpub6DkFAXWQ2dHxq...      <- :131
encode --xpub xpub6DkFAXWQ2dHxq...      <- :154
encode --keys .../keys.txt              <- :212

$ git show 6e6753c:...transcript_pathological.sh > t-old.sh; bash t-old.sh >/dev/null 2>&1
OLD total mk: 36  encode: 34  decode: 1
```

**Verdict: CONFIRMED.**

---

## NIT — `sed 's/key-0*//'` is unanchored and collides

`design/journeys/transcript_pathological.sh:197` (and the identical copy at `:312`)

```
key-00  -> '0'     key-01 -> '1'     key-09  -> '9'
key-10  -> '10'    key-100 -> '100'
key-010 -> '10'    <-- collides with key-10
```

`[ -z "$ki" ] && ki=0` masks exactly one case (`key-00` → empty → 0), which is correct and
intended. A non-numeric residue would reach `int(_ki)` in the builder and raise loudly, so
the failure is not silent. Not reachable with the current eleven zero-padded files.
**Verdict: CONFIRMED (not reachable).**

---

## NIT — `awk -v` performs escape-sequence processing on its values

`design/journeys/transcript_pathological.sh:257` (`-v meta=`, `-v out=`)

```
$ awk -v p='/tmp/a\tb' 'BEGIN{ printf "%s|len=%d\n", p, length(p) }'
/tmp/a	b|len=8
```

A `$W` containing a backslash would silently write `card-index.txt` somewhere else, or read
a different meta file. `$W` is the script's own directory, so not reachable here.
**Verdict: CONFIRMED (not reachable).**

---

## NIT — the over-count guard prints two FATAL lines

`design/journeys/transcript_pathological.sh:269` + `:275`

`exit 1` inside the main rule jumps to `END`, whose condition is then also true:

```
FATAL: more card blocks than key records
FATAL: 12 card blocks for 11 key records
  exit=1
```

Cosmetic; the exit status is correct. **Verdict: CONFIRMED.**

---

## Notes on remedy (non-authoritative — reproduce the defect, not the fix)

The Important finding is closed by *any* content-derived check, not by a particular one.
The cheapest ordering-independent shape is the one this review used: for each block, decode
it and require the decoded `origin_fingerprint`/`origin_path` to equal the meta record it
was joined to. `mk decode` is already on the path and already invoked at `:280`; the added
cost is 11 more `mk decode` calls, which is still an order of magnitude below the 34 the
change removed. Explicitly **not** sufficient: switching to `--keys --json`, whose card
objects carry no origin.

Separately worth considering under the same finding: nothing pins the `mk` version this
journey was validated against. `:76` prints `mk --version` into the transcript and asserts
nothing, and the two contracts the join rests on live in a repo this one's CI never builds.

---

## VERDICT: 0 Critical, 1 Important, 3 Minor, 3 Nit
