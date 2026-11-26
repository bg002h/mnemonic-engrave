# Final pre-merge review: bg002h/seedhammer PR #29 (bug hunt SH#1-SH#6)

Reviewer: Fable 5.2, 2026-11-25. Scope: `git diff origin/main...claude/project-thread-utm1aa` at head `c4529d4` (9 commits, 30 files). Nothing in the clone was edited; probes ran in a scratch copy.

**Verdict: 0 Critical, 1 Important, 4 Minor, 5 Nit.** The six fixes match the Rust rules they port and every pinned address is Rust's. The Important finding is not a defect in the diff: the consent line SH#2 reported as misleading is still printed, unchanged, for a `thresh(k<n)` policy, in Go and in Rust. Under the Rust-primary rule it cannot be fixed in Go first, so it needs a ruling and a Rust-first follow-up before the PR text can say SH#2's class is closed.

## What I ran

- `GOTOOLCHAIN=go1.26.7 go test ./md/ ./mk/ ./address/ ./nonstandard/ ./mt/ ./cmd/... -count=1`: all ok. Scoped `./gui/` run (legacy, no-change-chain, wallet policy, restore, consent, template): ok. I did not run the full gui package (r1 did, 1466/1466) or a TinyGo firmware build; no review has measured firmware size.
- **Pinned addresses against Rust** (`descriptor-mnemonic/target/debug/md` 0.21.1; `crates/md-codec` is identical to origin/main `c8cde234`):
  - all 28 addresses in `gui/legacy_check_address_test.go` are what `md address` prints for the seven legacy cards;
  - the three no-multipath cards' receive addresses match, and `--chain 1` gives `chain index 1 out of range; use-site multipath alt-count is 1`;
  - the four SH#3 strings get the quoted Rust refusals, and the clean-pad twin decodes.
- **New probes** (cards built in the scratch copy, then given to both sides):
  - legacy top-level `wsh(pk)`, `wsh(pkh)` and `wsh(and_v(v:pk,pk))`, shapes the PR's fixtures do not cover: device and Rust addresses agree on both chains;
  - wallet-policy id of a legacy card: Go and Rust agree (`c9548d2e…`). Both give the bare twin a different id (`cae5510e…`), so that is Rust's behaviour, not a port defect;
  - `Check` over a non-key child, and `thresh(k<n)`: findings M-1 and I-1 below.
- **Mutation:** disabling the guard in `address.Change` fails `TestNoMultipathHasNoChangeChain` and the gui consent test. r1 and r2 covered the other fixes.
- **Head commit `c4529d4`** (seam vectors) was pushed after r2 and no review had seen it. The file is byte-identical to mnemonic-engrave `cd269bc` (sha256 `e2a2308d…`), the pin matches, and `./nonstandard/` passes. The only vector change is the 16-key row's origins and its checksum; its two addresses are unchanged.
- Rust comparison by reading: `to_miniscript.rs`, `policy_shape.rs`, `decode.rs`, `validate.rs`, `tlv.rs`, `chunk.rs` (md-codec, origin/main) and mk-codec `bytecode/encode.rs` (mnemonic-key origin/main `113d8b6`). I agree with r1 that SH#1, SH#2, SH#3 (including validator order), SH#4 and SH#6 match.

## Findings

### I-1 (Important): the SH#2 consent line is still printed for `thresh(k<n)`, in Go and in Rust

`md/policy_shape.go:435-445` (`collect`, `tagThresh` arm) walks every child of a `thresh` as a conjunction. The comment at `md/policy_shape.go:217-220` calls this "honestly described by collect". Rust is the same: `policy_shape.rs` `Tag::Thresh => children.iter().all(|c| collect(c, br, keys))`. A `thresh` with k below n is a disjunction, the same thing the PR now refuses for `or_*`/`andor`.

**Reproduced.** Card for `wsh(thresh(2,pk(@0),s:pk(@1),sln:older(52560)))`, the textbook decaying 2-of-2. Rust decodes it and derives `bc1qc2uxjzn3x6xv3g3aty338tnc6c76eyx9s53sjjkyvegjejvcncuqlcgk9u`. Its template through `templateConsentLines` (`gui/multisig_build.go:808`, summary built at `gui/template_engrave.go:187`) shows:

