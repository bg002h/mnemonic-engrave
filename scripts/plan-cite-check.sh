#!/usr/bin/env bash
# plan-cite-check.sh — resolve every `path:line` citation in a design doc against
# the real source tree, and print the line each one actually points at.
#
# WHY THIS EXISTS. The S5 cycle measured the difference between gated and ungated
# facts in this repo's design docs: 16 of 16 gated code citations were true, and
# 5 of 22 ungated facts were false. Citations also decay on every merge — a line
# number that was right when written silently drifts to point at something else.
# A reviewer re-deriving forty anchors by hand is a reviewer paid design-review
# rates to act as grep, and the finding arrives a round late.
#
# A fold is authorship and re-earns this gate: run it again before committing any
# review response, because the fold is the text nobody has read.
#
# USAGE
#   ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_foo.md [more.md ...]
#
# EXIT
#   0  every citation resolved to an existing file and an in-range line
#   1  at least one citation is dangling (missing file, or line past EOF)
#
# ─── WHAT THIS DOES NOT COVER — stated because a gate that hides its blind ────
#     spot is worse than no gate at all.
#
#   * It proves a cited line EXISTS and shows what it says. It does NOT prove the
#     doc's INTERPRETATION of that line is correct. Reading the printed line
#     against the doc's claim is still a reviewer's job — but it is now a read,
#     not a lookup.
#   * It cannot check claims of ABSENCE ("no test pins this label", "zero call
#     sites"). Those are grep-negatives, and an empty result is also what a
#     broken query looks like. Docs must carry the command that produced each
#     negative, with a positive control.
#   * It does not compile code blocks. Fragments against a package the
#     implementer will edit remain a reviewer's execution pass.
#   * It only resolves citations that LOOK like `path/file.ext:N`. A claim
#     written as prose ("the flow derives unconditionally") is invisible to it.

set -uo pipefail

# Roots to resolve a citation against, in order. A `gui/...` path lives in the
# fork; a `crates/...` path lives here.
#
# THE FORK ROOT IS OVERRIDABLE, AND IT HAS TO BE. The default is the fork's
# checked-out `main`, so a doc citing UNMERGED branch work is resolved against a
# tree that does not contain it -- and this script only checks that a line is
# IN RANGE, never what is on it. So it prints `ok` for a citation pointing at an
# unrelated line, which is the worst possible output: a gate that reports
# success for the case it cannot see.
#
# Measured 2026-08-18, folding the falsified-elsewhere sweep: `gui/singlesig.go
# :217` (`for {` on branch s6b-pre-flash) resolved against fork main to a
# comment line from a different function, and was reported `ok`.
#
#   CITE_FORK_ROOT=/scratch/code/shibboleth/wt-s6b ./scripts/plan-cite-check.sh doc.md
#
# Set it to the worktree under review whenever the doc cites work that has not
# landed on `main` yet.
# The constellation is MULTI-REPO and the design docs cite across it freely.
# Until 2026-08-18 this list held only the fork and this repo, so every citation
# into descriptor-mnemonic or mnemonic-toolkit came back DANGLING — measured:
# 0 of 8 resolved for design/DoNextList.md, none of which was actually a wrong
# citation. A gate that reports a false DANGLING is worse than no gate, because
# it trains the reader to ignore its output.
#
# Order matters only for the advisory line printed at the end (ROOTS[0] is
# named there as "the fork root"); resolution tries every entry.
# THIS REPO is located from git, not hardcoded, so the script works from any
# checkout path. The sibling constellation repos are still absolute, and that
# is a real limitation — see the CI note below.
SELF_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo /scratch/code/shibboleth/mnemonic-engrave)"

ROOTS=(
  "${CITE_FORK_ROOT:-/scratch/code/shibboleth/seedhammer}"
  "$SELF_ROOT"
  "/scratch/code/shibboleth/descriptor-mnemonic"
  "/scratch/code/shibboleth/mnemonic-toolkit"
  "/scratch/code/shibboleth/mnemonic-key"
  "/scratch/code/shibboleth/mnemonic-secret"
  # mnemonic-transaction is LAST, deliberately. It is the only sibling that
  # carries a copy of a design document this repo also owns
  # (design/SPEC_mt_v0_1.md exists in both), and resolution tries the roots in
  # order -- so a `design/...` citation must land in THIS repo, not in the
  # sibling's copy of it. Its own code lives under `crates/mt-cli/` and
  # `crates/mt-codec/`, which collide with nothing above.
  #
  # Added 2026-08-27 for the P1 plan, whose subject IS `mt`: without this root
  # every one of its citations came back DANGLING, and the P0 plan's answer --
  # write sibling references as prose without `path:line` punctuation -- does
  # not scale from a handful to a document where nearly every anchor is
  # cross-repo. A gate that reports a false DANGLING trains the reader to skim
  # its output, which is the argument that added descriptor-mnemonic and the
  # toolkit above.
  "/scratch/code/shibboleth/mnemonic-transaction"
)

# ─── WHY THIS GATE IS NOT IN CI, measured 2026-08-18 ──────────────────────────
#
# It cannot be, as written. CI checks out THIS repo plus the `seedhammer`
# submodule at `third_party/seedhammer`. It has no
# `/scratch/code/shibboleth/...` at all, so every sibling root above is absent
# and the script would report 100% DANGLING regardless of citation quality —
# a gate that is always red teaches people to ignore it just as surely as one
# that is always green.
#
# Making it CI-able needs a decision, not a patch: either check the sibling
# repos out in the workflow (slow, and pins their versions), or accept that
# cross-repo citations are developer-machine-only and gate just the in-repo
# ones. Separately, the corpus is not currently clean — a 12-doc sample of the
# 201 top-level design docs found 13 dangling citations across 7 of them — so
# any CI adoption must be scoped to CHANGED docs, which is green by
# construction and matches the "a fold is authorship and re-earns the gate"
# rule anyway.

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <doc.md> [doc.md ...]" >&2
  exit 2
