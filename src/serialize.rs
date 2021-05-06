// BSD 3-Clause License
//
// Copyright © 2020-2021 Keegan Saunders
// Copyright © 2020 VTIL Project
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

use scroll::{
    ctx::{self, SizeWith},
    Endian, Pread, Pwrite,
};
use std::convert::TryInto;
use std::mem::size_of;

use super::{
    ArchitectureIdentifier, BasicBlock, Error, Header, Imm, Immediate, Instruction, Op, Operand,
    Reg, RegisterFlags, Result, RoutineConvention, SubroutineConvention, Vip, VTIL,
};

const VTIL_MAGIC_1: u32 = 0x4c495456;
const VTIL_MAGIC_2: u16 = 0xdead;

impl ctx::SizeWith<ArchitectureIdentifier> for ArchitectureIdentifier {
    fn size_with(_arch_id: &ArchitectureIdentifier) -> usize {
        size_of::<u8>()
    }
}

impl ctx::TryFromCtx<'_, Endian> for ArchitectureIdentifier {
    type Error = Error;

    fn try_from_ctx(source: &[u8], _endian: Endian) -> Result<(Self, usize)> {
        let arch_id = match source.pread::<u8>(0)? {
            0 => ArchitectureIdentifier::Amd64,
            1 => ArchitectureIdentifier::Arm64,
            2 => ArchitectureIdentifier::Virtual,
            arch_id => {
                return Err(Error::Malformed(format!(
                    "Invalid architecture identifier: {:#x}",
                    arch_id
                )))
            }
        };
        assert_eq!(ArchitectureIdentifier::size_with(&arch_id), 1);
        Ok((arch_id, 1))
    }
}

impl ctx::TryIntoCtx<Endian> for ArchitectureIdentifier {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        sink.pwrite::<u8>(self as u8, 0)?;
        Ok(size_of::<u8>())
    }
}

