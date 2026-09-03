#!/usr/bin/env python3
"""Assembles the composer journey: a wallet policy BUILT ON THE DEVICE and proved.

Same rule as the sibling builders: every CLI block is transcript_composer.sh's
real stdout+stderr with its actual exit code, and every screenshot is the
emulator's own framebuffer, captured by capture_composer.py. The document's
central claims are CHECKED BY THE CAPTURE, not by this script: the walk is
handed the host's digest, ids, stubs, addresses and engraved strings and
throws if the device disagrees, so these pages cannot show agreement that did
not happen.

    python3 build_pdf_composer.py [--allow-missing]
"""

import base64, html, json, os, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out")
COMP = os.path.join(OUT, "composer")

# ── Missing-asset gate (identical to the sibling builders; enforcement is the
# DEFAULT, --allow-missing is the explicit opt-OUT for a draft) ───────────────
MISSING = []
ALLOW_MISSING = "--allow-missing" in sys.argv


def missing_gate(artifact):
    if not MISSING:
        return
    uniq = sorted(set(MISSING))
    print(f"\n{len(MISSING)} missing asset(s), {len(uniq)} distinct -- "
          f"{artifact} is INCOMPLETE:", file=sys.stderr)
    for name in uniq:
        print(f"  missing: {name}", file=sys.stderr)
    if ALLOW_MISSING:
        print("--allow-missing given: exiting 0 with an incomplete document.", file=sys.stderr)
        return
    print("Refusing to report success. Run capture_composer.py --arm both, or pass "
          "--allow-missing for a draft.", file=sys.stderr)
    sys.exit(1)


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="shot", cap=None):
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path) or os.path.getsize(path) < 512:
        MISSING.append(name)
        return f'<p class="missing">missing: {html.escape(name)}</p>'
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(path)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def code(text, limit=None):
    lines = text.rstrip("\n").split("\n")
    clip = limit and len(lines) > limit
    if clip:
        lines = lines[:limit]
    body = html.escape("\n".join(lines))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in transcript_composer.txt</span>"
    return f"<pre>{body}</pre>"


def readfile(p):
    if not os.path.exists(p):
        MISSING.append(os.path.relpath(p, W))
        return ""
    with open(p) as f:
        return f.read()


def section(transcript, header):
    """The transcript block under `########## <header>`, sliced by the
    transcript's own headers rather than retyped."""
    marks = [i for i, l in enumerate(transcript) if l.startswith("########## ")]
    for n, i in enumerate(marks):
        if header in transcript[i]:
            end = marks[n + 1] if n + 1 < len(marks) else len(transcript)
            return "\n".join(transcript[i + 1:end]).strip("\n")
    MISSING.append(f"transcript section {header!r}")
    return ""


def kv(pairs):
    rows = "".join(f"<div><span>{html.escape(k)}</span><code>{html.escape(v)}</code></div>" for k, v in pairs)
    return f'<div class="kv">{rows}</div>'


CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]
tx = readfile(os.path.join(W, "transcript_composer.txt")).split("\n")
result_path = os.path.join(SHOTS, "composer-result.json")
result = json.load(open(result_path)) if os.path.exists(result_path) else MISSING.append(
    "composer-result.json") or {}
legA, legB, legK = result.get("keyed-A", {}), result.get("keyed-B", {}), result.get("keyless", {})
keyed_template = readfile(os.path.join(COMP, "keyed.template")).strip()
keyed_ids = readfile(os.path.join(COMP, "keyed.id.txt"))
recv = readfile(os.path.join(COMP, "keyed.receive.txt")).split()
chg = readfile(os.path.join(COMP, "keyed.change.txt")).split()
keyed_md1 = [l for l in readfile(os.path.join(COMP, "keyed.md1.txt")).split("\n") if l.strip()]
keyless_template = readfile(os.path.join(COMP, "keyless-tr.template")).strip()
keyless_md1 = readfile(os.path.join(COMP, "keyless-tr.md1.txt")).strip()
digest = readfile(os.path.join(COMP, "payload.digest.txt")).strip()
records = readfile(os.path.join(COMP, "records.txt"))


