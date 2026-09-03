# composer S4 — implementation report, Tasks 1–3

**Implementer:** single opus agent, UC off, against
`IMPLEMENTATION_PLAN_composer_S4_acceptance.md` at mnemonic-engrave
`5a5f3df977fe78a3c5c485c21b0715e8d07d1567` (R0 GREEN). Brief:
`design/agent-briefs/composer-S4-implementer-brief.md`.

**Outcome: Tasks 1 and 2 DONE and committed. Task 3 STOPPED — not written.**
The plan's Task 3 itineraries cannot be executed on the emulator, and not
because of anything in the plan's expected VALUES: the shipped
`composerPickScreen` moves its cursor only on `ButtonFilter(Up)` /
`ButtonFilter(Down)` and registers no touch target, and the SeedHammer II has
no directional buttons. **Every row but a page's first row is unselectable on
the machine**, which puts `n = 2` (the keyed arm's Path 1) and `n = 3` (the
keyless arm's only path) out of reach — and, once one path exists, `Done` too.
Measured below with a positive control. §5 of the brief forbids me to decide
what to do about that, so I stopped and wrote it down.

## Worktrees

| repo | path | branch | base |
| --- | --- | --- | --- |
| seedhammer fork | `/scratch/code/shibboleth/wt-composer-s4-emu` | `composer-s4-emu` | `main` `60bee00` |
| mnemonic-engrave | `/scratch/code/shibboleth/wt-engrave-s4-emu` | `composer-s4-emu` | `master` `a262e7d` |

Both trees are clean (`git status --porcelain` empty). Nothing was pushed,
nothing was flashed, no main checkout was written to, no sub-agent was
dispatched, no `.jsonl` was read.

`master` moved under me while I worked: the branch point is `a262e7d`
(`continuity: composer -- S4 plan R0 GREEN (5a5f3df), implementer dispatched`)
and `master` is now `03979e6`. The plan file is byte-identical at `5a5f3df`,
`a262e7d` and `03979e6` (`git diff --stat` empty), so the revision I executed
is the revision named.

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu log --oneline main..HEAD
05d903b emu: a THIRD test payload carrying the composer's own record classes (S4 Task 1)

$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu log --oneline master..HEAD
5040bb2 journeys: the composer's host half -- transcript_composer.sh (S4 Task 2)
```

## Environment

Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go/bin`, `CGO_ENABLED=0
GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`
for every build and test. Tools by path: `md` =
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (prints
`md 0.14.0`; it is the `1dc8d409` build and **does** carry `compose` — the
version string is shared with the installed 0.14.0 that does not, so the path
is the only thing that distinguishes them), `me` =
`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me` (0.8.0), `ms` =
`~/.cargo/bin/ms` (0.16.0), `mk` = `~/.cargo/bin/mk` (0.13.0).

---

# Task 1 — the third emulator payload — DONE (`05d903b`)

## Files

- `cmd/buildpayloadcomposer/main.go` — new. Emits §2's five records to stdout,
  deriving through the device's own path (`bip39.MnemonicSeed` →
  `hdkeychain.NewMaster` → `bip32.Derive` → `Neuter`), and **refuses to emit**
  if either `key:` xpub differs from the `ms`-derived pin in the file.
- `cmd/emu/sysw_composer_payload.bin` — new, 782 bytes.
- `cmd/emu/sysw_composer_payload.go` — new, `//go:build js`, `//go:embed`,
  `const syswComposerDigest`, `type syswComposerReader`, record inventory in
  the header as the cards blob's is.
- `cmd/emu/sysw_composer_payload_host_test.go` — new, untagged.
- `cmd/emu/sysw_composer_payload_live_test.go` — new, `//go:build oraclelive`.
- `cmd/emu/platform.go` — `case "composer"` in `SyswReader`.
- `cmd/emu/walk_js.go` — `shSysw` accepts `"composer"`, usage string updated.
- `.github/workflows/test.yml` — `./cmd/emu/` added to the `oraclelive`
  compile step. **See "Additions beyond the plan's letter" below.**

## Run / Expected

```
$ cd /scratch/code/shibboleth/wt-composer-s4-emu && CGO_ENABLED=0 go test -count=1 ./cmd/emu/ && GOOS=js GOARCH=wasm go vet ./cmd/emu/ && gofmt -l cmd/
ok  	seedhammer.com/cmd/emu	1.550s
CHAIN EXIT=0
```

`vet` exit 0, `gofmt -l cmd/` printed nothing. Expected met.

The confinement test passes UNCHANGED — it discovered the new embed itself, as
designed:

