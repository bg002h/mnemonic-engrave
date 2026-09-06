# Push + release report: me v0.9.0

Sonnet push+release agent. Brief: `design/agent-briefs/me-v0.9.0-release-brief.md` (its
Dispatch addendum) plus the sequence in
`design/agent-reports/decision-me-0.8.1-release-and-flash-rule.md` §"The exact sequence",
substituting `v0.9.0` for `v0.8.1`. Master tip at dispatch `1aee9db9e7396b3a6436d9b7175bcf748de8498b`
(a records/briefs commit); its parent `9e4ccad20ee744377598a6163084f345dbfd2712` is the release
commit, tagged. Nothing read as `.jsonl`. No FOLLOWUPS/continuity edit made (controller's job).

## Preconditions (all held)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
1aee9db9e7396b3a6436d9b7175bcf748de8498b
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse 1aee9db9^
9e4ccad20ee744377598a6163084f345dbfd2712
$ git -C /scratch/code/shibboleth/mnemonic-engrave show 9e4ccad2:crates/me-cli/Cargo.toml | grep '^version'
version = "0.9.0"
$ git -C /scratch/code/shibboleth/mnemonic-engrave status --short
(empty)
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
$ git -C /scratch/code/shibboleth/mnemonic-engrave tag -l v0.9.0
(empty)
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/tags/v0.9.0
(empty)
```

## Step 2 — master push, frozen window

```
$ scripts/push-via-staging.sh master
== staging 1aee9db9e7396b3a6436d9b7175bcf748de8498b (branch master, 24 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34045641881; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   f3e502f4..1aee9db9  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-x86_64): success
build me (macos-aarch64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 1aee9db9e7396b3a6436d9b7175bcf748de8498b is on master with the required check earned
```

No "Bypassed rule violations" line anywhere in the output.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
1aee9db9e7396b3a6436d9b7175bcf748de8498b
```

`origin/master` = tip. **Master push completed and verified.** This is the milestone the
controller was frozen on.

## Step 3 — tag (9e4ccad2, per the Dispatch addendum, NOT the tip)

Tag message (`/scratch/code/shibboleth/.tmp/tag-me-v0.9.0.msg`), first line plus items drawn
verbatim from `crates/me-cli/CHANGELOG.md`'s `[0.9.0]` section at `9e4ccad2`:

```
me v0.9.0 -- hashlock preimage and phrase plates on the host; ms-codec 0.9

- me sysw pack --pack-preimage admits a hashlock preimage into a payload, either
  an ms1 kind-0x03 plate under the id `hash` or a new phrase: record; without the
  flag both are refused by name, so a preimage can never arrive by accident.
- ms-codec 0.9 is where the hashlock phrase rule and the plate's QR text now
  live, so host and device derive the same digest from the same bytes.
- A kind-0x03 record whose X is not 32 bytes is now named exactly -- "under the
  id `hash` whose X is N bytes, not 32" -- instead of the old 1-in-256 collision
  misdiagnosis (F-503, the host half).
```

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave tag -a v0.9.0 9e4ccad20ee744377598a6163084f345dbfd2712 -F /scratch/code/shibboleth/.tmp/tag-me-v0.9.0.msg
$ git -C /scratch/code/shibboleth/mnemonic-engrave cat-file -p v0.9.0 | head -2
object 9e4ccad20ee744377598a6163084f345dbfd2712
type commit
$ git -C /scratch/code/shibboleth/mnemonic-engrave push origin v0.9.0
To github.com:bg002h/mnemonic-engrave.git
 * [new tag]           v0.9.0 -> v0.9.0
```

## Step 4 — tag-event run

```
$ gh run list --repo bg002h/mnemonic-engrave --commit 9e4ccad20ee744377598a6163084f345dbfd2712 --json databaseId,event,status,conclusion,headSha
[{"conclusion":"","databaseId":34045837267,"event":"push","headSha":"9e4ccad2...","status":"queued"}]
$ gh run watch 34045837267 --repo bg002h/mnemonic-engrave --exit-status
... (all jobs green) ...
EXIT=0
$ gh run view 34045837267 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | "\(.name): \(.conclusion)"'
test (rust + go): success
build me (macos-aarch64): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: success
```

All 8 jobs succeeded, including `build me (macos-aarch64)` (no artifact-upload timeout this
run) and `assemble + sign + release`.

## Step 5 — assets and binary verification

```
$ gh release view v0.9.0 --repo bg002h/mnemonic-engrave --json url,assets
url: https://github.com/bg002h/mnemonic-engrave/releases/tag/v0.9.0
assets (7):
  mnemonic-engrave-v0.9.0-linux-amd64.tar.gz    (3224986 bytes)
  mnemonic-engrave-v0.9.0-linux-arm64.tar.gz    (3021936 bytes)
  mnemonic-engrave-v0.9.0-macos-amd64.tar.gz    (3161641 bytes)
  mnemonic-engrave-v0.9.0-macos-arm64.tar.gz    (3044924 bytes)
  mnemonic-engrave-v0.9.0-windows-amd64.zip     (3044975 bytes)
  SHA256SUMS                                    (544 bytes)
  SHA256SUMS.minisig                            (296 bytes)
```

Downloaded `SHA256SUMS*` + linux-amd64 tarball into a scratch dir under
`/scratch/code/shibboleth/.tmp/me-v0.9.0-verify.1aockf/`:

```
$ minisign -Vm SHA256SUMS -P RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW
Signature and comment signature verified
Trusted comment: mnemonic-engrave v0.9.0 SHA256SUMS
$ sha256sum -c --ignore-missing SHA256SUMS
mnemonic-engrave-v0.9.0-linux-amd64.tar.gz: OK
$ tar xzf mnemonic-engrave-v0.9.0-linux-amd64.tar.gz   # extracts flat: me, me-preview, VERIFY.txt, minisign.pub, THIRD_PARTY_LICENSES
$ ./me --version
me 0.9.0
```

### Behaviour check 1 — refused without --pack-preimage

```
$ printf '%s\n' ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c \
    | ./me sysw pack --out ./p.bin
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed
record; this payload did not ask for one. A preimage backs a hashlock spend path, not a
wallet -- keep it with the policy it unlocks, and do not re-encode it as entropy. Re-run
with --pack-preimage if that is what you intend.
exit=4
ls p.bin -> No such file or directory (not written)
```

PASS: exit 4, names a hashlock PREIMAGE plate by name, no output file.

### Behaviour check 2 — --pack-preimage --no-passphrase

```
$ printf '%s\n' ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c \
    | ./me sysw pack --pack-preimage --no-passphrase --out ./q.bin
me: WARNING -- this payload carries a hashlock PREIMAGE. Anyone who holds the tag can read
it, and for a key-less hashlock path the preimage alone spends the coins. Treat this payload
as bearer material until it is on the machine and erased.
me: note -- this payload holds no `hash:` record, so nothing here says which policy the
preimage unlocks. The Hashlock plates flow will print the digest alone.
sealing:  NOT SEALED -- you passed --no-passphrase, and this payload HOLDS SECRET MATERIAL
(record 0 (hashlock preimage plate)). It will sit in flash in cleartext.
strength: no passphrase -- BELOW the threshold
me: WARNING -- this payload carries secret material with weak or no passphrase protection.
Proceeding (spec §13 D3).
digest:   0766 4c0c de9b 476a 3bb4 4ec0 7246 cb2a
          re-print it with: me sysw show ./q.bin
exit=0
q.bin written (127 bytes)
```

PASS: exit 0, q.bin written, run warns the payload carries a preimage.

### Behaviour check 3 — F-503, X is 16 bytes not 32

```
$ printf '%s\n' ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c \
    | ./me sysw pack --pack-preimage --no-passphrase --out ./r.bin
me: record 0 (records count from 0) is a kind-0x03 preimage payload under the id `hash`
whose X is 16 bytes, not 32. A preimage plate is kind 0x03 followed by exactly 32 bytes
(SPEC_ms_hashlock §1), and --pack-preimage admits only that. This string is a damaged or
hand-built plate: re-encode it with `ms hashlock` from the phrase or the 32-byte preimage
rather than editing the string.
exit=4
ls r.bin -> No such file or directory (not written)
```

PASS: exit 4, first stderr line names the id `hash` and says X is 16 bytes, not 32 (F-503) --
NOT the old "id is not hash" misdiagnosis.

## Verdict

RELEASED. Tag `v0.9.0` -> `9e4ccad20ee744377598a6163084f345dbfd2712`, run `34045837267` all 8
jobs success, 7 assets present, minisign + sha256 verify, binary reports `me 0.9.0`, all three
behaviour checks pass. Release url: https://github.com/bg002h/mnemonic-engrave/releases/tag/v0.9.0
