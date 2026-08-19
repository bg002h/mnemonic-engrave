# Exec review — c6c6943 "the journeys now BUILD what they engrave" (item 9 / F-210 / I-1)

Independent adversarial execution review. Read-only on tracked files; every claim below
carries a `file:line` or pasted command output. `design/journeys/out/**` was deleted and
regenerated repeatedly (untracked build output); the payload journey's untracked artifacts
were restored afterwards and `git status --short` is empty.

Tool versions used for every measurement: `md 0.13.0`, `mk 0.13.0`, `ms 0.16.0`,
`me 0.6.0`, `me-preview 0.6.0`.

---

## Verdict

**Yes on both halves of the question, with three Important defects that do not touch key
material.** Each journey now engraves the exact bytes it printed: `transcript.sh:110` and
`transcript_pathological.sh:147` `cat` the run's own md1 capture and the run's own
`mk-encode-raw.txt` into `out/backup-strings.txt`, and that same path is what
`me bundle --in` consumes (`transcript.sh:122`, `transcript_pathological.sh:152`). There is
no longer a tracked fixture to drift. **Nothing key-material changed.** I decoded all 23
key cards from the old deleted fixtures (recovered via `git show c6c6943^:…`) and all 23
from freshly generated bundles: xpub, `origin_fingerprint`, `origin_path` and
`policy_id_stubs` are identical on every card, 0 differences, and both journeys' md1
strings are *byte*-identical old-vs-new. Steel already cut from the old fixture stays
valid. The derived `--policy-id-stub` equals the old hardcoded `5b48af35` exactly. Both
journeys are byte-deterministic across runs, byte-identical to their committed transcripts,
and no longer disturb each other in either order.

What is wrong sits one layer out, in the documents: this commit renamed a transcript
heading and left `build_pdf_pathological.py:207` looking up the old name, so the
pathological document's key-card block now renders as `<pre></pre>` — the block that was
supposed to *show* the eleven cards this commit added. Separately, neither journey checks
that a per-key `mk encode` succeeded, so a one-character typo in a key file's origin header
silently drops that cosigner's card from the engraved bundle at exit 0; and both builders
now mix a *committed* transcript with a *live* bundle, so an input edit produces a document
carrying two different card strings for the same key. All three are demonstrated below with
real output.

---

## Key-material equivalence

Method: old fixtures recovered with `git show c6c6943^:design/journeys/inputs/backup-strings.txt`
and `…/inputs-pathological/backup-strings.txt`; new bundles produced by
`bash design/journeys/transcript.sh` / `transcript_pathological.sh` from a removed `out/`.
Each mk1 pair decoded with `mk decode <chunk1> <chunk2>` and all four fields compared.

### Operator (`inputs/keys/cosigner-*.xpub`, 12 cards)

`old mk1 lines: 24   new mk1 lines: 24`

| card | old fp | new fp | old path | new path | old stub | new stub | xpub | identical? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | ae6647ee | ae6647ee | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 1 | ff4bdd8b | ff4bdd8b | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 2 | b180e226 | b180e226 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 3 | 7ab0a774 | 7ab0a774 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 4 | bf3e6e44 | bf3e6e44 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 5 | c1826fbb | c1826fbb | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 6 | ecbd6d3c | ecbd6d3c | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 7 | 7aa32b8d | 7aa32b8d | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 8 | 68c883f9 | 68c883f9 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 9 | 5a3ae6a9 | 5a3ae6a9 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 10 | ee9b5972 | ee9b5972 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |
| 11 | dd7d1881 | dd7d1881 | 48'/0'/0'/2' | 48'/0'/0'/2' | 726a6663 | 726a6663 | SAME | IDENTICAL |

`cards compared: 12   differing: 0`

md1: `diff <(grep '^md1' old) <(grep '^md1' new)` → **empty; md1 BYTE-IDENTICAL**
(`md1ytpqqxpp3zcpydzk0zdt492xzr7r9qxfc`), so policy, `wallet-policy-id`
(`f05e8a1c282f7740bbfd902a759b5577`) and `wallet-descriptor-template-id`
(`726a666305756435b7c52c5b3fc69c41`) are trivially unchanged.

