# F-642 review: mnemonic-toolkit `f642-md-codec-0.47` (53457147..0a3c629f), adversarial

**Verdict: 0 Critical / 1 Important / 4 Minor. Not ready to ship.** The Important is a gap in the restore refusal. `restore --md1` refuses a wire-kind-1 (Liana) card only in the keyed wallet-policy route. The keyless-template completion route, which `restore` and `verify-bundle` share, has no Liana gate. Under `--search-address` that route reports a false **"✗ NO MATCH"** (exit 4) when the supplied keys are correct.

Reviewer: opus, independent. Binaries were built with pinned 1.85.0 and `CARGO_TARGET_DIR=/scratch/code/shibboleth/tk-worktrees/f642-review-target`:
- `mnemonic 0.104.0` from the worktree.
- `md 0.19.0` from `git archive cf35d61a` of descriptor-mnemonic.

The Liana cards below were minted with that md. The keyed one was checked first: `md descriptor --network mainnet` of it reproduces `vendor/md-codec/tests/fixtures/liana/cases.json` case `preset-kofn-recovery-tr` byte for byte (`…#8jc8gq6v`, receive `bc1pj6davmeet…`). The worktree was left clean (`git status --short` is empty).

## Important

### I-1. The Liana refusal is missing from the shared template-completion core. `--search-address` falsely reports NO MATCH, and the other modes refuse with exit 1 and developer-facing text

**Where.**
- The branch's refusal lives only in `classify_taproot_restore` (`crates/mnemonic-toolkit/src/cmd/restore.rs:3240-3249`). Only `run_multisig` reaches it, and that is the KEYED wallet-policy route.
- A KEYLESS multisig/general template md1 (n≥2) takes a different path:
  - `restore.rs:369-371` sends it to `run_multisig_template_completion` → `complete_multisig_template`.
  - `verify-bundle` reuses the same core: `verify_bundle.rs:977` and `:1030`.
- That core's L9 refusal block (`restore.rs:1741-1756`) gates hardened use-sites and unrestorable override cards, but not `InternalKey::LianaUnspendable`.
- Every candidate therefore goes through `candidate_descriptor_string` → `faithful_multisig_descriptor` → `to_miniscript_descriptor_multipath` (`restore.rs:3679`). Since md-codec 0.47.0 that call returns `Err(NetworkRequiredForUnspendable)` for any v8 tree (`vendor/md-codec/src/to_miniscript.rs:306-307`).
- In address-search mode the evaluator turns that `Err` into "not this assignment" (`restore.rs:2336`, `let Ok(desc_str) = candidate_descriptor_string(..) else { return false; }`). No candidate can ever match.

**Counterexample (run).**
- Keys: fixture keys @0 = `73c5da0a/48'/0'/0'/3'` (abandon…about), @1 = `3f635a63/48'/0'/0'/3'`, @2 = `66d455ea/48'/0'/0'/3'`.
- Cosigner cards: mk1 cards for @1/@2 from `mnemonic bundle --network mainnet --descriptor <the same wallet with a NUMS key>` (`.mk1[1]`, `.mk1[2]`).

```
T='tr(UNSPENDABLE(liana),{multi_a(2,@0/48'"'"'/0'"'"'/0'"'"'/3'"'"'/<0;1>/*,@1/48'"'"'/0'"'"'/0'"'"'/3'"'"'/<0;1>/*),pk(@2/48'"'"'/0'"'"'/0'"'"'/3'"'"'/<0;1>/*)})'
md encode --force-chunked "$T"          # keyless template md1 (v8)
md address --network mainnet <keyed encode of $T>   # -> bc1pv7xd8qs3lfvvrarulyehpzleuyj0qsnrsm00xsftdv2pt2fl03xq5yy4k3
mnemonic restore --md1 <template> --from "phrase=abandon … about" --allow-argv-secret \
  --origin "m/48'/0'/0'/3'" --cosigner <mk1 @1 chunks> --cosigner <mk1 @2 chunks> \
  --search-address bc1pv7xd8qs3lfvvrarulyehpzleuyj0qsnrsm00xsftdv2pt2fl03xq5yy4k3
```

Observed (branch):
```
  scan complete in 25ms
✗ NO MATCH
error: restore: multisig-template-search mismatch — derived no key→slot assignment of the supplied keys, expected the recorded wallet (--expect-wallet-id / --search-address)
exit 4
```

**Controls.**
- **The keys are correct.** The identical command with the internal key swapped for the NUMS hex (`tr(50929b74…,{…})`, target `bc1pac935…` from `md address`) reports `✓ wallet-id (completed): 9817986e…` and exits 0.
- **Master's behaviour.** On the same Liana template, `mnemonic 0.103.1` stops at decode with `--md1 decode: wire-format version mismatch: got 8, expected 4`. That is an honest refusal. **This is a regression the pin bump introduced.** v8 cards are now readable, so they reach an engine with no Liana gate.

