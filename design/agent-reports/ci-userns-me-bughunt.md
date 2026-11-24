DONE
# ci-userns-me-bughunt

Commit: 5e4f953e (one commit, not pushed) on fix/bughunt. Files: crates/me-cli/tests/bughunt.rs, crates/me-cli/CHANGELOG.md.

Gate (cargo +1.85.0, target /scratch/cache/bughunt/me/target-ci; outputs in /scratch/cache/bughunt/me/{fmt,clippy,nextest}.txt):
- fmt --check: exit 0
- clippy --locked --all-targets -D warnings: exit 0
- nextest run --locked: 705 tests run, 705 passed, 2 skipped

Row 01 by exact name (`test(/(^|::)bughunt_me_01_write_private_keeps_old_file_when_chmod_fails$/)`):
- (a) normal: PASS (1 passed, 706 skipped); the row ran.
- (b) ME_REQUIRE_USERNS=1: PASS.
- (c) probe forced to fail (scratch edit "-U" -> "-U-bogus", restored; grep bogus = 0):
  - no env: printed "SKIP bughunt_me_01: user-namespace chown (unshare -U --map-auto with an /etc/subuid range) is unavailable; set ME_REQUIRE_USERNS=1 to make this a failure", PASS.
  - ME_REQUIRE_USERNS=1: panicked "ME_REQUIRE_USERNS=1 but user-namespace chown (...) is unavailable; ...", FAIL.

Decisions the brief left open (flag for the controller):
1. CHANGELOG: the existing ME_REQUIRE_GO entry is in the released 0.x section; I put the new line under [Unreleased] ### Changed.
2. Commit trailer: used "Co-Authored-By: Claude Sonnet 5.5" plus Claude-Session (the session's attribution reminder), not the brief's "Claude Opus 5.5", since the author was Sonnet. Amend if you want the brief's text.
3. The skip message wording and the "SKIP bughunt_me_01:" prefix are mine.
4. cargo fmt reflowed my added code; assertions of the row are unchanged.
