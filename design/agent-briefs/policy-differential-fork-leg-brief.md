You are the implementer for the DEVICE LEG of a policy differential harness, in the seedhammer fork.

## What the harness is for
Nothing generates random wallet POLICIES and pushes them through the whole seam — compose, encode to md1, decode back, seat keys, derive an address — and compares what three independent implementations say. The decoder is already fuzzed four ways and the codec has property tests on its primitives; the POLICY space is untested. Your leg is the device's answer. Another implementer builds the driver, the Rust leg and the Bitcoin Core leg in `mnemonic-engrave`; you never touch that repo.

## Your deliverable
`cmd/policyprobe` in the fork: a batch tool that reads cases on stdin and writes results on stdout, both as JSON. One process, many cases, so the driver is not paying Go start-up per policy.

Input, one JSON object per line (JSONL):
```json
{"id":"case-0007","chunks":["md1...","md1..."],"indices":[0,1,2]}
```
Output, one JSON object per line, same `id`, in input order:
```json
{"id":"case-0007","ok":true,"keys":3,"receive":["bc1q...","bc1q..."],"change":["bc1q..."]}
{"id":"case-0008","ok":false,"stage":"expand","error":"md: chunk set incomplete"}
```
`stage` is one of `expand`, `source`, `derive` — the driver triages by it, so name the stage that actually failed and never collapse them.

## How to derive, and this is the point
Use the SAME path the device's inspect screen uses, not a library shortcut:
`md.ExpandWalletPolicyChunks(chunks)` then `complexAddressSource(chunks, keys)` from package `gui`, then `at(index, false)` for receive and `at(index, true)` for change. If `complexAddressSource` returns false, that is `stage: "source"` with a reason, NOT a crash — a policy the device cannot derive from is a RESULT, and one of the things the harness exists to count.

`complexAddressSource` is unexported in `gui`. Do not export it and do not copy it: add a tiny exported wrapper in `gui` beside it (`gui.PolicyAddressSource`) whose doc comment says it exists for `cmd/policyprobe` and carries no logic of its own. Copying the function would make the harness measure a copy, which is the one thing it must not do.

The device's policy address path is MAINNET-ONLY by design (`policy_address.go`, D1). Do not add a network flag, do not parameterise it; the driver knows to compare mainnet against mainnet.

## Rules
Fork repo `/scratch/code/shibboleth/seedhammer`, worktree `git -C /scratch/code/shibboleth/seedhammer worktree add -b policyprobe /scratch/code/shibboleth/.tmp/seedhammer-policyprobe main`. Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; `TMPDIR=/scratch/code/shibboleth/.tmp`. Your files: `cmd/policyprobe/main.go`, one wrapper in `gui/` (new file `gui/policy_address_export.go`), and a test for the wrapper. Touch nothing else. `gofmt` clean; `go build ./...` clean; `go vet ./cmd/policyprobe/` clean; the whole `gui` shard set must still pass via `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.

Prove the tool works on a case you generate yourself from the vendored vector `keyed_compose_wsh_timelock_hashlock`: its `md/testdata/vectors/keyed_compose_wsh_timelock_hashlock.phrase.txt` holds the md1 chunks and its `.conformance.json` holds the receive addresses the Rust primary derives. Your tool must reproduce those addresses. Quote the command and its output in your report.

Commit per deliverable with `git commit -s -F <file>`, trailers `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA`. Stage paths explicitly; nothing pushed; no commits on `main`; no sub-agents; never read any `.jsonl`.

FINAL ACTION: write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/policy-differential-fork-leg.md` (create; must not exist) with the tool's exact I/O contract as built, the vector reproduction, every gate's output, and any deviation. Return two lines plus the path and your branch tip.