impl ctx::SizeWith<Header> for Header {
    fn size_with(header: &Header) -> usize {
        let mut size = size_of::<u32>();
        size += ArchitectureIdentifier::size_with(&header.arch_id);
        size += size_of::<u8>();
        size += size_of::<u16>();
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for Header {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let magic = source.gread_with::<u32>(offset, endian)?;
        if magic != VTIL_MAGIC_1 {
            return Err(Error::Malformed(format!(
                "VTIL magic is invalid: {:#x}",
                magic
            )));
        }

        let arch_id = source.gread_with::<ArchitectureIdentifier>(offset, endian)?;
        let _zero = source.gread::<u8>(offset)?;

        let magic = source.gread_with::<u16>(offset, endian)?;
        if magic != VTIL_MAGIC_2 {
            return Err(Error::Malformed(format!(
                "VTIL magic is invalid: {:#x}",
                magic
            )));
        }

        let header = Header { arch_id };
        assert_eq!(Header::size_with(&header), *offset);
        Ok((header, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Header {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;
        sink.gwrite::<u32>(VTIL_MAGIC_1, offset)?;
        sink.gwrite::<ArchitectureIdentifier>(self.arch_id, offset)?;
        sink.gwrite::<u8>(0, offset)?;
        sink.gwrite::<u16>(VTIL_MAGIC_2, offset)?;
        Ok(*offset)
    }
}

impl ctx::SizeWith<Vip> for Vip {
    fn size_with(_vip: &Vip) -> usize {
        size_of::<u64>()
    }
}

impl ctx::TryFromCtx<'_, Endian> for Vip {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;
        let vip = Vip(source.gread_with::<u64>(offset, endian)?);
        assert_eq!(Vip::size_with(&vip), *offset);
        Ok((vip, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Vip {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        Ok(sink.pwrite::<u64>(self.0, 0)?)
    }
}

impl ctx::SizeWith<Reg> for Reg {
    fn size_with(_reg: &Reg) -> usize {
        let mut size = 0;
        size += size_of::<u64>();
        size += size_of::<u64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for Reg {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let flags = RegisterFlags::from_bits_truncate(source.gread_with::<u64>(offset, endian)?);

        let combined_id = source.gread_with::<u64>(offset, endian)?;
        if combined_id & (0xff << 56) > 2 {
            return Err(Error::Malformed(
                "Register flags are invalid: >2".to_string(),
            ));
        }

        let bit_count = source.gread_with::<i32>(offset, endian)?;
        let bit_offset = source.gread_with::<i32>(offset, endian)?;

        let reg = Reg {
            flags,
            combined_id,
            bit_count,
            bit_offset,
        };
        assert_eq!(Reg::size_with(&reg), *offset);
        Ok((reg, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Reg {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;
        sink.gwrite::<u64>(self.flags.bits(), offset)?;
        sink.gwrite::<u64>(self.combined_id, offset)?;
        sink.gwrite::<i32>(self.bit_count, offset)?;
        sink.gwrite::<i32>(self.bit_offset, offset)?;
        Ok(*offset)
    }
}

impl ctx::SizeWith<RoutineConvention> for RoutineConvention {
    fn size_with(routine_convention: &RoutineConvention) -> usize {
        let mut size = 0;

        size += size_of::<u32>();
        for reg in &routine_convention.volatile_registers {
            size += Reg::size_with(reg);
        }

        size += size_of::<u32>();
        for reg in &routine_convention.param_registers {
            size += Reg::size_with(reg);
        }

        size += size_of::<u32>();
        for reg in &routine_convention.retval_registers {
            size += Reg::size_with(reg);
        }

        size += Reg::size_with(&routine_convention.frame_register);
        size += size_of::<u64>();
        size += size_of::<u8>();

        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for RoutineConvention {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let volatile_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut volatile_registers = Vec::<Reg>::with_capacity(volatile_registers_count as usize);
        for _ in 0..volatile_registers_count {
            volatile_registers.push(source.gread_with(offset, endian)?);
        }

        let param_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut param_registers = Vec::<Reg>::with_capacity(param_registers_count as usize);
        for _ in 0..param_registers_count {
            param_registers.push(source.gread_with(offset, endian)?);
        }

        let retval_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut retval_registers = Vec::<Reg>::with_capacity(retval_registers_count as usize);
        for _ in 0..retval_registers_count {
            retval_registers.push(source.gread_with(offset, endian)?);
        }

        let frame_register = source.gread_with::<Reg>(offset, endian)?;
        let shadow_space = source.gread_with::<u64>(offset, endian)?;
        let purge_stack = source.gread_with::<u8>(offset, endian)? != 0;

        let routine_convention = RoutineConvention {
            volatile_registers,
            param_registers,
            retval_registers,
            frame_register,
            shadow_space,
            purge_stack,
        };
        assert_eq!(RoutineConvention::size_with(&routine_convention), *offset);
        Ok((routine_convention, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for RoutineConvention {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        sink.gwrite::<u32>(self.volatile_registers.len().try_into()?, offset)?;
        for reg in self.volatile_registers {
            sink.gwrite::<Reg>(reg, offset)?;
        }

        sink.gwrite::<u32>(self.param_registers.len().try_into()?, offset)?;
        for reg in self.param_registers {
            sink.gwrite::<Reg>(reg, offset)?;
        }

        sink.gwrite::<u32>(self.retval_registers.len().try_into()?, offset)?;
        for reg in self.retval_registers {
            sink.gwrite::<Reg>(reg, offset)?;
        }

        sink.gwrite::<Reg>(self.frame_register, offset)?;
        sink.gwrite::<u64>(self.shadow_space, offset)?;
        sink.gwrite::<u8>(self.purge_stack.into(), offset)?;
        Ok(*offset)
    }
}

impl ctx::SizeWith<Imm> for Imm {
    fn size_with(_imm: &Imm) -> usize {
        let mut size = 0;
        size += size_of::<u64>();
        size += size_of::<u32>();
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for Imm {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let value = source.gread_with::<u64>(offset, endian)?;
        let bit_count = source.gread_with::<u32>(offset, endian)?;

        let imm = Imm {
            value: Immediate { u64: value },
            bit_count,
        };
        assert_eq!(Imm::size_with(&imm), *offset);
        Ok((imm, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Imm {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;
        sink.gwrite::<u64>(self.value.u64(), offset)?;
        sink.gwrite::<u32>(self.bit_count, offset)?;
        Ok(*offset)
    }
}

impl ctx::SizeWith<Operand> for Operand {
    fn size_with(operand: &Operand) -> usize {
        let mut size = 0;
        size += size_of::<u32>();
        size += match operand {
            Operand::Imm(i) => Imm::size_with(i),
            Operand::Reg(r) => Reg::size_with(r),
        };
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for Operand {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let sp_index = source.gread_with::<u32>(offset, endian)?;
        let operand = match sp_index {
            0 => Operand::Imm(source.gread_with::<Imm>(offset, endian)?),
            1 => Operand::Reg(source.gread_with::<Reg>(offset, endian)?),
            i => return Err(Error::Malformed(format!("Invalid operand: {:#x}", i))),
        };
        assert_eq!(Operand::size_with(&operand), *offset);
        Ok((operand, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Operand {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;
        match self {
            Operand::Imm(i) => {
                sink.gwrite::<u32>(0, offset)?;
                sink.gwrite::<Imm>(i, offset)?;
            }
            Operand::Reg(r) => {
                sink.gwrite::<u32>(1, offset)?;
                sink.gwrite::<Reg>(r, offset)?;
            }
        }
        Ok(*offset)
    }
}

impl ctx::SizeWith<Op> for Op {
    fn size_with(op: &Op) -> usize {
        let mut size = 0;
        size += size_of::<u32>();
        size += op.name().as_bytes().len();
        size += size_of::<u32>();
        for operand in op.operands() {
            size += Operand::size_with(operand);
        }
        size
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for Op {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let name_size = source.gread_with::<u32>(offset, endian)?;
        let name = std::str::from_utf8(source.gread_with::<&'a [u8]>(offset, name_size as usize)?)?;

        let operands_count = source.gread_with::<u32>(offset, endian)?;

        let op = match name {
            "mov" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Mov(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "movsx" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Movsx(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "str" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Str(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "ldd" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Ldd(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "neg" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Neg(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "add" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Add(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "sub" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Sub(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "mul" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Mul(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "mulhi" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Mulhi(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "imul" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Imul(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "imulhi" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Imulhi(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "div" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Div(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "rem" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Rem(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "idiv" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Idiv(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "irem" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Irem(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "popcnt" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Popcnt(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "bsf" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Bsf(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "bsr" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Bsr(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "not" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Not(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "shr" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Shr(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "shl" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Shl(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "xor" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Xor(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "or" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Or(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "and" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::And(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "ror" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Ror(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "rol" => {
                if operands_count == 2 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Rol(op1, op2)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tg" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tg(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tge" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tge(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "te" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Te(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tne" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tne(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tl" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tl(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tle" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tle(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tug" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tug(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tuge" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tuge(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tul" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tul(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "tule" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Tule(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "ifs" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Ifs(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "js" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Js(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "jmp" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Jmp(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vexit" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vexit(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vxcall" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vxcall(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "nop" => {
                if operands_count == 0 {
                    Op::Nop
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "sfence" => {
                if operands_count == 0 {
                    Op::Sfence
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "lfence" => {
                if operands_count == 0 {
                    Op::Lfence
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vemit" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vemit(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vpinr" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vpinr(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vpinw" => {
                if operands_count == 1 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vpinw(op1)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vpinrm" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vpinrm(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            "vpinwm" => {
                if operands_count == 3 {
                    let op1 = source.gread_with::<Operand>(offset, endian)?;
                    let op2 = source.gread_with::<Operand>(offset, endian)?;
                    let op3 = source.gread_with::<Operand>(offset, endian)?;
                    Op::Vpinwm(op1, op2, op3)
                } else {
                    return Err(Error::OperandMismatch);
                }
            }
            _ => return Err(Error::Malformed(format!("Invalid operation: {}", name))),
        };
        assert_eq!(Op::size_with(&op), *offset);
        Ok((op, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Op {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        let name = self.name();

        sink.gwrite::<u32>(name.len().try_into()?, offset)?;
        sink.gwrite::<&[u8]>(name.as_bytes(), offset)?;

        sink.gwrite::<u32>(self.operands().len().try_into()?, offset)?;
        for operand in self.operands() {
            sink.gwrite::<Operand>(*operand, offset)?;
        }

        Ok(*offset)
    }
}

impl ctx::SizeWith<Instruction> for Instruction {
    fn size_with(instr: &Instruction) -> usize {
        let mut size = 0;
        size += Op::size_with(&instr.op);
        size += Vip::size_with(&instr.vip);
        size += size_of::<i64>();
        size += size_of::<u32>();
        size += size_of::<u8>();
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for Instruction {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let op = source.gread_with::<Op>(offset, endian)?;
        let vip = source.gread_with::<Vip>(offset, endian)?;
        let sp_offset = source.gread_with::<i64>(offset, endian)?;
        let sp_index = source.gread_with::<u32>(offset, endian)?;
        let sp_reset = source.gread::<u8>(offset)? != 0;

        let instr = Instruction {
            op,
            vip,
            sp_offset,
            sp_index,
            sp_reset,
        };
        assert_eq!(Instruction::size_with(&instr), *offset);
        Ok((instr, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for Instruction {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        sink.gwrite::<Op>(self.op, offset)?;
        sink.gwrite::<Vip>(self.vip, offset)?;
        sink.gwrite::<i64>(self.sp_offset, offset)?;
        sink.gwrite::<u32>(self.sp_index, offset)?;
        sink.gwrite::<u8>(self.sp_reset.into(), offset)?;

        Ok(*offset)
    }
}

impl ctx::SizeWith<BasicBlock> for BasicBlock {
    fn size_with(basic_block: &BasicBlock) -> usize {
        let mut size = 0;

        size += Vip::size_with(&basic_block.vip);
        size += size_of::<i64>();
        size += size_of::<u32>();
        size += size_of::<u32>();

        size += size_of::<u32>();
        for instr in &basic_block.instructions {
            size += Instruction::size_with(instr);
        }

        size += size_of::<u32>();
        for prev_vip in &basic_block.prev_vip {
            size += Vip::size_with(prev_vip);
        }

        size += size_of::<u32>();
        for next_vip in &basic_block.next_vip {
            size += Vip::size_with(next_vip);
        }

        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for BasicBlock {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let vip = Vip(source.gread_with::<u64>(offset, endian)?);
        let sp_offset = source.gread_with::<i64>(offset, endian)?;
        let sp_index = source.gread_with::<u32>(offset, endian)?;
        let last_temporary_index = source.gread_with::<u32>(offset, endian)?;

        let instruction_count = source.gread_with::<u32>(offset, endian)?;
        let mut instructions = Vec::<Instruction>::with_capacity(instruction_count as usize);
        for _ in 0..instruction_count {
            instructions.push(source.gread_with(offset, endian)?);
        }

        let prev_vip_count = source.gread_with::<u32>(offset, endian)?;
        let mut prev_vip = Vec::<Vip>::with_capacity(prev_vip_count as usize);
        for _ in 0..prev_vip_count {
            prev_vip.push(Vip(source.gread_with(offset, endian)?));
        }

        let next_vip_count = source.gread_with::<u32>(offset, endian)?;
        let mut next_vip = Vec::<Vip>::with_capacity(next_vip_count as usize);
        for _ in 0..next_vip_count {
            next_vip.push(Vip(source.gread_with(offset, endian)?));
        }

        let basic_block = BasicBlock {
            vip,
            sp_offset,
            sp_index,
            last_temporary_index,
            instructions,
            prev_vip,
            next_vip,
        };
        assert_eq!(BasicBlock::size_with(&basic_block), *offset);
        Ok((basic_block, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for BasicBlock {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        sink.gwrite::<Vip>(self.vip, offset)?;
        sink.gwrite::<i64>(self.sp_offset, offset)?;
        sink.gwrite::<u32>(self.sp_index, offset)?;
        sink.gwrite::<u32>(self.last_temporary_index, offset)?;

        sink.gwrite::<u32>(self.instructions.len().try_into()?, offset)?;
        for instr in self.instructions {
            sink.gwrite::<Instruction>(instr, offset)?;
        }

        sink.gwrite::<u32>(self.prev_vip.len().try_into()?, offset)?;
        for vip in self.prev_vip {
            sink.gwrite::<Vip>(vip, offset)?;
        }

        sink.gwrite::<u32>(self.next_vip.len().try_into()?, offset)?;
        for vip in self.next_vip {
            sink.gwrite::<Vip>(vip, offset)?;
        }

        Ok(*offset)
    }
}

impl ctx::SizeWith<VTIL> for VTIL {
    fn size_with(routine: &VTIL) -> usize {
        let mut size = 0;
        size += Header::size_with(&routine.header);
        size += Vip::size_with(&routine.vip);
        size += RoutineConvention::size_with(&routine.routine_convention);
        size += SubroutineConvention::size_with(&routine.subroutine_convention);

        size += size_of::<u32>();
        for convention in &routine.spec_subroutine_conventions {
            size += SubroutineConvention::size_with(convention);
        }

        size += size_of::<u32>();
        for basic_block in &routine.explored_blocks {
            size += BasicBlock::size_with(basic_block);
        }
        size
    }
}

impl ctx::TryFromCtx<'_, Endian> for VTIL {
    type Error = Error;

    fn try_from_ctx(source: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let header = source.gread_with::<Header>(offset, endian)?;
        let vip = source.gread_with::<Vip>(offset, endian)?;
        let routine_convention = source.gread_with::<RoutineConvention>(offset, endian)?;
        let subroutine_convention = source.gread_with::<SubroutineConvention>(offset, endian)?;

        let spec_subroutine_conventions_count = source.gread_with::<u32>(offset, endian)?;
        let mut spec_subroutine_conventions =
            Vec::<SubroutineConvention>::with_capacity(spec_subroutine_conventions_count as usize);
        for _ in 0..spec_subroutine_conventions_count {
            spec_subroutine_conventions.push(source.gread_with(offset, endian)?);
        }

        let explored_blocks_count = source.gread_with::<u32>(offset, endian)?;
        let mut explored_blocks = Vec::<BasicBlock>::with_capacity(explored_blocks_count as usize);
        for _ in 0..explored_blocks_count {
            explored_blocks.push(source.gread_with(offset, endian)?);
        }

        let routine = VTIL {
            header,
            vip,
            routine_convention,
            subroutine_convention,
            spec_subroutine_conventions,
            explored_blocks,
        };
        assert_eq!(VTIL::size_with(&routine), *offset);
        Ok((routine, *offset))
    }
}

impl ctx::TryIntoCtx<Endian> for VTIL {
    type Error = Error;

    fn try_into_ctx(self, sink: &mut [u8], _endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        sink.gwrite::<Header>(self.header, offset)?;
        sink.gwrite::<Vip>(self.vip, offset)?;
        sink.gwrite::<RoutineConvention>(self.routine_convention, offset)?;
        sink.gwrite::<SubroutineConvention>(self.subroutine_convention, offset)?;

        sink.gwrite::<u32>(self.spec_subroutine_conventions.len().try_into()?, offset)?;
        for convention in self.spec_subroutine_conventions {
            sink.gwrite::<SubroutineConvention>(convention, offset)?;
        }

        sink.gwrite::<u32>(self.explored_blocks.len().try_into()?, offset)?;
        for basic_block in self.explored_blocks {
            sink.gwrite::<BasicBlock>(basic_block, offset)?;
        }

        Ok(*offset)
    }
}
