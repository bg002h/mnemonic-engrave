# S0 D8 — md vendored golden-vector re-pin, 0.36.0 → current

## STATUS

DONE

## PIN

- **Old:** commit `c85cd49`, md-codec v0.36.0 (recorded in the pre-existing
  `md/testdata/README.md`; not re-derived, per brief).
- **New:** commit `5a0a4f41` (full: `5a0a4f41017d71d47f70684c145702d4ca0c3aa9`),
  md-codec v0.42.0, tag `md-cli-v0.13.0`.
  - Commit: `git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse HEAD`
    → `5a0a4f41017d71d47f70684c145702d4ca0c3aa9`; confirmed clean/current via
    `git -C /scratch/code/shibboleth/seedhammer status --porcelain` on the fork
    (clean at dispatch) and `git -C descriptor-mnemonic log -1 --format='%H %s'`
    → `5a0a4f41... release: md-codec 0.42.0 + md-cli 0.13.0 — pathless/dead-card
    partial-decode`.
  - Version: `grep -m1 '^version' crates/md-codec/Cargo.toml` →
    `version = "0.42.0"` (read from the crate manifest, not inferred from the
    commit subject, per instructions).
  - Tag: `git -C descriptor-mnemonic describe --tags --always` → `md-cli-v0.13.0`
    (HEAD sits exactly on this tag, no `-N-g<sha>` suffix).

## VECTOR DELTA

Primary vector directory: `descriptor-mnemonic/crates/md-codec/tests/vectors/`
(`find ... -type f | wc -l` = **62**: 15 named vector stems × 4 files
(`.bytes.hex`/`.descriptor.json`/`.phrase.txt`/`.template`) = 60, plus
`bip341-wallet-test-vectors.json` and `.gitkeep`). The primary's `MANIFEST`
(`crates/md-codec/src/test_vectors.rs`) is now doc-labeled a "canonical
15-entry corpus" — the original 10 plus 5 Part-3 additions.

The fork vendors only 10 of the 15 primary stems, as 3 files each (no
`.template`) = 30 files, inside `md/testdata/vectors/` (54 files total there,
the other 24 being fork-native `multisig_*`/`singlesig_*`/`README_*` fixtures
not sourced from the primary's `tests/vectors/`, plus 7 files under
`md/testdata/template/` — out of scope, no commit/version citation found via
`git grep`).

- **Identical (30/30 files, 10/10 shared vectors):** `pkh_basic`,
  `sh_wsh_multi`, `tr_keyonly`, `wpkh_basic`, `wsh_divergent_paths`,
  `wsh_multi_2of2`, `wsh_multi_2of3`, `wsh_multi_chunked`, `wsh_sortedmulti`,
  `wsh_with_fingerprints` — each `.bytes.hex`/`.descriptor.json`/`.phrase.txt`
  byte-identical via `cmp -s` between
  `descriptor-mnemonic/crates/md-codec/tests/vectors/` and
  `seedhammer/md/testdata/vectors/`, re-confirmed by physically re-copying all
  30 files from the primary and running `git diff --stat` afterward (0 files
  changed, 0 insertions/deletions on the data files). This reproduces the
  plan's 2026-08-13 "zero byte drift across all 30 vectors" measurement at the
  current commit.
- **NEW in primary (not vendored — out of scope for this re-pin):**
  `sh_wpkh`, `tr_with_leaf`, `nums_taproot`, `wsh_sortedmulti_2chunk`,
  `single_string_boundary` (5 stems, the Part-3 additions). Per
  `design/agent-reports/s0-tail-file-sets.md`'s D8 scope ("Newly created
  files: none required — this is a re-pin of existing vendored data, not new
  coverage"), these are NOT copied in. Flagged in `md/testdata/README.md` by
  name so a future coverage-expansion deliverable knows what's missing.
- **DISAPPEARED:** none. All 10 previously-vendored stems still exist in the
  primary, unchanged.
- **CHANGED (stop condition):** none. Zero byte drift — task proceeded to
  completion rather than stopping.

## FILES CHANGED

`git -C /scratch/code/shibboleth/seedhammer diff --name-only`:

```
md/bits.go
md/md_test.go
md/testdata/README.md
md/testdata/vectors/README_multisig.md
md/testdata/vectors/README_singlesig.md
```

`git -C /scratch/code/shibboleth/seedhammer status --porcelain`:

