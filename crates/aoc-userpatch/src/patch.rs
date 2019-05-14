use std::str;

/// Describes a patch as an offset and a hexadecimal string.
struct Injection(pub u32, pub &'static str);

/// Decode a hexadecimal string to a list of byte values.
fn decode_hex(hexa: &str) -> Vec<u8> {
    assert_eq!(hexa.len() % 2, 0, "hex string must have length divisible by 2");
    let mut bytes = Vec::with_capacity(hexa.len() / 2);
    for c in hexa.as_bytes().chunks(2) {
        let high = char::from(c[0]).to_digit(16)
            .expect("expected only hexadecimal characters");
        let low = char::from(c[1]).to_digit(16)
            .expect("expected only hexadecimal characters");
        bytes.push((high * 16 + low) as u8);
    }
    bytes
}

/// Overwrite bytes in buffer at an offset.
fn apply_patch(buffer: &mut [u8], offset: usize, patch: &[u8]) {
    let end = offset + patch.len();
    (&mut buffer[offset..end]).copy_from_slice(&patch);
}

/// Install UserPatch 1.5 into a buffer containing a 1.0c executable.
pub fn install_into(exe_buffer: &mut [u8]) {
    let injections = include!(concat!(env!("OUT_DIR"), "/injections.rs"));

    for Injection(addr, patch) in injections.iter() {
        let patch = decode_hex(&patch);
        apply_patch(exe_buffer, *addr as usize, &patch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_hex_test() {
        assert_eq!(decode_hex("ABCDEF"), vec![0xAB_u8, 0xCD_u8, 0xEF_u8]);
        assert_eq!(decode_hex("123456"), vec![0x12_u8, 0x34_u8, 0x56_u8]);
    }

    #[test]
    fn apply_patch_test() {
        let mut buffer = vec![0u8; 256];
        apply_patch(&mut buffer, 8, &[1u8; 8]);
        assert_eq!(&buffer[0..24], &[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]);
        apply_patch(&mut buffer, 10, &[2u8; 4]);
        assert_eq!(&buffer[0..24], &[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            1u8, 1u8, 2u8, 2u8, 2u8, 2u8, 1u8, 1u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]);
    }
}
