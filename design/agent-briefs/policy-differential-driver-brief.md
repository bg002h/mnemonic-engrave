You are the implementer for the DRIVER of a policy differential harness, in `mnemonic-engrave`.

## What it is for
Nothing generates random wallet POLICIES and pushes them through the whole seam — compose, encode to md1, decode back, seat keys, derive an address — and compares what three INDEPENDENT implementations say. The md1 decoder is already fuzzed four ways and the codec has property tests on its primitives; the policy space is untested, and every finding of the session that prompted this came from that seam.

The three answers you compare:
1. **Rust**, the primary: `md` at `/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md`.
2. **The device**, Go: `cmd/policyprobe` in the seedhammer fork, built by another implementer. Contract: JSONL in, JSONL out, one line per case, `{"id","chunks":[...],"indices":[...]}` to `{"id","ok","keys","receive":[...],"change":[...]}` or `{"id","ok":false,"stage","error"}` where stage is `expand|source|derive`. It is MAINNET-ONLY by design.
3. **Bitcoin Core**, which shares no code with either and is therefore the only oracle that can catch the two constellation halves agreeing and both being wrong.

## Your deliverable
`scripts/policy-differential.py`, plus whatever small files it needs under `scripts/`. It must:

**Generate** policies bounded by the composer's own limits (`ComposeMaxPaths` 8, `ComposeMaxKeysPerPath` 9, `ComposeMaxSlots` 32 — read them from the fork's `md/compose.go`, do not hardcode a guess). Vary: wrapper (`wsh`, `tr`, `sh-wsh`, `sh`), path count, k-of-n per path, lock kind (none, `older=N` blocks, `older=Nu` units, `after=H` height, `after=Tt` time), and the presence of a `sha256` hashlock. Seeded and deterministic: the same `--seed` must reproduce the same run exactly, and the seed must appear in the report.

**Run each policy** through: `md compose` → template; `md encode` with seated keys → md1 chunks; `md address --network mainnet` → Rust addresses; `md descriptor --network mainnet` → the concrete descriptor for Core; the device tool → device addresses; Core's `getdescriptorinfo` and `deriveaddresses` → Core addresses.

**Compare** the three address lists for the first few receive indices AND the change chain. Any disagreement is a finding. So is a policy that one implementation refuses and another accepts — record which, with the refusal text.

**Shrink** a failing case: reduce path count, then fragments, then key counts, until it stops failing, and report the smallest still-failing policy. A finding nobody can reduce to a two-line policy will not get fixed.

**Report** to a file: the seed, the counts (generated, agreed, disagreed, refused-by-whom), and each finding with its smallest reproduction and the exact commands to replay it.

## Bitcoin Core
Not installed. Download 31.1 yourself, verify against the published `SHA256SUMS` before extracting, run it on a THROWAWAY regtest datadir under `/scratch/code/shibboleth/.tmp/`, and **never point anything at `~/.bitcoin`** — that is the operator's live mainnet node on a copy-on-write filesystem, and the `-datadir` flag is what keeps you out of it. Bind an RPC port nothing else holds and pass `listen=0`. Note the network seam: the device is mainnet-only, Core on regtest wants testnet-version keys, so compare Core against the SAME network you asked Rust for and say in the report which comparisons are mainnet and which regtest. A `bitcoind` started with `-daemon` from a tool call gets reaped; start it as a tracked background process instead, and stop it when done.

## Rules
Repo `/scratch/code/shibboleth/mnemonic-engrave`, worktree `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add -b policy-harness /scratch/code/shibboleth/me-worktrees/policy-harness master`. Your files live under `scripts/` and `design/`. Do not modify either other repo; the fork is read-only to you. No sub-agents. Never read any `.jsonl` transcript.

If the device tool is not built yet, write against the contract above and say so; the controller wires the two legs together.

Commit with `git commit -F <file>`, trailers `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA`.

FINAL ACTION: write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/policy-differential-driver.md` (create; must not exist) with the harness's usage, what a run of at least 100 policies found, the seed that reproduces it, and every deviation. Return two lines plus the path and your branch tip.