```
 M md/bits.go
 M md/md_test.go
 M md/testdata/README.md
 M md/testdata/vectors/README_multisig.md
 M md/testdata/vectors/README_singlesig.md
```

No untracked additions — the 30 vector data files were re-copied from the
primary byte-for-byte, so `git status` shows no change for them (content was
already current). `md/bits.go` diff touches only line 3; `md/md_test.go` diff
touches only lines 77 and 327 — both verified via `git diff` on those two
files individually (single-hunk, single-line-per-hunk changes, no line-count
shift). All 5 changed paths are within the write boundary
(`md/testdata/**`, `md/bits.go` line 3, `md/md_test.go` lines 77/327).

### Provenance-text changes, by file

- `md/testdata/README.md`: commit/version citation bumped to `5a0a4f41` /
  v0.42.0 (+ tag); added a re-pin note recording the zero-drift measurement,
  the primary's new 15-entry `MANIFEST` size, and naming the 5 not-yet-vendored
  additions.
- `md/testdata/vectors/README_multisig.md`: the `c85cd49` Rust-CLI
  cross-check citation is preserved as a dated historical record (its literal
  command outputs were captured at that commit and were NOT re-executed
  against `5a0a4f41`), with an added note bridging to the current pin and
  explaining why the conclusion still stands (encode_payload wire format
  confirmed byte-identical 0.36.0 → 0.42.0).
- `md/testdata/vectors/README_singlesig.md`: same treatment — the
  `mnemonic-toolkit`/md-codec 0.36.0 generation lineage for the `singlesig_*`
  goldens is left as an accurate historical record (that toolkit run was not
  repeated at the new pin), with an added "Re-pin note" stating the current
  primary pin and the byte-identical-wire-format justification for why these
  goldens are not known to be stale.
- `md/bits.go:3`, `md/md_test.go:77,327`: `0.36.0` → `0.42.0`, single-line
  swaps only, no line-count change.

**Design choice, stated explicitly:** for `README_multisig.md` and
`README_singlesig.md` I did not overwrite the historical generation-lineage
details (toolkit binary SHA `4e21d94`, the literal `md encode` CLI outputs,
the Cargo.lock-verified `0.36.0`/`ms-codec 0.4.4` pins) to claim they were
produced at the new commit, since I did not re-run the toolkit or the CLI
cross-check — doing so would have been a false provenance claim outside this
task's scope (regenerating those goldens is a materially bigger task than a
citation re-pin). Instead each file now states the current pin explicitly
(satisfying "update provenance to name the new commit and version") while
preserving the true history of how the existing bytes were made.

## GATE

`export PATH="/nix/var/nix/profiles/default/bin:$PATH"; cd
/scratch/code/shibboleth/seedhammer; nix develop --command go test ./md/`:

```
warning: Git tree '/scratch/code/shibboleth/seedhammer' is dirty
ok  	seedhammer.com/md	0.035s
```

(Also ran `go test ./md/ -v`: every named test and fuzz seed-corpus entry
PASSed, ending `PASS` / `ok seedhammer.com/md 0.039s`. Also ran `gofmt -l
md/bits.go md/md_test.go` — no output, both clean.)

## UNRESOLVED

1. The `md/testdata/vectors/README_multisig.md` Rust-CLI cross-check (3
   `md encode ... --json` commands, hex outputs `0x36d1b`/`0x58624`/`0x90289`)
   and the `README_singlesig.md` `mnemonic-toolkit` generation run were NOT
   re-executed against the new pin — only their citations were annotated with
   a bridge note. If a future reviewer wants those specific golden sets
   independently re-verified at `5a0a4f41`/v0.42.0 (not just inferred safe via
   the shared wire-format zero-drift result), that is unactioned re-generation
   work, out of scope for this deliverable and outside my write boundary
   (would touch `md/encode_multisig_test.go` / `md/encode_singlesig_test.go`
   readers only if the *data* changed, which it did not).
2. The 5 new primary MANIFEST vectors (`sh_wpkh`, `tr_with_leaf`,
   `nums_taproot`, `wsh_sortedmulti_2chunk`, `single_string_boundary`) are
   named in `md/testdata/README.md` but not vendored — a coverage-expansion
   deliverable, deliberately deferred per this task's brief and the prior
   recon's scope call ("Newly created files: none required").
3. `md/testdata/template/` (7 files) was left untouched — confirmed out of
   scope by `git grep -n "commit\|v0.3\|v0.4\|provenance" -- md/testdata/template/`
   returning zero hits (re-verified: same result at the current tree state).
