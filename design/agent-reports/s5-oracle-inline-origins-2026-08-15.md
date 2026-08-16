# S5.0 oracle: inline per-`@N` origins, divergence no longer refused

**Date** 2026-08-15
**Worktree** `/scratch/code/shibboleth/seedhammer-s5`, branch `s5-oracle-block`
**Base** `5edb162` (two ahead of `main` `80d0c5d`) — working tree, uncommitted
**Scope** oracle code only. No `gui/` or tail code was written.

Files changed (4):

```
 oracle/expect.go       | 226 ++++++++++++++++++++++-------
 oracle/expect_test.go  | 224 ++++++++++++++++++++++++++---
 oracle/live_test.go    | 376 +++++++++++++++++++++++++++++++++++++++++++++++++
 scripts/oracle-live.sh |   2 +-
 4 files changed, 752 insertions(+), 76 deletions(-)
```

---

## 1. The md invocation, before and after

**BEFORE** — origins carried out of the template into a shared `--path` flag:

```
md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))" \
  --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> \
  --path m/48h/0h/0h/2h \
  --network mainnet --group-size 0 --force-chunked --policy-id-fingerprint
```

**AFTER** — one origin inline per placeholder, no `--path` at all:

```
md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/0'/2'/<0;1>/*))" \
  --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> \
  --network mainnet --group-size 0 --force-chunked --policy-id-fingerprint
```

The `d.note("md", …)` provenance string emits exactly the AFTER form. It was
machine-checked as **runnable**, not merely read: the note string was lifted
verbatim out of the live test log, the documented `<placeholder>` filled with the
three real xpubs, and run. It reproduced the derivation byte for byte.

```
$ diff note-replay-chunks.txt test-derived-chunks.txt
DIFF_EXIT=0
```

## 2. The compatibility fact — re-verified, not trusted

Measured against the pinned `md 0.13.0` (`~/.cargo/bin/md`, absolute path; `md`
is a shell alias for `mkdir -p` on this machine), 3-key S2 policy, real xpubs:

```
--path : md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))" ... --path "m/48'/0'/0'/2'"
inline : md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/...,@2/...))" ...

$ diff path-form.txt inline-form.txt
DIFF_EXIT=0
```

Byte-identical across the **whole** stdout, not just the chunks: same
`chunk-set-id: 0x30d86`, same six md1 strings, same
`policy-id-fingerprint: 0x06215ac0`.

Divergent form confirmed to encode, and `md decode --json` confirms what it is:

```
descriptor.path_decl = {"tag":"Divergent",
                        "data":["m/48'/0'/0'/2'","m/48'/0'/1'/2'","m/48'/0'/0'/2'"]}
```

while the uniform inline form decodes as `{"tag":"Shared","data":"m/48'/0'/0'/2'"}`
— i.e. inline-uniform and `--path` are the same declaration, which is why nothing
staled.

### Two facts I found that the brief did not state, and that would have been review rounds

**(a) `h` notation is REJECTED in the template.** Every committed tuple records
origins as `m/48h/0h/0h/2h`. md's `--path` **flag** accepts both notations, but its
**template parser** accepts only apostrophes:

```
@0/48h/0h/0h/2h/<0;1>/*    -> md: template parse error: @0: derivation steps after
                              the multipath group are not representable in md1;
                              the multipath `<...>` must be the final derivation
                              step before the wildcard
@0/m/48'/0'/0'/2'/<0;1>/*  -> the same refusal
@0/48'/0'/0'/2'/<0;1>/*    -> encodes
```

So the change needs a normalisation (`strip m/`, `h -> '`) that the brief's
example silently assumed. Note the diagnostic blames the multipath, not the
notation — a reader debugging from the message alone would look in the wrong
place, which is why it is quoted verbatim in the code.

