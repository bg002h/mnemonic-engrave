# IMPL-S1S3 — fold 1, against the adversarial execution review

**Folder:** the P2 implementer, 2026-08-29.
**Review folded:** `design/agent-reports/IMPL-S1S3-adversarial-review.md`
(commit `32b94c4`) — RED, **1C / 1I / 3M / 3N** over `c3fefe4..5b0007a`.
**Branch:** `impl/descriptor-s1s3`, head **`06d1557`**. Nothing pushed, no tags,
no publishes, nothing on-device; the main checkout and the fork worktree were
never written to.

**Disposition: 7 findings FIXED, 1 DECLINED (N1, per the controller's ruling).**
Both blocking findings are closed, each with a red-first test and a
whole-workspace mutation check.

The review was accurate on every point I could reproduce, and I reproduced all
of them. Where it gave a measured value I re-derived that value through an
independent route rather than transcribing it — the addresses through the fork's
own `address.Receive`, the string counts through the built binary.

---

## Summary table

| # | severity | disposition | the measurement that shows it fixed |
| --- | --- | --- | --- |
| C1 | Critical | **FIXED** | one file, three flag states, one sentence — §3 |
| I1 | Important | **FIXED** | the device's own address printed, 4 constructions — §4 |
| M1 | Minor | **FIXED** | `--help` build-marks `descriptor`, not `md1` — §5 |
| M2 | Minor | **FIXED** | hostile `Name:` escaped and bounded, `cat -v` — §6 |
| M3 | Minor | **FIXED** (controller rule) | 3 paths, presence/absence pinned — §7 |
| N1 | Nit | **DECLINED** (controller ruling) | §8 |
| N2 | Nit | **FIXED** | even backtick count on every line — §9 |
| N3 | Nit | **FIXED** | walk erratum 2, re-measured — §10 |

---

## 1. Commits

| sha | subject |
| --- | --- |
| `c2b6358` | fold C1: conjunct 1's flag-DEPENDENT arm runs last, not first |
| `550de33` | fold I1: derive `address 0:` KEY BY KEY, which is what the device does |
| `f9e3848` | fold M1/M2/M3/N2: the help's build mark, operator-text quoting, one rule for the label warning |
| `06d1557` | walk: erratum 2 — W14's parenthetical names the wrong artifact for its number (WALK DOC ONLY) |

C1 and I1 are separate, per the brief; the M/N sweep is batched; the walk
erratum is its own commit and touches no code. This report lands on top.

---

## 2. The gate, after the fold

```
$ cargo nextest run --locked
     Summary [  32.189s] 560 tests run: 560 passed, 1 skipped
```

P2 closed at `544 passed`. **+16 tests**, all of them driving surfaces the diff
had no coverage for.

```
$ cargo clippy --all-targets --locked -- -D warnings     clean (exit 0)
$ cargo fmt --check                                      clean (exit 0)
$ grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs        -> 0
$ grep -rn '^#\[ignore' crates/ | wc -l                             -> 0
$ grep -c '^fn row_' crates/me-cli/tests/descriptor_refusals.rs     -> 36
```

The fork's Go seam, vector file byte-untouched:

```
$ sha256sum .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584
$ git diff 44e121a..HEAD --stat -- crates/me-cli/testdata/descriptor_seam_vectors.json
(empty)
$ go test ./nonstandard/ -count=1   ->  ok  seedhammer.com/nonstandard  0.019s
$ go vet ./nonstandard/  -> clean   ·   gofmt -l nonstandard/  -> clean
$ git -C .../seam-fork status --porcelain  -> (empty)
```

---

## 3. C1 — CRITICAL, FIXED

**The defect.** `admit()` ran `conjunct_1_shape(d, path)?` first, and its
`(Some(Multi::Unsorted), _, true)` arm returned the `multi_under_descriptor`
refusal on `Path::Descriptor`. The `?` short-circuited conjuncts 2–8, so
`--as descriptor` answered an anyone-can-spend wallet with *"This wallet can
still be engraved"* — a claim that is false (`--as md1` refuses the same file
permanently) on top of a funds-at-risk warning that never reached the operator.

