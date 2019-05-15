use std::{
    env,
    fs::{self, File},
    io::{Result, Write},
    path::{Path, PathBuf},
    process::Command,
};
use encoding_rs::UTF_16LE;

/// The location of the PatchData hex string overload in memory space.
const HEX_PATCH_ADDRESS: u32 = 0x00402750;
/// The location of the PatchData hex string + name overload in memory space.
const NAMED_HEX_PATCH_ADDRESS: u32 = 0x004023E0;
/// The location of the PatchData bytes oveerload in memory space.
const BYTE_PATCH_ADDRESS: u32 = 0x00402AF0;
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

enum Patch {
    Header(String),
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
        .flat_map(|byte| vec![
            to_hex_char((byte & 0xF0) >> 4),
            to_hex_char(byte & 0x0F),
        ])
        .collect::<String>()
}

/// Check if a string contains only valid hexadecimal characters ([0-9A-Fa-f]).
fn is_hex_string(string: &str) -> bool {
    string.chars().all(|c| char::is_ascii_hexdigit(&c))
}

/// Find a list of hex code injections that the UserPatch installer does.
fn find_injections(exe: &[u8]) -> Result<Vec<Patch>> {
    let mut injections = vec![];
    let mut stack_args = vec![];
    let mut latest_named = String::new();

    for (op, va) in lde::X86.iter(exe, CODE_BASE_ADDRESS) {
        match op.read::<u8>(0) {
            ASM_CALL => {
                let (target, _) = (va + 5).overflowing_add(op.read::<u32>(1));
                if target == HEX_PATCH_ADDRESS {
                    stack_args.reverse();
                    let patch = read_c_str(exe, stack_args[1] - DATA_BASE_ADDRESS);
                    let addr = stack_args[0];
                    assert!(is_hex_string(&patch), "unexpected non-hex string");
                    injections.push(Patch::Hex(addr, patch));
                }
                if target == BYTE_PATCH_ADDRESS {
                    stack_args.reverse();
                    let start = (stack_args[1] - DATA_BASE_ADDRESS) as usize;
                    let patch = &exe[start..start + stack_args[2] as usize];
                    let addr = stack_args[0];
                    injections.push(Patch::Hex(stack_args[1] - DATA_BASE_ADDRESS, to_hex(patch)));
                }
                if target == NAMED_HEX_PATCH_ADDRESS {
                    if !latest_named.is_empty() {
                        injections.push(Patch::Header(latest_named.clone()));
                    }
                    stack_args.reverse();
                    let patch = read_c_str(exe, stack_args[1] - DATA_BASE_ADDRESS);
                    let addr = stack_args[0];
                    assert!(is_hex_string(&patch), "unexpected non-hex string");
                    injections.push(Patch::Hex(addr, patch));
                }
                if target == STRING_CONSTRUCTOR_ADDRESS16 && stack_args.len() > 0 {
                    stack_args.reverse();
                    let addr = stack_args[0];
                    if addr > DATA_BASE_ADDRESS {
                        latest_named = read_utf16_str(exe, addr - DATA_BASE_ADDRESS);
                    } else {
                        latest_named = String::new();
                    }
                }
                if target == STRING_CONSTRUCTOR_ADDRESS {
                    stack_args.reverse();
                    let addr = stack_args[0];
                    latest_named = read_c_str(exe, addr - DATA_BASE_ADDRESS);
                }
                stack_args.clear();
            }
            ASM_PUSH32 => stack_args.push(op.read::<u32>(1)),
            ASM_PUSH8 => stack_args.push(op.read::<u8>(1) as u32),
            _ => (),
        }
    }

    Ok(injections)
}

#[cfg(not(os = "windows"))]
fn upx_unpack(packed_bytes: &[u8], tempdir: &Path) -> Result<Vec<u8>> {
    fs::write(tempdir.join("packed.exe"), packed_bytes)?;
    let status = Command::new("upx")
        .arg("-d")
        .arg(format!("-o{}", tempdir.join("unpacked.exe").to_str().unwrap()))
        .arg(tempdir.join("packed.exe"))
        .status()
        .expect("could not run upx");
    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "upx exited nonzero"));
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
    let injections = find_injections(&bytes)?;
    write!(f, "&[\n")?;
    for inject in &injections {
        match inject {
            Patch::Header(name) => write!(f, "  // {}\n", name)?,
            Patch::Hex(addr, patch) => write!(f, "  Injection({:#x}, \"{}\"),\n", addr, patch)?,
        }
    }
    write!(f, "]")?;
    Ok(())
}
