# S5.0 implementation log — the oracle-only work block (2026-08-15)

**Status: BOTH STEPS COMPLETE AND GREEN. Nothing is blocked.**

Spec: `design/agent-reports/s5-prerequisites-ruling-2026-08-15.md`.
Worktree: `/scratch/code/shibboleth/seedhammer-s5`, branch `s5-oracle-block`, off `main` @ `80d0c5d`.

| commit | repo | what |
| --- | --- | --- |
| `3c879e7` | seedhammer-s5 | S5.0a — the `ms` pin bump and the one-commit S0 re-anchor |
| `5edb162` | seedhammer-s5 | S5.0b — the built-policy `ExpectKind` |
| `569fd82` | mnemonic-engrave | the one-line plan fold (S5's file-touch matrix) |

`/scratch/code/shibboleth/seedhammer` (main) was never touched: still `80d0c5d`, porcelain clean.
`mnemonic-secret` left exactly as found — HEAD `6fdfd36`, the same three untracked files, worktree list restored.

---

## STEP A — the `ms` pin bump and the S0 re-anchor

### Before / after

| field | before | after |
| --- | --- | --- |
| commit | `ddfa497090bd945dce9b53453c7eeebf3e8d623d` | `d49d5c099bab89a1738f0d0c3df9306b354d62c3` |
| sha256 | `e63d9cb524c839e94e24bd283620db489d47363a760f6f84541e882ba783cec0` | `9727689c5ee049c0135a386d7db1c0b58d45f239aced0ade7f6f288c5b1202e5` |
| version | `ms 0.15.0` | `ms 0.16.0` |
| `checkout_clean_when_recorded` | `true` | `false` |

`d49d5c09` is the target of the annotated tag `ms-cli-v0.16.0`, one CI-only commit behind `master` (`6fdfd36`). The tag is the release identity; the tag is pinned.

### How the binary was built, and what the honesty flag says

`cargo install --path crates/ms-cli --locked --force` from a dedicated `git worktree` detached at `d49d5c09`, whose own `git status --porcelain` was **empty**. The reference checkout `/scratch/code/shibboleth/mnemonic-secret` was **not** cleared — its three untracked files are not mine — and it sits at `master`, not at the tag.

**`checkout_clean_when_recorded` is recorded `false`, conservatively.** JUDGED, not transcribed: the tree the bytes actually came from was clean and at the pinned commit, so `false` *understates*. I chose the understating value because the flag's documented meaning is about the recorded checkout, and `pins.json`'s own `repo` field and its recording recipe point at the reference path. The full truth is in the `_comment` block, not smuggled into the flag.

**A pre-existing contradiction was fixed in passing:** the comment block already said mnemonic-secret was dirty and "is marked", while the `ms` flag read `true`. Nothing in the tree reads the flag mechanically (`grep` for `CheckoutCleanWhenRecorded` → one struct field, one comment, three JSON values), so this is a documentation correction, not a behaviour change.

**Extra measurement, volunteered because the pin binds a hash:** the recorded sha256 is build-directory-independent. Building the same commit in a second worktree at a different path produced a **byte-identical** binary (`9727689c…` both times), so a maintainer rebuilding from the canonical checkout gets this hash. `strings | grep -c` for the scratch path in the binary: `0`.

### The RED before the re-anchor — proof `-expect-only` would not have sufficed

After editing `pins.json` only, `go test ./oracle/` → **exit 1**, both files red, exactly as the ruling predicted:

```
--- FAIL: TestVendoredExpectationsWereDerivedFromThePinnedToolchain
    S0-trace-a.record.json: ... oracle ms was derived at commit ddfa4970..., pins.json now says d49d5c09...
--- FAIL: TestEveryGateRecordOnDiskVerifies
    gaterecords/S0-trace-a.record.json: ... oracle ms recorded at ddfa4970..., pins.json now says d49d5c09...
```

The first is the expectation's provenance; the second is `VerifyRecord` on the **record**. `-expect-only` rewrites only the first. So the rebuild was `gaterecord -force` over the saved walk, per F-177's explicit sanction.

### Re-anchor command and result

```
go run ./cmd/gaterecord -stage S0 \
  -walk oracle/gaterecords/S0-trace-a.walk.json \
  -inputs oracle/gaterecords/S0-trace-a.inputs.json \
  -base S0-trace-a -force
```
**TRUE exit 0**, stderr: `6 artifact(s) derived live by [ms mk] and matched the walk's census`.
No emulator walk was run and none was needed.

### Proof the census and digests did NOT change

Machine-compared with `python3 json`, not by eye:

| compared | result |
| --- | --- |
| walk file bytes (`cmp` against the pre-run copy) | **identical, exit 0**; `git status` does not see the file as modified |
| `record.walk` whole (census, plate_digests, sha256, pace, elapsed) | **EQUAL** |
| `record.walk.census` digest | `94b914e75f1d36fc` → `94b914e75f1d36fc` |
| `record.walk.plate_digests` digest | `ef073f974f4c153a` → `ef073f974f4c153a` |
| `record.inputs`, `record.payload` | **EQUAL** |
| `expect.artifacts` (all 6: string + origin + fingerprint) | `0ffdd7f938acf688` → `0ffdd7f938acf688`, **EQUAL** |
| `expect.derivation.args` | **EQUAL** |
| record top-level keys that changed | `oracles`, `recorded_at` — and nothing else |
| expect top-level keys that changed | `derivation`, `note` — and nothing else |
| counts | 6 artifacts / 6 census strings / 6 digests |

So **ms 0.16.0 reproduces the committed S0 expectation byte for byte**, which is the empirical version of the ruling's source-diff argument.

### The S2 md1 golden — confirmed untouched, not assumed

`git status --porcelain gui/testdata/s2_md1_golden.expect.json` → empty. Its derivation block names `[('md', 'md 0.13.0')]` and nothing else; 6 artifacts, all kind `md1`. `CheckProvenance` checks only the oracles an expectation names, so an `ms` bump cannot reach it.

### Step A gate — TRUE exit codes, unpiped, `GOCACHE` pinned

| check | exit |
| --- | --- |
| `go test ./oracle/ ./gui/ ./sysw/ -count=1` | **0** |
| `./scripts/oracle-live.sh` | **0** (`live checks: PASS`) — `TestLiveDerivationReproducesEveryCommittedExpectation` logged *"6 artifact(s) re-derived live and identical to the committed expectation (oracles invoked: [ms mk])"* |
| `go test ./... -count=1` | **0**, 51 ok / 0 FAIL (baseline match) |
| `gofmt -l .` | **0**, 0 files |
| `go vet ./...` (cold cache) | **1**, 40 findings, all `_test.go`, 0 outside — the clean baseline |

### JUDGED, flagged

The minted expectation's `note` now reads *"Derived live by cmd/gaterecord and compared against the walk's census before the record was written"* — the non-`-expect-only` wording. That is what the tool mints, and the walk it compared against was the **saved** one. I left it as minted rather than hand-editing it, because `CheckProvenance`'s own refusal says *"re-mint the expectation, do not edit it"*; the saved-walk fact lives in the commit message, per the ruling.

---

## STEP B — the built-policy `ExpectKind`

### The mode decision: TWO KINDS, `built-policy-full` and `built-policy-watch`

Not a `mode` field on `Expect`. Three reasons, all structural, written into the code:

1. **Every toolchain-free check dispatches on the kind alone.** `ArtifactKindsFor` and `CheckArtifactShape` answer *"which artifact kinds, in what order"* as a function of the kind. With a separate mode field they could only answer the **union**, and a full-mode expectation that had lost every ms1 would still be "consistent" — the exact vacuity this package exists to refuse.
2. **A missing kind REFUSES; a missing field DEFAULTS.** An inputs file that forgot `mode` yields `""` and takes whichever branch the zero value selects — fail-open, on the one input deciding whether a seed reaches steel.
3. **The set is closed and small.** Two consts and one switch arm, paid once.

### `ArtifactKindFor` → `ArtifactKindsFor` — the "richer answer"

Returns the kinds in the **flow's engrave order**, read off production code rather than off the ruling: `gui/multisig_engrave.go:11-35` appends `ms1` first in full mode, then `mk1`, then `md1`; the plan explicitly **DEFERRED** the "public plates first, secret last" reordering to its own R0.

| kind | artifact kinds |
| --- | --- |
| `cosigner-cards` | `mk1` |
| `built-policy-full` | `ms1`, `mk1`, `md1` |
| `built-policy-watch` | `mk1`, `md1` |

`CheckArtifactShape` holds a committed expectation to that order with no toolchain: consecutive non-empty runs in the declared sequence, nothing outside them. It catches the two failures a single-kind equality check could not express — a **missing class** (a "Full" backup with no seed in it) and a **reordered** one.

### The traps, each MEASURED against the pinned binaries

- **md's stdout header.** `md encode` prints `chunk-set-id: 0x30d86` **first** and `policy-id-fingerprint: 0x06215ac0` **last**, artifacts in between — and the header appears **whether or not** `--policy-id-fingerprint` is passed (checked both ways), so "skip the first line" would have been a wrong fix. `parseMdStdout` classifies every line by prefix, collects `md1`, consumes the two known lines deliberately, and **refuses** anything else — so a third header line in a later md version fails loudly rather than being adopted.
- **ms's separator.** `ms encode --help` says `[default: 5]`; the default output is `ms10e ntrsq qqqqq …`. `--group-size 0` **and** `--no-engraving-card` are passed, and the flag is **not trusted**: `refuseSeparated` rejects any space, hyphen or comma in a string the oracle just produced (the bech32 charset contains none).
- **Divergent origins are REFUSED, measured not assumed.** `md`'s `--path` is documented as flattening divergent mode to shared, and the bracketed per-key form the descriptor uses is rejected by the pinned md:
  `md: template parse error: internal: synthetic key [73c5da0a not found in key map`
  So no invocation this gate can make derives the right md1 for a divergent policy. Deriving a shared-origin one instead would compare a **different wallet's** bytes and fail with no explanation; the refusal names the reason. **This is a real limitation S5 will hit if Trace B is divergent-origin** — see "Open for S5" below.

### Other design decisions

- **The policy id is DERIVED, not stated.** `md encode --policy-id-fingerprint` computes it from the policy. `Expect.PolicyIDStub` becomes optional for a built policy and is **cross-checked** when present — a stated stub that disagrees with md is a refusal, never an override.
- **`Expect.HeldSlots`** added: required for a built policy, refused for `cosigner-cards`, strictly ascending and distinct so the mk1 engrave order is unambiguous. One mk1 per held slot; one ms1 per **distinct master** among them, keyed on the **words** (keying on the 4-byte fingerprint would let a collision silently drop a master out of a backup labelled "Full").
- **Fingerprint scoping per M-3, in both directions.** mk1/ms1 **must** carry one; md1 **must not** — a policy spans every slot and belongs to no master, and "not demanded" is the weaker rule that would let a hand-authored expectation decorate its chunks with a plausible value. The ≥2-distinct rule stays scoped to `cosigner-cards`: an ordinary single-self-slot build honestly has one master, and the old blanket rule would have failed a correct S5 record at its first artifact.

### SEEN TO WORK, not only seen to refuse

`TestBuiltPolicyDerivationMatchesTheS2Golden` (oraclelive) derives the built policy from the three published-vector masters and requires its md1 chunks to equal **S2's committed golden** byte for byte. That golden describes the same wallet and was minted through an entirely different code path. Result:

```
built-policy-full: 1 ms1 + 2 mk1 + 6 md1 = 9 artifact(s);
md1 byte-identical to S2's committed golden; oracles invoked [ms md mk]
built-policy-watch: 8 artifact(s), no ms1
```

The mode distinction is asserted in both directions: full has exactly one ms1, watch-only has none, and the 8 public artifacts are identical between them.

### Drift detection, and the comment that claimed it

`oracle/live_test.go`'s package comment said *"this file is the drift check … it asks whether the primary has changed under a pin that did not move"* and **nothing in it did**. Comment corrected in `live_test.go` **and** in `scripts/oracle-live.sh`, which carried the same false claim.

`TestPinsAreCurrentWithTheirPrimaries` now makes it true — **behind the `oraclelive` tag**, never untagged (CI has no sibling checkouts, so an untagged version would need a skip). Absence of a checkout is **fatal**, per the tier's rule.

Two things that would have made it wrong:

- **`--sort=v:refname` is required.** Git's default tag order is lexicographic, under which `ms-cli-v0.9.0` sorts after `v0.16.0`. Measured on mnemonic-secret: the unsorted list ends at `v0.9.0`.
- **"pin == newest tag" is the wrong question** and would have been red on arrival. The `mk` pin is two commits **ahead** of `mk-cli-v0.12.1`, reporting an untagged `mk 0.13.0`. The question asked is *whether a release exists that the pin does not name*: equal → current; tag-is-ancestor → logged NOTE; anything else → FAIL as a decision for a human.

Live output today:
```
md: pin is the newest release md-cli-v0.13.0 (5a0a4f41...)
NOTE: mk is pinned AHEAD of its newest release. pin a38a908..., newest tag mk-cli-v0.12.1 -> 4ac7ab49...
ms: pin is the newest release ms-cli-v0.16.0 (d49d5c09...)
```

**Also fixed, and it is the same defect class:** `oracle-live.sh`'s `-run` filter is an **allowlist**. A tagged test not named in it compiles, passes CI's `go vet -tags oraclelive`, and never executes anywhere. Both new tagged tests are added to it, and that property is now stated in the script.

### Mutation evidence — 15 mutants, 15 killed

Harness: patch the pristine source, prove the patch landed, run the named test, record the TRUE exit code, restore. A mutant that fails to apply or does not compile is reported **INVALID**, never counted as a kill.

| mutation | result | what it proves |
| --- | --- | --- |
| M1 md unknown stdout line adopted | KILLED (exit 1) | an unrecognised line is refused, not collected as an artifact |
| M2 `chunk-set-id` header no longer consumed | KILLED | the header cannot become an expected engraved string |
| M3 separator check disabled | KILLED | the `--group-size 0` trap is caught at the bytes, for md1 and ms1 |
| M4 shape check made order-blind | KILLED | the engrave order is enforced, not merely documented |
| M5 md1 fingerprint permitted | KILLED | direction 2 of the fingerprint scope |
| M6 mk1/ms1 fingerprint made optional | KILLED | direction 1 of the fingerprint scope |
| M7 cosigner ≥2-distinct rule dropped | KILLED | the rule still binds where it belongs |
| M8 divergent origins accepted | KILLED | a different wallet's md1 is never derived in place |
| M9 `held_slots` order/duplicates unchecked | KILLED | the expected census cannot be made ambiguous |
| M10 declared engrave order reversed | KILLED | the kind table is pinned to the device's tail |
| M11 `sortedmulti` → `multi` | KILLED | the generated template still equals the device's S2 template |
| M12 `cosigner-cards` accepts `held_slots` | KILLED | a field with no meaning is refused, not ignored |
| M13 unknown-kind refusal removed | KILLED **after a fix** | see below |
| M14 policy id made optional | KILLED | mk1 cards can never carry an empty stub |
| M15 several ms1s per master reduced to the first | KILLED | one master, one plate |

**One survivor, fixed rather than explained.** M13 initially **SURVIVED**: deleting the `KnownExpectKind` refusal at the top of `DeriveExpected` left `TestDeriveRefusesAnUnknownKind` green, because the new `default` arm of the dispatch switch caught the same input. Belt and braces is right, but a test that cannot tell which mechanism fired proves only that one of them exists — and would stay green while the first was removed and the second later "simplified" away. The test now asserts the refusal's own wording, and the mutant dies. Re-run: **15/15**.

### Order mutation, specifically

Two independent ones, both required and both killed:
- `CheckArtifactShape` — `TestCheckArtifactShapeRefusesTheWrongShape` includes `{"full, ms1 last"}` and `{"full, md1 before mk1"}`, which is precisely the deferred "public plates first, secret last" reordering shipping by accident. Killed by M4 and M10.
- `CompareCensus` — `TestCompareCensusCatchesAMultiKindReorder` swaps the secret plate and the descriptor in a multi-kind census and requires the refusal to name **both** plates.

### Step B gate — TRUE exit codes, unpiped, `GOCACHE` pinned

| check | exit | detail |
| --- | --- | --- |
| `go test ./oracle/ -count=1 -v` | **0** | 40 tests, 73 subtests, 0 FAIL, 0 SKIP |
| `go test ./... -count=1` | **0** | 51 ok / 0 FAIL |
| `go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/` | **0** | CI's tagged compile check |
| `./scripts/test-32bit.sh` | **0** | |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **0** | |
| `gofmt -l .` | **0** | 0 files |
| `go vet ./...` (cold cache) | **1** | 40 findings, all `_test.go`, 0 outside — the clean baseline |
| `./scripts/oracle-live.sh` | **0** | all six live checks PASS |
| mutation sweep | **0** | 15/15 killed |

Both `oracle-live.sh` runs (Step A and Step B) recorded: **exit 0** each.

---

## The plan fold, and a red gate I had to clear

Four `oracle/**` rows added to S5's file-touch matrix with the F-a rationale, plus a paragraph naming S5.0 and its ordering constraint.

**The cite gate came back RED on a pre-existing defect**, so it was fixed: line 672 cited `oracle/oracle_test.go:283` for "the real pins resolve the installed binaries", and that file has **272** lines. Not decay from an edit above it — `TestRealPinsResolveTheInstalledOracles` was **moved** to `oracle/live_test.go` during the S0b work, and its own comment says so. Pre-existing confirmed by running the gate against `git show HEAD:…`, which fails identically.

**JUDGED, and stated because a gate that hides its blind spot is worse than no gate.** The replacement names the file and the test rather than a line number, because the S5.0 commit adds ~24 lines above that function and any line number written today decays on merge. That form is **outside the cite gate's coverage**: `plan-cite-gate.sh` resolves `file:line` and `pkg.Symbol`, and its `pkg.Symbol` extractor matches an allowlist `(md|mk|codex32|seal|bip39|backup|engrave)` that does **not** include `oracle`. The trade is a claim that is always true and ungated, against one that was gated and false. Widening the gate's package allowlist is worth doing separately.

Cite gate, TRUE exit codes: **0** against the S5 worktree, **0** against the default `seedhammer` root. Before the commit: **1** unresolvable, both roots.

---

## WHAT S5.0 DOES NOT DO

**It does not discharge the end-to-end mint.** The built-policy kind's first real execution is S5's own gate record, minted by `cmd/gaterecord` from a real Trace B walk. Per the lens-closure doctrine **S5 cannot close before that mint has RUN**, and nothing in S5.0 pretends otherwise. What S5.0 establishes is that the instrument exists, that it refuses fifteen distinct ways, and that on the one policy an independent artifact already describes it derives byte-identical output.

No `gui/` file was edited. The whole diff is four files: `oracle/expect.go`, `oracle/expect_test.go`, `oracle/live_test.go`, `scripts/oracle-live.sh` (plus Step A's three JSON files in the preceding commit).

---

## Open for S5, found while doing this

1. **Divergent-origin md1 derivation is REFUSED, and S5's Trace B may need it.** The plan's unit tests cover `OriginDivergent`, and §6 P5 requires a divergent build at S6. If the Trace B **walk** is divergent-origin, the mint will refuse with a named error. Closing it means finding md's own invocation form for per-slot origins — the bracketed `[fp/path]@i` form is a template parse error on md 0.13.0, measured — and proving that form against a real walk. This is work, not a flag, and it belongs to whoever writes Trace B.
2. **`mkEncode` has no separator guard**, while the new md1 and ms1 parsers do. Deliberate scope call: mk1's unbroken form is already proven by a committed, live-verified expectation, and widening the S0 derivation path in this commit was not worth the blast radius. A 2-line convergence, worth a follow-up.
3. **`VerifyRecord`'s error text says "re-walk, do not edit it"**, which is wrong advice for a pin move — the ruling already noted it as a wording follow-up. Confirmed still present; not fixed here.
4. **`mk-cli-v0.13.0` is untagged.** `TestPinsAreCurrentWithTheirPrimaries` logs it as a NOTE every run; the ruling's S6/ship hygiene item stands, and now has an instrument that keeps asking.
5. **`plan-cite-gate.sh`'s `pkg.Symbol` allowlist excludes `oracle`** (and does not index `_test.go` symbols), so citations into that package cannot be machine-checked except as `file:line`.
