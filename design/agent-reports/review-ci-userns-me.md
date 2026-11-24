0C/0I/1M/1N

# Review: me bughunt_me_01 userns probe (5b2d3551..5e4f953e)

Scope: `crates/me-cli/tests/bughunt.rs` probe + CHANGELOG line. Author of the diff: implementer (previous round; not a fix commit of a review, so 0 of these findings come from a prior fix round).

## Answer to the one question
No false skip found on a machine where the real setup would work, and no pass-without-testing-the-fix path.

- Probe: `unshare -U --map-auto --map-root-user sh -c 'touch p && chown 1 p'` in a fresh tempdir. Row setup: same unshare flags, `chown 1 old.bin` in a tempdir. Same capability (subuid-mapped chown), same uid (1), same cwd class. A probe failure implies setup failure (chown EPERM, or unshare absent), so a skip there never hides a working setup.
- Reverse (probe passes, setup fails): setup is not under the probe. Remaining unprobed steps are `chmod 666`, `setpriv --reuid=2 --regid=2 --clear-groups`, and mapping of uid 2. These fail through the existing `assert!(out.status.success(), "setup failed ...")` or the "premise: me must fail" assert: a loud FAIL, never a false PASS. Acceptable, consistent with the old behaviour.
- Fix-testing: after the probe passes, the body is unchanged. With `write_private` reverted as a scratch change (`.truncate(true)` at open in `crates/mnemonic-io-lib/src/write.rs`), row 01 run by exact name FAILED at bughunt.rs:82 ("old file was destroyed", left `[]`), i.e. the row ran (not skipped) and caught the fault. Tree restored; `git status` clean.
- Consistency with `ME_REQUIRE_GO`: same shape (env `== "1"`, skip note on stderr + return, hard fail when set). Difference only in form (panic! vs assert!, inline vs helper): no behavioural gap.

## Findings
- M1: Nothing sets `ME_REQUIRE_USERNS=1` anywhere (`.github/workflows/release.yml` sets only `ME_REQUIRE_GO` at line 86), so on the GitHub runner the row now always skips and the FIX #1 regression is not tested in CI at all; it is covered only by local runs. By design per the brief, but it is an unmet CI guarantee worth a FOLLOWUPS line (owner: whichever phase next touches CI, e.g. a runner that supports subuid, or a `.github/` env line on a self-hosted/rootless-capable job). Fix wording: "FOLLOWUP: bughunt_me_01 is skipped on GitHub-hosted runners; run it with ME_REQUIRE_USERNS=1 where /etc/subuid is provisioned."
- N1: A skip is only an `eprintln!` (visible only with `--no-capture` or on failure); nextest reports it as PASS. Same as the `ME_REQUIRE_GO` pattern, so consistent; noting only.

C / I / M / N: 0 / 0 / 1 / 1
