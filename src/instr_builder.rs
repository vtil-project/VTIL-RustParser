// BSD 3-Clause License
//
// Copyright © 2021 Keegan Saunders
// Copyright © 2021 VTIL Project
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//

use crate::{
    BasicBlock, ImmediateDesc, Instruction, Op, Operand, RegisterDesc, RegisterFlags, Vip,
};
use std::convert::TryInto;

const VTIL_ARCH_POPPUSH_ENFORCED_STACK_ALIGN: usize = 2;

/// Builder for VTIL instructions in an associated [`BasicBlock`]
pub struct InstructionBuilder<'a> {
    /// Insertion point, *must* be cleared after use
    pub vip: Vip,
    /// The current [`BasicBlock`]
    pub basic_block: &'a mut BasicBlock,
}

// Helper for inserting instructions with no associated metadata
fn insert_instr(builder: &mut InstructionBuilder, op: Op) {
    let vip = if builder.vip != Vip::invalid() {
        let vip = builder.vip;
        builder.vip = Vip::invalid();
        vip
    } else {
        builder.vip
    };

    let sp_offset = builder.basic_block.sp_offset;
    let sp_index = builder.basic_block.sp_index;

    builder.basic_block.instructions.push(Instruction {
        op,
        vip,
        sp_offset,
        sp_index,
        sp_reset: false,
    });
}

