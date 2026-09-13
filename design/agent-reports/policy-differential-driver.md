# Policy differential harness — driver leg

**Agent**: driver implementer. **Branch**: `policy-harness`, tip `76732bd8` (worktree
`/scratch/code/shibboleth/me-worktrees/policy-harness`, branched from engrave master
`dd61d5e5`).

Deliverables, all under the worktree:

| path | what it is |
| --- | --- |
| `scripts/policy-differential.py` | the harness: generate → run three legs → compare → shrink → report |
| `scripts/policy_keys.py` | self-contained BIP-32 + network-independent address normaliser |
| `scripts/policy-differential-selftest.py` | mutation-tests the comparator and the safety guards |
| `scripts/policy-keyreuse-adjudicate.py` | asks Core to settle the key-reuse trio |
| `design/policy-differential-seed1.md` | the committed 200-policy run |
| `design/policy-keyreuse-core-adjudication.json` | the adjudication's machine-readable output |

---

## Usage

```sh
# the default run: 200 policies, all three legs, report to design/
scripts/policy-differential.py --seed 1 --count 200 --report design/run.md

scripts/policy-differential.py --seed 1 --count 200 --no-core   # fast loop
scripts/policy-differential.py --seed 1 --count 200 --replay 16 # one case
scripts/policy-differential-selftest.py                         # prove it can fail
scripts/policy-keyreuse-adjudicate.py --json-out /tmp/k.json    # needs bitcoind up
```

Exit status is 1 when an unexpected divergence was found, 0 otherwise, so it drops into
CI as-is. Core is started on demand and stopped when the run ends (`--keep-core` leaves
it up; `--no-core` skips the leg). A run of 200 with all three legs takes ~13 s.

**Generator bounds are read at runtime** from the fork's `md/compose.go`
(`ComposeMaxPaths` 8, `ComposeMaxKeysPerPath` 9, `ComposeMaxSlots` 32) rather than
hardcoded — the point is to generate *at* the boundary, and a stale boundary tests
nothing. Varied per case: wrapper (`wsh`, `tr`, `sh-wsh`, `sh`), path count, k-of-n per
path, lock kind (none / `older=N` blocks / `older=Nu` units / `after=H` height /
`after=Tt` time), `sha256` hashlock, `unsorted`, and keyless paths. Seeded and
deterministic: the same `--seed` reproduces the same stream exactly, and the seed is in
the report.

Findings are **deduplicated by signature**, so one divergence hit by 26 policies is one
entry with a count and one minimal reproduction, not 26 copies; shrinking then runs once
per signature rather than once per case.

---

## What a run of 1000 policies found

Five seeds × 200, all three legs live, against the **fixed** policyprobe
(fork branch `policyprobe-fix`, tip `b12c32a`):

| seed | agreed | unexpected | documented | refused |
| --- | --- | --- | --- | --- |
| 1 | 166 | **0** | 18 | 16 |
| 2 | 170 | **0** | 15 | 15 |
| 3 | 161 | **0** | 27 | 12 |
| 4 | 158 | **0** | 27 | 15 |
| 5 | 159 | **0** | 19 | 22 |
| **total** | **814** | **0** | **106** | **80** |

`--seed 1 --count 200 --indices 3` reproduces the committed run. **814 three-way
agreements** — Rust, the device and Bitcoin Core identical on 3 receive + 3 change
indices each, **4,884 addresses**. **No unexpected divergence at any seed.**

### Deviation 1 — Core refuses signature-less spend paths (106 cases, documented)

Every one of the 106 is the same class: a policy `md` will only encode under
`--experimental` is one Core refuses with `is not sane: witnesses without signature
exist`. `md encode --experimental`'s own help names this rule and says it relaxes exactly
it; Core enforces it and has no flag to relax it. **Not a defect.** It is counted and
named in its own report section rather than dropped, and the suppression is deliberately
narrow — the self-test proves that the same refusal *without* `--experimental`, and a
*different* Core sanity reason, both stay findings.

### Deviation 2 — md refuses 80 policies the composer's own bounds admit

Two kinds, both **outside** what `validate()` checks, so neither is a composer bug —
but the first is worth a ruling:

- **10 × the wire cap**: `encoding requires 65 chunks; max is 64 per spec §9.8`
  (65, 66 and 67 seen). `ComposeMaxSlots = 32` does **not** imply an encodable card:
  the composer validates slot count, and the chunk count that falls out of it can still
  exceed §9.8. A user can compose a policy that cannot be minted, and only finds out at
  `md encode`.
- **5 × miniscript malleability**, refused even with `--experimental` — correct, and the
  message says so plainly.

### Deviation 3 — the three device divergences were artefacts, and are now zero

An earlier run against policyprobe `f51d096` produced three device classes — bare `sh`,
`sh(wsh(…))` and key-path-only `tr`, each shrunk to a one-line policy. Re-running the
identical seed against the fixed build takes all three to **zero**. They were F-512, not
device behaviour. The fixed build is now first in `POLICYPROBE_CANDIDATES` so a stale
binary on the path cannot silently re-manufacture them.

---

## The Core adjudication of the key-reuse trio

`md address` refuses all three with *"@N appears at 2 use sites in this template with the
same path expression"*, citing BIP-388's disjointness rule; the device derives addresses
matching each vector's `.conformance.json` (F-513/F-514). A refusal and a derivation are
not comparable answers, so neither constellation half settles it. **Core splits the
trio** — they are not one class and should not be ruled on together:

