# F-449 stage 4a: whole-branch adversarial review

Branch `f449-stage4a`, `434d94c3..e8034282` (5 commits). Reviewer: opus, independent of the implementer and the plan author.

**Verdict: 0 Critical / 0 Important / 4 Minor.** Every behaviour difference from the base on a real or corpus input falls into a class the plan authorised. I could not build a wrong count, a false confirmation or a new false PASS.

## Method (all RUN, not read)

- I built three `me` binaries at opt-level 2 (the dev profile) with toolchain 1.85.0, each in its own target under `f449-stage4a-review-target/`: **v0.10.0** (`1fa8dd99`), the **base** `434d94c3` (0.10.0 plus post-tag `1cbecbfd`), and the **head** `e8034282`. Differences were measured base against head, so the branch alone is isolated.
- The differential harness ran each input set through `me bundle`, `me --hex` (convert), `me sysw pack --no-passphrase --no-now` with an `me sysw show` of the output and a sha256 of the container, `pack --expect descriptor`, and `me hash --unsealed`. Stdout, stderr, the exit code and the output-file bytes were all compared, with the version string normalised.
- Corpus:
  - 308 md1 and 21 mk1 strings grepped from the base tree (`crates/`, `demo/`, `design/`).
  - 120 fresh `md encode` outputs from md 0.18.0: 20 templates (wpkh, pkh, sh-wpkh, tr key-only, wsh and sh multi/sortedmulti, tr script trees, multi_a/sortedmulti_a, wsh timelock, sha256 and hash160 hashlocks, 3 tr-NUMS templates, and 5-key multi). Each was encoded 6 ways: bare, `--path bip48`, with keys, with keys and fingerprints, `--force-chunked`, and keyed plus chunked. 69 sets are single strings and 51 are multi-chunk (up to 10 chunks).
  - md 0.19.0 (built from descriptor-mnemonic `cf35d61a`): v8 Liana compose outputs, both keyed and template.
- Descriptor input: all 74 `descriptor_seam_vectors.json` vectors × {`--as md1`, `--as descriptor`, no `--as`}. I added 3 hand-written `tr(...)` descriptors under `--as md1`, because no seam vector that `--as md1` admits is P2TR, and P2TR is the one encoder site the branch touched (`md1.rs` `InternalKey::Slot(0)`).

## Findings

### Critical: none
### Important: none

### Minor

**M1. The CHANGELOG understates the newly refused class, and the refusal text contradicts `md decode`.** `crates/me-cli/CHANGELOG.md:36-39` gives one example, a `tr(<key>,{pk,pk})` template. Measured with md 0.18.0, **12 of my 20 template shapes** encoded without `--path` produce a short unchunked plate that 0.10.0 bundled as "backup needs 1 public plate" and the head refuses with exit 4. Examples:
- `wsh(or_d(pk(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(144))))` becomes `md1yppqqxpxpgnxjacqqqqzgq9mrz54gkwsyqv`.
- `wsh(and_v(v:pk(@0/<0;1>/*),sha256(6c60…5333)))` becomes `md1yqpqqxpye55ad3s0gp8czear3lrsatu259avx…`.
- `wsh(or_d(multi(2,…),and_v(v:pkh(@2/…),older(52560))))` becomes `md1yzpqqxpxqcyy2v6tnsqqpn2s3yzxktpqqqn33`.
- All three tr-NUMS templates.

That is every shape with no canonical default path: wsh miniscript, hashlocks, timelocks and tr script trees. The refusal is authorised (plan F7, Review Focus 1) and consistent: the chunked and keyed forms of the same shapes were already refused, identically, by base and head (`SetIncompleteMd`). So there is **no defect in behaviour**. Two things are still off:
- (a) The CHANGELOG should say "any template whose shape has no canonical derivation path, encoded without origins".
- (b) The stderr says `md1 plate does not decode (non-canonical wrapper requires explicit origin …)`, while `md decode` on the same string exits 4 (VERIFY-ME) and prints the template. It names no remedy. `md compose` output carries inline origins and is unaffected; I checked the `demo/sh2/WALKS.md:131` string, which head bundles with exit 0. A hint such as "re-encode with `--path` or inline origins" would turn this into a recoverable refusal. That belongs with F-652 or as a wording follow-up.

**M2. A rebuilt local `me` confirms v8 cards before the firmware can read them.** A local `cargo install` from master after the merge would do this before stage 3 lands. Task 5 Step 0 gates only the tag and the release install. The standing "keep local binaries current" permission, plus `demo/sh2/build-payload.sh` running `me` from PATH (plan F15), means a merged-but-unreleased head could reach the operator's PATH. It would then print `public record N: md1/mk1 — confirmed` for a v8 card that the flashed board treats as a SECRET. The CHANGELOG's `### Note` states this. The branch does nothing wrong here, but the merge should carry the same "do not rebuild the local `me` before stage 3" hold as the tag.

