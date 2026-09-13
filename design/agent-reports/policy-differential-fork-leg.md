# Policy differential harness — DEVICE LEG (seedhammer fork)

Branch `policyprobe`, tip **f51d096**, branched from fork main **812ff06**.
Worktree `/scratch/code/shibboleth/.tmp/seedhammer-policyprobe`. Nothing pushed;
no commits on `main` (still `812ff06`, clean).

Two commits, one per deliverable:

| commit | what |
| --- | --- |
| `302e677` | `gui/policy_address_export.go` + `gui/policy_address_export_test.go` |
| `f51d096` | `cmd/policyprobe/main.go` |

`git diff --stat 812ff06..HEAD` → 3 files, 352 insertions, 0 deletions. No other
file in the fork is touched.

---

## 1. The I/O contract as built

### Input — one JSON object per line on stdin

```json
{"id":"case-0007","chunks":["md1...","md1..."],"indices":[0,1,2]}
```

| field | type | meaning |
| --- | --- | --- |
| `id` | string, **required, non-empty** | echoed back verbatim; how the driver matches a result to a case |
| `chunks` | array of string | the md1 chunk set, as handed to `md.ExpandWalletPolicyChunks` |
| `indices` | array of uint32 | derived on **both** chains, in the order given |

- Blank lines are skipped and produce **no** result line.
- Max line length 16 MiB (md1 chunks run ~80 B each, so this is room for tens of
  thousands); the cap exists so a corrupt stream fails with a named line rather
  than being read until memory runs out.

### Output — one JSON object per line on stdout, same `id`, input order

Success:

```json
{"id":"case-0007","ok":true,"keys":3,"receive":["bc1q…","bc1q…"],"change":["bc1q…","bc1q…"]}
```

Failure:

```json
{"id":"case-0008","ok":false,"stage":"expand","error":"md: chunk set incomplete"}
```

| field | type | present when |
| --- | --- | --- |
| `id` | string | always |
| `ok` | bool | always |
| `keys` | int | whenever **expand succeeded** — including `keys: 0` |
| `receive` | array of string | `ok:true`; `len == len(indices)`, same order; `[]` never `null` |
| `change` | array of string | `ok:true`; same shape as `receive` |
| `stage` | `"expand"` \| `"source"` \| `"derive"` | `ok:false` |
| `error` | string | `ok:false` |
| `panic` | bool `true` | only when the stage panicked (omitted otherwise) |

`keys` is a `*int` in the source **on purpose**: a keyless template is a real and
interesting `source` refusal, and `omitempty` on a plain `int` would erase
exactly that case. So `keys` absent means "expand failed, count unknown", and
`"keys":0` means "the policy seats zero keys".

**Exactly one result line per non-blank input line, in input order.** Results are
flushed per line, so the tool works as a co-process answering case by case as
well as on a whole file (measured, §3).

### The three stages, never collapsed

| `stage` | what failed | `error` is |
| --- | --- | --- |
| `expand` | `md.ExpandWalletPolicyChunks` | the codec's error, verbatim |
| `source` | `gui.PolicyAddressSource` returned `!ok` | an **observation** — see below |
| `derive` | `at(index, change)` | `receive[N]: …` or `change[N]: …`, naming the index and chain |

A policy the device cannot derive from is `stage:"source"` — **a result, not an
error and not a crash**. That is one of the things the harness exists to count.

### The `source` note is an observation, never a diagnosis

`complexAddressSource` returns `(func, bool)` and **no reason**, so the verdict is
its alone and the note can never change it. The text describes the expanded keys
— facts the driver can re-check — and never claims which branch inside the device
fired. A branch claim would go stale the first time the refusal set changed and
would then be a confident wrong answer rather than a missing one.

```
the device declined: the policy seats no keys, so there is nothing to derive from
the device declined: no xpub for @1, @2
the device declined this policy shape: an unsupported use-site, or its index-0 derive probe failed
```

### Failing loudly instead of fabricating a result

A line that is not a well-formed case (bad JSON, empty `id`, a negative index) is
a **driver bug, not a measurement**. Turning one into a policy result would
corrupt the counts the harness produces; skipping it would silently drop a case
the driver is waiting on. So the tool names the line on stderr and exits **1**,
leaving every result already emitted intact.

`stdout` is the contract; `stderr` carries diagnostics only (malformed-line
reports, panic stacks).

### Network

**Mainnet only, and there is no flag.** `gui/policy_address.go` is mainnet-only by
design (D1), so a network flag here could only be a lie about what the device
does.

---

## 2. Derivation path — the device's, wrapped not copied

```go
_, keys, err := md.ExpandWalletPolicyChunks(c.Chunks)   // stage "expand"
at, ok := gui.PolicyAddressSource(c.Chunks, keys)       // stage "source"
at(idx, false) / at(idx, true)                          // stage "derive"
```

`gui.PolicyAddressSource` (new, `gui/policy_address_export.go`, 23 lines incl.
doc comment) has **one statement**:

```go
func PolicyAddressSource(collected []string, keys []md.ExpandedKey) (func(uint32, bool) (string, error), bool) {
	return complexAddressSource(collected, keys)
}
```

`complexAddressSource` is neither exported nor copied. Copying it would make the
harness measure the copy — reporting three-way agreement while the screen an
operator actually reads drifted away from the function under test.

`TestPolicyAddressSourceIsTheDeviceFunction` asserts the wrapper equals
`complexAddressSource` on **both verdicts** (a wrapper that agreed only where the
device succeeds would miss the worse mutation: turning a refusal into an answer).

**The test's own blind spot, stated in its doc comment:** a byte-identical copy of
`complexAddressSource` would produce identical output and pass. No Go test can
tell a call from an identical copy; that one is caught by reading the diff, which
is why the wrapper's doc comment says the body must stay one call.

---

## 3. Everything below was RUN, not reasoned about

### 3a. The vector reproduction — 6/6 addresses agree with the Rust primary

Command (from the worktree root, `V=md/testdata/vectors/keyed_compose_wsh_timelock_hashlock`):

```sh
go build -o /scratch/code/shibboleth/.tmp/policyprobe ./cmd/policyprobe/
python3 -c "
import json
chunks=[l.strip().replace(' ','') for l in open('$V.phrase.txt')]
chunks=[c for c in chunks if c.startswith('md1')]
print(json.dumps({'id':'tlhl','chunks':chunks,'indices':[0,1,2]}))
" | /scratch/code/shibboleth/.tmp/policyprobe
```

Output (one line, wrapped here for reading):

```json
{"id":"tlhl","ok":true,"keys":3,
 "receive":["bc1q7wcdsw56sme3zread4qlq533gdavtmgacsg8t4hkxhrcqw2ajdgssvu6tm",
            "bc1qhtca9pgmy6kez9kmrqrt2xc22s28xtwlzc3ukg7hjdmr7jjg0emqp82dsr",
            "bc1qe4t3m9mf64jzdfgmgqcmuwke6dq46cj4fk6sjgxnp3lh5upcw6msfphchk"],
 "change": ["bc1q88ua3a7sltvfrwe74tkmj3tytx33dc6t8n33xd9rtdmude8j6slscxcjgk",
            "bc1qhn2h73hcxcnr4v4jlq6sw3d66358suxr7grl4ya785rzuxpjn77s43ra2n",
            "bc1qdu3tyl03cpgqnu56s0km3kr440fl34y38eznv5c3c95lmvhljrmqde8s5m"]}
```

Compared against `$V.conformance.json` (the record the Rust primary generates,
vendored and sha-pinned by `md/compose_vectors_pin_test.go`):

```
MATCH   chain 0 receive 3 addresses
MATCH   chain 1 change  3 addresses
```

All six byte-identical. **Both chains checked, not just receive** — the brief
asked for the receive addresses; deriving change from the same source and
checking it too costs nothing and catches a chain-flag mistake that a
receive-only comparison cannot see.

### 3b. The whole vendored corpus — 67 vectors in one batch, 0 panics

Every `md/testdata/vectors/*.phrase.txt` as one case with `indices:[0,1]`:

```
cases: 67 {'ok': 38, 'source': 16, 'expand': 13}

 13  expand  md: wire version mismatch
  8  source  the device declined this policy shape: an unsupported use-site, or its index-0 derive probe failed
  3  source  the device declined: no xpub for @0, @1, @2, @3, @4, @5, @6, @7
  2  source  the device declined: no xpub for @0 … @31
  2  source  the device declined: no xpub for @0, @1
  1  source  the device declined: no xpub for @0, @1, @2

stderr: 0 bytes.  "panic":true in output: 0.
```

All three stages and all three `source` notes fire on real input. Exit 0.

### 3c. Stage triage, ordering and edge cases

```
{"id":"ok-tlhl",         "ok":true, "keys":3, "receive":[…2],"change":[…2]}
{"id":"expand-garbage",  "ok":false,"stage":"expand","error":"codex32: not a valid md1 string"}
{"id":"expand-empty",    "ok":false,"stage":"expand","error":"md: empty chunk set"}
{"id":"source-keyless",  "ok":false,"keys":2,"stage":"source","error":"the device declined: no xpub for @0, @1"}
{"id":"derive-hardened", "ok":false,"keys":3,"stage":"derive","error":"receive[2147483648]: cannot derive a hardened key from a public key"}
{"id":"ok-noindices",    "ok":true, "keys":3, "receive":[],"change":[]}
{"id":"after-blank",     "ok":true, "keys":3, "receive":[…1],"change":[…1]}
```

7 cases in (one blank line among them), 7 results out, in order. Note
`source-keyless` carries `"keys":2` while the `expand` failures carry none —
that is the `*int` decision doing its job.

### 3d. The loud-failure path

