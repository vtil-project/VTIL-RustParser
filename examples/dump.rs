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

use std::{env, fmt::Write};
use vtil_parser::{BasicBlock, Instruction, Operand, Result, VTILReader};

pub fn dump_instr(instr: &Instruction) -> Result<String> {
    let mut buffer = String::new();

    if instr.sp_index != 0 {
        write!(buffer, "[{:04}] ", instr.sp_index)?;
    } else {
        write!(buffer, "       ")?;
    }

    if instr.sp_reset {
        write!(
            buffer,
            ">{}{:>#4x} ",
            if instr.sp_offset >= 0 { '+' } else { '-' },
            instr.sp_offset.abs()
        )?;
    } else {
        write!(
            buffer,
            " {}{:>#4x} ",
            if instr.sp_offset >= 0 { '+' } else { '-' },
            instr.sp_offset.abs()
        )?;
    }

    write!(buffer, "{:<8} ", instr.name)?;

    for op in &instr.operands {
        match op {
            Operand::Reg(r) => {
                write!(buffer, "{:<12}", format!("{}", r))?;
            }
            Operand::Imm(i) => {
                write!(buffer, "{:<#12x}", i.i64())?;
            }
        }
    }

    Ok(buffer)
}

fn dump_routine(basic_blocks: &Vec<BasicBlock>) {
    for basic_block in basic_blocks {
        println!("Entry point VIP:       {:#x}", basic_block.vip.0);
        println!("Stack pointer:         {:x}", basic_block.sp_offset);

        for instr in &basic_block.instructions {
            println!("{}", dump_instr(instr).unwrap());
        }
    }
}

fn main() -> Result<()> {
    let mut argv = env::args();
    let routine = VTILReader::from_path(argv.nth(1).unwrap())?;
    dump_routine(&routine.explored_blocks);
    Ok(())
}