```
COMPLEX POLICY (advanced)
Script: general miniscript
Key slots: 2
Spend path: 1
  1: 2 key(s), custom +timelock
```

That is character for character the line SH#2 was filed for, and it is wrong in the same way: after the lock either key spends alone. `thresh(1,pk,s:pk,s:pk)` (any one of three) prints `3 key(s), custom`, identical to the 2-of-3 corpus vector `keyed_wsh_thresh`. The same arm is hit when a `thresh` sits inside an or-branch, for example a k-of-n recovery tier written with `thresh`.

**Failure scenario.** An operator is handed a template by someone else and engraves it template-only. The consent says one path with two keys and a timelock; the policy lets one cosigner take the funds alone after the lock. No encoder trick is needed: this is ordinary miniscript that Rust mints and derives.

**Why it is not fixed here, and what I recommend.** The Rust primary has the same rule, so a Go-only fix would lead Rust. Rust evidence is by reading (the arm above is unambiguous); I did not run a Rust test. The diff is correct for what it ports, so I would not hold the merge on a code change, but three things should happen:
1. File it against md-codec `policy_shape` first (refuse a `thresh` whose k is below its child count, with a vector), then port.
2. Qualify the claims that the class is closed: `md/bits.go:11-12` ("policy_shape refuses a disjunction inside a branch"), the comment at `md/policy_shape.go:217-220`, and r1's Q4 answer ("For the SH#2 class, no").
3. Decide whether the device should fall back to "Cannot fully display on-device" for `thresh` in the meantime. That is a ruling for Brian, since it is the Go port refusing a summary Rust still gives.

### M-1 (Minor, challenges logged `sh-bh1125-check-check-pkk-type-refusal`): the whole fallthrough of the `tagCheck` arm is wrong whenever it is reached, not only for `Check(Check(..))`

`md/script_emit.go:228-232`. After the new idempotence branch, the remaining path emits the child and appends `OP_CHECKSIG`. On the md1 wire a bare `PkK`/`PkH` is always `c:pk_k`/`c:pk_h` (type B), Rust re-wraps every one, and Rust refuses `RawPkH` outright. So no K-typed child can exist, and Rust refuses every tree that reaches this path.

**Reproduced.** `wsh(Check(or_i(PkK@0,PkK@1)))`, which Rust's own comment calls "shape C": Rust says `c:or_i(...) cannot wrap a fragment of type B`; the device takes the complex route and shows `bc1q02je4cma4e537jw0k89g52sjq3pz0uflvv8ga9xlcdgdu3klyrssf625k3` with a full receive and change proof on the Wallet Policy consent. `IF <A> CHECKSIG ELSE <B> CHECKSIG ENDIF CHECKSIG` cannot be satisfied. `wsh(and_v(v:Check(Check(pk)),pk))` behaves the same.

**Grade.** I keep Minor: it needs a hand-built or foreign-encoder card, the same basis on which SH#3 was graded Low. But the follow-up's fix direction (refuse only when the child is itself `tagCheck`) is too narrow, and `md/legacy_check_test.go:236-245` pins the unspendable bytes as the wanted output. The full convergence is one line: after the bare-key branch, return `ErrScriptUnsupported`. It is a convergence fix, in the arm this PR rewrote, on the screen that shows addresses, so I would fold it now and flip the test to expect a refusal.

### M-2 (Minor): `md/bits.go` says every known lag has a follow-up; two do not

