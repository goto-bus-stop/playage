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
const STRING_CONSTRUCTOR_ADDRESS: u32 = 0x004AA9C0;
const STRING_CONSTRUCTOR_ADDRESS16: u32 = 0x004AA8F0;

/// Base address of the code section.
const CODE_BASE_ADDRESS: u32 = 0x00400C00;
/// Base address of the data section.
const DATA_BASE_ADDRESS: u32 = 0x00401600;

/// Opcode for `call` instructions.
const ASM_CALL: u8 = 0xE8;
/// Opcode for `push` instructions with 32 bit operand.
const ASM_PUSH32: u8 = 0x68;
/// Opcode for `push` instructions with 8 bit operand.
const ASM_PUSH8: u8 = 0x6A;

struct Feature {
    name: String,
    optional: bool,
    enabled_by_default: bool,
    always_enabled: bool,
    affects_sync: bool,
    patches: Vec<Patch>,
}

#[derive(Debug)]
enum Patch {
    Header(String),
    Jmp(u32, u32),
    Hex(u32, String),
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

fn to_hex(bytes: &[u8]) -> String {
    fn to_hex_char(c: u8) -> char {
        match c {
            0xA...0xF => char::from(b'A' - 10 + c),
            0x0...0x9 => char::from(b'0' + c),
            _ => panic!("expected number to be 4 bits, got {}", c),
        }
    }

    bytes
        .iter()
        .flat_map(|byte| vec![to_hex_char((byte & 0xF0) >> 4), to_hex_char(byte & 0x0F)])
        .collect::<String>()
}

/// Check if a string contains only valid hexadecimal characters ([0-9A-Fa-f]).
fn is_hex_string(string: &str) -> bool {
    string.chars().all(|c| char::is_ascii_hexdigit(&c))
}

/// Find a list of hex code injections that the UserPatch installer does.
fn find_injections(exe: &[u8]) -> Result<Vec<Feature>> {
    let mut stack_args = vec![];
    let mut latest_named = String::new();
    let mut features: Vec<Feature> = vec![Feature {
        name: "Pre-patch".to_string(),
        optional: false,
        enabled_by_default: true,
        always_enabled: true,
        affects_sync: false,
        patches: vec![],
    }];

    let push_patch = |features: &mut Vec<Feature>, patch| {
        if let Some(feature) = features.last_mut() {
            feature.patches.push(patch);
        } else {
            panic!("could not add feature: {:?}", patch);
        }
    };

    for (op, va) in lde::X86.iter(exe, CODE_BASE_ADDRESS) {
        match op.read::<u8>(0) {
            ASM_CALL => {
                let (target, _) = (va + 5).overflowing_add(op.read::<u32>(1));
                match target {
                    FEATURE_ADDRESS => {
                        stack_args.reverse();
                        features.push(Feature {
                            name: latest_named.clone(),
                            optional: stack_args[0] != 0,
                            enabled_by_default: stack_args[1] != 0,
                            always_enabled: stack_args[2] != 0,
                            affects_sync: stack_args[3] != 0,
                            patches: vec![],
                        });
                        latest_named.clear();
                    }
                    HEX_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let patch = read_c_str(exe, stack_args[1] - DATA_BASE_ADDRESS);
                        let addr = stack_args[0];
                        assert!(is_hex_string(&patch), "unexpected non-hex string");
                        push_patch(&mut features, Patch::Hex(addr, patch));
                    }
                    BYTE_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let start = (stack_args[1] - DATA_BASE_ADDRESS) as usize;
                        let patch = &exe[start..start + stack_args[2] as usize];
                        let addr = stack_args[0];
                        push_patch(
                            &mut features,
                            Patch::Hex(stack_args[1] - DATA_BASE_ADDRESS, to_hex(patch)),
                        );
                    }
                    NAMED_HEX_PATCH_ADDRESS => {
                        if !latest_named.is_empty() {
                            push_patch(&mut features, Patch::Header(latest_named.clone()));
                        }
                        stack_args.reverse();
                        let patch = read_c_str(exe, stack_args[1] - DATA_BASE_ADDRESS);
                        let addr = stack_args[0];
                        assert!(is_hex_string(&patch), "unexpected non-hex string");
                        push_patch(&mut features, Patch::Hex(addr, patch));
                    }
                    JMP_PATCH_ADDRESS => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        let to_addr = stack_args[1];
                        push_patch(&mut features, Patch::Jmp(addr, to_addr));
                    }
                    STRING_CONSTRUCTOR_ADDRESS16 if stack_args.len() > 0 => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        if addr > DATA_BASE_ADDRESS {
                            latest_named = read_utf16_str(exe, addr - DATA_BASE_ADDRESS);
                        } else {
                            latest_named = String::new();
                        }
                    }
                    STRING_CONSTRUCTOR_ADDRESS => {
                        stack_args.reverse();
                        let addr = stack_args[0];
                        latest_named = read_c_str(exe, addr - DATA_BASE_ADDRESS);
                    }
                    _ => {}
                }
                stack_args.clear();
            }
            ASM_PUSH32 => stack_args.push(op.read::<u32>(1)),
            ASM_PUSH8 => stack_args.push(op.read::<u8>(1) as u32),
            _ => (),
        }
    }

    Ok(features)
}

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


    write!(&mut features_definition, "static ref FEATURES: Vec<Feature> = vec![\n")?;
    for feature in &features {
        let mut patch_group = Vec::new();
        for inject in &feature.patches {
            match inject {
                Patch::Header(name) => write!(&mut patch_group, "    // {}\n", name)?,
                Patch::Hex(addr, patch) => {
                    write!(&mut patch_group, "    Injection({:#x}, \"{}\"),\n", addr, patch)?
                }
                Patch::Jmp(addr, to_addr) => write!(
                    &mut patch_group,
                    "    Injection({:#x}, \"E9{:08X}\"),\n",
                    addr,
                    to_addr.overflowing_sub(addr + 5).0.to_be()
                )?,
            }
        }
        patch_definitions.push(patch_group);
        write!(&mut features_definition, "  // {:?}\n  Feature {{ name: \"{}\", optional: {:?}, affects_sync: {:?}, patches: &PATCH_GROUP_{}\n", feature.always_enabled, feature.name, feature.optional, feature.affects_sync, patch_definitions.len() - 1)?;
        write!(&mut features_definition, ", enabled: {} }},\n", feature.enabled_by_default)?;
    }
    write!(&mut features_definition, "];\n")?;

    for (i, text) in patch_definitions.iter().enumerate() {
        write!(f, "static PATCH_GROUP_{}: [Injection; {}] = [\n", i, features[i].patches.iter().fold(0, |acc, p| if let Patch::Header(_) = p { acc } else { acc + 1 }))?;
        f.write_all(&text)?;
        write!(f, "];\n")?;
    }

    write!(f, "lazy_static::lazy_static! {{\n");

    f.write_all(&features_definition)?;

    write!(f, "}}\n")?;

    Ok(())
}
