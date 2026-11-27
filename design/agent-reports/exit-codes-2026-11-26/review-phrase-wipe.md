# Adversarial review: phrase-material wipe commits (2026-11-27)

Scope: mnemonic-engrave 70f352e, mnemonic-secret ba72a5c, seedhammer 68de0d5.
Lens: correctness regressions (behaviour change, use-after-wipe, wipe of a live
buffer, predicate non-equivalence). Secret-residue findings are reported as
Minor per the operator's 2026-08-27 ruling (the secret-handling class is never
Critical or Important).

## Verdict

**0 Critical / 0 Important / 4 Minor.** No behaviour regression and no
use-after-wipe found in any of the three commits.

## What was executed

- me (70f352e): `cargo test --locked -p mnemonic-engrave --no-fail-fast`: 34
  binaries ok. 4 failures, all from the environment and none touch the diff:
  `history_purge` x3 (no `/usr/bin/zsh` on this box) and
  `sysw_cli::pack_does_not_print_a_digest_for_a_write_that_failed`. That test
  expects a write to `/dev/null` to be refused, and here `/dev/null` is
  `crw-------` with uid 0. I reproduced it by hand: `me sysw pack --out
  /dev/null` exits 0 on this box, and `emit` was not touched by the commit.
  `cargo fmt --check` and `cargo clippy --all-targets -D warnings` are clean.
  (cargo-nextest is not installed here.)
- ms-codec (ba72a5c): lib tests 95/95, including
  `streaming_predicate_matches_the_allocating_one`. I also ran a differential
  fuzz of 3,000,000 random inputs, old against new. The inputs mixed bech32,
  upper case, `-`, `,`, ASCII and Unicode whitespace (U+00A0, U+2003, U+3000,
  U+FEFF, U+0085, U+2028), `é` and invalid prefixes. 0 differences.
- seedhammer (68de0d5), GOTOOLCHAIN=go1.26.0 with a scratch `-modfile` (go
  1.26.0): `go vet` passes on sysw, hashlock and gui, and `go test ./gui ./sysw
  ./hashlock` passes in full (gui 313 s). The fuzz was a scratch test, removed
  afterwards with the tree left clean: 2,000,000 random inputs comparing
  `ms1ShapedOld` with `IsMS1Shaped` and `isMS1ShapedBytes`. The inputs included
  Unicode whitespace at the ends and in the middle, `\v` and `\f`, and a
  trailing invalid UTF-8 byte. 0 differences.

## Commit 1: me 70f352e

Checked and sound:
- **Record order and the `-` splice.** These are unchanged. `out` is still
  built by the same loop over argv, with `from_stdin` cloned in at the `-`
  position. Its capacity is exact, and a realloc of a `Vec<String>` moves only
  the headers anyway.
- **BOM.** `starts_with('\u{feff}')` followed by `drain(..3)` is equivalent to
  the old `strip_prefix`, and it removes exactly one BOM. The existing six-case
  test still passes. The tail bytes the drain leaves behind sit in spare
  capacity, and they are wiped. With zeroize 1.9.0 (Cargo.lock), `String::zeroize`
  calls `Vec<u8>::zeroize`, which also zeroes `spare_capacity_mut()`.
- **Empty-input refusal.** `no_records_guard(Vec::new(), None)` in the splice
  arm is called only when `from_stdin.is_empty()`, so its message is the same
  as before. The exit code is still `EXIT_INVALID` on all three channels.
- **`take()`.** `mem::take` moves the original `Vec` out intact and leaves an
  empty one behind, so no string is copied. `no_records_guard` returns its input
  by value on `Ok` and errs only when the input is empty, so a refusal drops no
  secret unwiped.
- **`pack_with(take(recs), ...)`.** `pack_deterministic_with` calls `split`
  first (sysw/mod.rs:601), so every later refusal (iterations, empty or long
  passphrase, crypto, bound) happens after the records have been partitioned
  into `Payload`. `split` itself has no error exit inside its loop, so the
  `mem::take` at :777 cannot strand the unprocessed records on a refusal. Every
  refusal before the loop (`admit_check`, `SecondNow`) now drops them wiped.
