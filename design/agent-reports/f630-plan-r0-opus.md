# R0 adversarial review — `IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`

**Reviewer:** opus, independent (not the author).
**Artifact:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at mnemonic-engrave `5cc0a0a2`.
**Trees measured:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97`;
primary `/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`. Go 1.26.7
at `/scratch/code/shibboleth/.toolchain/go`.
**Method:** the plan's D1/D2/D3 gate was IMPLEMENTED (as an executable oracle) and
run against both the stale and the re-vendored corpus, then mutation-tested. T3
and T4 were EXECUTED in a throwaway `git worktree` of the fork at `95716e97`
(widened script + real re-vendor from `b2c5d693`), and the affected `./md/` and
`./gui/` tests were run against that state. The worktree was removed; both repos
verify clean (`git status --short` empty in each).

**Counts: 1 Critical, 7 Important, 4 Minor, 2 Nit.**

---

## Answer to the ONE question

**Will the gate fail on the drift it claims to catch?** Yes. Implemented exactly
as D1+D2+D3 specify, it reds **44 of 46** keyed vectors on the stale corpus and
**0 of 46** after the re-vendor, and every D1/D2/D3 clause is individually
load-bearing (six single-clause mutations, six catches — see §Q1). The gate has
teeth. Two holes remain (I-4, I-7).

**Will T2/T4 preserve the duplicate-key evidence?** **No — and worse, T2+T4
together red a fourth test the plan never names, by comparing a policy against a
different policy's addresses.** That is C-1, and it is the finding of this
review. The plan is built on the premise that F-529's re-vendor *deletes* the
three key-reuse vectors. Measured: `b2c5d693` still ships all three, **changed**.
Every mechanism in the fork that was written for "the vendored file is gone"
therefore never fires, and two of them fail instead.

---

## Critical

### C-1 — T2's pin + T4's re-vendor red `TestEveryKeyedVectorReachesAnAddress` on all three F-529 vectors: the pin supplies the card, the re-vendored record supplies the expected addresses

`gui/policy_address_test.go`'s `TestEveryKeyedVectorReachesAnAddress` takes its
**record** from a glob of the vendored corpus (`policy_address_test.go:124`,
`filepath.Glob(filepath.Join("..","md","testdata","vectors","keyed_*.conformance.json"))`)
and its **card** from `loadVectorChunks(t, name)` (`policy_address_test.go:168`),
which prefers `md/testdata/forkbuilt/<name>.md1.txt`
(`gui/taproot_script_path_test.go:136-147`). Those two halves are the same policy
today. After T4 they are different policies, and `assertMatchesRust` compares
addresses derived from one against addresses derived from the other.

Executed: worktree at `95716e97`, T2 simulated (the pre-re-vendor
`keyed_wsh_timelock_hashlock` card copied to `md/testdata/forkbuilt/`), T3+T4
simulated (all 247 `^(keyed_|compose_)` files copied from `b2c5d693`), then:

```
$ go test ./gui/ -run 'TestEveryKeyedVectorReachesAnAddress'
--- FAIL: TestEveryKeyedVectorReachesAnAddress/keyed_tr_multi_a
    policy_address_test.go:220: chain 0 index 0 via complex:
         got  bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt
         want bc1pgrupj0fjv79xtj05mzthds4dqzvdhcptx2gzpgzt90uzc86qzfes2a0yhh (rust)
--- FAIL: TestEveryKeyedVectorReachesAnAddress/keyed_tr_sortedmulti_a
    policy_address_test.go:220: chain 0 index 0 via complex:
         got  bc1p588jmtx4ptv76t9sclt6gt33eyydvsrea4njyayerqj2frw5m5aq5gzycw
         want bc1pkr9ldx6mqct9wk4lxj2qlu72gahnmp47lzhm2p7wx6mlnzqvlrpsxq3yul (rust)
--- FAIL: TestEveryKeyedVectorReachesAnAddress/keyed_wsh_timelock_hashlock
    policy_address_test.go:220: chain 0 index 0 via complex:
         got  bc1q6h9y4ngdfacaplw0qk67rugxs3vanv0jayk3n3xnhed7uke76zksfqt7py
         want bc1qa9wapjm45uthw7r806zuz9mev2a5cwqmrlwnyuj4pjs0c78d75rqp6j29e (rust)
