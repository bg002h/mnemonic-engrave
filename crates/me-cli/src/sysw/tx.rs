//! Minimal structural parser for a serialized Bitcoin transaction, and its
//! txid.
//!
//! Exists for TWO consumers, both in this container's mt handling:
//!
//! 1. `[mt-decode]` confirmation (`sysw::mt`): an `mt1` chunk set reassembles
//!    to bytes with **no semantic decoder of its own** — any complete set of
//!    BCH-valid strings "decodes", which is exactly the entropy-smuggling
//!    channel `[mdmk-decode]` closes for md/mk with their real decoders. The
//!    semantic arbiter mt has is this: the bytes must parse as a transaction,
//!    and the set's 20-bit `chunk_set_id` must equal the top 20 bits of the
//!    parsed transaction's display txid (SPEC_mt_v0_1 §10.13 c). A wrapped
//!    32-byte seed fails the parse; a forged transaction with a random header
//!    fails the txid binding at 1 in 2^20.
//!
//! 2. The `tx:` record class (`sysw::record::TX_PREFIX`): a raw signed
//!    transaction delivered for QR engraving. Classification requires the body
//!    to parse, so a payload cannot smuggle arbitrary bytes under the prefix.
//!
//! This is a STRUCTURAL parser only. It checks the serialization shape —
//! version, inputs, outputs, optional BIP-144 witness section, locktime,
//! nothing trailing — and computes the txid over the witness-stripped form.
//! It validates no script, checks no signature, and does not judge whether
//! the transaction is signed; `mt` owns those refusals at encode time.
//!
//! The Go port is `seedhammer.com/mt`'s `ParseTx`; the two are bound by the
//! vectors in `mt-codec/src/test_vectors/mt1_v1.json`, whose `raw_hex`/`txid`
//! pairs were produced by a real node.

use sha2::{Digest, Sha256};

/// What a structural parse learns. Everything a review screen needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxSummary {
    /// The txid in DISPLAY form: byte-reversed, lowercase hex — the form a
    /// user reads and the form `chunk_set_id` binds to.
    pub txid_display: String,
    /// Serialized length, witness included.
    pub size: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub segwit: bool,
    /// **Every input carries a signature** — a non-empty scriptSig, or at least
    /// one witness item. False means the transaction cannot be broadcast.
    ///
    /// This is the ONLY thing that separates a signature-stripped transaction
    /// from the honest one it was made from: stripping the witness is precisely
    /// the operation the txid is defined to ignore, so **both have the same
    /// txid**, and no carried identifier can tell them apart.
    ///
    /// The design this replaces carried a second identifier (a `wtxid`) to
    /// catch it, and could only catch the case where the carried wtxid was
    /// *inconsistent* with the body — an encoder bug. A body whose txid and
    /// wtxid were both recomputed from stripped bytes passed, and that document
    /// called the gap unclosable *"because it IS an honest witness-free
    /// transaction"*. **It is not.** An honest witness-free transaction is a
    /// legacy transaction, and a signed one has non-empty scriptSigs. This
    /// predicate catches both cases and carries no field at all.
    pub every_input_signed: bool,
    /// The INDICES of the inputs that carry neither. Empty exactly when
    /// [`TxSummary::every_input_signed`] is true.
    ///
    /// Carried rather than recomputed by the caller because a refusal that
    /// says only *"an input is unsigned"* sends the operator back to a wallet
    /// with nothing to look at, and because `--allow-unsigned-inputs` has to
    /// name what it let through -- an override whose warning is as vague as
    /// the refusal it replaces is a switch, not a decision.
    pub unsigned_inputs: Vec<usize>,
}

