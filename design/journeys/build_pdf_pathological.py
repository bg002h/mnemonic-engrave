#!/usr/bin/env python3
"""Assembles the pathological-wallet operator journey.

Same rule as the first document: every CLI block is transcript.sh's real
stdout+stderr, every screenshot is the emulator's own framebuffer, every plate
is a real render. The refusals are reported with their actual exit codes rather
than edited out -- three of them are the most useful thing here.
"""

import base64, html, json, os, re, shutil, subprocess, sys

# ── Missing-asset gate (P1, 2026-08-19) ────────────────────────────────────
# `img()` renders a "missing:" placeholder when a screenshot is absent, and the
# build used to carry on and exit 0. Measured by the constellation recon: BOTH
# wallet journeys were building with 100 % of their referenced screenshots
# missing and still reporting success -- a document that silently asserts less
# than it claims, which is the failure the journeys README is itself about.
#
# A skipped step must FAIL, not pass. Enforcement is the DEFAULT and needs no
# flag to switch on; `--allow-missing` is the explicit opt-OUT for a deliberate
# draft build. Never the reverse -- a gate that an env var or flag has to ENABLE
# is a gate that is off whenever someone forgets it.
#
# (Duplicated verbatim in the three build_pdf*.py scripts, which already carry
# their own copies of img(). Keep them identical.)
MISSING = []
ALLOW_MISSING = "--allow-missing" in sys.argv


def missing_gate(artifact):
    """Exit non-zero if any referenced asset was absent. Call last."""
    if not MISSING:
        return
    uniq = sorted(set(MISSING))
    print(
        f"\n{len(MISSING)} missing asset(s), {len(uniq)} distinct -- "
        f"{artifact} is INCOMPLETE:",
        file=sys.stderr,
    )
    for name in uniq:
        print(f"  missing: {name}", file=sys.stderr)
    if ALLOW_MISSING:
        print("--allow-missing given: exiting 0 with an incomplete document.",
              file=sys.stderr)
        return
    print("Refusing to report success. Capture the screenshots (see "
          "README.md 'Reproducing'), or pass --allow-missing for a draft.",
          file=sys.stderr)
    sys.exit(1)


W = os.path.dirname(os.path.abspath(__file__))
OUT, SHOTS = os.path.join(W, "out", "pathological"), os.path.join(W, "shots")

# THE PLATE CAPTIONS ARE CAPTURED DATA, NOT PROSE (F-210).
#
# They used to be typed beside the images -- "head 23.2, 20.6 mm - into word 1".
# A typed caption can drift from the picture it labels while both still look
# fine, and this one HAD: re-capturing showed all twelve words already cut at
# the sample the text called "three words down". capture_pathological.py writes
# what the overlay itself rendered at each sample, so the numbers under a plate
# are that plate's own.
CAPTIONS = {}
_capfile = os.path.join(SHOTS, "captions-pathological.json")
if os.path.exists(_capfile):
    import json as _json
    for _c in _json.load(open(_capfile)):
        CAPTIONS[_c["name"]] = _c

def plate_caption(name):
    """The caption for one plate shot, or a refusal to invent one.

    The overlay renders "plate N - plan in grey, cut in black ... head X,Ymm".
    Only the head reading and the cut progress belong under the image; the rest
    is said once in the prose above it.
    """
    c = CAPTIONS.get(name)
    if not c:
        MISSING.append(name + " (caption)")
        return "no caption captured for this plate"
    m = re.search(r"head\s*([\d.]+),\s*([\d.]+)mm", c["caption"])
    head = f"head {m.group(1)}, {m.group(2)} mm" if m else c["caption"]
    return f"{head} — {c.get('percent', '?')}% of the cut ({c.get('steps', '?')} steps)"


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(path, cls="", cap=None, crop=None):
    if not os.path.exists(path):
        MISSING.append(os.path.basename(path))
        return f'<p class="missing">missing: {html.escape(os.path.basename(path))}</p>'
    src = path
    if crop:
        from PIL import Image
        im = Image.open(path)
        src = path.replace(".png", f"-c{int(crop*100)}.png")
        im.crop((0, 0, im.width, int(im.height * crop))).save(src)
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(src)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def code(text, limit=None):
    lines = text.rstrip("\n").split("\n")
    clip = limit and len(lines) > limit
    if clip:
        lines = lines[:limit]
    body = html.escape("\n".join(lines))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in out/transcript.txt</span>"
    return f"<pre>{body}</pre>"


