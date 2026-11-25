# Review r1: bg002h/seedhammer PR #29 (bug hunt SH#1-SH#6)

Reviewer: independent adversarial review (Opus 5.5), 2026-11-25.
Scope: `git diff origin/main...claude/project-thread-utm1aa` (6 commits, 7b9720e..dde21fe), 26 files.
Rust primaries: descriptor-mnemonic `origin/main` c8cde234 (md-codec 0.48.7; its working tree has local
edits under `crates/md-cli` and docs only, and `crates/md-codec` matches origin/main, so the `md` binary's
codec verdicts are origin/main's). mnemonic-key `origin/main` 113d8b6 (mk-codec 0.6.0, unreleased).

**Verdict: 0 Critical, 1 Important, 2 Minor, 4 Nit.** All five fixes match the Rust rules they port. The
Important finding is a regression the SH#6 fix causes in the Wallet Policy consent screen, which no test
covers.

## What was run

- `GOTOOLCHAIN=go1.26.7 go test ./md/ ./mk/ ./address/ ./mt/ -count=1`: all ok.
- Full `./gui/` suite in 16 shards with `mnemonic-engrave/scripts/gui-shard-test.sh` (it checks that the
  partition covers every test): **1466/1466 ok**, 215 s wall. Every other package: ok.
- `gofmt -l`: only the known five-file baseline (`gui/transaction*.go`, `mt/mt.go`, `mt/mt_test.go`).
  `go vet` reports only the two `testing.ArtifactDir` go1.26 notices in golden tests, which were there
  before this PR.
- **Rust cross-checks** with `descriptor-mnemonic/target/debug/md` (md 0.21.1):
  - All 7 `md/testdata/legacy_check/*.md1.txt` cards: `md decode` gives the expected template, and
    `md address --count 2` and `--chain 1` give exactly the 28 addresses pinned in
    `gui/legacy_check_address_test.go`.
  - All 3 no-multipath cards in `gui/no_change_chain_test.go`: the Rust receive addresses match the pinned
    ones, and `--chain 1` gives `chain index 1 out of range; use-site multipath alt-count is 1`.
  - All 4 SH#3 refusal strings: Rust prints the quoted error text. The clean-pad twin decodes to
    `wpkh(@0/<0;1>/*)`.
- **Mutation checks** on a copy of the tree in the scratchpad (the repo itself was not touched):
  - Reverting `md/script_emit.go` fails `TestLegacyCheckOverKeyEmitsTheBareScript` and
    `TestCheckIdempotenceIsOnlyForABareKey`.
  - Reverting `md/tapleaves.go` fails `TestCheckIdempotenceIsOnlyForABareKey` (the describe half).
  - Reverting `md/policy_shape.go` fails both policy-shape tests.
  - Reverting `gui/policy_address.go` fails `TestNoMultipathCardHasNoChangeChain` on its or_b and tr
    subtests.
  - The tests do assert what they claim.

## Q1: does each fix match Rust?

- **SH#1, matches.** `emitFragment`'s `tagCheck` arm (`md/script_emit.go:212-227`) unwraps only when the
  child is `tagPkK`/`tagPkH` with a `keyArgBody`. That is exactly Rust's
  `matches!((&children[0].tag,&children[0].body),(Tag::PkK|Tag::PkH, Body::KeyArg{..}))`
  (`to_miniscript.rs:780-785`). `v:`/`s:` over the unwrapped key still fold correctly, because
  `CHECKSIG`→`CHECKSIGVERIFY` happens on the emitted bytes. The vectors cover pk, s:pk, pkh, v:pkh, a tap
  pk leaf and v:pk in a tap leaf, and all match Rust.
- **SH#2, matches.** `collect`'s disjunction arm (`md/policy_shape.go:446-483`) is Rust's
  `policy_shape.rs` rule as of 0.48.5:
  - an `or_i` with any `False` child counts as a zero arm;
  - an `andor` whose child 2 is `False` counts as a zero arm;
  - a body that is not children refuses;
  - every other disjunction returns false.
  - `splitBranches` is unchanged and identical to Rust's `split_branches`.
  - The composer emits `or_d`/`or_i` only as the top-level chain (`md/compose.go:739-751`), which
    `splitBranches` splits, so composer self-check and consent are unaffected (full gui suite green).
- **SH#3, matches, including order.** Rust's `decode_payload_with_opts` runs these in order:
  1. placeholder usage;
  2. when overrides are `Some`: multipath consistency, then use-site overrides canonical;
  3. tap tree;
  4. no empty origin overrides;
  5. explicit origin;
  6. xpub bytes.

  `decodePayloadValidated` (`md/md.go:1239-1272`) runs the same sequence. `useSitePathEqual` matches
  Rust's derived `PartialEq` on `UseSitePath`, since `alternative` is a comparable struct. The
  `rejectNonZeroPad` call sites match `tlv.rs` at both break points, peek-and-restore included. The check
  sits in `readTLV`, so it covers the chunked path (`chunk.go:279`, `len*8`, as Rust's `chunk.rs:451`) and
  the single-string path (`5*len(syms)`).
- **SH#4, matches.** `checkKeyRules` (`mk/encode.go:84-124`) is R2/R3/R4 of `check_key_rules`
  (`encode.rs:123-155`):
  - R2 is unconditional;
  - R3 compares against hash160(compressed pub)[:4], exempting an all-zero chain code with fp 00000000;
  - R4 compares against the parent fp;
  - depth ≥2 carries no rule.

  It runs after the structural checks, as in `validate_card`. `DerivedChunkSetID` skips it, as Rust's
  `encode_canonical_payload` does. Every device-side mint pairs the master fp with its own derived key
  (`derive_xpub.go:447`, `multisig_derive.go:49`, `singlesig_derive.go:84`), so normal flows are
  unaffected.
- **SH#6, matches the rule.** `HasChangeChain` matches `check_chain_in_range`: a chain exists if any key
  has a multipath. A key without one ignores the chain, as Rust's `use_site_to_derivation_path` returns an
  empty path for it. `useSiteToChildren` maps a bare `*` to `[Wildcard]`, never to an empty Children, so
  the default `<0;1>/*` cannot leak in.
- **Divergences that remain** (Go accepts or shows something Rust refuses):
  - `Check(Check(PkK))` (M-2 below; it existed before this PR and is disclosed);
  - the lags listed in `md/bits.go`.
  - I found no case where the PR makes Go refuse a card Rust accepts at the codec layer, or shows a
    different address.

## Findings

### I-1 (Important): SH#6 makes the Wallet Policy consent screen say "Do not engrave this card" for a valid card

`gui/wallet_policy.go:362-369` (`walletPolicyAddressLines`) loops over Receive then Change. On **any** error
from `at` it replaces the whole address block with:

```
Address derivation failed.
Do not engrave this card.
```

After this PR, `policyAddressAt` returns `address.ErrNoChangeChain` for chain 1 on both routes when no key
has a multipath: the flat route through `address/address.go:24`, the complex route through
`gui/policy_address.go:201-209`. Only `multisigRestoreLines` was taught to handle it (`firstChange`).

**Reproduced.** In a scratch copy, a test called `walletPolicyAddressLines` on the PR's own two keyed
no-multipath cards, `wsh(sortedmulti(1,@0/*,@1/*))` and `wsh(or_b(pk(@0/*),s:pk(@1/*)))`. Both returned
exactly `["", "Address derivation failed.", "Do not engrave this card."]`.

**Failure scenario.** An operator loads a template with `@N/*` keys plus its mk1 cards on the Wallet
Policy route. Rust's `md encode` mints such templates and Rust derives their receive chain. Before this PR
the consent screen showed the correct receive addresses (and mislabelled duplicates as change). Now it
shows no address proof at all, and tells the operator that a card Rust accepts is bad. That is a false
statement on a consent screen. The same pattern sits at `gui/composer_consent.go:337-343` (an error ends
consent). It cannot be reached today, because the composer always mints `<0;1>/*` (`md/compose.go:894`),
but it is one change away.

**Fix.** Treat `ErrNoChangeChain` on both consent loops as `firstChange` does:
- keep the Receive lines;
- print `Change: none (no change chain)`;
- add a test that drives `walletPolicyConsentLines` with a no-multipath template.

Better still, put the "no change chain" rendering in one helper that all three surfaces use.

### M-1 (Minor): the address-list screen leaves entirely when the operator toggles to "change" on a no-change-chain policy

`gui/address_polish.go:72-75` (`recompute`) calls `showError(ctx, th, "Address", err.Error())` and the list
flow returns. This is reachable from `md1_inspect.go:173` (md1 cards with `@N/*`). It is also reachable
from `gui.go:3339` for any imported descriptor with a fixed chain step, such as `wpkh([fp/84h/0h/0h]xpub/0/*)`,
which is a common export spelling.

Before the PR the operator saw receive addresses titled "Change addresses", which is a mislabel. Now they
see a developer string, `address: no change chain (no key has a <0;1> multipath step)`, and lose the
receive list as well. This fails closed and is not a funds risk.

**Suggestion:** disable the toggle, or show "No change chain" in place, when `HasChangeChain` is false.

### M-2 (Minor): a test pins Go's unspendable script for `Check(Check(PkK))`, a tree Rust refuses

`md/legacy_check_test.go:236-241` asserts that `Check(Check(PkK))` emits `<K> CHECKSIG CHECKSIG` as the
wanted result. Rust's `to_miniscript.rs:777-779` refuses it at type-check ("never a wrong descriptor").

**Failure scenario.** A hand-built or foreign-encoder card with `c:` over a key that is already
`Check`-wrapped. The device shows a well-formed address for a script nobody can spend, while every Rust
tool refuses the card. This existed before the PR (pre-PR output: `CHECKSIG CHECKSIG CHECKSIG`) and is
disclosed in `md/bits.go`.

The problem is that the new test makes the wrong output a required one, so a later convergence fix will
look like a regression. **Suggestion:**
- In the `tagCheck` arm, return `ErrScriptUnsupported` when the child is itself `tagCheck`. That is a
  narrow subset of Rust's type-B refusal, and safe because no encoder writes it.
- Or at least assert "not the bare script" instead of pinning the exact bytes, and file a FOLLOWUP for
  the missing type check.

### N-1 (Nit): a comment in `mk/encode.go` is now stale

`mk/encode.go:129-131` still says "Encode itself is untouched: Encode is still the two-line composition at
:39-45". Encode now calls `checkKeyRules` and the line numbers have moved.

### N-2 (Nit): `DerivedChunkSetID` follows a different mk-codec function than its docstring cites

The docstring (`mk/encode.go:133-136`) says it matches me-cli's
`derive_chunk_set_id(encode_bytecode(card))`. Under mk-codec 0.6.0, `encode_bytecode` applies the key
rules, so once me-cli moves off `mk-codec 0.5.0` (its `Cargo.lock`), its `.ok()?` will return None and skip
the csid warning for a legacy card. Go follows `encode_canonical_payload` instead, which is the right
choice, but the two will then diverge. The comment should name `encode_canonical_payload`, and a
cross-repo FOLLOWUP should cover the me-cli side.

### N-3 (Nit): a pkh tap leaf is still described as a pk leaf

`describeTapLeaf` (`md/tapleaves.go:107-113`) still describes a `PkH` leaf (and now `Check(PkH)`) as
`TapLeafPK`. `TapLeavesChunks`/`tapLeafSpecs` have no production caller (only tests), so no address is
affected. This predates the PR.

### N-4 (Nit): legacy `sh(pk)`/`sh(pkh)`/`sh(wsh(pk))` cards get no address on the device

The toolkit 0.55.0 changelog lists these as legacy shapes. `EmitWitnessScriptChunks` refuses any root that
is not `wsh` (`md/script_emit.go:105`), so the device shows "can't derive" while Rust derives. This fails
closed and predates the PR. It should be a FOLLOWUP, so that "legacy wire converged" is not read as
covering these shapes.

## Q2: test integrity

- **Pinned addresses are Rust's.** The PR pins 28 legacy-card addresses, 6 receive addresses on the
  no-multipath cards, and 4 SH#3 verdicts. I re-ran every one with the Rust `md` binary and all match.
  The legacy cards also cross-check against the corpus vectors' recorded addresses.
- **The fixtures are what they claim.** `md/legacy_check_test.go` regenerates each legacy fixture with
  `wrapKeyChecks` and requires the bytes to be identical.
- **No false PASS paths found.** Each fix's test fails when that fix is reverted (mutation results above).
- **One gap:** no test drives `walletPolicyAddressLines` or `composerConsentLinesFor` with a
  no-change-chain policy, which is where I-1 sits.

## Q3: regressions in other callers

- `address.Change`:
  - `singlesig_restore.go:102`: always `<0;1>/*`, safe.
  - `multisig_restore.go`: handled.
  - `address_polish.go`: M-1.
  - `wallet_policy.go:389` through the consent screen: **I-1**.
- `address.Find` (`verify_address.go:138`): it now skips the change scan when there is no change chain.
  Before, that scan was a duplicate of the receive scan, so the result is identical.
- `mk.Encode`:
  - device-derived mints are unaffected;
  - the re-mint paths (`reStubMk1`, `composer_cards.go:53`) now refuse an imported card that breaks a key
    rule, which is the intended Rust parity, and the error propagates.
  - `TestKeyRulesLeaveLegacyCardsReadable` covers decode and `DerivedChunkSetID` on a legacy card.
- `describeTapLeaf`: only test callers.
- The full gui suite is green.

## Q4: can the consent screen still show an incomplete or misleading summary?

For the SH#2 class, no:
- An `or` or `andor` anywhere inside a branch (under and_v, a wrapper or thresh) now makes the summary
  incomplete.
- `policySummaryLines` (`gui/template_engrave.go:151-154`) and the composer consent (`gui/composer_consent.go:171`)
  then fall back to "Cannot fully display on-device".

What remains is the same in Go and Rust: a top-level `l:`/`u:` (`or_i` with a `0` arm) is split into a
"0 key(s), custom" path that can never be satisfied. It is odd but not dangerous. The new misleading
surface this PR adds is the false "Do not engrave this card" in I-1.

## Recommended disposition

- Fold I-1, with a test on `walletPolicyConsentLines`.
- Fold M-2, either the narrow `Check(Check(..))` refusal or relaxing the pin.
- Log M-1 and N-1 to N-4 to the follow-up file.
- Then a re-review scoped to the fold.