**M3. The `[0.11.0] - 2026-09-23` heading is dated before any tag exists.** The implementer already flagged this. It must be re-dated when the tag is cut.

**M4. The `the_two_derivations_agree_wherever_both_can_derive` skip rests on admission, not on the codec.** It is correct and pinned by name. But the rows it skips still `md1::build` and `split` into 29 and 38 strings: I measured `accepted/sh-wsh-sortedmulti-16-keys`, `narrowed/sh-sortedmulti-16-keys` and `narrowed/wsh-sortedmulti-21-keys`. `me bundle` on the `narrowed/sh-sortedmulti-16-keys` strings says **"backup needs 29 public plates", exit 0, on base AND head**, and `sysw pack` confirms all of them. That is a P2SH script over 520 bytes (an unspendable wallet) that is stated complete. This is **pre-existing, not introduced by this branch** (identical before and after), and neither `md encode` (refused: "cannot be larger than 520 bytes") nor `me --as md1` (refused at admission) can produce it. Only a library-crafted plate can. I record it for F-651's scope: a decoder-side resource-limit check would close it.

## Lens results (verified sound)

1. **Regression on existing wallets.** 2,185 corpus cases produced 71 differences, all in authorised classes. **Every** difference is one of these:
   - (a) A v8 string (`md1c…`) that now decodes. For these, pack/show/expect/hash say "confirmed" and the bundle counts key slots.
   - (b) An unsupported-version or undecodable string (versions 0, 2 and 12, junk, and the v8 origin-less `md1gppqqxq799p20d5hxuzu2c9la`). It is now refused by `bundle`, or its message now names the version, and it is still unconfirmed with the same exit code.
   - (c) The M1 origin-less v4 class.
   - (d) The md-codec `WireVersionMismatch` Display changing from "expected 4" to "accepted versions: 4, 8" inside the pre-existing `hash`/`seal` refusal (`seal/record.rs` `decode_public_set`, unchanged code).

   There were **zero differences** in: `me` convert, all keyed or `--path` v4 bundles (single and chunked, up to 10 chunks, including tr-NUMS keyed 12 chunks), every sysw container's bytes (field 3, including v8, where the base and head `identity:` hashes are equal), `sysw show` for v4, and all 222 descriptor-path cases. That covers `--as md1` output, wallet-id and `address 0`, and a P2TR `--as md1` pack (identical container sha256 `4371f62a…` and `78b29485…`).
2. **F-635.** No set newly passes that should be refused. The only newly passing sets are v8 plates that genuinely decode: a v8 Liana 2-of-3/1-of-3 keyed wallet gives 12 plates and 6 slots, and a v8 hashlock template gives a TEMPLATE note plus a `sha256` hashlock note, the same notes as its v4 NUMS twin. The refusal catches the origin-less class (intended), unsupported versions, and non-decoding junk, and nothing else in the corpus. The unchunked rule is now the chunked one, so the two shapes agree on every template I encoded.
3. **The unpin.** `Cargo.lock` has exactly one `miniscript` (13.0.0 from git `ff4732e5`) and one `bitcoin`. A fresh `git clone --branch f449-stage4a` followed by `cargo build --locked --workspace` succeeds with no "Patch … was not used" warning and leaves the lock untouched. Outside F-651's test, the patch changes no output in the differential above.
4. **False passes.** I re-applied **M7** (`.filter(|_| false)` on `print_records`' `version_note`) in a clean clone. Exactly `show_names_the_wire_version_beside_the_record` failed (10 passed, 1 failed), so the mutation applies and reddens. I also ran one unplanned mutation: deleting the `Unmet::UnreadableVersion` push in `expect::check`, which would let `--expect descriptor` pass on an unreadable card and write the container. The full suite gave 674 passed and 1 failed (`expect_descriptor_names_the_version_instead_of_calling_it_incomplete`), so that fail-open is caught.
5. **CHANGELOG and spec.** They match the code, apart from M1(a) and M3. The FOLLOWUPS F-635 closure line is accurate.
6. **Merge.** `git merge-tree master e8034282` is clean. An actual merge in a scratch clone auto-merges `design/FOLLOWUPS.md`, and `scripts/followups-status.sh` prints `OK`. The duplicate `### F-` headings (F-59, F-62, F-99, F-100, F-530) are already on master. No F-number collision: the branch adds none, and master adds F-656.

Scratch worktrees and clones were removed. The `f449-stage4a` worktree is clean at `e8034282`. Nothing was committed.

ready to ship: yes