def filebox(path, label=None, limit=None):
    with open(path) as f:
        return (f'<div class="file"><div class="fname">'
                f'{html.escape(label or os.path.relpath(path, W))}</div>{code(f.read(), limit)}</div>')


# F-210 class: was out/transcript.txt — an intermediate nothing writes, AND
# the OPERATOR journey's filename, so this document was being built from the
# wrong journey's transcript. Reads its own tracked artifact now.
transcript = open(os.path.join(W, "transcript_pathological.txt")).read()
S, cur, buf = {}, None, []
for line in transcript.split("\n"):
    m = re.match(r"^########## (.*)$", line)
    if m:
        if cur:
            S[cur] = "\n".join(buf).strip("\n")
        cur, buf = m.group(1), []
    else:
        buf.append(line)
if cur:
    S[cur] = "\n".join(buf).strip("\n")


def sect(name):
    """Fetch a transcript section, LOUDLY.

    I-1: every lookup here used to be `S.get(name, '')`, so renaming a
    `##########` heading in the transcript script rendered that block as an
    empty <pre> and the document still built. A rename broke a consumer nobody
    grepped for, and the only symptom was a blank box in a 700 KB page. Missing
    sections are now a hard failure that names the heading and lists what the
    transcript actually contains.
    """
    if name not in S:
        raise SystemExit(
            f"transcript has no section {name!r}\n"
            f"  (a heading was renamed in transcript_pathological.sh?)\n"
            "  available: " + "\n             ".join(sorted(S)))
    return S[name]


# Was json.load(open("keys.json")) -- a file that was never committed, so this
# script could not run at all. The two fields it uses (fingerprint and origin
# path) are already in the committed key files' header comments, which makes the
# captions traceable to an input rather than to a vanished artifact.
def _keys_from_inputs():
    d = os.path.join(W, "inputs-pathological", "keys")
    out = []
    for f in sorted(os.listdir(d)):
        m = re.search(r"\[([0-9a-f]{8})/([^\]]+)\]", open(os.path.join(d, f)).read())
        if not m:
            raise SystemExit(f"{f}: no [fingerprint/path] header -- cannot caption its plates")
        out.append({"fp": m.group(1), "path": "m/" + m.group(2)})
    return out


keys = _keys_from_inputs()
# design/journey/ has never existed; the sibling builder is in THIS directory.
# PLATE COUNTS ARE COUNTED, NEVER TYPED.
#
# This document asserted 22 mk1 strings, 25 public plates and 28 total. Measured
# against the artifacts the transcript beside it had just written: 30, 34 and 35.
# The "each key splits into 2 chunks" premise was the error -- some keys need
# three -- and nothing recomputed it when the wallet changed. Now nothing can.
def _count_lines(path, prefix):
    try:
        with open(path) as f:
            return sum(1 for l in f if l.startswith(prefix))
    except OSError:
        return 0


MK1_CHUNKS = _count_lines(os.path.join(OUT, "backup-strings.txt"), "mk1")
MD1_CHUNKS = _count_lines(os.path.join(OUT, "backup-strings.txt"), "md1")
PUBLIC_STRINGS = _count_lines(os.path.join(OUT, "backup-strings.txt"), "")
try:
    with open(os.path.join(OUT, "manifest.json")) as _f:
        _m = json.load(_f)
    TOTAL_PLATES = len(_m if isinstance(_m, list) else _m.get("plates", _m.get("entries", [])))
except Exception:
    TOTAL_PLATES = 0
if not (MK1_CHUNKS and MD1_CHUNKS and PUBLIC_STRINGS and TOTAL_PLATES):
    sys.exit("plate counts could not be measured; run transcript_pathological.sh first")

CSS = open(os.path.join(W, "build_pdf.py")).read()
CSS = CSS.split('CSS = """')[1].split('"""')[0]

P = []