```
$ printf '{"id":"a",…}\n{"id":"b","chunks":[],  BROKEN\n{"id":"c",…}\n' | policyprobe
{"id":"a","ok":false,"stage":"expand","error":"md: empty chunk set"}
policyprobe: stdin line 2: malformed case: invalid character 'B' looking for beginning of object key string
exit=1
```

Line 1's result survived. Also:

```
empty id       -> policyprobe: stdin line 1: case has no id: the driver could not match a result to it   exit=1
negative index -> policyprobe: stdin line 1: malformed case: json: cannot unmarshal number -1 …          exit=1
```

### 3e. Co-process streaming

A Python parent wrote one case, flushed, and read a result **before closing
stdin**:

```
got while stdin still open: {"id":"live","ok":false,"stage":"expand","error":"md: empty chunk set"}
```

### 3f. Mutation tests — every gate here has been made to fail

**The wrapper test** (both mutants caught, then reverted and re-run green):

| mutant | result |
| --- | --- |
| wrapper derives at `index+1` | FAIL — `at(0,true) wrapper bc1qhn2h73… device bc1q88ua3a7…` |
| wrapper returns a deriver where the device said `!ok` | FAIL — `the wrapper answers true where the device answers false` |

**The panic-recovery path** (injected `panic()` at the source stage, built to a
throwaway binary, source restored):

```
{"id":"before","ok":true,"keys":3,…}
{"id":"BOOM","ok":false,"stage":"source","error":"panic: injected: the address layer exploded","panic":true}
{"id":"after","ok":true,"keys":3,…}
exit=0

stderr: policyprobe: BOOM panicked in source: injected: the address layer exploded
        goroutine 1 [running]: …
```

The batch survived, the stage was named correctly, and `panic:true` keeps it from
ever being counted as an ordinary refusal. This was run rather than assumed
because a recovery path that has never fired is a hypothesis, not a gate.

### 3g. Required gates

| gate | result |
| --- | --- |
| `gofmt -l` on the three new files | clean (no output) |
| `go build ./...` | clean, exit 0 |
| `go vet ./cmd/policyprobe/` | clean, exit 0 |
| `scripts/gui-shard-test.sh ./gui/ 24` | **`RESULT: ok -- all 1292 tests ran across 24 shards`**, `partition verified exhaustive: 1292 == 1292`, 32s wall |

`gofmt -l .` over the whole tree also lists `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`,
`mt/mt_test.go`. **Those are pre-existing at main `812ff06` and untouched here**
— `git status` is clean apart from the three added files, so they are byte-for-byte
as `main` has them. Not fixed, because the brief says touch nothing else.

---

## 4. Deviations and judgment calls

1. **`panic` field, not in the brief's schema.** The brief specifies `stage` as
   one of three values and says a refusal is a result, not a crash. It does not
   say what to do with a genuine panic in `md`/`address` on a hostile policy. I
   catch it, report it as a failure of the stage it happened in (so `stage` stays
   honest), and add `panic:true` so it can never be confused with a refusal. A
   crash found by a random-policy harness is arguably the most valuable thing
   this leg can produce, and losing it — with every remaining case in the batch —
   to a process death would be the worst way to learn it. **0 of the 67 vendored
   vectors panicked**, so the field is absent from all real output so far.

2. **The `source` reason is synthesised at the call site.** The brief asks for a
   reason; `complexAddressSource` supplies none and the wrapper may carry no
   logic. So the note is computed in `cmd/policyprobe` from the `ExpandedKey`
   values, strictly as an observation of the input, never as a claim about which
   branch fired. The verdict is unaffected. Flagging it because it is the one
   place where text in the output does not come from the device function itself.

3. **`keys` is `*int`.** Needed to distinguish `keys:0` from "unknown". The
   brief's example failure line has no `keys` and still serialises correctly.

4. **Sequential, not parallel across cores.** The house directive says to consider
   parallelism for long CPU-bound work, and this is CPU-bound (secp256k1). I kept
   it single-threaded for two reasons: the goroutine-safety of `md` + `address`
   under concurrent derivation is unverified, and a data race there would corrupt
   the differential silently — the exact failure this harness exists to detect;
   and ordered per-line streaming (which makes co-process use possible) fights
   with reordering. **The lever is left with the driver**: cases are independent,
   so sharding a case file across N `policyprobe` processes parallelises this
   perfectly with no shared state. The brief's stated motivation was Go start-up
   cost per policy, which one process per shard already solves.

5. **Both chains checked against the conformance record**, where the brief asked
   only for the receive addresses. See §3a.

6. **Duplicate `id`s are not policed.** The id space is the driver's; the tool
   echoes what it is given.

## 5. Open question for the driver author

Nothing blocking. One contract detail worth agreeing on explicitly: on a
malformed input line the probe **exits 1 mid-batch**, so the driver must treat a
non-zero exit as "my input was bad, results are truncated at the named line"
rather than "the device failed". Every device-side outcome, including a panic,
is reported as a result line with exit 0.
