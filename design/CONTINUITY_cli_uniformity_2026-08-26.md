# CONTINUITY — the CLI-uniformity cycle, 2026-08-26

## State in one line

**The SPEC is NOT GREEN.** Eight review rounds done; the ninth
(`R0-cli-uniformity-rulings.md`, 0C/4I/5M/1N) is **persisted and unfolded**.
No plan may be written until R0 closes 0C/0I.

## The operator's approved plan — four steps, in order

1. **Fold** the rulings review's 4 Importants + 5 Minors, plus two controller
   finds, with §5c marked **decided, not scheduled**.
2. **Fix the publish gate** — the recorded crates.io command needs `-A`.
3. **Re-gate + a sonnet claim-check** (mechanical: did the fold match, did it
   introduce anything).
4. **Write the P0 plan.**

## The ruling that shapes step 1

**The §5c verb migration is DECIDED, NOT SCHEDULED.** `split`, `combine`,
`compile`, `derive` and `address` move to the toolkit — but in **their own
cycle**, not this one. This spec is CLI *uniformity*: its decisions are IO and
safety, and its phases are structured around adopting the shared crate.
Absorbing a verb-relocation program would roughly double the cycle and delay the
shared safety layer, which is the part with security value.

## What the ninth round found — the two that matter

**I-1 — "There is no `mnemonic-*` library" is FALSE.** `mnemonic-engrave`
declares `[lib] name = "mnemonic_engrave"` and is published. **The method was
circular:** the evidence offered was `cargo install --list`, which enumerates
**binaries by construction** and can never show a library. *The operator's
ruling survives* — the corrected fact, "no `mnemonic-*` package is
library-**only**", still supports the `-lib` suffix.

**I-2 — the publish gate cannot distinguish free from taken.** The command as
recorded omits the User-Agent that was actually used:

```
no  -A:   mnemonic-io-lib → 403      serde → 403
with -A:  mnemonic-io-lib → 404      serde → 200
```

`serde` is definitely taken. **P0 runs this command immediately before an
irreversible publish**, and as written it returns 403 for everything. Sixth
instance in this document of a true number beside a command that cannot produce
it — and the first wired to something unrecoverable.

## Controller finds, not in the report

- **No phase owns the verb migration.** Across §7's phase rows: `split` 0,
  `combine` 0, `compile` 0, `address` 0, `toolkit` 0. Step 1's "decided, not
  scheduled" sentence is what closes this. The reviewer found the other half —
  §7 P2 still *schedules* `--in`/argv work **on** `ms split`/`combine`/`derive`.
- **A stale bullet** still declares the two `mnemonic` exit-code cells open.
  Both are measured. It is the **third** site of the M-4 fold, closed at two.

## Settled — do not re-derive

- `repair` **stays** in the encoders; D26's parity set is intact.
- The crate is **`mnemonic-io-lib`**, operator-approved, genuinely free
  (404/200 discrimination confirmed **with** `-A`).
- §5b: `encode`/`decode`/`verify`/`inspect` in every `m*-cli`; criterion is
  *basic m-string manipulations*; 16/16 verified present.
- `plan-glyph-check.sh` exits 1 and is **out of domain** for this artifact
  (F-257). Not a finding.

## P0's first move is NOT where it looks

`me-cli` is the only crate with both `lib.rs` and `main.rs`, which reads like a
head start. It is not: **`write_private` and `stdout_world_readable_mode` are
both in `main.rs`** — the binary half. Only `is_argv_forbidden` is already in
the library. So P0 starts with a move *inside* `me`, before anything crosses a
crate boundary.

## Method requirement for every agent dispatched from here

**Invoke binaries by ABSOLUTE PATH.** The login shell aliases `md` to
`mkdir -p`; a bare `md decode x` returns **exit 0 and silently creates
directories**. Two separate reviewers hit this, one nearly filing a fabricated
Critical. Redirect stdin from `/dev/null`, and never read an exit code through a
pipe.

## Outside this cycle

- **The tag** on the tx-engraving cycle — everything else is shipped and pushed
  (`origin/master` 9c6214a, fork 18c7522, both API-confirmed).
- **39 untracked design files** — 38 in `mnemonic-toolkit`, 1 in `mnemonic-key`.
  Uncommitted work in trees nobody watches; unrelated to disk.
- **F-255** (`md` shadowed) and **F-247** (NFC fit) await operator rulings.