P.append(f"""
<div class="page">
<h1>SeedHammer II — the pathological wallet</h1>
<div class="sub">Backing up the constellation's <b>pathological example</b> from
files: 11 keys, all four Bitcoin timelock kinds, and a hashlock. Host CLIs, the
firmware GUI in the emulator, and the plates. No NFC anywhere.</div>

<div class="warn"><b>Test material only — never put funds behind these keys.</b>
The three masters are BIP-39's own published test vectors
(<code>abandon…about</code>, <code>legal winner…yellow</code>,
<code>letter advice…above</code>). They are public by construction.</div>

<h2>The wallet</h2>
<p>From <code>mnemonic-toolkit</code>'s Examples §5, "Custom degrading-miniscript
wallet — <b>the pathological example</b>". A four-tier vault that deliberately
mixes every timelock kind Bitcoin has:</p>
<table>
<tr><th>Tier</th><th>Spend condition</th><th>Timelock</th><th>Keys</th></tr>
<tr><td>1</td><td>3-of-3 + secret word</td><td><code>after(1000000)</code> — absolute <b>height</b></td><td>@0 @1 @2</td></tr>
<tr><td>2</td><td>2-of-3 + secret word</td><td><code>after(1893456000)</code> — absolute <b>time</b> (2030-01-01)</td><td>@3 @4 @5</td></tr>
<tr><td>3</td><td>both</td><td><code>older(65535)</code> — relative <b>blocks</b> (~455 d)</td><td>@6 @7</td></tr>
<tr><td>4</td><td>any 1 of 3</td><td><code>older(4255898)</code> — relative <b>time</b> (~365 d)</td><td>@8 @9 @10</td></tr>
</table>
<p style="margin-top:7px">The secret word is <code>opensessame</code>; the
descriptor commits to <code>H = sha256(sha256(word))</code>, shared by tiers 1
and 2. Reusing a <em>hash</em> across tiers is fine — it is not a key. The 11
keys come from three masters at divergent account indices
(<code>48'/0'/0..3'/2'</code>), so no key is reused.</p>

<div class="note"><b>This is the case that actually forces chunking.</b>
The 12-key <code>multi(5,…)</code> policy tried first encodes to 13 bytes and one
md1 string — a keyless BIP-388 template does not pay per key. This one carries
two 32-byte hash literals and four timelock arguments, and comes to
<b>182 data symbols against a single-string cap of 80</b>. It is the first
policy in this work that genuinely needs a chunk set.</div>

<h2>Toolchain</h2>
{code(sect('versions'))}
</div>
""")

