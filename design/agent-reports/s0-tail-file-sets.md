# S0 D4/D5/D8 file-set enumeration — can they run concurrently?

Scope: S0 deliverables D4, D5, D8 only, from
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (lines ~245-280). D6/D7 are
DONE and out of scope; S1-S5 are covered by
`design/agent-reports/parallel-implementation-feasibility.md`, not redone here.

Repos measured at: seedhammer fork `0ae3756` (clean), mnemonic-engrave `f2a43e2`
(clean).

## SUMMARY

| deliverable | concurrent with the other two? |
| --- | --- |
| **D4** | **Yes** — as currently scoped it touches no files at all (see D4 below); if a future author adds a frame-capturing walk harness it would land under `mnemonic-engrave/design/journeys/`, still disjoint from D5/D8's fork-side `.go` work. |
| **D5** | **Yes** — new greenfield Go component in the fork, with no existing package identified that D8 also touches. Location is not yet decided by the plan (unresolved, see below), but every plausible location (`cmd/emu/` new files, or a new top-level package e.g. `oracle/`) is disjoint from `md/`. |
| **D8** | **Yes** — confined to `md/testdata/**` plus a small, enumerated set of `md/*.go` files that cite the old pin in comments/docs. No overlap with `address/**` (D6/D7, occupied) or with any plausible D5 location. |

All three are pairwise file-disjoint on current evidence. The one thing that is
**not** disjoint is a *shared resource*: if D5's harness ends up living in
`cmd/emu/` (plausible but undecided), it and D8 both sit under the same Go
module and both must pass a clean `go test ./...` baseline before/after — that's
the ordinary "one module, one baseline" caveat the prior report already named,
not a file collision.

## D4 — frame receiver keeps its existing security properties

**Does a frame receiver exist today, and where?**

Yes, exactly one: `design/journeys/shot_server.py` (mnemonic-engrave repo,
4142 bytes, verified `ls -la`). It receives POSTed canvas frames (PNG/SVG data
URLs) over HTTP, pinned to one CORS origin, with a flat-filename whitelist plus
a post-resolution realpath re-check (read in full; docstring lines 9-20 state
both restrictions explicitly). It was hardened for exactly this shape of bug in
commit `e46ae05` ("fix arbitrary file write in the screenshot receiver",
2026-08-11) — a real path-traversal + open-CORS vulnerability, fixed and
verified in that commit's message.

