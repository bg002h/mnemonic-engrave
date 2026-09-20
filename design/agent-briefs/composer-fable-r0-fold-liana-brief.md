# Fold — lens 5 (Liana) on the device: one consent notice naming Liana's model

Single implementer (sonnet). Repo `/scratch/code/shibboleth/seedhammer`, base
`781dc7cd71703412034d8188dbc62c463336bbe5` (fork main, fold B merged). Branch
`fable-r0-fold-liana` in your own worktree:
`git -C /scratch/code/shibboleth/seedhammer worktree add -b fable-r0-fold-liana /scratch/code/shibboleth/.tmp/fold-liana 781dc7cd71703412034d8188dbc62c463336bbe5`.
RED first, one commit (two if the test file is large), push nothing, no
sub-agents, never read `.jsonl`, never build under `/tmp`. Go is
`/scratch/code/shibboleth/.toolchain/go/bin/go` FIRST on PATH;
`export TMPDIR=/scratch/code/shibboleth/.tmp`. One gui test:
`CGO_ENABLED=0 go test ./gui/ -run '^Name$' -count=1 -v`; the whole gui
package ONLY via `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.
`gofmt -l .` baseline is five files (`gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`,
`mt/mt.go`, `mt/mt_test.go`); `go vet ./...` only the `ArtifactDir` notes.
Copy rules: ASCII; every FIXED body passes the modal-fits assertion; the notice
is a consent line group like `composerCopyMixedLockBases` (§8w) — study that
commit (`git log --oneline -3 -- gui/composer_copy.go gui/composer_consent.go`
on main) and copy its shape: predicate in `gui/composer_consent.go` over the
DECODED shape or the path list, body in `gui/composer_copy.go`, positives AND
negatives in the test, and the doc-comment gate's owner list
(`gui/composer_doc_comment_test.go`, `composerDocOwners`) extended with every
new helper so it cannot steal a neighbour's doc.

The finding: `mnemonic-engrave/design/agent-reports/composer-fable-r0-liana-core.md`
I-1 and I-2, and its section "Liana v8.0 — what the importer requires" (the
nine refusal classes with `analysis.rs` line cites). Read those in full.

## What to build

A consent notice, fired when the composed policy is OUTSIDE Liana 8.0's
model, that names the FIRST class that applies, in this order (Liana's own
order of refusal):

1. `sh` / `sh(wsh)` wrapper — Liana takes `wsh` or `tr` only.
2. NUMS internal key under `tr` (the shape already reports `KeyPathNUMS`).
3. No lock on any path (a plain multisig or single key has no recovery path).
4. A hash on any path (keyed or key-less, any kind).
5. `after(...)` on any path.
6. `older` in 512-second units on any path.
7. No unlocked path (every path locked).
8. Two locked paths with the same `older` value.
9. A second unlocked path (a second unlocked multi-key path is refused; a
   single-key unlocked path AFTER a multi-key one is silently folded into the
   multi as an extra key without changing k — lens 5 I-2 — so name BOTH as
   "Liana will not show this wallet as built").

Body (one FIXED head, one variable class line; keep to the modal):

> OUTSIDE LIANA'S MODEL
> Liana takes one unlocked path, at least one path locked by older in
> blocks, and no hash. This policy: <class>. Bitcoin Core imports it.

where `<class>` is one short ASCII phrase per row above (write them; e.g.
"legacy wrapper", "NUMS key path", "no locked path", "a hash lock", "an
absolute lock", "a lock in time units", "no unlocked path", "two paths with
one lock", "a second unlocked path"). Under `tr` with a real key and a
Liana-shaped list the notice must NOT fire; the six shipped presets under `wsh`
that Liana accepts (measure with the list in the report's runbook §4) must NOT
fire it; the demo payload's own plain 2-of-3 MUST fire it (class 3). Same-seed
inside one path is already §8g's "Liana will refuse it" — do not duplicate.

Tests: a table of at least 12 path lists (each class once, three negatives
including a tr-with-real-key and the kofn-recovery preset), RED before the
predicate exists, GREEN after; a mutation note for each class row in the
commit message; the modal-fits assertion for the longest class line.

Spec mirror: the controller writes §8x from the body you report — quote the
final body and every class phrase verbatim in your report.

## Gate — paste the numbers
vet; gofmt baseline; the whole gui shard set; firmware size
(`PATH=/nix/var/nix/profiles/default/bin:$PATH nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`),
baseline at 781dc7cd first.

## Report — FINAL action
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-liana-implementation.md`:
commit SHA(s), RED before / GREEN after, the body and class phrases verbatim,
the gate numbers, deviations with reasons. Return only the verdict line, the
tip SHA, and the path.