```

Without T2's pin the wsh row fails differently and just as hard — the new policy
reuses nothing, so the `refusedByPolicy` entry goes stale:

```
policy_address_test.go:207: keyed_wsh_timelock_hashlock is listed as refused by
policy (@1 repeats inside one wsh miniscript; ...) but reaches the complex route
— either the gate regressed or the entry is stale
```

**Why this is Critical and not Important.** The plan's T4 gate is "the full
`./md/` and `./gui/` suites", and its own scope clause says a red "for a reason
other than a stale expected-value ... stops the plan". No task in T1–T5 addresses
this, so the plan self-terminates at T4 with three red subtests and a three-way
choice, two of whose branches destroy evidence the cycle exists to protect:

- move the three into `stillUnsupported` / delete the `refusedByPolicy` entries —
  the test's own comment names this outcome: *"Exempting these outright would
  have retired the cross-language address check for every shape the refusal
  covers"* (`policy_address_test.go:143-147`);
- stop preferring the pin in `loadVectorChunks` — deletes the F-533 and F-514
  witnesses outright, which is what D5 exists to prevent;
- keep the pin and split the record source from the card source — the only
  branch that preserves both, and the plan does not contain it.

D5 states a guarantee ("`keyed_wsh_timelock_hashlock` ... gets a pin of its own
first") that the plan's own T4 then breaks for all three vectors. Unmet
guarantee.

---

## Important

### I-1 — T4 reds `TestPinnedKeyReuseVectorsStillMatchTheVendoredCorpus`; its escape hatch fires only on a deletion that does not happen

`md/f533_pinned_vectors_test.go:82-86` skips when the vendored file is **gone**:

```go
raw, err := os.ReadFile(filepath.Join("testdata", "vectors", name+".phrase.txt"))
if err != nil {
    t.Skipf("the vendored %s is gone (%v); the pin is now the only copy, "+
        "which is exactly what it is for", name, err)
}
```

Measured — the primary still ships all three at `b2c5d693`, changed rather than
removed:

```
$ ls crates/md-codec/tests/vectors/ | grep -E '^keyed_(tr_multi_a|tr_sortedmulti_a|wsh_timelock_hashlock)\.'
keyed_tr_multi_a.{bytes.hex,conformance.json,descriptor.json,phrase.txt,template}
keyed_tr_sortedmulti_a.{...}
keyed_wsh_timelock_hashlock.{...}

