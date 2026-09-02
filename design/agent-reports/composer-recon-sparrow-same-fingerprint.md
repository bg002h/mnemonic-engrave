# Recon: Sparrow/drongo behavior on a wallet with two cosigner keystores sharing one master fingerprint

**Question.** In Sparrow Wallet (`sparrowwallet/sparrow`) and its library `sparrowwallet/drongo`, is a
multisig wallet ACCEPTED, REFUSED, or WARNED when two cosigner keystores share the SAME master
fingerprint (one person's two keys from one seed, different derivation paths / different xpubs)?

**Method.** Cloned both repos at HEAD (`--depth 1`), read source directly, and — because JDK 26 broke
the pinned Gradle 9.1.0 Groovy toolchain (`Unsupported class file major version 70`) — bypassed Gradle
entirely: stripped `module-info.java`, resolved the ~8 runtime deps (bouncycastle, slf4j, dnsjava,
caffeine, pgpainless, argon2-jvm-nolibs, jna) directly from Maven Central, and compiled/ran real Java
programs against drongo's actual `OutputDescriptor` / `Wallet` / `Keystore` classes. Where marked
MEASURED, this is a real JVM run against the cloned HEAD, not a reading of the code.

- drongo clone HEAD: `d66694369f492fc79827758987c5a1c4968f1a9d` (2026-08-31 11:44:49 +0200)
- sparrow clone HEAD: `aeeeeb54048400f6345ba33c5411a79aa253f814` (2026-08-31 14:37:20 +0200)
- Repo URLs: `https://github.com/sparrowwallet/drongo`, `https://github.com/sparrowwallet/sparrow`

## Answer

**ACCEPTED. No refusal, no warning, anywhere in the parse → wallet-build → validity-check → UI-gate
pipeline.** MEASURED: a real 2-of-3 `wsh(sortedmulti(...))` wallet with cosigner A and cosigner C
sharing one master fingerprint (different derivation paths, different xpubs, both genuinely derived
from the same BIP32 seed) parses to 3 distinct keystores, `Wallet.isValid()` returns `true`, and
`Wallet.checkWallet()` throws nothing. Sparrow's own "Apply" button in wallet settings is gated purely
on `wallet.isValid()` (see below), so this wallet would be accepted into the UI with zero indication
anything is unusual.

## 1. `Keystore.java` — identity/comparison

SOURCED: `/tmp/.../drongo/src/main/java/com/sparrowwallet/drongo/wallet/Keystore.java` (555 lines, read
in full).

- `Keystore` declares **no `equals()`/`hashCode()` override**. It extends `Persistable`
  (`src/main/java/com/sparrowwallet/drongo/wallet/Persistable.java`), which also declares neither —
  confirmed by grep (`grep -n -i "equals\|hashCode\|class Persistable"` → only the class declaration
  line matched). So `Keystore` uses default reference identity.
- `getKeyDerivation()` (line 86-88) returns a `KeyDerivation` object holding `masterFingerprint` +
  `derivationPath` (see §KeyDerivation below).
- `checkKeystore()` (lines 359-441) validates label, source, walletModel, keyDerivation presence,
  xpub/silent-payment-address presence, label length, path validity, and fingerprint **format**
  (8 hex chars, line 392-394) — but never compares this keystore's fingerprint against any other
  keystore's. It is a per-keystore self-check only.

## 2. `KeyDerivation.java` — equals/hashCode basis

SOURCED: `/tmp/.../drongo/src/main/java/com/sparrowwallet/drongo/KeyDerivation.java`, lines 136-155.

```java
public String toString() {
    return masterFingerprint + (derivationPath != null ? derivationPath.replace("m", "") : "");
}

@Override
public boolean equals(Object o) {
    if(this == o) { return true; }
    if(o == null || getClass() != o.getClass()) { return false; }
    KeyDerivation that = (KeyDerivation)o;
    return that.toString().equals(this.toString());
}

@Override
public int hashCode() {
    return toString().hashCode();
}
```

`equals`/`hashCode` are keyed on `masterFingerprint + derivationPath` **together**, never on
fingerprint alone. So even if some collection *were* keyed on `KeyDerivation`, two keystores sharing a
fingerprint but differing in path would NOT collide there either. (No such collection exists — see
§4.)

## 3. `Wallet.java` — keystores collection + validity check

SOURCED: `/tmp/.../drongo/src/main/java/com/sparrowwallet/drongo/wallet/Wallet.java` (2510 lines).

**Collection type — a `List`, not a `Set`:**

```
44:    private List<Keystore> keystores = new ArrayList<>();
```

`OutputDescriptor.toWallet()` (see §4) calls `wallet.getKeystores().add(keystore)` per parsed cosigner
— an `ArrayList.add`, so duplicates by any key can never be silently collapsed by container semantics.

**`checkWallet()` (lines 2184-2255, read in full — this is the entire method body, it ends at 2255):**

The full sequence of checks: null policyType/scriptType/defaultPolicy, empty keystores, script-type
vs. policy-type compatibility, signature-count vs. keystore-count arithmetic, max-cosigners, per-policy
xpub/scan-address presence, `containsDuplicateKeystoreLabels()` (line 2232), per-keystore
`checkKeystore()` + cross-script-type / cross-network derivation-path collision checks (lines
2236-2249), and finally:

```java
2252:        if(containsDuplicateExtendedKeys()) {
2253:            throw new InvalidWalletException("Wallet keystores have duplicate extended public keys");
2254:        }
```

That is the **last** statement in `checkWallet()`. **There is no fingerprint-duplicate check anywhere
in this method, or anywhere else in the class** — confirmed by
`grep -rn -i "duplicate.*fingerprint\|fingerprint.*duplicate\|sameFingerprint\|containsDuplicateFingerprint" --include="*.java" .`
across the whole drongo tree: zero hits (one unrelated comment in `StonewallUtxoSelector.java` about
"the same seed" for deterministic UTXO selection, not fingerprint dedup).

```java
2286:    public boolean containsDuplicateExtendedKeys() {
2287:        if(keystores.size() <= 1) { return false; }
2288:        return !keystores.stream().map(Keystore::getExtendedPublicKey).allMatch(new HashSet<>()::add);
2289:    }
```

This compares `Keystore::getExtendedPublicKey` (the `ExtendedKey`, i.e. the xpub object) via a
`HashSet`, which relies on `ExtendedKey.equals()`/`hashCode()` — SOURCED
(`ExtendedKey.java` lines 140-153) to be based on `toString()` (the base58 xpub string). **Two
different xpubs (different derivation paths under the same seed) are different strings and never
collide here**, even though they share a master fingerprint. It only catches the literal-duplicate-xpub
case, not the shared-fingerprint case.

`isValid()` (lines 2174-2181) is a try/catch wrapper around `checkWallet()`.

## 4. `OutputDescriptor.java` — descriptor → keystores mapping

SOURCED: `/tmp/.../drongo/src/main/java/com/sparrowwallet/drongo/OutputDescriptor.java` (1098 lines).

Key/derivation storage is a `Map<ExtendedKey, KeyDerivation>` — keyed on the **xpub object**, not on
fingerprint:

```
45:    private final Map<ExtendedKey, KeyDerivation> extendedPublicKeys;
```

Parsing (`getOutputDescriptorImpl`, lines 561-624) builds this map via:

```
605:                keyDerivationMap.put(extendedKey, keyDerivation);
```

`extendedKey` here is the parsed xpub (`ExtendedKey.fromDescriptor(extKey)`), not the fingerprint from
the `[fp/path]` origin. Two cosigners with the same fingerprint but different xpubs produce two
distinct map keys and are never overwritten/collapsed.

`toWallet()` (lines 321-364) then iterates this map and does, per entry:

```
356:            wallet.getKeystores().add(keystore);
```

— one `Keystore` object per distinct xpub entry, appended to the `List`.

### MEASURED: end-to-end run against the exact scenario in the question

Built three real xpubs with drongo's own `HDKeyDerivation`/`ExtendedKey` API (not hand-typed vectors):
seed A → `xpub...NGcwAP1kg` at `m/48'/0'/1'/2'` (fingerprint `3e916f32`) and → `xpub...CSMngAd` at
`m/48'/0'/2'/2'` (**same fingerprint** `3e916f32`, since same seed); independent seed B →
`xpub...RMsp5iVYdx` at `m/48'/0'/0'/2'` (fingerprint `04212830`). Fed this descriptor to
`OutputDescriptor.getOutputDescriptor(...)`:

```
wsh(sortedmulti(2,[3e916f32/48'/0'/1'/2']xpubA/<0;1>/*,[04212830/48'/0'/0'/2']xpubB/<0;1>/*,[3e916f32/48'/0'/2'/2']xpubC/<0;1>/*))
```

Program output (`DupFingerprintTest.java`, run via `java -cp out:<jars> DupFingerprintTest`):

```
Parsed OK. extendedPublicKeys map size = 3
wallet.getKeystores().size() = 3
  keystore label=Keystore 1 fingerprint=3e916f32 path=m/48'/0'/1'/2' xpub=xpub6DmaWs5NDZSapkbS...
  keystore label=Keystore 2 fingerprint=04212830 path=m/48'/0'/0'/2' xpub=xpub6DsaFRteX4GTMdNh...
  keystore label=Keystore 3 fingerprint=3e916f32 path=m/48'/0'/2'/2' xpub=xpub6EzPFHW5ojbnrLwz...

containsDuplicateExtendedKeys() = false
containsDuplicateKeystoreLabels() = false
wallet.checkWallet() -> NO EXCEPTION (wallet accepted as VALID)
wallet.isValid() = true
Number of keystores carrying fingerprint 3e916f32: 2
```

**All three cosigners survive intact, `isValid()` is `true`, no exception anywhere.** This is a direct
JVM execution against drongo HEAD `d666943`, not an inference from reading the code.

## 5. Sparrow UI — any warning on shared fingerprint?

SOURCED, sparrow HEAD `aeeeeb5`.

- `grep -rn -i "same seed\|duplicate.*fingerprint\|fingerprint.*duplicate\|same master fingerprint\|shares.*fingerprint" --include="*.java" --include="*.fxml" .` across the entire sparrow
  tree: **zero hits**.
- `grep -rln -i "duplicate"` files that also mention `keystore`/`fingerprint`:
  `ReceiveController.java`, `SendController.java`, `HeadersController.java`, `ElectrumServer.java`,
  `WalletLabels.java`, `SettingsController.java`, `TransactionDiagram.java` — inspected the
  keystore-relevant one (`SettingsController.java:978`): the only "duplicate" hit there is a comment
  about **label** collisions, not fingerprints.
- `containsDuplicateExtendedKeys()` (the one xpub-level dedup drongo has) is **never called anywhere in
  the sparrow application source** — `grep -rn "containsDuplicateExtendedKeys" .` → zero hits. It is
  exercised only indirectly, inside drongo's own `checkWallet()`.
- `KeystoreController.java` (the per-keystore edit pane) validates the fingerprint **field** only for
  format (`fingerprint.length() != 8 || !Utils.isHex(...)`, line 357) — no cross-keystore uniqueness
  check is registered on that validator.
- The "Apply" gate in `SettingsController.java` is wired purely to `wallet.isValid()` /
  `walletForm.getWallet().isValid()` (lines 326-894, multiple call sites; the finishing action at line
  967 calls `importedWallet.checkWallet()` directly). Since §3-4 show `isValid()` returns `true` for the
  shared-fingerprint case, **the Apply button would be enabled and the wallet accepted with no dialog,
  no color change, no icon — nothing.**

**Negative-claim scope, stated exactly:** the greps above covered `--include="*.java" --include="*.fxml"`
over the full checked-out `sparrow` working tree (all of `src/`, including `src/test`). No `.properties`
resource-bundle strings were searched (Sparrow's UI text may partly live in `.properties` files instead
of literals) — that is the one gap in this negative; a targeted follow-up would be
`grep -rn -i "fingerprint" --include="*.properties" .` if bundle-string coverage matters later.

## 6. Miniscript support

**Confirmed: Sparrow/drongo does not implement general miniscript.** Two independent lines of evidence:

**(a) Maintainer statement, live and current.** `gh issue view 1700 --repo sparrowwallet/sparrow` (state:
**OPEN**, title "Add support for any miniscript descriptors", filed by `yukibtc`). Maintainer
`craigraw` (association: collaborator) replied:

> "Supporting any miniscript expression is not a goal, given there are near infinite possibilities to
> express different spending conditions, the vast majority of which are not interesting. Most people
> when they say "miniscript" actually mean "decaying multisig", and existing implementations (like
> Nunchuk and Liana) support only a narrow set of options wrt decaying multisigs given the UI
> complexity involved."

This directly confirms the premise attributed to issue #1700 — it is a live, open, explicitly-declined
feature request, not a resolved/closed one.

**(b) `Miniscript.java` is not a miniscript engine.** SOURCED, full file (59 lines):
`/tmp/.../drongo/src/main/java/com/sparrowwallet/drongo/policy/Miniscript.java`. It has one real method,
`getNumSignaturesRequired()`, implemented entirely by regex:

```java
private static final Pattern KEYHASH_PATTERN = Pattern.compile("pkh?\\(");
private static final Pattern TAPROOT_PATTERN = Pattern.compile("tr\\(");
private static final Pattern SILENT_PAYMENTS_PATTERN = Pattern.compile("sp\\(");
private static final Pattern MULTI_PATTERN = Pattern.compile("multi\\((\\d+)");
```

— if the string contains `pk(`/`pkh(` or `tr(` or `sp(` it says "1 signature required"; else it looks
for literal `multi(N` and returns `N`; else it throws. There is no fragment type system, no AST, no
`or_d`/`and_v`/`andor`/`thresh`/`older`/`after`/wrap-function handling anywhere. Confirmed by
repo-wide grep: `grep -rln "or_d\|and_v\|multi_a\|older(\|Miniscript" --include="*.java" src/main` in
drongo returns exactly `Miniscript.java`, `Policy.java` (which just holds a `Miniscript` string) and
`FinalizingPSBTWallet.java` — none contain fragment-combinator logic.

`OutputDescriptor`'s own top-level dispatch, `ScriptType.fromDescriptor()` (`ScriptType.java` lines
1519-1526), matches only fixed literal prefixes per script type: `"pk("`, `"pkh("`, `"sortedmulti("`,
`"sh("`, `"sh(wpkh("`, `"sh(wsh("`, `"wpkh("`, `"wsh("`, `"tr("`, `"addr("` (each `getDescriptor()`
quoted directly from source, lines 71/195/364/540/686/792/898/1018/1141/1262). Inside the wrapper, key
extraction is a blind regex scan for anything matching `XPUB_PATTERN` over the *whole descriptor
string* (line 28), with no awareness of which combinator wraps each key.

