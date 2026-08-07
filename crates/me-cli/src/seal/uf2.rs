//! `data`-family UF2 emission (§9.1).
//!
//! Written by the RP2350 bootrom in BOOTSEL mode, not by the running firmware.
//! Verified on real hardware 2026-08-06: a data-family UF2 at 0x10E00000 lands
//! byte-exact and leaves the signed image untouched.

/// Normative and not configurable. §5 derives it so the blob clears the signed
/// image and stays inside physical flash; past 0x11000000 a write wraps to
/// 0x10000000 and destroys the firmware.
pub const TARGET_ADDR: u32 = 0x10E0_0000;

/// `data`. NOT 0xe48bff59 (`rp2350_arm_s`), the bootable-image family the TinyGo
/// target uses — correct for firmware, wrong for a blob.
pub const FAMILY_DATA: u32 = 0xE48B_FF58;

const MAGIC_START0: u32 = 0x0A32_4655;
const MAGIC_START1: u32 = 0x9E5D_5157;
const MAGIC_END: u32 = 0x0AB1_6F30;
const FLAG_FAMILY_ID_PRESENT: u32 = 0x0000_2000;
const PAYLOAD: usize = 256;

pub fn to_uf2(blob: &[u8]) -> Vec<u8> {
    debug_assert!(!blob.is_empty(), "a blob is always >= 52 bytes");
    let num_blocks = blob.len().div_ceil(PAYLOAD) as u32;
    let mut out = Vec::with_capacity(num_blocks as usize * 512);
    for (i, chunk) in blob.chunks(PAYLOAD).enumerate() {
        let mut b = [0u8; 512];
        b[0..4].copy_from_slice(&MAGIC_START0.to_le_bytes());
        b[4..8].copy_from_slice(&MAGIC_START1.to_le_bytes());
        b[8..12].copy_from_slice(&FLAG_FAMILY_ID_PRESENT.to_le_bytes());
        b[12..16].copy_from_slice(&(TARGET_ADDR + i as u32 * PAYLOAD as u32).to_le_bytes());
        // Always 256, even for a short final chunk: the bootrom requires it, and
        // the device bounds every read by pub_len/ct_len so padding is unseen.
        b[16..20].copy_from_slice(&(PAYLOAD as u32).to_le_bytes());
        b[20..24].copy_from_slice(&(i as u32).to_le_bytes());
        b[24..28].copy_from_slice(&num_blocks.to_le_bytes());
        b[28..32].copy_from_slice(&FAMILY_DATA.to_le_bytes());
        b[32..32 + chunk.len()].copy_from_slice(chunk);
        b[508..512].copy_from_slice(&MAGIC_END.to_le_bytes());
        out.extend_from_slice(&b);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(b: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
    }

    #[test]
    fn every_block_conforms_not_just_the_first() {
        // 600 bytes = 3 blocks, the last short. §9.1 requires payloadSize 256 on
        // EVERY block; writing chunk.len() for the final block would pass a
        // first-block-only check.
        let uf2 = to_uf2(&vec![0xABu8; 600]);
        assert_eq!(uf2.len(), 3 * 512);
        for (i, b) in uf2.chunks(512).enumerate() {
            assert_eq!(field(b, 0), 0x0A32_4655, "block {i} magicStart0");
            assert_eq!(field(b, 4), 0x9E5D_5157, "block {i} magicStart1");
            assert_eq!(field(b, 8), 0x0000_2000, "block {i} flags");
            // LITERALS, not TARGET_ADDR / FAMILY_DATA. Asserting a constant
            // against itself moves both sides of the comparison, so the
            // assertion holds for every value the constant could take. Measured:
            // FAMILY_DATA -> 0xE48B_FF59 and TARGET_ADDR -> 0x1000_0000 each
            // survived the WHOLE 169-test suite with zero failures. Those are
            // the two constants whose corruption is physically destructive —
            // 0xE48B_FF59 is `rp2350_arm_s`, the bootable-image family, and
            // 0x1000_0000 aims the write at the signed firmware. Every other
            // field here was already literal-pinned; these two were the gap.
            assert_eq!(field(b, 12), 0x10E0_0000 + i as u32 * 256, "block {i} addr");
            assert_eq!(field(b, 16), 256, "block {i} payloadSize must be 256");
            assert_eq!(field(b, 20), i as u32, "block {i} blockNo");
            assert_eq!(field(b, 24), 3, "block {i} numBlocks");
            assert_eq!(
                field(b, 28),
                0xE48B_FF58u32,
                "block {i} familyID (data, NOT rp2350_arm_s 0xE48BFF59)"
            );
            assert_eq!(field(b, 508), 0x0AB1_6F30, "block {i} magicEnd");
        }
        // The pinned vector sha256s assume 0x00 padding.
        assert!(uf2[2 * 512 + 32 + 88..2 * 512 + 32 + 256]
            .iter()
            .all(|&b| b == 0));
    }

    #[test]
    fn single_block_for_a_short_blob() {
        assert_eq!(to_uf2(&vec![0xABu8; 211]).len(), 512);
    }
}
