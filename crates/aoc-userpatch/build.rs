//! The original UserPatch installer works by applying binary patches to an Age of Empires 2
//! executable.
//!
//! This build script reads a UserPatch installer executable, decompresses it using upx, and then
//! reads through the bytes to find references to the patches that the installer applies. (Most)
//! patches belong to a feature. Features are identified by calls to their constructor function. All
//! patches that are referenced _after_ a particular Feature constructor are assumed to belong to
//! this feature; for the current installer version targeted by this crate, that assumption is
//! accurate. Patches are referenced by various constructor functions, and are sometimes stored as
//! hex strings, sometimes as byte arrays. There is a special case for patches that just CALL or JMP
//! to an address.
//!
//! After collecting all the patches, this script normalizes them so they are all a simple offset +
//! byte array combo. They are serialized to Rust structures and grouped by feature. The output is a
//! file `injections.rs` that defines a static `FEATURES` variable. We can `include!` that file in
//! the crate's source code.

#[allow(clippy::unreadable_literal)]

use encoding_rs::UTF_16LE;
use std::{
    env,
    fs::{self, File},
    io::{Result, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// The location of the FeatureData constructor.
const FEATURE_ADDRESS: u32 = 0x00402130;
/// The location of the PatchData hex string overload in memory space.
const HEX_PATCH_ADDRESS: u32 = 0x00402750;
/// The location of the PatchData hex string + name overload in memory space.
const NAMED_HEX_PATCH_ADDRESS: u32 = 0x004023E0;
/// The location of the PatchData bytes overload in memory space.
const BYTE_PATCH_ADDRESS: u32 = 0x00402AF0;
/// The location of the PatchData JMP overload in memory space.
const JMP_PATCH_ADDRESS: u32 = 0x00402FA0;
/// The location of the std::wstring constructor in memory space.
const STRING_CONSTRUCTOR_ADDRESS: u32 = 0x004AB8B0;
const STRING_CONSTRUCTOR_ADDRESS16: u32 = 0x004AB7E0;

/// Base address of the code section.
const CODE_BASE_ADDRESS: u32 = 0x00400C00;
/// Base address of the read-only data section.
const RDATA_BASE_ADDRESS: u32 = 0x00401400;
/// Base address of the data section.
const DATA_BASE_ADDRESS: u32 = 0x00401800;

/// Opcode for `call` instructions.
const ASM_CALL: u8 = 0xE8;
/// Opcode for 32-bit `jmp` instructions.
const ASM_JMP: u8 = 0xE9;
/// Opcode for `push` instructions with 32 bit operand.
const ASM_PUSH32: u8 = 0x68;
/// Opcode for `push` instructions with 8 bit operand.
const ASM_PUSH8: u8 = 0x6A;

/// Feature data.
struct Feature {
    name: String,
    optional: bool,
    enabled_by_default: bool,
    #[allow(unused)]
    always_enabled: bool,
    affects_sync: bool,
    patches: Vec<Patch>,
}

/// A patch.
#[derive(Debug)]
enum Patch {
    /// Not actually a patch, just metadata that we serialize as a comment
    /// for manual inspection of the injections.rs file. Sorry…
    Header(String),
    /// Insert a function call, (at_address, target, padding)
    ///
    /// `padding` bytes are filled with NOPs after the newly inserted CALL.
    Call(u32, u32, u32),
    /// Insert an unconditional jump, (at_address, target, padding)
    ///
    /// `padding` bytes are filled with NOPs after the newly inserted JMP.
    Jmp(u32, u32, u32),
    /// A binary patch, (at_address, bytes)
    Hex(u32, Vec<u8>),
}

/// Read a NUL-terminated string from a byte slice.
fn read_c_str(bytes: &[u8], start: u32) -> String {
    let str_bytes = (&bytes[start as usize..])
        .iter()
        .cloned()
        .take_while(|&c| c != 0)
        .collect::<Vec<u8>>();

    String::from_utf8(str_bytes).unwrap()
}

/// Read a NUL-terminated UTF-16 string.
fn read_utf16_str(bytes: &[u8], start: u32) -> String {
    let str_bytes = (&bytes[start as usize..])
        .chunks(2)
        .take_while(|c| c[0] != 0 || c[1] != 0)
        .flat_map(|c| c.iter().cloned())
        .collect::<Vec<u8>>();

    UTF_16LE.decode(&str_bytes).0.to_owned().to_string()
}

/// Decode a hexadecimal string to a list of byte values.
fn decode_hex(hexa: &str) -> Vec<u8> {
    assert_eq!(
        hexa.len() % 2,
        0,
        "hex string must have length divisible by 2"
    );
    let mut bytes = Vec::with_capacity(hexa.len() / 2);
    for c in hexa.as_bytes().chunks(2) {
        let high = char::from(c[0])
            .to_digit(16)
            .expect("expected only hexadecimal characters");
        let low = char::from(c[1])
            .to_digit(16)
            .expect("expected only hexadecimal characters");
        bytes.push((high * 16 + low) as u8);
    }
    bytes
}

/// Check if a string contains only valid hexadecimal characters ([0-9A-Fa-f]).
fn is_hex_string(string: &str) -> bool {
    string.chars().all(|c| char::is_ascii_hexdigit(&c))
}

/// Find a list of hex code injections that the UserPatch installer does.
fn find_injections(exe: &[u8]) -> Result<Vec<Feature>> {
    // Keeps track of 32 bit and 8 bit literal PUSH calls _only_. These are the instructions
    // used to put addresses and sizes on the stack before calling feature and patch constructors.
    // There are more PUSH instructions but the things they push are not interesting.
    let mut stack_args = vec![];
    // Separately keep track of the most recently constructed string value. Some constructors take
    // a single string argument, but they PUSH a register to the stack, and that's harder to
    // follow. Since there aren't any functions that take multiple strings at this time, we can
    // just use the most recently seen one when we need it.
    let mut latest_string = String::new();

    // Initialize features with a "pseudo"-feature to collect some of the stray patches that are
    // not associated with any feature.
    let mut features: Vec<Feature> = vec![Feature {
        name: "Pre-patch".to_string(),
        optional: false,
        enabled_by_default: true,
        always_enabled: true,
        affects_sync: false,
        patches: vec![],
    }];

    // Add a patch to the current feature.
    let push_patch = |features: &mut Vec<Feature>, patch| {
        if let Some(feature) = features.last_mut() {
            feature.patches.push(patch);
        } else {
            panic!("could not add feature: {:?}", patch);
        }
    };

    let opcodes = lde::X86.iter(exe, CODE_BASE_ADDRESS);
    for (op, va) in opcodes {
        match op.read::<u8>(0) {
            ASM_CALL => {
                let relative_jmp = op.read::<u32>(1);
                let (target, _) = (va + 5).overflowing_add(relative_jmp);
                match target {
                    // CreateFeature(name, optional, enabled_by_default, always_enabled, affects_sync)
                    FEATURE_ADDRESS => {
                        stack_args.reverse();
                        features.push(Feature {
                            name: latest_string.clone(),
                            optional: stack_args[0] != 0,
                            enabled_by_default: stack_args[1] != 0,
                            always_enabled: stack_args[2] != 0,
                            affects_sync: stack_args[3] != 0,
                            patches: vec![],
                        });
                        latest_string.clear();
                    }
                    // CreateHexPatch(address, "HEXSTRING")
                    HEX_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let patch = read_c_str(exe, stack_args[1] - RDATA_BASE_ADDRESS);
                        let addr = stack_args[0];
                        assert!(is_hex_string(&patch), "unexpected non-hex string");
                        push_patch(&mut features, Patch::Hex(addr, decode_hex(&patch)));
                    }
                    // CreateBytePatch(address, byte_ptr, byte_len)
                    BYTE_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let start = (stack_args[1] - DATA_BASE_ADDRESS) as usize;
                        let patch = &exe[start..start + stack_args[2] as usize];
                        let addr = stack_args[0];
                        push_patch(&mut features, Patch::Hex(addr, patch.to_vec()));
                    }
                    // CreateHexPatch("Description of the patch", address, "HEXSTRING")
                    NAMED_HEX_PATCH_ADDRESS => {
                        if !latest_string.is_empty() {
                            push_patch(&mut features, Patch::Header(latest_string.clone()));
                        }
                        stack_args.reverse();
                        let patch = read_c_str(exe, stack_args[1] - RDATA_BASE_ADDRESS);
                        let addr = stack_args[0];
                        assert!(is_hex_string(&patch), "unexpected non-hex string");
                        push_patch(&mut features, Patch::Hex(addr, decode_hex(&patch)));
                    }
                    // CreateJmpOrCallPatch(address, target, padding, is_jmp)
                    JMP_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        let to_addr = stack_args[1];
                        let padding = stack_args[2];
                        push_patch(
                            &mut features,
                            if stack_args[3] != 0 {
                                Patch::Jmp(addr, to_addr, padding)
                            } else {
                                Patch::Call(addr, to_addr, padding)
                            },
                        );
                    }
                    // CreateString(address_of_utf16_string)
                    STRING_CONSTRUCTOR_ADDRESS16 if stack_args.len() > 0 => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        if addr > RDATA_BASE_ADDRESS {
                            latest_string = read_utf16_str(exe, addr - RDATA_BASE_ADDRESS);
                        } else {
                            latest_string = String::new();
                        }
                    }
                    // CreateString(address_of_ansi_string)
                    STRING_CONSTRUCTOR_ADDRESS => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        latest_string = read_c_str(exe, addr - RDATA_BASE_ADDRESS);
                    }
                    _ => {}
                }

                // Reset current arguments after processing each CALL
                stack_args.clear();
            }
            ASM_PUSH32 => stack_args.push(op.read::<u32>(1)),
            ASM_PUSH8 => stack_args.push(op.read::<u8>(1) as u32),
            _ => (),
        }
    }

    Ok(features)
}