fork tpl  keyed_wsh_timelock_hashlock: ... multi(1,@1/48'/0'/1'/2'/<0;1>/*,@2/...)   3 keys
prim tpl  keyed_wsh_timelock_hashlock: ... multi(1,@3/48'/0'/3'/2'/<0;1>/*,@4/...)   5 keys
```

So the file is present and different, and the test fails instead of skipping:

```
$ go test ./md/            # worktree, after the simulated re-vendor
--- FAIL: TestPinnedKeyReuseVectorsStillMatchTheVendoredCorpus
    --- FAIL: .../keyed_tr_multi_a       f533_pinned_vectors_test.go:89: pin has 4 chunks, vendored has 6
    --- FAIL: .../keyed_tr_sortedmulti_a f533_pinned_vectors_test.go:89: pin has 4 chunks, vendored has 6
```

This fires for the two **existing** pins with no T2 change at all, and D5's
"anti-drift check against the vendored file while it is still present" adds a
third row with the same fate. The failure message then prescribes the destructive
remedy: *"Re-copy the vendored phrase into testdata/forkbuilt/&lt;name&gt;.md1.txt
after checking WHY the primary changed it"* — doing that replaces the
reuse-bearing witnesses with reuse-free cards and reds every duplicate assertion
in the repo.

### I-2 — T3's stated gate cannot pass: `TestComposeVectorsMatchTheirProvenancePin` hardcodes 36 vector names and 176 files

T3's gate is `run the script, then go test ./md/ -run 'Provenance|Pin' -v`. T3's
text covers the provenance JSON and the directory-scan converse. It does not
cover the test's own two literals:

- `md/compose_vectors_pin_test.go:103` — `if p.Vectors != len(composeVectorNames)`,
  where `composeVectorNames` is a hand-maintained list of **36** names;
- `md/compose_vectors_pin_test.go:107` — `if len(p.Files) != 176`.

Executed, with the script's line 16 widened to `^(keyed_|compose_)` exactly as D4
prescribes:

```
$ bash scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic
vendored 247 files, 51 vectors, primary b2c5d6938432
$ go test ./md/ -run 'Provenance|Pin' -v
    compose_vectors_pin_test.go:104: pin says 51 vectors, this test knows 36
--- FAIL: TestComposeVectorsMatchTheirProvenancePin
--- FAIL: TestPinnedKeyReuseVectorsStillMatchTheVendoredCorpus   (I-1)
FAIL	seedhammer.com/md
```

There is a second, latent failure behind the `Fatalf`. The script already selects
one file the current pin does not carry:

```
$ python3 -c "...pin=176 files; script selects 177..."
selected but not pinned: ['compose_refusal_keyless_cap.json']
```

That file is deliberately excluded from `composeVectorNames` and covered by
`compose_refusal_vectors.provenance.json` (`isComposeVectorFile`,
`compose_vectors_pin_test.go:79-84`). Re-running the script pulls it into the
compose pin, and `compose_vectors_pin_test.go:134` then reports it as a *"pinned
file whose vector is not named here"*. This is pre-existing rot in the script,
but T3's only instruction is "run the script", so T3 is where it detonates.

### I-3 — T3 and T4 are the same action, so T4's "verify that expectation with a diff count before committing" is foreclosed (ordering)

`scripts/vendor-compose-vectors.sh` copies **and then** hashes the destination:

```sh
18  for f in "${files[@]}"; do cp "$VEC/$f" "$DST/$f"; done
22  rows = [{"name": f, "sha256": ...open(os.path.join(dst, f))...} for f in files]
```

The pin's sha256 rows are taken from the files it has just overwritten, so there
is no tree state in which the script is widened, the pin is regenerated, and the
corpus is still stale. T3's gate instruction ("run the script") performs T4's
re-vendor. Consequences:

- T3 cannot commit "script + pin" alone — a pin over a stale corpus is red by
  construction;
- T4's "Expect exactly 41 records to change ... Verify that expectation with a
  diff count before committing" must be taken against the T3 commit, not the
  working tree, and the plan does not say so or give the command.

T1→T2 order is right (the pin must precede the re-vendor). T3/T4 is not two
tasks.

### I-4 — D3's allowlist pins the **Go** side of the gap; the vendored (drifting) side is unpinned, and a real divergence hides behind it

D3 exempts `keyed_wpkh` and `keyed_tr_keyonly` from the bracket-path comparison
and pins "their measured Go paths", failing "if an allowlisted vector stops
diverging". The Go path is not the half that drifts — the *record* is, and after
T3 those two records are newly under the vendor script's authority.

Measured Go origins: `keyed_wpkh` → `m/84h/0h/0h`, `keyed_tr_keyonly` →
`m/86h/0h/0h`; both records carry a bare `[73c5da0a]`.

Mutation A1, run through the implemented gate — the record re-pointed to a
different account, consistently (bracket `[73c5da0a/84'/0'/9']`, header depth 3,
child `9h`, parent fp 0, key material untouched):

```
PASS (gate blind) :: A1 keyed_wpkh re-pointed to account 9h — still diverges from Go
PASS (gate blind) :: A1 keyed_tr_keyonly re-pointed to account 9h — still diverges from Go
```

A wrong-account descriptor on the two plainest single-key vectors in the corpus
passes every clause. The second half of the question is sound, though: the decay
clause **does** work in both directions —

```
FAIL (gate caught) :: A2 DECAY: Rust converges to the Go canonical path
    keyed_wpkh: D3 allowlisted vector STOPPED diverging (both [84h,0h,0h])
```

— and the same fires if the Go port drops its `canonicalOrigin` fallback
(`md/expand.go:76-81`), since Go's path then becomes empty and equals the
record's. So D3 catches convergence from either side and misses **replacement**.

### I-5 — T1's RED acceptance criterion is 41; the gate as specified reds 44

T1: *"it must fail on the 41 drifted records and pass on the 2 identical ones."*
Implemented D1+D2+D3, run against the stale corpus at `95716e97`:

```
STALE corpus: 44 vectors FAIL, 2 PASS
PASS: ['keyed_tr_keyonly', 'keyed_wpkh']
```

The extra 3 are the F-529 trio, whose descriptors are stale too. Independently
classified:

```
identical 2 ['keyed_tr_keyonly','keyed_wpkh']
descriptor-only 41
semantic 3 ['keyed_tr_multi_a','keyed_tr_sortedmulti_a','keyed_wsh_timelock_hashlock']
```

An implementer who takes "41" as the acceptance number sees 44 and has three
plausible readings, one of which — tune the gate until exactly 41 fail — exempts
precisely the three vectors this cycle is most exposed on. The criterion should
be 44 (41 + 3) or stated as "all but the 2 identical".

### I-6 — T1's loader is unspecified and load-bearing: the pin-preferring one reds the new gate on three vectors after T4

The plan places the new test in `md/conformance_keyed_test.go`, where two loaders
are in scope: `loadPhraseChunks` (vendored, `conformance_keyed_test.go:133`) and
`vectorChunksFor` (pin-preferring, `duplicate_keys_test.go:16`). The repo's own
F-614 note tells a reader to pick the second — *"Every other consumer of those
two names already went through a pin-preferring loader ... this one did not"*
(`md/policy_shape_test.go:7-16`).

Measured, both ways, against the re-vendored corpus with T2's pin in place:

```
T1 gate w/ loadPhraseChunks (vendored) : 0 of 46 FAIL
T1 gate w/ vectorChunksFor (pin-pref)  : 3 of 46 FAIL
  keyed_tr_multi_a           -> D2b descriptor xpub material not in Go expansion
  keyed_tr_sortedmulti_a     -> D2b descriptor xpub material not in Go expansion
  keyed_wsh_timelock_hashlock-> D2b descriptor xpub material not in Go expansion
```

D2b binds the record to "the Go port's own expansion **of the same card**", and
after T2+T4 "the same card" is ambiguous for exactly three vectors. The plan must
say which, and T4's "T1's gate must now be GREEN" is only true for one of the two
readings.

### I-7 — the origin bracket's **fingerprint** is bound by nothing, in a gate whose whole subject is the origin bracket

D1 relates the bracket's *path* to the header; D3 relates the path to the Go
expansion; D2 binds key material. Nothing binds `[XXXXXXXX/...]`. Mutation,
through the implemented gate on the re-vendored corpus:

```
PASS (gate blind) :: M2 every bracket fingerprint -> deadbeef   (keyed_wsh_multi_2of3)
```

This is squarely inside the class the gate exists for: in the Rust function
0.44.0 rewrote, the fingerprint is assembled on the same line as the path —

```rust
// crates/md-codec/src/to_miniscript.rs, assemble_origin_and_xkey
let origin = e.fingerprint.map(|fp| (Fingerprint::from(fp), path));
```

— so a regression there emits a uniformly wrong fingerprint that agrees with
itself, which is the exact failure mode D2 was written for. The assertion is free
and already available in the expansion D2b calls: `ExpandedKey.Fingerprint` /
`FingerprintPresent` (`md/expand.go:61-68`), and it agrees with every record
today:

```
descriptor bracket fingerprint vs Go ExpandedKey.Fingerprint: 284/284 AGREE, 0 disagree
```

Addresses do not depend on the fingerprint, so `TestEveryKeyedVectorReachesAnAddress`
cannot see this; `wallet_policy_id` is computed from the card, not the descriptor
string, so `TestKeyedConformanceAgreesWithRust` cannot either.

---

## Minor

### M-1 — D5's stated rationale is false: none of the three dependent sites goes green

D5: *"Re-vendoring it without a pin first would ... leave those assertions green
against a shape that no longer carries the defect."* Executed — re-vendor
applied, **no** pin added:

```
$ go test ./md/
--- FAIL: TestDuplicateKeySlotOnTheCorpusKeyReuseVectors/keyed_wsh_timelock_hashlock
    duplicate_keys_test.go:93: kind=0 (slot @0), want 1 -- Core REFUSES: ...
$ go test ./gui/ -run 'TestConsentWarnsOnDuplicateKeys|TestEveryAddressSurface...'
--- FAIL: TestConsentWarnsOnDuplicateKeys/keyed_wsh_timelock_hashlock
    composer_flow_test.go:619: the predicate says 0 (@0), want 1; ...
--- FAIL: TestEveryAddressSurfaceCarriesTheDuplicateWarning
    composer_flow_test.go:690: the fixture no longer carries a miniscript duplicate (0); ...
```

All three fail **loudly**, by explicit kind assertions. T2 is still the right
task — the witness must survive — but it is justified by "the evidence would be
deleted", not by "the assertions would go green". The distinction matters because
the false version is what leads to C-1: it frames T2 as sufficient, when what T4
actually needs is a card/record source split.

### M-2 — "Each task ends green before the next begins" contradicts T1, which must end RED

The Tasks preamble states the invariant; T1 states *"Run it against the stale
corpus and record the output: it must fail."* The plan does not say what state T1
commits in (skipped? `t.Skip` behind a build tag? committed red?), and T2's and
T3's gates run on whatever that is. Both use `-run` filters that miss the new
test, so it is survivable — but the invariant as written is false at T1.

### M-3 — a fourth dependent site on `keyed_wsh_timelock_hashlock` is not named

The plan names three. There is a fourth: `md/compose_shape_test.go:111`
(`TestPolicyShapeSplitsTheShippedOrCards`), reached through `shapeFromVector` →
`vectorChunksFor`. Measured harmless — the re-vendored policy has the same branch
shape (`{K:2,N:3,Keys:3,...}` / `{K:1,N:2,Keys:2,...}`), so it stays green with or
without the pin — but an enumeration used to justify D5 should be complete.

### M-4 — "88 chain-descriptor entries move across the 41"

Measured: **82** across the 41; 88 across all 44 (41 + the 3 semantic, 2 chains
each). The number is right for the wrong denominator, and T4 asks the implementer
to verify the expectation with a diff count, so a reader checking "88 across the
41" finds 82 and cannot tell which half is wrong.

```
outside the 3: descriptor-entries changed=82  address-entries=0  id-fields=0  other-fields=0
```

(That run also confirms T4's "0 address and 0 id lines outside those 3": true,
and additionally 0 changes to `template`, `path`, `keys`, `fingerprints`, `name`.
The file-set is stable — 247 files both sides, 0 added, 0 removed — so the
expectation's silence on additions costs nothing today.)

---

## Nit

### N-1 — D3's two halves contradict each other

*"The gate therefore does **not** assert bracket-path == Go origin path globally;
it carries an explicit allowlist ... and fails if an allowlisted vector stops
diverging **or a non-allowlisted one starts**."* The second clause *is* a global
assertion over non-allowlisted vectors. Read literally, the first clause deletes
D3's teeth: with the comparison removed for everyone, a uniformly-wrong origin
(bracket and header drifting together) passes. Implemented with the comparison
in place, it is caught:

```
FAIL (gate caught) :: D3: origin 48'/0'/N'/2' -> /3' with the header following
    D3 bracket path [...,2147483651] != Go origin [...,2147483650]
```

### N-2 — four other parts of the descriptor string are outside the gate's scope

Recorded for completeness, not as things to add. On the re-vendored corpus, these
mutations pass all six clauses: the BIP-380 checksum corrupted; `chains["0"]` and
`chains["1"]` descriptors swapped; the derivation suffix `/0/*` → `/7/*`; two
`multi()` key positions swapped. D1's scoping decision ("over the header, not the
descriptor string") is justified in the plan and these are consequences of it.
The last two would be caught as *drift* — the primary would emit matching wrong
addresses, and `TestEveryKeyedVectorReachesAnAddress` compares Go-derived
addresses per chain against the record's — so only an internally inconsistent
record slips through. The checksum and the chain-index suffix are covered by
nothing.

---

## Q1 — does D1+D2 have teeth? (detail)

Yes. Each clause was mutated alone on the re-vendored corpus, one vector
(`keyed_wsh_multi_2of3`), and every one was caught:

| mutation | outcome |
| --- | --- |
| header `depth` → 0 (the literal 0.44.0 defect) | caught by D1 |
| header `child_number` → 0 | caught by D1 |
| `parent_fingerprint` → the real parent fp `1cf29716` | caught by D1 |
| one base58 char flipped inside an xpub | caught by D1 (checksum) |
| `keys[]` replaced with another vector's key | caught by D2a |
| one `[origin]xpub` dropped from the descriptor | caught by D2b ("Go slot @0 never appears") |
| origin `…/2'` → `…/3'` with the header following it | caught by D3 |

D2b's cross-language clause is real rather than decorative: all 46 vectors have
every slot carrying an xpub (0 with `XpubPresent=false`), and no vector has one
xpub at more than one slot, so "some slot" and "every slot appears" are both
unambiguous and neither is vacuous.

The mutations that **pass** are I-7 (bracket fingerprint) and N-2's four.

## Q5 — ordering (detail)

T1→T2 is correct and necessary. T2→T4 is correct and necessary. The defects are
not in the sequence but in two tasks' contents:

- **T3 and T4 are one action** (I-3) — running the script is the re-vendor.
- **T4's gate cannot pass** (C-1, I-1, I-2), and the plan provides no task that
  makes it passable. No task leaves a *later* gate unable to **fail**; the
  problem is the inverse — T3's and T4's gates cannot **pass**.

## What was NOT found

- No defect in the D1 header rule itself: the fork's `bip380.Key.ExtendedKey()`
  path and the corpus at `b2c5d693` agree on all 46 vectors under the full gate
  (0 of 46 fail).
- No ambiguity in D2b/D3's material→slot lookup (no shared material across
  slots).
- The widened vendor pattern adds and removes nothing: 247 files on each side,
  `comm` empty in both directions.
- T4's "0 address and 0 id lines outside those 3" is true as stated.