**The fix shape.** Conjunct 1 is split into its two halves, and only the
flag-dependent one moves:

```rust
conjunct_1_shape(d)?;                        // --as-INDEPENDENT, first, unchanged
conjunct_2_threshold(d)?;  … conjunct_8_key_identity(d)?;
conjunct_1_multi_under_descriptor(d, path)?; // the ONE flag-dependent piece, LAST
```

§5.4's carriage rule is the authority: *"the §4.7 admission refusal where no
path admits the wallet — that determination quantifies over both paths, so it
needs no flag."* Ordering it last also makes conjunct 1's referral **true by
construction**: it is reached only when 2–8 hold, which is exactly when
`--as md1` admits the wallet. §6's row hedges the remainder ("for use-site paths
md1 can represent"), which is §5.3's business and not this predicate's.

`tr(multi(…))`, `wpkh(sortedmulti(…))` and `wsh(KEY)` keep their position at the
front — they are wrong whatever flag is given, and they name the wrongness
better than a threshold does.

**THE MEASUREMENT** — `wsh(multi(0,K1,K2))`, the review's own file, all three
flag states, after the fix. One sentence, three times:

```
$ me sysw pack --no-passphrase --in anyonecanspend.txt
me: threshold 0 means NO signature is required: anyone who can see this script can
    spend from it. This is almost certainly not the wallet you meant -- and if it
    already holds funds, treat them as at risk now. Nothing was packed.
    rc=3

$ me sysw pack --no-passphrase --as md1 --in anyonecanspend.txt
me: threshold 0 means NO signature is required: anyone who can see this script can
    spend from it. This is almost certainly not the wallet you meant -- and if it
    already holds funds, treat them as at risk now. Nothing was packed.
    rc=3

$ me sysw pack --no-passphrase --as descriptor --in anyonecanspend.txt
me: threshold 0 means NO signature is required: anyone who can see this script can
    spend from it. This is almost certainly not the wallet you meant -- and if it
    already holds funds, treat them as at risk now. Nothing was packed.
    rc=3
```

Before the fix the third of these was conjunct 1's referral. The review's paste
of that output is at `IMPL-S1S3-adversarial-review.md` §1.

**The test hole, closed — which the review named as the finding underneath the
bug.** No test drove `--as descriptor` on a `multi` at all: the vector file's
`multi` rows are `gate` rows, and gate rows are `--as`-omitted by construction,
so `gate/colliding-origin-multi` passed while the flag path went unexercised.

Eight new named tests in `tests/descriptor_as.rs` — the review's **seven**
constructed instances, one per suppressed conjunct, each asserting the SAME
refusal under all three flag states and asserting conjunct 1's referral is
absent:

| test | conjunct | the sentence that had been swallowed |
| --- | :-: | --- |
| `as_descriptor_on_multi_still_reports_threshold_below_one` | 2 | anyone-can-spend, "treat them as at risk now" |
| `as_descriptor_on_multi_still_reports_threshold_exceeds_keys` | 2 | "can never be satisfied" |
| `as_descriptor_on_multi_still_reports_key_count_exceeded` | 3 | "carries at most 15 keys" |
| `as_descriptor_on_multi_still_reports_mixed_network` | 5 | "All keys must share one network." |
| `as_descriptor_on_multi_still_reports_use_site_hardened` | 7 | "cannot be derived from an xpub (BIP-32)" |
| `as_descriptor_on_multi_still_reports_use_site_non_consecutive` | 7 | "only `<i;i+1>` pairs" |
| `as_descriptor_on_multi_still_reports_key_identity` | 8 | "contradicts itself" |