P.append(f"""
<div class="page">
<h2>The input files</h2>
<h3>The policy</h3>
{filebox(os.path.join(W,'inputs-pathological','wallet-policy.txt'), 'inputs-pathological/wallet-policy.txt')}
<h3>The three masters (SECRET — BIP-39 test vectors)</h3>
<div class="grid2">
{''.join(filebox(os.path.join(W,'inputs-pathological','seeds',f), 'inputs-pathological/seeds/'+f)
         for f in sorted(os.listdir(os.path.join(W,'inputs-pathological','seeds'))))}
</div>
<h3>The eleven keys</h3>
<div class="grid2">
{''.join(filebox(os.path.join(W,'inputs-pathological','keys',f), 'inputs-pathological/keys/'+f)
         for f in sorted(os.listdir(os.path.join(W,'inputs-pathological','keys'))))}
</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>Host step 1 — it does not fit one string</h2>
{code(sect('1. the policy — 11 keys, four timelock kinds, a hashlock'), 8)}
{code(sect('2. it does not fit one string'), 8)}

<h2 style="margin-top:14px">Host step 2 — so it is chunked, with an explicit origin</h2>
<p>Encoded without <code>--path</code>, this policy's card carries no origin and
<code>md</code> says so: its <code>wsh(or_i(…))</code> wrapper has no canonical
default derivation path, so <code>md decode</code> only PARTIAL-decodes (exit 4,
VERIFY-ME) and <code>me bundle</code> rejects the set outright. Supplying the
origin the warning asks for is what makes the rest of the chain work.</p>
{code(sect('3. so it is chunked -- WITH the origin the warning asked for'), 14)}
<div class="note"><b>What <code>--path</code> does and does not say.</b>
It records ONE shared origin — <code>bip48</code> and <code>m/48'/0'/0'/2'</code>
produce a byte-identical chunk set here. These eleven keys actually sit at four
account indices (<code>48'/0'/0..3'/2'</code>) across three masters, and each mk1
card carries its own true <code>origin_path</code>. So the md1's shared value is
a default the key cards override, and a restore that trusted the descriptor card
alone would derive the wrong key for most slots.</div>

<div class="note"><b>That is now pinned, and the pin corrected the claim.</b>
This paragraph used to say the wrong keys were <b>@3–@10</b> and that the case
was "worth pinning with a restore test". It has been pinned —
<code>restore_test_pathological.py</code>, run below — and the measured answer
is <b>@1, @2, @3, @5, @6, @7, @9, @10</b>: eight of eleven, and the range was
wrong in <i>both</i> directions. It included @4 and @8, which recover fine, and
omitted @1 and @2, which do not. Only <b>@0, @4 and @8</b> — one per master,
each the account-0 key — come back.</div>

<h2 style="margin-top:14px">Host step 3 — the chunk set decodes back</h2>
<p>Three cards, reassembled, give the same 11-key policy back. <b>That is a
transcription check, not a restore test</b> — it proves the bytes survive the
card format and says nothing about whether the origins they carry are the ones
the keys actually live at. The restore test is the next page, and this wallet
fails it.</p>
{code(sect('4. the chunk set decodes back to the same 11-key policy'), 16)}
</div>

<div class="page">
<h2>Obstacle 1 — <code>mk</code> cannot derive the policy stub from a chunked md1</h2>
<p>Every mk1 key card must carry a 4-byte policy-id stub; <code>mk encode</code>
refuses without one. The only automatic way to get it is
<code>--from-md1</code>, and that path rejects a chunk:</p>
{code(sect('5. OBSTACLE 1 — mk cannot derive the stub from a CHUNKED md1'), 10)}
<p><code>mk</code> vendors <code>md-codec 0.34.0</code>; the primary crate is at
<code>0.42.0</code>. The "version 9" it will not parse is the chunked wire form.</p>

<h3>Working around it — and a spec/implementation divergence found on the way</h3>
<p>The stub is a property of the <em>descriptor</em>, not of the md1 string, so
it is well defined for a chunked policy; <code>md inspect</code> prints the
identities. But which one? <code>SPEC_mk_v0_1.md</code> §3.3 is explicit —
"the top 4 bytes of the MD-encoded policy's <b>WalletPolicyId</b>" — and today's
<code>md</code> prints a field by exactly that name. <b>That is not the one
<code>mk</code> uses.</b> Measured on a single-string wallet where
<code>--from-md1</code> does work:</p>
{code('''wallet-descriptor-template-id: 726a666305756435b7c52c5b3fc69c41
wallet-policy-id:              f05e8a1c282f7740bbfd902a759b5577
policy_id_stubs (what mk embedded):  726a6663''')}
<p>The stub tracks the <b>template-id</b>, not the <code>wallet-policy-id</code>.
Most likely a rename across versions — <code>mk</code> vendors md-codec 0.34.0
and the primary is 0.42.0, which now exposes both — but as it stands the spec
sentence and the binary disagree about which identity a key card indexes.</p>
{code(sect('9. the ids, and which one mk actually uses for the stub'), 10)}
<p>Following the binary rather than the sentence, this wallet's stub is
<b><code>5b48af35</code></b>, supplied with <code>--policy-id-stub</code>.</p>
</div>

<div class="page">
<h2>Host step 4 — the eleven key cards</h2>
{code(sect('7. the eleven key cards — ALL of them, each with its own origin'), 30)}
<p>Each key splits into 2 or 3 chunks — {MK1_CHUNKS} strings for eleven cards,
counted from the transcript's own output rather than assumed.
Note the decode: the card carries the origin fingerprint and path, so the
origins the descriptor card lacks are present in the bundle.</p>
</div>

<div class="page">
<h2>Host step 5 — the bundle validates, and names every plate</h2>
<p>Every set verifies, the manifest emits, and the operator gets the number that
matters before touching the machine.</p>
{code(re.sub(r'^me: rendered plate (?!1 ).*\n', '',
             sect('8. me bundle: validates, and prints the plate checklist'),
             flags=re.M).replace(
      'me: rendered plate 1', 'me: rendered plate 1 … (2–25 elided) …'), 40)}
</div>

<div class="page">
<h2>Host step 6 — the seed, and the refusal that still holds</h2>
<h2 style="margin-top:14px">The addresses — the check this wallet could never do</h2>
<p>Until the eleven keys were re-derived at BIP-48's four-level origin, <code>md</code>
refused every one of them (<code>expected depth 4 for this script context, got 3</code>)
and <b>no address could be derived for this wallet by any tool</b>. That is why no
earlier version of this document showed one. These are the structural half's
counterpart: proof the bytes mean the wallet that was intended.</p>
{code(sect('9b. THE ADDRESSES — the check this wallet could never do before'))}

<h2 style="margin-top:14px">What this backup will NOT do</h2>
<div class="note">
<p><b>Tiers 1–2 cannot be spent from these plates alone.</b> Both need the
32-byte preimage of the <code>sha256</code> hashlock — the "secret word". <b>No
plate carries it.</b> Record it separately, with the vault, or those two tiers
are decoration.</p>

<p><b>The tiers do not open in the order they read.</b> Tier 4 (1-of-3) matures
at ~365 days and tier 3 (2-of-2) at ~455 — the <i>weakest</i> key-set unlocks
~90 days <i>first</i>. And each <code>older()</code> clock runs per-coin from
that coin's confirmation, not from the day you engraved.</p>

<p><b>In master terms this is a 1-of-3 vault with delays, not an 11-key
multisig.</b> Three of its four tiers are satisfiable by a single master:</p>
<ul>
<li><b>A</b> alone — tier 1 (3-of-3 over @0–@2, all master A), unlocked since
block 1,000,000 in 2016 — but only with the secret word;</li>
<li><b>C</b> alone — tier 4 (1-of-3 over @8–@10), after ~365 days, <i>no</i>
word needed;</li>
<li><b>B</b> alone — tier 3 (2-of-2 over @6–@7), after ~455 days.</li>
</ul>
<p>Only tier 2 needs two masters, and it is the furthest out. So this vault
<i>survives losing any two</i> seed plates — and <i>falls to the theft of any
one</i>, given time. That is a deliberate trade if you chose it and a nasty
surprise if you did not.</p>
</div>

<h2 style="margin-top:14px">The restore test — what a card-only restore recovers</h2>
<p>The decode step is a transcription check. <b>This is the restore test</b>: it
derives, from each master seed, the key the <i>descriptor card's own declared
origin</i> would lead a restorer to, and compares it against the key that slot
actually holds.</p>
{code(sect('9c. THE RESTORE TEST — what a card-only restore actually recovers'), 26)}
<div class="note"><b>Three of eleven.</b> Only <code>@0</code>, <code>@4</code>
and <code>@8</code> — one account-0 key per master — come back. Every tier of
this vault needs signatures this restore cannot produce, so the descriptor card
alone is not a backup of this wallet: the mk1 key cards carry the origins that
make it one, and they are not optional.</div>

{code(sect('10. the seed, and the refusal that still holds'), 26)}
</div>
""")

