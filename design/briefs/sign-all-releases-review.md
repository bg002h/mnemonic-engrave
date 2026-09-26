# Brief — adversarial review: minisign signing across all six repos + installer verification

**One question:** after these branches merge, can a release ship with a
checksum file that is unsigned, signed with the wrong key, or whose signature
doesn't cover the bytes actually uploaded, while CI reports success? And can
the installer trust a checksum file whose signature it didn't verify, or refuse
a correctly signed one? Not a fresh audit of the release pipelines.

## Inputs

- Branch `sign-releases` in six repos (diff each against origin's default branch):
  engrave `me-worktrees/sign`, toolkit `tk-worktrees/sign`, ms `ms-worktrees/sign`,
  md `dm-worktrees/sign`, mk `mk-worktrees/sign`, gui `gui-worktrees/sign`
  (all under `/scratch/code/shibboleth/`).
- Reports (claims to re-run): `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/sign-all-releases.md`
  and `sign-all-releases-2.md`.

## Settled — do not re-derive

- One key for everything (operator). New key: id `EF2B8D34D8409754`, pubkey
  `RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5`. The retired key
  `CA39ECB257009A0F` / `RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW`
  is trusted only for engrave <= v0.12.0. Engrave's trust-anchor switch is
  operator-approved.
- Secrets `MINISIGN_SECRET_KEY` and `MINISIGN_SECRET_KEY_PASSWORD` exist in all six repos.
- Proof runs (non-publishing, sign + verify succeeded): toolkit 36215620417,
  36215621875; ms 36215623161, 36215624391; md 36215625533, 36215626780;
  mk 36215628013, 36215629411; gui 36215630581; engrave 36217201128.
- **Never read or print the secret key or password.** You may verify real
  signatures only with public keys.

## Push hardest on

1. **Tag path vs dispatch path:** for each repo, read the workflow
   conditions and construct the case where a release tag run ships unsigned:
   the secret missing, a signing step skipped by an `if:`, `continue-on-error`,
   `|| true`, or an upload step that runs before signing or doesn't depend on
   it. Does every uploaded `SHA256SUMS*` get its `.minisig` uploaded with it?
2. **Sign/verify covers the final bytes:** is anything (a regenerated sums file,
   a later `--clobber` upload from another workflow) able to replace the
   checksum file after it was signed? The toolkit has several workflows
   uploading to the same tag (release.yml, man-pages.yml, doc workflows). Check
   the race.
3. **Reproducibility path:** confirm signing happens on the host after the
   digest is final, and changes nothing the repro gate hashes; no secret enters
   the container.
4. **Trust anchor:** every verify step uses the NEW key (engrave's retired key
   only in docs for old releases). grep every repo for both keys.
5. **Installer:** construct: a bad signature, a missing `.minisig` at or after
   first-signed, a signature by the retired key on a component that doesn't
   allow it, a signature over a DIFFERENT sums file (valid sig, wrong file),
   minisign absent, and `--require-signature` with each. Re-run
   `install-signature.test.sh` and `install-verify.test.sh` (both modes), and
   check a mutation of the signature check turns them red.
6. **Real artifacts:** download a signed checksum file and `.minisig` from one
   proof run's artifacts (if uploaded as artifacts) or from engrave's run, and
   verify with `minisign -V -P <new key>` yourself.
7. Docs: the README and verify docs commands, run as written, work.

Scratch under `/scratch/code/shibboleth/review-sign-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branches. No tags or releases.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to merge: yes` or `ready to merge: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/sign-all-releases-review.md`**
and return only a short summary plus that path.
