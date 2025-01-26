// A number of these test cases are taken from the examples on the uxntal reference:
//   https://wiki.xxiivv.com/site/uxntal_reference.html

use std::str;

use crate::Core;

#[test]
fn test_inc() {
    assert_eq!(execute("#01 INC BRK"), [2]); // Byte mode
    assert_eq!(execute("#00ff INC2 BRK"), [01, 00]); // Short mode
    assert_eq!(execute("#00ff INC2k BRK"), [00, 0xff, 01, 00]); // Keep mode
}

#[test]
fn test_jmp() {
    assert_eq!(execute("#01 #02 ,&skip-rel JMP BRK BRK BRK &skip-rel #03"), [1, 2, 3]); // Relative mode
    assert_eq!(execute("#01 #02 ;&skip-abs JMP2 BRK BRK BRK &skip-abs #03"), [1, 2, 3]); // Absolute mode
}

#[test]
fn test_jcn() {
    assert_eq!(execute("#01 ,&true JCN ,&false JMP  &true #42 BRK  &false #ff BRK"), [0x42]); // True
    assert_eq!(execute("#00 ,&true JCN ,&false JMP  &true #42 BRK  &false #ff BRK"), [0xff]); // False
}

#[test]
fn test_ldr() {
    assert_eq!(execute(",cell LDR BRK @cell 12"), [0x12]); // Byte
    assert_eq!(execute(",cell LDR2 BRK @cell abcd"), [0xab, 0xcd]); // Short
}

#[test]
fn test_sft() {
    assert_eq!(execute("#34 #10 SFT BRK"), [0x68]);
    assert_eq!(execute("#34 #01 SFT BRK"), [0x1a]);
    assert_eq!(execute("#1248 #34 SFTk2 BRK"), [0x12, 0x48, 0x34, 0x09, 0x20]);
}

fn execute(code: &str) -> Vec<u8> {
    let mut core = Core::new_with_uxntal(code);
    core.execute_until_break();
    core.working_stack.bytes().to_vec()
}