### Pathological (`inputs-pathological/keys/key-*.xpub`, 11 cards)

`old mk1 lines: 22   new mk1 lines: 22`

| card | old fp | new fp | old path | new path | old stub | new stub | xpub | identical? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 73c5da0a | 73c5da0a | 84'/0'/0' | 84'/0'/0' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 1 | 73c5da0a | 73c5da0a | 84'/0'/1' | 84'/0'/1' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 2 | 73c5da0a | 73c5da0a | 84'/0'/2' | 84'/0'/2' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 3 | 73c5da0a | 73c5da0a | 84'/0'/3' | 84'/0'/3' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 4 | b8688df1 | b8688df1 | 84'/0'/0' | 84'/0'/0' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 5 | b8688df1 | b8688df1 | 84'/0'/1' | 84'/0'/1' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 6 | b8688df1 | b8688df1 | 84'/0'/2' | 84'/0'/2' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 7 | b8688df1 | b8688df1 | 84'/0'/3' | 84'/0'/3' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 8 | 28645006 | 28645006 | 84'/0'/0' | 84'/0'/0' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 9 | 28645006 | 28645006 | 84'/0'/1' | 84'/0'/1' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |
| 10 | 28645006 | 28645006 | 84'/0'/2' | 84'/0'/2' | 5b48af35 | 5b48af35 | SAME | IDENTICAL |

`cards compared: 11   differing: 0`

md1 chunk set (3 chunks): **BYTE-IDENTICAL** old-vs-new.
`md inspect` of the new chunk set:

```
wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc
wallet-policy-id: d3dda0f3a9ef2eef1f1de404b8a352a5
wallet-policy-id-fingerprint: 0xd3dda0f3
derived STUB = [5b48af35]   old hardcode = [5b48af35]
STUB MATCHES OLD CONSTANT
```

### Ordering and origin extraction (angles 2 and 4)

Both key directories contain **only** `.xpub` files (12 and 11, `git ls-files` confirms no
other tracked file under `inputs/keys` or `inputs-pathological/keys`), and every name is
zero-padded to two digits, so bash's `cosigner-*.xpub` / `key-*.xpub` glob order,
`sorted(os.listdir())` in `build_pdf.py:95`, and numeric order all coincide — including at
`cosigner-10`/`-11` and `key-10`, the 10+ range where lexical and numeric order can diverge.
There is no `cosigner-9` to sort after `cosigner-10`.

Every key file's `origin [fp/path]` header parses under both scripts' `sed` (12/12 and
11/11, no FATAL), and each generated card decodes to the origin its own file declares:

```
0  cosigner-00.xpub   OK  file_fp=ae6647ee card_fp=ae6647ee file_path=48h/0h/0h/2h card_path=48'/0'/0'/2'
…
11 cosigner-11.xpub   OK  file_fp=dd7d1881 card_fp=dd7d1881 file_path=48h/0h/0h/2h card_path=48'/0'/0'/2'
keys: 12  mismatches: 0

0  key-00.xpub        OK  file_fp=73c5da0a card_fp=73c5da0a file_path=84'/0'/0' card_path=84'/0'/0'
…
10 key-10.xpub        OK  file_fp=28645006 card_fp=28645006 file_path=84'/0'/0'… card_path=84'/0'/2'
keys: 11  mismatches: 0
```

The operator headers use `h` hardening (`48h/0h/0h/2h`) and the pathological ones use
apostrophes (`84'/0'/0'`); `mk` normalises both to `'`, and the resulting `origin_path` is
identical to what the old fixture encoded (see the tables above). The operator doc's own
key table was cross-checked against the bundle: 12 rows, caption fingerprint == decoded
card fingerprint, chunk prefixes match, `mismatches: 0`.

### Stub failure path (angle 3)

`transcript_pathological.sh:115-120` derives `STUB` and aborts on empty. The guard is real
(`if [ -z "$STUB" ]; then … exit 1; fi`) and is reached before any `mk encode`, so an empty
derivation cannot produce cards with an empty stub. Note the script has `set -u` but not
`set -e`, so this explicit `if` is doing the whole job — it is present and correct. The
weaker sibling case (derivation returns *garbage* rather than empty) falls into I-2 below.

