use uxn_core_emulator::Core;

fn main() {
    let mut core = Core::new_with_uxntal(r#"
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
    core.execute_until_break();
}