**The other modes of the same core refuse, but not with the CHANGELOG's promise.**
- `--cosigner @N=` explicit mode and `--expect-wallet-id 468e0a1e8d1c8ce01f2eaa984eae9bdc` (the card's own `wallet-policy-id`) both exit **1**.
- Both print `--md1 → descriptor: a wire-kind-1 (Liana unspendable) internal key needs a network to render its derived xpub prefix; call the _with_network entry point instead of guessing mainnet.`
- That is md-codec's internal Display wrapped in `bad()`, not the friendly text.
- The 0.104.0 CHANGELOG says "`mnemonic restore --md1` REFUSES such a card (exit 2) … The refusal names `md descriptor --network <net>`". That is true only for the keyed route.

**Why it blocks.** The tool asserts something false: no key→slot assignment of these keys reproduces the recorded wallet, when one does. An operator holding correct cosigner cards and a correct receive address is told their material does not match their wallet. They then go hunting for a key error that does not exist, and may conclude the backup set is bad. `verify-bundle`'s keyless-template verification calls the same engine (`verify_bundle.rs:977`, `candidate_descriptor_string` at `:1030`), so a correct bundle there gets the same false mismatch. That was not run, because verify-bundle also requires `--mk1` template-stub cards; this is from reading the code.

**Fix direction (not prescriptive).** Refuse `Body::Tr { internal_key: LianaUnspendable, .. }` at the top of `complete_multisig_template`'s L9 block, with the same ModeViolation and text as `restore.rs:3240-3249`. Add a CLI test in `--search-address` mode, which a gate-less build fails with exit 4 "NO MATCH". Consider hoisting the Liana predicate into `taproot_override_classify.rs`, so the keyed and keyless routes share one copy.

## Minor

- **M-1: the restore refusal is not in the manual.** `docs/manual/src/40-cli-reference/41-mnemonic.md` has no mention of the Liana/wire-kind-1 refusal (`grep -in 'liana\|unspendable'` finds only unrelated NUMS lines). Only CHANGELOG documents it.
- **M-2: the new verdict string may be a GUI schema gap.** A JSON consumer that enumerates `verdict` meets `"unreadable_version"`, but only on a path that previously exited 2 with empty stdout. Already filed as F-650, so nothing new here.
- **M-3: a BCH miscorrection can land on the kept-correction branch (F-643's class).** The branch now keeps the correction when a >4-error BCH miscorrection lands on a header version outside {4, 8}. F-643 measured the rate on the order of 1e-5. The toolkit exits **4** (VERIFY-ME), and the manual says to verify with a build that reads the version. That is honest, so it is not blocking. It is noted because the prior exit 2 made this case impossible.
- **M-4: `NetworkRequiredForUnspendable`'s friendly text is unreachable where it fires.** The `friendly.rs` prose ("toolkit bug — use `md descriptor --network`") never shows on the one route that actually produces the error: restore's faithful arm formats `{e}` with md-codec's Display. This is the text half of I-1, and it goes away with I-1's fix.

## Verified sound

- **Lens 1, the 37-site migration.**
  - Every production arm maps `is_nums:true` to `NumsPoint` and `is_nums:false, key_index` to `Slot(key_index)`. The sites are `restore.rs:3219-3231`, `taproot_override_classify.rs:65-68`, `parse_descriptor.rs:684-689`, and `template.rs:146-151` and `:202-212`. These are semantically identical for kinds 0 and real-key.
  - The remaining `Body::Tr { tree, .. }` sites are shape walkers that never read the key: `timelock_advisory.rs:250`, `unrestorable_advisory.rs:255`, `synthesize.rs:347/375`, `bundle.rs:1225`, `restore.rs:3345`.
  - No toolkit constructor emits kind 1.
  - Keyed Liana card, run:
    - `restore` exits 2 with the refusal text and no stdout.
    - `inspect` exits 0 with `template: tr(UNSPENDABLE(liana),…)`, the same as `md decode`.
  - `xpub-search`'s md1 intake reads only slot keys (`descriptor_intake.rs:262-276`).
  - Kind 1 cannot reach single-sig completion. A key-path-only `tr(Liana)` is unencodable even unadmitted ("placeholder @0 not referenced", measured with an md-codec probe). `cli_template_from_tree` needs `tree: None`.
  - I hand-built an unadmitted Liana card with a non-canonical use-site (`md1gq80tgggzskq72tjtxujdlyrk2t`). Decode admits it by design (`encode.rs:246-251`). `inspect` and `repair` handle it the same way md does.
- **Lens 2, exit routing.**
  - All five new variants route to exit 2 (`error.rs:613-627`).
  - `template_admissible`'s `.is_ok()` turns `NetworkRequiredForUnspendable` into a refusal, not an acceptance.
  - `WireVersionMismatch` is still intercepted into FutureFormat.
  - The only success-or-VERIFY-ME output on an md-codec refusal is I-1.
- **Lens 3, `repair`.**
  - The unreadable branch is reached only on a structured `WireVersionMismatch`.
  - It requires exactly one string and a non-empty `correct_chunks` result (`repair.rs:1792-1807`).
  - md1 is the last group emitted, so no later group can fail after its stdout is written (D28 holds).
  - A single stray chunk of a v4 or v8 chunked set exits 2 with "chunk set incomplete" and never reaches the branch (run).
  - `--max-indel 1` on a clean or indel-damaged v12 card exits 2, the same as before.
  - Uppercase input: output is lowercase, not mixed case.
  - Against md 0.19.0 on V12 and V9: the stdout report and both advice lines are identical apart from the `md: ` prefix. The exit is 4 against md's 5, per ruling 8.
  - A Liana v8 card at one error is corrected normally.
  - The JSON shape is unchanged except for the new verdict value.
- **Lens 4, pins.** install.sh, the four doc workflows and the golden's install row all say `descriptor-mnemonic-md-cli-v0.19.0`. `manual-gui.yml`'s v0.11.0 is the separate tier. The golden diff is version strings only.
- **Lens 5, tests.** `cargo nextest run --locked -p mnemonic-toolkit --test cli_repair_unsupported_version --test cli_restore_taproot`: 28 run, 28 passed. `liana_unspendable_internal_key_refuses_not_nums` asserts code 2, no `tr(` on stdout, and the refusal text, so the Nums-fold mutant fails it. Nothing tests the template-completion route with a Liana card, which is why I-1 passes the suite.

ready to ship: no
