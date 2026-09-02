# Composer S1 whole-diff review — round 1, FOLD VERIFICATION lens

**Reviewer:** independent fold-verification reviewer (sonnet), did not author the fold.
**Date:** 2026-09-02.
**Under review:** mnemonic-engrave `wt-composer-s1`, branch `composer-s1`, fold commit
`5720e3c` (`git diff 90560cb..5720e3c`) responding to
`design/agent-reports/composer-S1-exec-review-r0.md` (0C/1I/3M/5N).

**Read-only discipline honoured.** Two mutations were applied (a `digits_in_range`
bound change for M-2, and a deletion of the derivation-suffix arm for M-3) and both
were reverted; `git status --porcelain` and `git diff --stat` are empty at the end
of this review, checked after each mutation and again at the close.

**mnemonic-secret worktree confirmed untouched by the fold:**
`git log --oneline 5f37b43..HEAD` in `wt-ms-bip48-p2tr` shows exactly one commit
(`7f979e5`, the original Task 5); `git status --porcelain` there is empty.

**Verdict: 0 findings against the fold. All five findings folded correctly; no new defect.**

---

## I-1 — spec :374 cell now quotes the current message

**Where:** `design/SPEC_sh2_sysw_consumption.md:374`.

**Command:**
```
$ ./target/debug/me sysw pack --no-passphrase "this is not a record of any class"
me: record 0 (records count from 0) is not a form this container can place: not a BIP-39
mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`/`key:`/`hash:`/`now:`
record. Addresses are not classifiable here, and neither is a wallet descriptor `me` refuses
— see sysw::classify
exit=4
```

The binary was rebuilt on the reverted, fold-commit tree (`target/debug/me` mtime
newer than both `main.rs` and `composer_records.rs`) before this run.

**Character-for-character check** (Python, prefix comparison, not eyeballing): the
spec's `:374` cell content, extracted verbatim from the markdown table cell, equals
the measured stderr **up to and including "record."**:
```
spec cell: 'me: record 0 (records count from 0) is not a form this container can place:
  not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a
  `text:`/`pass:`/`tx:`/`key:`/`hash:`/`now:` record.'