…plus a **CONTROL**,
`as_descriptor_on_a_sound_multi_still_gets_conjunct_1s_permanent_refusal`, which
stops the fix from degenerating into "delete the arm": a `multi` that passes
2–8 must STILL get conjunct 1's permanent refusal, and the test additionally
runs `--as md1` on that same file at rc=0 to prove the referral it makes is
true.

**RED first:** `19 run: 12 passed, 7 failed` — the seven instances red, the
control green throughout.

**Mutation, whole workspace:**

```
MUTANT: the flag-dependent arm restored to the front
  -> 552 tests run: 545 passed, 7 failed
```

---

## 4. I1 — IMPORTANT, FIXED (the review's preferred direction)

**The defect.** The block declined to derive `address 0:` for a wallet whose
keys want different receive indices, and blamed their use-site *depths*. Every
clause was false: the address exists, the keys blamed were at the same depth,
and a pair that genuinely differs in depth never reached the branch.

**I re-derived the review's evidence rather than inheriting it**, through the
fork's own `address.Receive` (`scripts/descriptor-seam-vectors/goprobe` against
`_work/seam-fork` @ `1f09537`, Go 1.26.3) — the DEVICE, not this build and not
`md_codec`:

```
wsh(sortedmulti(2,K1/<2;3>,K2/<0;1>/*)) -> bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy
wsh(sortedmulti(2,K1/<2;3>,K2/<0;1>))   -> bc1qlccgxwlhr0rp7xfedcau022p50ulf9r3e33anqqdrevvdrdeqj9s8leyuw
wsh(sortedmulti(2,K1/<2;3>,K2/<2;3>))   -> bc1qxwcmdqhtvjp6uu6asj0vgz9yvylhwy3ky2y9r3r9lz68rgkwalgqq9dyds
wsh(sortedmulti(2,K1/*,K2/<0;1>/*))     -> bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd
```

Both of the review's values reproduce to the character. The evidence
generalises, so I took the controller's **preferred** option: derive, rather
than describe the limitation.

**Why the old route could not, stated because it is the reason a second
derivation exists at all.** `md_codec::Descriptor::derive_address` takes ONE
`(chain, index)` for the whole descriptor, and md1's use-site path always ends
in exactly one wildcard level which consumes that index. The wallet above wants
`K1/2` and `K2/0/0`: the **last** element differs per key, and no amount of
pre-derivation can change a trailing element. I worked the alternatives before
writing code — pre-deriving either key, moving the fixed step into the
multipath, absorbing it into the origin (md_codec does not consult the origin
for derivation) — and each fails on the same trailing level. It is a limitation
of the twin, not a property of the wallet, exactly as the review said.

**The fix shape.** `descriptor::derive::address_0` walks each key at ITS own
receive path:

| use-site | that key's receive-address-0 path |
| --- | --- |
| absent | `/0/0` — §5.3(a′)'s materialised `<0;1>/*` |
| `/*` | `/0` |
| `/i/*` | `/i/0` |
| `<i;i+1>` | `/i` — one address per chain, so receive 0 IS the alternative |
| `<i;i+1>/*` | `/i/0` |

CKDpub comes from `bitcoin::bip32::Xpub::derive_pub` — **no new dependency**,
`bitcoin` has been direct since P2.3 — and the script is built for the seven
shapes, with `sortedmulti` sorting the DERIVED keys at the use site (BIP-67),
which is what makes `multi` and `sortedmulti` not synonyms.

**Why a second derivation is safe, and how it is held.** Two implementations of
one answer is the F-212 divergence class this cycle exists to guard against. So
the new one is gated against the old, in `tests/descriptor_seam.rs`:

```
the_two_derivations_agree_wherever_both_can_derive
  · every vector row the cascade parses, both routes run, equality required
  · AND every one of the 20 device-measured `address_0` values in the file is
    reached by the per-key walk (assert_eq against POP.address_0)
```

**It passed on the first run** — across 16-key `sh(wsh(sortedmulti))`, `tr`,
`pkh`/`wpkh`/`sh(wpkh)` promotions, unsorted `multi`, and all three multisig
wrappers. So agreement here is agreement with the DEVICE, not with itself.
`md_codec` remains the sole authority for what is PACKED; nothing in the new
module ever reaches a card.

