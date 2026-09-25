# Brief — re-review of F-687 fold 1

**One question:** did fold 1 fix the F-687 review's findings (M1–M5, N1–N4), and
did it introduce a defect, especially a wrong or empty passphrase, or a stdin
read twice or by the wrong input? Not a fresh audit.

## Inputs

- ms `/scratch/code/shibboleth/ms-worktrees/f687`, `git diff e534917..79343e2`;
  toolkit `/scratch/code/shibboleth/tk-worktrees/f687`, `git diff 9846a784..6034922e`
  (branch `f687-passphrase-channels` in both).
- Review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-review.md`.
- Fold report: `.../design/agent-reports/f687-fold1.md`.

## Settled

- The main change was reviewed ready to ship (0C/0I). Empty-passphrase
  behaviour is deliberately unchanged (awaiting an operator ruling); don't
  report it.

## Check, with evidence

1. **M1, stdin by path or inode:** `/dev/stdin`, `/dev/fd/0`,
   `/proc/self/fd/0`, and a same-inode path, with `-` or `--passphrase-stdin`,
   all refused on every listed flag. Now the false positives: does a
   *regular file* ever get refused because it matched fd 0's inode wrongly?
   E.g. stdin redirected FROM that same file (`< f` plus `--in f`, which is
   arguably fine to refuse, so say which), stdin closed, stdin a TTY, and
   stdin `/dev/null` alongside `--in /dev/null`.
2. **M2 (ms):** one predicate. `--passphrase " -"`, `"\t-"`, `"- "` and `"--"`
   behave identically in ms and the toolkit.
3. **M3:** re-run the G1, G2, G3 mutations and one new one of yours; each goes red.
4. **M4, M5, N1–N4:** spot-check each against the binaries, including N1's
   non-UTF-8 exit 64 without an echo.
5. Gates: nextest both repos, pinned clippy, fmt; the vector files still
   byte-identical across repos.

Build with your own `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f687b-scratch/`; delete with `find … -delete`.
Don't commit, push or edit the branches.

## Output

Per finding: fixed / not fixed. NEW findings Critical / Important / Minor / Nit.
End with `ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-rereview.md`**
and return only a short summary plus that path.
