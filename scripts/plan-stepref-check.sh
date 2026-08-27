#!/usr/bin/env bash
# plan-stepref-check.sh -- prose in an implementation plan must not name a step
# NUMBER. Rationale refers to work by NAME so a renumbering cannot falsify it.
#
# WHY THIS EXISTS. The P0 plan failed three consecutive review rounds on one
# defect, alternating sides: round 2 found the table renumbered and the prose
# stale; round 3 found the prose rewritten and the table stale; round 4 found
# the rule itself violated in the document that states it -- NINE surviving
# references, six stale.
#
# It survived because the certifying grep was `grep 'step [0-9]'`, a FALSE
# NEGATIVE on four separate mechanisms, every one of which was present:
#     case          STEP 6's AUTHORITY
#     plurality     steps 1 and 7
#     hyphenation   step-3 gate
#     line-wrap     step\n1's
# A negative inherits the scope of the search that produced it.
#
# COVERED since round-5 I-5: prose INSIDE a table cell, spelled-out numerals
# (`step three`), bare row ids (`Row 9b`), and ORDINALS (`the 3rd step`, `the
# third step`) -- the last of which a commit message once claimed were covered
# when a planted probe showed they were not. The row's own number is exempt.
#
# NOT COVERED: a reference by NAME that has gone stale ("the move" pointing at a
# row that no longer does the moving). That class is real -- round 4's C-2 was
# exactly it -- and no lexical check can see it. It needs a reader.
set -u
[ "$#" -eq 0 ] && { echo "usage: plan-stepref-check.sh <plan.md>..." >&2; exit 2; }
bad=0
for f in "$@"; do
  echo "═══ $f"
  # Join wrapped lines so `step\n1` is caught, keeping original line numbers.
  hits=$(perl -0777 -ne '
    my @l = split /\n/, $_;
    for my $i (0..$#l) {
      my $probe = $l[$i] . " " . ($l[$i+1] // "");
      # The TABLE may number its ROWS -- strip the leading `| N |` cell and keep
      # checking the rest. Exempting the whole LINE left ~900 characters of
      # rationale per cell unchecked, which is exactly where cross-step ordering
      # claims get written (round-5 I-5: `done after step 3` planted in a cell
      # passed at exit 0).
      $probe =~ s/^\s*\|\s*\d+[a-z]?\s*\|// if $l[$i] =~ /^\s*\|/;
      if ($probe =~ /\bsteps?\s*-?\s*\d+[a-z]?/i
       || $probe =~ /\bsteps?\s+(?:zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve)\b/i
       || $probe =~ /\brows?\s+\d+[a-z]?\b/i
       || $probe =~ /\b\d+(?:st|nd|rd|th)\s+step\b/i
       || $probe =~ /\b(?:first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth|eleventh|twelfth)\s+step\b/i) { printf "%d:%s\n", $i+1, $l[$i]; }
    }' "$f")
  if [ -n "$hits" ]; then
    echo "$hits" | while IFS= read -r h; do
      echo "  STEP NUMBER IN PROSE  line ${h%%:*}"
      echo "     ${h#*:}" | cut -c1-96
    done
    # Count DISTINCT lines. The wrap-join probe reports a hit on both the line
    # carrying the reference and the one before it, so a raw count doubles.
    bad=$((bad + $(echo "$hits" | cut -d: -f1 | sort -u | grep -c .)))
  fi
done
echo
echo "─── step numbers in prose: $bad  (the TABLE is exempt; it IS the ordering)"
echo "─── NOT covered: a by-NAME reference that has gone stale. Needs a reader."
exit $([ "$bad" -gt 0 ] && echo 1 || echo 0)
