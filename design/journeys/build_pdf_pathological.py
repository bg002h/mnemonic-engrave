#!/usr/bin/env python3
"""Assembles the pathological-wallet operator journey.

Same rule as the first document: every CLI block is transcript.sh's real
stdout+stderr, every screenshot is the emulator's own framebuffer, every plate
is a real render. The refusals are reported with their actual exit codes rather
than edited out -- three of them are the most useful thing here.
"""

import base64, html, json, os, re, sys

W = os.path.dirname(os.path.abspath(__file__))
OUT, SHOTS = os.path.join(W, "out"), os.path.join(W, "shots")


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(path, cls="", cap=None, crop=None):
    if not os.path.exists(path):
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


transcript = open(os.path.join(OUT, "transcript.txt")).read()
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
(<code>84'/0'/0..3'</code>), so no key is reused.</p>

<div class="note"><b>This is the case that actually forces chunking.</b>
The 12-key <code>multi(5,…)</code> policy tried first encodes to 13 bytes and one
md1 string — a keyless BIP-388 template does not pay per key. This one carries
two 32-byte hash literals and four timelock arguments, and comes to
<b>182 data symbols against a single-string cap of 80</b>. It is the first
policy in this work that genuinely needs a chunk set.</div>

<h2>Toolchain</h2>
{code(S.get('versions',''))}
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
{code(S.get('1. the policy — 11 keys, four timelock kinds, a hashlock',''), 8)}
{code(S.get('2. it does not fit one string',''), 8)}

<h2 style="margin-top:14px">Host step 2 — so it is chunked, with an explicit origin</h2>
<p>Encoded without <code>--path</code>, this policy's card carries no origin and
<code>md</code> says so: its <code>wsh(or_i(…))</code> wrapper has no canonical
default derivation path, so <code>md decode</code> only PARTIAL-decodes (exit 4,
VERIFY-ME) and <code>me bundle</code> rejects the set outright. Supplying the
origin the warning asks for is what makes the rest of the chain work.</p>
{code(S.get('3. so it is chunked -- WITH the origin the warning asked for',''), 14)}
<div class="note"><b>What <code>--path</code> does and does not say.</b>
It records ONE shared origin — <code>bip84</code> and <code>m/84'/0'/0'</code>
produce a byte-identical chunk set here. These eleven keys actually sit at four
account indices (<code>84'/0'/0..3'</code>) across three masters, and each mk1
card carries its own true <code>origin_path</code>. So the md1's shared value is
a default the key cards override; a restore that trusted the descriptor card
alone would derive the wrong keys for @3–@10. Worth pinning with a restore test
before this shape is recommended to anyone.</div>

<h2 style="margin-top:14px">Host step 3 — the chunk set decodes back</h2>
<p>Three cards, reassembled, give the same 11-key policy back — the round trip
that makes the backup worth engraving.</p>
{code(S.get('4. the chunk set decodes back to the same 11-key policy',''), 16)}
</div>

<div class="page">
<h2>Obstacle 1 — <code>mk</code> cannot derive the policy stub from a chunked md1</h2>
<p>Every mk1 key card must carry a 4-byte policy-id stub; <code>mk encode</code>
refuses without one. The only automatic way to get it is
<code>--from-md1</code>, and that path rejects a chunk:</p>
{code(S.get('5. OBSTACLE 1 — mk cannot derive the stub from a CHUNKED md1',''), 10)}
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
{code(S.get('9. the ids, and which one mk actually uses for the stub',''), 10)}
<p>Following the binary rather than the sentence, this wallet's stub is
<b><code>5b48af35</code></b>, supplied with <code>--policy-id-stub</code>.</p>
</div>

<div class="page">
<h2>Host step 4 — the eleven key cards</h2>
{code(S.get('7. the eleven key cards',''), 30)}
<p>Each key splits into <b>2 chunks</b>, so the eleven cards are 22 strings.
Note the decode: the card carries the origin fingerprint and path, so the
origins the descriptor card lacks are present in the bundle.</p>
</div>

<div class="page">
<h2>Host step 5 — the bundle validates, and names every plate</h2>
<p>Every set verifies, the manifest emits, and the operator gets the number that
matters before touching the machine.</p>
{code(re.sub(r'^me: rendered plate (?!1 ).*\n', '',
             S.get('8. me bundle: validates, and prints the plate checklist',''),
             flags=re.M).replace(
      'me: rendered plate 1', 'me: rendered plate 1 … (2–25 elided) …'), 40)}
</div>

<div class="page">
<h2>Host step 6 — the seed, and the refusal that still holds</h2>
{code(S.get('10. the seed, and the refusal that still holds',''), 26)}
</div>
""")

plates = sorted((f for f in os.listdir(os.path.join(OUT, "plates")) if f.endswith(".png")),
                key=lambda s: int(re.search(r"(\d+)", s).group(1)))
cells = []
for f in plates:
    n = int(re.search(r"(\d+)", f).group(1))
    if n <= 3:
        cap = f"plate {n} — md1 policy, chunk {n}/3"
    else:
        ki, ch = (n - 4) // 2, (n - 4) % 2 + 1
        k = keys[ki]
        cap = f"plate {n} — @{ki} [{k['fp']}/{k['path'][2:]}] chunk {ch}/2"
    cells.append(img(os.path.join(OUT, "plates", f), "plate", cap))

P.append(f"""
<div class="page">
<h2>The 25 public plates</h2>
<p>Three descriptor chunks, then eleven key cards at two chunks each.</p>
<div class="grid3">{''.join(cells[:12])}</div>
</div>
<div class="page">
<h2>The 25 public plates, continued</h2>
<div class="grid3">{''.join(cells[12:])}</div>
<div class="note"><b>Plus three seed plates that are not here.</b>
Each master is engraved from words typed on the machine. No host tool will
render, transmit or preview them — so the real total is <b>28 plates</b>.</div>
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
{img(os.path.join(SHOTS,'b0-plate.png'),'plate','head 35.7, 13.0 mm — mid-fingerprint, all 12 words still pending', 0.82)}
{img(os.path.join(SHOTS,'b3-plate.png'),'plate','head 23.2, 20.6 mm — into word 1', 0.82)}
{img(os.path.join(SHOTS,'b6-plate.png'),'plate','head 35.9, 28.2 mm — three words down', 0.82)}
{img(os.path.join(SHOTS,'b8-plate.png'),'plate','head 36.9, 33.4 mm — four of twelve', 0.82)}
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
descriptor card alone would restore @3–@10 wrongly. This is the one finding here
that is a <em>design</em> question rather than a defect: a restore test should
pin which source wins.</div>

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
bash transcript.sh > out/transcript.txt 2>&1   # every CLI block, refusals included
python3 build_pdf.py                            # this PDF''')}
<div class="foot">Emulator frames were captured by POSTing
<code>canvas.toDataURL()</code> to a local receiver, so each screenshot is the
device framebuffer exactly. Plate overlays are the page's own SVG, rendered with
<code>rsvg-convert</code>.</div>
</div>
""")

doc = ("<!doctype html><meta charset=utf-8><title>SeedHammer II — the pathological wallet</title>"
       f"<style>{CSS}</style>" + "".join(P))
p = os.path.join(OUT, "journey.html")
open(p, "w").write(doc)
print(f"wrote {p} ({len(doc)//1024} KB)")
