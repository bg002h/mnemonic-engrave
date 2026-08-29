# IMPL-S1S3 — proportional RE-REVIEW of fold 1

**Scope:** `git diff 32b94c4..83703b4` on `impl/descriptor-s1s3`, worktree
`/scratch/code/shibboleth/_work/impl-s1s3`, binary `target/debug/me` built from
`83703b4`. Two questions only: **(1) did the fold fix each finding of
`IMPL-S1S3-adversarial-review.md`, and (2) did the fold introduce a new
defect.** Everything before `32b94c4` is taken as closed and was not
re-derived.

**Counts: 0 Critical / 1 Important / 1 Minor / 3 Nit.**

**Verdict: RED — 0C but one Important (I-A) is open.** All seven review
findings the fold claims FIXED are fixed, verified by execution; the Important
is fold-INTRODUCED and is in the records, not the code.

---

## 0. Instruments

Three independent oracles, none of them `me`'s own code:

1. **A from-scratch Python BIP-32 / BIP-67 / script / bech32 oracle** written
   for this re-review (`oracle.py` in the scratchpad) — secp256k1 point
   arithmetic, CKDpub, Core's `CScript << int64_t` number encoding, bech32 and
   bech32m, BIP-340 `lift_x`. Calibrated before use against two addresses the
   adversarial review had already measured on the device
   (`bc1qv70wqy0t9vp…` for the I1 construction, `bc1qadgf37z…` for the
   corpus's `<0;1>/*` wallet) — both reproduced to the character.
2. **The device**, via `scripts/descriptor-seam-vectors/goprobe` rebuilt
   against `/scratch/code/shibboleth/_work/seam-fork` @ `1f09537`
   (Go 1.26.3, `/nix/store/33fw5m31…-go-1.26.3`) — `address.Receive`.
3. **A PRE-FOLD baseline binary**, built from a `git archive 32b94c4` extract
   at `/scratch/code/shibboleth/_work/_rr-baseline` (no git metadata touched,
   nothing in the worktree modified).

Nothing tracked was modified, nothing committed, nothing pushed.

---

## 1. Findings

### I-A (IMPORTANT, FOLD-INTRODUCED) — C1's reorder changed the `--as descriptor` outcome for the colliding-origin `multi`, and three normative/record sites still assert the OLD order — one of them a file pinned byte-identically in two repos

**Constructed measurement.** The corpus row `gate/colliding-origin-multi`, its
own input, under `--as descriptor`, before and after the fold:

```
$ .../_rr-baseline/target/debug/me sysw pack --no-passphrase --in collide-multi.txt --as descriptor
me: the device's descriptor parser accepts `sortedmulti` and not `multi`. This wallet can
    still be engraved: `--as md1` encodes `multi` policies (…)
rc=3

$ .../impl-s1s3/target/debug/me sysw pack --no-passphrase --in collide-multi.txt --as descriptor
me: this wallet description contradicts itself: keys 0 and 1 both claim origin
    `dc567276/48h/0h/0h/2h` but name different keys -- one origin identifies exactly one
    key, so no wallet matches this description. …
rc=3
```

**The new behaviour is right.** Under the old order this input received
conjunct 1's referral — *"This wallet can still be engraved: `--as md1` …"* —
while `--as md1` refuses the same file permanently with the key-identity
refusal, in every build. That is C1's finding 1 verbatim, on the one `multi`
input the corpus actually carries. The fold is correct and §6's key-identity
row (*"**`EXIT_REFUSED` (3)**, both `--as` paths"*, spec line 1415) is now true
where it previously was not. **No code change is being asked for.**

**What is now false.** Three sites still state the superseded order, none of
them touched by the fold, none mentioned in the fold report or in any of the
four commit messages:

1. **`design/SPEC_descriptor_input.md:1663-1665` — §7 clause 8, NORMATIVE**
   (this is the coverage manifest that governs the vector file):

   > *"…and the colliding-origin `wsh(multi(…))` twin — whose conjunct-8
   > refusal binds the `--as md1` path ONLY: **under `--as descriptor` a
   > `multi` gets conjunct 1's permanent refusal first**, per the ruling stated
   > at §5.1, §6 and §11 (PLAN-r3's I2)."*

   Unambiguously an ordering claim about conjunct 1 versus conjunct 8, and
   unambiguously false against `83703b4`.

2. **`design/SPEC_descriptor_input.md:1205-1211` — §5.4's tier parenthetical:**

   > *"…a conjunct-8-FAILING `multi`, the colliding-origin twin, is PARTIAL
   > like every conjunct-8 failure … **But that refusal, under explicit
   > `--as descriptor`, is conjunct 1's PERMANENT shape refusal naming
   > `--as md1` — in EVERY build**…"*

   The justification that follows it is a contrast with the *window* refusal,
   so this sentence is at least ambiguous — but as written it now reads false.

3. **`crates/me-cli/testdata/descriptor_seam_vectors.json:1291`** — the
   `source` field of `gate/colliding-origin-multi`:

   > `"…the multi twin; conjunct 8 binds the --as md1 path, and under --as
   > descriptor conjunct 1 refuses first"`

   This file is pinned **byte-identically in two repositories** — verified this
   session:

   ```
   0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
   0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  crates/me-cli/testdata/descriptor_seam_vectors.json
   ```

   so the false provenance line is in the fork too, where it is the only
   explanation a Go-side implementer has for the row.

**Why this is Important and not Minor.** §7 clause 8 is not description, it is
a rule — and the rule it states is the one that PRODUCED C1. Anyone
implementing §7 clause 8 as written re-introduces a refusal whose referral is
false on a funds-relevant path, and the vector row's `source` line is the exact
place the next implementer (in either language) would look. The tests cannot
catch it: gate rows are `--as`-omitted by construction, so the row still passes
while its own prose describes the opposite outcome (this is the same blind spot
the review named under C1). The fold report's propagation sweep (§11) was
scoped to `crates/` code and tests only, so none of these three sites was in
its search space.

**Not prescribing the fix.** At least two shapes exist — amend §7 clause 8 and
§5.4's parenthetical to the carriage rule the fold implements (the same shape
as this cycle's §6 amendment, `de35e30`), with or without regenerating the
vector file's `source` prose. Regenerating the vector file moves the sha256 and
forces the two-repo re-pin the README describes, which is real cost and a real
decision; leaving it leaves a false line in a shared artifact. That trade is
the controller's to make. What is not optional is that all three currently
contradict the build.

**Not counted separately:** `design/SPEC_descriptor_input.md:1916` (§11 item 5,
case 5) is NOT affected — its test (`item_5_the_five_case_matrix`,
`descriptor_refusals.rs:823-905`) uses `neither/wsh-multi`, a `multi` that
passes conjuncts 2–8, which still gets conjunct 1's refusal. Verified by
execution. Likewise `§5.1`'s line 948 is about conjunct 1 versus the *window*
and remains true.

---

### M-A (MINOR) — `quote_operator` neutralises C0/C1 control bytes but not the two other classes that can rewrite the line it prints: Unicode `Cf` (bidi overrides / isolates) and the `"` delimiter it is interpolated into

M2's own class, with the escapes now in place. Both constructed against
`83703b4`.

**(a) The `"` is not escaped, so a label can close the quote and continue in
`me`'s own voice.** Label `ok" -- nothing is wrong with this wallet. "`
(43 chars, inside the 48-char bound, no ellipsis):

```
me: warning: the label "ok" -- nothing is wrong with this wallet. "" is not carried by any
    record format and will not appear on the device. Nothing else is lost.
```

The `""` is the only tell. A second construction reaches a fabricated address
fragment before truncation cuts it:
`me" address 0: 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2` renders as
`the label "me" address 0: 1BvBMSEYstWetqTFn5Au4m4GFg7xJaN…"`.

**(b) `char::is_control()` covers only `Cc`, so bidi controls pass through
raw.** Label `a\u{202E}KCATTA\u{202C}b\u{200B}\u{2066}x\u{2069}` — the emitted
bytes, read off stderr:

```
b'me: warning: the label "a\xe2\x80\xaeKCATTA\xe2\x80\xacb\xe2\x80\x8b\xe2\x81\xa6x\xe2\x81\xa9" is not carried …'
```

U+202E, U+202C, U+200B, U+2066 and U+2069 all survive. In a bidi-aware
terminal `a\u{202E}KCATTA` renders as `aATTACK`, and an unterminated override
reorders the remainder of the warning line.

**Why it is Minor and not more.** The block prints `address 0:` *before* the
warning and on its own line (measured on every probe: `address 0:` at output
line 4, the warning at line 10), so neither construction can move or alter the
address the operator is told to compare — which is the harm M2 was folded to
prevent, and which IS prevented: no raw ESC, CR, BS or VT reaches the terminal
in any of the 14 hostile labels tried, and a legitimate non-ASCII label
(`Grüße — Konto Nº1 ✓ 日本語`) passes through unmangled. Recorded as the
residual of a Minor, not as a new blocking defect. Not secret handling.

---

### N-a (NIT) — the backtick-parity property N2 installs does not hold in general, and the fold report says it does

`IMPL-S1S3-fold1.md` §9: *"The test asserts every emitted line has an **even**
backtick count, which pins the general property rather than this one case."*
The loop in `a_quoted_fragment_never_spans_a_newline` runs over the stderr of
**one** input. One character falsifies the property:

```
$ printf 'wpkh([dc567276/48h/0h/0h/2h]xpub6DiYr…/a`b)' > n2b.txt
$ me sysw pack --no-passphrase --in n2b.txt --as md1
me: … which failed because: the use-site path is not a path: `a`b`.
```

Three backticks on the line — the same ambiguous rendering N2 was about, from
an operator-supplied fragment `quote_operator` passes through. Also reproduces
via the origin path (`[dc567276/48h/0`h]…`). The outcome and exit code are
right in every case; only the rendering. Nit, as N2 was.

### N-b (NIT) — `quote_operator`'s bound is 48 CHARACTERS, documented as 48 columns

`refusal.rs`'s doc says *"bounded at 48 columns"* and the module note repeats
it; `width` accumulates `piece.chars().count()`. A 48-character CJK or emoji
label occupies ~96 columns (measured: a 120× `日` label truncates to 48 glyphs
+ `…`). Still bounded and still far short of a screen, so this is wording, not
behaviour.

### N-c (NIT) — N1's declination has no entry in `design/FOLLOWUPS.md`

`IMPL-S1S3-fold1.md` §8 declines N1 (`descriptor` vs `canonical` line) and
records *"**P3 notes it in records** so the spec and the code are reconciled
deliberately"*. `grep` over `design/FOLLOWUPS.md` finds no entry for it, so the
only record is prose inside a report, and the standing rule ("record the owning
phase in each follow-up entry so reconciliation is a grep") is not satisfied.
One line in `FOLLOWUPS.md` with `owning phase: P3` closes it.

---

## 2. Question 1 — every finding, verified against the CODE

The fold report's dispositions were re-derived, not inherited.

| # | claim | verified how | verdict |
| --- | --- | --- | --- |
| C1 | fixed | all **7** review instances × 3 flag states, executed | **FIXED** |
| I1 | fixed | 91 constructed shapes vs 2 independent oracles | **FIXED** |
| M1 | fixed | `--help` executed; parsing differentially unchanged | **FIXED** |
| M2 | fixed | 14 hostile labels; no raw control byte emitted | **FIXED** (residual M-A) |
| M3 | fixed | 3 paths × 2 labelled fixtures, presence AND absence | **FIXED** |
| N1 | declined | controller ruling; no code change present | **DECLINED** (see N-c) |
| N2 | fixed | two-descriptor stdin executed | **FIXED** (residual N-a) |
| N3 | fixed | both numbers re-measured from the packed blob | **FIXED** |

### 2.1 C1 — all seven instances, all three flag states

Each of the review's seven suppressed conjuncts, reconstructed as a `multi`
input and run under `--as` omitted / `--as md1` / `--as descriptor`. **Every
one gives the identical flag-independent refusal at rc=3 under all three, and
none carries conjunct 1's *"This wallet can still be engraved"* referral:**

| conjunct | input | the sentence, now reaching the operator under `--as descriptor` |
| :-: | --- | --- |
| 2 | `wsh(multi(0,K1,K2))` | *"…treat them as at risk now."* |
| 2 | `wsh(multi(5,K1,K2))` | *"can never be satisfied"* |
| 3 | `narrowed/wsh-sortedmulti-21-keys` as `multi` | *"`sh(multi(…))` carries at most 15 keys…"* |
| 5 | `narrowed/mixed-network` as `multi` | *"All keys must share one network."* |
| 7 | `…/<0;1>/*h` | *"cannot be derived from an xpub (BIP-32)"* |
| 7 | `…/<0;2>` | *"only `<i;i+1>` pairs"* |
| 8 | `gate/colliding-origin-multi` | *"contradicts itself … no wallet matches this description"* |

**The control holds.** A SOUND `multi` (`wsh(multi(2,K1/<0;1>/*,K2/<0;1>/*))`)
still receives conjunct 1's permanent refusal under `--as descriptor`, after
the full FULL-tier block (`wallet-id: 0501609a…`, `address 0: bc1qadgf37z…`),
and its referral is TRUE: the same file packs at rc=0 under `--as md1`. So the
fix is not "delete the arm", and the ordering makes the referral true by
construction.

**Ordering promises checked, per the brief.** `conjunct_1_shape` — the
`--as`-independent half (`tr(multi(…))`, `wpkh(sortedmulti(…))`, `wsh(KEY)`) —
still runs FIRST, so no §6 shape row lost its precedence. The admission SET is
unchanged (the same eight conjuncts, all required), so every carriage decision
keyed on `admit(…).is_err()` is untouched; only refusal SELECTION moved. §11
item 5's five-case matrix is intact (its `multi` case is a sound one). The one
place the new order contradicts a written promise is **I-A**.

### 2.2 I1 — the per-key walk, against two oracles that are not `me`

**The review's own constructions reproduce.** All four shapes now print the
device's address, and the review's two measured values match to the character
(`bc1qv70wqy0t9vp…`, `bc1qlccgxwlhr0rp…`).

**The unexercised `push_int` branch the fold report flagged (17–20 keys) —
CONSTRUCTED and clean.** Twenty independent cosigners derived from scratch,
eight wallets, three flag states each = 24 comparisons, **0 mismatches**
against the Python oracle, and the device agrees on all seven it parses:

| wallet | `me` = Python = device |
| --- | --- |
| 17-of-17 `wsh(sortedmulti)` | `bc1qcjmuu7jjx22mku7d7gzwdvpu280pjsc7zzh9g0cwx0x4x3yaaw4q82uwhu` |
| 2-of-17 `wsh` | `bc1qg3c3q2hs0hxllj6qs4sdp52n2s6neaf6h27a58yazs7e6l0t6uss3hf4m3` |
| 20-of-20 `wsh` | `bc1qadzs0ytkv87trrsy8f4gvdvmqc3f6yqtwlycdmfwltnf4ndcpw5q3gdsff` |
| **17-of-20 `wsh`** (`push_int` on BOTH k and n) | `bc1qwg5lsse28wxz78a4xvu6axyxu3uvhmdmg9tncha8hhtyv2zd3tkqxxnkzy` |
| 2-of-20 `wsh` | `bc1qwt74fm4msh9emtkpyy23fcpgx4esah7raxwqn4fspjpxm42vghhqcm3un3` |
| 17-of-20 `sh(wsh(sortedmulti))` | `3N92ZkU9SSS5v3Dy5ryBMznPZLA7v2Mb7f` |
| 16-of-16 (control, `OP_16`) | `bc1q56cvwasm6a5d00ennw5mzvzyjee04k77nrkeukevs0ckw9dr3kuqc4f94v` |
| 17-of-17 `wsh(multi)` (device refuses at parse, as §6 says) | `bc1qgxaan6h6htqstzp8y0kzul9n22phx3kcqqg8ex9gq8shefh3sd9qyuj3ft` |

So `Builder::push_int` agrees with Core's `CScript << int64_t` for 17–20 in
both the threshold and the key-count position.

**Edge sweep — 78 further constructed descriptors, 0 mismatches** against the
Python oracle, with the device agreeing on all 77 it parses (the 78th is a
`multi`): every one of the **49** use-site pairs over conjunct 7's closed set
(`absent`, `/*`, `/<0;1>/*`, `/<2;3>/*`, `/<0;1>`, `/<4;5>`, `/7/*` × the
same), uniform **and mixed**, including the mixed childless/wildcard pair and
the wildcard-less `<i;i+1>` at `i≠0`; all three multisig wrappers; the bare
`sh(sortedmulti)` at conjunct 3's 15-key ceiling (513-byte redeemScript, one
byte class short of `Address::p2sh`'s 520-byte error); `pkh`/`wpkh`/`sh(wpkh)`
at seven use-sites each; **depth-0 master xpubs with no origin block**; and
testnet (`tpub` → `tb1…`). Plus **5** `tr(KEY)` forms against a BIP-86
oracle — 0 mismatches after a bug in *my* oracle (a missing `lift_x`
even-y normalisation) was found and fixed; `me` and the device were right.

**Index boundary:** `/2147483647/*` and `/<2147483646;2147483647>` — the last
non-hardened index — derive correctly, three-way.

**The refused classes cannot reach `derive`.** `derive::address_0` is called
only inside the FULL-tier branch, whose predicate is `admit(d, Path::Md1)`, so
conjunct 7 has already run. Executed: `/<0;1>/*h`, `/*h`, `/0h/*` (hardened),
`/<0;2>` (non-consecutive), `/0/1/*`, `/<0;1>/*/*` and `/2147483648/*` all
refuse at rc=3 with **no `address 0:` line at all**. Combined with the 49-pair
sweep, the `None` arm ("this build could not derive one") is unreachable for
every member of the closed set, not merely for the 20 corpus rows
`every_full_tier_wallet_has_an_address_0` walks.

**The gate test is not vacuous.** `the_two_derivations_agree_wherever_both_can_derive`
carries an `agreed >= 25` floor and asserts `against_file == POP.address_0`;
independently counted, the vector file carries `address_0` on exactly **20** of
its 71 rows, matching `POP.address_0`. Its device-anchored half is the
`assert_eq!(a, want)` against the file, as the fold report says.

### 2.3 M1 / M2 / M3 / N2 / N3

* **M1.** `me sysw pack --help` marks `descriptor` *(not available in this
  build)* and leaves `md1` unmarked, agreeing with the choice block. The
  hand-written `ValueEnum` is behaviourally identical to the derive it
  replaced: `--as md1` / `--as descriptor` accepted, `--as MD1` /
  `--as Descriptor` / `--as` (empty) rejected with byte-identical clap errors
  on both binaries.
* **M2.** 14 hostile labels (clear-screen, cursor-up, `\r` overwrite,
  BEL/BS/VT, 300 bytes, 120 CJK, 30 emoji, C0+DEL, backticks, quote injection,
  bidi). **No raw control byte reaches stderr in any of them**, output is
  bounded, `address 0:` is printed above the warning in all 14, and a
  legitimate non-ASCII label is unmangled. Residual: **M-A**.
* **M3.** Executed on both labelled corpus fixtures:

  | fixture | omitted | `--as md1` | `--as descriptor` |
  | --- | --- | --- | --- |
  | `bluewallet-sh-fixture` | rc 2, no pack, **no warning** | rc 0, **packs, warning** | rc 3, no pack, **no warning** |
  | `json-label-descriptor` | rc 3, no pack, **no warning** | rc 3, no pack, **no warning** | rc 3, no pack, **no warning** |

  Warning ⟺ packed, on both fixtures. Rule holds.
* **N2.** Two descriptors on stdin now render
  ``the use-site path is not a path: `<0;1>/*))\nwsh(sortedmulti(2`.`` — one
  line, backticks closed, row and exit code unchanged. Residual: **N-a**.
* **N3.** Re-measured from the packed blob rather than transcribed:
  `promotion/02-bare-zpub` → **2 records, 85 + 83 = 168**;
  `promotion/07-origin-84h-zpub` → **3 records, 67 + 67 + 67 = 201**. Exactly
  what erratum 2 records. The commit touches only the W14 site and the new
  "Corrections to this log" heading (32 insertions, 1 deletion, walk doc only).

---

## 3. Question 2 — the whole-corpus differential, pre-fold vs post-fold

The instrument for "did the fold change anything it did not mean to": **157
inputs** (all 71 vector rows + 86 constructed) × **3 flag states** = **471
invocations**, run on the `32b94c4` baseline binary and on `83703b4`, comparing
exit code, the full stderr+stdout text, and the sha256 of the packed container.

**471 invocations, 40 differ, in exactly 4 classes — every one an intended
fix:**

| class | count | what changed |
| --- | ---: | --- |
| I1 | 36 | the false *"not derived … different depths"* line replaced by a real address + compare prompt (all on `<4;5>` shapes; addresses independently confirmed) |
| M3 | 3 | the label warning removed from `bluewallet-sh-fixture --as descriptor` and both non-packing paths of `json-label-descriptor` |
| C1 | 1 | `gate/colliding-origin-multi --as descriptor` — conjunct 8 instead of conjunct 1 (**I-A**) |

**431 invocations byte-identical**, including every packed container blob
(`--as md1` sha256 unchanged on all 40 packing inputs). No exit code changed
anywhere. Nothing in the record surface, the gate, the window, the carriage
rule, the promotion announcement or the §6 texts moved.

### 3.1 Propagation sweep (the brief's item 5)

Whole tree, not just `crates/`:

| string | live hits outside `design/agent-reports/` |
| --- | ---: |
| `different depths` | **0** |
| `no single first address to compare` | **0** |
| `address 0: not` | **0** |
| `not derived` (in `crates/`) | **1** — a NEGATIVE assertion, `descriptor_as.rs:686: !err.contains("not derived")` |
| `conjunct_1_shape(d, path)` | **0** |

Persisted reports retain the old strings, which is what they are for. No spec
or plan text pinned I1's superseded sentence — §5.4's own wording is just
*"**`address 0:` receive address 0**"* with no escape hatch, so the fold moved
the implementation TOWARD the spec here. The sweep for the reverse direction —
text the diff falsified without touching — is what produced **I-A**.

---

## 4. Items explicitly NOT re-derived

Per the brief: the spec's GREEN, the plan's GREEN, P0/P1/P2, the adversarial
review's own §2 clean list (the md1 round trip, the record-surface baseline,
the §6 amendment, the two walk journeys, the `--as` flag surface, the grammar
edges), the controller's machine-checks (nextest 560/560 + 1 skipped,
clippy/fmt, the vector sha, `0 #[ignore]`, `36 == 36`, the two mutation runs),
and the `bitcoin` dependency question. The fold's own mutation tables were
taken as given; the anti-vacuity guards in the two new gate tests were read and
are real.

---

## 5. Verdict

**RED — 0C / 1I / 1M / 3N.**

The code half of this fold is sound, and the two blocking findings are closed
with room to spare. C1's fix is right in shape as well as in outcome: splitting
conjunct 1 makes its referral true *by construction* rather than by a text
edit, the control test stops it degenerating into a deletion, and all seven of
the review's instances now answer identically under all three flag states. I1's
new `derive.rs` is the largest risk in the diff and it survived everything I
could aim at it — 91 constructed wallets across the whole of conjunct 7's
closed set, three script wrappers, four single-sig forms, both networks,
depth-0 keys, the max-index boundary and the 17–20-key `push_int` branch no
corpus row reaches — agreeing to the character with a from-scratch BIP-32
oracle and with the device on every input the device will parse. The
whole-corpus differential says the fold changed 40 of 471 invocations and every
one of them is a fix.

What is open is the fold's shadow on the records. C1's reorder silently
falsified a NORMATIVE §7 clause, a §5.4 parenthetical, and a provenance line
inside the one artifact this cycle pins byte-identically across two
repositories — and the rule those sites state is precisely the rule that
produced C1. That is a fold-introduced defect in the class this project has
measured to be its weakest: a diff falsifying text it never touched, invisible
to a suite whose gate rows cannot reach the flag path in question.