impl TxSummary {
    /// The top 20 bits of the display txid — the value an `mt1` chunk set's
    /// `chunk_set_id` must carry. First five hex characters, read as a number,
    /// exactly as `mt-codec`'s `content_id_from_txid_display` reads them.
    pub fn chunk_set_id(&self) -> u32 {
        u32::from_str_radix(&self.txid_display[..5], 16).expect("txid is lowercase hex")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxError {
    /// Ran out of bytes mid-field.
    Truncated,
    /// Parsed cleanly but bytes remain — this is not one transaction.
    TrailingBytes,
    /// A compactSize encoded non-canonically. Bitcoin Core's `ReadCompactSize`
    /// rejects these, so accepting one here would admit bytes no node would.
    NonCanonicalVarInt,
    /// Zero inputs. (A zero first varint is how the segwit marker is
    /// recognised; a zero input count WITH a valid marker is still refused.)
    NoInputs,
    NoOutputs,
    /// Marker byte 0x00 not followed by flag 0x01 (BIP-144).
    BadSegwitFlag,
    /// P5 M-6 — the segwit marker and flag are present, and EVERY witness stack
    /// is empty.
    ///
    /// Both this parser and the device's accepted that shape, and if every
    /// scriptSig is non-empty it also passed the signature predicate — so a QR
    /// plate could be cut, under a legend saying "raw signed bitcoin tx … then
    /// broadcast", for bytes **no node will accept**. Bitcoin Core's
    /// deserializer rejects it ("Superfluous witness record") and its legacy
    /// re-parse then fails on the 0x00 input count.
    ///
    /// A transaction that carries the marker is claiming witness data. If it
    /// has none, the honest serialization is the legacy one, without the
    /// marker.
    EmptyWitnessOnSegwitMarker,
    /// A declared length larger than the buffer could ever satisfy.
    LengthOverflow,
}

impl std::fmt::Display for TxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxError::Truncated => "transaction truncated",
            TxError::TrailingBytes => "trailing bytes after the transaction",
            TxError::NonCanonicalVarInt => "non-canonical compactSize",
            TxError::NoInputs => "transaction has no inputs",
            TxError::NoOutputs => "transaction has no outputs",
            TxError::BadSegwitFlag => "segwit marker without the 0x01 flag",
            TxError::EmptyWitnessOnSegwitMarker => "the segwit marker is set but every witness stack is empty; no node will accept these bytes",
            TxError::LengthOverflow => "declared length exceeds the data",
        };
        f.write_str(s)
    }
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], TxError> {
        // Checked against the REMAINDER, not `pos + n`, so a huge declared
        // length cannot wrap the addition.
        if n > self.buf.len() - self.pos {
            return Err(TxError::Truncated);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, TxError> {
        Ok(self.take(1)?[0])
    }

    /// Bitcoin compactSize, canonical form required.
    fn varint(&mut self) -> Result<u64, TxError> {
        let first = self.u8()?;
        let v = match first {
            0..=0xFC => u64::from(first),
            0xFD => {
                let b = self.take(2)?;
                let v = u64::from(u16::from_le_bytes([b[0], b[1]]));
                if v < 0xFD {
                    return Err(TxError::NonCanonicalVarInt);
                }
                v
            }
            0xFE => {
                let b = self.take(4)?;
                let v = u64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                if v <= u64::from(u16::MAX) {
                    return Err(TxError::NonCanonicalVarInt);
                }
                v
            }
            0xFF => {
                let b = self.take(8)?;
                let v = u64::from_le_bytes(b.try_into().unwrap());
                if v <= u64::from(u32::MAX) {
                    return Err(TxError::NonCanonicalVarInt);
                }
                v
            }
        };
        Ok(v)
    }

    /// A count that will drive a loop. Bounded by what the buffer could hold
    /// at ≥1 byte per element, so a hostile count cannot spin the parser.
    fn count(&mut self) -> Result<usize, TxError> {
        let v = self.varint()?;
        if v > (self.buf.len() - self.pos) as u64 {
            return Err(TxError::LengthOverflow);
        }
        Ok(v as usize)
    }

    /// A length-prefixed byte string (script, witness item). **Returns it**,
    /// because the signature predicate needs to know whether a scriptSig was
    /// empty and `()` cannot say.
    fn skip_bytes(&mut self) -> Result<&'a [u8], TxError> {
        let n = self.varint()?;
        if n > (self.buf.len() - self.pos) as u64 {
            return Err(TxError::LengthOverflow);
        }
        self.take(n as usize)
    }
}

