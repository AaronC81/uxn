use std::{env::args, fs::File, io::Read};

use uxn_core_emulator::{device::VarvaraDevice, Core};

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
            |00 @System &vector $2 &expansion $2 &wst $1 &rst $1 &metadata $2 &r $2 &g $2 &b $2 &debug $1 &state $1
            |10 @Console [ &vector $2 &read $1 &pad $5 &write $1 &error $1 ]
            |20 @Screen [ &vector $2 &width $2 &height $2 &auto $2 &x $2 &y $2 &addr $2 &pixel $1 &sprite $1 ]
        
            |0100 

            @on-reset ( -> )
                ;on-screen .Screen/vector DEO2
                #0320 .Screen/width  DEO2 ( 800px )
                #0258 .Screen/height DEO2 ( 600px )

                #af00 .System/r DEO2
                #0f00 .System/b DEO2
                #0f00 .System/g DEO2

                ;hello_world_str
                &print_loop
                    LDAk                    ( Load pointed character )
                    .Console/write DEO      ( Print it )
                    INC                     ( Increment pointer )
                    LDAk ,&print_loop JCN   ( If it's non-zero, iterate again )
                POP                         ( Drop pointer once we're done )
            BRK

            @on-screen ( -> )
                ;counter LDA INC
                DUP #20 NEQ ,&skip_forward JCN [ #0f00 .System/r DEO2          ] &skip_forward
                DUP #40 NEQ ,&skip_back    JCN [ #af00 .System/r DEO2  POP #00 ] &skip_back
                ;counter STA

                ( Also paint a white pixel )
                #0100 .Screen/x DEO2
                #0100 .Screen/y DEO2
                #01 .Screen/pixel DEO
            BRK

            @counter 00

            @hello_world_str "Hello 2c 20 "World 21 0a $1
        "#);
    }

    core.set_device(VarvaraDevice::new());
    core.execute_until_exit();
}
