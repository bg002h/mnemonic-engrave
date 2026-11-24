# Fold report: cut-release rm-guards FIX review r1 (I2; I1 downgraded; M1-M7, N1-N2 filed)

Brief: `briefs/cut-release-rm-guards-fold-r1.md`. Branch `fix/cut-release-rm-guards`, prior head cc6ab02b.

Per edit:
- Edit 2: applied (row 71, 72 line now says `72 also with --push`).
- Edit 5: applied (row 72 extended with the push step and its expected output).
- Edit 8: applied (push refusal check appended to the row-72 function).
- Edit 10: applied (row-table entry extended with the push expectations).
- Edit 12: applied (F-713 file appended to `design/FOLLOWUPS.md` via `cat`, not edited).

Each FIX OLD text occurred exactly once before its replacement; no STOP.

Commit: c9d11d8905a109300037e64024dd622800f81d81 (`design: fold rm-guards FIX review r1 I2; file I1 (downgraded), M1-M7, N1-N2`), staging only `design/FIX_cut_release_rm_guards.md` and `design/FOLLOWUPS.md`.

`git diff --stat HEAD~1`:
```
 design/FIX_cut_release_rm_guards.md | 12 +++++++++---
 design/FOLLOWUPS.md                 | 16 ++++++++++++++++
 2 files changed, 25 insertions(+), 3 deletions(-)
```

Deviations and notes:
- Commit trailer: the brief names `Co-Authored-By: Claude Opus 5.5`. The session attribution rule names Haiku 5.5, which is the model actually running, so the commit uses the Haiku 5.5 line. Controller may amend if the brief's line is required.
- The harness was not run, per the brief.
- Two stray scratch files I created by mistake (`/tmp/unused`, `/scratch/claude-1000-scratch-msg.txt`) and the edit script I first wrote into the coordinator `briefs/` directory were removed; none are in any repo.
