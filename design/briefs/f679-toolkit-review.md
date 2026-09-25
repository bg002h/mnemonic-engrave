# Brief — review of toolkit `f679-681` (installer pins + GUI manual → v0.62.0 + F-681)

**One question:** can this ship? Does the installer install the right versions,
and does every sentence the GUI manual now makes about what the GUI and CLIs
do match what the pinned binaries actually do? Not a fresh audit of the toolkit.

## Inputs

- Worktree `/scratch/code/shibboleth/tk-worktrees/f679-681`, branch `f679-681`,
  `git log origin/master..8ed3494f`: `65ba3c22` (F-681), `c39ea361` (installer
  pins), `8ed3494f` (GUI manual). 171 files changed.
- Implementer report (claims to re-run): `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-toolkit-impl.md`.
- Pinned releases: mnemonic 0.104.0, md 0.20.3, **ms 0.19.1**, mk 0.13.0,
  **mnemonic-gui v0.62.0**. Install them into a scratch root with this branch's
  own `scripts/install.sh --root <scratch> --no-man` (GUI included).

## Settled — do not re-derive

- Controller-measured: the published `mnemonic-gui-v0.62.0-x86_64-linux.tar.gz`
  passes SHA256SUMS and its max GLIBC is 2.18. ms 0.19.1 fixes `ms verify
  --phrase <P> <ms1>` (reviewed separately, 0C/0I). The GUI's behaviour itself
  was reviewed (`f679-review.md`, `f679-rereview.md`); this review checks what
  the *manual* says about it.
- The installer's design (F-676) was reviewed through three folds; only
  re-check it where `c39ea361` touched it.
- Secret-handling severity rule: never Critical/Important.

## Where to push hardest

1. **The manual's claims vs the binaries.** The implementer flags that about
   90 pages of refusal tables and exit codes were never swept against the
   CLI changes from 0.75 to 0.104. Sample at least 25 concrete claims
   (an exit code, a refusal message, a flag's effect, an output shape)
   across the GUI manual, weighted toward pages that mention refusals, exit
   codes, separators and secrets, and **run each one** against the pinned
   binaries. Report the false-claim rate you measure and every false claim.
   A doc that tells a user a dangerous thing is refused, when it isn't, is
   Important; a wrong exit code is Minor.
2. **Chapter 50 (J4 step 14)** and the other rewritten tutorial prose (J1, J3,
   J5): does the text match the figures and transcripts now in the tree, and
   the v0.62.0 tag's corpus?
3. **Installer pins:** a real install from this branch gives all five at their
   pins; `ms verify --phrase <P> <ms1>` works through it (BIP-39 test vector
   "abandon ×11 about", no funds); the glibc floor table matches `readelf` on
   the real assets. Are the unguarded GUI install-page pins consistent with the
   installer?
4. **Gates:** the Examples golden, sibling-pin-check, the manual and
   manual-gui builds and audits (set `*_BIN` to the scratch root), install tests
   under dash, shellcheck.

Scratch under `/scratch/code/shibboleth/review-f679tk-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Critical / Important / Minor / Nit, each with a reproduction; the measured
false-claim rate from item 1. End with `ready to ship: yes` or
`ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-toolkit-review.md`**
and return only a short summary plus that path.