---

## Findings

### Critical

**None.** No finding affects key material, the engraved bytes, or the validity of steel cut
from the old fixture.

### Important

**I-1 — this commit renamed transcript heading 7 and left the builder looking up the old
name, so the pathological document's key-card block now renders empty.**
`design/journeys/build_pdf_pathological.py:207`

`transcript_pathological.sh:122` was changed by this commit from
`########## 7. the eleven key cards` to
`########## 7. the eleven key cards — ALL of them, each with its own origin`. Verified
against the parent commit:

```
=== heading at parent commit ===
70:########## 7. the eleven key cards
=== heading now ===
70:########## 7. the eleven key cards — ALL of them, each with its own origin
```

`build_pdf_pathological.py:207` still reads `{code(S.get('7. the eleven key cards',''), 30)}`
— untouched by this commit (`git blame` → `bdf954f6 (bg 2026-08-11)`). `S.get` returns `''`,
so the failure is silent. A machine cross-check of every builder lookup against every
transcript heading:

```
=== build_pdf_pathological.py  (reads transcript_pathological.txt) ===
  builder lookups: 10
    MISS [7. the eleven key cards]
  MISSING LOOKUPS: 1
=== build_pdf.py  (reads transcript.txt) ===
  MISSING LOOKUPS: 0
```

Rendered result in `out/pathological/journey_pathological.html`:

```
'<h2>Host step 4 — the eleven key cards</h2>\n<pre></pre>\n<p>Each key splits into <b>2 chunks</b>, so the eleven cards are 22 strings.\nNote the decode: the card carries the origin fingerprint and path, so the\norigins the descriptor card lacks are present in the bundle.'
```

Failure scenario: the document page whose entire purpose is to evidence this commit's
headline claim — "step 7 said 'the eleven key cards' and encoded ONE. It now encodes all
eleven" — shows nothing, and the prose immediately below (*"Note the decode: the card
carries the origin fingerprint and path…"*) now points at an empty box. This is the
document-layer instance of exactly the class the commit is closing, introduced *by* the
commit. `build_pdf.py` is clean on the same check.

**I-2 — no journey checks that a per-key `mk encode` succeeded, so a one-character typo in
a key header silently drops that cosigner's card from the ENGRAVED bundle at exit 0.**
`design/journeys/transcript.sh:97-99`, `design/journeys/transcript_pathological.sh:133-135`

Both loops run `"$MK" encode … 2>/dev/null | grep '^mk1' >> "$…/mk-encode-raw.txt"`. stderr
is discarded, the pipeline's exit status is discarded, and a failed encode simply appends
nothing. The pre-loop FATAL guard only checks that the xpub/fingerprint/path strings are
*non-empty*, not that `mk` accepts them.

Demonstrated on a copy of the tree, changing `c1826fbb` to `c1826fb` (one character) in
`inputs/keys/cosigner-05.xpub`:

```
# cosigner 5 — origin [c1826fb/48h/0h/0h/2h]
JOURNEY EXIT = 0
--- stderr ---            (empty)
--- bundle line count ---
23 …/out/backup-strings.txt
--- what the transcript reported ---
$ wc -l …/out/mk-encode-raw.txt
22 …/out/mk-encode-raw.txt
[exit 0]
--- me bundle ---
[exit 0]
--- plates rendered ---
23
cards on the engraved bundle: 11 (expected 12); c1826fbb present? 0
```

`me encode` had exited 64 with `error: --origin-fingerprint: invalid hex "ae6647e": Odd
number of digits` — none of which survives `2>/dev/null`. The operator engraves a 5-of-12
backup that is missing cosigner 5's key card, and `me bundle` blesses it with exit 0.

The pathological side is worse, because `build_pdf_pathological.py:71-82` derives its
captions from the *key files* and indexes plates by position with no cross-check against
the bundle. Same experiment on `inputs-pathological/keys/key-05.xpub` (`84'/0'/1'` →
`84'/0'/1x'`):

