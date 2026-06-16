//! `mnemonic-engrave` (`me`) — converts public constellation strings (md1/mk1)
//! into NFC NDEF payloads for SeedHammer II. Refuses the secret ms1.

pub mod classify;
pub mod ndef;
pub mod validate;
