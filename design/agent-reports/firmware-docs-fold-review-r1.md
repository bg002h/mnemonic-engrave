**⚠ 8 Important findings (0 Critical): the fold left a wrong signed-digest claim, a missing OTP row byte order, a wrong `k >= 2` admission statement, a device-refused `now:`/`hash:` grammar, three GEMINI.md commands that fail as printed, and guide links pinned to a branch the merge deletes.**

# Fold review r1: SH2 firmware architecture docs + seedhammer GEMINI.md

Reviewer: independent Opus slice. Date: 2026-11-01.
Targets: engrave `docs/firmware-architecture` fold `7fa58f5e..8ddfab80`
(4 files, `design/firmware-architecture/`); seedhammer `docs/gemini-init` fold
`ec4fffd..b1d0a29` (`GEMINI.md`). Round-1 findings: Codex I1-I7, M1-M5, H01-H12.
Ground truth: seedhammer `abdcb1825ed3`, engrave `origin/master`, RP2350 datasheet.
This is a first review of a fold authored by Gemini (model/effort not recorded in the
fold commit); none of the findings below came from a previous reviewer's fix commit.

## Findings (appended as found)

### I-1 (Important). HOST_INTERFACES §2.6 says the signed digest covers the embedded public key; it does not

**Where:** `design/firmware-architecture/HOST_INTERFACES.md:228` (added by the fold, answering H04).

**Wrong text:** "Because `picobin.Image.HashData` (`picobin/picobin.go:343–344`) hashes the load-map
ranges **and** the defined portion of the `PICOBIN` block **including the embedded 64-byte public key**
(`X || Y`), signing must: (1) embed the real public key ..."

**Evidence:**
- RP2350 datasheet (build 01/10/2026) §5.9.2.2 HASH_DEF item: "If using this item for a signature, the
  number of words of the block hashed must include all items except the SIGNATURE and HASH_VALUE." The
  public key lives in the SIGNATURE item (§5.9.2.4, Table 491, words 1-16).
- seedhammer `abdcb1825ed3` `picobin/testdata/signed.bin.gz`, parsed: block at 0x1700, HASH_DEF at +48
  says 14 words hashed (bytes +0..+56), SIGNATURE item starts at +56. The key is outside the range.
- Executed probe (`/scratch/cache/fwdocs-review/probe/pk/main.go`, real `seedhammer.com/picobin`): XOR
  every one of the 64 public-key bytes and 64 signature bytes of `signed.bin`, re-run `HashData`:
  `hash ... 15cf016d...783557` before and after, `unchanged=true`.
- engrave `scripts/sign-firmware.sh:139-145` only asserts the *signature* is outside the hashed region;
  embedding the key before hashing is a safe order, not a requirement of the hash.

**Failure:** a reader who builds an external signer from this contract (an HSM or a second
implementation that computes the digest rather than calling `picosign hash`) appends 64 key bytes to the
hashed data, and every signature it produces fails the bootrom check. It also teaches a false security
property: the signature does not bind the key; the OTP `BOOTKEY` hash does. Fails closed (step 7's
`signature: verified` catches it), so Important, not Critical. The same false premise is in Codex H04
("hashing before replacing the dummy public key, can produce a signature over the wrong message"); the
fold copied it.

**Fix:** in `HOST_INTERFACES.md`, replace the anchor
`Because \`picobin.Image.HashData\` (\`picobin/picobin.go:343–344\`) hashes the load-map ranges **and** the defined portion of the \`PICOBIN\` block **including the embedded 64-byte public key** (\`X || Y\`), signing must:`
with
`\`picobin.Image.HashData\` (\`picobin/picobin.go:300–347\`) hashes the load-map data followed by the first \`HASH_DEF\`-count words of the \`PICOBIN\` block, which by the RP2350 datasheet (§5.9.2.2) excludes the \`SIGNATURE\` item, so neither the embedded 64-byte public key nor the signature is covered by the digest; the key is bound only by its SHA-256 in OTP \`BOOTKEY\`. \`scripts/sign-firmware.sh\` nevertheless embeds the real key before hashing and asserts the digest is unchanged after embedding the signature. Its sequence is:`

### M-1 (Minor). §2.1 host NDEF writer: language "en" and the `MAX_TEXT_LEN` constant are invented

`HOST_INTERFACES.md:31` says the `me` writer emits Text "language `"en"`" and cites `MAX_TEXT_LEN = 249`.
engrave `origin/master` `crates/me-cli/src/ndef.rs:7` writes `STATUS_UTF8_NOLANG = 0x00` ("lang-code length
0"; line 29 "empty language"), and no `MAX_TEXT_LEN` exists (`git grep MAX_TEXT_LEN origin/master` is
empty). 249 itself is right (5 header bytes + text < 255, test `ndef_boundary_249_char_text_encodes_with_1byte_len`).
Fails safe: the device strips any `status & 0x3f` language bytes. Conditions for harm: (1) a second host
writer copies "en" and (2) runs at the 248/249-byte edge, losing 2 bytes. Fix: replace
`Well-Known Text (\`'T'\`, language \`"en"\`)` with `Well-Known Text (\`'T'\`, status \`0x00\`: UTF-8, empty language code)`
and `(\`MAX_TEXT_LEN = 249\`)` with `(\`text_record\` + \`tlv_wrap\` length checks, \`ndef.rs:30–57\`)`.

### M-2 (Minor). §1/§2.1 admission table omits the start-screen NFC route; "mk1 chunked only" is wrong