**(b) `pathRe` makes the BIP-48 script_type level OPTIONAL.** My first
`inlineOrigin` validated against `pathRe` directly, and it accepted
`m/48h/0h/0h` — a path `templateForOrigin` refuses. That would have spliced a
script-type-less origin straight into a policy template. Caught by the function's
own test on first run; fixed by validating **through** `templateForOrigin`, so
the gate has one authority on what an origin may be rather than two readings that
can drift.

## 3. What changed

1. **`policyTemplate(msTemplate string, k int, origins []string)`** — was
   `(msTemplate, k, n int)`. Emits `@i/<origin_i>/<0;1>/*` per slot, universally,
   uniform and divergent alike. Refuses an empty slot set and any origin
   `templateForOrigin` cannot name.
2. **`inlineOrigin(origin)`** — new; the notation normalisation above.
3. **`uniformOrigins` -> `uniformScriptAndNetwork`**, now returning
   `(template, network, err)`. The divergence refusal and the whole comment block
   that documented the false premise are **deleted**, replaced by a block that
   records what was actually measured and why the old measurement was wrong. The
   two refusals that are still true are **kept** and their justifications
   restated: mixed script types (one md1 carries one root tag) and mixed networks.
4. **`mdEncode`** — `path` parameter dropped, `--path` dropped. Its doc now states
   that re-adding the flag would be a **funds defect** rather than a redundancy,
   quoting md's own help ("flattens Divergent mode to Shared"): it would silently
   overwrite per-slot paths and encode a different wallet without erroring.
5. **md1 artifacts no longer carry `Origin`.** `sharedPath` ceased to exist as a
   concept, and populating the field with slot 0's origin would assert that a
   two-account policy sits at one account — the same argument
   `CheckFingerprintScope` already makes for refusing a fingerprint on an md1.
   The information is not lost: the label carries the full template, which now
   holds *every* slot's origin in slot order — strictly more than the single path
   it replaced. S2's committed golden already records no origin on its md1
   artifacts, so this converges on the primary's own output. **This is a judgement
   call, not a transcription** — flagged here for that reason. No committed
   expectation holds an md1 artifact (S0 is cosigner-cards), so nothing staled.

## 4. Proof the S0 record and S2 golden are unchanged

**The files themselves**, sha256 of working tree vs `git show HEAD:`:

```
IDENTICAL  4cb8ae09af0cab10adab72ce8f205bb52fd7a345b7dc2d683dfac5bda5296da5  oracle/gaterecords/S0-trace-a.expect.json
IDENTICAL  18099d11912b40dd8437921cdda5f4e7bc214d6829a9f00bddb3c7fe580714b5  oracle/gaterecords/S0-trace-a.record.json
IDENTICAL  f292d51ad816f6231c7ba929421ed49c4c050d09428dd61c9ea552043d95f077  oracle/gaterecords/S0-trace-a.inputs.json
IDENTICAL  2dc9c5252d0ec188c961de56ed3ddbff5a73199cc7889fcc194c2c9453b0b5a3  oracle/gaterecords/S0-trace-a.walk.json
IDENTICAL  c6d80c5da05e859dc0f65b732cfd1fd0241a86994b8409ce299264ddd483866d  gui/testdata/s2_md1_golden.expect.json

$ git diff --stat -- oracle/gaterecords/ gui/testdata/
(empty)
```

**And the gates that would have caught staleness ran and passed**, which is the
part that matters — unchanged files prove nothing on their own:

- `TestLiveDerivationReproducesEveryCommittedExpectation` PASS — re-derives S0
  through the **new** inline code path and requires string, origin and
  fingerprint to reproduce byte for byte.
- `TestBuiltPolicyDerivationMatchesTheS2Golden` PASS — derives the built policy
  through the new inline path and compares its md1 against S2's committed golden
  byte for byte. This is the byte-identity proof that inline-uniform ==
  `--path`, run through the real code rather than by hand.
- `TestAssembledMd1MatchesThePrimaryByteForByte` PASS (gui, untouched).

## 5. The new divergent test

