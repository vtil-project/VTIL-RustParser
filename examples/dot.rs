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

    write!(buffer, "{:<8} ", instr.op.name())?;

    for op in instr.op.operands() {
        match op {
            Operand::RegisterDesc(r) => {
                write!(buffer, "{:<12}", format!("{}", r))?;
            }
            Operand::ImmediateDesc(i) => {
                write!(buffer, "{:<#12x}", i.i64())?;
            }
        }
    }

    Ok(buffer)
}

fn escape(data: String) -> String {
    data.replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("|", "\\|")
}

fn dump_routine(basic_blocks: &Vec<BasicBlock>) {
    println!("digraph G {{");

    for basic_block in basic_blocks {
        let pc = basic_block.vip.0;

        println!(
            r#"vip_{0:x} [
    shape="Mrecord"
    fontname="Courier New"
    label=<
        <table border="0" cellborder="0" cellpadding="3">
            <tr><td align="center" colspan="2" bgcolor="grey">{0:x}</td></tr>"#,
            pc
        );

        for instr in &basic_block.instructions {
            let pretty = dump_instr(instr).unwrap();
            println!(
                r#"            <tr><td align="left">{}</td></tr>"#,
                escape(pretty)
            );
        }

        println!(
            r#"        </table>
    >
];"#
        );

        let successors = &basic_block.next_vip;
        if successors.len() == 2 {
            println!(
                r#"vip_{:x} -> vip_{:x} [color="green"];"#,
                pc, successors[0].0
            );
            println!(
                r#"vip_{:x} -> vip_{:x} [color="red"];"#,
                pc, successors[1].0
            );
        } else {
            for successor in successors {
                println!(r#"vip_{:x} -> vip_{:x} [color="blue"];"#, pc, successor.0);
            }
        }
    }

    println!("}}");
}

fn main() -> Result<()> {
    let mut argv = env::args();
    let routine = VTILReader::from_path(argv.nth(1).unwrap())?;
    dump_routine(&routine.explored_blocks);
    Ok(())
}
