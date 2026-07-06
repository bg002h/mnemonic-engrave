# Funds-safety audit — Dimension D5: secret hygiene across the first-party tree

Auditor: D5 finder (multi-agent funds-safety audit)
Date: 2026-07-06
Scope: `crates/me-cli/src/**`, `preview/**`, `firmware/ndef-roundtrip/**`
Method: READ-ONLY on the repo; built `me` (release) + the `me-preview` Go sidecar in a
scratch dir OUTSIDE the repo; ran concrete probes; verified codec error text against the
pinned crate sources (`md-codec 0.36.0`, `mk-codec 0.4.0` in `~/.cargo/registry`).

## Sensitivity model (established first)

- `ms1` = **secret** seed entropy. Refused everywhere (converter: `lib.rs:59`; bundle
  pre-scan: `bundle.rs:188-192`). This is the tool's core safety contract.
- `md1` (descriptor/policy) and `mk1` (xpubs) are documented "public" (README:3,17). Per the
  audit brief I treat them as **privacy-critical**: an `mk1` carries xpubs, and an xpub +
  descriptor reveal a wallet's entire past and future address set (full balance/tx history and
  forward addresses). Not spendable-key material, so exposure is a **privacy** breach, not
  direct funds loss — hence the md1/mk1-only findings below are moderate/low, while the one
  path that can leak the actual `ms1` secret is rated important.

The tool's own hardening posture already reflects this model (Zeroizing input buffer, argv-free
input, `--echo` gated to success). The findings below are the places where that posture is
incomplete or applied asymmetrically.

---

## FINDINGS

### D5-1 (important) — `me bundle` echoes the full input line to stderr; leaks an `ms1` secret body on the mangled-HRP path and systematically echoes privacy-critical xpubs

**Files:** `crates/me-cli/src/bundle.rs:54` (`Classify`), `:55` (`Validate`), `:60`
(`Md1HeaderRead`) — the `Display` impl interpolates the full input string `s`.

The converter path was deliberately hardened to **never** echo the input: `ConvertError`
prints only bounded codec messages (`lib.rs:26-40`), and the codec `Error` `Display` strings
themselves carry only metadata (error position, symbol counts, HRP), never the payload
(verified in `mk-codec-0.4.0/src/string_layer/bch.rs:421,437` and
`md-codec-0.36.0/src/error.rs:246` — all bounded). The `bundle` path regressed this: its own
first-party error wrappers embed the whole line.

Two consequences, both demonstrated with the built `me`:

1. **Secret `ms1` leak (the funds/secret-exposure class (c) in the brief — "error-text
   leakage").** The bundle ms1 refusal is a *classify-only* pre-scan that only fires when the
   HRP is exactly `ms` after lowercasing (`bundle.rs:189`, `classify.rs:46-51`). An `ms1`
   whose HRP is mangled by a transcription typo (a stray char in the 2-char HRP, or a spurious
   early `1`) is **not** classified as `Ms`, dodges the refusal, and then fails `classify` in
   `parse_line` → `BundleError::Classify(s, _)` → the **entire secret body is written to
   stderr**:

   ```
   $ printf 'msx10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n' | me bundle
   me: cannot classify 'msx10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f': unrecognized HRP 'msx' ...
   ```

   The codex32 data body after the mangled HRP is the intact secret entropy; it now lands in
   shell scrollback, `2>logfile` redirects, and CI logs. The converter path does **not** do
   this (verified: `me --hex < bad_md1` prints only `codex32 decode error: BCH ...`, no
   string). This is exactly the "never let secret entropy reach a log/RF" invariant the ms1
   refusal exists to enforce, defeated on a plausible typo.

2. **Systematic xpub echo (privacy).** *Every* invalid `md1`/`mk1` line — the common case,
   not a corner — is echoed verbatim, e.g.:

   ```
   $ printf '<mk1 with 1 flipped symbol>\n' | me bundle
   me: invalid string 'mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6q': mk1 string is not pristine ...
   ```

   The full xpub-bearing string goes to stderr on any BCH slip.

**Fix direction:** in the `bundle.rs` `Display` impl, replace `'{s}'` with a redacted form
(HRP + length, or the first/last few symbols) exactly as the converter already does; and/or run
the ms1 refusal on the *data-part alphabet/shape*, not just an exact-HRP classify, so a
mangled-HRP secret is still refused (without echo) rather than printed.

**Automated test that would have caught it** (integration, `assert_cmd`):
```rust
// A mangled-HRP ms1 must NOT have its data body appear in stderr.
let secret_body = "0entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
let out = Command::cargo_bin("me").unwrap()
    .arg("bundle").write_stdin(format!("msx1{secret_body}\n")).assert().failure();
assert!(!String::from_utf8_lossy(&out.get_output().stderr).contains(secret_body),
        "bundle stderr must never echo an ms1 data body");
// And an invalid mk1 must not be echoed verbatim.
let mk1 = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6q";
let out2 = Command::cargo_bin("me").unwrap()
    .arg("bundle").write_stdin(format!("{mk1}\n")).assert().failure();
assert!(!String::from_utf8_lossy(&out2.get_output().stderr).contains(mk1));
```

---

### D5-2 (moderate) — output artifacts are written world-readable (0o644) and depict/contain privacy-critical md1/mk1 material; no restrictive perms, no runtime warning

**Files:** `crates/me-cli/src/main.rs:140` (`fs::write` NDEF `--out`), `:205` (`fs::write`
manifest), `preview/main.go:128` (`os.WriteFile(path, payload, 0o644)` for SVG/PNG).

`std::fs::write` and `os.WriteFile(...,0o644)` both create files with mode `0o644` (minus
umask). Under the default umask 022 all three artifacts are **world-readable**. Demonstrated:

```
-rw-r--r-- 644 wallet.ndef        # me --out : NDEF Text record = the md1/mk1 string
-rw-r--r-- 644 manifest.json      # me bundle --manifest : embeds every raw md1/mk1 (PlateEntry.string)
-rw-r--r-- 644 prev/plate-1.svg   # me bundle --preview : 150 KB SVG that RENDERS the payload as strokes + QR
```

The manifest literally contains the full strings (`bundle.rs:235,261,283` set
`string: Some(s.clone())`), and the SVG/PNG are a faithful visual of the engraved plate — a
scannable QR of the descriptor/xpub plus its text. On any shared/multi-user host, another local
user can read a user's complete watch-only wallet (all xpubs + policy) from these files. There
is no `0o600` tightening, no umask note, and — verified at runtime — no warning is printed when
previews are rendered (`me: rendered plate N → path` only). A user is not told these images
expose their wallet.

Severity moderate (privacy, not spend): the artifacts hold no `ms1` (ms1 is `string: None`
and never rendered — see negatives), so no seed leaks; but a full xpub/descriptor disclosure is
the exact thing a "durable private backup" workflow should not scatter world-readable.

**Fix direction:** create these files `0o600` (Rust: `OpenOptions::new().mode(0o600)`; Go: open
with `0o600`), and/or print a one-line warning that the manifest/preview depict the wallet's
public keys.

**Automated test:** after `me --out f`, `me bundle --manifest m`, and `--preview d`, assert
`metadata(f).permissions().mode() & 0o077 == 0` (no group/other bits). Currently 0o644 → fails.

---

### D5-3 (low) — stale preview images from a prior run are never cleaned; a second wallet's `--preview` into the same dir leaves the first wallet's payload images behind

**File:** `preview/main.go:123-129` writes `plate-<idx>.<ext>`; `main.rs:271-294` iterates only
the *current* run's plates and never clears the target dir.

Demonstrated: run 1 (md1 + 2× mk1) writes `plate-1..3.svg`; run 2 (one md1) into the same dir
writes only `plate-1.svg`, leaving `plate-2.svg` and `plate-3.svg` — the **previous wallet's
mk1 (xpub) plates** — sitting in the directory:

```
after run1: plate-1.svg plate-2.svg plate-3.svg
after run2: plate-1.svg plate-2.svg plate-3.svg   # plate-2/3 are stale, depict wallet #1
```

Combined with D5-2's world-readable mode, an unrelated later user of the tool cannot tell which
images belong to the current wallet, and old wallets' xpub images accumulate silently.

**Fix direction:** write previews into a fresh/empty subdir, or refuse a non-empty target, or
prune `plate-*.{svg,png}` for indices beyond the current run.

**Automated test:** render N=3 plates then N=1 into the same dir; assert the dir contains
exactly one `plate-*.svg` afterward. Currently 3 remain → fails.

---

### D5-4 (low) — PATH-based sidecar discovery plus a spoofable version gate lets a planted `me-preview` receive the (public) payload

**File:** `crates/me-cli/src/preview.rs:76-83` (`locate_sidecar` falls back to `$PATH`),
`:94-108` (`sidecar_version` just string-matches `me-preview <ver>` on stdout).

When `me` is installed alone (e.g. `cargo install`, no sidecar beside the exe), `--preview`
searches `$PATH` and pipes the md1/mk1 string to the first `me-preview` found. The version gate
is not an integrity barrier — a hostile stand-in need only print `me-preview 0.3.0` to pass
(`main.rs:246-259`). The payload is public (no `ms1` ever reaches the sidecar — see negatives),
so this is a privacy/xpub-exfil surface plus arbitrary writes into the preview dir, not a seed
leak; hence low.

**Fix direction:** document that `--preview` requires the trusted co-located sidecar, or prefer
exe-adjacent only (drop the `$PATH` fallback) for the default trust path.

**Automated test:** place a fake `me-preview` earlier on `$PATH` than a real one and assert
discovery still prefers the exe-adjacent binary (documents/locks the trust order).

---

## Checked and found SOUND (negative results — coverage record)

- **No payload via argv.** `Cli`/`Bundle` (`main.rs:12-57`) declare only flags; the payload is
  read from **stdin or `--in`** (`main.rs:84-96,167-179`). `grep` for `env::args`/positional:
  none. clap never sees the payload, so `ps`/shell-history exposure is genuinely avoided (the
  README's headline claim holds). The sidecar likewise reads the string from **stdin**
  (`preview/main.go:61`); only file **paths** are passed as args (`preview.rs:130-140`).
- **`ms1` never reaches the preview sidecar.** Double-guarded: the ms1 plate has
  `string: None` (`bundle.rs:297-306`) and `wire_previews` also `continue`s on
  `PlateKind::Ms1` (`main.rs:271-277`). So no secret can transit to Go (where zeroize cannot
  reach). Verified by code.
- **Converter never echoes the input.** `ConvertError` prints only bounded codec text
  (`lib.rs:26-40`); probe `me --hex < bad_md1` emitted no string. The ms1 refusal message
  (`lib.rs:30-35`) contains no secret. Verified by probe.
- **Codec `Error` `Display` is bounded.** `mk-codec` variants carry HRP / offending char+pos /
  counts (`error.rs:23-60`), and `BchUncorrectable` is a fixed string / position message
  (`bch.rs:421,437`); `md-codec` `Codex32DecodeError(String)` is a category message, not the
  input. So the converter's `{e}` printing cannot leak the payload; only the bundle wrappers
  (D5-1) do, and they do it in first-party code.
- **`--echo` is safe.** Built only when `cli.echo && result.is_ok()` (`main.rs:109-115`), i.e.
  only for a validated **public** md1/mk1, and wrapped in `Zeroizing`. ms1 (an `Err`) never
  reaches it. Matches the prior I-1 fix in FOLLOWUPS. No leak.
- **Input zeroization.** Input is read into `Zeroizing<String>` (`main.rs:84,167`); for `ms1`
  the only owned copy is that buffer (ms1 is refused *before* any codec call in the converter
  — `lib.rs:59` precedes `validate`), scrubbed on drop (`main.rs:117`). *Caveat, not a
  finding:* Rust's incremental `stdin().read_to_string` growth can leave un-zeroized
  reallocation fragments in the allocator; this is inherent to std and is pure defense-in-depth
  beyond the (still-intact) refusal. The `main.rs:81-83` comment scopes its claim to `--in`
  (`fs::read_to_string`, which pre-sizes to file length → typically one allocation), so it is
  **not** false-confidence — it does not claim stdin coverage.
- **Go side has no secret-hygiene footguns.** `grep` of `preview/` + `firmware/` for
  `TempFile`/`ioutil`/`MkdirTemp`/`pprof`/`Getenv`/`os.Args`(payload): only `os.Args[1:]`
  (flags, no payload) and the single `os.WriteFile(...,0o644)` (D5-2). No temp intermediates
  (PNG is encoded to an in-memory `bytes.Buffer`, `render_png.go:85-89`), no profiling output,
  no env reads. Writes go only to the caller-provided `--out` path.
- **No writes outside user-chosen locations.** `me` writes only `--out`/`--manifest`; the
  preview dir must already exist (`main.rs:262-268`) and the sidecar writes only
  `dir/plate-N.ext` inside it.
- **NDEF output non-atomicity is downstream-safe.** `fs::write` is truncate-then-write (a crash
  could leave a partial NDEF), but a truncated record fails the device's TLV/BCH parse — not a
  silent wrong-plate. Out of D5-core; noted for completeness.
- **`firmware/ndef-roundtrip`** is a self-test harness reading NDEF on stdin and printing the
  recovered text to stdout; no files, no secrets held, no argv payload. Clean.

## FOLLOWUPS cross-check

Reviewed `design/FOLLOWUPS.md`. None of the open items covers the D5-1..D5-4 hygiene surface
(they concern the SeedHammer firmware fork and codec/verify internals, not the `me`/sidecar
stderr-echo, file-permission, or stale-preview behavior). The historical `me` nits
(`me-in-stdin-intermediate-zeroize`, `me-canonical-string-stderr`, converter-polish I-1) are
resolved and consistent with the SOUND results above; the bundle-path echo (D5-1) is a distinct,
unrecorded regression of that same "don't echo the input" discipline. No knownFollowup overlaps.