```
JOURNEY EXIT = 0
23 …/out/pathological/backup-strings.txt      (should be 25)
23 plates
build_pdf_pathological exit=0                 (no error at all)

plate 14: caption @5 [b8688df1/84'/0'/1x']   card [b8688df1/84'/0'/2']  MIS-CAPTIONED
plate 16: caption @6 [b8688df1/84'/0'/2']    card [b8688df1/84'/0'/3']  MIS-CAPTIONED
plate 18: caption @7 [b8688df1/84'/0'/3']    card [28645006/84'/0'/0']  MIS-CAPTIONED
plate 20: caption @8 [28645006/84'/0'/0']    card [28645006/84'/0'/1']  MIS-CAPTIONED
plate 22: caption @9 [28645006/84'/0'/1']    card [28645006/84'/0'/2']  MIS-CAPTIONED
MIS-CAPTIONED PLATES: 10
```

Ten of twenty key plates are captioned with the wrong master and the wrong derivation
path, at exit 0. (`build_pdf.py`'s `_keys_from_run` *does* catch the short-bundle case —
`cosigner-11.xpub: expected 2 mk1 chunks at offset 22, got 0`, `build_pdf.py:100-101` — but
that backstop only fires if someone rebuilds the document, and the engrave path never
touches it.) A `wc -l` assertion against `2 × (number of key files)` inside each loop would
close both halves.

**I-3 — both builders now mix a COMMITTED transcript with a LIVE bundle, and nothing checks
they came from the same run.** `design/journeys/build_pdf.py:66` + `:90`,
`design/journeys/build_pdf_pathological.py:54` + `:13`

This commit repointed the narrative source to the tracked `transcript*.txt`
(`build_pdf.py:66`, `build_pdf_pathological.py:54`) while pointing the card data at
`out/backup-strings.txt`, which is rebuilt by every run (`build_pdf.py:90`). Today those
two agree byte-for-byte, so nothing is visibly wrong — but the agreement is a coincidence
of the current inputs, not a structural property, which is precisely the distinction the
commit message draws.

Demonstrated by re-keying `inputs/keys/cosigner-00.xpub` to a different valid xpub (a
routine input edit) on a copy:

```
JOURNEY EXIT=0
diff (live run vs committed transcript.txt): 20 lines
build_pdf exit=0

committed transcript card-0 chunk1: mk1qpj6vvpqqsqhy6nxvwhxv3lwq5zg3vs76cp0whqh5
live bundle       card-0 chunk1: mk1qpr8xqpqqsqhy6nxvwhxv3lwq5zg3vs7wf5s0sfdg
SAME? False

journey.html contains the COMMITTED (stale) chunk : True
journey.html contains the LIVE (this-run) chunk   : True
```

Failure scenario: one document asserts two different key cards for cosigner 0 — the stale
one in its CLI blocks, the fresh one in its key table and plate captions — with no error
anywhere. That is the F-210 print/engrave split relocated from the bundle into the
deliverable. A guard comparing the committed transcript's `head -3` block against the live
bundle, or simply reading both from the same place, would make it structural.

### Minor

**M-1 — `mkdir -p "$W/out/pathological/pathological"` creates a stray empty directory.**
`design/journeys/transcript_pathological.sh:65`. The doubled path component is a typo;
`-p` masks it by creating the intended parent, so nothing breaks, but every run leaves an
empty `out/pathological/pathological/` behind. Confirmed present after each run, and also
present in the author's own pre-existing `out/` snapshot — i.e. it has never been noticed.

**M-2 — `README.md`'s reproduction block is now wrong and was not updated by this commit.**
`design/journeys/README.md:111-112`:

```sh
bash transcript_pathological.sh > out/transcript.txt 2>&1
python3 build_pdf_pathological.py          # writes out/journey.html
```

After this commit the builder reads the tracked `transcript_pathological.txt`, not
`out/transcript.txt`, and writes `out/pathological/journey_pathological.html`. Following
the README verbatim redirects the fresh transcript into a file nothing reads and then
builds the document from the *old* narrative — the exact defect the commit's second bullet
says it fixed, still live in the README. `README.md` does not appear in this commit's diff.

