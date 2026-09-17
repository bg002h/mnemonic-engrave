# Continuity — SH2 demo + mnemonic-toolkit v0.98.0 (2026-09-17)

Resume anchor for a `/clear`. Everything below is on git/quantoshi (durable);
the only unpushed bits are two LOCAL commits noted in §Resume.

## DONE and live/pushed
- **Demo** at https://quantoshi.xyz/SH2/ (engrave master **b73dd872**), live ==
  built dist, rehearse.sh green:
  - John the Ripper generates the passphrase candidate list (→ `passphrase-of-xpub`).
  - Shamir §1 proves ALL 10 triples rebuild / 10 pairs refuse + a mutation check.
  - Toolkit install now points at **v0.98.0 mac/Windows binaries** (Linux = v0.97.0 musl).
  - Recovery searches (3b) + argv refusal/escape-hatch bonus.
- **push-via-staging.sh unified**, byte-identical in all 5 repos (dm/ms/engrave/toolkit/fork).
- **mnemonic-toolkit v0.98.0** PUBLIC release: macOS amd64+arm64 + Windows binaries,
  checksummed (macOS-amd64 verified Mach-O). Tag `mnemonic-toolkit-v0.98.0` @ `8c731ca4`.
- **Stale musl `miniscript_rev` fixed** (95fdd1c→ff4732e5) in man-pages.yml/repro-drift.yml
  (PR #78 merged; toolkit master **baf288b2**), so the NEXT tag builds Linux musl.
- Earlier this session: F-590/608 + Minor/Nit residue; toolkit ceiling-test flake fix;
  the g6/sibling-pin reds are documented non-required.

## OPEN
1. **Wallet-id auto-enumerate feature** — the queued spec-first deliverable.
   - Spec written + committed LOCALLY (unpushed) in mnemonic-toolkit:
     `design/SPEC_restore_wallet_id_prefix_enumerate.md` (commit 327e13c7).
   - Operator decisions: **spec+R0 gate first**; **auto-enumerate on short prefix**
     (no flag; list matches, never auto-pick). Normative + funds-adjacent → R0 to 0C/0I.
   - **Next step: R0-review the spec (opus), persist verbatim to
     design/agent-reports/, fold, re-review to GREEN, THEN implement (TDD).**
   - Shipping it falsifies the demo 3b "72d94d49 refused" claim → rewrite 3b (in spec §7).
2. **v0.98.0 Linux musl gap** — filed in toolkit design/FOLLOWUPS.md. Operator's call:
   re-cut (force-move the v0.98.0 tag to a commit with the pin fix; ~60min aarch64 QEMU repro)
   OR ship Linux at the next version. Not demo-blocking.
3. **Hardware-blocked**: F-583/584/585/591 + device legs (need the SH2 or an emulator payload).
4. **Signing**: add MINISIGN_SECRET_KEY(+PASSWORD) to md/ms/mk/mt repos; workflows already
   branch on the secret, so signing starts with no code change.

## Repo tips
- engrave master b73dd872 (pushed); toolkit master baf288b2 (pushed) + local commit 327e13c7 (spec, UNPUSHED).
- Push: `./scripts/push-via-staging.sh master` (each repo has its own copy). toolkit required
  contexts: examples, test (ubuntu-latest), clippy. A design/-only or workflow-only change is
  NOT in the push-paths → route via a PR (pull_request is unfiltered).
- toolkit local mnemonic is 0.97.0; the v0.98.0 build is target/debug or `cargo install`.

## Resume
Push the local spec commit (via a PR, since design/-only won't earn the required
contexts on a direct push), then dispatch the R0 review of
`SPEC_restore_wallet_id_prefix_enumerate.md`.
