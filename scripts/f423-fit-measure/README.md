# f423-fit-measure

S2 plan P4.1's measurement for F-423 (does more than one md1 string fit one
SeedHammer plate side?). `go run .` prints the analytic upper bound
(`backup.CharsPerLine` / `backup.LinesPerPlate`) and then trial-fits 1, 2, and
3 real md1 strings as separate `backup.Paragraph`s in one `backup.Text` plate,
through the same `backup.EngraveText` → fit-or-error path
`validateDescriptor` uses (gui/gui.go:722-736). Full output is pasted verbatim
in `design/agent-reports/MEASURE-S2-P4-1.md`.

`go.mod` pins the fork worktree via a local `replace` directive:

```
replace seedhammer.com => /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm
```

measured at fork rev `fe9475c` (branch `s2/descriptor-arm`). This mirrors the
existing precedent in `scripts/descriptor-seam-vectors/goprobe/go.mod`. Re-run
against a different fork rev by updating that path (or vendoring); the numbers
in the persisted report are only valid as of `fe9475c`.

`toPlate` (gui/gui.go:3515) is unexported and unreachable from this module.
`fitCheck` in `main.go` reproduces its five-line bounds check using only
`toPlate`'s own exported dependencies (`gui.SquarePlate`/`.Dims`,
`gui.ErrTooLarge`, `engrave.PlanEngraving`, `bspline.Measure`/`.Bounds`,
`bezier.Pt`/`.Sub`) plus the bare numeral `safetyMargin = 3` (gui/gui.go:52) —
a minimal shim over public API, not a fork of private logic.
