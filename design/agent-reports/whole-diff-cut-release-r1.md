# Whole-diff review r1: cut-release (engrave PR 25, head e1d25113)

Reviewer: single Opus slice, low effort (s1 design/threat + s2 tests-can-fail). First round; no findings come from a prior fix commit.
Scope: `git diff 49ff198f..e1d25113`, focused on the tag/push/reset path (`scripts/cut-release.sh` 1205-1460) against SPEC §5.3 steps 3-4a.

## Answer to the one question
No path found that publishes a wrong tag, tags the wrong commit, reports a partial push as success, or resets destructively:
- Tags are annotated, created on an explicit `TAG_SHA` (release commit, or (c) for the toolkit) and pushed with one `git push --atomic`; failure paths read `ls-remote` per tag and match SPEC §5.3 step 4 bullet for bullet.
- Tagging happens only after push-via-staging exits 0, prints `== OK:` and shows no bypass line; push-via-staging pushes `HEAD:<branch>` non-force, so the landed commit is the local one.
- The toolkit reset is `git reset --keep (b)` (refuses to discard local edits), run only after the tag push.
- Probe: `git ls-remote origin 'refs/tags/<t>^{}'` in a local bare clone returns the peeled line for an annotated tag and nothing for a lightweight one; the script only creates annotated tags, so the post-push check is sound.

## Findings

### M1 — post-push peel-mismatch `exit 6` is untested (plant J survives; deviation 12)
`scripts/cut-release.sh:1328-1333`. Fails closed in practice only if something moves the tag on origin between push and `ls-remote` (2+ unusual conditions), hence Minor. With the guard removed (plant J) the script prints `== tag ... pushed` for a tag that peels elsewhere and continues to the wait (observed: exit 7 "no 'release' run appeared").
Fix: add this row to `scripts/test/cut-release.test.sh` (before `cr_push_run_never_appears`, and append `cr_push_tag_moved_after_push` to `ROW_LIST`; a spec row 79 in SPEC §11 table to match):
```
cr_push_tag_moved_after_push() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    printf '#!/bin/sh\nwhile read old new ref; do\n  case "$ref" in refs/tags/*) git tag -f -a -m moved "${ref#refs/tags/}" master~1 >/dev/null ;; esac\ndone\n' >"$FXBARE/hooks/post-receive"
    chmod +x "$FXBARE/hooks/post-receive"
    run_cr --push ms-cli-v0.20.3
    x_rc 6
    local tip moved
    tip=$(bsha refs/heads/master); moved=$(bsha refs/heads/master~1)
    x_err "cut-release: origin's ms-cli-v0.20.3 peels to $moved, not $tip; STOP and report it"
    x_no_line "== tag ms-cli-v0.20.3 -> $(s12 "$tip") pushed"
}
```
Run as printed in a scratch clone (`/scratch/cache/cut-release-review/wt`, `CUT_RELEASE_TEST_ONLY=cr_push_tag_moved_after_push`):
- unmodified e1d25113: `ok cr_push_tag_moved_after_push`
- plant J applied: `FAIL cr_push_tag_moved_after_push: exit 7, wanted 6 (stderr: cut-release: no 'release' run appeared for ms-cli-v0.20.3 within 10 minutes|...); stderr lacks line: cut-release: origin's ms-cli-v0.20.3 peels to c55a670a…, not 892d98f9…; STOP and report it; unexpected line: == tag ms-cli-v0.20.3 -> 892d98f97988 pushed`

### N1 — test runs leave `~/.cache/cut-release-test/run-*` behind under `CUT_RELEASE_TEST_ONLY`
Harness prints "kept <dir>" on a non-full run; harmless dev-aid clutter.

## Deviations 1-13 (build report)
1 accept (exempt entries with comment; no workflow change). 2 accept. 3 accept. 4 accept (defensive, fail closed). 5 accept (no run id to print). 6 accept (per-repo cap; printed command reproduces the hidden hits). 7 accept. 8 accept (stale lock on resume fails closed at `cargo build --locked`, exit 4). 9 accept. 10 accept (matches §5.3 4a: variable set the moment the atomic push returns 0). 11 accept (post-check "no P left" holds). 12 -> M1. 13 accept (read-only network calls).

F-709/F-710/F-711: build does not change their severity; not re-graded.

Not re-run here: the full offline suite (settled 70/0), online rows 71-78.

C / I / M / N: 0 / 0 / 1 / 1
