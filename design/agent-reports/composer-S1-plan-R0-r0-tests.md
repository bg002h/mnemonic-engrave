# R0 review — composer S1 host inputs implementation plan — mutation & claims lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md`, mnemonic-engrave `master` at `46fc91b8` (plan itself last changed `108fd4c9`).
**Lens:** Can every test in the plan fail, and are the plan's claims true? Independent adversarial mutation-testing pass, scoped to Tasks 1-4 (Task 5 was already hand-checked by the controller per the settled facts; Stage 2/3, Task 6 out of scope).
**What I ran:** Built a private scratch copy via `scripts/plan-build-gate-me.sh`, hand-wired Task 2's and Task 4's fragments exactly as the plan's ```text blocks specify (`Class::{Key,Hash,Now}` in `record.rs`; `classify_with`/`UnknownReason::Composer`/`SyswError::SecondNow`/the `split` check in `sysw/mod.rs`; `sysw_error`/`class_name` arms, `Pack{no_now}`, the auto-append block, `print_composer_confirmation` in `main.rs`; the `Class::Key/Hash/Now` arms in `tests/record_corpus.rs`), then `git init`'d the scratch copy to make mutate/run/revert mechanical. Toolchain `RUSTUP_TOOLCHAIN=1.85.0`, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.plan-gate-target-s1b`. Baseline: 23/23 composer tests pass (22 + the sha256-matching fixture test after running `regenerate`), `cargo clippy --all-targets -- -D warnings` clean, whole-suite 609 tests / 602 pass / 7 fail (see I-1).

## VERDICT: 0C/3I/2M/0N

## Mutation table

| mutation | file:line (scratch) | tests that failed | verdict |
| --- | --- | --- | --- |
| (a) `unhex_lower` accepts uppercase hex | `composer_records.rs:113` | `hash_must_be_exactly_64_lowercase_hex`, `key_origin_rules_are_each_enforced`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line` | CAUGHT |
| (b) drop `origin.len() != depth` check | `composer_records.rs:229` (`if false {`) | *(none)* | **NOT CAUGHT** — see I-2 |
| (c) drop last-component check | `composer_records.rs:232` (`if false {`) | `every_case_classifies_as_its_row_says_and_refuses_with_its_line`, `key_origin_rules_are_each_enforced` | CAUGHT |
| (d) allow xpub depth 5 | `composer_records.rs:223` (`3 \| 4 \| 5`) | *(none)* | **NOT CAUGHT** — see I-3 |
| (e) hash length lower bound dropped (`> 64` not `!= 64`) | `composer_records.rs:173` | `malformed_records_are_refused_with_the_8n_lines`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line`, `hash_must_be_exactly_64_lowercase_hex` | CAUGHT |
| (f) `MAX_SECONDS` off by one (2147483648 admitted) | `composer_records.rs:37` | `now_must_be_seconds_and_optional_height_in_range`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line` | CAUGHT |
| (g) `MAX_HEIGHT` off by one (500000000 admitted) | `composer_records.rs:35` | `now_must_be_seconds_and_optional_height_in_range`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line` | CAUGHT |
| (h) `digits_in_range` trims whitespace | `composer_records.rs:199` | `now_must_be_seconds_and_optional_height_in_range` | CAUGHT ONLY BY that one test |
| (i) `now_indices` counts malformed `now:` (prefix-only test) | `composer_records.rs:151` | `now_indices_counts_only_valid_now_records` | CAUGHT ONLY BY that one test |
| (j) `split` names the FIRST `now:` not the second | `sysw/mod.rs:454` | `the_payload_holds_at_most_one_now_record`, `two_operator_supplied_now_records_are_refused_naming_the_second` | CAUGHT |
| (k) `classify_with` composer arm moved after the BIP-39 sniffer | `sysw/mod.rs:256` | *(none)* | **NOT CAUGHT** — see M-1 |
| (l) `Class::Key` made secret | `record.rs:85` | `the_three_classes_classify_before_the_sniffers_and_are_not_secret`, `the_classes_pack_as_public_records_and_read_back`, `show_prints_each_class_legibly` | CAUGHT |
| (m) CLI auto-appends even with `--no-now` | `main.rs:1705` | `no_now_suppresses_the_auto_append_so_a_fixture_is_a_pure_function_of_its_inputs` | CAUGHT ONLY BY that one test |
| (n) CLI appends a second `now:` when one is already present | `main.rs:1705` | `an_operator_supplied_now_wins_silently_and_nothing_is_appended`, `show_prints_each_class_legibly` (via the library-level `SecondNow` refusal, not a wrong-but-silent success) | CAUGHT |
| (o) `line()` text changed by one word (Hash) | `composer_records.rs:87` | `malformed_records_are_refused_with_the_8n_lines`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line`, `hash_must_be_exactly_64_lowercase_hex` | CAUGHT |
| (o) `line()` text changed by one word (Now) | `composer_records.rs:88` | `now_must_be_seconds_and_optional_height_in_range`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line`, `malformed_records_are_refused_with_the_8n_lines` | CAUGHT |
| (p) a `CASES` row's `class` string flipped (hash-valid → "Key") | `composer_records.rs:273` | `the_fixture_covers_every_class_and_every_8n_line_at_least_twice`, `the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line` | CAUGHT — this is the Critical-shape check (a wrong fixture row a Go port would copy) and it holds |
| (q) `fixture_rows()` drops `host_line` | `composer_records.rs:311` | `the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest` | CAUGHT ONLY BY that one test |

17 of 19 mutations attempted (including both `line()` sub-cases) are caught; 2 are not (I-1, I-2), plus one architectural-ordering claim with no observable consequence (M-1).

## Findings

### I-1: Task 4 Step 4's "exactly six" pre-existing-failure claim is false — a seventh test fails from the identical mechanism and its file is not staged anywhere in the plan

*(Rated Important, not Critical, under this review's rubric: it is a false claim an implementer would act on, not a test that passes for a wrong classification or a wrong fixture row — mutation (p) checked that specific Critical-shape risk directly and it is CAUGHT. But it is the highest-impact finding here, so it is listed first.)*

**Plan location:** Task 4, Step 4 ("Run the CLI tests and the whole suite"), the `Expected:` paragraph naming six tests; Task 4 Step 5's `git add` list.

**Evidence.** With Tasks 1, 2 and 4 fully wired (no `--no-now` added anywhere), `cargo nextest run -p mnemonic-engrave --locked --no-fail-fast` (609 tests) reports **602 passed, 7 failed** — not six:
```
FAIL mnemonic-engrave::descriptor_as item_1_every_format_packs_one_descriptor_record
FAIL mnemonic-engrave::descriptor_as item_2_every_format_packs_reads_back_and_derives_the_device_address
FAIL mnemonic-engrave::sysw_cli a_payload_past_the_old_8191_cap_packs_and_reads_back
FAIL mnemonic-engrave::sysw_cli an_incomplete_set_still_packs_and_is_readable
FAIL mnemonic-engrave::sysw_cli a_secrets_only_payload_reports_no_digest
FAIL mnemonic-engrave::sysw_cli show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats   <- NOT in the plan's list
FAIL mnemonic-engrave::sysw_cli the_descriptor_show_block_leaves_every_other_container_byte_identical
```
The unlisted seventh, `sysw_cli::show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats`, fails for the **exact same mechanism** the plan names for the other six — the assertion output shows the auto-appended `now:` record as an extra `show` line:
```
left:  ["public record 0: descriptor — complete in one record", "public record 1: pack time (now:) — 1788337184 (…)"]
right: ["public record 0: descriptor — complete in one record"]
```
It is genuinely pre-existing: `git log -S` shows it added in `cde5c8b` and it is present at the plan's own cited baseline `b44fb61` (`git show b44fb61:crates/me-cli/tests/sysw_cli.rs | grep -c show_reports_exactly_one…` → 1).

Separately, Task 4 Step 5's `git add` line is:
```
git add crates/me-cli/src/main.rs crates/me-cli/tests/sysw_composer_cli.rs crates/me-cli/tests/sysw_cli.rs
```
`crates/me-cli/tests/descriptor_as.rs` is never staged, even though two of the plan's own six named fixes (`item_1_every_format_packs_one_descriptor_record`, `item_2_every_format_packs_reads_back_and_derives_the_device_address`) live in that file, not in `sysw_cli.rs`.

**Why it matters.** The plan's own instruction is "Add the flag; do NOT weaken an assertion; name each in the commit. Any OTHER pre-existing failure is a finding: stop and record it" — so this is not silently swallowed, but an implementer following the plan literally hits an unlisted failure the plan told them was impossible, and separately would leave `descriptor_as.rs`'s edits unstaged unless they notice the `git add` list is short (project convention is explicit staging, no `git add -A`, which is exactly the discipline this gap defeats). This also means the "already settled" measurement handed to me (also citing "six pre-existing tests") is itself wrong — worth flagging back, since it was presented as machine-checked.

**Remedy:** add `sysw_cli::show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats` to the named list (with `--no-now` on its `pack_then_show` call), and add `crates/me-cli/tests/descriptor_as.rs` to Task 4's `git add`.

### I-2: The origin-length-equals-xpub-depth rule has no test that isolates it

**Plan location:** Global Constraints ("origin component count == xpub depth"); Task 1 `parse_key`, the `origin.len() != usize::from(xpub.depth)` check; the `key-origin-shorter-than-depth` fixture row.

**Evidence.** Replacing the check's body with `if false { … }` (mutation b) leaves **all 23 composer tests passing**. The only fixture/unit case exercising a short origin (`KEY_SHORT_ORIGIN`, 2 components against a depth-4 xpub) also fails the very next check (the origin's last component, `0'`, differs from the xpub's own child number, `2'`), so check (c) alone fully explains every current failure and the depth-count check is exercised by nothing that isolates it — e.g. an origin with the RIGHT last component but the WRONG length (impossible to construct without picking a different xpub, since a wrong-length-but-right-last-component case needs an origin whose last element happens to equal the xpub's child number while the count still differs) is never tried.

**Why it matters:** this is a rule the spec's Global Constraints state explicitly, and the code's own comment (`"origin component count differs from the xpub's depth"`) documents it as load-bearing, but no test would catch a broken or removed implementation of it. **Remedy:** add a case whose origin length is wrong but whose last component matches the xpub's own child number (e.g. an origin with an extra bogus interior component ahead of the correct final one, or a length-3 origin ending in the same child number as a depth-4 xpub).

### I-3: The xpub-depth-3-or-4 rule has no test at the boundary

**Plan location:** Global Constraints ("xpub depth 3 or 4"); Task 1 `parse_key`, `if !matches!(xpub.depth, 3 | 4)`.

**Evidence.** Widening the match to `3 | 4 | 5` (mutation d) leaves **all 23 composer tests passing** — no fixture row or unit test constructs an xpub at any depth other than 3 or 4 (every `CASES`/unit-test xpub derives from `KEY0`'s own depth-4 chain). **Why it matters:** same class as I-1 — an explicit rule in both the spec's Global Constraints and the code, with zero coverage of the boundary it claims to enforce. **Remedy:** add a depth-5 (or depth-2) xpub case, refused for depth even though its origin length happens to match.

### M-1: The "matched before the BIP-39 sniffer" ordering claim is not exercised by anything that would notice a reorder

**Plan location:** Global Constraints ("matched BEFORE the sniffers"); the module's own doc comment; Task 2 Step 3.3 ("directly after the TEXT_PREFIX arm and before the BIP-39 sniffer").

**Evidence.** Moving the composer `classify_with` arm to *after* the BIP-39 sniffer (mutation k) leaves all 23 tests passing, because no composer prefix (`key:`, `hash:`, `now:`) is lexically a valid BIP-39 mnemonic phrase, so the two arms never actually compete for the same input in any existing test or fixture row. Rated Minor rather than Important because there is no plausible real input where the current ordering and the mutated ordering diverge — the prefixes are colon-delimited and no BIP-39 wordlist entry contains a colon — so this is an untested architectural invariant rather than a live risk. No action required beyond noting it if a future prefix is ever chosen without a colon.

### M-2: Two of the plan's `main.rs` line citations do not point at what they describe

**Plan location:** Task 4 Step 3: `` `print_mdmk_confirmation` (`crates/me-cli/src/main.rs:2060`) slices the public section into `records` and calls `print_mt_confirmation(&records); print_descriptor_confirmation(&records);` (`:2117-2118`) ``.

**Evidence**, against the real repo at `46fc91b8` (no plan edits applied): line 2060 is `/// the only one that matters, since \`Class::MdMk\` is not secret and so never` (a doc-comment sentence, not the function signature, which is actually at line 2063); lines 2117-2118 are `// \`Some\` here; the \`else\` is a total function rather than an \`unwrap\`` / `// on an operator path.` — comments inside `print_descriptor_confirmation`, not the two confirmation-call lines, which are actually at lines 2088-2089. By contrast, the plan's other two file:line citations checked are exact: `crates/me-cli/tests/record_corpus.rs:67-76` (Task 2 Step 8) is precisely the `class_name` match arms from `Class::Mnemonic` to `Class::Unknown`, and the unlined `descriptor-mnemonic parse/keys.rs, matches!(depth, 3 | 4)` (Task 5 Step 3.4) is verified present verbatim at `crates/md-cli/src/parse/keys.rs:130` in the descriptor-mnemonic repo (current HEAD `66bdf2f4`). **Why it matters:** low — both target functions/calls are named uniquely and are trivially found by grep, so an implementer is not actually misdirected, only briefly. **Note:** my dispatch brief also named `sysw/mod.rs:288`, `:416` and `template.rs:2618` (descriptor-mnemonic) as citations to check; none of these three appear anywhere in this plan's text (`grep -n '288\|416'` and `grep -n template.rs` both return nothing) — I could not verify them because they are not actually claims this plan makes; flagging in case that list was carried over from a different document.

## Claims checked

| claim | verdict | evidence |
| --- | --- | --- |
| Task 1 Step 4 "all PASS" | TRUE | 20/20 `sysw_composer_records` unit tests pass once the module is wired (measured as part of the 23-test composer run) |
| Task 2 Step 2 "FAIL to compile" | TRUE (trivial) | confirmed structurally — `Class::Key`/`UnknownReason::Composer`/`SyswError::SecondNow` do not exist before Task 2 |
| Task 2 Step 4 "all PASS…none of them starts with one of the new prefixes" | TRUE | `grep -n '"key:\|"hash:\|"now:'` over `record_corpus.rs` (excluding the composer test files) returns nothing; the corpus test is among the 602 passing in the full run |
| Task 3 Step 2 "FAIL to compile" | TRUE (trivial) | `CASES`/`fixture_rows`/`FixtureRow` do not exist before Task 3 |
| Task 3 Step 4 fixture behavior and pinned sha256 | TRUE | before regenerating: 22 pass, `the_committed_fixture_…` fails (file absent); `cargo test … regenerate -- --ignored --nocapture` prints `wrote 27 rows` and `sha256 2215285fad952316e8e190ca5563e55f06c0ae021328278accf341f841522eaf`, matching `FIXTURE_SHA256` exactly; all 23 pass after |
| The 27-row count | TRUE | `grep -c '^    Case { name:' composer_records.rs` → 27 |
| "every class and every §8n line at least twice" | TRUE | `the_fixture_covers_every_class_and_every_8n_line_at_least_twice` passes in the baseline, and mutation (p) shows it is a real check, not vacuous |
| Task 4 Step 2 "no_now FAILS/pack_appends FAILS/refusals PASS/show FAILS" | TRUE in substance | reproduced by reverting only Task 4's fragments over Task 1-2: 4 of 6 CLI tests fail (`no_now_suppresses…`, `pack_appends…`, `show_prints_each_class_legibly`, `an_operator_supplied_now_wins…` — the last not named but same "show lines don't exist yet" cause), `two_operator_supplied_now_records…` and `malformed_records_are_refused…` pass already, matching the plan's description |
| Task 4 Step 4 "exactly SIX pre-existing tests fail" | **FALSE** | see I-1: 7 fail, for the same stated reason |
| Hex bodies in `CASES` decode to their named intent | TRUE | every `key:`/`now:` body in `CASES` decoded with `bytes.fromhex(...).decode()` matches its row name's implied text (`[73c5da0a/48'/…]xpub…`, `1756684800`, `1756684800,910000`, `2147483648`, `1756684800,500000000`, etc.); `now-body-not-hex`/`now-body-not-utf8` bodies are deliberately non-hex/non-UTF-8 as their names say |
| Pinned sha256 `2215285f…22eaf` | TRUE | regenerate test reprints the identical digest |
| `main.rs:2060`, `:2117-2118` citations | **FALSE** (off by 3 and 29 lines respectively) | see M-2 |
| `record_corpus.rs:67-76` citation | TRUE | exact span of the `class_name` match arms |
| `descriptor-mnemonic parse/keys.rs`, `matches!(depth, 3 \| 4)` | TRUE | present verbatim at `crates/md-cli/src/parse/keys.rs:130`, descriptor-mnemonic HEAD `66bdf2f4` |
| `sysw/mod.rs:288`, `:416`, `template.rs:2618` (from my dispatch brief) | NOT A PLAN CLAIM | these strings do not occur anywhere in the plan text; nothing to verify |
| False-PASS shapes (untyped `is_err()`, vacuous lists, tautological equality, Debug-rename-passes-silently) | NONE FOUND | no bare `is_err()`/`is_ok()` used to check a refusal kind in the composer test files; the class/line-coverage test iterates a fixed required-class list so an emptied `CASES` fails it, not passes it; a Debug rename of a `Class` variant breaks the `format!("{got:?}") == c.class` comparison immediately (mutation p's mechanism generalizes) rather than passing silently |

## What I ran

- `TMPDIR=… CARGO_TARGET_DIR=/scratch/code/shibboleth/.plan-gate-target-s1b bash scripts/plan-build-gate-me.sh design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` — expected pre-fragment compile failure, confirmed.
- Hand-wired Task 2 (`record.rs` `Class::{Key,Hash,Now}`; `sysw/mod.rs` classifier arm, `UnknownReason::Composer`, `SyswError::SecondNow`, the `split` check; `main.rs` `sysw_error`/`class_name` arms; `tests/record_corpus.rs` name-table arms) and Task 4 (`Pack{no_now}`, the auto-append block, `print_composer_confirmation` + call site) exactly per the plan's ```text blocks.
- `cargo build -p mnemonic-engrave --all-targets --locked` (RUSTUP_TOOLCHAIN=1.85.0): clean.
- `cargo nextest run -p mnemonic-engrave --locked --no-fail-fast -E 'binary(/^sysw_composer/)'`: 22 pass / 1 fail (fixture, pre-regenerate) → `cargo test … regenerate -- --ignored --nocapture` → 23 pass / 1 skipped.
- `cargo nextest run -p mnemonic-engrave --locked --no-fail-fast` (whole suite, no `--no-now` anywhere): 609 tests, 602 passed, **7 failed**, 2 skipped (see I-1).
- `cargo clippy -p mnemonic-engrave --locked --all-targets -- -D warnings`: clean.
- `git init` in the scratch copy to make 19 mutate/build/test/revert cycles mechanical (see Mutation table); reverted to the wired baseline (`git checkout -- .`) after every mutation and re-confirmed 23/23 green at the end.
- One additional reconstruction: reverted only Task 4's fragments (keeping Task 1-2) to verify Task 4 Step 2's described RED state.
- `python3 -c "bytes.fromhex(...).decode()"` over every `key:`/`now:` hex body in `CASES`.
- `git log -S`/`git show <baseline>:<path>` in the real mnemonic-engrave repo to confirm the unlisted 7th failing test predates this plan.
- `grep -n` in the real mnemonic-engrave and descriptor-mnemonic repos to check the plan's `file:line` citations.
