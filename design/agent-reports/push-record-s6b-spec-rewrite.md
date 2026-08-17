# Push record — S6b spec rewrite — 2026-08-17

**Verdict: SATISFIED.**

**Note on authorship:** this record is written by the **controller**, not by the
push agent, and that is a deviation from the standing rule that agents persist
their own reports. The reason is below — the agent returned mid-ritual, so it
never reached its final action. The steps it did not perform were completed by
the controller and are recorded here with their actual output.

## What happened

The push agent staged correctly and then **returned before the ritual was
complete**, reporting *"The CI run is still in progress. I'll pause here and wait
for the background wait task to notify me."* It had spawned a wait it could not
be resumed from, so the task ended with:

| ref | state at handoff |
| --- | --- |
| `ci/staging` | `6c85d2830d50aba2359b992a37e87fb99d9583ce` — staged, correct |
| `origin/master` | `7e3a0f360e0ff64f0d14b749b51d8890a88dc329` — **3 commits behind** |
| local `HEAD` | `6c85d283…` — frozen |

**This is a pause, not a failure.** The agent staged the right SHA — it read
`HEAD` rather than trusting the controller's brief, which is what saved it: the
brief said "two commits" when there were **three** (`77a6e38`, `3eb2f60`,
`6c85d28`). A controller miscount that an agent inherits becomes a wrong push;
this one did not, because the agent measured.

## The freeze was the risk, and it held

The window between staging and the final push is exactly where this project's
worst push incident occurred: a previous run staged one SHA while two more
landed, `strict: false` accepted the newer tip against the older gated ancestor,
and two commits reached `origin/master` printing "Bypassed rule violations" with
zero CI signal.

So no commits were made while the run finished. Verified before pushing:

```
HEAD   6c85d2830d50aba2359b992a37e87fb99d9583ce
staged 6c85d2830d50aba2359b992a37e87fb99d9583ce
FREEZE HELD
```

## CI, per job, on the staged SHA

Run **32079858749** (release workflow), `headSha` =
`6c85d2830d50aba2359b992a37e87fb99d9583ce`, `status: completed`,
`conclusion: success`.

| job | conclusion |
| --- | --- |
| **`test (rust + go)`** | **success** ← the required context |
| `build me (linux-x86_64)` | success |
| `build me (linux-aarch64)` | success |
| `build me (macos-x86_64)` | success |
| `build me (macos-aarch64)` | success |
| `build me (windows-x86_64)` | success |
| `build me-preview (all targets)` | success |
| **`assemble + sign + release`** | **skipped** ← tag-gated; a `ci/**` push cannot sign or publish |

## Final push — verbatim

```
To github.com:bg002h/mnemonic-engrave.git
   7e3a0f3..6c85d28  master -> master
```

**No "Bypassed rule violations" string.** A plain fast-forward.

## Staging deletion, with positive control

```
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging

$ git ls-remote origin 'refs/heads/master' 'refs/heads/ci/*'
6c85d2830d50aba2359b992a37e87fb99d9583ce	refs/heads/master
```

`master` present in the same query proves the query ran; no `ci/*` row proves the
deletion. An empty result for both would have proved nothing.

## What was published

- the comprehension + unfounded-assumptions review, committed verbatim (RED
  3C/7I);
- the clean rewrite of the spec folding its nineteen findings — 417 lines
  replacing 661.

## Worth carrying forward

**A push agent can return mid-ritual with the remote in a half-applied state**:
`ci/staging` created, `master` un-advanced. That state is safe — nothing is
bypassed and nothing is lost — but it is **not** the state the next dispatch
assumes, and a second agent launched blindly against it would re-stage a ref
that already exists.

The remaining steps are three commands. Finishing them in the controller is
cheaper and safer than a re-dispatch, provided the freeze is re-verified first —
which is the one step that must not be skipped, because the controller has been
committing all session and the freeze is only as good as the check.
