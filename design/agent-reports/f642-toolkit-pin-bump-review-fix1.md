# F-642 fix-round review: mnemonic-toolkit `f642-md-codec-0.47` fix commit (0a3c629f..d02382d6), scoped re-review

**Verdict: 0 Critical / 0 Important / 0 Minor. Ready to ship.**

Independent of the implementer. Reviewed strictly inside the fix diff (`git -C
/scratch/code/shibboleth/tk-worktrees/f642 diff 0a3c629f..d02382d6`, 6 files,
335 insertions / 18 deletions). Binaries built fresh with pinned toolchain
1.85.0 (`~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` prepended),
`CARGO_TARGET_DIR=/scratch/code/shibboleth/tk-worktrees/f642-review-target`:

- `mnemonic 0.104.0` from the worktree at `d02382d6`.
- `md 0.19.0` from `descriptor-mnemonic`, confirmed `HEAD == cf35d61a` before
  building (`cargo build --locked --release -p md-cli`, target dir
  `f642-review-target/md-target`).

Worktree left clean (`git status --short` empty, confirmed at the end).

## Per-finding table

| Finding | Status | Evidence |
|---|---|---|
| **I-1** — Liana refusal missing from the shared template-completion core; false NO MATCH under `--search-address` | **ADDRESSED** | Reproduced all four counterexamples from the original review directly against the fixed binary (commands and full stdout/stderr below). All four now exit 2 with the shared refusal wording; none prints `NO MATCH`, `descriptor:`, or md-codec's internal `_with_network entry point` text. |
| **M-1** — refusal not in the manual | **ADDRESSED** | `docs/manual/src/40-cli-reference/41-mnemonic.md:1496-1512` (verified by `grep`/`Read`), new paragraph "Liana unspendable internal key (v0.104.0)". Covers keyed + keyless, all three completion modes, verify-bundle, the two recipe forms, and the `--from-mk1` ambiguity caveat. |
| **M-2** — verdict-string GUI schema gap | **NOT IN SCOPE (unchanged, correctly)** | Already filed as F-650 per the original review; the fix diff does not touch this, and the implementer's notes correctly record "no change." |
| **M-3** — BCH miscorrection on kept-correction branch | **NOT IN SCOPE (unchanged, correctly)** | Unrelated to I-1's completion-engine gap; the original review already judged exit 4 honest and non-blocking. Fix diff does not touch `repair.rs`. |
| **M-4** — `NetworkRequiredForUnspendable`'s friendly text unreachable / divergent wording | **ADDRESSED** | `friendly.rs:469-478`: the arm now returns `taproot_override_classify::LIANA_UNSPENDABLE_REFUSAL` (the same string the refusal uses), not the old "toolkit bug" text. Test needle at `friendly.rs:965-967` updated to `"md descriptor --network"`. Confirmed by running `cargo nextest -E 'test(friendly)'`: 17/17 pass. |

## 1. I-1 counterexample reproduction (all four routes)

Fixture: `TPL_LIANA = md13s9pvqqzqjtvyyyhqqxq79yqs39gq6vdk9zapley66`, keys @0
(seed `abandon…about`, own slot) / @1 / @2 at `m/48'/0'/0'/3'`, true address
`bc1pv7xd8qs3lfvvrarulyehpzleuyj0qsnrsm00xsftdv2pt2fl03xq5yy4k3`, wallet-id
`468e0a1e8d1c8ce01f2eaa984eae9bdc` — the same values the review's counterexample
and the fix's new test file both pin.

**`restore --search-address` (the review's headline counterexample):**
```
$ mnemonic restore --network mainnet --md1 <TPL_LIANA> --from phrase=... --allow-argv-secret \
    --origin m/48'/0'/0'/3' --cosigner <mk1 x6> --search-address bc1pv7xd8...yy4k3
exit=2
error: this md1 carries a Liana unspendable internal key (wire kind 1, md-codec wire
version 8) — the toolkit cannot render it yet, and substituting the BIP-341 NUMS point
would describe a different wallet; use `md descriptor --network <net> <md1…>` to recover
the descriptor from a keyed card, or for a keyless template card `md descriptor --network
<net> --template <what `md decode <md1…>` prints> --key @i=<xpub> --fingerprint @i=<fp>`.
The engraved card remains a faithful backup
```
No `✗ NO MATCH`, no stdout. Before the fix this was exit 4 / `✗ NO MATCH` (the
false verdict).

**`restore --cosigner @N=` (explicit assignment):** exit 2, identical wording.
Before the fix: exit 1, md-codec's internal `_with_network entry point` text.

**`restore --expect-wallet-id 468e0a1e...`:** exit 2, identical wording. Before
the fix: exit 1, same internal text.

**`verify-bundle --search-address`** (with the 9 `--mk1` template-stub cards):
exit 2, identical wording, no `NO MATCH`. This route was not run in the
original review (verify-bundle needs mintable stub cards); it is run here and
confirmed. `complete_multisig_template` is the shared engine (`verify_bundle.rs:977`
calls it directly, unchanged by this diff — the gate at the top of the shared
function covers it for free).

**NUMS control** (`restore --search-address`, same wallet, NUMS internal key
instead of Liana): exit 0, `✓ wallet-id (completed): 9817986e66962e355e2d5e3fae5f449f`,
first receive `bc1p4wtccfsr47cslecug6tqu934hdt6rfrr70mmjzmld075pje86lasggy3x7`,
full rendered descriptor on stdout. Confirms the refusal is keyed to the
internal-key kind, not to something coarser that would also block a legitimate
wallet.

## 2. Boundary: one rule, two call sites — not a single universal choke point

The implementer's own report states this correctly rather than overclaiming a
single boundary, and it checks out by direct grep of the built tree:

- The refusal logic and wording exist exactly once:
  `taproot_override_classify::liana_unspendable_card` (predicate, md-codec's
  own `wire_version() == WF_UNSPENDABLE_VERSION`) and
  `taproot_override_classify::LIANA_UNSPENDABLE_REFUSAL` (the one string), used
  by `cmd::restore::refuse_liana_unspendable`.
- That function is *called* at exactly two production sites
  (`grep -rn refuse_liana_unspendable crates/mnemonic-toolkit/src/`):
  `restore.rs:1700` (first statement of `complete_multisig_template`, shared by
  `restore`'s template-completion route and `verify-bundle`'s
  `verify_multisig_template`) and `restore.rs:3835` (`run_multisig`, the keyed
  wallet-policy route, right after decode).
- `classify_taproot_restore`'s old Liana arm (`restore.rs:3247-3253`) is now
  unreachable in practice (its only caller, `run_multisig`, already refused one
  line above) but is kept as a second copy of the *call*, not a second copy of
  the *wording* — it returns the same `liana_unspendable_refusal()`.

**Routes checked for a bypass, none found:**
- Restore's single-sig template dispatch (`is_singlesig_template`,
  `restore.rs:369-371`) cannot carry a Liana key: the original review's Lens 1
  already established a key-path-only `tr(Liana)` is unencodable (no admitted
  use-site), and this diff does not touch that dispatch or the encoder.
