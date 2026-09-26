---
description: Resume the "release script for all repos" task after the 2026-09-26 reboot
---

Read `design/CONTINUITY_2026-09-26_release-script.md` in full, then check the
current state before acting: `git status` and unpushed commits in each
constellation repo, and the latest release tag per repo (`gh release list`).
Summarise what (if anything) changed since the continuity file was written.

Then start the release-script task it describes. It is risk-set work (tags are
irreversible): brainstorm the design with the operator first, then a short spec
with an R0 review, before any implementation. Don't cut any release without the
operator asking for one.