impl<'a> InstructionBuilder<'a> {
    /// Build an [`InstructionBuilder`] from an existing [`BasicBlock`]
    pub fn from(basic_block: &'a mut BasicBlock) -> InstructionBuilder<'a> {
        InstructionBuilder {
            vip: Vip::invalid(),
            basic_block,
        }
    }

    /// Queues a stack shift
    pub fn shift_sp(&mut self, offset: i64) {
        self.basic_block.sp_offset += offset;
    }

    /// Pushes an operand up the stack queueing the shift in the stack pointer
    pub fn push(&mut self, op1: Operand) -> &mut Self {
        if let Operand::RegisterDesc(sp) = op1 {
            if sp.flags.contains(RegisterFlags::STACK_POINTER) {
                let tmp0 = self.basic_block.tmp(64);
                self.mov(tmp0, op1).push(tmp0.into());
                return self;
            }
        }

        let misalignment = (op1.size() % VTIL_ARCH_POPPUSH_ENFORCED_STACK_ALIGN) as i64;
        if misalignment != 0 {
            let padding_size = VTIL_ARCH_POPPUSH_ENFORCED_STACK_ALIGN as i64 - misalignment;
            self.shift_sp(-padding_size);
            self.str(
                RegisterDesc::SP,
                self.basic_block.sp_offset.into(),
                ImmediateDesc::new(0u64, TryInto::<u32>::try_into(padding_size).unwrap() * 8)
                    .into(),
            );
        }

        self.shift_sp(-(op1.size() as i64));
        self.str(RegisterDesc::SP, self.basic_block.sp_offset.into(), op1);

        self
    }

    /// Pops an operand from the stack queueing the shift in the stack pointer
    pub fn pop(&mut self, op1: RegisterDesc) -> &mut Self {
        let offset = self.basic_block.sp_offset;

        let misalignment = (op1.size() % VTIL_ARCH_POPPUSH_ENFORCED_STACK_ALIGN) as i64;
        if misalignment != 0 {
            self.shift_sp(VTIL_ARCH_POPPUSH_ENFORCED_STACK_ALIGN as i64 - misalignment);
        }

        self.shift_sp(op1.size() as i64);
        self.ldd(op1, RegisterDesc::SP, offset.into());

        self
    }

    /// Push flags register
    pub fn pushf(&mut self) -> &mut Self {
        self.push(RegisterDesc::FLAGS.into())
    }

    /// Pop flags register
    pub fn popf(&mut self) -> &mut Self {
        self.push(RegisterDesc::FLAGS.into())
    }

    /// Insert an [`Op::Mov`]
    pub fn mov(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Mov(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Movsx`]
    pub fn movsx(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Movsx(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Str`]
    pub fn str(&mut self, op1: RegisterDesc, op2: ImmediateDesc, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Str(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Ldd`]
    pub fn ldd(&mut self, op1: RegisterDesc, op2: RegisterDesc, op3: ImmediateDesc) -> &mut Self {
        insert_instr(self, Op::Ldd(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Neg`]
    pub fn neg(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Neg(op1.into()));
        self
    }

    /// Insert an [`Op::Add`]
    pub fn add(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Add(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Sub`]
    pub fn sub(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Sub(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Mul`]
    pub fn mul(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Mul(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Mulhi`]
    pub fn mulhi(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Mulhi(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Imul`]
    pub fn imul(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Imul(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Imulhi`]
    pub fn imulhi(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Imulhi(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Div`]
    pub fn div(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Div(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Rem`]
    pub fn rem(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Rem(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Idiv`]
    pub fn idiv(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Idiv(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Irem`]
    pub fn irem(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Irem(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Popcnt`]
    pub fn popcnt(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Popcnt(op1.into()));
        self
    }

    /// Insert an [`Op::Bsf`]
    pub fn bsf(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Bsf(op1.into()));
        self
    }

    /// Insert an [`Op::Bsr`]
    pub fn bsr(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Bsr(op1.into()));
        self
    }

    /// Insert an [`Op::Not`]
    pub fn not(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Not(op1.into()));
        self
    }

    /// Insert an [`Op::Shr`]
    pub fn shr(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Shr(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Shl`]
    pub fn shl(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Shl(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Xor`]
    pub fn xor(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Xor(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Or`]
    pub fn or(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Or(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::And`]
    pub fn and(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::And(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Ror`]
    pub fn ror(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Ror(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Rol`]
    pub fn rol(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self, Op::Rol(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Tg`]
    pub fn tg(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tg(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tge`]
    pub fn tge(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tge(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Te`]
    pub fn te(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Te(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tne`]
    pub fn tne(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tne(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tl`]
    pub fn tl(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tl(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tle`]
    pub fn tle(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tle(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tug`]
    pub fn tug(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tug(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tuge`]
    pub fn tuge(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tuge(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tul`]
    pub fn tul(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tul(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tule`]
    pub fn tule(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Tule(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Ifs`]
    pub fn ifs(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Ifs(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Js`]
    pub fn js(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self, Op::Js(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Jmp`]
    pub fn jmp(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self, Op::Jmp(op1.into()));
        self
    }

    /// Insert an [`Op::Vexit`]
    pub fn vexit(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self, Op::Vexit(op1.into()));
        self
    }

    /// Insert an [`Op::Vxcall`]
    pub fn vxcall(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self, Op::Vxcall(op1.into()));
        self
    }

    /// Insert an [`Op::Nop`]
    pub fn nop(&mut self) -> &mut Self {
        insert_instr(self, Op::Nop);
        self
    }

    /// Insert an [`Op::Sfence`]
    pub fn sfence(&mut self) -> &mut Self {
        insert_instr(self, Op::Sfence);
        self
    }

    /// Insert an [`Op::Lfence`]
    pub fn lfence(&mut self) -> &mut Self {
        insert_instr(self, Op::Lfence);
        self
    }

    /// Insert an [`Op::Vemit`]
    pub fn vemit(&mut self, op1: ImmediateDesc) -> &mut Self {
        insert_instr(self, Op::Vemit(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinr`]
    pub fn vpinr(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Vpinr(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinw`]
    pub fn vpinw(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self, Op::Vpinw(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinrm`]
    pub fn vpinrm(
        &mut self,
        op1: RegisterDesc,
        op2: ImmediateDesc,
        op3: ImmediateDesc,
    ) -> &mut Self {
        insert_instr(self, Op::Vpinrm(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Vpinwm`]
    pub fn vpinwm(
        &mut self,
        op1: RegisterDesc,
        op2: ImmediateDesc,
        op3: ImmediateDesc,
    ) -> &mut Self {
        insert_instr(self, Op::Vpinwm(op1.into(), op2.into(), op3.into()));
        self
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn basic() {
        use crate::*;

        let mut routine = Routine::new(ArchitectureIdentifier::Virtual);
        let basic_block = routine.create_block(Vip(0)).unwrap();
        let tmp0 = basic_block.tmp(64);
        let mut builder = InstructionBuilder::from(basic_block);
        builder.mov(tmp0, 0xA57E6F0335298D0u64.into());

        assert_eq!(basic_block.instructions.len(), 1);
        let instr = &basic_block.instructions[0];
        assert!(matches!(instr.op, Op::Mov(_, _)));
    }
}