- `verify-bundle`'s *other*, older `run_multisig` (`verify_bundle.rs:1287`,
  the `--template`/`--slot` "expected bundle" comparison path, distinct from
  `complete_multisig_template`) builds its expected descriptor via
  `synthesize_unified`, which — per `parse_descriptor.rs:682` — never emits
  `InternalKey::LianaUnspendable`. It compares the supplied md1 as a card
  binding, never calls `to_miniscript_descriptor*` on the supplied tree. Traced
  by reading the function; not independently executed (out of the fix diff,
  and the original review's own scope), but it structurally cannot reach
  `NetworkRequiredForUnspendable`, so it is not a bypass of I-1's false-success
  class.
- `synthesize.rs:994` (`template_admissible`, `.is_ok()`-swallowed) was
  already verified sound by the original review's Lens 2 and is untouched by
  this diff.
- Full-repo grep for `to_miniscript_descriptor` production call sites found
  only `restore.rs:3679` (`faithful_multisig_descriptor`, now gated) and
  `synthesize.rs:994` (already-sound, gated by `.is_ok()`); the two hits in
  `template.rs` are test code.

So: not a single universal choke point, but every reachable route into a
render that could produce the false-success/false-refusal-text class is
covered by one of the two calls to the one function.

## 3. NUMS control

Covered under I-1 above: `restore --search-address` on the NUMS variant of the
identical wallet completes at exit 0 with the correct wallet-id and address.
Also ran the two other NUMS controls as part of the test-suite execution
(`restore_nums_control_explicit_cosigners_completes`,
`restore_nums_control_expect_wallet_id_completes`) and the verify-bundle NUMS
control (`verify_bundle_nums_control_verifies`) — all pass.

## 4. Mutation: would the new tests fail if the refusal were removed?

Applied the mutation directly (not re-derived from the implementer's claim):
removed the `refuse_liana_unspendable(d)?;` call at `restore.rs:1700` (the
`complete_multisig_template` entry site only; left the `run_multisig`
backstop call intact), rebuilt, ran `cargo test -p mnemonic-toolkit --test
cli_liana_template_refusal`:

```
test restore_liana_template_search_address_refuses_not_no_match ... FAILED (left: 4, right: 2, "✗ NO MATCH")
test restore_liana_template_explicit_cosigners_refuses ... FAILED (left: 1, right: 2, md-codec internal text)
test restore_liana_template_expect_wallet_id_refuses ... FAILED (left: 1, right: 2, md-codec internal text)
test verify_bundle_liana_template_refuses_not_mismatch ... FAILED (left: 4, right: 2, "✗ NO MATCH")
4 NUMS controls: all still passed
```

All four Liana tests fail, reproducing exactly the pre-fix defect shapes from
the original review (false `NO MATCH` under search, md-codec internal text
under explicit/expect-id). The four NUMS controls stay green, confirming the
mutation only removed the Liana gate and nothing else. Restored the file
(`diff` against the pre-mutation backup confirmed byte-identical), rebuilt,
re-ran the same test target: 8/8 pass again.

## 5. Minors — see table above. M-1 and M-4 addressed; M-2 and M-3 correctly left alone (out of I-1's scope, already dispositioned by the original review).

## Build gate (full workspace, this fix commit)

- `cargo nextest run --locked --workspace --no-fail-fast`: **4057 passed / 0
  failed / 20 skipped** (matches the implementer's reported count).
- `cargo clippy --locked --all-targets -- -D warnings`: clean (rc=0).
- `cargo fmt --all --check`: clean (rc=0).
- Targeted run `cargo nextest -E 'test(liana) + test(friendly)'`: 25/25 pass,
  including the keyed-route regression test
  `liana_unspendable_internal_key_refuses_not_nums` and all 17 `friendly.rs`
  unit tests.
- Worktree confirmed clean (`git status --short` empty) before and after the
  mutation round-trip.

## New findings

None. No new Critical, Important, Minor, or Nit beyond what is already
recorded above (all resolved or correctly out of scope).

ready to ship: yes