# Caption each plate from WHAT IS ON IT, via two records neither of which
# this file guesses at:
#   out/pathological/manifest.json   — written by `me bundle`: plate -> string
#   out/pathological/card-index.txt  — written by the journey: string -> key
#
# The previous attempt walked per-key chunk COUNTS in key order. `me bundle`
# emits plates in chunk_set_id order, so every card caption named the wrong
# key — silently. Keying on the plate's own string removes the ordering
# assumption instead of trying to reproduce it.
_MANIFEST = os.path.join(OUT, "manifest.json")
_CARD_INDEX = os.path.join(OUT, "card-index.txt")
for _p in (_MANIFEST, _CARD_INDEX):
    if not os.path.exists(_p):
        raise SystemExit(f"{_p} missing — run transcript_pathological.sh first")

_MAN = json.load(open(_MANIFEST))["plates"]
_PLATE_KINDS = {int(e["plate"]): e["kind"] for e in _MAN}
_PLATE_STRING = {int(e["plate"]): e["string"] for e in _MAN if "string" in e}
_CARD_OWNER = {}
for _l in open(_CARD_INDEX).read().strip().split("\n"):
    if not _l.strip():
        continue
    _card, _ki, _nth, _tot, _fp, _path = _l.split()
    _CARD_OWNER[_card] = (int(_ki), int(_nth), int(_tot), _fp, _path)


def _caption(n):
    """Caption plate n from its own content, or fail loudly."""
    if n not in _PLATE_KINDS:
        raise SystemExit(f"manifest has no plate {n}")
    st = _PLATE_STRING.get(n)
    if st is None:
        # The ms1 seed plate carries no string in the manifest (kind "ms1",
        # integrity "n/a") — it is the operator's own seed card, not something
        # `me bundle` encoded. Caption it as what it is rather than crashing.
        return f"plate {n} — {_PLATE_KINDS[n]} (seed card — not machine-encoded)"
    if st.startswith("md1"):
        md1s = sorted(p for p, v in _PLATE_STRING.items() if v.startswith("md1"))
        return f"plate {n} — md1 policy, chunk {md1s.index(n) + 1}/{len(md1s)}"
    owner = _CARD_OWNER.get(st)
    if owner is None:
        raise SystemExit(
            f"plate {n} carries an mk1 string that card-index.txt does not "
            f"know — the bundle and the index are from different runs")
    ki, nth, tot, fp, path = owner
    return f"plate {n} — @{ki} [{fp}/{path}] chunk {nth}/{tot}"


