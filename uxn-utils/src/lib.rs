#![feature(exit_status_error)]

use std::{error::Error, io::{Read, Write}, process::Command};

use tempfile::NamedTempFile;

/// Assembles uxntal code using the `uxnasm` command-line tool, which must be on your PATH.
/// 
/// Returns the sequence of bytes of the ROM.
/// This should be loaded at 0x0100 in an uxn interpreter.
/// 
/// Returns an error if `uxnasm` is not on your PATH, or if assembly fails.
pub fn assemble_uxntal(code: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    // Write code to a file
    let mut code_file = NamedTempFile::new()?;
    write!(code_file, "{}", code)?;

    // Execute `uxnasm` to write to a new ROM file
    let mut rom_file = NamedTempFile::new()?;
    Command::new("uxnasm")
        .arg(code_file.path())
        .arg(rom_file.path())
        .status()?
        .exit_ok()?;

    // Read ROM out of file
    let mut bytes = vec![];
    rom_file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod test {
    use crate::assemble_uxntal;

    #[test]
    fn test_asm() {
        let rom = assemble_uxntal("|100 01 02 03").unwrap();
        assert_eq!(rom, vec![1, 2, 3])
    }
}
