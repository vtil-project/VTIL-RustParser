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

use std::fmt::Write;
use vtil_parser::{Instruction, Operand, Result};

pub fn dump_instr(instr: &Instruction) -> Result<String> {
    let mut buffer = String::new();

    if instr.sp_index() != 0 {
        write!(buffer, "[{:04}] ", instr.sp_index())?;
    } else {
        write!(buffer, "       ")?;
    }

    if instr.sp_reset() {
        write!(
            buffer,
            ">{}{:>#4x} ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        )?;
    } else {
        write!(
            buffer,
            " {}{:>#4x} ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        )?;
    }

    write!(buffer, "{:<8} ", instr.name())?;

    for op in instr.operands() {
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