plates = sorted((f for f in os.listdir(os.path.join(OUT, "plates")) if f.endswith(".png")),
                key=lambda s: int(re.search(r"(\d+)", s).group(1)))
cells = []
for f in plates:
    n = int(re.search(r"(\d+)", f).group(1))
    cap = _caption(n)
    cells.append(img(os.path.join(OUT, "plates", f), "plate", cap))

P.append(f"""
<div class="page">
<h2>The {PUBLIC_STRINGS} public plates</h2>
<p>{MD1_CHUNKS} descriptor chunks, then eleven key cards at two or three chunks each.</p>
<div class="grid3">{''.join(cells[:12])}</div>
</div>
<div class="page">
<h2>The {PUBLIC_STRINGS} public plates, continued</h2>
<div class="grid3">{''.join(cells[12:])}</div>
<div class="note"><b>Plus three seed plates that are not here.</b>
Each master is engraved from words typed on the machine. No host tool will
render, transmit or preview them — so the real total is <b>{TOTAL_PLATES} plates</b>.</div>
</div>
""")


def shot(n, cap):
    return img(os.path.join(SHOTS, n), "screen", cap)


P.append(f"""
<div class="page">
<h2>On the machine — the seed, typed</h2>
<p>No NFC in this journey. The seed goes in the way the machine's own checklist
says it must: by hand, on the device.</p>
<div class="grid3">
{shot('a00-boot.png','Boot: Backup Wallet')}
{shot('a01-input-seed.png','Input Seed — 12 words')}
{shot('a02-word-entry.png','Word 1 of 12')}
{shot('a03-typing-aba.png','Three letters is enough: “1 match”, auto-completed')}
{shot('a05-after-seed.png','All 12 accepted — the BIP-39 checksum holds')}
{shot('a06-after-seed-confirm.png','Optional BIP-39 passphrase, skipped')}
</div>
</div>

<div class="page">
<h2>On the machine — the seed plate</h2>
<div class="grid3">
{shot('a07-after-passphrase.png','Insert a blank plate; hold to start')}
{shot('b1-screen.png','Cutting')}
{shot('b6-screen.png','Still cutting — a seed plate is the long one')}
</div>

<h2 style="margin-top:16px">The plate overlay, live</h2>
<p>The whole planned layout is drawn in <b>grey</b> the moment the engrave
begins, from the same <code>PlanEngraving</code> call the firmware makes. What
the driver has actually stepped is filled in <b>black</b>, decoded from the real
step stream. The red circle is the head. The title is the master fingerprint
<code>73C5DA0A</code> — the same one the descriptor's origins name.</p>
<div class="grid2">
{img(os.path.join(SHOTS,'b0-plate.png'),'plate',plate_caption('b0-plate.png'), 0.82)}
{img(os.path.join(SHOTS,'b3-plate.png'),'plate',plate_caption('b3-plate.png'), 0.82)}
{img(os.path.join(SHOTS,'b6-plate.png'),'plate',plate_caption('b6-plate.png'), 0.82)}
{img(os.path.join(SHOTS,'b8-plate.png'),'plate',plate_caption('b8-plate.png'), 0.82)}
</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>What this run turned up</h2>

<div class="warn"><b>1. <code>mk encode --from-md1</code> cannot read a chunked md1.</b>
<code>wire-format version mismatch: got 9, expected 4</code>, exit 2 — and a stub
is mandatory, so for any policy large enough to need chunking the documented
route to a key card does not work. <code>mk</code> vendors
<code>md-codec 0.34.0</code> against the primary's <code>0.42.0</code>; version 9
is the chunked wire form its copy predates. The provenance pin is what should
have caught this. Workaround: read the identity out of <code>md inspect</code>
and pass <code>--policy-id-stub</code> by hand — which nothing tells an operator
to do, and which requires knowing the next finding.</div>

<div class="warn"><b>2. The spec and the binary disagree about which identity the
stub indexes.</b> <code>SPEC_mk_v0_1.md</code> §3.3 says the stub is the top 4
bytes of the <b>WalletPolicyId</b>, and <code>md</code> prints a field of exactly
that name. Measured, <code>mk</code> embeds the top 4 bytes of the
<b>wallet-descriptor-template-id</b> instead — <code>726a6663</code> where the
<code>wallet-policy-id</code> began <code>f05e8a1c</code>. Probably a rename that
landed in md-codec after 0.34.0, but the sentence and the code now name different
things, and a stub is the index a recovering operator uses to tell wallets apart.</div>

<div class="note"><b>3. <code>--path</code> is required here, and it flattens.</b>
Without it the descriptor card carries no origin, <code>md decode</code>
PARTIAL-decodes (exit 4) and <code>me bundle</code> refuses the set. With it,
everything validates — but it records a single shared origin while these eleven
keys sit at four account indices. The true paths survive on the mk1 cards; the
descriptor card alone restores only <b>3 of 11</b> slots correctly — @0, @4 and
@8, one account-0 key per master. Measured by the restore test above, which also
corrected this sentence's earlier claim of "@3–@10". Which source wins is a
<em>design</em> question rather than a defect, and it is now pinned rather than
argued.</div>

<div class="note"><b>4. <code>md encode</code> has no per-key origin flag.</b>
<code>--key</code> takes a bare xpub and rejects an origin-annotated one
(<code>base58check decode</code>), and <code>--fingerprint</code> carries no path.
So a divergent-origin wallet cannot state its origins in the descriptor card at
all — only the flattened form of finding 3 is expressible.</div>

<div class="note"><b>5. Carried over from the first journey:</b> presenting an
NFC tag to a gathering flow freezes the emulator (filed as <b>F-126</b>). This
journey avoids NFC entirely, so it is not exercised here.</div>

<div class="note"><b>Corrected from the first draft of this document.</b> It
reported <code>me bundle</code>'s refusal as a tool defect and rendered the
plates by calling the sidecar directly. That was wrong: the refusal was the
consequence of omitting <code>--path</code>, which <code>md</code> had warned
about at encode time. The plates and checklist here come from
<code>me bundle --preview</code>.</div>

<h2 style="margin-top:16px">Reproducing this document</h2>
{code('''cd <this directory>
bash transcript_pathological.sh > transcript_pathological.txt 2>&1   # every CLI block, refusals included
python3 capture_pathological.py                                       # the 13 screenshots, from the emulator
python3 build_pdf_pathological.py                                     # this document, as a PDF''')}
<div class="foot">Emulator frames were captured by POSTing
<code>canvas.toDataURL()</code> to a local receiver, so each screenshot is the
device framebuffer exactly. Plate overlays are the page's own SVG, rasterised in
the browser that drew them. Both are produced by
<code>capture_pathological.py</code>, which drives
<code>cmd/emu/shots_pathological.js</code> — the capture used to be console code
in a session that no longer exists, which is why this document could not be
rebuilt (F-210).</div>
</div>
""")