`md/bits.go:17-27`.
- The `plainMulti` lag (md-codec 0.48.0's `nkeys == slots.len()` guard) has no fork-side entry. F-661 in engrave's FOLLOWUPS is the Rust-side item. I confirmed the lag: `wsh(multi(2,@0,@0,@1))` reports `K:2 N:3 Keys:2`, so the template consent would print `2-of-3` over two distinct keys. Rust already has the guard, so the port is a one-line convergence at `md/policy_shape.go:274` and could ride with this PR.
- The "NOT AUDITED" block (the rest of 0.48.0-0.48.7) has no entry either. A pin that says "not audited" with nothing tracking the audit is the state SH#5 was filed against.

### M-3 (Minor, agree with logged `sh-bh1125-address-list-no-change-chain-toggle`; raise its priority)

`gui/address_polish.go:77`. The grade is right. But it is a behaviour this PR introduces on a common input: any imported descriptor with a fixed chain step (`.../0/*`) now shows `address: no change chain (no key has a <0;1> multipath step)`, a developer string, and drops the operator out of the receive list as well. "End-of-project minors" is too late for operator-facing text on the address screen; I would fix it in the next fork PR.

### M-4 (Minor): the vendored seam vectors come from an unmerged engrave commit

`nonstandard/descriptor_seam_test.go:42`, `nonstandard/testdata/descriptor_seam_vectors.json`. `cd269bc` is on engrave's PR 68 branch, not on `master` (master still has `71bba6f2…`). If PR 68 takes another fold or is squash-merged, the fork pins bytes that no engrave master commit holds. Merge PR 68 first and confirm master's file hashes to `e2a2308d…` before merging #29.

### N-1 (Nit): the mk csid corpus was not re-vendored with the 0.6.0 pin

`mk/chunk_set_id_parity_test.go:18,36` still vendors the "mk-codec 0.5" corpus, while `mk/mk.go:7-11` now pins Encode to 0.6.0. mk-codec 0.6.0 regenerated that file. The only difference is `family_token`, and the Go tests pass against the 0.6 file (checked in the scratch copy), so this is a two-line re-vendor.

### N-2 (Nit): "Not found in the first 20 receive or change addresses" when no change chain was scanned

`gui/verify_address.go:148`, after `address/address.go:111-115`. For a descriptor with no change chain, `Find` now returns after the receive scan, but the screen still says change addresses were checked. An operator verifying a real change address from the sibling `/1/*` descriptor reads it as a full miss. Say "receive addresses" when `HasChangeChain` is false.

### N-3 (Nit): two spellings of one statement

`gui/wallet_policy.go:370` (`noChangeChainLine`) and the literal at `gui/multisig_restore.go:175`. r1 asked for one helper across the three surfaces; two share it and the restore document has its own string. "No change chain" is also jargon for an operator; "This wallet has no separate change addresses" would read better.

### N-4 (Nit): `cmd/policyprobe` discards the receive list for a no-multipath card

`cmd/policyprobe/main.go:247-253`. The probe treats `ErrNoChangeChain` as a derive-stage failure and returns no addresses, so the differential harness cannot compare the receive chain of an `@N/*` card. It should record receive and report the change chain as absent.

### N-5 (Nit): `address.Change` on a keyless descriptor

`address/address.go:24`. The `len(desc.Keys) > 0 &&` guard sends a keyless descriptor on to `addressAt`, which `Find`'s own comment says panics on one. No caller does this today; returning `ErrUnsupported` there would make the new exported entry point total.

## Logged items I reviewed and agree with

`sh-bh1125-mk-encode-stale-comment`, `sh-bh1125-derived-csid-vs-me-cli-mk-codec-060`, `sh-bh1125-pkh-tap-leaf-described-as-pk`, `sh-bh1125-legacy-sh-pk-no-address` (confirmed: legacy `sh(wsh(pk))` and `sh(pk)` give "This device can't derive addresses" while Rust derives; fail-closed) and `sh-bh1125-composer-no-change-chain-untested` are graded correctly.

## Recommended disposition

- Rule on I-1 (file Rust-first; qualify the three claims; decide on an interim device fallback).
- Fold M-1 and the `plainMulti` half of M-2 if another commit is going on anyway; both are one-line convergences with Rust.
- Merge engrave PR 68 before this PR (M-4).
- Log M-3 with a nearer owner, and the Nits.

Probe outputs (the generated cards and the device lines), with the surviving probe sources: `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable-probes/`.
