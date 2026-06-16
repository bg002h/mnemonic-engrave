//! NDEF well-known Text record, TLV-wrapped for NFC Forum Type-2/Type-5 tags,
//! per `design/SPEC_seedhammer_engrave.md` §6. Matches `nfc/ndef/ndef.go`.

/// NDEF record header: MB=1 ME=1 CF=0 SR=1 IL=0 TNF=001 (well-known) => 0xD1.
const NDEF_HEADER: u8 = 0xD1;
const TYPE_TEXT: u8 = b'T'; // 0x54
const STATUS_UTF8_NOLANG: u8 = 0x00; // bit7=0 (UTF-8), lang-code length 0
const TLV_NDEF: u8 = 0x03;
const TLV_TERMINATOR: u8 = 0xFE;

/// Errors from NDEF encoding.
#[derive(Debug, PartialEq, Eq)]
pub enum NdefError {
    /// Text/message too large for the single-byte short-record/TLV length form.
    TooLong(usize),
}

impl std::fmt::Display for NdefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NdefError::TooLong(n) => {
                write!(f, "payload too long for a short NDEF record: {n} bytes")
            }
        }
    }
}
impl std::error::Error for NdefError {}

/// Build a bare NDEF message: one well-known Text record (UTF-8, empty language).
pub fn text_record(text: &str) -> Result<Vec<u8>, NdefError> {
    let payload_len = 1 + text.len(); // status byte + text bytes
    if payload_len > u8::MAX as usize {
        return Err(NdefError::TooLong(text.len()));
    }
    let mut out = Vec::with_capacity(4 + payload_len);
    out.push(NDEF_HEADER);
    out.push(0x01); // type length = 1
    out.push(payload_len as u8); // payload length (SR=1, single byte)
    out.push(TYPE_TEXT);
    out.push(STATUS_UTF8_NOLANG);
    out.extend_from_slice(text.as_bytes());
    Ok(out)
}

/// Wrap an NDEF message in the NFC Forum NDEF TLV (`0x03 <len> .. 0xFE`),
/// 1-byte length form (len < 255).
pub fn tlv_wrap(message: &[u8]) -> Result<Vec<u8>, NdefError> {
    if message.len() >= 0xFF {
        return Err(NdefError::TooLong(message.len()));
    }
    let mut out = Vec::with_capacity(message.len() + 3);
    out.push(TLV_NDEF);
    out.push(message.len() as u8);
    out.extend_from_slice(message);
    out.push(TLV_TERMINATOR);
    Ok(out)
}

/// Canonical encoding: TLV-wrapped NDEF Text record carrying `text`.
pub fn encode_text_tlv(text: &str) -> Result<Vec<u8>, NdefError> {
    tlv_wrap(&text_record(text)?)
}

/// Minimal decoder mirroring SeedHammer's reader: unwrap the NDEF TLV, parse a
/// single well-known Text record, return the UTF-8 text. Used for the
/// round-trip self-test; `None` on any structural mismatch.
///
/// Intentionally handles only the 1-byte TLV length form and does NOT check the
/// `0xFE` terminator — it only needs to round-trip `me`'s own bounded output,
/// which never uses the 3-byte length form. Not a general-purpose NDEF parser.
pub fn decode_text_tlv(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 2 || bytes[0] != TLV_NDEF {
        return None;
    }
    let len = bytes[1] as usize;
    let msg = bytes.get(2..2 + len)?;
    decode_text_record(msg)
}

fn decode_text_record(msg: &[u8]) -> Option<String> {
    if msg.len() < 3 {
        return None;
    }
    let flags = msg[0];
    if flags & 0x07 != 0x01 {
        return None; // TNF must be well-known
    }
    let type_len = msg[1] as usize;
    let plen = msg[2] as usize;
    let rest = &msg[3..];
    let typ = rest.get(..type_len)?;
    if typ != [TYPE_TEXT] {
        return None;
    }
    let payload = rest.get(type_len..type_len + plen)?;
    let status = *payload.first()?;
    if status & 0x80 != 0 {
        return None; // UTF-16 unsupported (matches ndef.go)
    }
    let lang_len = (status & 0x3F) as usize;
    let text_bytes = payload.get(1 + lang_len..)?;
    std::str::from_utf8(text_bytes).ok().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_expected_bytes() {
        // "md1q" -> TLV(03,len) D1 01 plen 54 00 <text>
        let got = encode_text_tlv("md1q").unwrap();
        let expected: Vec<u8> = vec![
            0x03, 0x09, // TLV: NDEF, message length 9
            0xD1, 0x01, 0x05, 0x54, 0x00, // header, typelen, plen(=1+4), 'T', status
            b'm', b'd', b'1', b'q', // text
            0xFE, // terminator
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn round_trips() {
        let s = "mk1qpzry9x8gf2tv";
        let bytes = encode_text_tlv(s).unwrap();
        assert_eq!(decode_text_tlv(&bytes).as_deref(), Some(s));
    }

    #[test]
    fn rejects_oversize() {
        let big = "a".repeat(255);
        assert_eq!(text_record(&big), Err(NdefError::TooLong(255)));
    }
}
