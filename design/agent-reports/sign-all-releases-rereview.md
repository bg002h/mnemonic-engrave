# Re-review of fold 1 (sign-all-releases)

Brief: `design/briefs/sign-all-releases-rereview.md`. Reviewer: Sonnet 5, independent
context. Date: 2026-09-25/26.

One question: did fold 1 fix I1 and I2 (and the Minors) of
`sign-all-releases-review.md`, and did it introduce a defect? Not a fresh audit.

Branches re-checked, all `sign-releases`, unchanged from the fold report's tips
(confirmed via `git status --short --branch` in each worktree, all clean, all
`...origin/sign-releases`): toolkit e14b2302, ms d59d6a4, md b15330de, mk a49db39,
gui 2e2fae3, engrave d6955054. No release named `sign-releases` exists in any of
the six repos (`gh release view` → "release not found", all six). Nothing was
committed, pushed, tagged or released by this re-review. All scratch work lived
under `/scratch/code/shibboleth/review-sign2-scratch/`, generated-into copies only
(regeneration was redirected away from the live `/scratch/code/shibboleth/<repo>`
checkouts so it could never dirty them); the directory was deleted with
`find … -delete` when done. No secret key material was read or printed at any
point — only public keys (`RWRUl0D...`, `RWQPmg...`) and downloaded artifacts.

**Counts: 0 Critical, 0 Important, 1 new Minor, 0 new Nit.**

---

## 1. I1 — the stripped-signature downgrade — FIXED

- `install-assets.test.sh` (live, real releases): **32/32 mapped assets
  verified, 32/32 passed the signed-pin gate in both directions** — matches the
  fold report exactly.
- `install-assets-gate.test.sh` (offline, exercises `signed_pin_gate` extracted
  from the real test file): **8/8 ok**, unmutated.
