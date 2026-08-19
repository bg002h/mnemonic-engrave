# Round-trip journey recon — `shibboleth-wallet`

**Scope:** read-only inventory of what round-trip journeys already exist in
`/scratch/code/shibboleth/shibboleth-wallet`, per the definition in
`design/DRAFT_round_trip_journey_definition.md` (mnemonic-engrave repo),
sections 1–7, with §8 operator rulings binding (decoder reads rendered
preview; fixed seed OK; inventory-what-exists only; passphrase/network/account
are variations not separate journeys — none of which end up applying here,
since there is nothing to apply them to).

## What I actually ran

```
cd /scratch/code/shibboleth/shibboleth-wallet && git status / git log     # -> "fatal: not a git repository"
ls -la /scratch/code/shibboleth/ ; ls -la /scratch/code/shibboleth/shibboleth-wallet
find . -maxdepth 5 -type f | sort
find . -type f | wc -l
find . -iname ".git*"
du -sh .
find .                       # unlimited depth, all entries incl. hidden
find . -type d                # directories
find . -type l                # symlinks
stat COLDCARD_BIP388.md REFERENCES.md
grep -inE "journey|round.?trip|\bcli\b|test_|#\!/|fn main|def main|npm|cargo|go build" COLDCARD_BIP388.md REFERENCES.md
grep -inE "mnemonic|seedhammer|md1|mk1|ms1|codex32|slip39|constellation|engrave" COLDCARD_BIP388.md REFERENCES.md
```
Plus full `Read` of both files in the repo (not excerpted, not inferred from
filename).

## What this repo IS

`shibboleth-wallet` is **not a code repository** — it is not even a git
repository (`git status` fails with "fatal: not a git repository"). It
contains exactly **two files, zero subdirectories, zero code, zero tests,
zero scripts, zero CLI entry points**, totalling 20 KB on disk:

- `COLDCARD_BIP388.md` (8208 bytes) — a constraints summary for a *planned*
  wallet, derived from reading BIP 388 and Coldcard's edge-firmware docs:
  what a future "Shibboleth Wallet" must/must-not do to register a miniscript
  wallet policy with a Coldcard and drive it for signing.
- `REFERENCES.md` (7659 bytes) — a tiered reading list / research plan
  ("Suggested reading order (1-2 week plan)") for choosing an engine (BDK vs.
  bdk-ffi vs. libnunchuk) before implementation starts.

Both files were created 2026-04-26 and have not been modified since (mtime
== ctime-of-creation for both; the one later ctime bump at 16:56 the same day
is a metadata-only change, e.g. a move, not a content edit). Neither file
contains executable content — the grep for CLI/test/shebang/build-tool
markers hit only prose: two mentions of "bdk-cli" as an external tool name
and three prose uses of "round-trip"/"roundtrip" describing PSBT semantics
and a future serializer's design goal, not a journey or test.

**Relation to the constellation:** the two files do not reference
`mnemonic`, `seedhammer`, `md1`/`mk1`/`ms1`, `codex32`, `slip39`, or
`engrave` anywhere except one generic "BIP 39 (mnemonic seed)" list entry.
This is pre-implementation planning for a *separate*, not-yet-started
Bitcoin cold-wallet application (PSBT + arbitrary miniscript + Coldcard
hardware-signer support) that happens to live under the same
`/scratch/code/shibboleth/` parent directory as the mnemonic-* constellation.
Nothing in these two files establishes it as a consumer of `md1`/`mk1`
output; that would have to be inferred, not measured, so I am not asserting
it.

## Journey inventory: NONE FOUND

**Zero round-trip journeys exist in this repo**, at any tier, generative or
custodial. This is established positively, not by a single empty grep:

1. **Positive file enumeration**, not a pattern search: `find . -type f`
   returns exactly 2 files; `find .` (unlimited depth) confirms no
   subdirectories, no hidden files, no symlinks beyond those 2 regular files.
   A wrong grep pattern cannot hide a journey from a full file listing.
2. **No git history to hold one**: the directory is not a git repository at
   all (`git status` errors), so there is no possibility of a journey that
   existed in a prior commit and was deleted, nor any commit log to search.
3. **No executable surface**: no `.sh`, `.rs`, `.go`, `.py`, `.toml`,
   `Cargo.toml`, `go.mod`, `package.json`, `Makefile` — nothing that could
   be "the one command that runs it." A journey requires a
   single-command-executable path (§5); there is no command of any kind here.
4. **Content-level check of the only two files that exist**: read in full
   (not by doc-comment or filename), both are prose. No test vectors, no
   fixed seeds, no CLI invocations, no assertions of any kind.

Since no journey exists, the §7 unit schema (name / kind / tier / origin /
invocations / structural assertion / functional assertion / one command /
non-coverage) has nothing to instantiate, and the §5 anti-requirement checks
(reads an unwritten intermediate; asserts against its own output; skip-that-
passes; a never-run gate; empty-output-as-absence) are **not applicable** —
there is no step, gate, or assertion in this repo for any of them to apply
to. This is different from "checked and clean"; it is "there is no journey
for the rule to bind."

## Known blind spot (per §8.3, restated for this repo)

A per-repo sweep — this one included — cannot see gaps *between* repos. If a
future generative journey is meant to run `mnemonic-toolkit` or
`mnemonic-engrave` to produce a card and then feed it into a not-yet-built
Shibboleth Wallet importer, that link cannot exist yet: there is nothing on
this side to link to. That absence is total right now (no code at all, per
above), not partial.

## Bottom line

`shibboleth-wallet` currently holds two static planning documents for a
future application and contains no round-trip journeys — no journeys of any
kind, because it contains no code. "None found" here is not a coverage gap
in an existing surface; there is no surface yet.