```
$ go test -count=1 -v -run 'Composer|Confine' ./cmd/emu/
    embed_confinement_test.go:229: confined 16 embed token(s) across 694 scanned files
--- PASS: TestEveryEmbeddedPayloadIsStructurallyConfined (0.23s)
--- PASS: TestSyswComposerPayloadMatchesItsDigest (0.00s)
    sysw_composer_payload_host_test.go:131: 5 records: 2 key, 1 hash, 1 now, 1 mnemonic
--- PASS: TestSyswComposerPayloadCarriesTheComposerClasses (0.00s)
```

Digest, from the plan's own pipeline:

```
$ go run ./cmd/buildpayloadcomposer | me sysw pack --no-passphrase --out cmd/emu/sysw_composer_payload.bin
digest:   dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b
```

**= the plan's `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b`.** `me sysw show` gives
`sealed: false`, `pub_len: 730`, records 0–1 "cosigner key (key:)", 2 "sha256
hashlock (hash:)", 3 "pack time (now:) — 1788220800 (seconds), height 905000";
record 4 (the mnemonic) is not listed by `show` and is named only in the
sealing warning, exactly as §2 says. The stdin-packed blob is byte-identical
to the `--in` form (`cmp` clean).

The generator's stdout diffs empty against a §2 records file built
independently on the host from `ms derive` + `xxd`.

## Deviations from the plan's Expected — Task 1

**None in the values.** Two things the plan did not anticipate:

1. **`me sysw show` prints its `digest:` line on STDERR** and everything else on
   stdout (measured; `me` 0.8.0). The first version of the `oraclelive` audit
   used `exec.Command(...).Output()`, found no digest, and failed with
   "comparing nothing" — the honest failure, but the stream split is the fact.
   Both the audit and `transcript_composer.sh` now read both streams and say
   why. This also means any future step capturing that digest with a
   stdout-only redirect silently writes an empty file.

2. **The plan's third host-test assertion needs a binary CI does not have.**
   "…`me sysw show`'s digest line equals it" cannot live in the untagged test:
   `go test ./...` runs on a runner with no `me`, and a `t.Skip` there is the
   skipped-gate failure this tree forbids. I put it behind `//go:build
   oraclelive` with ABSENCE FATAL, which is this tree's established split
   (`sysw/vendored_vectors_live_test.go`, `gui/chain_fixture_live_test.go`), and
   ran it:

   ```
   $ ME=.../target/debug/me go test -count=1 -v -tags oraclelive -run TestSyswComposerPayloadDigestAgreesWithMe ./cmd/emu/
       sysw_composer_payload_live_test.go:55: auditing against /scratch/code/shibboleth/mnemonic-engrave/target/debug/me (me 0.8.0)
   --- PASS: TestSyswComposerPayloadDigestAgreesWithMe (0.01s)
   ```

## Additions beyond the plan's letter — Task 1 (declare, do not hide)

- **`.github/workflows/test.yml`:** `./cmd/emu/` appended to
  `CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`.
  Without it the new tagged file is compiled by no CI step and rots — which is
  the stated purpose of that step. Verified it compiles clean
  (`ok seedhammer.com/cmd/emu 0.006s [no tests to run]`). Revert the one token
  if the controller would rather own this in Task 6.
- **The generator also checks master B's account xpub** (`xpub6FQya…F8mX`)
  though it does not emit it. The plan asks for "either" of the two `key:`
  records; B@0 is the origin the whole keyed oracle (Policy-ID `4dd749a8…`,
  four addresses) is minted against, and a divergence there would surface as a
  wrong ADDRESS rather than as a wrong key. One extra comparison, no behaviour
  change.

---

# Task 2 — `design/journeys/transcript_composer.sh` — DONE (`5040bb2`)

```
$ FORK=/scratch/code/shibboleth/wt-composer-s4-emu ./transcript_composer.sh > transcript_composer.txt; echo $?
0
$ grep -c '^GATE PASS' transcript_composer.txt   ->  27
$ grep -c '^GATE FAIL' transcript_composer.txt   ->   0
$ grep -c '\[exit [^0]'  transcript_composer.txt ->   0
```