| vector | Core | reading |
| --- | --- | --- |
| `keyed_tr_multi_a` | **agrees** with device + vector, every index, both chains | md's refusal is a **guard** |
| `keyed_tr_sortedmulti_a` | **agrees** with device + vector, every index, both chains | md's refusal is a **guard** |
| `keyed_wsh_timelock_hashlock` | **refuses**: `is not sane: contains duplicate public keys` | md's refusal has **independent support**; the device is the outlier |

The mechanism behind the split: a taproot internal key sits **outside** the miniscript, so
`tr(K,multi_a(2,K,K2))` holds no duplicate within any one miniscript expression and Core
derives it happily. In `keyed_wsh_timelock_hashlock`, `@1` and `@2` really are repeated
inside **one** miniscript, across both arms of the `or_i`.

**The sharpest consequence**, and the thing only Core could establish: for that wsh
vector the device derives a receive address for a script **Bitcoin Core will not accept
as sane**. A user could be shown that address and fund it, and a coordinator running Core
would then reject the descriptor. The two taproot vectors carry no such risk — the
dispute there is purely at the BIP-388 wallet-policy layer, where md's *"unsupported"*
wording already matches the standing ruling that key reuse is BIP-forbidden, not invalid.

---

## Bitcoin Core: what was run, and the safety story

Core **31.1**, downloaded and verified against the published `SHA256SUMS`
(`b80d9c3e04da78fb6f0569685673418cf686fadba9042d926d13fb87ff503f9e`) **before**
extracting. Run on a throwaway **regtest** datadir at
`/scratch/code/shibboleth/.tmp/policy-diff-regtest`, RPC port 18988, with `listen=0`,
`maxconnections=0`, `dnsseed=0` — it binds a port nothing else holds, never connects to a
peer and never downloads anything. `getdescriptorinfo` and `deriveaddresses` are pure
functions of their argument, so an empty datadir is sufficient.

`~/.bitcoin` was **never addressed**. The operator's live mainnet node (PID 139471) was
running throughout on the default port and was not touched: every invocation here carries
an explicit `-datadir` and an explicit `-rpcport=18988`, and `_assert_safe_datadir()`
refuses to run at all if the datadir resolves inside `$HOME` or outside the scratch tree.
Verified end to end — `--datadir ~/.bitcoin` exits with a refusal before any RPC. The node
is started detached (`start_new_session=True`) rather than with `-daemon`, so a tool call
ending does not reap it, and `stop()` only ever stops a node the run itself started. Both
the harness node and the one started by hand during development were stopped; port 18988
is free.

### The network seam

The device is mainnet-only; a regtest Core will not parse a mainnet `xpub`. The legs are
therefore not asked the same question in the same units, and the harness does not pretend
they are:

| comparison | Rust asked for | other leg answers in | compared as |
| --- | --- | --- | --- |
| Rust vs device | mainnet (`bc1…`) | mainnet (`bc1…`) | normalised scriptPubKey |
| Rust vs Core | regtest (`bcrt1…`) | regtest (`bcrt1…`) | normalised scriptPubKey |
| Rust vs itself | mainnet **and** regtest | — | normalised scriptPubKey |

`normalize_address` discards the network and keeps the witness program or script hash, so
`bc1q<prog>` and `bcrt1q<prog>` both become `wit:0:<prog>` and the **script** is compared
rather than its encoding. Core is fed the same descriptor with its keys re-serialised to
tpub version bytes — a pure version-byte swap that validates the record is 78 bytes and
carried the mainnet public version first, so a corruption there cannot read as a Core
disagreement.

The third row is a check in its own right and runs on every case: a network flag must
never change a script, and `RUST_NETWORK_VARIANCE` fires if it ever does. It never did.

---

## Notes for the controller

1. **The self-test found a bug in itself, which is the point.** A differential harness
   whose comparator cannot report a disagreement is a false-PASS machine that prints
   `agreed=1000` forever. `policy-differential-selftest.py` makes each leg wrong in turn
   and asserts the finding fires; asserts silence when all agree; proves the **change**
   chain is really compared and not just receive; and proves the documented-divergence
   suppression is narrow. Its first draft hand-wrote a `bcrt1…` address with the wrong
   checksum and three "comparator failures" turned out to be that fixture — the fixtures
   are now encoded from witness programs with the encoder cross-checked against the
   published BIP-173 vectors.

2. **One tautological assertion was found and fixed in my own code.** `to_tpub_descriptor`
   asserted `swapped[4:] == raw[4:]` on a value constructed as `version + raw[4:]` — a
   gate that could not fail. It now validates the 78-byte length and the incoming version
   byte, and the self-test proves both refusals fire.

3. **`md descriptor --network regtest` still emits `xpub`.** The addresses correctly
   become `bcrt1…`, but the descriptor keeps mainnet key version bytes, so it cannot be
   pasted into a regtest Core. That is why this harness converts the keys itself. Not
   pursued — possibly deliberate, but worth a ruling.

4. **The composer admits policies that cannot be minted** (deviation 2, the 64-chunk cap).
   That is the one finding here I would put in front of a user-facing decision: a compose
   that succeeds and an encode that then refuses is a divergence *within* the Rust leg,
   and the composer has the slot count in hand at validate() time.

5. **Coverage this harness does not have.** It compares addresses, not spendability — no
   PSBT is signed and no transaction is broadcast, so a script that derives the right
   address but cannot actually be satisfied would pass. Core's `deriveaddresses` is the
   strongest oracle available without a funded chain, and `keyed_wsh_timelock_hashlock`
   shows the sanity check catches real things, but it is not a spend test.