fi

# The citation loop runs in a subshell (it is on the right of a pipe), so counts
# cannot come back through variables. They come back through this file.
TALLY=$(mktemp)
trap 'rm -f "$TALLY"' EXIT

for doc in "$@"; do
  if [ ! -f "$doc" ]; then
    echo "MISSING DOC: $doc" >&2
    bad=$((bad + 1))
    continue
  fi

  echo "═══ $doc"

  # Pull out `some/path/file.ext:N`, `:N-M` and `:N,M,K` forms. The trailing
  # number group is expanded below, so one citation with three line numbers is
  # checked as three citations.
  grep -oE '(\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(go|rs|sh|md|toml|yml|yaml):[0-9]+([,-][0-9]+)*' "$doc" \
    | sort -u \
    | while IFS= read -r cite; do
        path="${cite%%:*}"
        nums="${cite#*:}"
        path="${path#./}"

        # Locate the file. AMBIGUITY IS AN ERROR, NOT A TIE-BREAK.
        #
        # This used to `break` on the first root that had the path, which in a
        # multi-repo constellation silently resolves a bare `Cargo.toml:29`
        # against whichever repo happens to be listed first — measured on
        # 2026-08-18: it reported `Cargo.toml:29 (file has 6 lines)` for a
        # citation meant for mnemonic-toolkit's 6-line-plus workspace manifest.
        # A gate that resolves a citation to the WRONG repo's file and prints
        # `ok` is worse than no gate at all, because the reader then trusts it.
        # REPO-QUALIFIED FORM: `descriptor-mnemonic/crates/…/x.rs:12`.
        #
        # This is how an author disambiguates a path that genuinely exists in
        # several repos — `Cargo.toml`, `vendor/miniscript/…`, `README.md`.
        # If the first segment matches a root's basename, that root is the ONLY
        # one tried, and the rest of the citation is taken as relative to it.
        file=""
        hits=0
        prefix="${path%%/*}"
        for root in "${ROOTS[@]}"; do
          if [ "$(basename "$root")" = "$prefix" ] && [ -f "$root/${path#*/}" ]; then
            file="$root/${path#*/}"
            hits=1
            break
          fi
        done

        if [ -z "$file" ]; then
          for root in "${ROOTS[@]}"; do
            if [ -f "$root/$path" ]; then
              hits=$((hits + 1))
              [ -z "$file" ] && file="$root/$path"
            fi
          done
        fi
        if [ "$hits" -gt 1 ]; then
          printf 'AMBIGUOUS %-52s (exists under %d roots — qualify the path)\n' \
            "$cite" "$hits"
          echo "TOTAL" >>"$TALLY"
          echo "BAD"   >>"$TALLY"
          echo "AMB"   >>"$TALLY"
          continue
        fi

        if [ -z "$file" ]; then
          printf 'DANGLING  %-52s  (no such file under any root)\n' "$cite"
          # Counted in BOTH tallies: a citation whose file cannot be found is
          # still a citation. Omitting it from the total would report "43 / 47
          # resolved" for a doc with 47 good citations and 4 broken ones.
          echo "TOTAL" >>"$TALLY"
          echo "BAD" >>"$TALLY"
          continue
        fi

        eof=$(wc -l <"$file" | tr -d ' ')

        # Expand `a-b` to its endpoints and `a,b,c` to each element. A range is
        # checked at both ends rather than every line, which is what catches the
        # common decay (the block moved) without printing fifty lines.
        for n in $(echo "$nums" | tr ',-' ' '); do
          echo "TOTAL" >>"$TALLY"
          if [ "$n" -gt "$eof" ] || [ "$n" -lt 1 ]; then
            printf 'DANGLING  %-52s  (file has %s lines)\n' "$path:$n" "$eof"
            echo "BAD" >>"$TALLY"
            continue
          fi
          text=$(sed -n "${n}p" "$file" | sed 's/^[[:space:]]*//' | cut -c1-96)
          printf 'ok  %-52s  %s\n' "$path:$n" "$text"
        done
      done
done

# `grep -c` PRINTS "0" and EXITS 1 when there is no match, so the obvious
# `$(grep -c ... || echo 0)` yields the two-line string "0\n0" and every later
# arithmetic test dies. `|| true` keeps grep's own "0" and swallows only the
# status. (This is the same shape as the S5 lesson that an empty result is also
# what a broken query looks like — here the gate caught it on itself.)
total=$(grep -c '^TOTAL$' "$TALLY" || true)
bad=$(grep -c '^BAD$' "$TALLY" || true)
amb=$(grep -c '^AMB$' "$TALLY" || true)
: "${total:=0}" "${bad:=0}" "${amb:=0}"

echo
echo "─── citations resolved: $((total - bad)) / $total ; dangling: $((bad - amb)) ; ambiguous: $amb"
echo "─── resolved against fork root: ${ROOTS[0]}"
echo "─── NOT covered: interpretation, absence-claims, code-block compilation,"
echo "───              and WHAT IS ON THE LINE -- only that the line exists. A"
echo "───              citation to unmerged work resolves against whatever is at"
echo "───              that line in the root above; set CITE_FORK_ROOT to the"
echo "───              worktree under review, or this prints ok for a wrong line."

[ "$bad" -eq 0 ]