/// Unpack a byte array with upx, returning the unpacked bytes.
///
/// It requires a upx executable accessible somewhere in the $PATH.
#[cfg(not(os = "windows"))]
fn upx_unpack(packed_bytes: &[u8], tempdir: &Path) -> Result<Vec<u8>> {
    fs::write(tempdir.join("packed.exe"), packed_bytes)?;
    let status = Command::new("upx")
        .arg("-d")
        .arg(format!(
            "-o{}",
            tempdir.join("unpacked.exe").to_str().unwrap()
        ))
        .arg(tempdir.join("packed.exe"))
        .status()
        .expect("could not run upx");
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "upx exited nonzero",
        ));
    }
    let result = fs::read(tempdir.join("unpacked.exe"))?;
    fs::remove_file(tempdir.join("packed.exe"))?;
    fs::remove_file(tempdir.join("unpacked.exe"))?;
    Ok(result)
}

/// TODO this can honestly probably do the same as the unix version for the most part
#[cfg(os = "windows")]
fn upx_unpack(packed_bytes: &[u8], tempdir: &Path) -> Result<Vec<u8>> {
    unimplemented!()
}

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut f = File::create(out_dir.join("injections.rs"))?;

    let packed_bytes = fs::read("resources/SetupAoC.exe")?;
    let bytes = upx_unpack(&packed_bytes, &out_dir)?;
    let features = find_injections(&bytes)?;
    let mut patch_definitions: Vec<Vec<u8>> = Vec::new();
    let mut features_definition: Vec<u8> = Vec::new();

    writeln!(
        &mut features_definition,
        "static FEATURES: [Feature; {}] = [",
        features.len()
    )?;

    fn serialize_jmp_or_call(
        mut f: impl std::io::Write,
        instr: u8,
        addr: u32,
        to_addr: u32,
        mut padding: u32,
    ) -> std::io::Result<()> {
        // Turn into a relative offset
        let bytes = to_addr.overflowing_sub(addr + 5).0.to_le_bytes();
        write!(
            f,
            "    Injection({:#x}, &[{:#04X}, {:#04X}, {:#04X}, {:#04X}, {:#04X}",
            addr, instr, bytes[0], bytes[1], bytes[2], bytes[3],
        )?;
        // Fill the rest up with NOPs, /((0x66){0,3} 0x90)*/
        while padding > 0 {
            let mut group = if padding > 4 { 4 } else { padding };
            while group > 1 {
                write!(f, ", 0x66")?;
                group -= 1;
            }
            write!(f, ", 0x90")?;
            padding = padding.saturating_sub(4);
        }
        writeln!(f, "]),\n")?;
        Ok(())
    }

    for feature in &features {
        let mut patch_group = Vec::new();
        for inject in &feature.patches {
            match inject {
                Patch::Header(name) => writeln!(&mut patch_group, "    // {}", name)?,
                Patch::Hex(addr, patch) => {
                    write!(
                        &mut patch_group,
                        "    Injection({:#x}, &[{:#04X}",
                        addr, patch[0]
                    )?;
                    for b in patch.iter().skip(1) {
                        write!(&mut patch_group, ", {:#04X}", b)?;
                    }
                    writeln!(&mut patch_group, "]),")?;
                }
                Patch::Call(addr, to_addr, padding) => {
                    serialize_jmp_or_call(&mut patch_group, ASM_CALL, *addr, *to_addr, *padding)?
                }
                Patch::Jmp(addr, to_addr, padding) => {
                    serialize_jmp_or_call(&mut patch_group, ASM_JMP, *addr, *to_addr, *padding)?
                }
            }
        }
        patch_definitions.push(patch_group);
        writeln!(
            &mut features_definition,
            "    Feature {{ name: \"{}\", optional: {:?}, affects_sync: {:?}, patches: &PATCH_GROUP_{}, enabled: {:?} }},",
            feature.name,
            feature.optional,
            feature.affects_sync,
            patch_definitions.len() - 1,
            feature.enabled_by_default
        )?;
    }
    writeln!(&mut features_definition, "];")?;

    for (i, text) in patch_definitions.iter().enumerate() {
        writeln!(
            f,
            "static PATCH_GROUP_{}: [Injection; {}] = [",
            i,
            features[i]
                .patches
                .iter()
                .fold(0, |acc, p| if let Patch::Header(_) = p {
                    acc
                } else {
                    acc + 1
                })
        )?;
        f.write_all(&text)?;
        writeln!(f, "];")?;
    }

    f.write_all(&features_definition)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hex_string_test() {
        assert_eq!(is_hex_string("ABCDEF"), true);
        assert_eq!(is_hex_string("ABCDEFG"), true);
        assert_eq!(is_hex_string("123456"), false);
        assert_eq!(is_hex_string("whatever"), false);
    }

    #[test]
    fn decode_hex_test() {
        assert_eq!(decode_hex("ABCDEF"), vec![0xAB_u8, 0xCD_u8, 0xEF_u8]);
        assert_eq!(decode_hex("123456"), vec![0x12_u8, 0x34_u8, 0x56_u8]);
    }
}
