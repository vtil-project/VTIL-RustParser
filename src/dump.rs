use crate::{Instruction, Operand, Result, Routine, Vip};
use std::io;

/// Dump a VTIL [`Instruction`] to a [`String`]. This format is **not** stable
pub fn dump_instr(buffer: &mut dyn io::Write, instr: &Instruction) -> Result<()> {
    if instr.vip != Vip::invalid() {
        write!(buffer, "[{:08x}] ", instr.vip.0)?;
    } else {
        write!(buffer, "[ PSEUDO ] ")?;
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
                if i.i64() < 0 {
                    write!(buffer, "-{:<#12x}", -i.i64())?;
                } else {
                    write!(buffer, "{:<#12x}", i.i64())?;
                }
            }
        }
    }

    Ok(())
}

/// Dump a VTIL [`Routine`] to a [`String`]. This format is **not** stable
pub fn dump_routine(buffer: &mut dyn io::Write, routine: &Routine) -> Result<()> {
    for (_, basic_block) in &routine.explored_blocks {
        writeln!(buffer, "Entry point VIP:       {:#x}", basic_block.vip.0)?;
        write!(buffer, "Stack pointer:         ")?;
        if basic_block.sp_offset < 0 {
            writeln!(buffer, "-{:#x}", -basic_block.sp_offset)?;
        } else {
            writeln!(buffer, "{:#x}", basic_block.sp_offset)?;
        }

        for instr in &basic_block.instructions {
            dump_instr(buffer, instr)?;
            writeln!(buffer)?;
        }
    }

    Ok(())
}