MATCH prefix: True
rest (not quoted by the spec, correctly so — it's a truncated cell): ' Addresses are not
  classifiable here, and neither is a wallet descriptor `me` refuses — see sysw::classify'
```

**Reproduction re-run:**
```
$ ./scripts/fold-propagation-check.sh design/SPEC_sh2_sysw_consumption.md '`text:`/`pass:`/`tx:` record'
  gone   `text:`/`pass:`/`tx:` record
   no superseded phrasing survives
```

**VERIFIED.** The row is the current six-prefix message, truncated exactly at
"record." as the fold commit claims — not a paraphrase, not a partial prefix list.

---

## M-2 — five fixture rows

**Rows and claimed content, decoded independently (Python, `bytes.fromhex` +
UTF-8 decode, not by reading the row names):**

| row | hex body | decoded | claim | match |
| --- | --- | --- | --- | --- |
| `now-unicode-digits` | `d9a1d9a7d9a5d9a6d9a6d9a8d9a4d9a8d9a0d9a0` | `١٧٥٦٦٨٤٨٠٠` | UTF-8 of Arabic-Indic 1756684800 | yes |
| `now-leading-zeros-valid` | `30303031373536383030` | `0001756800` | "0001756800" | yes |
| `key-body-odd-length` | (literal) `key:5b3` | `5b3` | `key:5b3` (odd-length hex, 3 chars) | yes, trivially |
| `now-seconds-eleven-digits` | `3031373536363834383030` | `01756684800` (11 chars) | "01756684800" | yes |
| `now-height-ten-digits` | `313735363638343830302c30343939393939393939` | `1756684800,0499999999` (10,10) | "1756684800,0499999999" | yes |

**Class-return check:** ran the fixture's own consumer test —
`cargo test -p mnemonic-engrave --test sysw_composer_records` → 18 passed, 1
ignored (`regenerate`), including `every_case_classifies_as_its_row_says_and_refuses_with_its_line`
(iterates all `CASES`, including the five new rows, asserting class and, where
present, the §8n `host_line`) and
`the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest`.

**Fixture row count and digest:**
```
$ python3 -c "import json; print(len(json.load(open('crates/me-cli/testdata/record_class_vectors.json'))))"
45
$ sha256sum crates/me-cli/testdata/record_class_vectors.json
eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e
$ grep FIXTURE_SHA256 crates/me-cli/tests/sysw_composer_records.rs
const FIXTURE_SHA256: &str = "eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e";
```
45 rows; file sha256 equals the pinned constant, byte for byte.

**Mutation test:** `digits_in_range`'s length check
(`crates/me-cli/src/sysw/composer_records.rs:204`) changed
`s.len() > max_digits` → `s.len() > max_digits + 1`.

- Full-suite run (`every_case_classifies_as_its_row_says_and_refuses_with_its_line`)
  FAILED at the first digit-count row it reaches in iteration order:
  `now-seconds-eleven-digits: now:3031373536363834383030 — left: "Now" right: "Unknown"`.
- Because `assert_eq!` inside the loop aborts at the first mismatch, the second row
  is masked by iteration order, not proven passing. To check it independently, the
  first offending case was skipped with one `if c.name == "..." { continue; }` line
  in the test body (reverted immediately after) and the suite re-run: it then
  FAILED on `now-height-ten-digits: now:313735363638343830302c30343939393939393939 —
  left: "Now" right: "Unknown"`.
- Both digit-count rows independently confirmed to red the mutation. Reverted both
  the `sed` on `composer_records.rs` and the test-file copy from backup;
  `git diff --stat` empty; re-ran the suite clean (18 passed, 1 ignored) to confirm
  the revert.

**VERIFIED.** All five rows decode to exactly the content their names and the r0
report claim; the consumer test proves their classes; the two digit-count rows
demonstrably can fail (the mutation reds each one, independently).

---

## M-3 — mis-paste detail arm

**Placement check** (`crates/me-cli/src/sysw/composer_records.rs:218-244`): the new
arm (`if xpub_text.contains('/') { return Err(K("the key carries a derivation
suffix…")); }`) sits after origin/fingerprint/path parsing and immediately
**before** `Xpub::from_str(xpub_text)` — confirmed by reading the function body in
order. It fires only on `xpub_text` (the substring after `]`), not on the whole
body, so it cannot trigger on an origin containing `/`.

**Behavioural check**, clean binary, two invocations:
```
--- bare xpub, no [origin] at all ---
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:` record whose body fails
    its rule (no [origin]: a bare xpub).
      record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a
        key record
--- descriptor-form: [origin]xpub/<0;1>/* ---
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:` record whose body fails
    its rule (the key carries a derivation suffix; give the account xpub alone, as
    `md decompose --emit keys` prints it).
      record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a
        key record
```
The bare xpub still gets the old detail (`"no [origin]: a bare xpub"`, from the
earlier `strip_prefix('[')` arm — untouched by this fold). The §8n line (second
indented line) is **byte-identical** across both cases — confirmed also by reading
`ComposerRecordError::line()`, which formats the same fixed string for every
`Key(_)` variant regardless of the wrapped detail string.

**Mutation test:** deleted the 7-line arm entirely
(`crates/me-cli/src/sysw/composer_records.rs:238-244`). Ran
`a_descriptor_form_key_names_the_suffix_not_the_xpub`:
```
FAILED — left: "not an extended public key"
         right: "the key carries a derivation suffix; give the account xpub alone, as
                 `md decompose --emit keys` prints it"
```
Reverted from backup; `git diff --stat` empty; re-ran full
`sysw_composer_records` suite clean (18 passed, 1 ignored).

**VERIFIED.** Arm fires before `Xpub::from_str`, gated correctly on `/` in
`xpub_text`; bare-xpub path and the §8n line are unchanged; the new test can fail.

---

## N-4 — test rename

**Old name search** (`grep -rn packs_byte_identically_to_before` across the whole
tree, not just source): the string appears in **two places**, both dated,
historical artifacts, not live source or documentation of current behavior:
- `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md:1049` — a `rust` code
  block reproducing the test **as it read when the plan was written**, inside a
  plan document whose own status line is dated `R0 GREEN 2026-09-02` (pre-dating
  this exec-review fold).
- `design/agent-reports/composer-S1-plan-R0-r1-fold-verification.md:75,123` — a
  **prior** fold-verification report (for the plan's own R0 round, not this exec
  review), quoting the old name verbatim as the finding it verified at that time.

Both are the same class as the r0 report's own N-2 (a pre-existing, dated
transcript, not a live claim, "do not charge it to this fold") — the finding N-4
was scoped to the test's self-description in `tests/sysw_composer_cli.rs`, and
that is the only place a rename was ever proposed. Neither historical document
claims to describe current test names.

**New name:**
```
$ grep -n a_payload_without_a_composer_record_gains_no_pack_time_record crates/me-cli/tests/sysw_composer_cli.rs
73:fn a_payload_without_a_composer_record_gains_no_pack_time_record() {
```
Passes as part of the 622/622 nextest run already recorded in the fold commit
message (not re-run here as a separate claim — the test file's only change is the
`fn` line, confirmed by `git diff 90560cb..5720e3c -- crates/me-cli/tests/sysw_composer_cli.rs`, a 1-line diff).

**VERIFIED**, with the note above recorded so a later round does not mistake the
two historical hits for a fold defect.

---

## N-5 — changelog clause

**Diff (`crates/me-cli/CHANGELOG.md`):** adds "A sealed payload that gets a `now:`
therefore carries its pack time in cleartext: the class is public by design,
because the device reads the bound before any passphrase," and bumps the fixture
row count in the same paragraph from 40 to 45 (correct — see M-2 above).

**Accuracy check:** `Class::Now` being public (not secret) is exercised by
`the_three_classes_classify_before_the_sniffers_and_are_not_secret`, which passed
in the suite run above. The clause states exactly the r0 report's own N-5 language
("Class::Now is public by design … the device door must read the bound without a
passphrase") — not a new, unverified claim.

**VERIFIED.**

---

## What else moved — full hunk list, `git diff 90560cb..5720e3c --stat`

```
crates/me-cli/CHANGELOG.md                       |  7 ++++--
crates/me-cli/src/sysw/composer_records.rs       | 19 +++++++++++++++
crates/me-cli/testdata/record_class_vectors.json | 30 ++++++++++++++++++++++++
crates/me-cli/tests/sysw_composer_cli.rs         |  2 +-
crates/me-cli/tests/sysw_composer_records.rs     | 13 +++++++++-
design/SPEC_sh2_sysw_consumption.md              |  2 +-
6 files changed, 68 insertions(+), 5 deletions(-)
```

Every hunk, read individually:

1. `CHANGELOG.md` — one hunk: N-5's clause + M-2's row-count bump (40→45). Both accounted for.
2. `src/sysw/composer_records.rs` — two hunks: the M-3 derivation-suffix arm in
   `parse_key`; the five M-2 `CASES` rows (with an explanatory comment). Both accounted for.
3. `testdata/record_class_vectors.json` — one hunk, the five M-2 JSON rows, matching `CASES` 1:1.
4. `tests/sysw_composer_cli.rs` — one hunk, the N-4 `fn` rename. Nothing else in the file changed.
5. `tests/sysw_composer_records.rs` — two hunks: the new M-3 test
   (`a_descriptor_form_key_names_the_suffix_not_the_xpub`); the `FIXTURE_SHA256`
   re-pin required by M-2's fixture regeneration.
6. `design/SPEC_sh2_sysw_consumption.md` — one hunk, the I-1 cell.

No hunk falls outside I-1/M-2/M-3/N-4/N-5. Nothing extraneous crept in.

**`sysw_vectors.json` byte-identical to master**, confirmed by sha256 rather than
by trusting the unchanged file list:
```
wt-composer-s1 (5720e3c): 7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac
master (3b5ca834):        7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac
```

**M-1 and N-3 filed as follow-ups, as the fold commit message claims** — confirmed
present on master's `design/FOLLOWUPS.md`:
- `F-451` (`:15352`) — M-1 (append overflow blames the operator's records),
  reproduction and fix-hypothesis carried over accurately, owning phase
  "composer S4 journey polish."
- `F-452` (`:15364`) — N-3 (`--now` silent no-op), plus N-1 and N-2 folded into its
  body as notes rather than separate entries, both attributed correctly.

**mnemonic-secret**: `wt-ms-bip48-p2tr` untouched — `git log --oneline
5f37b43..HEAD` shows only `7f979e5`; the fold commit's diff (`git diff
90560cb..5720e3c --stat`, above) touches only mnemonic-engrave files.

---

## Closing counts

**0 Critical / 0 Important / 0 Minor / 0 Nit against the fold.** All five findings
(I-1, M-2, M-3, N-4, N-5) folded correctly and verified by independent
reproduction, character-exact comparison, or mutation (not by re-reading the fold
commit's own claims); M-1 and N-3 correctly deferred to `F-451`/`F-452` rather than
fixed in place; no hunk falls outside the five findings' scope; no new defect
found; `sysw_vectors.json` and the mnemonic-secret worktree are unchanged.
