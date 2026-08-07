//! `me seal` — encrypt a constellation payload for delivery to SeedHammer II
//! flash. See design/SPEC_encrypted_payload_delivery.md.

pub mod crypto;
pub mod passphrase;
pub mod record;
pub mod wire;
