use std::io::{Result, Write};

static setupaoc_exe: &'static [u8] = include_bytes!("../resources/SetupAoC.exe");

pub fn extract_installer<W>(output: &mut W) -> Result<()>
    where W: Write
{
    output.write_all(setupaoc_exe)
}