def shots_matching(prefix):
    return [n for n in sorted(os.listdir(SHOTS)) if n.startswith(prefix)] if os.path.isdir(SHOTS) else []


P = []
P.append(f"""
<section class="page">
<h1>SeedHammer II — the composer journey</h1>
<p class="sub">A wallet policy <em>built on the device</em>, from the operator's own keys, and proved before it reaches steel.</p>
<p>Every other journey in this repo hands the device a wallet somebody built
elsewhere. This one builds it <em>on</em> the SeedHammer II: the operator picks a
script type, lists spend paths (keys, a wait, a hashlock), seats keys from a
systemwide payload, and the device lowers the list to a BIP-388 template by the
same fixed rules <code>md compose</code> uses on the host. The point of the walk
is that <strong>three implementations meet on the same bytes</strong>: the host
(<code>md</code>, <code>me</code>, <code>ms</code>, <code>mk</code>), the firmware
on the emulator, and — for the key-less arm — the string on the one plate the
operator cuts.</p>
<h2>The wallet</h2>
<p>Segwit (wsh). Path 1: 2-of-2. Path 2: one key, after a wait of 12960 blocks,
with a sha256 hashlock. Lowered, that is:</p>
<pre>{html.escape(keyed_template)}</pre>
<p class="note"><strong>Nothing secret is committed.</strong> Master A is BIP-39's
"abandon … about" vector and master B its "legal winner … yellow" vector.
<strong>Path 1's two keys are two ACCOUNTS of master A, deliberately</strong>: it is
the "one master, two accounts" case the door labels for, and it makes the
mapping review fire its <em>same seed, same path</em> warning — a §8 body no
other fixture reaches, photographed on page 4. Do not read that warning as a
defect, and never put funds behind these keys.</p>
<h2>The payload</h2>
<p>Five records, packed with <code>me sysw pack</code>: two <code>key:</code>
records (A's accounts 0 and 1), a <code>hash:</code> record, a <code>now:</code>
record (the pack time, a lower bound the device echoes beside a time lock), and
master B's words as a seed.</p>
{code(records)}
{kv([("payload digest", digest)])}
</section>
""")
P.append(f"""
<section class="page">
<h2>1 · The host mints the payload…</h2>
{code(section(tx, "2. the composer payload's records"), 20)}
<p class="note">The fork's own Go generator (<code>cmd/buildpayloadcomposer</code>)
emits the same five records byte for byte, and the transcript gates on it:</p>
{code(section(tx, "2a. GATE"), 12)}
<h2>2 · …and, separately, what the device must prove to</h2>
<p>The ids come from <code>md inspect</code> of the strings the device will
engrave, the addresses from <code>md address</code> — a different implementation
in a different language from the firmware's.</p>
{code(section(tx, "4b. what the device must prove to"), 34)}
{kv([("receive 0..1", chr(10).join(recv[:2])), ("change 0..1", chr(10).join(chg[:2]))])}
</section>
""")
P.append(f"""
<section class="page">
<h2>3 · The device: loading, the door, the shape</h2>
<div class="row">
{img("c01-payload-digest.png", cap="The payload's digest on screen. The walk compares it with `me sysw show`'s before going on.")}
{img("c02-door.png", cap="Wallet Policy's door states the key state: two keys and a seed are loaded. There is no 'From payload' row -- the payload holds no finished wallet.")}
</div>
<div class="row">
{img("c03-start-from.png", cap="'Start from?' opens on 'Build my own paths' (the S4 walk's W-1), then the six presets.")}
{img("c04-lock-echo-p0.png", cap="Path 2's wait, echoed in the operator's units. A relative lock carries no pack-time bound line.")}
</div>
<div class="row">
{img("c05-hash-rule.png", cap="Choosing the payload's hashlock first shows the hash rule: a preimage must be a 32-byte value hashed once more; a hashed passphrase can never be spent.")}
{img("c06-stub-p0.png", cap="The template screen: the key-independent Template-ID, the mk1 stub every key card must carry, and the mk encode line to mint one. As captured here the Template-ID's last hex digit sits under the Back button -- the S4 walk's W-3, fixed on the fork after this run.")}
</div>
</section>
""")
P.append(f"""
<section class="page">
<h2>4 · Seating, and the mapping review</h2>
<div class="row">
{img("c07-seat-slot0.png", cap="Slot @0 offers the payload's two key records by fingerprint and origin, plus 'Type a seed' and 'Leave unseated'.")}
{img("c08-seat-slot2-seed.png", cap="Slot @2 is filled from the payload's seed (master B), offered as 'seed 1 (any slots)' after 'Type a seed' -> 'FROM PAYLOAD'.")}
</div>
<div class="row">
{img("c09-mapping-p0.png", cap="The mapping review: every slot's fingerprint and origin, printed verbatim, with the line that the device cannot confirm a key was derived there.")}
{img("c09-mapping-p1.png", cap="Page 2: 'same seed, same path' -- Path 1's 2-of-2 is two accounts of one master and can be satisfied by one person. Deliberate here; see page 1.")}
</div>
<p>The device seats the seed's slot at <strong>master B's own account 0'</strong>
(<code>m/48'/0'/0'/2'</code>), not at the lowest account free in the template. The
host oracle was minted with that origin, and the Policy-ID on the next page is
the proof that the two rules agree.</p>
</section>
""")
consentA = shots_matching("c11-consent-p")
stub2 = shots_matching("c10-stub2-p")
mA = legA.get("matched", {})
P.append(f"""
<section class="page">
<h2>5 · The proof, on the consent path</h2>
<div class="row">
{"".join(img(n, cap=f"template screen after seating, page {i + 1}: both ids and both stubs") for i, n in enumerate(stub2[:2]))}
</div>
<div class="row">
{"".join(img(n, cap=f"consent, page {i + 1}") for i, n in enumerate(consentA))}
</div>
{kv([("Template-ID", str(mA.get("templateId", "—"))), ("mk1 stub (template)", str(mA.get("templateStub", "—"))),
     ("Policy-ID", str(mA.get("policyId", "—"))), ("mk1 stub (policy)", str(mA.get("policyStub", "—"))),
     ("addresses matched", chr(10).join(mA.get("addresses", [])))])}
<p>Both ids are <strong>named</strong> on the screen (the key-stable Template-ID
and the key-dependent Policy-ID look identical as 16 bytes of hex), and the four
addresses are lines the operator must page through to engrave.</p>
</section>
""")
P.append(f"""
<section class="page">
<h2>6 · Two forms, two censuses, and what was cut</h2>
<div class="row">
{img("c12-census-A.png", cap=f"Form A, the policy itself: {legA.get('censusClaim', '—')} plates -- the {len(keyed_md1)} md1 chunks pack onto them.")}
{img("c12-census-B.png", cap=f"Form B, template plus key cards: {legB.get('censusClaim', '—')} plates -- one key-less template plate and one mk1 card per seated slot.")}
</div>
<p>Every plate the emulator cut was read back from its toolpath and compared
<strong>byte for byte</strong> with the host's strings: form A's {len(legA.get('engraved', []))} entries
against <code>keyed.md1.txt</code>, form B's {len(legB.get('engraved', []))} against the fingerprinted
template and the three host-minted cards (both stubs on each). Form A, as the host minted it:</p>
{code(chr(10).join(keyed_md1))}
<p class="note">Both runs take <em>Watch-only</em>. 'Full (seed + keys)' would add a
bearer plate of master B's seed, which an automated run must not cut.</p>
{img("c13-A-bundle-engraved.png", cap="After the last plate: the bundle-engraved note, then the door again. (The ms1 reminder shows even when no share was cut -- F-463.)")}
</section>
""")
mK = legK.get("matched", {})
P.append(f"""
<section class="page">
<h2>7 · The key-less arm — and the plate the operator cuts</h2>
<p>With no payload loaded the door says so, and the composer builds a
<em>key-less template</em>: Taproot, one 2-of-3 path, every slot given a
distinct-account origin for a key to be minted later.</p>
<pre>{html.escape(keyless_template)}</pre>
<div class="row">
{img("k01-door.png", cap="'No keys loaded. This builds a key-less template.'")}
{img("k02-stub-p1.png", cap="Each unseated slot 'expects a key at' its own account: 0', 1', 2' under 48'/0'/…/3'.")}
</div>
<div class="row">
{img("k03-consent-p1.png", cap="Consent for a template: no addresses, verify off-device.")}
{img("k04-census.png", cap="One plate.")}
</div>
{kv([("Template-ID", str(mK.get("templateId", "—"))), ("the plate", keyless_md1), ("characters", str(len(keyless_md1)))])}
<p class="note"><strong>Why the string is checked byte for byte and not merely
verified.</strong> The device is chunk-form-always; a template this short encodes
UNCHUNKED on the host by default, and <code>md verify</code>, <code>md decode</code>
and <code>md inspect</code> accept both forms identically. The first draft of this
journey's plan pinned the 47-character form; only the byte comparison could have
told it from the 56-character plate. The transcript mints the oracle with
<code>--force-chunked</code> and demonstrates the substitution:</p>
{code(section(tx, "6. THE KEYLESS ARM"), 30)}
</section>
""")
legs = [(k, result[k]) for k in ("keyed-A", "keyed-B", "keyless") if k in result]
rows = "".join(
    f"<div><span>{html.escape(k)}</span><code>{len(v.get('shots', []))} shots · census {v.get('censusClaim', '—')} plate(s) · "
    f"{len(v.get('engraved', []))} engraved entr{'y' if len(v.get('engraved', [])) == 1 else 'ies'} · {v.get('elapsedSec', '—')} s</code></div>"
    for k, v in legs)
