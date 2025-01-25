use std::{env::args, fs::File, io::Read};

use uxn_core_emulator::Core;

fn main() {
    // Current interface:
    //   - If this has an argument, assume it's a ROM, and load it
    //   - Otherwise, run some hardcoded text
    //
    // Keeping the latter means I can try Varvara stuff quickly.
    // TODO: tidy this up at some point

    let mut core;
    if args().len() > 1 {
        let rom_path = args().nth(1).unwrap();
        let mut rom_data = vec![];

        File::open(rom_path).unwrap().read_to_end(&mut rom_data).unwrap();

        core = Core::new_with_rom(&rom_data);
    } else {
        core = Core::new_with_uxntal(r#"
            |10 @Console [ &vector $2 &read $1 &pad $5 &write $1 &error $1 ]

            |0100 ( -- )
                ;hello_world_str
                &print_loop
                    LDAk                    ( Load pointed character )
                    .Console/write DEO      ( Print it )
                    INC                     ( Increment pointer )
                    LDAk ,&print_loop JCN   ( If it's non-zero, iterate again )
                POP                         ( Drop pointer once we're done )
            BRK

            @hello_world_str "Hello 2c 20 "World 21 0a $1
        "#);
    }

    core.execute_until_break();
}
