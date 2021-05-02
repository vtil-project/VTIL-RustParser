// Copyright © 2020-2021 Keegan Saunders
//
// Permission to use, copy, modify, and/or distribute this software for
// any purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
// AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
// OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use std::env;
use vtil_parser::{BasicBlock, Instruction, Operand, RegisterFlags, Result, VTILReader};

mod shared;
use shared::dump_instr;

fn dump_routine(basic_blocks: &Vec<BasicBlock>) {
    for basic_block in basic_blocks {
        println!("Entry point VIP:       {:#x}", basic_block.vip().0);
        println!("Stack pointer:         {:x}", basic_block.sp_offset());

        for instr in basic_block.instructions() {
            println!("{}", dump_instr(instr).unwrap());
        }
    }
}

fn main() -> Result<()> {
    let mut argv = env::args();
    let routine = VTILReader::from_path(argv.nth(1).unwrap())?;
    dump_routine(routine.explored_blocks());
    Ok(())
}
