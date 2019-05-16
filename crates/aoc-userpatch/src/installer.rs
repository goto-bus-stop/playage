use std::io::{Result, Write};

static SETUPAOC_EXE: &'static [u8] = include_bytes!("../resources/SetupAoC.exe");

pub fn extract_installer<W>(output: &mut W) -> Result<()>
where
    W: Write,
{
    output.write_all(SETUPAOC_EXE)
}