P.append(f"""
<section class="page">
<h2>8 · What this run actually established</h2>
<p>The capture is a <strong>comparison, not a photo shoot</strong>.
<code>shots_composer.js</code> is handed the host's digest, ids, stubs, addresses
and engraved strings and throws if the device disagrees; the capture exits
non-zero unless every leg matched and every shot arrived.</p>
<div class="kv">{rows}</div>
<p>Rows are selected by <em>tapping</em> them where the layout draws them
(<code>shTargets()</code> reads the frame's hit regions; the walk injects no
button event the machine lacks). That is not a nicety: the first attempt at
this walk found that the composer's pick lists had no touch targets at all, so
on the device only a page's first row could be taken (the S4 walk's W-2, fixed
before this run).</p>
<h2>The controls</h2>
<ul>
<li><code>capture_composer.py --arm keyed --prove-it-can-fail</code> corrupts one
character of one expected address and exits 0 only if the walk's failure
<em>names that address</em>; a walk that fails earlier is reported
INCONCLUSIVE.</li>
<li>Substituting the unchunked key-less string for the expected one makes the
key-less arm fail naming 56 versus 47 characters.</li>
</ul>
<h2>Reproducing</h2>
<pre>bash transcript_composer.sh &gt; transcript_composer.txt 2&gt;&amp;1   # the host half, 27 gates
python3 capture_composer.py --arm both       # rebuilds emu.wasm, drives three legs, writes shots/
python3 capture_composer.py --arm keyed --prove-it-can-fail
python3 build_pdf_composer.py                # this document</pre>
<h2>What this journey does NOT show</h2>
<ul>
<li><strong>No real plate.</strong> The emulator's toolpath is read back; the
physical key-less plate is the S4 device walk's, cut and read aloud at the
machine.</li>
<li><strong>No Full-mode secret plate</strong>, no NFC seating, no on-device
preimage derivation (spec §14).</li>
</ul>
</section>
""")

doc = ("<!doctype html><meta charset='utf-8'>"
       "<title>SeedHammer II — the composer journey</title>"
       f"<style>{CSS}</style>" + "".join(P))
p = os.path.join(OUT, "composer-journey.html")
os.makedirs(OUT, exist_ok=True)
with open(p, "w") as f:
    f.write(doc)
print(f"wrote {p} ({os.path.getsize(p)//1024} KB)")

missing_gate(p)

PDF = os.path.join(W, "SeedHammer-II-composer-journey.pdf")
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