The per-key walk is now the **single production path** for `address 0:`; the
twin survives as the differential's second opinion — one production answer, an
independent check in the suite.

**The `None` arm** now claims nothing about the wallet, and is proved
unreachable in the FULL tier rather than argued:
`every_full_tier_wallet_has_an_address_0` runs the walk over every parsed vector
row whose conjuncts 2–8 hold.

**THE MEASUREMENT** — the review's construction, after the fix:

```
$ me sysw pack --no-passphrase --as md1 'wsh(sortedmulti(2,K1/<2;3>,K2/<0;1>/*))'
      wallet-id: none -- this wallet has no md1 policy form; identify it by the
                 checksum in the descriptor line and by address 0.
      address 0: bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy
      compare against your wallet software's first receive address before engraving.
```

The dead end the review read twice is gone: `wallet-id: none` says "identify it
by address 0", and address 0 is now there — and it is the device's.

**RED first:** `22 run: 20 passed, 2 failed`.

**Mutation, whole workspace:**

```
MUTANT: `address 0:` back on the whole-descriptor twin
  -> 556 tests run: 554 passed, 2 failed
```

Four constructions pinned, including the two the twin already handled — so a
future change cannot fix the mixed case by regressing the uniform one.

---

## 5. M1 — FIXED

`AsForm` implements `clap::ValueEnum` **by hand**, because the derive macro's
help text comes from doc comments and cannot read `DESCRIPTOR_PATH_SHIPPED`.
Both markings are now gated on the same two constants, so they cannot drift.

```
$ me sysw pack --help
          Possible values:
          - descriptor: The canonical re-encoded descriptor, as one `Descriptor` record (not available in this build)
          - md1:        The BIP-388 decomposition, as md1 text cards (`MdMk` records)
```

The test pins **both halves** — `md1` unmarked as well as `descriptor` marked,
so it cannot pass on a help that marks everything — and additionally asserts
that the choice block and the help agree about what this build carries.

---

## 6. M2 — FIXED

One helper, `refusal::quote_operator`, for every fragment of the operator's own
file that a refusal quotes back: control bytes **escaped** rather than stripped
(the operator should still see something odd is in their file), bounded at 48
columns with an ellipsis.

```
$ me sysw pack --as md1 --in evil.txt … 2>&1 | grep warning | cat -v
me: warning: the label ""; rm -rf /  \x1b[31mRED\x1b[0mAAAAAAAAAAAAAAAAAM-bM-^@M-&"
    is not carried by any record format and will not appear on the device.
```

No raw ESC reaches the terminal, and the test asserts `address 0:` is still on
screen after the warning. That adjacency is the whole point of the finding: the
warning sits beside the verification surface, and a clear-screen sequence could
have scrolled it away at the moment it is meant to be compared.

Folded now rather than filed: this is output **integrity**, not secret handling
— the bytes are public wallet-export text and nothing here decides what is
packed — so the 2026-08-27 severity ruling does not apply to it.

---

## 7. M3 — FIXED, per the controller's rule

The label warning prints **exactly on paths that pack**. Implemented by moving
it after the follower decision and gating it on `Decision::Pack`.

| invocation | before | after |
| --- | --- | --- |
| `--as md1` (packs) | printed | **printed** |
| `--as descriptor` (window refusal) | printed | **absent** |
| `--as` omitted (choice block) | absent | **absent** |

One test, `the_label_warning_fires_exactly_on_the_paths_that_pack`, pins all
three with presence AND absence. The reason is in the code at the site: the
text is a statement about what was just packed, and *"Nothing else is lost"* is
actively wrong beside a refusal where the whole wallet was.

---

## 8. N1 — DECLINED, per the controller's ruling