- **The `--as` reassignment.** `recs = Zeroizing::new(strings)` at main.rs:2100
  drops the previous wrapper wiped.

Minor:
- **M1. The stdin read reallocates, so the claim "no second copy exists" is
  too strong** (main.rs:3866-3868; FOLLOWUPS line `sh-phrase-record-decoded-buffer-unwiped`).
  `stdin().read_to_string(&mut *raw)` grows the `String` with no size hint on a
  pipe. Each reallocation frees the previous buffer, unwiped, and that buffer
  holds a prefix of the stream. Scenario: `cat seed.txt | me sysw pack ...`
  with a multi-KiB stream leaves freed heap chunks that hold `phrase:`/`ms1`
  text. This was true before the commit too, so it is not a regression. The
  only problem is that the FOLLOWUP update and the commit message read as if
  the stdin read were fully covered. To close it, pre-reserve a cap such as
  REGION_LEN (and refuse anything over it), or read into a fixed
  `Zeroizing<Vec<u8>>` and grow it by hand with wipe-on-grow. `--in`
  (`fs::read_to_string`) reserves from the file's metadata, so in practice it
  does not reallocate.
- **M2. Doc-comment placement** (main.rs:3584-3588). The new `///` paragraph
  comes after `#[derive(Debug)]`. That compiles, and rustdoc attaches it, but
  it splits the struct's doc in two around the attribute. Cosmetic. Separately,
  and from before this commit: the `Debug` derive on `Records` prints the
  records in full through `Zeroizing`'s `Debug`. Nothing formats it today.

## Commit 2: ms-codec ba72a5c (`looks_like_ms1`)

I found no defects. Exact equivalence, argued and then confirmed by fuzz:
- **Filter before lowercase versus lowercase before filter.**
  `to_ascii_lowercase` maps only `A`-`Z`, and none of those letters is
  whitespace, `-` or `,`. Swapping the two steps therefore changes nothing.
- **`trim()`.** On both sides, Unicode end-trim is subsumed by the
  `is_whitespace` filter, so the trim is a no-op on the filtered stream.
- **Length.** Old: `t.len()` in bytes. New: `n` adds `len_utf8()` only for
  accepted characters. `m`, `s`, `1` and the charset are all ASCII, and any
  non-ASCII character returns false at once (old: either the `starts_with`
  check or the charset check over `t[3..]` fails). So `n` equals `t.len()`
  whenever the result can be true. `MIN_MS1_LEN = 48` is at least 3, so
  inputs shorter than the prefix are false on both sides.
- **No allocation.** `trim`, `chars`, `filter`, `map` and `str::contains(char)`
  allocate nothing.

## Commit 3: seedhammer 68de0d5

Checked and sound:
- **Predicate equivalence.** `IsMS1Shaped` keeps the same Unicode-aware
  `strings.TrimSpace` and then filters bytewise over the same six ASCII
  separators. A non-ASCII byte (or the U+FFFD that an invalid byte decoded to
  in the old code) is never in the charset and never matches m, s or 1, so the
  new code fails at exactly the position the old one did. `len` counted bytes
  in the old code, and every accepted byte is ASCII, so `n` matches it.
  `isMS1ShapedBytes` uses `bytes.TrimSpace`, which is Unicode-aware just like
  `strings.TrimSpace`. Its only caller is `ValidatePhrase`, after the
  printable-ASCII check. Host-and-device drift on interior Unicode whitespace
  existed before the commit and is unchanged.