The fold changed the mk1 transport cell (`HOST_INTERFACES.md:15`) from "NFC NDEF" to "NFC NDEF (chunked
only)" and wrote a three-entry "Per-program caller admission" list (`:47–50`). seedhammer `abdcb1825ed3`
`gui/gui.go:2254` starts the NFC scanner on the start screen and `:2280–2290`/`:2210` pass every object to
`engraveObjectFlow` (`:2636–2663`), which admits `bip39.Mnemonic`, `codex32.String`, `*bip380.Descriptor`,
`mdmkText` (a single mk1 included: `mdmkFlow`, `:2798`), `mtText`, `freeTextScan` and `passScan`. The
single-mk1 refusal is the bundle door's only (`gui/bundle.go:173–174`). Scanners also run in
`md1_gather.go:117`, `mk1_inspect.go:210`, `derive_xpub.go:343`, `verify_address.go:82`. Also `:49` quotes
`"Secret share refused"`, which is in no file at `abdcb1825ed3`; the operator text is
`"Type the ms1 share on-device, never over NFC."` (`gui/bundle_flow.go:85`). Fails closed (a reader
under-uses NFC, never sends wrong bytes). Fix: in `:15` replace `NFC NDEF (chunked only)` with
`NFC NDEF (single or chunked from the start screen; chunked only at the bundle door)`; in `:49` replace
`(\`clsMs1Refuse\` → \`"Secret share refused"\`)` with `(\`clsMs1Refuse\` → \`"Type the ms1 share on-device, never over NFC."\`)`;
insert before `:48` a bullet: `Start screen (\`gui/gui.go:2254\`, \`engraveObjectFlow\` \`:2636–2663\`): \`bip39.Mnemonic\` → Backup Wallet, \`codex32.String\` → \`engraveCodex32\`, \`*bip380.Descriptor\` → \`descriptorFlow\`, \`mdmkText\` (single or chunked md1/mk1) → \`mdmkFlow\`, \`mtText\` → transaction gather, \`freeTextScan\`/\`passScan\` → text/passphrase programs; \`addressText\` is reported as unknown format.`
and retitle the Backup Wallet bullet to `Backup Wallet (reached from the start screen)`.


### I-2 (Important). §2.2 seam sentence says `me` admits `k >= 2` shares; it refuses them

**Where:** `design/firmware-architecture/HOST_INTERFACES.md:86` (rewritten by the fold, answering H10).

**Wrong text:** "`me` (`crates/me-cli/src/seal/record.rs:177–231`) validates seed records through
`ms_codec::decode` (requiring `Entr` `0x00`, `Mnem` `0x02`, or `k >= 2` share)".

**Evidence:**
- engrave `origin/master` `crates/me-cli/src/seal/record.rs:210–231`: the Ms arm returns
  `RecordKind::Ms` only for `Ok((_, Payload::Entr(_) | Payload::Mnem { .. }))`; every `Err` becomes
  `RecordError::Invalid`/`TagKindMismatch`/`PreimagePlate`.
- mnemonic-secret `e5dff6f9` (the engrave `Cargo.lock` pin) `crates/ms-codec/src/envelope.rs:136–153`:
  a threshold `'2'..'9'` string returns `Err(Error::IsShareNotSingleString)`, never a payload.
- The seam corpus the sentence cites (seedhammer `abdcb1825ed3` `sysw/testdata/codex32_seam_vectors.json`,
  byte-identical to engrave `crates/me-cli/testdata/codex32_seam_vectors.json`): row `bip93-share`
  (`ms13cash…`, k = 3) and row `bip93-share-payload-0x03` (`ms12test…`, k = 2) are both
  `host_admits: false, device_admits: true`. Rust `sysw::classify` reaches `Class::Codex32Secret` only
  through `seal::record::validate_record` → `RecordKind::Ms` (`crates/me-cli/src/sysw/mod.rs:384–385`).

**Failure:** a reader plans to carry a k-of-n `ms1` share set through `me sysw pack` / `me seal` and
the device; `me` refuses every share. It is a wrong host/device interface contract (brief: Important),
and it inverts the seam's own corpus. Fails closed (`me` refuses).

**Fix:** in `HOST_INTERFACES.md`, replace the unique anchor
`(requiring \`Entr\` \`0x00\`, \`Mnem\` \`0x02\`, or \`k >= 2\` share)` with
`(admitting only an unshared single, threshold \`0\` and share index \`s\`, whose payload is \`Entr\` \`0x00\` or \`Mnem\` \`0x02\`, in the constellation profile's lengths; a \`k >= 2\` share is refused with \`IsShareNotSingleString\`)`.
Anchor checked unique: `grep -c 'or \`k >= 2\` share)' HOST_INTERFACES.md` = 1.

### I-3 (Important). §2.2 `ClassNow` wire grammar written by the fold is refused by the device; `ClassHash` grammar makes the kind mandatory

**Where:** `design/firmware-architecture/HOST_INTERFACES.md:96` (`ClassNow`) and `:95` (`ClassHash`), both
rewritten by the fold (round 1 had "`now:` composer locktime height/time record", vague but not wrong);
the same `hash:<algo>:` form recurs at `:172`.

**Wrong text:** "`ClassNow`: `now:<uint32>` composer locktime reference record." and
"`ClassHash`: `hash:<algo>:<digest-hex>` composer digest record (`sha256`, `hash256`, `ripemd160`, `hash160`)."