Every artifact the plan names is written into `out/composer/`: `records.txt`,
`payload.bin`, `payload.digest.txt`, `compose.json`, `keyed.template`,
`keyed.md1.txt`, `keyed.id.txt`, `keyed.receive.txt`, `keyed.change.txt`,
`keyed-template.md1.txt`, `cards/slot{0,1,2}.mk1.txt`, `keyless-tr.template`,
`keyless-tr.md1.txt`, `keyless-tr.id.txt` (plus `records-from-fork.txt`, the
generator's stdout, kept so the diff gate has both sides on disk).

## Every §2 oracle, reproduced

| value | plan §2 | this run |
| --- | --- | --- |
| A@0 xpub | `xpub6DkFA…KFrf` | same |
| A@1 xpub | `xpub6Dzhy…d6Vk` | same |
| B@0 xpub | `xpub6FQya…F8mX` | same |
| payload digest | `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b` | same |
| `pub_len` | 730 | 730 |
| keyed `template_with_origins` @2 | `48'/0'/2'/2'` (md lowest-free) | same |
| keyed md1 chunks | 7 | 7 |
| keyed md1 chunk 1 | `md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv` | same |
| `wallet-descriptor-template-id` | `531ab9e1777f018ae53694387dd0d128` | same |
| `wallet-policy-id` | `4dd749a8372af515a61d7104faf944ef` | same |
| `wallet-policy-id-fingerprint` | `0x4dd749a8` | same |
| `md1-encoding-id` (keyed) | `fb28698ee8bdbc18c6ee36598f2124fe` | same |
| Receive 0 | `bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l` | same |
| Receive 1 | `bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a` | same |
| Change 0 | `bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru` | same |
| Change 1 | `bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9` | same |
| form-B template | 2 chunks, `chunk-set-id 0x34c51` | same |
| form-B chunk 1 | `md1fxnz3qs9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2shlte30qvuhvrq` | same |
| form-B chunk 2 | `md1fxnz3qsw46h2at4w46h2at4w46h2at4w46h2msqqqv4qp0npeutks2tnchdq4ts6yd7yq5swf47peq533w` | same |
| card chunk counts | @0 = 2, @1 = 3, @2 = 2 | same |
| keyless template | `tr(50929b74…3ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))` | same |
| keyless md1 | 1 chunk, `chunk-set-id 0xb0884`, **56 chars**, `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` | same |
| keyless Template-ID | `e0863d3ccac31a64d3b5e14b85ccd6c0` | same |
| keyless origins | `@0: m/48'/0'/0'/3'`, `@1: …1'/3'`, `@2: …2'/3'` | same |

**Not one value differed. Nothing was re-pinned.**

## The chunked/unchunked substitution, demonstrated rather than asserted

The transcript now shows both forms and runs `md verify` against each:

```
chunked    md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3   (56)  -> OK, exit 0
unchunked  md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc            (47)  -> OK, exit 0
```

Same template, same `wallet-descriptor-template-id`, same `md1-encoding-id`
`b0884601fa89b3d294c599d8a6bb1602`. The plan's C-1 is now a fact the record
carries, not a warning it repeats. **The 47-character string is the exact
mutation a Task-3 driver must fail on**, and it is on disk for whoever writes
the mutation test.

## Additions beyond the plan's letter — Task 2

- **27 gates, and a failing one exits the script non-zero.** The sibling
  transcripts exit with their last command's status. A transcript that records
  a wrong value and reports success is worse than none.
- **Two cross-implementation byte equalities**, both gated:
  `records.txt` == `go run ./cmd/buildpayloadcomposer` stdout, and
  `payload.bin` == `cmd/emu/sysw_composer_payload.bin`. The second is what makes
  the emulator's digest screen comparable to `payload.digest.txt` at all.
- **`FORK=` override** (default `$C/seedhammer`, unchanged), for the same
  reason the brief gives for `capture_composer.py`: run from a worktree, the
  default resolves to the main checkout, which is the wrong tree while the fork
  change is on a branch. This run used
  `FORK=/scratch/code/shibboleth/wt-composer-s4-emu` (recorded in the
  transcript as `fork checkout: … 05d903b`). `MD=`, `MK=`, `MS=`, `ME=`, `GO=`
  are overridable the same way.
- **`runcapboth()`** beside `runcap()`, for the stderr digest measured in Task 1.

---

# Task 3 — STOPPED. `shots_composer.js` and `capture_composer.py` were not written

## What blocks it

`gui/composer_paged.go`'s `composerPickScreen` moves its cursor **only** on
`ButtonFilter(Up)` / `ButtonFilter(Down)`. Its rows are drawn by
`composerPageLines` as bare `widget.Labelw` ops — **no `Clickable`, no
`op.Input` hit area** — and its nav row is `Button1` / `Button2` / `Button3`
only, with no scroll arrows. The SeedHammer II has no directional buttons: its
only production input is the ft6x36 panel emitting `PointerEvent`s
(`cmd/controller/platform_sh2.go`; stated in `gui/unlock_flow.go:201` and in
`design/FOLLOWUPS.md`'s `seedhammer-warning-scroll-untouchable`, whose survey
of this class checked `ChoiceScreen`, `SeedScreen` and both keyboards and found
`Warning` the only button-only path **at that time** — `composerPickScreen` did
not exist yet).

**Four production call sites**, all of them on the composer's critical path:

| site | screen | consequence on the machine |
| --- | --- | --- |
| `gui/composer_shape.go:139` | `composerCountPick` — `Path N: how many keys?` and `how many must sign?` | only the first value of each page is selectable: `1`, then `6`, then … |
| `gui/composer_shape.go:380` | the `Spend paths` list | once one path exists, row 0 is `Path 1: …`, so **`Add a spend path`, `Change the script` and `Done` are all unreachable** |
| `gui/composer_hash.go:149` | `Which hash?` | only the first row |
| `gui/composer_seat.go:127` | `Seat keys` | only the first row |

Paging does move the cursor — `composerPickScreen` sets `sel = start` after a
`Button2` — so the reachable set is exactly **the first row of each page**. On
a list that fits one page, `start` wraps to 0 and the cursor never moves at all.

## The measurements

All on the emulator (`GOOS=js`, the shipped firmware GUI) at fork worktree
`05d903b`, driven with `shTap` — the same events the canvas emits, which is the
same event the panel emits.

**(a) The screens up to the first pick screen are exactly as the plan says.**

```
boot        "Asystemwidepayloadispresent.Loadit?LOADSKIPPayload"
digest      "PayloadDigestComparethisagainst`mesyswshow<file>`onthehost:dbe9e774e9a492310b62626c2b41cf4b"
warnings    "PayloadWarningsASECRETisstoredunencryptedinflash."
keep        "Keepthispayloadloaded?KEEPUNLOADPayload"
door        "Keysloaded:2,plus1seed.ScancardsBuildanewpolicyWalletPolicy"
script      "Whichscript?Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)Newpolicy"
startfrom   "Startfrom?Buildmyownpathsplain-multisigsimple-timelocked-inheritancekofn-recoverytiered-recoveryhashlock-gateddecaying-multisigNewpolicy"
paths       "Spendpathsslots:0/keysavailable:2AddaspendpathChangethescriptDone"
whatcanspend "Whatcanspendonthispath?KeysAhash,nokeysPath1"
howmanykeys "KeysPath1:howmanykeys?12345"
```

The device's digest screen prints `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b` —
**the Task-2 host oracle, matched across the air gap.** The door reads `Keys
loaded: 2, plus 1 seed.` with rows `Scan cards`, `Build a new policy` and no
`From payload`, and `Start from?` opens on `Build my own paths` — plan rows 2,
3, 4, 5, 6, 7 and 8 confirmed on the emulator, including the live line
`slots: 0 / keys available: 2` with no seed line. With the payload NOT loaded
the door reads `A payload is in flash but not loaded. Load it from the carousel
first.`, which is r0 I-10's prediction for the keyless arm's state.

**(b) The negative: 205 taps across the whole body of the `n` picker change
nothing.**

```
n picker                    "KeysPath1:howmanykeys?12345"
taps issued: 205 -- never left the picker
n picker after body sweep   "KeysPath1:howmanykeys?12345"
what the take produced      "ThresholdPath1:howmanymustsign?1"
```

Taps at x ∈ {24, 120, 240, 340, 400} for every y from 56 to 300 in steps of 6,
one at a time with a settle, asserting after each row that the screen is still
the picker (it always was — so no tap reached a nav button and the negative is
not "the screen changed underneath me"). The take then yielded **n = 1**: the
threshold picker offers a single row. Tapping the `2` row directly, before the
sweep, gave the same result.

**(c) The positive control: `ChoiceScreen` row taps DO land, at the same
geometry.**

Row 2 of 4 on `Which script?` is `Nested (sh-wsh)`, and the legacy-wrapper rule
in `composerKeysEdit` starts that wrapper's key picker at 2 rather than 1. So a
correct row-2 tap is observable downstream:

```
path list (sh-wsh)                                  "Spendpathsslots:0AddaspendpathChangethescriptDone"
n picker under a LEGACY wrapper -- must start at 2  "KeysPath1:howmanykeys?23456"
```

`2 3 4 5 6`, not `1 2 3 4 5`. **The tap mechanism and the `rowY(i,n) = 160 −
(n−1)·12 + i·24` geometry are correct**; the pick-screen negative is not a
coordinate error.

**(d) `Done` is unreachable once a path exists.**

```
path list                          "Spendpathsslots:1/keysavailable:2Path1:1keyAddaspendpathChangethescriptDone"
after PAGE x1..x4                  (identical each time)
what a take after 4 pages produced "Path1:1keyKeysTimelockHashlockRemovepathPath1"
```

Four `Button2` presses moved nothing (the four rows fit one page, so `start`
wraps to 0 and `sel` is clamped back to 0), and the take opened Path 1's editor
— row 0. There is no touch path from here to `Done`.

**(e) Paging lands the cursor on a page boundary, confirming the exact rule.**

```
n picker page 0                    "KeysPath1:howmanykeys?12345"
n picker page 1                    "KeysPath1:howmanykeys?6789"
what the take on page 1 produced   "ThresholdPath1:howmanymustsign?12345"
```

`Button2` → the take yields **n = 6**, the first row of page 1. So the
selectable set is {1, 6, …}: page-boundary rows only.

**(f) Why nothing caught this.** Every composer gate drives these screens with
synthetic button events — `gui/composer_flow_test.go:239`:
`click(&ctx.Router, Down, Down) // 1 -> 3`. `Down` has no non-test source on the
SH2 (the only one in the tree is `cmd/controller/debug_sh2.go`, a UART debug
harness). The suite is green against an input path the machine does not have.
That is why the plan's §4 coverage line was right to say the emulator-only rows
were unverified, and it is the "a control can test the wrong layer" shape.

## Consequence for the plan

- **Keyed arm, row 9** — `Path 1: how many keys?` **2** → unreachable.
- **Keyed arm, row 13** — `Done` → unreachable once path 1 exists.
- **Keyless arm, row 3** — `Add a spend path → Keys → 3 → 2` → unreachable.
- Rows 10 (`1 key`), 12 (hash row 0) and 14/14a (seating, which happens to
  resolve to row 0 at every slot once `used` filtering is applied) would have
  been reachable. The blockage is not universal, but it is on both arms.

A driver could only get past it by (i) adding a button-injection primitive to
`cmd/emu` — which `cmd/emu/walk_js.go`'s own header forbids in as many words
("Anything that let a walk skip a step would make the walk prove less than the
operator's own hands do, which is the opposite of the point"), and which would
paper over the defect rather than record it; or (ii) changing shipped GUI
behaviour. Both are decisions above an implementer, so I stopped.

## The fix shape, for whoever owns it — precedent already in this tree

S6b did exactly this for `Warning`: `gui/gui.go:437` declares `arrowUp` /
`arrowDown` `Clickable`s bound to `Up` / `Down`, drawn at `gui/gui.go:569` with
`scrollArrow(...)` and `assets.ArrowUp` / `assets.ArrowDown`; `Clickable.Next`
routes `PointerFilter(c)` alongside its button filters and carries
press-and-hold auto-repeat, so **no gesture handling is required**. Backed by
`gui/scroll_arrows_test.go`. `composerPickScreen` needs the same two
`Clickable`s and the same two drawn targets, and its `navs` slice already has
the shape to carry them. This is wiring, not new input work — the same
conclusion `design/FOLLOWUPS.md` reached for `Warning`.

**Suggested owning phase: the Task 4 device-walk fold (`composer-s4b`)**, which
is already the branch for "fixes found on the walk, each with a regression test
that fails under its named mutation". A natural mutation for that test: remove
the arrow `Clickable`s and assert the cursor cannot leave row 0 under
pointer-only input.

## What Task 3 would still need after that fix

Nothing else discovered. The itineraries' non-pick rows all matched on the
emulator as far as I drove them, and the host oracle is on disk and gated. The
`shots_composer.js` / `capture_composer.py` pair, the `--prove-it-can-fail`
control and the byte comparison against `keyless-tr.md1.txt` remain unwritten.

## Negative control

**Not run — there is no `capture_composer.py` to run it with.** The mutation it
must catch is available and measured: substituting
`md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc` (47 chars, `md verify` exit
0) for `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` (56 chars).

---

# The three shipped drivers — all exit 0 against the Task 1 fork worktree

Run UNMODIFIED, against `/scratch/code/shibboleth/wt-composer-s4-emu` (not the
main checkout), by staging a tree at `/scratch/code/shibboleth/.tmp/s4run/`
whose `seedhammer` is a symlink to the worktree, so each driver's own relative
`../../../seedhammer/cmd/emu` resolves there. `out/` artifacts copied read-only
from the main checkout; nothing in either main checkout was written.

```
===== capture_walletpolicy.py
chunks presented: 8, cards gathered: 1 / consent pages read: 3
MATCHED  wallet id 4e67c6fd8220c32e51c9ad9947e24141 + 4 addresses
===== capture_walletpolicy.py exit=0
===== capture_seating.py
template chunks: 1, cards: 1, key cards: 2 / consent pages read: 3
MATCHED  wallet id c8fe87cd5fb7351db12479a2bab8f8ad + 4 addresses
===== capture_seating.py exit=0
===== capture_tr_pathological.py
chunks presented: 24, cards gathered: 1 / consent pages read: 4
MATCHED  wallet id 590f3abcaad2aca5a3f526917f5bb57a + 4 addresses
===== capture_tr_pathological.py exit=0
```

No regression from the third payload, the `case "composer"` arm or the
`shSysw` widening.

---

# Everything I decided, could not do, or stopped on

1. **STOPPED Task 3** — the pick-screen input gap above. Both remedies are
   design decisions; §5 of the brief reserves them.
2. **Added `./cmd/emu/` to CI's `oraclelive` compile step** so the new tagged
   test cannot rot. One token; revert if Task 6 would rather own it.
3. **Put the `me sysw show` assertion behind `oraclelive`, absence fatal**,
   rather than in the untagged test where CI has no `me` and the only
   alternative would have been a skip.
4. **Checked master B's account xpub in the generator** though it is not
   emitted — the keyed oracle is minted against it.
5. **Gave `transcript_composer.sh` a `FORK=` override and 27 exit-code-bearing
   gates**; the plan named one gate (the records diff) and no exit contract.
6. **`me sysw show` prints `digest:` on stderr** — measured, not a guess;
   it changes how any caller must capture it.
7. **Observation, not blocking, for the record:** with the composer payload
   loaded the door's lead is `Keys loaded: 2, plus 1 seed.`; with it merely
   present in flash it is `A payload is in flash but not loaded. Load it from
   the carousel first.` Both were photographed on the emulator; the second is
   what Task 4's device walk should expect to read first (r0 I-10).

---

# Task 3 — DONE

**Resumed after W-2 was verified (0C/0I) and merged into fork `main` as
`3cc71d9bbe0f211afe2a8e3facdf57f4a3a66d1b`.** Executed against the plan at
mnemonic-engrave `fdce82f`, whose Task 3 preamble now requires rows to be
selected by tapping at the frame's own geometry and never by injected
`Up`/`Down`. Brief: `design/agent-briefs/composer-S4-task3-resume-brief.md`.

## The merge

```
$ git -C /scratch/code/shibboleth/seedhammer rev-parse main
3cc71d9bbe0f211afe2a8e3facdf57f4a3a66d1b
$ git merge --no-ff main -F <msg>
Merge made by the 'recursive' strategy.
 gui/composer_measure_test.go | 2 +-  gui/composer_paged.go | 109 +++++++---
 gui/composer_paged_test.go   | 4 +-  gui/composer_pick_touch_test.go | 185 +++++++++
 gui/composer_stub_test.go    | 4 +-
 5 files changed, 288 insertions(+), 16 deletions(-)
```

**No conflict**, as predicted — Task 1 touched `cmd/` only, W-2 touched `gui/`
only. `git status --porcelain` empty after.

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu log --oneline main..HEAD
86cec95 emu: the composer journey's walk -- shots_composer.js, and shTargets to tap rows by (S4 Task 3)
a79a454 Merge main into composer-s4-emu: W-2, the pick screen's rows are touch targets (S4 Task 3 needs it)
05d903b emu: a THIRD test payload carrying the composer's own record classes (S4 Task 1)

$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu log --oneline master..HEAD
c6adac2 journeys: the composer's device half -- capture_composer.py (S4 Task 3)
5040bb2 journeys: the composer's host half -- transcript_composer.sh (S4 Task 2)
```

Both trees clean. Nothing pushed, nothing flashed, no sub-agent, no `.jsonl`.

## What was written

- `cmd/emu/shots_composer.js` (762 lines) — both arms, every itinerary row an
  assertion, the engrave loop with the `Bundle engraved` handler ending on the
  door, paged screens read until the first page recurs.
- `design/journeys/capture_composer.py` (317 lines) — `--arm keyed|keyless|both`
  (a fresh page per leg), `--prove-it-can-fail`, `EMU`/`--emu` override,
  shot-size checking.
- `cmd/emu/screen.go` + `screen_js.go` — `window.shTargets()`, below.
- `gui/composer_digitpad_geometry_test.go` — the digit-pad pin, below.

### `shTargets()` — how a row is tapped, and why it is not a shortcut

`walk_build_policy.js`'s `rowY = 160 − (n−1)·12 + i·24` is right for
`ChoiceScreen` and **wrong for the composer's paged screens**, whose rows
advance by each line's own measured height under a lead that wraps; re-deriving
it in JavaScript means reimplementing the text layout without the font metrics.
The brief permits "measure the row bands … **or read them from the layout**", so
`screenRecorder.Frame` — which already builds an `op.Drawer` for `ExtractText` —
now also probes it with `op.Drawer.Hit` down the centre line and keeps the
rectangles, and `shTargets()` hands them to the page.

It is a **reading** primitive, beside `shScreen`: it injects no event, reaches no
flow, and lets a walk do nothing a hand could not — it says where the targets
are, which is what the operator's eyes do. That is load-bearing rather than
rhetorical: **on a pre-W-2 build it returns zero rows for a composer pick
screen**, so a driver written against it fails with "no tappable rows" instead of
quietly injecting `Up`/`Down` and reporting a walk the machine cannot perform.
Measured on the door: two targets for two rows, and the navigation column
correctly absent (it sits off the centre line).

### The digit pad, pinned and mutation-tested

The 12960-block lock is typed at coordinates, as `walk_trace_b.js` types the
BIP-39 keyboard. `gui/composer_digitpad_geometry_test.go` reads
`DIGIT_KEY_PITCH`/`DIGIT_KEY_ROWS` out of `shots_composer.js` and types `12960`
with them through `op.Drawer.Hit`. The last row is **not** like the others —
`NewKeyboard` appends its own backspace, so `"0"` is laid out as two centred
keys starting one pitch right — and the naive mutation is caught:

```
  { digits: "0", x0: 206, y: 290 }        // "same as the rows above"
  digit 5 (0): the walk taps (206,290) and there is no touch target there at all
```

## Every value the emulator printed, beside the plan's

| what | plan / host | emulator |
| --- | --- | --- |
| payload digest (Payload Digest screen) | `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b` | same |
| door lead, payload loaded | `Keys loaded: 2, plus 1 seed.` | same |
| door lead, reader off | `No keys loaded. This builds a key-less template.` | same |
| Template-ID | `531ab9e1777f018ae53694387dd0d128` | same |
| `mk1 stub (template)` | `531ab9e1` | same |
| Policy-ID | `4dd749a8372af515a61d7104faf944ef` | same |
| `mk1 stub (policy)` | `4dd749a8` | same |
| Receive 0 | `bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l` | same |
| Receive 1 | `bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a` | same |
| Change 0 | `bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru` | same |
| Change 1 | `bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9` | same |
| key-less Template-ID | `e0863d3ccac31a64d3b5e14b85ccd6c0` / stub `e0863d3c` | same |
| slot @2 unseated origin | `m/48h/0h/2h/2h` | same |
| slot @2 seated origin | `@2: b8688df1 m/48'/0'/0'/2'` | same |

**Census screens, verbatim from the device:**

```
form A   PlatesToCutThisengraves2plates.md1policy:2plates(thewalletpolicy,withitskeys)…
form B   PlatesToCutThisengraves4plates.md1template:1plate(key-lesswalletpolicy)
         mk1key@0:1plate(m/48'/0'/0'/2')mk1key@1:1plate(m/48'/0'/1'/2')
         mk1key@2:1plate(m/48'/0'/0'/2')…
keyless  PlatesToCutThisengraves1plate.md1template:1plate(key-lesswalletpolicy)…
```

Rows @0 and @2 differ only by the `@i` (A's account 0' and B's account 0'), as
r0 M-5 predicted.

**The engraved strings, byte for byte against the host.** Form A — 2 plates, 7
chunks = `keyed.md1.txt`:

```
md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv
md1flv5xrqw46h2at4w46h2at4w46h2at4w4hqqqqe2qzlxrnchdq5h83w6p2hp5gmug4u83waqg8k6ntsjqqddg
md1flv5xrqscl9pvz58pmltjs9tjrg0g2z0agd4urfpzanhaq3lcdlz64mrqgdrha0m7umapumg67jfj6m3fvh7y
md1flv5xrqej075dhzfzvynh66n94j5lcxlmx9ayav9mj0jjejcxy50llpx82qfmryv7l68w6hqqhypxt9s9j68n
md1flv5xrpzragnj3g5qrl85zeape8wq0vdczfyy55tqsd5576trsa3p40nfpd7hsyjyf7vlx6saxquqfv5er72p
md1flv5xrp0k2j6ckr4wf0m36nn9pm0wkz5duhr4fq2pwsjch4zfmsclyyxap2w2ua7583pn5tsnkrk4cfxj37uw
md1flv5xrpj7qeewyp4dfykwfkgg6fxyxetdcmythf4hsqzd3v879jprztejzs7ru967l2aj4n0rcs
```

Form B — 4 plates, 9 strings = `keyed-template.md1.txt` (2) + `cards/slot0` (2) +
`slot1` (3) + `slot2` (2), the first two being
`md1fxnz3qs9qjtvyyy…shlte30qvuhvrq` and `md1fxnz3qsw46h2at…f47peq533w`, then the
three `mk1qp0tx8p…`, `mk1qpcqd3z…` and `mk1qpvyhfp…` sets. Key-less — 1 plate:

```
md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3      (56 chars, CHUNKED)
```

**Paging and variants, measured:** stub screens 2 then 3 pages, mapping review 2,
consent 4, key-less stub 2 and consent 2. `Choose engraving` on a packed plate
offers `TEXT ONLY` alone (`Card 1 of 4 | Plate 1 of 1`); on the key-less plate it
offers `TEXT + QR`, `TEXT ONLY`, `QR ONLY` with `TEXT + QR` first — the plan's
row 6, and the device has no camera to read a QR-only plate back.

## Gates

```
$ python3 capture_composer.py --arm both
0        three legs (keyed-A 60s, keyed-B 71s, keyless 28s), 50 shots,
         "all legs matched the host."

$ python3 capture_composer.py --arm keyed --prove-it-can-fail
0        NEGATIVE CONTROL PASSED: the walk refused the corrupted address.
         DRIVER FAILED on leg keyed-A: the device's proof does not match the host's:
           address bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4q
```

**The plan's named mutation** — the unchunked keyless string substituted into a
copy of `out/composer/keyless-tr.md1.txt`, then restored:

```
$ printf 'md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc\n' > out/composer/keyless-tr.md1.txt
$ python3 capture_composer.py --arm keyless
DRIVER FAILED on leg keyless: the key-less template plate: string 1 of 1 does not
match the host's BYTE FOR BYTE.
  device: "md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3" (56 chars)
  host:   "md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc" (47 chars)
Every md verb accepts the chunked and the unchunked form of a short template
identically, so this comparison is the only one that can tell them apart.
MUTATION EXIT=1
```

| gate | result |
| --- | --- |
| `capture_walletpolicy.py` | exit 0, wallet id `4e67c6fd…` + 4 addresses |
| `capture_seating.py` | exit 0, wallet id `c8fe87cd…` + 4 addresses |
| `capture_tr_pathological.py` | exit 0, wallet id `590f3abc…` + 4 addresses |
| `gofmt -l cmd/` | clean |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | exit 0 |
| `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` | `ok  seedhammer.com/cmd/emu  1.415s` |
| `gui-shard-test.sh ./gui/ 24` | `all 1188 tests ran across 24 shards`, 60s |

`needle_test.go` pins no count this driver changes: `shots_composer.js` declares
no `NEEDLE_*` constant, and that test binds only declared needles. The three
shipped drivers were run **unmodified** against the fork worktree by staging a
tree whose `seedhammer` symlinks to it, so each driver's own relative
`../../../seedhammer/cmd/emu` resolves there; neither main checkout was written.

## Deviations from the plan's Expected, verbatim

**1. The lock's echo is a SCREEN, not the digit pad's line.** Plan row 11: *"→
digits 12960 | -- | the echo screen is ONE line, `12960 blocks (about 90.0
days)` … (shot)"*. The plan is right and my first draft was not: the pad's echo
is live validation (`composerBlocksBandEcho`), and §6b's echo is a separate
`composerReadScreen` drawn after `composerLockAccept`. The first run hung on the
pad. Fixed in the driver; the shot is now `c04-lock-echo-p0.png` on the echo
screen, and its one-line shape is asserted by paging it (1 page) plus the
absence of `This device cannot tell the time.` — `composerLockBoundLine` returns
`""` for `LockOlderBlocks`, which is r0 I-7 asserted against the production copy
rather than a paraphrase.

**2. A frame settles on the NEXT event.** After a flow transition the standing
frame can be partial — measured at the lock accept, where the pad stopped
drawing and the Path menu did not appear until another pointer event arrived.
`waitFor` now nudges at `(5,5)`, inside the 44 px title band, above every
content box and left of the navigation column. `walk_s3_nested.js` nudges the
same way before reading the engrave screen; this is that behaviour, not a new
one.

**3. FIRMWARE SIZE — a finding, as §4 asks.** The recipe reads **1,580,580 B
flash / 62,800 B RAM** against the plan's pinned `1,579,940 B / 62,800 B`.
**+640 B flash, and it is W-2 entirely.** Measured rather than argued: building
`composer-s4b` (W-2 alone on `60bee002`, no Task 3 change present) gives the
same `1,580,580 / 62,800` byte for byte. Task 3 adds **zero** — `cmd/emu` and
`cmd/buildpayloadcomposer` are outside `cmd/controller`, exactly as §4 says. The
pin is simply the pre-W-2 number and wants updating to `1,580,580 / 62,800`;
RAM is unchanged.

## What I decided, and what I could not do

1. **`shTargets()` is new emulator surface** the plan did not name. The brief
   sanctioned it ("or read them from the layout"); it is a reader beside
   `shScreen`, costs nothing (the Drawer already existed for `ExtractText`), and
   is what makes "a row the driver cannot reach by tapping is a finding" true by
   construction rather than by promise. It deliberately probes the centre line
   only, so the navigation column is not mixed into the row list.
2. **The keyed arm is two legs, not one run.** Plan row 19b says "(second run of
   rows 1-18)", so form A and form B each get a fresh page and walk the whole
   itinerary. `--arm both` is therefore three page loads.
3. **Watch-only on both keyed forms.** `Full (seed + keys)` adds a BEARER plate
   of master B's seed; the plan reserves that for the operator, and an automated
   run must not cut one. The mode picker's presence is asserted (it is drawn
   because a seed-seated slot exists, r0 I-1).
4. **The stubs are derived in the capture, not pinned** — the first four bytes
   of ids it already reads from the host. A separately written stub would be a
   second source of truth for a value that has one.
5. **Not done, and not mine:** Task 4 (the live device walk, which the plan now
   gates on the W-2 fix being flashed), Task 5, Task 6, and merging or pushing
   either branch.