- **Broke the gate 2 ways of my own** (distinct from the review's 7 mutations and
  the fold's own M2 mutation list), against a scratch copy of
  `install-assets.test.sh`, run through the unmodified
  `install-assets-gate.test.sh` harness:
  1. Flipped the `yes`/`no` branch selector (`if [ "$3" = yes ]` →
     `if [ "$3" = no ]`) — **RED, 8/8 FAIL**. The very first failure it reports
     is the original I1 hole reappearing: *"signed release, first_signed empty
     (the stripped-signature hole): expected bad, got: ok"*.
  2. Swapped the `version_ge` argument order in the signed branch
     (`version_ge "$2" "$_g_first"` → `version_ge "$_g_first" "$2"`) — **RED,
     2/8 FAIL** (the two cases that depend on comparison direction, exactly as
     expected of a targeted argument-order bug).
- **Runtime fail-safe ("only a 404 counts as missing")** — tested against a real
  HTTP stack, not just by reading the code. Built a local mock: a plain-HTTP
  Python backend (403 / 500 / 404 / truncated-body / 302-redirect-to-a-200-HTML-page
  routes) behind a `socat openssl-listen` TLS terminator (self-signed cert,
  trusted via `CURL_CA_BUNDLE`, no `-k`), so the real `curl` invocation inside
  `fetch_sig` (`--proto '=https' --tlsv1.2`) ran unmodified against real TLS.
  - `fetch_sig` classification: `200→ok`, `404→missing`, `403→error`,
    `500→error`, truncated body (declared Content-Length not delivered, curl
    exit 18) `→error`.
  - **`check_signature` end-to-end** (extracted from `install.sh`, called
    directly, `mk 0.14.0`, the real trusted key): 403 → **REFUSED** ("could not
    download... refusing rather than treating the release as unsigned"); 500 →
    **REFUSED** (same message); truncated → **REFUSED** (same message); 404 →
    installed with the "predates signed releases" note (correct and expected:
    `first_signed mk` is empty today, so a real 404 is the one thing this
    design currently accepts — this is what "only a 404 counts as missing"
    means, not that 404 itself refuses).
  - The redirect-to-HTML-200 construct: `fetch_sig` itself classifies it as
    **`ok`** (curl reports the *final* code after following the redirect, and
    that final response is a genuine, complete 200), not `error`. **The install
    still refuses** — but via the downstream `minisign -V` failing to parse/
    verify the HTML body as a signature, not via `fetch_sig` recognizing a
    failure. See the one new Minor, below.
- `install-signature.test.sh`: **18/18 ok** (17 numbered cases + the shipped-key-
  table check), matching the fold's claimed count exactly, including the four
  cases the fold added for this review (14–17: component-scope, lower-bound,
  network failure, HTTP 500).

**I1: fixed.**

## 2. I2 — over-broad tag filter — FIXED

- `emit.py`'s `tag_prefix` per repo, read directly from the source: `md`→
  `descriptor-mnemonic-md-cli-v`, `ms`→`ms-cli-v`, `mk`→`mk-cli-v`, `mnemonic`→
  `mnemonic-toolkit-v` (plus `mt`→`mt-cli-v`, unregenerated this round as the
  fold discloses).
- **Byte-identity**: copied `gen.py`/`emit.py` into scratch, redirected their
  hardcoded output path away from the live checkouts, regenerated all five, and
  ran `diff -q` against each branch's actual `.github/workflows/release.yml`:
  **toolkit, ms, md and mk are IDENTICAL**. `mnemonic-transaction` correctly
  *differs* (not part of this fold, as the fold report states).
  Trigger lines read directly from each branch: toolkit `mnemonic-toolkit-v*`,
  ms `ms-cli-v*`, md `descriptor-mnemonic-md-cli-v*`, mk `mk-cli-v*`.
- **Pattern matching**, tested with POSIX `sh -c` glob-case matching (the same
  simple-glob semantics GitHub Actions' tag filter uses for these
  prefix-plus-`*` patterns; note this only works correctly under a true `sh`,
  not the box's default `zsh`, whose `case` needs different options for the
  same result — verified the discrepancy directly):
  - `manual-gui-v9.9.9` → **no match** against all four CLI patterns.
  - `md-codec-v9.9.9`, `descriptor-mnemonic-md-codec-v9.9.9` → **no match**
    against md's pattern.
  - `ms-codec-v9.9.9`, `mnemonic-secret-ms-codec-v9.9.9` → **no match** against
    ms's pattern.
  - `mk-codec-v9.9.9`, `mnemonic-key-mk-codec-v9.9.9` → **no match** against
    mk's pattern.
  - `mnemonic-toolkit-v9.9.9`, `ms-cli-v9.9.9`, `descriptor-mnemonic-md-cli-v9.9.9`,
    `mk-cli-v9.9.9` → **MATCH** their own repo's pattern.

**I2: fixed.**

## 3. New `check-minisign-pin.sh` — sound, cannot pass vacuously

- Wired into engrave's `test` job (`.github/workflows/release.yml:90`,
  `sh scripts/check-minisign-pin.sh`), which the file's own header comment
  states fires on every push/PR and is the required context — confirmed by
  reading the job, not just the fold's claim.
- **Live CI**: fetched run 36220168609 directly (`gh run view`) — all 8 jobs
  succeeded, including `test (rust + go)`, which contains this step.
- Passes cleanly on the unmutated tree (3/3: release.yml 2 copies, README.md 2
  copies, gen.py 1 copy, all matching `minisign.pub`).
- **Mutation-tested all 4 pin locations**, one character each, in a disposable
  scratch copy of the engrave worktree (never the branch): `minisign.pub`
  itself, one of `release.yml`'s 2 copies, one of `README.md`'s 2 copies, and
  `gen.py`'s 1 copy. **All 4 mutations independently produced FAILED**, each
  with a message naming the exact file/line that no longer matches the pin.
- **Vacuous-pass resistance**: deleting `README.md` → FAILED ("is missing");
  redacting every key-shaped string out of `release.yml` (an empty grep) →
  FAILED ("carries the pin 0 time(s); expected at least 2"); deleting
  `minisign.pub` itself → the script aborts non-zero under `set -eu` rather
  than silently reporting OK. No path to a silent pass was found.
- Scratch copy diffed clean against the original worktree afterward (fully
  restored, no residue).

**M4: fixed, and independently confirmed non-vacuous.**

## 4. Concurrency and upload — FIXED

- All four generated `release.yml` (toolkit, ms, md, mk) now carry
  `concurrency: { group: ..., cancel-in-progress: false }`. `cancel-in-progress`
  is a bare `false`, unconditional — not gated on tag-vs-non-tag — so a tag run
  is never cancelled by a later one. Grouping keys off the tag
  (`release-<tag>`) for tag-push/backfill-dispatch and `github.run_id`
  otherwise, confirmed in toolkit's expression and spot-checked (`grep
  cancel-in-progress`) present in ms, md and mk.
- **No `2>/dev/null || true` remains on any `gh release upload ... tar.gz|zip`
  line** in any of the four repos (grepped all four `release.yml`s directly).
  The only surviving `|| true` in each is on `gh release create` — deliberately
  kept, per its own comment, because another tag workflow may have already
  created the release. Toolkit additionally keeps two unrelated,
  pre-existing `|| true` guards (an optional `LICENSE` copy, a keyfile
  cleanup) that have nothing to do with the archive-upload fix.
- Confirmed the two upload globs (`*.tar.gz`, `*.zip`) can never both come up
  empty: toolkit's matrix (`macos-x86_64`, `macos-aarch64`, `windows-x86_64`)
  produces 2 `.tar.gz` + 1 `.zip` every run (the packaging step's
  `[ windows ] → zip / else → tar.gz` branch, read directly), so removing
  `|| true` cannot turn a legitimately-absent archive kind into a hard failure
  for these three-target CLI repos.
- GUI and engrave correctly have **no** concurrency block added — this matches
  the fold's own disclosed "Open items" note (neither has a backfill dispatch,
  so a same-tag race there can only come from a manual re-run) rather than
  being a silently missed spot.

**M1: fixed.**

## 5. Artifact signature — verified myself, against the new key

- Downloaded `signed-sums-dry-run` from mk's run 36220164206
  (`bg002h/mnemonic-key`, `gh run download`).
- `minisign -V -P RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5 -m
  SHA256SUMS.portable -x SHA256SUMS.portable.minisig` → **"Signature and
  comment signature verified" / "Trusted comment: mk sign-releases
  SHA256SUMS.portable"** — byte-identical to the fold report's own quoted
  output for this exact run.
- Negative controls, same files: the **retired** key
  (`RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW`) refuses it (key
  id mismatch reported: signature is `EF2B8D34D8409754`, key given is
  `CA39ECB257009A0F`); a single-byte-tampered copy of `SHA256SUMS.portable`
  fails verification outright.

**M3: verified.**

---

## New findings

### Minor (new): `fetch_sig`'s own classification does not flag a redirect-to-200 as a failure; the refusal comes from the layer below

A `.minisig` URL that 302-redirects to a *different*, complete, HTTP-200
response (e.g. a CDN/hosting "soft error" page) is classified by `fetch_sig` as
**`ok`**, not `error` — `curl -w '%{http_code}'` with `-L` reports the *final*
response's code, and a genuinely complete 200 response is not a transfer
failure by any signal `fetch_sig` inspects. The end-to-end install still
**refuses** (confirmed above), but only because the downloaded bytes then fail
`minisign -V` as a malformed/wrong signature — a different, later gate than the
one the fold report's own wording implies ("`fetch_sig` classifies the
`.minisig` download, and only an HTTP 404 counts as missing; any other failure
is refused, never read as unsigned"). That sentence is true of the *outcome*
but not of the *mechanism* for this one input shape: `fetch_sig` does not
recognize it as a failure at all.

No exploit follows from this: an attacker who can only produce an HTTP-200
redirect target cannot forge bytes that verify against the pinned Ed25519 key,
so there is no path from this gap to an accepted forged signature. It does not
change the "ready to merge" answer. Recorded because the brief's own item 1
specifically asked for this construct, and the classification behavior is
worth knowing precisely (e.g. if `fetch_sig`'s classification is ever reused
elsewhere for a decision that isn't backstopped by a second cryptographic
check). No test currently exercises this input shape; a case could be added to
`install-signature.test.sh` cheaply via the existing `CURL_STUB_SIGERR`-style
stub, but this is optional polish, not a defect to fix before merge.

Nothing else new was found. M5 (documentation-only, header comments) and N1–N3
(wording, `pipefail`, macOS `shasum` spelling) were spot-checked and found
present as the fold describes (GUI's `sha256sums` step now has `shell: bash` +
`set -euo pipefail`; toolkit's README carries the `shasum -a 256 -c ...
--ignore-missing` line; both toolkit's and engrave's `release.yml` carry the
"A tag runs the workflow files AT THE TAGGED COMMIT" header). N4 needed no
change and none was made.

---

**ready to merge: yes**
