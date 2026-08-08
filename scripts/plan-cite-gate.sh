#!/usr/bin/env bash
#
# plan-cite-gate.sh -- resolve every `file:line` and `pkg.Symbol` citation in a
# plan against the REAL source, so a reviewer never spends a round being a grep.
#
# WHY THIS EXISTS (F-68)
#   plan-build-gate.sh compiles the Rust a plan carries. Plan B carries almost no
#   code -- it carries CITATIONS: "md.ParseChunkHeader (md/chunk.go:185)",
#   "layoutNavigation indexes a fixed [3]int (gui/gui.go:1857)". Those are just as
#   machine-checkable as a code block, and just as expensive to get wrong: the
#   spec this plan descends from cited driver/otp/otp_rp2350.go:13 as precedent
#   for a fixed-address read, and line 13 is a #define. That citation survived
#   nine rounds of review because every reviewer read it as authoritative.
#
#   A citation decays every time someone edits the file above it. This resolves
#   them at review time and PRINTS WHAT IS ACTUALLY THERE.
#
# WHAT IT DOES
#   1. `path.go:NNN`  -> file exists? has >= NNN lines? print line NNN.
#   2. `pkg.Symbol`   -> a top-level `func`/`type`/`const`/`var Symbol`, or an
#                        entry inside a grouped `const (` / `var (` block,
#                        exists in package pkg? Grouped hits print as `ok*`.
#   3. Reports every unresolvable citation as a FAILURE.
#
# WHAT IT DOES NOT DO
#   It cannot tell you the line SAYS what the plan claims -- only that it exists
#   and what it contains. Reading the printed line against the claim is still a
#   reviewer's job. A gate that hides its blind spot is worse than no gate.
#
#   The `ok*` (grouped-declaration) pattern is looser than the top-level one: it
#   matches an INDENTED line beginning with the symbol, which an assignment
#   inside a function body can also satisfy. The matched line is printed for
#   exactly that reason. `ok` is a declaration; `ok*` is "a line that looks like
#   one" -- read it.
#
# Usage: scripts/plan-cite-gate.sh <plan.md> [repo-root-for-go-citations]
set -uo pipefail

PLAN="${1:?usage: plan-cite-gate.sh <plan.md> [go-repo-root]}"
GOREPO="${2:-/scratch/code/shibboleth/seedhammer}"
[ -f "$PLAN" ] || { echo "no plan at $PLAN" >&2; exit 2; }
[ -d "$GOREPO" ] || { echo "no repo at $GOREPO" >&2; exit 2; }

fail=0
echo "== file:line citations =="
# Backtick-quoted path:line, e.g. `gui/gui.go:1857` or `md/chunk.go:52`
grep -oE '`[a-zA-Z0-9_./-]+\.(go|rs):[0-9]+`' "$PLAN" \
  | tr -d '`' | sort -u | while IFS=: read -r path line; do
    # Go citations resolve against the fork; .rs against this repo.
    case "$path" in
      *.rs) root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" ;;
      *)    root="$GOREPO" ;;
    esac
    # Try the path as given, then search for it by basename.
    f="$root/$path"
    if [ ! -f "$f" ]; then
      f="$(find "$root" -path "$root/third_party" -prune -o -path "*/$path" -print 2>/dev/null | head -1)"
    fi
    if [ -z "$f" ] || [ ! -f "$f" ]; then
      printf '  FAIL  %-42s no such file\n' "$path:$line"; echo FAILMARK >> /tmp/.citegate
      continue
    fi
    n=$(wc -l < "$f")
    if [ "$line" -gt "$n" ]; then
      printf '  FAIL  %-42s file has only %s lines\n' "$path:$line" "$n"
      echo FAILMARK >> /tmp/.citegate
      continue
    fi
    printf '  ok    %-42s | %s\n' "$path:$line" \
      "$(sed -n "${line}p" "$f" | sed 's/^[[:space:]]*//' | cut -c1-72)"
  done

echo
echo "== pkg.Symbol citations (Go) =="
grep -oE '`(md|mk|codex32|seal|bip39|backup|engrave)\.[A-Z][A-Za-z0-9_]*`' "$PLAN" \
  | tr -d '`' | sort -u | while IFS=. read -r pkg sym; do
    if [ ! -d "$GOREPO/$pkg" ]; then
      printf '  skip  %-42s (package not in fork yet -- this plan CREATES it)\n' "$pkg.$sym"
      continue
    fi
    # Top-level declaration: func, type, const or var at column 0.
    top="^(func|type|const|var) (\([^)]*\) )?$sym\b"
    # Grouped declaration: an entry inside a `const (` / `var (` block, which is
    # indented and therefore invisible to the pattern above. This is how most of
    # seal's surface is declared -- MaxRecords, ErrTooManyRecords, ClassMDMK --
    # and until 2026-08-07 every one of them reported a FALSE FAIL. A gate that
    # cries wolf gets ignored, which is worse than one that stays silent.
    grouped="^[[:space:]]+$sym([[:space:]]|,|=)"
    if grep -rqE "$top" "$GOREPO/$pkg"/*.go 2>/dev/null; then
      loc=$(grep -rnE "$top" "$GOREPO/$pkg"/*.go | head -1)
      printf '  ok    %-42s | %s\n' "$pkg.$sym" "$(echo "$loc" | cut -c1-72)"
    elif grep -rqE "$grouped" "$GOREPO/$pkg"/*.go 2>/dev/null; then
      loc=$(grep -rnE "$grouped" "$GOREPO/$pkg"/*.go | head -1)
      # Labelled distinctly: the grouped pattern is looser than the top-level one
      # and CAN match an assignment rather than a declaration. The matched line is
      # printed so that is visible rather than assumed.
      printf '  ok*   %-42s | %s\n' "$pkg.$sym" "$(echo "$loc" | cut -c1-72)"
    else
      printf '  FAIL  %-42s no func/type/const/var %s in %s/\n' "$pkg.$sym" "$sym" "$pkg"
      echo FAILMARK >> /tmp/.citegate
    fi
  done

echo
echo "== RESULT =="
if [ -f /tmp/.citegate ]; then
  c=$(wc -l < /tmp/.citegate); rm -f /tmp/.citegate
  echo "   $c unresolvable citation(s) -- fix before review"
  fail=1
else
  echo "   every citation resolves"
fi
echo "   NOT covered: whether each line SAYS what the plan claims. The line's"
echo "   text is printed above so a reviewer can check the claim, not hunt for it."
exit $fail
