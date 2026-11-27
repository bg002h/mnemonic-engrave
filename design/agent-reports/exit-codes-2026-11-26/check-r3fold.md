# Check of the R3 fold, SPEC_constellation_exit_codes.md (revision 4)

Line numbers are the spec's own. Tables checked by script (column counts, blank line before each table): no breakage. Nested lists and the R11 blockquote render correctly.

## (1) R3-I1 and R3-I2 against the disposition rows

R3-I1 as claimed: decision 2 (S:107-113) does say a malformed input value is 2 even when clap rejects it, and names `--hot-parent-fp zz`, `--template bogus` and `--expect-fingerprint zz`. §6 (S:505-506) moves E45's `--hot-parent-fp` entry to 2. §7g (S:698-699) sends a malformed hot-family value to 2. No leftover sends a malformed `--hot-parent-fp` to 64 except the wording below.

R3-I2 as claimed: test 4 (S:262-265) lists what counts as a match and excludes limits, caps, deadlines and deltas. §7g (S:701-705) splits `--accept-*` into mismatches (1) and guardrails (8). §7h (S:718-720) sends a foreign hand-back to 1. §3's worked table has no row sending a foreign hand-back to 2 or an `--accept-*` mismatch to 8. 130 is placed in test 0, the §3 table row (S:237), R10, R11, R12 and §7h as both a stop and a signal-like code. No leftover describes it as only a signal.

Problems found:

1. **Important, S:650-652 (§7f).** "Usage moves to 64: ... parser-shaped `BadInput` checks (1)". It is the same R3-I1 conflict left in the toolkit section. "Parser-shaped" is undefined, and a toolkit scheme name rejected by a value parser (`restore --template bogus`) fits it literally. Decision 2 (S:107-110) says that case is 2. An implementer following §7f sends it to 64. The same sentence sends the argv guard firing on a malformed `--from` or `--slot` to 64, though S:652 labels it "(2)" as today's value. Say which `BadInput` checks are settings (64) and which are input values (2), and drop "parser-shaped".

2. **Minor, S:93-95 (decision 2).** "64 covers ... the parser's errors" has no carve-out for the input values that S:107-113 then send to 2. The text is correct if read to the end. Add "except invalid values of input arguments" to the first statement.

3. **Minor, S:694-696 (§7g).** "Usage moves from 2 to 64 for the parser's errors" has the same missing carve-out. §6 (S:505-506) and S:698-699 cover the hot family, but a reader of this bullet alone would send `--hot-parent-fp zz` to 64.

4. **Minor, S:255-258 (test 1).** It asks only whether a *setting* is wrong. An unknown flag, a missing required argument and "no subcommand" are command-line errors that are not settings (decision 3, S:124, and §7a, S:533, say they are 64). Read literally, they fall through the tests to test 7 (101). Reword test 1 as "is the command line itself wrong, or a setting on it".

5. **Minor, §5 (S:446-460).** There is no row for an operator stop (130), though decision 9 and §7h move it. §5 is the "today to new, by class" summary, so the omission is visible but harmless.

## (2) Test-number references

Every reference matches the renumbered 0-7 list:
- S:265 "test 5" (limits)
- S:701 and S:720 "test 4"
- S:706 "test 4 or 5"
- S:858 and S:867 (the Round 2 disposition order: network 3, comparison 4, safety 5, input 6, bug 7)
- S:870 "test 7" (the rest goes to 101)
- S:877-878 (tests 4, 5, 0)

S:840 "§3 test 1 (the parser or a documented closed list)" is a stale paraphrase of test 1. The Round 0 note at S:828 says later rows win, so I count it as acceptable and raise no problem. No reference uses the old numbering.

## (3) Markdown

No problems. All 8 tables have consistent column counts and a blank line before them. The §3 row `| 128+N | | ... | |` has an intentionally empty cell. The R11 blockquote sits correctly inside its list item.

## Verdict

5 problems (1 Important, 4 Minor).
