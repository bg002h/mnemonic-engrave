#!/usr/bin/env bash
# plan-glyph-check.sh — scan a design doc's OPERATOR-FACING strings for glyphs the
# SeedHammer II display font does not carry.
#
# WHY THIS EXISTS. The device's body face lacks the em dash, en dash, middle dot,
# curly quotes and the ellipsis character. A string containing one of them does
# NOT DRAW on the machine -- the operator gets a blank or broken line on a screen
# whose whole job is to tell them what their backup does and does not contain.
# The codebase already guards this at the unit level (the ContainsAny check in
# gui/multisig_build_prose_test.go), but a PLAN is where the strings are authored,
# and a plan reviewed with an em dash in it becomes code with an em dash in it.
#
# It earned its place immediately: on the first S6a fold it caught a string that
# had been copied verbatim out of a review report, where the reviewer had joined
# showNotice's two ASCII arguments with an em dash. The plan would have carried a
# misquote of shipped code into implementation. That is the standing rule -- never
# describe code from an earlier report -- caught by a command rather than by luck.
#
# USAGE
#   ./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_foo.md [more.md ...]
#
# EXIT
#   0  every operator-facing string is drawable
#   1  at least one string would not draw
#
# ─── WHAT THIS DOES NOT COVER ────────────────────────────────────────────────
#
#   * It only inspects what it can RECOGNISE as an operator-facing string:
#     markdown blockquotes, and backticked spans of 40+ characters. A string
#     authored in ordinary prose, or split across lines, is invisible to it.
#   * It does not check LENGTH or line-fit. A drawable string can still overflow
#     a panel; that is assertChoiceLabelFits' job, in the test suite.
#   * It does not check the strings actually IN the Go source -- only the ones a
#     plan proposes. The suite guards the source.
#   * The banned set is the one the shipped test uses. If the font changes, these
#     two lists must change together; nothing enforces that coupling.

set -uo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <doc.md> [doc.md ...]" >&2
  exit 2
fi

python3 - "$@" <<'PY'
import re, sys

# The glyph set gui/multisig_build_prose_test.go refuses, verbatim.
BANNED = "—–·‘’“”…"

bad = 0
scanned = 0
for path in sys.argv[1:]:
    print(f"═══ {path}")
    try:
        lines = open(path, encoding="utf-8").read().split("\n")
    except OSError as e:
        print(f"MISSING DOC: {e}", file=sys.stderr)
        bad += 1
        continue

    for n, s in enumerate(lines, 1):
        cands = []
        # A blockquote is how this repo's plans present operator-facing prose.
        # "> **" is a bolded editorial aside, not a device string.
        if s.startswith("> ") and not s.startswith("> **"):
            cands.append(s[2:])
        # Backticked spans long enough to be a sentence rather than an identifier.
        cands += re.findall(r"`([^`]{40,})`", s)
        for c in cands:
            scanned += 1
            hits = sorted({ch for ch in c if ch in BANNED})
            if hits:
                bad += 1
                print(f"WOULD NOT DRAW  line {n}: {''.join(hits)!r} in")
                print(f"                {c[:100]!r}")

print()
print(f"─── operator strings scanned: {scanned} ; undrawable: {bad}")
print("─── NOT covered: prose-embedded strings, line-fit, the Go source itself.")
sys.exit(1 if bad else 0)
PY