§5.4 says *"identify it by the checksum in the **canonical** line"*; the code
says *"the **descriptor** line"*. **No change.** The block labels that line
`descriptor:`, so the code's wording is the more executable of the two — the
operator can find the line it names. Recorded here as a deliberate divergence
rather than a drift; **P3 notes it in records** so the spec and the code are
reconciled deliberately rather than by whoever notices next.

---

## 9. N2 — FIXED

The same `quote_operator` helper. Two descriptors in one document:

```
before:  ... the use-site path is not a path: `<0;1>/*))
         wsh(multi(2`.
after:   ... the use-site path is not a path: `<0;1>/*))\nwsh(multi(2`.
```

The refusal row and the exit code are unchanged, as the brief required — only
the rendering. The test asserts every emitted line has an **even** backtick
count, which pins the general property rather than this one case.

---

## 10. N3 — FIXED, walk doc only (`06d1557`)

Re-measured through the built binary rather than transcribed:

```
zpub6qpFgGWoG7bKm…        (bare — Journey 2's actual clipboard line)
    -> 2 strings, 85 + 83 = 168 chars
[4bbaa801/84h/0h/0h]zpub… (keyed, BIP-84 origin)
    -> 3 strings, 67 + 67 + 67 = 201 chars
```

The walk measured the BARE key and wrote down the keyed one. Marked as
**ERRATUM 2** at the site, with a numbered entry under a new dated "Corrections
to this log" heading, in the same style as W14's existing erratum. The original
text is left in place — a walk log is a record of what was asserted, and
rewriting it in place would destroy what corrections are for.

The entry records the consequence: correction 1 in the same log already
re-derived the PLATE COUNT from this number (two plates, which stands for the
bare card); the keyed card would be three. And it names the pattern — this is
the second citation-by-description error in W14, both of the same shape, a claim
recorded about an artifact the log did not re-read.

---

## 11. Propagation sweep

Over this fold's own edits, `crates/` (code and tests):

| superseded form | hits | |
| --- | ---: | --- |
| `different depths` | **0** | I1's old sentence |
| `no single first address to compare` | **0** | I1's old sentence |
| `conjunct_1_shape(d, path)` | **0** | C1's old signature |
| `#[ignore` at column 0, crate-wide | **0** | the P2 gate |

`This wallet can still be engraved` — C1's false referral — appears three times
in `crates/`, and each is checked to be correct rather than merely present: a
comment in `admit.rs` explaining the defect, a doc comment on the C1 test, and a
**negative** assertion (`!err.contains(…)`). It is never asserted as an expected
output. The sentence itself is still the live text of `multi_under_descriptor`,
which is correct — the control test proves it still fires for a sound `multi`,
and that the referral it makes is now true.

**The verbatim old I1 sentence survives only in the persisted agent reports**
(`IMPL-P2-report.md`, `IMPL-S1S3-adversarial-review.md`), which is what they are
for; a persisted report is not rewritten. The I1 test's doc comment was
rewritten during this fold to paraphrase the defect and point at the review
rather than reproduce the string, so a future grep of the tree finds the record
and not a live line.

---

## 12. What a re-reviewer should look at first

1. **`descriptor/derive.rs` is new code on the funds path.** It is gated
   differentially against `md_codec` and against 20 device-measured addresses,
   and it passed both on the first run — which is either good code or a
   differential that agrees for the wrong reason. The assertion that makes it
   the former is the `assert_eq!(a, want, …)` against the FILE's values, not the
   `assert_eq!(a, b, …)` between the two routes.
2. **`multisig()`'s `push_int` for n > 16.** Reachable only for 17–20-key
   `wsh`/`sh(wsh)` wallets; the 16-key row in the corpus exercises `push_int(16)`
   but nothing exercises 17–20. The differential would catch a divergence only
   if `md_codec` can also derive those, which it can. Not asserted directly.
3. **C1's ordering is now load-bearing in a new way.** `conjunct_1_shape` and
   `conjunct_1_multi_under_descriptor` must stay split; merging them back is the
   defect. The mutation above is the guard.