`TestBuiltPolicyDerivesDivergentOrigins` (oracle/live_test.go, `oraclelive` tag,
added to the `-run` allowlist in `scripts/oracle-live.sh` per that file's own
rule). Four assertions:

1. **It derives.** A divergent-origin expectation produces a full artifact set
   satisfying `CheckArtifactShape` and `CheckFingerprintScope`.
2. **It differs, with the xpubs held constant.** See §7 — the obvious control is
   not sufficient, and I only found that out by running the mutation.
3. **The mapping holds through the encoder.** `md decode --json` on the derived
   chunks, `path_decl.data[i]` compared positionally to slot i's origin.
4. **S5's own shape derives** — one master at two accounts plus another master.

Evidence (final green run):

```
divergent md1 differs from the SAME xpubs under a flattened template — the per-slot origins reached md
divergent md1 (6 chunk(s)); uniform control (6 chunk(s))
  divergent chunk 0: md1feex7zs9qjtvyyy5jmpprjjtvyy49gqpsgwzyxzhs79m5rru59s2su80aw2q4wgdpaq2x96ghfr2aeyj
  divergent chunk 1: md1feex7zs2snl2rd0q6gghvalgy07r0ck4wcczrgalt7lhxlg0x6vnl4rdcjgnpya7k5cqe7e6un26qs9t
  divergent chunk 2: md1feex7zsj6e20ur0anz7jwkzae8ef2l0uwvjq67znc3fa47zw97hmwk2vvhq2h6g9y9cwl79prqktnxxv
  divergent chunk 3: md1feex7zs6exshv8pd6dpvcr2p3wpgqzc7rjy286xe7dz4s054ug5dte8esaem8ax3a6vj49jts5pj4npe
  divergent chunk 4: md1feex7z39wv08c2h0xr9jk0pea8pgskhshw6nqffrult4ula2cj6maydu4y25lktqkqqz8z8wsv2ltevn
  divergent chunk 5: md1feex7z3tj3us7k4x9dcp2a2urh4yc3adpp0q03zhkh663frhy0yqhv0gqc2sq5jtlcffsv99vc
divergent path_decl: Divergent [m/48'/0'/0'/2' m/48'/0'/1'/2' m/48'/0'/2'/2']
uniform   path_decl: Shared [m/48'/0'/0'/2']
Trace B (one master at two accounts + another master): 1 ms1 + 5 mk1 + 6 md1; path_decl Divergent [m/48'/0'/0'/2' m/48'/0'/1'/2' m/48'/0'/0'/2']
  slot 0 card: fp 73c5da0a at m/48'/0'/0'/2'
  slot 1 card: fp 73c5da0a at m/48'/0'/1'/2'
```

The uniform control's md1 is byte-identical to S2's golden (asserted by the
sibling test), so "differs" is anchored to a known-good value, not to another
unverified derivation. Trace B shows the property only that shape has: two slots,
**one fingerprint**, **different paths**, and **one** ms1 for two held slots.

Untagged (no toolchain, runs everywhere):
`TestPolicyTemplateMapsSlotIOriginToPlaceholderI`,
`TestBuiltPolicyAcceptsDivergentOriginsAndReachesTheOracle`,
`TestPolicyTemplateEncodesTheDevicesOwnS2Wallet` (rewritten from
`TestPolicyTemplateMatchesTheDevicesOwnS2Template` — the binding to the device's
template is now a **relation**, since the two forms are no longer string-equal;
the byte-level binding is the live golden comparison, and the comment says so).

## 6. Mutations — each on a compiling tree, each proven RED

### (a) Flatten every origin to slot 0's

```go
o := origins[0] // MUTATION (a)
```

```
go build ./oracle/                     BUILD_EXIT=0
go vet -tags oraclelive ./oracle/      VET_TAGGED_EXIT=0
go test ./... -count=1                 UNTAGGED_EXIT=1
  --- FAIL: TestPolicyTemplateEncodesTheDevicesOwnS2Wallet
  --- FAIL: TestPolicyTemplateMapsSlotIOriginToPlaceholderI
go test -tags oraclelive -run TestBuiltPolicyDerivesDivergentOrigins  LIVE_EXIT=1
```

