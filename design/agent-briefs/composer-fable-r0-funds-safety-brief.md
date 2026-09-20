# Lens 1 — FUNDS SAFETY and CORRECTNESS of policies constructed on the SH2

Read `composer-fable-r0-common.md` first. Your worktree slug: `funds`.
Your regtest ports: `-rpcport=18543 -port=18544`.
Your report: `design/agent-reports/composer-fable-r0-funds-safety.md`.

## THE ONE QUESTION

Construct a policy on the SH2 composer — through the grammar the device
admits (§4) and the flow it walks (§7) — whose engraved artifact, decoded on
the host with `md 0.16.2`, yields a wallet whose **spending conditions or
addresses differ from what the device's screens told the operator**, or
whose funds can be **lost, locked, or spent by fewer parties than shown**.
Find one, or show by execution that none exists in the grammar's corners.

The screens are the promise: the shape/review/consent copy (§8), the k-of-n,
the lock kind and value, the hash, the key order, the NUMS note (§8f), the
same-seed note (§8g), the all-hash warning (§8h). The script is the fact.
The delta between them is your finding.

## Method — execution, not reading

1. Generate policies at the layer the device uses: `md.Compose` /
   `md.ComposeWith` (`md/compose.go:493,503`) for breadth, and the gui harness
   (`gui/composer_*_test.go`, `gui/wallet_policy_descriptor_walk_test.go`)
   for the screens-vs-artifact comparison on the shapes that matter most.
   `cmd/policyprobe` and `cmd/buildpayloadcomposer` may save you time.
2. Decode the md1 the device would cut with the Rust `md` CLI (that is what
   the operator does; treat `md` as the decoder without auditing it — lens 3
   audits the encode/decode layer). Get the descriptor text.
3. **Bitcoin Core is the oracle for semantics.** On regtest:
   `getdescriptorinfo`, `importdescriptors`, `deriveaddresses`; then FUND and
   SPEND under each branch to prove the conditions — `older(n)` unspendable
   before n blocks and spendable after; time-type `older` (0x400000+u)
   correct in 512-second units; `after(h)` at both sides of the boundary; the
   hash path needs the preimage and only the preimage; k-of-n needs exactly
   k; the taproot key path is NUMS when the note says so (no key-path spend
   possible); `or_d`/`or_i` chains reachable on every path. Core v25 lacks
   `tr()` miniscript and multipath — use nix Core for those, and say which
   Core produced each result.
4. Cross-check addresses: `md` / `mnemonic` derived addresses vs Core's
   `deriveaddresses` for the first few indices on both chains.

## Hunt list (cover these; add your own)

- §4c boundaries: `older(1)`, `older(65535)`, `older(0x400000+1)`,
  `older(0x400000+65535)`, `after(499999999)`, `after(500000000)`,
  `after(2147483647)`, the date-entry floor 2009-01-03 — and what the
  digit pad does at each edge (§6b): can a masked or zero-units operand
  reach the artifact? §12 item 7 says a test fails if so — run it, then try
  to get past it.
- Every preset (§4d) under `wsh` and `tr`, then EDITED (a preset the
  operator changes is the common case): does the edited shape's screen copy
  still match the emitted script?
- `sh(wsh)` / `sh` single `sortedmulti` (the Multisig migration, C7): key
  ORDER — `sortedmulti` sorts; the screen numbers slots by first appearance
  in emitted text (`md/compose.go` header comment). Can the operator's "@1"
  be a different key than they think when it matters (multi vs sortedmulti;
  `multi_a` under tr)?
- Keyless `wsh` paths (C16, EXPERIMENTAL): hash-only and hash+lock; a
  lock-only path is REFUSED (anyone can spend) — prove the refusal refuses
  through the flow, not only in `ValidatePathList`.
- All-hash policy (§8h): funds recoverable by whoever holds the preimage —
  is the warning shown BEFORE consent on every route to consent?
- Same seed in one path reaching threshold (§8g, C29): does the notice fire
  on both bodies, and is the resulting wallet exactly what the notice says?
- Slot ceilings: n=9 per path, 8 paths, 32 total; `multi` ≤ 20 under wsh —
  what happens at 21? (A wsh path with 9 keys is fine; several paths can
  each hold 9; the wire's 5-bit `path_decl.n`.)
- `tr` with a single-key head path vs bare-multi head — the `or_d` vs
  `or_i` choice and the taptree spine (right-leaning): is every leaf
  reachable and is the internal key the one the screen names?
- Timelock mixing (C11): one lock per path — can `and_v` chains ever mix a
  height and a time lock through path chaining? Prove with Core's
  `getdescriptorinfo` on the widest composition you can reach.
- Network: the device's chain setting vs the xpub version on the artifact
  vs the address the host derives.

## Already settled for this lens (do not re-derive)

- The two lowering expert reviews closed on the SPEC's rules. If the code
  departs from §5, that is a finding; if you disagree with §5 itself, that
  is an out-of-lens note.
- The same-fingerprint admission recon is done (common file).

## Deliverable specifics

Beyond the common report structure, include a **matrix**: one row per
shape you executed — wrapper, paths, locks, hash, k-of-n — with columns:
screen promise / descriptor / Core version / addresses match (Y/N) / each
branch spend-tested (Y/N/which). Rows you could not execute are marked and
the reason given, so the controller knows the coverage, not just the
verdict.
