use std::{
    env,
    fs::{self, File},
    io::{Result, Write},
    path::PathBuf,
};

/// The location of the apply_hex_patch function in memory space.
const APPLY_HEX_PATCH_ADDRESS: u32 = 0x00402750;

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

struct Injection(pub u32, pub String);

/// Read a NUL-terminated string from a byte slice.
fn read_c_str(bytes: &[u8], start: u32) -> String {
    let str_bytes = (&bytes[start as usize..])
        .iter()
        .cloned()
        .take_while(|&c| c != 0)
        .collect::<Vec<u8>>();

    String::from_utf8(str_bytes).unwrap()
}

/// Check if a string contains only valid hexadecimal characters ([0-9A-Fa-f]).
fn is_hex_string(string: &str) -> bool {
    string.chars().all(|c| char::is_ascii_hexdigit(&c))
}

/// Find a list of hex code injections that the UserPatch installer does.
fn find_injections() -> Result<Vec<Injection>> {
    let mut injections = vec![];
    let mut stack_args = vec![];

    let exe = fs::read("resources/SetupAoC.exe")?;
    for (op, va) in lde::X86.iter(&exe, CODE_BASE_ADDRESS) {
        match op.read::<u8>(0) {
            ASM_CALL => {
                let (target, _) = (va + 5).overflowing_add(op.read::<u32>(1));
                if target == APPLY_HEX_PATCH_ADDRESS {
                    stack_args.reverse();
                    let patch = read_c_str(&exe, stack_args[1] - DATA_BASE_ADDRESS);
                    let addr = stack_args[0];
                    assert!(is_hex_string(&patch), "unexpected non-hex string");
                    injections.push(Injection(addr, patch));
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

fn main() -> Result<()> {
    let path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut f = File::create(path.join("injections.rs"))?;
    let mut injections = find_injections()?;
    injections.sort_by_key(|a| a.0);
    write!(f, "&[\n")?;
    for Injection(addr, patch) in &injections {
        write!(f, "  Injection({:#x}, \"{}\"),\n", addr, patch)?;
    }
    write!(f, "]")?;
    Ok(())
}