```
--- FAIL: TestBuiltPolicyDerivesDivergentOrigins
  THE ORIGINS WERE FLATTENED. Encoding the SAME three xpubs under the divergent template
  and under a template with every slot at "m/48h/0h/0h/2h" produced byte-identical md1, so
  the per-slot accounts never reached md and this expectation describes a different wallet
  than its own tuple.
  md decode reports path_decl tag "Shared" for a policy whose slots sit at different
  accounts, want "Divergent". A Shared declaration here means the per-slot origins were
  collapsed to one path.
```

### (b) Reverse the mapping — `(0..n).rev()`

```go
o := origins[len(origins)-1-i] // MUTATION (b)
```

```
go build ./oracle/                     BUILD_EXIT=0
go vet -tags oraclelive ./oracle/      VET_TAGGED_EXIT=0
go test ./... -count=1                 UNTAGGED_EXIT=1
  --- FAIL: TestPolicyTemplateMapsSlotIOriginToPlaceholderI
go test -tags oraclelive -run TestBuiltPolicyDerivesDivergentOrigins  LIVE_EXIT=1
```

Untagged, naming the slots:

```
SLOT 0's origin did not land on @0.
  placeholder 0 holds "@0/48'/0'/2'/2'/<0;1>/*"
  slot 0 records  "m/48h/0h/0h/2h", which is "@0/48'/0'/0'/2'/<0;1>/*" inline
SLOT 2's origin did not land on @2.
  placeholder 2 holds "@2/48'/0'/0'/2'/<0;1>/*"
  slot 2 records  "m/48h/0h/2h/2h", which is "@2/48'/0'/2'/2'/<0;1>/*" inline
```

Live, read out of md's own encoded bytes:

```
SLOT 0's origin did not land on @0 IN THE ENCODED BYTES.
  md decode reports path_decl.data[0] = "m/48'/0'/2'/2'"
SLOT 2's origin did not land on @2 IN THE ENCODED BYTES.
  md decode reports path_decl.data[2] = "m/48'/0'/0'/2'"
divergent path_decl: Divergent [m/48'/0'/2'/2' m/48'/0'/1'/2' m/48'/0'/0'/2']
```

Both mutations reverted; `oracle/expect.go` sha256 restored to
`1be9bfb2d769f50d1f2b16298619195a9add3e16b0df29b3f80c4b8cd8ffa7a1`, and
`grep -rn "MUTATION (" oracle/ scripts/` returns nothing.

## 7. Two things the mutations found that I would otherwise have shipped wrong

These are the parts I judged rather than transcribed, and both were **measured**
findings — neither was visible by reading.

**The brief's stated rationale for the "it differs" half is empirically false.**
The brief says: *"'it derived' without 'it differs' would pass on a silent
flatten."* I wrote that assertion as specified — divergent md1 vs the same three
masters at uniform origins — and under mutation (a) it **passed**. The reason:
each slot's xpub is derived at its own account by `ms derive`, so it carries the
account whether the template does or not. The two derivations differ because the
*keys* differ, not because the *origins* reached md. That comparison proves two
tuples are two wallets; it does not detect a flatten.

The control that actually detects it holds the xpubs **fixed** and varies only
the origins written into the template: encode the same three xpubs once under the
divergent template and once under a flattened one, and require those to differ.
That is what is committed, and it is what produced the `THE ORIGINS WERE
FLATTENED` output above. The weaker uniform comparison is kept, labelled for what
it actually proves.

**The live fixture was a palindrome and could not see mutation (b).** My first
divergent fixture was `(acct0, acct1, acct0)` — chosen because it is S5's Trace B
shape and isolates one variable against the uniform control. It is also **its own
reverse**, so under `(0..n).rev()` md's `path_decl` came back byte-identical and
the entire live test passed:

```
=== live test under mutation (b), palindromic fixture ===
LIVE_EXIT=0
ok  	seedhammer.com/oracle	0.195s
```

That is exactly the class of defect the brief's anecdote describes, reproduced
inside this very change. Fixed by moving the primary fixture to three distinct
accounts `(0,1,2)`, keeping Trace B's real `(0,1,0)` in its own section where the
claim is the same-master-two-accounts property rather than ordering, and adding a
**self-check that fails if the fixture ever becomes palindromic again**. Both
untagged and live mapping tests carry an equivalent guard: the untagged one
asserts that the reversed construction does not produce the same template.

## 8. Gate exit codes — all unpiped, all on the final tree

| check | command | exit |
| --- | --- | --- |
| format | `gofmt -l .` | 0, **empty output** |
| vet (cold `GOCACHE`) | `go vet ./...` | **1** — 40 findings, **40 of 40 in `_test.go`**, 0 outside. Identical to the stated baseline; this IS clean |
| vet, tagged | `go vet -tags oraclelive ./oracle/ ./gui/ ./sysw/` | 1 — one finding, `gui/freetext_sizeproof_golden_test.go:111 testing.ArtifactDir requires go1.26`. **Pre-existing**: it is finding #40 of the untagged baseline set, in a file not in this diff |
| vet, tagged, oracle only | `go vet -tags oraclelive ./oracle/` | **0** |
| suite (cold `GOCACHE`) | `go test ./... -count=1` | **0** — **51 ok / 0 FAIL** (baseline: 51 ok / 0 FAIL) |
| **live** | `./scripts/oracle-live.sh` | **0** — `live checks: PASS (exit 0)` |

Live breakdown, all PASS:

```
--- PASS: TestLiveDerivationReproducesEveryCommittedExpectation
--- PASS: TestRealPinsResolveTheInstalledOracles
--- PASS: TestPinsAreCurrentWithTheirPrimaries
--- PASS: TestBuiltPolicyDerivationMatchesTheS2Golden
--- PASS: TestBuiltPolicyDerivesDivergentOrigins
ok  	seedhammer.com/oracle
--- PASS: TestAssembledMd1MatchesThePrimaryByteForByte
ok  	seedhammer.com/gui
--- PASS: TestVendoredVectorsAreInSyncWithThePrimary
ok  	seedhammer.com/sysw
live checks: PASS (exit 0)
```

`GOCACHE` was pinned to a scratch dir and wiped cold on both sides of every vet
and suite comparison. No verdict in this report was read through a pipe.

**ASCII purity:** 0 non-ASCII string literals added to `oracle/expect.go`
(production). The 9 non-ASCII literals added are em-dashes in `_test.go` failure
messages, matching existing style in this file; the glyph guard is among the 51 ok.

## 9. Nothing is blocked.

## 10. Judgement calls, for a reviewer to overrule

1. **Dropping `Origin` from md1 artifacts** (§3.5). Defensible and argued in
   code, but it is a semantic change beyond the literal brief. Nothing committed
   depends on it.
2. **`TestPolicyTemplateMatchesTheDevicesOwnS2Template` became a relation, not an
   equality.** The oracle and the device now write the same wallet in two forms,
   so string equality was no longer available. The untagged test derives the
   expected string *from* the device's own literal by splicing, rather than
   restating a third literal; the byte-level binding remains the live golden
   comparison. The pre-existing weakness that gui's `s2WantTemplate` is *copied*
   into the oracle test (package `oracle` cannot import package `gui`'s test
   const — gui imports oracle) is unchanged, not worsened, and now stated.
3. **`scripts/oracle-live.sh` was edited** to add the new tagged test to its
   `-run` allowlist. That file's own header requires it ("EVERY TAGGED TEST MUST
   BE NAMED IN THE -run FILTER BELOW"), so I treated it as in-scope oracle
   plumbing rather than tail code.
