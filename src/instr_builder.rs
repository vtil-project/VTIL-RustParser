use crate::{BasicBlock, ImmediateDesc, Instruction, Op, Operand, RegisterDesc, Vip};

/// Builder for VTIL instructions in an associated [`BasicBlock`]
pub struct InstructionBuilder<'a> {
    basic_block: &'a mut BasicBlock,
}

// Helper for inserting instructions with no associated metadata
fn insert_instr(basic_block: &mut BasicBlock, op: Op) {
    basic_block.instructions.push(Instruction {
        op,
        vip: Vip::invalid(),
        sp_offset: 0,
        sp_index: 0,
        sp_reset: false,
    });
}

impl<'a> InstructionBuilder<'a> {
    /// Build an [`InstructionBuilder`] from an existing [`BasicBlock`]
    pub fn from(basic_block: &'a mut BasicBlock) -> InstructionBuilder<'a> {
        InstructionBuilder { basic_block }
    }

    /// Insert an [`Op::Mov`]
    pub fn mov(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Mov(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Movsx`]
    pub fn movsx(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Movsx(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Str`]
    pub fn str(&mut self, op1: RegisterDesc, op2: ImmediateDesc, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Str(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Ldd`]
    pub fn ldd(&mut self, op1: RegisterDesc, op2: RegisterDesc, op3: ImmediateDesc) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Ldd(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Neg`]
    pub fn neg(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Neg(op1.into()));
        self
    }

    /// Insert an [`Op::Add`]
    pub fn add(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Add(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Sub`]
    pub fn sub(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Sub(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Mul`]
    pub fn mul(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Mul(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Mulhi`]
    pub fn mulhi(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Mulhi(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Imul`]
    pub fn imul(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Imul(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Imulhi`]
    pub fn imulhi(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Imulhi(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Div`]
    pub fn div(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Div(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Rem`]
    pub fn rem(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Rem(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Idiv`]
    pub fn idiv(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Idiv(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Irem`]
    pub fn irem(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Irem(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Popcnt`]
    pub fn popcnt(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Popcnt(op1.into()));
        self
    }

    /// Insert an [`Op::Bsf`]
    pub fn bsf(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Bsf(op1.into()));
        self
    }

    /// Insert an [`Op::Bsr`]
    pub fn bsr(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Bsr(op1.into()));
        self
    }

    /// Insert an [`Op::Not`]
    pub fn not(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Not(op1.into()));
        self
    }

    /// Insert an [`Op::Shr`]
    pub fn shr(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Shr(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Shl`]
    pub fn shl(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Shl(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Xor`]
    pub fn xor(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Xor(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Or`]
    pub fn or(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Or(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::And`]
    pub fn and(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::And(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Ror`]
    pub fn ror(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Ror(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Rol`]
    pub fn rol(&mut self, op1: RegisterDesc, op2: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Rol(op1.into(), op2.into()));
        self
    }

    /// Insert an [`Op::Tg`]
    pub fn tg(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Tg(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tge`]
    pub fn tge(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tge(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Te`]
    pub fn te(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Te(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tne`]
    pub fn tne(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tne(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Tl`]
    pub fn tl(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Tl(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Tle`]
    pub fn tle(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tle(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Tug`]
    pub fn tug(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tug(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Tuge`]
    pub fn tuge(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tuge(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Tul`]
    pub fn tul(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tul(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Tule`]
    pub fn tule(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Tule(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Ifs`]
    pub fn ifs(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Ifs(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Js`]
    pub fn js(&mut self, op1: RegisterDesc, op2: Operand, op3: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Js(op1.into(), op2.into(), op3.into()));
        self
    }

    /// Insert an [`Op::Jmp`]
    pub fn jmp(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Jmp(op1.into()));
        self
    }

    /// Insert an [`Op::Vexit`]
    pub fn vexit(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Vexit(op1.into()));
        self
    }

    /// Insert an [`Op::Vxcall`]
    pub fn vxcall(&mut self, op1: Operand) -> &mut Self {
        insert_instr(self.basic_block, Op::Vxcall(op1.into()));
        self
    }

    /// Insert an [`Op::Nop`]
    pub fn nop(&mut self) -> &mut Self {
        insert_instr(self.basic_block, Op::Nop);
        self
    }

    /// Insert an [`Op::Sfence`]
    pub fn sfence(&mut self) -> &mut Self {
        insert_instr(self.basic_block, Op::Sfence);
        self
    }

    /// Insert an [`Op::Lfence`]
    pub fn lfence(&mut self) -> &mut Self {
        insert_instr(self.basic_block, Op::Lfence);
        self
    }

    /// Insert an [`Op::Vemit`]
    pub fn vemit(&mut self, op1: ImmediateDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Vemit(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinr`]
    pub fn vpinr(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Vpinr(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinw`]
    pub fn vpinw(&mut self, op1: RegisterDesc) -> &mut Self {
        insert_instr(self.basic_block, Op::Vpinw(op1.into()));
        self
    }

    /// Insert an [`Op::Vpinrm`]
    pub fn vpinrm(
        &mut self,
        op1: RegisterDesc,
        op2: ImmediateDesc,
        op3: ImmediateDesc,
    ) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Vpinrm(op1.into(), op2.into(), op3.into()),
        );
        self
    }

    /// Insert an [`Op::Vpinwm`]
    pub fn vpinwm(
        &mut self,
        op1: RegisterDesc,
        op2: ImmediateDesc,
        op3: ImmediateDesc,
    ) -> &mut Self {
        insert_instr(
            self.basic_block,
            Op::Vpinwm(op1.into(), op2.into(), op3.into()),
        );
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
        builder.mov(
            tmp0,
            Operand::ImmediateDesc(ImmediateDesc::new(0xA57E6F0335298D0u64, 64)),
        );

        assert_eq!(basic_block.instructions.len(), 1);
        let instr = &basic_block.instructions[0];
        assert!(matches!(instr.op, Op::Mov(_, _)));
    }
}