**M-3 — both documents' own "Reproducing this document" blocks are now wrong.**
`design/journeys/build_pdf.py:416` and `:420`, `design/journeys/build_pdf_pathological.py:351`.
`build_pdf.py` still prints `bash transcript.sh > out/transcript.txt 2>&1` and
`go run ./cmd/journeykeys > keys.json` — neither file is read any more. The pathological
document prints `bash transcript.sh > out/transcript.txt 2>&1`, naming the **operator**
script, which is the same wrong-journey mistake this commit fixed one line up in the code.
These strings are baked into the published documents.

### Nit

**N-1 — `import json` at `design/journeys/build_pdf.py:12` is now unused** (the only
remaining `json` token is inside a literal at `:420`).

**N-2 — the clip footer still points readers at the removed intermediate.**
`design/journeys/build_pdf.py:51` and `design/journeys/build_pdf_pathological.py:41` render
"full text in `out/transcript.txt`".

**N-3 — the new `2b` / `7b` transcript sections are not surfaced in either document.** Both
journeys now print a section explaining the F-210 fix (`transcript.sh:104`,
`transcript_pathological.sh:140`) and neither builder looks it up, so the fix's own
narrative never reaches the deliverable. Not a defect; noted because the sections read as
if they were written to be published.

---

## Determinism and independence

All runs from a fully removed `design/journeys/out/`, invoked with `bash`.

| check | result |
| --- | --- |
| operator run 1 vs run 2 — stdout | **BYTE-IDENTICAL** (`diff` empty) |
| operator run 1 vs run 2 — whole `out/` tree incl. 25 PNGs, manifest.json, sysw-public.bin | **BYTE-IDENTICAL** (`diff -r` empty) |
| pathological run 1 vs run 2 — stdout | **BYTE-IDENTICAL** |
| pathological run 1 vs run 2 — whole `out/` tree | **BYTE-IDENTICAL** |
| operator run vs committed `transcript.txt` | **0 diff lines** |
| pathological run vs committed `transcript_pathological.txt` | **0 diff lines** |
| operator re-run into a DIRTY `out/` (no clean between) | bundle 25 → 25 lines, stdout identical — no accumulation (`: >` truncation at `transcript.sh:88` and the `plates` `rm -rf` at `:121` both hold) |
| pathological re-run into a DIRTY `out/` | bundle 25 → 25 lines, stdout identical |
| operator then pathological: operator's `backup-strings.txt`, `md-encode-raw.txt`, `mk-encode-raw.txt`, `ms-encode.txt`, `manifest.json`, `sysw-public.bin`, `plates/` | **all UNCHANGED**; both bundles present at 25 lines each |
| pathological then operator: whole `out/pathological/` subtree | **UNCHANGED** |
| non-zero exits, operator | **1** (`[exit 3]`, the ms1 refusal at `transcript.sh:130`) — matches the author's count |
| non-zero exits, pathological | **3** (`[exit 1]`, `[exit 2]`, `[exit 3]`) — matches the author's count |

The third journey (`transcript_payload.sh`) still writes into the shared `$W/out`, but its
filenames (`rejected.bin`, `payload.bin`, `payload-region.bin`, `wipe-*.bin`,
`transcript_payload.txt`) do not collide with anything the operator journey writes, and
`build_pdf_payload.py` already writes `journey_payload.html`. No cross-clobber found —
noted only because the commit's separation claim covers two journeys and there are three.

Both builders run to completion and write distinct files:

```
wrote …/design/journeys/out/journey.html (763 KB)
wrote …/design/journeys/out/pathological/journey_pathological.html (818 KB)
<title>SeedHammer II operator journey</title>
<title>SeedHammer II — the pathological wallet</title>
```

and the pathological document is now built from its own journey — `timelock` ×3,
`hashlock` ×1, `key-00.xpub` ×1, `eleven keys` ×2 in the pathological HTML and **0** of
each in the operator HTML; `cosigner-00.xpub` ×2 in the operator HTML and **0** in the
pathological one. The wrong-transcript defect is genuinely fixed.

---

## Commit-message claims checked

