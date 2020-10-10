use ::VTIL::{BasicBlock, Instruction, Operand, RegisterFlags, Result, VTILReader};
use ansi_term::Colour::{Blue, Yellow};
use std::env;

fn is_volatile(instr: &Instruction) -> bool {
    instr.name() == "sfence"
        || instr.name() == "lfence"
        || instr.name() == "vemit"
        || instr.name() == "vpinr"
        || instr.name() == "vpinw"
        || instr.name() == "vpinrm"
        || instr.name() == "vpinwm"
}

fn dump_instr(instr: &Instruction, prev_instr: Option<&Instruction>) {
    if instr.sp_index() != 0 {
        print!("[{}] ", format!("{}", instr.sp_index()));
    } else {
        print!("    ");
    }

    if instr.sp_reset() {
        print!(
            ">{}{:-#4x} ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        );
    } else if prev_instr.as_ref().map(|i| i.sp_offset()).unwrap_or(0) == instr.sp_offset() {
        print!(
            "{}{:-#4x}  ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        );
    } else if prev_instr.as_ref().map(|i| i.sp_offset()).unwrap_or(0) >= instr.sp_offset() {
        print!(
            "{}{:-#4x}  ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        );
    } else {
        print!(
            "{}{:-#4x}  ",
            if instr.sp_offset() >= 0 { '+' } else { '-' },
            instr.sp_offset().abs()
        );
    }

    if is_volatile(instr) {
        print!("{:-8} ", instr.name());
    } else {
        print!("{:-8} ", instr.name());
    }

    for op in instr.operands() {
        match op {
            Operand::Reg(r) => {
                if r.flags().contains(RegisterFlags::STACK_POINTER) {
                    print!("{:-12} ", r);
                } else if r.flags().contains(RegisterFlags::PHYSICAL) {
                    print!("{:-12} ", r);
                } else {
                    print!("{:-12} ", r);
                }
            }
            Operand::Imm(i) => {
                if i.i64() >= 0 {
                    print!("{:-#12x} ", i.i64());
                } else {
                    print!("{:-#12x} ", i.i64());
                }
            }
        }
    }

    println!()
}

fn dump_routine(basic_blocks: &Vec<BasicBlock>) {
    for basic_block in basic_blocks {
        print!("Entry point VIP:       ");
        println!("{:#x}", basic_block.vip().0);
        print!("Stack pointer:         ");
        if basic_block.sp_offset() < 0 {
            println!("{:x}", basic_block.sp_offset());
        }

        let mut prev_instr = None;
        for instr in basic_block.instructions() {
            dump_instr(instr, prev_instr);
            prev_instr = Some(instr);
        }
    }
}

fn main() -> Result<()> {
    let mut argv = env::args();
    let routine = VTILReader::from_path(argv.nth(1).unwrap())?;
    dump_routine(routine.explored_blocks());
    Ok(())
}
