# S0 D4(b) -- shot_server.py's three claimed security properties

Scope: verify, against the real code (not the docstring), that
`design/journeys/shot_server.py` actually enforces the three properties its
docstring (lines 1-20) claims. Full file read (110 lines, 4142 bytes).

## VERDICT

- **Pinned to one origin: HOLDS.** Enforced in `do_POST` at line 78
  (`if origin != ALLOWED_ORIGIN: return self._fail(403, ...)`) and in
  `do_OPTIONS` at line 68. `_cors()` (lines 54-59) only ever emits
  `Access-Control-Allow-Origin` when the request's `Origin` header equals
  `ALLOWED_ORIGIN`, so a rejected response never carries it either.
- **Flat filenames only: HOLDS.** Enforced at line 43 in `safe_path()`:
  `NAME_RE.match(name)` (regex defined line 37: `\A[A-Za-z0-9][A-Za-z0-9._-]
  {0,63}\.(png|svg)\Z`, no `/` or `\` in the character class, first char
  must be alnum so no dotfiles) plus an explicit `".." in name` check.
- **Resolved-path re-check: HOLDS.** Enforced at lines 45-49 in
  `safe_path()`: `path = os.path.realpath(os.path.join(OUT, name))` then
  `if os.path.dirname(path) != OUT: raise ValueError(...)`.

No code change was needed. `git diff -- design/journeys/shot_server.py` is
empty.

## TESTS

**File:** `scripts/test/shot-server-security-test.py` (new).

**Why `scripts/test/` and not `design/journeys/`:** the repo's one existing
standalone-Python-test precedent, `scripts/test/mutation-run-crashtest.py`,
tests a script that lives in `scripts/` (not `scripts/test/`) -- i.e.
`scripts/test/` is already the repo's general-purpose location for a
no-framework, assert-based test script, independent of where the file under
test lives. `design/journeys/` has no test-file precedent at all (its
`.py` files are all generators: `build_pdf*.py`). Followed the same
no-pytest, plain-`assert`, sequential-function, nonzero-exit-on-failure
style as `mutation-run-crashtest.py`.

**How it loads the module under test:** `shot_server.py` can't be
`import`ed directly -- its top level reads `sys.argv[1:2]` and then calls
`socketserver.TCPServer(...).serve_forever()` unconditionally (no
`if __name__ == "__main__":` guard). The test's `load_module()` reads the
source, `exec()`s everything up to (not including) the line
`socketserver.TCPServer.allow_reuse_address = True` (all needed names --
`NAME_RE`, `safe_path`, class `H`, `OUT`, `ALLOWED_ORIGIN` -- are defined
before that line), with `sys.argv` set to a scratch temp dir + a test
origin first. This uses the module's own `os.path.realpath(sys.argv[1])`
line to compute `OUT` rather than reimplementing that logic, and never
starts the real server or opens a socket.

**How it drives the HTTP handler:** `_invoke()` constructs a real `H`
instance via `H.__new__(H)` (bypassing `socketserver`'s `__init__`, which
wants a live connection) and calls `do_POST`/`do_OPTIONS` on it directly,
with a real `email.message.Message` for `.headers` (case-insensitive
`.get()`, matching what `http.client.HTTPMessage` actually is in
production) and real `io.BytesIO` for `.rfile`/`.wfile`. This is "running
the handler directly" per the brief -- no socket is bound anywhere in this
suite.

**Run:**
```
python3 scripts/test/shot-server-security-test.py
python3 scripts/test/shot-server-security-test.py --target /path/to/mutated/copy.py
```

**Output (against the real file, this run):**
```
== property 1: pinned to one origin ==
  [legit]           200, wrote /tmp/shot-server-out-0pqy8qbo/frame001.png
  [wrong origin]    403, no ACAO header leaked
  [missing origin]  403, no ACAO header leaked
  [OPTIONS]         204 for matching origin, 403 + no ACAO for mismatch
PASS: origin pinning

== property 2: flat filenames only ==
  [legit name]              accepted -> .../frame001.png
  [path traversal] rejected: '../../etc/passwd.png'
  [leading slash] rejected: '/etc/passwd.png'
  [backslash] rejected: '..\\..\\win.ini.png'
  [url-encoded traversal] rejected: '%2e%2e%2f.png'
  [nested path] rejected: 'a/b.png'
  [double-dot, no separator ...] rejected: 'x..svg'
  [dotfile] rejected: '.hidden.png'
PASS: flat filenames

== property 3: resolved-path re-check ==
  [symlink escape, safe_path()] rejected: escapes .../out: 'evil.png'
  [symlink escape, full POST]  400, .../outside/pwned.png not created
PASS: resolved-path re-check

ALL PASS
```
Exit code 0.

Adversarial cases covered per the brief: wrong-Origin request AND a
*missing* Origin header (both, separately, for `do_POST`; also for
`do_OPTIONS`); filenames with `../`, leading `/`, backslash, URL-encoded
`%2e%2e%2f`, and nested `a/b.png`; a real symlink planted in the target
directory pointing outside it, exercised both via `safe_path()` directly
and via a full `do_POST` (confirming no write occurs anywhere, not just
that the function raises); and a legitimate request that must succeed
(asserts 200, body `b"ok"`, and the written file's bytes match), so the
suite is not refusal-only. Also covered, beyond the minimum: preflight
(`do_OPTIONS`) in both directions, `Access-Control-Allow-Origin` absence on
every rejection (not just the status code), a dotfile name, and
`"x..svg"` -- a name that *matches* `NAME_RE` (verified: `NAME_RE.match("x..svg")`
is truthy) and is rejected only by the separate `".." in name` check,
confirming that check has real bite and isn't redundant with the regex (the
docstring's own comment at line 42, "Both checks are required").

## MUTATION PROOF

All three mutations were applied to throwaway copies in the scratchpad
directory (never to the tracked file), run against via `--target`, and
discarded. `git diff -- design/journeys/shot_server.py` was empty
throughout and remains empty now.

- **Property 1 (origin pinning).** Mutation: in `do_POST`, changed
  `if origin != ALLOWED_ORIGIN:` (line 78) to `if False:`. Result: RED --
  `test_origin_pinning` fails at the wrong-origin assertion:
  `AssertionError: expected 403 for wrong origin, got 200`. (The legit case
  still passed 200, as expected -- only the rejection path broke.)

- **Property 2 (flat filenames).** Mutation: widened `NAME_RE` (line 37)
  from `\A[A-Za-z0-9][A-Za-z0-9._-]{0,63}\.(png|svg)\Z` to
  `\A.{1,200}\.(png|svg)\Z`. Result: RED -- `test_flat_filenames` fails at
  the `url-encoded traversal` case: `AssertionError: url-encoded traversal
  should have been rejected, got '.../%2e%2e%2f.png'`. Notably, the
  `path traversal`, `leading slash`, and `backslash` cases *still* got
  rejected under this mutant -- not by the (now-broken) regex, but because
  those names still resolve outside `OUT` and property 3's independent
  dirname re-check caught them anyway. That is direct evidence the two
  checks are genuinely layered defense-in-depth, exactly as the docstring
  (lines 13-16) claims: the case that only the whitelist (not the
  resolved-path check) is positioned to catch -- a weird-but-non-escaping
  literal string -- is the one that went red.

- **Property 3 (resolved-path re-check).** Mutation: in `safe_path()`,
  changed `if os.path.dirname(path) != OUT:` (line 48) to `if False:`.
  Result: RED -- `test_resolved_path_recheck` fails exactly at the
  symlink-escape assertion: `AssertionError: symlink escape should have
  been rejected, got '/tmp/shot-server-outside-.../pwned.png'`. Property 2
  continued to pass cleanly under this mutant, confirming isolation: the
  symlink attack (`evil.png`, a whitelist-legal name) is invisible to the
  regex and caught only by the dirname re-check.

## FINDINGS

None. All three properties hold in the deployed code as of this read; no
attack in the adversarial set above succeeds against the unmodified file.

## FILES CHANGED

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave status --porcelain
?? scripts/test/shot-server-security-test.py
```
(`design/journeys/shot_server.py` is untouched -- 0 lines changed.)

## UNRESOLVED

- Not tested: `MAX_BODY` enforcement (line 84, `content-length` bounds) and
  malformed-JSON / non-dict-body handling (lines 86-92). These are real
  robustness checks in the file but are outside the three properties named
  in the brief (origin pinning, flat filenames, resolved-path re-check), so
  left untested here rather than scope-creeping the report.
- Not tested: concurrent-request / TOCTOU between the symlink-existence
  check inside `realpath()` and the later `open(path, "wb")` (lines 45 vs
  94) -- i.e. a symlink planted *after* `safe_path()` returns but *before*
  `open()` runs. This is a narrow race window in a single-threaded
  `socketserver.TCPServer` handling one request at a time from a
  known-fixed operator page on localhost during a manual test session; a
  reliable test would need real thread/process concurrency and precise
  timing, which is exactly the "excessive machinery for a script-under-test
  that doesn't warrant it" case the brief says to flag rather than fake.