/// Parse one serialized transaction. The WHOLE buffer must be consumed.
pub fn parse(bytes: &[u8]) -> Result<TxSummary, TxError> {
    let mut c = Cursor { buf: bytes, pos: 0 };
    let version = c.take(4)?;

    // BIP-144: a 0x00 where the input count belongs is the witness marker —
    // unambiguous because a real input count of zero is invalid either way.
    let mark = c.pos;
    let mut segwit = false;
    if c.u8()? == 0x00 {
        if c.u8()? != 0x01 {
            return Err(TxError::BadSegwitFlag);
        }
        segwit = true;
    } else {
        c.pos = mark;
    }

    // The witness-stripped span: everything from the input count through the
    // last output. Recorded rather than re-serialized, so the txid is computed
    // over bytes that provably came from the wire.
    let core_start = c.pos;
    let n_in = c.count()?;
    if n_in == 0 {
        return Err(TxError::NoInputs);
    }
    // Per input, not per transaction: a mixed transaction keeps its legacy
    // scriptSigs when the witnesses are stripped, so a whole-transaction test
    // would pass while the segwit inputs were left unsigned.
    let mut input_has_script_sig = Vec::with_capacity(n_in);
    for _ in 0..n_in {
        c.take(36)?; // outpoint: txid + vout
        input_has_script_sig.push(!c.skip_bytes()?.is_empty()); // scriptSig
        c.take(4)?; // sequence
    }
    let n_out = c.count()?;
    if n_out == 0 {
        return Err(TxError::NoOutputs);
    }
    for _ in 0..n_out {
        c.take(8)?; // value
        c.skip_bytes()?; // scriptPubKey
    }
    let core_end = c.pos;

    let mut input_has_witness = vec![false; n_in];
    if segwit {
        for slot in input_has_witness.iter_mut() {
            let items = c.count()?;
            for _ in 0..items {
                c.skip_bytes()?;
            }
            *slot = items > 0;
        }
        // P5 M-6 — the marker CLAIMS witness data. All-empty stacks means the
        // bytes are neither a valid segwit transaction (Core: "Superfluous
        // witness record") nor a legacy one (its re-parse hits the 0x00 input
        // count). Accepting them lets a plate be cut for a transaction that
        // cannot be broadcast, under a legend that says it can.
        if input_has_witness.iter().all(|&w| !w) {
            return Err(TxError::EmptyWitnessOnSegwitMarker);
        }
    }
    let locktime = c.take(4)?;
    if c.pos != bytes.len() {
        return Err(TxError::TrailingBytes);
    }

    // PER INPUT, and the list is the answer rather than a bool derived from a
    // list: `unsigned_inputs` is what the messages print and `every_input_signed`
    // is defined from it, so the two cannot drift.
    let unsigned: Vec<usize> = (0..n_in)
        .filter(|&i| !(input_has_script_sig[i] || input_has_witness[i]))
        .collect();

    // txid = SHA256d(version ‖ inputs ‖ outputs ‖ locktime) — the
    // witness-STRIPPED serialization (BIP-141), displayed byte-reversed.
    let mut h = Sha256::new();
    h.update(version);
    h.update(&bytes[core_start..core_end]);
    h.update(locktime);
    let d1 = h.finalize();
    let d2 = Sha256::digest(d1);
    let txid_display: String = d2.iter().rev().map(|b| format!("{b:02x}")).collect();

    Ok(TxSummary {
        txid_display,
        size: bytes.len(),
        inputs: n_in,
        outputs: n_out,
        segwit,
        every_input_signed: unsigned.is_empty(),
        unsigned_inputs: unsigned,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The "even" vector from `mt-codec/src/test_vectors/mt1_v1.json` — a REAL
    /// signed 1-in/2-out P2WPKH transaction, txid from the node that made it.
    /// 222 bytes.
    pub const EVEN_RAW_HEX: &str = "020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000";

    /// The SAME transaction with every witness stripped: 113 bytes, and its
    /// txid is **byte-identical** to the 222-byte original, because stripping
    /// the witness is precisely the operation the txid is defined to ignore.
    /// Built by parsing `EVEN_RAW_HEX` and re-emitting the legacy form.
    ///
    /// This is the artifact the whole signature check exists for: it parses,
    /// it round-trips, its txid matches what an operator would compare against
    /// `mt inspect` — and **not one input carries a signature**, so a plate cut
    /// from it can never be broadcast.
    pub const EVEN_STRIPPED_HEX: &str = "02000000017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e60000000";

    /// A MIXED transaction with its witnesses stripped: input 0 is legacy and
    /// still carries its scriptSig, input 1 is a segwit input whose witness was
    /// removed. **A whole-transaction test passes this** — some input is signed
    /// — while input 1 is unspendable. Only the PER-INPUT predicate catches it,
    /// and mutation testing is what proved the difference is real.
    pub const MIXED_STRIPPED_HEX: &str = "020000000211111111111111111111111111111111111111111111111111111111111111110000000048473030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030ffffffff22222222222222222222222222222222222222222222222222222222222222220100000000ffffffff0150c3000000000000160014333333333333333333333333333333333333333300000000";
    pub const EVEN_TXID: &str = "2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630";

    pub fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn parses_a_real_signed_segwit_tx_and_reproduces_its_txid() {
        let b = unhex(EVEN_RAW_HEX);
        let t = parse(&b).unwrap();
        assert_eq!(t.txid_display, EVEN_TXID);
        assert_eq!((t.size, t.inputs, t.outputs, t.segwit), (222, 1, 2, true));
        assert_eq!(t.chunk_set_id(), 0x2dcf2);
    }

    /// A legacy (pre-segwit) shape: txid == sha256d(whole serialization).
    /// Hand-built: 1 input with an empty scriptSig, 1 output, so it parses but
    /// is unsigned — this parser deliberately does not care.
    #[test]
    fn parses_a_legacy_tx() {
        let mut b = vec![0x01, 0x00, 0x00, 0x00]; // version
        b.push(0x01); // one input
        b.extend_from_slice(&[0xAA; 36]); // outpoint
        b.push(0x00); // empty scriptSig
        b.extend_from_slice(&[0xFF; 4]); // sequence
        b.push(0x01); // one output
        b.extend_from_slice(&[0x00; 8]); // value
        b.extend_from_slice(&[0x02, 0x51, 0x51]); // 2-byte script
        b.extend_from_slice(&[0x00; 4]); // locktime
        let t = parse(&b).unwrap();
        assert!(!t.segwit);
        assert_eq!((t.inputs, t.outputs), (1, 1));
        // Legacy: the stripped form IS the wire form.
        let d = Sha256::digest(Sha256::digest(&b));
        let want: String = d.iter().rev().map(|x| format!("{x:02x}")).collect();
        assert_eq!(t.txid_display, want);
    }

    #[test]
    fn refuses_the_shapes_a_smuggler_or_a_scratch_produces() {
        let good = unhex(EVEN_RAW_HEX);
        // 32 bytes of entropy — the §5.3.2 smuggling case, now aimed at mt.
        assert!(parse(&[0xAB; 32]).is_err());
        // Truncated at every prefix length: never a panic, never an accept.
        for n in 0..good.len() {
            assert!(parse(&good[..n]).is_err(), "prefix of {n} bytes accepted");
        }
        // One trailing byte.
        let mut long = good.clone();
        long.push(0x00);
        assert_eq!(parse(&long), Err(TxError::TrailingBytes));
        // Marker without flag.
        let mut bad = good.clone();
        bad[5] = 0x02;
        assert_eq!(parse(&bad), Err(TxError::BadSegwitFlag));
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn refuses_zero_inputs_and_zero_outputs() {
        // version + 0x00 0x01 (segwit) + zero input count.
        let b = [0u8, 0, 0, 0, 0x00, 0x01, 0x00];
        assert_eq!(parse(&b), Err(TxError::NoInputs));
        let mut b = vec![0x01, 0x00, 0x00, 0x00, 0x01];
        b.extend_from_slice(&[0xAA; 36]);
        b.push(0x00);
        b.extend_from_slice(&[0xFF; 4]);
        b.push(0x00); // zero outputs
        b.extend_from_slice(&[0x00; 4]);
        assert_eq!(parse(&b), Err(TxError::NoOutputs));
    }

    #[test]
    fn refuses_non_canonical_varints() {
        // Input count as 0xFD 0x01 0x00 — the value 1 in the 3-byte form.
        let mut b = vec![0x01, 0x00, 0x00, 0x00, 0xFD, 0x01, 0x00];
        b.extend_from_slice(&[0xAA; 36]);
        b.push(0x00);
        b.extend_from_slice(&[0xFF; 4]);
        b.push(0x01);
        b.extend_from_slice(&[0x00; 8]);
        b.push(0x00);
        b.extend_from_slice(&[0x00; 4]);
        assert_eq!(parse(&b), Err(TxError::NonCanonicalVarInt));
    }

    /// A hostile count cannot make the parser loop past the buffer: a declared
    /// script length of u32::MAX against a 300-byte buffer is LengthOverflow,
    /// not 4 GiB of takes.
    #[test]
    fn a_huge_declared_length_fails_fast() {
        let mut b = vec![0x01, 0x00, 0x00, 0x00, 0x01];
        b.extend_from_slice(&[0xAA; 36]);
        b.extend_from_slice(&[0xFE, 0xFF, 0xFF, 0xFF, 0xFF]); // scriptSig len u32::MAX
        b.extend_from_slice(&[0x00; 64]);
        assert_eq!(parse(&b), Err(TxError::LengthOverflow));
    }
}

#[cfg(test)]
mod signature_predicate_tests {
    use super::tests::{unhex, EVEN_RAW_HEX, EVEN_STRIPPED_HEX, MIXED_STRIPPED_HEX};
    use super::*;

    /// RED FIRST. A witness-stripped transaction has the SAME txid as the
    /// honest one, so no identifier — carried or computed — distinguishes them.
    /// What distinguishes them is that no input carries a signature.
    #[test]
    fn a_signature_stripped_transaction_is_refused() {
        let stripped = unhex(EVEN_STRIPPED_HEX);
        let honest = unhex(EVEN_RAW_HEX);

        // The premise, asserted rather than assumed: the txids are identical.
        let s = parse(&stripped).expect("the stripped body still parses — that is the problem");
        let h = parse(&honest).expect("the honest body parses");
        assert_eq!(
            s.txid_display, h.txid_display,
            "premise broken: if the txids differed, the txid alone would catch this"
        );

        assert!(
            !s.every_input_signed,
            "the stripped body has no witness and empty scriptSigs — it is unsigned"
        );
        assert!(
            h.every_input_signed,
            "the honest body's single input carries a witness"
        );
    }

    /// The predicate is PER INPUT, and this is the vector that proves it.
    /// Mutation-tested: replacing the `all` with an `any` over the whole
    /// transaction leaves `a_signature_stripped_transaction_is_refused` GREEN
    /// and turns this one RED.
    #[test]
    fn one_signed_input_does_not_vouch_for_the_others() {
        let t = parse(&unhex(MIXED_STRIPPED_HEX)).expect("a mixed stripped body parses");
        assert_eq!(t.inputs, 2);
        assert!(
            !t.every_input_signed,
            "input 1 has no scriptSig and no witness — input 0 being signed does not make the \
             transaction spendable, and a whole-transaction test would call this signed"
        );
        // The INDICES, not just the verdict: this is what `--allow-unsigned-inputs`
        // and the refusal both print, and naming input 0 would send the operator
        // to the one input that is fine.
        assert_eq!(t.unsigned_inputs, vec![1]);
    }

    /// `every_input_signed` is DEFINED from `unsigned_inputs`, and this asserts
    /// the two agree on all three shapes rather than trusting the definition to
    /// stay that way.
    #[test]
    fn the_verdict_and_the_index_list_are_one_answer() {
        for hex in [EVEN_RAW_HEX, EVEN_STRIPPED_HEX, MIXED_STRIPPED_HEX] {
            let t = parse(&unhex(hex)).unwrap();
            assert_eq!(t.every_input_signed, t.unsigned_inputs.is_empty(), "{hex}");
            assert!(t.unsigned_inputs.iter().all(|&i| i < t.inputs), "{hex}");
        }
        assert_eq!(
            parse(&unhex(EVEN_STRIPPED_HEX)).unwrap().unsigned_inputs,
            vec![0]
        );
        assert!(parse(&unhex(EVEN_RAW_HEX))
            .unwrap()
            .unsigned_inputs
            .is_empty());
    }

    /// P5 M-6 — a segwit-marked transaction whose every witness stack is empty
    /// parses as well-formed in both implementations and, with non-empty
    /// scriptSigs, passes the signature predicate too. Bitcoin Core rejects
    /// exactly these bytes, so a plate cut from them could never be broadcast.
    #[test]
    fn a_segwit_marker_with_no_witness_data_is_refused() {
        // version | 00 01 | 1 input (prevout, 1-byte scriptSig, sequence)
        // | 1 output (value, 1-byte script) | empty witness stack | locktime
        let mut b = Vec::new();
        b.extend_from_slice(&[0x02, 0, 0, 0]); // version
        b.extend_from_slice(&[0x00, 0x01]); // segwit marker + flag
        b.push(0x01); // 1 input
        b.extend_from_slice(&[0x11; 32]); // prevout txid
        b.extend_from_slice(&[0, 0, 0, 0]); // prevout vout
        b.push(0x01); // scriptSig length
        b.push(0x51); // a non-empty scriptSig -> "signed" by the predicate
        b.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // sequence
        b.push(0x01); // 1 output
        b.extend_from_slice(&[0x10, 0x27, 0, 0, 0, 0, 0, 0]); // value
        b.push(0x01); // scriptPubKey length
        b.push(0x51);
        b.push(0x00); // THE DEFECT: witness stack with ZERO items
        b.extend_from_slice(&[0, 0, 0, 0]); // locktime

        assert_eq!(
            parse(&b),
            Err(TxError::EmptyWitnessOnSegwitMarker),
            "the marker claims witness data; with none, no node accepts these bytes"
        );
    }

    /// THE CONTROL: the corpus's real segwit transaction, which DOES carry
    /// witness data, must still parse. Without this, refusing every segwit
    /// transaction would satisfy the test above.
    #[test]
    fn a_real_segwit_transaction_still_parses() {
        let raw = unhex(EVEN_RAW_HEX);
        let s = parse(&raw).expect("the pinned segwit vector must still parse");
        assert!(s.every_input_signed, "and still report its inputs signed");
    }
}