| claim | holds? |
| --- | --- |
| "Both journeys now assemble out/backup-strings.txt from their own md1 chunks and their own freshly-encoded key cards, and engrave THAT" | **YES** — `transcript.sh:110`→`:122`, `transcript_pathological.sh:147`→`:152` |
| "The tracked fixtures are deleted" | **YES** — both paths absent from `git ls-files`; recoverable only via `git show c6c6943^:…` |
| "operator 12 cards / pathological 11 cards / SEMANTIC DIFF: NONE" | **YES** — independently reproduced, 0 differing of 23 cards on all four fields |
| "the md1 decodes to the same policy in both" | **YES**, and stronger than claimed: both md1 sets are *byte*-identical old-vs-new |
| "NOTHING KEY-MATERIAL CHANGED … any steel already cut from the old fixture stays valid and decodable" | **YES** — all 46 old chunks still decode cleanly under `mk 0.13.0` |
| "Superseded generations: mk1qpmn4upq…/mk1qpf7f8pq…/mk1qp0jgzpq…/mk1qpdw8zpq…/mk1qpghz4pq… all still in git history" | **YES** — `git log -S` finds 4 commits for each of the five prefixes |
| "step 7 said 'the eleven key cards' and encoded ONE. It now encodes all eleven (and all twelve on the operator side)" | **YES** — 22 and 24 mk1 lines produced |
| "The hardcoded --policy-id-stub 5b48af35 is now DERIVED … verified equal to the old constant" | **YES** — derived value is exactly `5b48af35`; old cards decode with `policy_id_stubs: 5b48af35` |
| "each with its own origin read from its own key file" | **YES** — 23/23 cards carry their own file's fingerprint and path |
| "the PATHOLOGICAL builder read the OPERATOR journey filename … Each now reads its own tracked transcript" | **YES** for the repointing (`build_pdf_pathological.py:54`) — but see **I-3** for what the new source pairing costs, and **M-3**: the pathological document *still prints* `bash transcript.sh` as its reproduction step |
| "build_pdf.py required keys.json, a file NEVER COMMITTED, so it could not run at all. It now derives fingerprints from the committed key headers and mk1 pairs from the bundle" | **YES** — `keys.json` has never been tracked (`git log --all -- design/journeys/keys.json` is empty); `build_pdf.py` now exits 0 |
| "both builders wrote out/journey.html … The pathological one is journey_pathological.html now" | **YES** — distinct paths, both written in one session without clobbering |
| "the two journeys SHARED $W/out … Confirmed: both bundles survive a back-to-back run, 25 lines each" | **YES** — verified in both orders |
| "design/journeys/shots/ has ZERO TRACKED FILES. All 39 present are untracked" | **YES** — `git ls-files shots` → 0; `ls shots` → 39 |
| "Rebuilding gives HTML with 19 and 13 'missing' placeholders" | **YES** — exactly 19 and 13 |
| "the a*/b* screenshots these two documents embed do not exist anywhere in the repo" | **PARTLY** — true for the pathological document (`a00-boot.png` … `b8-plate.png`, 13 files). The operator document's 19 missing shots are `j00-boot.png`…`k14-engrave-start.png`, `menu-4.png`, `p0/p2/p5/p11-plate.png`, `p2-screen.png`, `p17-settled.png` — no `a*`/`b*` among them. The *fact* (nothing regenerable, nothing tracked) is right; the `a*/b*` label describes only one of the two documents |
| "the PDFs are deliberately NOT overwritten" | **YES** — no `.pdf` appears in the diff |

---

## Open / could not determine

- **The screenshot layer is untestable from here, as the commit states.** I confirmed the
  counts and the zero-tracked fact but did not attempt an emulator re-walk, so I cannot
  independently confirm that re-walking would recover the missing frames.
- **`me bundle`'s own validation depth is out of scope.** I established that it exits 0 on
  a bundle with a silently missing key card (I-2), but I did not investigate whether it
  *could* detect that — a 5-of-12 bundle carrying 11 cards may be legitimately
  indistinguishable from an 11-key bundle at the codec level. The fix I suggest sits in the
  journey scripts, not in `me`.
- **I did not evaluate whether the published PDFs should now be regenerated.** The commit
  argues they should not (screenshots unrecoverable); with I-1 open, regenerating the
  pathological document today would in any case publish an empty key-card page.
- **`transcript_payload.sh` and `build_pdf_payload.py` were not reviewed** beyond checking
  for filename collisions with the two journeys in this commit's scope.