**Evidence:** seedhammer `abdcb1825ed3` `sysw/composer_records.go:293–317` `ParseNowRecord` unhexes the
body as lowercase hex, requires UTF-8, cuts at the first comma, and reads decimal seconds `1..2147483647`
and an optional decimal height `1..499999999` (`composerMaxSeconds`/`composerMaxHeight`, lines 39–40).
engrave `crates/me-cli/src/sysw/composer_records.rs:31` documents the same:
"`now:<hex of \"<seconds>[,<height>]\">`". `ParseHashRecord` (`:225–258`) splits on the last colon; an
absent kind means `sha256`, and `HashRecord` (`:264–269`) emits sha256 BARE ("output is conservative").
Executed probe (`/scratch/cache/fwdocs-review/probe/cls/main.go`, real `seedhammer.com/sysw` from the
`docs/gemini-init` worktree, no diff from `abdcb1825ed3` under `sysw/`, `md/`):

```
now:1700000000                              0   (ClassUnknown)
now:31373030303030303030                    12  (ClassNow)
now:313730303030303030302c383530303030      12  (ClassNow)
hash:<64 hex>                               11  (ClassHash)
hash:sha256:<64 hex>                        11  (ClassHash)
hash:hash160:<40 hex>                       11  (ClassHash)
```

**Failure:** a host producer written from `:96` emits `now:<decimal>`; the device classifies it
`ClassUnknown` and refuses the payload. A parser written from `:95` (for example a host-side payload
inspector) requires a kind token and refuses the bare `hash:<hex>` that both `me` and the device emit
for sha256. Wrong host-to-device wire grammar (brief: Important). Fails closed.

**Fix:** in `HOST_INTERFACES.md`, replace the unique anchor
`` `now:<uint32>` composer locktime reference record. `` with
`` `now:<lowercase-hex(UTF8("<seconds>[,<height>]"))>` composer pack-time record: decimal `seconds` `1..2147483647`, optional decimal `height` `1..499999999`, cut at the first comma (`ParseNowRecord`, `sysw/composer_records.go:293–317`); a bare `now:<decimal>` is `ClassUnknown`. ``
Replace the unique anchor
`` `hash:<algo>:<digest-hex>` composer digest record (`sha256`, `hash256`, `ripemd160`, `hash160`). `` with
`` `hash:[<kind>:]<lowercase-hex digest>` composer digest record (`ParseHashRecord`, `sysw/composer_records.go:225–258`): an absent kind means `sha256`, the only form emitted for it (`HashRecord`, `:264–269`); `hash256` takes 64 hex digits and `ripemd160`/`hash160` take 40, each with its token. ``
In `:172` replace `` belongs to `hash:<algo>:<digest-hex>` composer records `` with
`` belongs to `hash:[<kind>:]<digest-hex>` composer records ``.
Anchors checked unique with `grep -c` (1, 1; the `:172` phrase occurs once).

### I-4 (Important). GEMINI.md §4 manual seal step passes an undefined `$DUMMY_PEM` as the key

**Where:** seedhammer `docs/gemini-init` `GEMINI.md:53` (written by the fold, answering I1).

**Wrong text:** `picotool seal --sign --clear --quiet build/firmware.uf2 build/firmware.signed.uf2 "$DUMMY_PEM"`,
under "Equivalent manual steps inside `nix develop`".

**Evidence:** `git grep -n DUMMY_PEM abdcb1825ed3` is empty, and `b1d0a29` defines it nowhere either. The
flake's dummy key is the Nix store path `dummy_pem = pkgs.writeText "dummy.pem"` (`flake.nix:90–99`),
interpolated only into the `build-firmware` script (`:130`). The devShell (`:172–184`) exports no
variable for it. Inside `nix develop` the printed line therefore hands picotool an empty `<key>`
positional (`picotool help seal`: `[<key>] ... Key file (.pem)`). Codex I1 asked for "a quoted
signing-key path or variable". The fold quoted a variable but never set one.

**Failure:** the "equivalent" path fails as printed at the seal step. Fails closed: no image is produced,
and the key is a dummy whose signature `picosign sign -clear` zeroes. A printed command that fails as
printed (brief: Important).

**Fix:** in seedhammer `GEMINI.md`, replace the unique anchor
`# Equivalent manual steps inside \`nix develop\` (seals load-map Clear 0x20000000->0x20082000, then zeroes dummy sig):`
and the three command lines after it (`tinygo build -o build/firmware.uf2 …`, its continuation line,
`picotool seal … "$DUMMY_PEM"`, `go run ./cmd/picosign sign -clear build/firmware.signed.uf2`) with the
single comment line
`# Its steps (tinygo build; patched picotool seal --sign --clear with the flake's own dummy key, which has no shell variable; picosign sign -clear) are in flake.nix:102–136. Do not re-type them; it writes seedhammerii-<version>.uf2 in the current directory.`
Nothing in that line is executable. I did not run `nix run .#build-firmware` because it runs
`picotool seal --sign` (forbidden by the brief).

### I-5 (Important). GEMINI.md §4 plateview command fails as printed in a fresh checkout