- **`defer clear(b)` cannot wipe the returned phrase.** The deferred `clear`
  captures `b`'s header (dst from `hex.DecodeString`), while the returned
  `Phrase` is `bytes.Clone(phrase)`, a separate backing array. On `!ok`, `b` is
  nil and `clear(nil)` is a no-op. In Go 1.26, `hex.DecodeString` allocates
  `dst` at exactly `DecodedLen` and the input is pre-validated, so
  `b[:len] == b[:cap]` and the whole buffer is cleared. Parsing is unchanged:
  the first comma (`IndexByte`) is cut as `strings.Cut` cut it, and
  `utf8.Valid` still runs first.
- **`classifyComposer`.** `rec.Wipe()` on a zero-value `PhraseRecord` is safe,
  and the class result is unchanged.
- **composerHashEdit row-set scrub (composer_hash.go:640-652).** The deferred
  closure captures `rows` by reference, so it scrubs the latest set. A rebuild
  scrubs the previous set first. `hashlockPayloadRoute` receives
  `rows.phrases[k]` by value, which aliases the row set's buffer. Its first
  statement is `bytes.Clone(rec.Phrase)` (composer_hashlock.go:157), and only
  that clone reaches `hashlockDeriveFlow` and `composerHoldHashlockMaterial`.
  `st.hashlockHeld` therefore never aliases a buffer the scrub zeroes. I found
  no use-after-wipe. `hashlockDerivedDigest` (`bytes.Equal`) runs only during
  `composerHashRows`, before any scrub of that set, and scrubbed sets are
  nil'd. The `bytes.Equal` compare matches the old `string(...) !=` compare,
  including the case of empty versus nil.
- **Plates (composer_hashlock_plates.go:92, 159-165, 241-243).** The plates list
  now owns the only copy, and `hashlockPlatesScrub` (deferred at :243) wipes
  it. `hashlockPlatesMaterial` copies the slice header into a local `m`. That
  `m` reaches `hashlockPlatesFormPick` and `composerHashlockPlateFor` only.
  This flow has no `composerState`, so nothing is put into `st.hashlockHeld`,
  and `m` does not outlive the deferred scrub. **Not a use-after-wipe.** The
  same aliasing existed before (it pointed at the `[]byte(rec.Phrase)` copy).
- **TinyGo.** `clear` already ships in firmware code (engrave/engrave.go:550,
  1158), and generics ship there too (md/canonicalize.go:270 `cloneSlice[T any]`).
  `bytes.Clone` is plain std. The `string | []byte` union with indexing has a
  repo precedent in bc/bytewords (`~string | ~[]byte`), but I found no non-test
  importer of bc/ur, so that precedent may not be in the firmware image. **I
  did not build with TinyGo**: no tinygo binary is in /nix/store on this box.
  Run the CLAUDE.md firmware size recipe once to confirm.

Minor:
- **M3. Unwiped copies remain on the derive paths, from before this commit.**
  `hashlockPayloadRoute` (composer_hashlock.go:157-170) clones the phrase. On
  the two `return hashlockBackToWhichHash` exits (Back during the countdown,
  or confirm declined), the clone is dropped without a wipe. Scenario: the
  operator picks a payload phrase record, then backs out of the confirm, and
  the phrase's bytes stay on the heap until GC reuse. Before the commit the
  `[]byte(rec.Phrase)` copy did the same, so this is not a regression. Fix:
  `defer` a wipe unless the clone was handed to `composerHoldHashlockMaterial`.
  Similarly, `composer_preimage_plate.go:304` `plate.Phrase =
  string(p.material.phrase)` still makes an immutable string copy for the
  engrave. That is out of this commit's scope.
- **M4. ParsePhraseRecord's residue outside `b`** (sysw/composer_records.go:162-170).
  The record body is a Go string of the phrase's hex, so it is the phrase in a
  trivially decodable form. It lives in `s.records[i].body`, and
  `hex.DecodeString` makes a temporary `[]byte(s)` copy of it. Neither can be
  wiped. The FOLLOWUP should say plainly that the payload-record path is not
  fully wiped while `body` is a string. (This is a statement of scope, not a
  regression.)