doc = ("<!doctype html><meta charset=utf-8><title>SeedHammer II — the pathological wallet</title>"
       f"<style>{CSS}</style>" + "".join(P))
# Was "journey.html" — the SAME filename build_pdf.py writes, so whichever
# builder ran last silently clobbered the other's output and only one of the
# two journeys could exist at a time. Named per journey now, matching
# build_pdf_payload.py's journey_payload.html convention.
p = os.path.join(OUT, "journey_pathological.html")
open(p, "w").write(doc)
print(f"wrote {p} ({len(doc)//1024} KB)")

# GATED BEFORE THE PRINT, NOT AFTER. The PDF path is the committed deliverable,
# so printing first and checking second OVERWRITES the good document with an
# incomplete rebuild and only then reports failure -- which is how the first run
# of this step replaced the published PDF with one whose plate captions said
# "no caption captured for this plate". The gate exits non-zero, so reaching the
# print at all means the HTML referenced every asset it names.
missing_gate(p)

# THE PDF STEP (half of F-156). This builder used to stop at HTML, and the
# published PDF came from a manual headless-Chrome pass that lived nowhere in
# the repo -- so the deliverable was committed and the command that made it was
# not. Copied from build_pdf_payload.py, which already did it right.
PDF = os.path.join(W, "SeedHammer-II-pathological-wallet-journey.pdf")
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + p],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
missing_gate(PDF)