### MEASURED: the two example descriptors from the question

**`wsh(or_d(multi(2,keyA,keyB),and_v(v:pkh(keyC),older(26280))))`** — vault-style, built with the same
three real xpubs as §4:

```
-> PARSED without exception. scriptType=P2WSH keyCount=3
-> toWallet() OK, keystores=3 isValid=true
-> getAddress(0,0) THREW: java.lang.IllegalStateException: Cannot derive receiving address from
   output descriptor wsh(sortedmulti(2,[...]keyC/<0;1>/*,[...]keyA/<0;1>/*,[...]keyB/<0;1>/*))
```

The parser accepts the *string* — because it never actually parses the `or_d`/`and_v`/`older`
structure, it just harvests three xpubs via regex and silently treats the whole thing as a flat
`sortedmulti(2, keyA, keyB, keyC)` (visible in the reconstructed descriptor in the exception message,
`wsh(sortedmulti(2,...))`, which drongo re-synthesizes from the flat key list once it tries to derive a
real address). It then fails as soon as something real is asked of it (address derivation), because
the reconstructed 2-of-3 flat multisig script doesn't correspond to what the actual `or_d(...)` locking
script would be. **This is silent misinterpretation at parse time, caught only downstream at
use time** — not correct miniscript support, and not an honest refusal either (it should refuse at
parse time, but doesn't).

**`tr(NUMS,{multi_a(2,keyA,keyB),multi_a(2,keyC,keyB)})`** (NUMS = the standard BIP-341 H point,
`02` + `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac`):

```
-> getOutputDescriptor() THREW: java.lang.IllegalArgumentException:
   Cannot determine the multisig threshold in a descriptor providing 3 keys
```

**Refused outright**, because `getMultisigThreshold()`'s regex is `Pattern.compile("multi\\(\\s*(\\d+)")`
— it requires the literal substring `multi(`, and `multi_a(` does not match it (the `_a` breaks the
match). With no threshold found and >1 key present, `getOutputDescriptorImpl` throws at parse time. So
**`multi_a()` (taproot script-path multisig) is not recognized by the descriptor grammar at all** —
confirmed both by this direct run and by the regex source (`OutputDescriptor.java` line 30:
`MULTI_PATTERN = Pattern.compile("multi\\(\\s*(\\d+)", Pattern.CASE_INSENSITIVE)`).

## Severity note (for the calling context)

Per this repo's constellation-wide severity rule, this is a **funds-safety / wrong-result class**
finding, not a secret-handling one: Sparrow will silently accept a 2-of-3 wallet where two "cosigners"
are actually one seed holder with no warning, degrading real security to 1-of-2 (attacker who
compromises that one seed can produce 2 of the 3 required signatures alone) while the UI shows 3 green
cosigner slots. It is a genuine gap in Sparrow, not a misreading of its source — confirmed at the
source level (§1-4), the UI level (§5), and by direct execution (§4, §6) against unmodified HEAD.