**Where:** seedhammer `GEMINI.md:63` (rewritten by the fold, answering I1's `-out` defect).

**Wrong text:** `go run ./cmd/plateview -plate bothproof -o build/preview.svg`

**Evidence (executed):** in a `git archive b1d0a29` copy at `/scratch/cache/fwdocs-review/shsrc`, using
the block's own `export PATH=…` (go1.26.7), the command as printed renders the plate and then fails
with `plateview: open build/preview.svg: no such file or directory` / `exit status 1`.
`cmd/plateview/main.go:77` is a bare `os.WriteFile(out, …)`. No `build/` directory is tracked or created
by a checkout, and none exists in `sh-worktrees/gemini-init`. The tinygo lines above it do create
`build/` (verified: TinyGo 0.41.1 `-o nodir/x.uf2` creates `nodir/`), so the line works only if a
firmware build ran first in the same tree.

**Failure:** the flag fix landed, but the command still fails as printed when run on its own. This is
the I1 class (brief: Important). Fails closed.

**Fix:** replace the unique anchor `go run ./cmd/plateview -plate bothproof -o build/preview.svg` with
`mkdir -p build && go run ./cmd/plateview -plate bothproof -o build/preview.svg`. I ran that as printed
in the same copy: exit 0, `build/preview.svg` is 665,863 bytes and starts `<svg xmlns=…`.

### I-6 (Important). GEMINI.md §4 Refugium ELF gate is printed with one argument; the script needs two

**Where:** seedhammer `GEMINI.md:59`, inside the I1-named "Verified" command block (the line was
written by the fold).

**Wrong text:** `./scripts/refugium-elf-check.sh build/controller-refugium.elf`

**Evidence (executed):** `scripts/refugium-elf-check.sh:4` gives the synopsis
`scripts/refugium-elf-check.sh DEFAULT.elf REFUGIUM.elf`, and `:39–42` exits 2 unless `$# -eq 2`. The
script checks each marker as PRESENT in the default ELF (positive control) and ABSENT in the Refugium
ELF, so it cannot run on one file. CI runs it with two files (`.github/workflows/test.yml:160`,
`./scripts/refugium-elf-check.sh default.elf refugium.elf`). I ran `GEMINI.md:57–59` as printed in a
local clone at `b1d0a29` (`/scratch/cache/fwdocs-review/shclone`), with the flake's TinyGo 0.41.1 store
path and go1.26.7 (`/scratch/cache/fwdocs-review/probe/elfgate.sh`). The Refugium build exits 0. The
check as printed prints `usage: ./scripts/refugium-elf-check.sh DEFAULT.elf REFUGIUM.elf` and exits 2.

**Failure:** the block heading says "forbidden-symbol ELF gate", but the gate fails as printed, so a
reader who follows the guide never runs the device half of the Refugium absence gate. Fails closed:
exit 2 with usage, no false PASS. It is still a printed command that fails as printed (brief:
Important).

**Fix:** in seedhammer `GEMINI.md`, replace the unique anchor line
`./scripts/refugium-elf-check.sh build/controller-refugium.elf`
with these three lines:
```
tinygo build -size short -stack-size 16kb -gc precise -opt 2 -scheduler tasks -target pico-plus2 \
  -ldflags="-X main.Version=v0.0.0-bg$(git rev-parse --short HEAD)" -o build/controller.elf ./cmd/controller
./scripts/refugium-elf-check.sh build/controller.elf build/controller-refugium.elf
```
I ran them as printed, in the same clone, straight after the Refugium build. The default build exits 0
and the check exits 0 (`fix-check-exit=0`; ELFs are 14,441,172 and 13,942,332 bytes).

### M-3 (Minor). ARCHITECTURE §6 ELF-check row pairs it with a UF2-only command and names markers the gate cannot see

**Where:** `design/firmware-architecture/ARCHITECTURE.md:192` (the fold rewrote this row's command
column and added the `Clear` claim).

**Wrong text:** "`nix run .#build-firmware` + `scripts/refugium-elf-check.sh`" and "absence of forbidden
symbols (`LockBoot`, `passphraseFlow`) under `-tags refugium`".

**Evidence:**
- `build-firmware` (`flake.nix:102–136`) builds the default profile only and leaves only
  `seedhammerii-$VERSION.uf2`, so it produces neither of the two ELFs the check needs (see I-6).
- The script's markers (`scripts/refugium-elf-check.sh:48–71`) are four things:
  - the data literals `lock-boot: %v`, the OTP redirect URL and the proof triggers;
  - the symbols `Poller.Read`, `gui.qaEngraveFlow`, `otp.writeECC` and `otp.writeOrRow`;
  - the reverse-control profile notices;
  - nothing named `LockBoot` or `passphraseFlow`.
- `:26–29` of the script says the OTP-writing names have no symbols "even in the default ELF".
- Executed: `readelf -sW` on both ELFs from I-6 shows 0 `gui.passphraseFlow` symbols in either. The only
  `LockBoot` hit in both ELFs is the `Platform` interface method-set string, so no defined `LockBoot`
  symbol is checked.
- The real OTP-writer absence evidence is `otp.writeECC`/`otp.writeOrRow`. The `Clear 0x20000000->0x20082000`
  range itself is right (engrave `design/PICOTOOL_PIN.md:51`).

**Why Minor:**
- (1) Fails closed. The gate does check the OTP writer's absence, under other names, so a reader is not
  misled about the guarantee, only about the evidence.
- (2) No full command line is printed.
- (3) Harm needs two unusual conditions:
  - (a) a maintainer edits the marker list from this row rather than the script header;
  - (b) the added `LockBoot` marker would then fail the default-ELF positive control and stop CI, so the
    harm is wasted work, not a false PASS.

**Fix:** replace the unique anchor
`` `nix run .#build-firmware` + `scripts/refugium-elf-check.sh` `` with
`` `nix run .#build-firmware` (default UF2); `scripts/refugium-elf-check.sh DEFAULT.elf REFUGIUM.elf` on two `-size short` ELF builds (seedhammer `GEMINI.md` §4) ``.
Also replace `absence of forbidden symbols (\`LockBoot\`, \`passphraseFlow\`) under \`-tags refugium\``
with `the Refugium marker set of scripts/refugium-elf-check.sh:48–71 (OTP write primitives \`otp.writeECC\`/\`otp.writeOrRow\`, \`Poller.Read\`, \`qaEngraveFlow\`, the lock-boot log format, the OTP redirect URL and the proof triggers) present in the default ELF and absent under \`-tags refugium\`, with the profile notices the other way round`.

### I-7 (Important). HOST_INTERFACES §2.6 still does not state the OTP row byte order H02 asked for

**Where:** `design/firmware-architecture/HOST_INTERFACES.md:230`, `BOOTKEY0..3` bullet (written by the fold,
answering H02).

**Current text:** "In OTP storage, the 32-byte digest is written across 16 two-byte ECC rows
(`scripts/pico2-bootkey-rehearsal.sh:227–234`)." The text gives no byte order. The same bullet puts
"**Big-Endian byte order**" in bold, two sentences earlier, for the hash *preimage*.

**Evidence:**
- H02's correction has two parts: "Specify SHA-256 over 32-byte big-endian X followed by 32-byte
  big-endian Y ... **Describe OTP row storage byte order separately.**" The fold did the first part only.
- RP2350 datasheet Table 1527 (`OTP_DATA: BOOTKEY0_0… BOOTKEY3_15`, offset `0x080 + k`): bits 15:0 hold
  "Bits N:M of SHA-256 hash of boot key L (ECC)".
- §5.4.8.21 `otp_access`, `IS_ECC`: "each value in the buffer is 2 bytes", little-endian on the M33.
- `driver/otp.WriteBootKey` (`otp.go:159–170`) passes the 32 digest bytes, in order, to
  `writeECC(keyHash, BOOTKEY0_0+slot*16)`. Row `k` therefore holds digest byte `2k` in bits 7:0 and byte
  `2k+1` in bits 15:8.
- The host reader agrees. Engrave `scripts/pico2-bootkey-rehearsal.sh:228` says "Each row holds two
  bytes, LOW BYTE FIRST, so each row is byte-swapped", and `scripts/lib/otp-read.sh:319` reassembles
  each 4-hex row value `v` as `${v:2:2}${v:0:2}`.
- Executed: I built the 16 row values from digest bytes `00..1f` by the rule below (`0100 0302 …`) and
  fed them through line 319's reassembly; the result equals the digest.

**Failure:** a reader provisioning or auditing a slot by raw rows from this contract has to choose a
byte order, for example `picotool otp set -e` per row, or comparing `picotool otp get` output with
`sha256(X||Y)`. The bold "Big-Endian" nearby points them the wrong way. A slot burned with byte-swapped
rows holds a digest that matches no key, and that slot is spent for good. If the reader then sets
`KEY_VALID` and `SECURE_BOOT_ENABLE` from the same table, no image boots on that device. This is an
"implementer must choose" gap in an OTP contract that a round-1 finding named, and it is not a wrong
printed instruction, so: Important.

**Fix:** in `HOST_INTERFACES.md`, replace the unique anchor
`In OTP storage, the 32-byte digest is written across 16 two-byte ECC rows (\`scripts/pico2-bootkey-rehearsal.sh:227–234\`).`
with
`In OTP storage the 32-byte digest is written across 16 two-byte ECC rows, low byte first: row \`0x080 + 16*i + k\` (\`k\` = \`0..15\`) holds digest byte \`2k\` in bits \`7:0\` and byte \`2k+1\` in bits \`15:8\`, so a row read or written as a 16-bit value is \`byte[2k] | byte[2k+1] << 8\` (datasheet Table 1527 and §5.4.8.21 \`IS_ECC\`; \`driver/otp.WriteBootKey\` passes the digest bytes in order; \`scripts/pico2-bootkey-rehearsal.sh:228\` and \`scripts/lib/otp-read.sh:319\` byte-swap each row on read). This storage order is separate from the big-endian \`X || Y\` preimage above.`

### I-8 (Important). GEMINI.md §1 links the four guides by branch name, which the standing merge procedure deletes

**Where:** seedhammer `GEMINI.md:15–18` (written by the fold, answering I6). Each of the four lines uses
`https://github.com/bg002h/mnemonic-engrave/blob/docs/firmware-architecture/design/firmware-architecture/`,
4 occurrences in all.

**Evidence (executed):**
- Today these links resolve: `git ls-remote origin refs/heads/docs/firmware-architecture` gives
  `8ddfab80…`, and `curl` of the ARCHITECTURE link returns HTTP 200.
- The same path on `blob/master/` returns 404. `origin/master` (`85b72a97`) has no
  `design/firmware-architecture/` tree.
- The Refugium merge procedure deletes a remote head branch once
  `git merge-base --is-ancestor` puts it in master. After that, all four primary links in the init file
  404.
- Round-1 I6 asked for "repository-qualified remote links, preferably pinned to the reviewed
  documentation revision" and for standalone views to be checked. The fold's own pinned links
  (`blob/a75719996304…`, `blob/abdcb1825ed3…`) are durable and return 200, but the four guide links are
  not pinned.

**Why not Minor:** the links fail closed (404) and print no command, but they break on one condition
only, and that condition (deleting the branch after merge) is the standard procedure, not an unusual
one. The brief's Minor test needs two or more unusual conditions.

**Fix:** in seedhammer `GEMINI.md`, replace each of the 4 occurrences of
`https://github.com/bg002h/mnemonic-engrave/blob/docs/firmware-architecture/design/firmware-architecture/`
with `https://github.com/bg002h/mnemonic-engrave/blob/master/design/firmware-architecture/`.
Then replace each of the 4 link labels `[GitHub branch]` on those lines with `[GitHub master]`. Landing
order, which the controller owns: merge engrave `docs/firmware-architecture` before seedhammer
`docs/gemini-init`, so that the `master` links resolve the moment `GEMINI.md` lands.

I checked the fix as far as it can be checked before the merge:
- the 4 target files exist at `8ddfab80`;
- `origin/master` has no tree at that path, so the clean merge (settled) adds them unchanged;
- the URL shape `blob/<ref>/<path>` returns 200 for the pinned links already in the file.

A `master` link cannot return 200 until the merge.

### M-4 (Minor). `HOST_INTERFACES.md:12`, `:61` (fold, I5) name `mr` and `mr-gui` as `sysw` producers, but neither has any `sysw` code

**Evidence:** `git grep -il sysw` finds 0 crate files at `mnemonic-refugium` `a0866aa3`, and 0 at
`refugium-wallet` `f081c7be`. The mr-gui spec only *plans* to pack "through mnemonic-engrave's
container code" (`design/SPEC_ui_refugium_wallet.md` §4.1). `ARCHITECTURE.md:25` says `mr` consumes
`sysw` payloads.

**Why Minor:**
- Fails closed: there is no such subcommand, and the wire contract (`me sysw pack`) is right.
- No printed command.
- Harm needs two conditions: (a) a reader needs a non-`me` producer; (b) they trust this column over
  the producer's own CLI.

**Fix:**
- At `:12`, replace `` `me sysw pack` (`crates/me-cli/src/sysw`), `mr` / `mr-gui` (`mnemonic-refugium` / `refugium-wallet`) ``
  with `` `me sysw pack` (`crates/me-cli/src/sysw`); `mr-gui` (`refugium-wallet`) is specified to pack through it (`design/SPEC_ui_refugium_wallet.md` §4.1) but has no packer at `f081c7be`, and `mr` has none ``.
- At `:61`, replace `` orchestrated by `mr` / `mr-gui` (`mnemonic-refugium` / `refugium-wallet`) or host scripts ``
  with `` run directly or from host scripts (`mr-gui` is specified to drive it, not built at `f081c7be`; `mr` has no `sysw` code) ``.

### M-5 (Minor). `HOST_INTERFACES.md:228` (fold, H04) signing sequence drops three of `sign-firmware.sh`'s guards

**Missing guards** (`scripts/sign-firmware.sh` at `a7571999`):
- step 0b (`:82`), which refuses to re-sign an official release;
- step 6 (`:147–150`), the offline `openssl pkeyutl -verify`;
- step 7 (`:179`), "exactly 2 metadata blocks". The script says "Three means the image was sealed twice".

Step 1 (`:98–116`), seal with a throwaway key then `picosign sign -clear`, appears only as the separate
`:225` bullet.

**Why Minor:**
- `sign-firmware.sh` is cited and enforces all three, and `signature: verified` still gates.
- No printed command fails.
- Harm needs two conditions: (a) the reader re-implements signing instead of running the script;
  (b) the input is already sealed, or is an official release.

**Fix:** replace the anchor `` `seal_has_clear_entry` confirms `load map entry 0: Clear 0x20000000->0x20082000`. ``
with `` `seal_has_clear_entry` confirms `load map entry 0: Clear 0x20000000->0x20082000`, `openssl pkeyutl -verify` accepts the signature offline, and `picotool info -a` shows exactly 2 metadata blocks (three means it was sealed twice: rebuild, do not re-sign). Before step (1), `scripts/sign-firmware.sh` refuses an official SeedHammer release (step 0b) and gives an image without a `SIGNATURE` section one by sealing it with a throwaway key and zeroing it with `picosign sign -clear` (step 1). ``

### M-6 (Minor). `HOST_INTERFACES.md:217` (fold) says TinyGo 0.41.1 comes from engrave's flake, but it is pinned in seedhammer's

**Evidence:**
- engrave `flake.nix:27–40` is `picotoolFor`, the patched picotool only. Its `devShells.otp` takes
  an unpinned `pkgs.tinygo` from nixpkgs-unstable (`:65`).
- TinyGo `0.41.1` is seedhammer's own `tinygo-overlay` (`flake.nix:50–52`).
- seedhammer imports only `picotool` from engrave (`:74`).

**Why Minor:** no command, and the firmware build is still reproducible. Harm needs two conditions:
(a) a reader builds firmware from engrave's `otp` shell; (b) nixpkgs' TinyGo differs from 0.41.1.

**Fix:** replace the anchor
`` Nix devshell in `mnemonic-engrave/flake.nix:27–40` (imported by `seedhammer/flake.nix:73–74`) providing TinyGo `0.41.1` + **patched `picotool` 2.3.1** ``
with
`` `seedhammer/flake.nix` devshell: TinyGo `0.41.1` (`tinygo-overlay`, `:50–52`) + **patched `picotool` 2.3.1** imported from `mnemonic-engrave/flake.nix:27–40` (`seedhammer/flake.nix:73–74`) ``.

## Nits (one line each)

- N-1 `SOURCE_MAP.md` `mt` row (fold): `Reassemble` is not a symbol in `mt/` at `abdcb182`; the only hit is
  the comment `md.Reassemble` at `mt/mt.go:17`. Delete `` `Reassemble`, `` from the row.
- N-2 `SOURCE_MAP.md` `driver/*`, `nfc/*` row (fold, M1): labelled "Upstream", but `driver/st25r3916` and
  `nfc/poller` carry fork additions (+244 lines vs `ea4b65be`: the Refugium F5 close path). Use "Upstream +
  fork close path (`driver/st25r3916`, `nfc/poller`, Refugium F5)".
- N-3 `HOST_INTERFACES.md:191`: the `mt/mt.go:136–155` cite stops before `UnsignedInputs []int` at `:157`.
  Make it `136–157`.
- N-4 `HOST_INTERFACES.md:225`: `"$DUMMY_PEM"` is an undefined name (see I-4). Use "a throwaway `secp256k1`
  PEM (`sign-firmware.sh:106` generates one)".
- N-5 `OPEN_QUESTIONS.md` OQ-2 (fold, I4): "checks the module's `go` directive against its `go1.18..go1.25`
  compatibility range" is not in F-494. F-494 measured `cannot compile with Go toolchain version go1.26
  (TinyGo was built using toolchain version go1.25.10)`, and `tinygo version` prints `using go version
  go1.25.10`. Quote that instead.
- N-6 seedhammer `GEMINI.md` §4: `build/` (tinygo, ELF and plateview outputs) is not in `.gitignore`, which
  forbids unignored build outputs. Add `/build/` there, or write to the ignored `_artifacts/`.
- N-7 seedhammer `GEMINI.md:52`, `:58` and the I-6 fix lines stamp `v0.0.0-bg<sha>` without the flake's
  `-dirty` suffix (`flake.nix:118–122`). Acceptable for local ELF gates; say so.
- N-8 `ARCHITECTURE.md` §6 validation table mixes engrave and seedhammer `scripts/` paths with no repo
  prefix. Prefix each with its repo.
- N-9 seedhammer `GEMINI.md:77`: `push-via-staging.sh` is cited by its engrave path, but seedhammer carries a
  byte-identical copy (`cmp` equal). `./scripts/push-via-staging.sh` is shorter.

## Per-finding resolution (round-1 Codex findings)

| finding | status | evidence |
|---|---|---|
| I1 | fixed partly | `--no-overwrite`, `<key.pem>` and plateview `-out` are gone (grep empty). The rewritten block still has three failing lines: `"$DUMMY_PEM"` (I-4), plateview into a missing `build/` (I-5), and a one-argument `refugium-elf-check.sh` (I-6). |
| I2 | fixed | GEMINI §5 and ARCHITECTURE describe `go home()` spawned in `Init()` before LCD init when 20–28 V is present, concurrent with display startup (`platform_sh2.go:267–282`; `engraver.go:186` `home`). |
| I3 | fixed | OQ-3 quotes are verbatim from `seedhammer/CLAUDE.md` and `mnemonic-engrave/CLAUDE.md:11–26`. The GEMINI §6 risk set (a)–(d) and gates match engrave `CLAUDE.md` at `a7571999`. |
| I4 | fixed | OQ-2 separates the host Go 1.26 floor from the `go 1.25.10` directive and names TinyGo `0.41.1` and F-494. The "go1.18..go1.25 range" wording is unsourced (N-5). |
| I5 | fixed partly | `mnemonic`, `mnemonic-gui`/`gui-render` (egui/eframe), `mr` and refugium-wallet `mr-gui` (Slint) match the manifests at the pinned revisions; the CLIs `mnemonic-gui` drives (`mnemonic`, `ms`, `mk`, `md`) match the string literals in its `src` at `46e1c0f3`. The fold also made `mr`/`mr-gui` `sysw` producers, which neither is (M-4). |
| I6 | fixed partly | All relative and pinned links resolve: 11 relative targets (8 engrave, 3 seedhammer) and 5 pinned blobs exist at their revisions, and all 5 pinned blobs and the 4 branch links return HTTP 200 today (`curl -L`). The four guide links are pinned to a branch that the merge procedure deletes (I-8). |
| I7 | fixed | OQ-9 separates `mm = 6400` microsteps/mm, the 25 ms `needlePeriod` dot spacing, and the timing-word stream (`platform_sh2.go:151, 195–223`). |
| M1 | fixed | `cmd/picosign`, `picobin` and `uf2` are relabelled Upstream (`git diff ea4b65be abdcb182` is empty for all three); `oracle` is split out as Fork-only. The `driver/*`/`nfc/*` label overstates (N-2). |
| M2 | fixed | F-76 is removed from OQ-12. |
| M3 | fixed | `cmd/emu/emu.wasm` in SOURCE_MAP and GEMINI. |
| M4 | fixed | Convergence and firmware-native exceptions appear in SOURCE_MAP §5, GEMINI §3 and ARCHITECTURE, matching engrave `CLAUDE.md:28`. |
| M5 | fixed | Text gives 3 attempts, 100 ms settle and 500 ms retry sleep (`platform_sh2.go:459–479`), and `rebootIntoBOOTSEL` on failure. |
| H01 | fixed | CRIT1 `0x040` ×8, BOOT_FLAGS1 `0x04b` ×3 with KEY_VALID 3:0 and KEY_INVALID 11:8, page locks `0xf82–0xf85`, LOCK1 `0x040404`. Checked against `otp.go` and the datasheet OTP tables. |
| H02 | fixed partly | Preimage is now big-endian `X‖Y` without `0x04` (correct). The OTP row storage order the finding asked for is still missing (I-7). |
| H03 | fixed | The patch is `picotool-2.3.1-seal-clear-fix.patch`; the Clear entry is `0x20000000->0x20082000`, enforced by `seal-check.sh:22` and `sign-firmware.sh:195–199`. |
| H04 | fixed wrongly | The CLI synopsis is now right (`hash`/`sign`/`extract`, flags checked against `main.go:30–56`). The fold's rationale says the digest covers the embedded public key; it does not (I-1, executed). Three guards are dropped (M-5). |
| H05 | fixed | `ct_len` excludes the 16-byte tag; the tag slice is separate. |
| H06 | fixed | NDEF decoding, URI transform and 8 KiB bound are reassigned correctly. The host writer's "en"/`MAX_TEXT_LEN` details are invented (M-1). |
| H07 | fixed partly | Multiline/format promises are removed. The start-screen NFC route is missing, "mk1 chunked only" is wrong, and a quoted refusal does not exist (M-2). |
| H08 | fixed | The ms1 profile index, ID, entropy widths and checksum boundary match `codex32/mspayload.go`. |
| H09 | fixed | The `phrase:` and preimage grammars match `ParsePhraseRecord` and `codex32/msencode.go`. The fold-written `now:`/`hash:` grammars beside them are wrong (I-3, executed). |
| H10 | fixed wrongly | The asymmetry section exists, but says `me` admits `k >= 2` shares; it refuses them with `IsShareNotSingleString` (I-2). |
| H11 | fixed | No header version zero. `EveryInputSigned` is described as structural, not cryptographic. Cite range off by two lines (N-3). |
| H12 | fixed | `me-preview render --format … --out …` with stdin text matches `preview/main.go`. |
| shared shell-redirection item | fixed | No `<x.pem>` or `--no-overwrite` remains in any of the 5 files (grep empty). The replacement `"$DUMMY_PEM"` is undefined (I-4, N-4). |

## Printed commands: run and not run

All runs used copies under `/scratch/cache/fwdocs-review/` (a `git archive b1d0a29` copy `shsrc` and a
clone `shclone` at `b1d0a29`), never `/tmp` and never the live worktrees.

**Run, as printed unless stated:**

| printed command (file:line at head) | result |
|---|---|
| GEMINI.md:37–40 host `go test` block (its own `export PATH=…`, go1.26.7) | exit 0 (`data/hosttest.txt`) |
| GEMINI.md:41 `…/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24` | exit 0; "partition verified exhaustive: 1413 == 1413", "all 1413 tests ran across 24 shards", 56 s |
| GEMINI.md:57–58 refugium `tinygo build -tags refugium … -o build/controller-refugium.elf ./cmd/controller` (TinyGo 0.41.1 from the flake's store path) | exit 0, ELF 13,942,332 bytes |
| GEMINI.md:59 `./scripts/refugium-elf-check.sh build/controller-refugium.elf` | exit 2, usage error (I-6) |
| I-6 fix text: default `tinygo build … -o build/controller.elf` then the two-argument check | exit 0 and exit 0; ELF 14,441,172 bytes |
| GEMINI.md:63 `go run ./cmd/plateview -plate bothproof -o build/preview.svg` | exit 1, `open build/preview.svg: no such file or directory` (I-5) |
| I-5 fix text `mkdir -p build && go run ./cmd/plateview …` | exit 0, 665,863-byte SVG |
| `picotool help seal` (no device) | confirms the `[<key>]` positional (I-4) |
| `git show`, `git grep`, `git diff`, `git ls-remote`, `readelf -s` on both ELFs | evidence cited per finding |
| `curl -sf` of the RP2350 datasheet PDF, then a text extract and `grep` (Table 1527, §5.4.8.21, §5.9.2.2) | evidence for I-1, I-7 |
| Go probes: picobin `HashData` on a signed test image (I-1), `ClassNow`/`ClassHash` classify on fold-grammar strings (I-3), the `k >= 2` seam (I-2) | cited per finding |
| A simulation of `otp-read.sh:319` reassembly against the I-7 replacement rule | rule and script agree |
| `curl -L` on every GitHub blob link in GEMINI.md and the four engrave docs (9 URLs) | all 200 today; `blob/master/design/firmware-architecture/ARCHITECTURE.md` is 404 until the merge (I-8) |

**Not run, and why:**

| printed command | reason |
|---|---|
| `nix run .#build-firmware` (GEMINI.md and ARCHITECTURE §6) | runs `picotool seal --sign`; the brief forbids sign/seal in a signing mode |
| GEMINI.md:51–54 manual `tinygo build` / `picotool seal --sign` / `picosign sign -clear` | same; the seal line is also shown to fail as printed by reading (I-4) |
| `scripts/sign-firmware.sh`, any `picotool` device command, `otp-*`, rehearsal or flash script | brief forbids OTP writes, signing, flashing and device access |
| `./scripts/refugium-tagged-test.sh`, `./scripts/test-32bit.sh` | the fold did not change these lines and no round-1 finding names them |
| `./scripts/oracle-live.sh` | same; it also needs the Rust CLIs on PATH |
| `cmd/emu` `build.sh` plus `python3 -m http.server` | the fold changed only the `emu.wasm` name (M3), checked against the tree, not by serving |

C / I / M / N: 0 / 8 / 6 / 9