**Who uses it today:** `design/journeys/build_pdf_payload.py:434` shells it
(`python3 shot_server.py "$PWD/shots" 8732 http://127.0.0.1:8731 &`) to capture
frames for the **manual, static operator-journey PDF generator**
(`build_pdf.py`, `build_pdf_payload.py`, `build_pdf_pathological.py`). This
pipeline does **not** drive the emulator — confirmed by
`design/agent-reports/multisig-build-repair-plan-lens-journeys.md:83,121`
("`build_pdf_payload.py` shells headless Chrome only to print HTML→PDF"; "Nothing
in the repo drives the emulator").

**Does the automated walk (S0 D1-D3, done) use any frame receiver?** No.
`cmd/emu/walk_trace_a.js` (read in full, 266 lines) never posts an image frame
anywhere. Its only read channels are `window.shScreen()` (text) and
`window.shToolpath.strings()`/`.summary()` (JSON toolpath data). It has no
`fetch`, `XMLHttpRequest`, or `toDataURL` call. So today's walk gate produces no
visual artifact and needs no receiver at all.

**Conclusion.** D4 is a **hardening constraint on an existing file**
(`shot_server.py`), not a request to build something new — and the file it
constrains is not even used by the walk today. Corroborating: the plan's own
"Tests first" list (8 named tests, lines 284-306) names **zero** tests for D4 —
every other deliverable in scope (D1, D2, D3, D5, D6) has at least one named
test; D4 does not. Read together with
`design/agent-reports/multisig-build-repair-plan-lens-spec-coverage.md:122`
("S0 d1 builds a new capture harness and says nothing about either
restriction") — an earlier round's finding, from before this deliverable
existed — D4 was filed defensively: **if** some later S0/S1-S5 work adds a
frame-capturing harness for the automated walk, it must not drop
`shot_server.py`'s two properties. On the evidence gathered here, no other S0
tail deliverable (D5, D8) adds frame capture, so D4 plausibly resolves to **zero
file changes** in this cycle — the deliverable is satisfied by *not* building an
insecure receiver, which is enforceable by leaving `shot_server.py` untouched
and pointing any future capture need at it or an equivalent.

- **Existing files touched:** none required. `design/journeys/shot_server.py`
  (exists, 4142 bytes) is the file the constraint binds to, but nothing in D4's
  own text requires editing it.
- **Newly created files:** none required by current evidence. If an
  implementer chooses to add a regression test pinning `shot_server.py`'s two
  security properties, that would be a new file under `design/journeys/`
  (path not yet chosen — proposed, does not exist).
- **Shared resources:** none beyond ordinary git/filesystem access to the
  mnemonic-engrave repo.

## D5 — oracle resolution by source commit

**Does D5 have to modify `cmd/emu/walk_trace_a.js`?** No — determined, not
guessed, from two independent facts:

1. **Oracle resolution is impossible from inside the browser sandbox.**
   Resolving a primary binary to a source commit means shelling to `git`
   (`git rev-parse` against a pinned checkout) or hashing a binary on disk —
   filesystem/process operations `walk_trace_a.js` cannot perform; it runs
   inside the compiled `emu.wasm` page (`cmd/emu/index.html`), which is exactly
   the sandbox §4.6 already fences off ("Not tier 1: the harness shells out to
   primary binaries and builds wasm" — plan line 315). Grepped for precedent:
   `git grep -n "exec.Command"` across the fork shows shelling only to
   `rsvg-convert` (`cmd/glyphtrace/main.go:539`, `cmd/plateview/main.go:182`) —
   no existing code shells to `md`/`mk`/`ms`/`cargo`/`git`, confirming this is
   genuinely new plumbing, and it is Go-side plumbing by necessity.
2. **`walk_trace_a.js`'s `run()` already returns everything a host-side caller
   needs**, unedited: `pace`, `census`, `digests`, `gathered`, `acts`, `screen`
   (lines 251-265, read in full). Nothing currently *prints* or *persists* this
   return value anywhere — `git grep -in "gate record"` across the fork returns
   **zero hits** (the term exists only in the mnemonic-engrave design docs);
   the walk today is invoked interactively per its own header comment (`const w
   = await import("./walk_trace_a.js"); await w.run();`) and its return value is
   only ever inspected in a browser console. So "print the gate record" is
   **greenfield infrastructure regardless of D5** — whatever component ends up
   doing it can obtain the walk's own data by *importing/calling*
   `walk_trace_a.js`'s exported `run()` (already how its own docstring says to
   use it), not by editing it.

The plan text (§1a, lines 58-64) says "the walk script MUST resolve each oracle
to a source commit... and print that commit... into the gate record" — but
"the walk script" here is used as shorthand for the walk-plus-harness whole
(the same section's deliverable-list text at line 252 calls it "the harness"),
not literally the 266-line `walk_trace_a.js` file. The two named tests —
`TestOracleHarnessRefusesVendoredTestdata`, `TestOracleHarnessPinsBySourceCommit`
— are Go-test-shaped (`func TestXxx(t *testing.T)`, the fork's universal
convention, e.g. `cmd/emu/embed_confinement_test.go`), and their behavior
(refuse `md/testdata`; distinguish a spoofed `--version` from a real source
commit) is pure host-side logic, testable with no browser at all.

**Where would the harness live?** Not yet decided by the plan (**UNRESOLVED**,
a real design gap, not a fact I can resolve by reading). What I can rule in/out:

- **Ruled out:** `address/**` (occupied, D6/D7), `md/**` (D8's territory, and
  `cmd/emu` does not currently import `seedhammer.com/md` at all — `git grep -n
  "seedhammer.com/md" -- 'cmd/emu/*.go'` returns nothing, so there's no existing
  coupling that would pull D5 into `md/`).
- **Plausible, both disjoint from D5/D8:**
  - New Go file(s) under `cmd/emu/` (fits the existing pattern of host-side test
    infra co-located there: `embed_confinement_test.go`,
    `sysw_cards_payload_host_test.go`).
  - A new top-level package, e.g. `oracle/` (none of that name exists today;
    `ls -d */` in the fork root lists no such directory).
  - A companion script in `mnemonic-engrave/scripts/` is possible but not
    required by the plan text — the fork's own Go `exec.Command` machinery is
    sufficient to shell to primary binaries and to `git`, so nothing here
    structurally *requires* a second-repo script.
- Primary Rust source repos are available **locally**, not just theoretically:
  `/scratch/code/shibboleth/descriptor-mnemonic/`,
  `/scratch/code/shibboleth/mnemonic-toolkit/`,
  `/scratch/code/shibboleth/mnemonic-key/`,
  `/scratch/code/shibboleth/mnemonic-secret/` all exist as sibling checkouts,
  so source-commit resolution does not require network access on this machine.

- **Existing files touched:** none identified with confidence. (`git grep -in
  "OracleHarness"` across the fork: zero hits — nothing to converge with.)
- **Newly created files:** at least one new Go file carrying
  `TestOracleHarnessRefusesVendoredTestdata` and
  `TestOracleHarnessPinsBySourceCommit`, plus the resolver logic itself. Exact
  path/package is an open design decision, not yet made anywhere in the repo.
- **Shared resources:** read access to the primary Rust checkouts listed above;
  the primary `md`/`mk`/`ms` CLI binaries (see below); ordinary git/filesystem
  access in the fork. Does **not** need the emulator, the browser, or the wasm
  build to satisfy its own two named tests (they're pure resolver-logic unit
  tests) — only the end-to-end "prints into every gate record" integration
  would eventually touch the walk's output, and per above that's achieved by
  *calling* `run()`, not editing it.

**Primary CLI availability (measured, not assumed):** `md`, `mk`, `ms` are all
installed and on `PATH` via `~/.cargo/bin/{md,mk,ms}` (each present,
`ls -la` verified). Self-reported versions: `md 0.13.0`, `mk 0.12.1`,
`ms 0.14.0` — these are CLI-crate versions, not directly comparable to the
plan's cited library-crate pins (`md-codec 0.42.x`, `mk-codec 0.4.2`,
`ms-codec 0.7.0`); resolving what source commit these installed binaries
actually correspond to (verifying they are not stale relative to the primary
repos' current `HEAD`) is exactly the unbuilt logic D5 exists to add — I did
not attempt that resolution myself, since doing so *is* D5's own deliverable.

## D8 — md vendored-vector re-pin, 0.36.0 → current

**`md/testdata/**` enumerated in full**, `find md/testdata -type f | wc -l` =
**62 files**:

- `md/testdata/README.md` — 1 file. The provenance block D8's gate names
  explicitly (pins commit `c85cd49`, md-codec v0.36.0).
- `md/testdata/template/` — **7 files** (`ls md/testdata/template | wc -l`):
  `degrade2_11key.policy.md1.txt`, `degrade2_11key.tmpl.md1.txt`,
  `tr4_depth2.tmpl.md1.txt`, `wpkh.policy.md1.txt`, `wpkh.tmpl.md1.txt`,
  `wsh_sortedmulti.policy.md1.txt`, `wsh_sortedmulti.tmpl.md1.txt`. **Not
  in scope for the re-pin**: these carry no commit/version citation anywhere
  (`git grep -n "commit\|v0.3\|v0.4\|provenance" -- md/testdata/template/`
  returns zero hits) and their git history (`f70456f`, `d912bca`) is unrelated
  to the vendoring commit (`ac00093`, "md: vendor md-codec golden vectors for
  the encoder (#10a)") — they are fork-generated template fixtures, not
  vendored-from-primary vectors.
- `md/testdata/vectors/` — **54 files** (`ls md/testdata/vectors | wc -l`):
  the golden-vector triples (`.bytes.hex`/`.phrase.txt`/`.descriptor.json` for
  17 named sets, plus `.meta.json`/`.md1.txt`/`.xpub.txt` for the multisig and
  singlesig golden sets) and 2 provenance docs,
  `README_multisig.md` and `README_singlesig.md`, both of which **also**
  hard-cite the old commit inline (`README_multisig.md:10`:
  `descriptor-mnemonic @ c85cd49`; `README_singlesig.md:13-14`:
  `v0.36.0, git c85cd49`). **These need updating too**, not just the top-level
  `README.md` the plan text names — the plan's gate line only names
  `md/testdata/README.md`, but the citation is duplicated in two more files a
  faithful re-pin would leave stale otherwise.

**Go files that read `md/testdata/**`** (`git grep -l "testdata" -- 'md/*.go'`,
4 files):
- `md/encode_multisig_test.go` — reads `testdata/vectors/<name>.meta.json` and
  `.md1.txt`.
- `md/encode_singlesig_test.go` — same, for the singlesig sets.
- `md/template_strip_test.go` — reads `testdata/template/<name>` (the
  out-of-scope template files above).
- `md/testdata_test.go` — path helper, `testdata/vectors/<name>.<ext>`.

None of these four need editing for a byte-identical re-pin (the gate is
literally "`go test ./md/` passes" against re-pinned *data*, not new test
code) — listed because they are the readers, and would need to change only if
the re-pin adds/removes/renames a vector.

**Anything outside `md/` that would need touching?**
- `gui/template_engrave_test.go:199,214` reads two files under
  `md/testdata/template/` by relative path (`../md/testdata/template/...`) —
  but only the **template** files, which are out of scope for the vector
  re-pin (no version pin, see above), so this file needs no change for D8.
- Two more files **inside** `md/` (not outside, correcting scope but worth
  flagging since they're easy to miss) cite the old pin in prose/comments
  rather than data: `md/bits.go:3` ("format: descriptor-mnemonic/crates/md-codec
  @ 0.36.0") and `md/md_test.go:77,327` ("the verified md-codec 0.36.0 wire
  layout..."). The plan's stated gate (`go test ./md/` passes +
  `README.md`'s provenance block names the new commit/version) does not
  strictly require touching these comments, but a re-pin that updates the data
  and the READMEs while leaving these comments citing `0.36.0` would be
  internally inconsistent. Full repo-wide census of every file mentioning the
  old pin, both markers: `git grep -n "c85cd49"` → 3 files
  (`md/testdata/README.md`, `md/testdata/vectors/README_multisig.md`,
  `md/testdata/vectors/README_singlesig.md`); `git grep -n "0\.36\.0"` → 5
  files (those 3 plus `md/bits.go`, `md/md_test.go`). **All 5 are inside
  `md/`.** Zero hits outside `md/` for either marker.

- **Existing files touched:** `md/testdata/README.md`,
  `md/testdata/vectors/README_multisig.md`,
  `md/testdata/vectors/README_singlesig.md` (provenance text, all 3 verified
  to exist and cite the old commit), the 54 `md/testdata/vectors/**` data
  files (verified to exist, re-pinned to new byte content), optionally
  `md/bits.go` and `md/md_test.go` (comment consistency, not gated). Measured
  2026-08-13 elsewhere in this plan: 0.36→0.42 shows **zero byte drift** across
  all 30 MANIFEST vectors, so per the plan's own prior measurement this re-pin
  is expected to touch commit/version text far more than vector bytes.
- **Newly created files:** none required — this is a re-pin of existing
  vendored data, not new coverage.
- **Shared resources:** the primary `md` toolchain/source (to regenerate
  vectors against "current"), which is the same local checkout
  (`/scratch/code/shibboleth/descriptor-mnemonic/` and/or
  `/scratch/code/shibboleth/mnemonic-toolkit/`) D5 would also read — **read
  access**, not a write conflict; both can read the same checkout
  concurrently.

## INTERSECTION

Pairwise file-path overlap, computed from the sets above:

- **D4 ∩ D5:** empty. D4's file (if any) is `design/journeys/shot_server.py`
  in the **mnemonic-engrave** repo; D5's files are all in the **seedhammer**
  fork. Different repos — structurally disjoint.
- **D4 ∩ D8:** empty. Same reasoning — different repos.
- **D5 ∩ D8:** empty on all evidence gathered. D8's file set is fully
  enumerated above and is entirely inside `md/`. D5's exact location is
  unresolved, but every location ruled in above (`cmd/emu/` or a new
  `oracle/`-style package) is outside `md/`, and `cmd/emu` does not currently
  import `seedhammer.com/md` at all, so there is no existing coupling that
  would force D5 into `md/testdata/` or `md/*.go`. **Caveat:** if a future
  implementer chooses to nest D5's oracle-resolution logic *inside* the `md`
  package (not ruled out by the plan text, just not suggested by anything
  found here), this would become non-empty. Flagged in UNRESOLVED.

**All three intersections are empty on current evidence.** The one caveat
worth restating from the SUMMARY: D5 and D8 share the same Go module
(`seedhammer.com`) and, if D5 lands in `cmd/emu/`, the same directory family as
other S0 work — so "disjoint files" still means both must build against a
green, clean baseline (`gofmt -l`, `go vet ./...`, `go test ./...`), per the
prior report's guideline 7-8. That is a shared-baseline concern, not a file
collision.

## UNRESOLVED

1. **D5's exact file location** — `cmd/emu/` new files, a new top-level
   `oracle/` package, or something else. Not decided anywhere in the plan or
   the tree. I ruled out `address/` and made a documented case against `md/`,
   but did not find a definitive answer.
2. **Whether D4 requires any new file at all.** My reading (zero named tests
   for D4, and no frame-capture path in either the done S0 work or my best
   read of D5) is that D4 is satisfiable with zero file changes this cycle.
   This is an interpretation of the plan's intent, not a fact I can verify by
   running a command — the plan's author would need to confirm whether a
   defensive regression test on `shot_server.py`'s two properties is wanted
   regardless.
3. **Whether the installed primary CLIs (`md 0.13.0`/`mk 0.12.1`/`ms 0.14.0`
   on `PATH`) actually correspond to the plan's cited pins**
   (`md-codec 0.42.x`/`mk-codec 0.4.2`/`ms-codec 0.7.0`) or are stale relative
   to the primary repos' current `HEAD`. I deliberately did not resolve this —
   doing so is D5's own deliverable, not a fact for this recon to settle.
4. **Whether a re-pin's exact target commit for D8** ("current") has been
   chosen yet. Not addressed here — out of scope for a file-set enumeration,
   but the eventual implementer will need to pick one from
   `/scratch/code/shibboleth/descriptor-mnemonic/`.
