**Sonnet attempt 2: S1-S4 folded into `design/FIX_bughunt_sh2.md` and committed (one commit, not pushed). One STOP left: the M-S2a entry.**

## STOP (1)
M-S2a is decided to go to FIX §9.2 as a `deferred-minor` entry (owner: sh2 FIX build), but the ruling gives it no slug, and the entry shape needs one. I did not invent it, so the entry is not in the doc. Once a slug is named, add a §9.2 table row `| M-S2a | M | <slug> | the "without two distinct fingerprints" arm (composer_selfcheck.go:237) has no row left after the S2 retarget; check whether a fault-injection row (two seated st.assigned at one origin, same fingerprint) is cheap | sh2 FIX build |` (the ruling suggests the whole-diff review checks this).

## Fold text -> FIX doc (section)
- S1: §2.13 (30d `s` sentence and the `Args[2]` check), §6.1 30d row (five subtests, :102 texts) and its trailing note, §1 table 30c/30d/49 line, "How checked" RED s5 line.
- S2: §2.5 string, §3 row "fix-21", §7 row 1 (case now gets the new text; want-substring changed), §7 row 2 deleted, §8 item 4 drops the STOP 2 note, §1 table fix-21 line (grade's "§4f's default" -> "the default origin").
- S3: §4 (all seven spec texts plus the patch/sha/count paragraph), §8 items 1, 2, new 11, §9.1 impacts (1), (2), new (11), §9.2 `sh2-me-canonical-depth` heading entry and the `sh2-me-refusal-device-claim` row, §7 DeviceColumn row ("fixed by the re-pin (29/45, probe 0)"), §3 rows for the fork test and the me patch, scope paragraph.
- S4: §2.14 (file text verbatim, tag, sha), §3 row, §8 items 5 and 6, host-build row of the "How checked" table.
- §10 is now the one line. No "STOP" text remains except the §1 counts line and the §10 heading (grep).

## Editorial choices (mine)
- Line anchor: the case is at `composer_selfcheck_test.go:147`, its want-substring at `:150` (read at the s2 head 1df27de); neither `:150`-only nor `:167` is exact, so the doc cites `:147-150`.
- Spec anchors in S3 are the ruling's; at this head the "...are unchanged." and `device_admits=true` anchors are lines 1671 and 1714 (ruling says 1670 and 1713); the doc says so and relies on the quoted text.
- §4 header widened to §4.2/4.3/4.7/7/9; §1 counts line gained a sentence on what the rulings changed; the `sh2-me-canonical-depth` Important and the `sh2-me-refusal-device-claim` Minor are noted there.
- Fork gate item numbering: spec amendments are new §8 item 11 (matches new Build impact (11)).

## Checks
- `git apply --check design/FIX_bughunt_sh2-s3-me.diff` in this worktree: rc=0. `git apply --check` of the fork patch in `/scratch/code/shibboleth/sh-worktrees/bughunt-triage-s3`: rc=0.
- Both committed patches `cmp`-identical to the probe copies; sha256 `5f719fba…da612` (me) and `b9d95f1d…48f6f` (fork).
- Greps of ~30 ruling strings in the FIX doc: all found; three that wrap across lines were found against a whitespace-flattened copy (item 2 differs only in capitalising "Copy").
- Not run: the patches' build/test results (taken from the ruling), tinygo.

counts: 0 C, 0 I, 1 STOP
