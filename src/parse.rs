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

use getset::{CopyGetters, Getters};
use scroll::{ctx, Endian, Pread};
use std::fmt;

use super::{arch_info, Error, Result};

const VTIL_MAGIC_1: u32 = 0x4c495456;
const VTIL_MAGIC_2: u16 = 0xdead;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Architecture for IL inside of VTIL routines
pub enum ArchitectureIdentifier {
    /// AMD64 (otherwise known as x86_64) architecture
    Amd64,
    /// AArch64 architecture
    Arm64,
    /// Virtual architecture (contains no physical register access)
    Virtual,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for ArchitectureIdentifier {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], _endian: Endian) -> Result<(Self, usize)> {
        Ok((
            match source.pread::<u8>(0)? {
                0 => ArchitectureIdentifier::Amd64,
                1 => ArchitectureIdentifier::Arm64,
                2 => ArchitectureIdentifier::Virtual,
                arch_id => {
                    return Err(Error::Malformed(format!(
                        "Invalid architecture identifier: {:#x}",
                        arch_id
                    )))
                }
            },
            1,
        ))
    }
}

#[derive(Debug, CopyGetters)]
#[get_copy = "pub"]
/// Header containing metadata regarding the VTIL container
pub struct Header {
    /// The architecture used by the VTIL routine
    arch_id: ArchitectureIdentifier,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for Header {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
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

        Ok((Header { arch_id }, *offset))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
/// VTIL instruction pointer
pub struct Vip(pub u64);

impl<'a> ctx::TryFromCtx<'a, Endian> for Vip {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;
        Ok((Vip(source.gread_with::<u64>(offset, endian)?), *offset))
    }
}

bitflags! {
    /// Flags describing register properties
    pub struct RegisterFlags: u64 {
        /// Default value if no flags set. Read/write pure virtual register that
        /// is not a stack pointer or flags
        const VIRTUAL = 0;
        /// Indicates that the register is a physical register
        const PHYSICAL = 1 << 0;
        /// Indicates that the register is a local temporary register of the current basic block
        const LOCAL = 1 << 1;
        /// Indicates that the register is used to hold CPU flags
        const FLAGS = 1 << 2;
        /// Indicates that the register is used as the stack pointer
        const STACK_POINTER = 1 << 3;
        /// Indicates that the register is an alias to the image base
        const IMAGE_BASE = 1 << 4;
        /// Indicates that the register can change spontanously (e.g.: IA32_TIME_STAMP_COUNTER)
        const VOLATILE = 1 << 5;
        /// Indicates that the register can change spontanously (e.g.: IA32_TIME_STAMP_COUNTER)
        const READONLY = 1 << 6;
        /// Indicates that it is the special "undefined" register
        const UNDEFINED = 1 << 7;
        /// Indicates that it is a internal-use register that should be treated
        /// like any other virtual register
        const INTERNAL = 1 << 8;
        /// Combined mask of all special registers
        const SPECIAL = Self::FLAGS.bits | Self::STACK_POINTER.bits | Self::IMAGE_BASE.bits | Self::UNDEFINED.bits;
    }
}

#[derive(Debug, CopyGetters)]
/// Describes a VTIL register in an operand
pub struct RegisterDesc {
    #[get_copy = "pub"]
    /// Flags describing the register
    flags: RegisterFlags,
    combined_id: u64,
    #[get_copy = "pub"]
    /// The bit count of this register (e.g.: 32)
    bit_count: i32,
    #[get_copy = "pub"]
    /// The bit offset of register access
    bit_offset: i32,
}

impl RegisterDesc {
    /// Local identifier that is intentionally unique to this register
    pub fn local_id(&self) -> u64 {
        self.combined_id & !(0xff << 56)
    }

    /// The underlying architecture of this register
    pub fn arch_id(&self) -> ArchitectureIdentifier {
        match self.combined_id & (0xff << 56) {
            0 => ArchitectureIdentifier::Amd64,
            1 => ArchitectureIdentifier::Arm64,
            2 => ArchitectureIdentifier::Virtual,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for RegisterDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut prefix = String::new();

        if self.flags.contains(RegisterFlags::VOLATILE) {
            prefix = "?".to_string();
        }

        if self.flags.contains(RegisterFlags::READONLY) {
            prefix += "&&";
        }

        let mut suffix = String::new();

        if self.bit_offset() != 0 {
            suffix = format!("@{}", self.bit_offset());
        }

        if self.bit_count() != 64 {
            suffix.push_str(&format!(":{}", self.bit_count()));
        }

        if self.flags.contains(RegisterFlags::INTERNAL) {
            write!(f, "{}sr{}{}", prefix, self.local_id(), suffix)?;
            return Ok(());
        } else if self.flags.contains(RegisterFlags::UNDEFINED) {
            write!(f, "{}UD{}", prefix, suffix)?;
            return Ok(());
        } else if self.flags.contains(RegisterFlags::FLAGS) {
            write!(f, "{}$flags{}", prefix, suffix)?;
            return Ok(());
        } else if self.flags.contains(RegisterFlags::STACK_POINTER) {
            write!(f, "{}$sp{}", prefix, suffix)?;
            return Ok(());
        } else if self.flags.contains(RegisterFlags::IMAGE_BASE) {
            write!(f, "{}base{}", prefix, suffix)?;
            return Ok(());
        } else if self.flags.contains(RegisterFlags::LOCAL) {
            write!(f, "{}t{}{}", prefix, self.local_id(), suffix)?;
            return Ok(());
        }

        if self.flags().contains(RegisterFlags::PHYSICAL) {
            match self.arch_id() {
                ArchitectureIdentifier::Amd64 => {
                    write!(
                        f,
                        "{}{}{}",
                        prefix,
                        arch_info::X86_REGISTER_NAME_MAPPING[self.local_id() as usize],
                        suffix
                    )?;
                    return Ok(());
                }
                ArchitectureIdentifier::Arm64 => {
                    write!(
                        f,
                        "{}{}{}",
                        prefix,
                        arch_info::AARCH64_REGISTER_NAME_MAPPING[self.local_id() as usize],
                        suffix
                    )?;
                    return Ok(());
                }
                _ => unreachable!(),
            }
        }

        write!(f, "{}vr{}{}", prefix, self.local_id(), suffix)?;
        Ok(())
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for RegisterDesc {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let flags = RegisterFlags {
            bits: source.gread_with::<u64>(offset, endian)?,
        };

        let combined_id = source.gread_with::<u64>(offset, endian)?;
        if combined_id & (0xff << 56) > 2 {
            return Err(Error::Malformed(
                "Register flags are invalid: >2".to_string(),
            ));
        }

        let bit_count = source.gread_with::<i32>(offset, endian)?;
        let bit_offset = source.gread_with::<i32>(offset, endian)?;

        Ok((
            RegisterDesc {
                flags,
                combined_id,
                bit_count,
                bit_offset,
            },
            *offset,
        ))
    }
}

#[derive(Debug, CopyGetters, Getters)]
/// Routine calling convention information and associated metadata
pub struct RoutineConvention {
    #[get = "pub"]
    /// List of registers that may change as a result of the routine execution but
    /// will be considered trashed
    volatile_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    /// List of regsiters that this routine wlil read from as a way of taking arguments
    /// * Additional arguments will be passed at `[$sp + shadow_space + n * 8]`
    param_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    /// List of registers that are used to store the return value of the routine and
    /// thus will change during routine execution but must be considered "used" by return
    retval_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    /// Register that is generally used to store the stack frame if relevant
    frame_register: RegisterDesc,
    #[get_copy = "pub"]
    /// Size of the shadow space
    shadow_space: u64,
    #[get_copy = "pub"]
    /// Purges any writes to stack that will be end up below the final stack pointer
    purge_stack: bool,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for RoutineConvention {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let volatile_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut volatile_registers =
            Vec::<RegisterDesc>::with_capacity(volatile_registers_count as usize);
        for _ in 0..volatile_registers_count {
            volatile_registers.push(source.gread_with(offset, endian)?);
        }

        let param_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut param_registers =
            Vec::<RegisterDesc>::with_capacity(param_registers_count as usize);
        for _ in 0..param_registers_count {
            param_registers.push(source.gread_with(offset, endian)?);
        }

        let retval_registers_count = source.gread_with::<u32>(offset, endian)?;
        let mut retval_registers =
            Vec::<RegisterDesc>::with_capacity(retval_registers_count as usize);
        for _ in 0..retval_registers_count {
            retval_registers.push(source.gread_with(offset, endian)?);
        }

        let frame_register = source.gread_with::<RegisterDesc>(offset, endian)?;
        let shadow_space = source.gread_with::<u64>(offset, endian)?;
        let purge_stack = source.gread_with::<u8>(offset, endian)? != 0;

        Ok((
            RoutineConvention {
                volatile_registers,
                param_registers,
                retval_registers,
                frame_register,
                shadow_space,
                purge_stack,
            },
            *offset,
        ))
    }
}

#[derive(Clone, Copy)]
union Immediate {
    u64: u64,
    i64: i64,
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Immediate")
            .field("u64", &self.u64())
            .field("i64", &self.i64())
            .finish()
    }
}

impl Immediate {
    fn u64(&self) -> u64 {
        unsafe { self.u64 }
    }

    fn i64(&self) -> i64 {
        unsafe { self.i64 }
    }
}

#[derive(Debug, CopyGetters)]
/// Describes a VTIL immediate value in an operand
pub struct ImmediateDesc {
    value: Immediate,
    #[get_copy = "pub"]
    /// The bit count of this register (e.g.: 32)
    bit_count: u32,
}

impl ImmediateDesc {
    /// Access the underlying immediate as an `i64`
    pub fn u64(&self) -> u64 {
        self.value.u64()
    }

    /// Access the underlying immediate as a `u64`
    pub fn i64(&self) -> i64 {
        self.value.i64()
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for ImmediateDesc {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let value = source.gread_with::<u64>(offset, endian)?;
        let bit_count = source.gread_with::<u32>(offset, endian)?;

        Ok((
            ImmediateDesc {
                value: Immediate { u64: value },
                bit_count,
            },
            *offset,
        ))
    }
}

#[derive(Debug)]
/// VTIL instruction operand
pub enum Operand {
    /// Immediate operand containing a sized immediate value
    Imm(ImmediateDesc),
    /// Register operand containing a register description
    Reg(RegisterDesc),
}

impl<'a> ctx::TryFromCtx<'a, Endian> for Operand {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let sp_index = source.gread_with::<u32>(offset, endian)?;
        Ok((
            match sp_index {
                0 => Operand::Imm(source.gread_with::<ImmediateDesc>(offset, endian)?),
                1 => Operand::Reg(source.gread_with::<RegisterDesc>(offset, endian)?),
                i => return Err(Error::Malformed(format!("Invalid operand: {:#x}", i))),
            },
            *offset,
        ))
    }
}

#[derive(Debug, CopyGetters, Getters)]
/// VTIL instruction and associated metadata
pub struct Instruction<'a> {
    #[get_copy = "pub"]
    /// The name of the instruction (e.g.: `ldd`)
    name: &'a str,
    #[get = "pub"]
    /// List of operands used in this instruction (in order)
    operands: Vec<Operand>,
    #[get_copy = "pub"]
    /// The virtual instruction pointer of this instruction
    vip: u64,
    #[get_copy = "pub"]
    /// Stack pointer offset at this instruction
    sp_offset: i64,
    #[get_copy = "pub"]
    /// Stack instance index
    sp_index: u32,
    #[get_copy = "pub"]
    /// If the stack pointer is reset at this instruction
    sp_reset: bool,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for Instruction<'a> {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let name_size = source.gread_with::<u32>(offset, endian)?;
        let name = std::str::from_utf8(source.gread_with::<&'a [u8]>(offset, name_size as usize)?)?;

        let operands_count = source.gread_with::<u32>(offset, endian)?;
        let mut operands = Vec::<Operand>::with_capacity(operands_count as usize);
        for _ in 0..operands_count {
            operands.push(source.gread_with(offset, endian)?);
        }

        let vip = source.gread_with::<u64>(offset, endian)?;
        let sp_offset = source.gread_with::<i64>(offset, endian)?;
        let sp_index = source.gread_with::<u32>(offset, endian)?;
        let sp_reset = source.gread::<u8>(offset)? != 0;

        Ok((
            Instruction {
                name,
                operands,
                vip,
                sp_offset,
                sp_index,
                sp_reset,
            },
            *offset,
        ))
    }
}

#[derive(Debug, CopyGetters, Getters)]
/// Basic block containing a linear sequence of VTIL instructions
pub struct BasicBlock<'a> {
    #[get_copy = "pub"]
    /// The virtual instruction pointer at entry
    vip: Vip,
    #[get_copy = "pub"]
    /// The stack pointer offset at entry
    sp_offset: i64,
    #[get_copy = "pub"]
    /// The stack instance index at entry
    sp_index: u32,
    #[get_copy = "pub"]
    /// Last temporary index used
    last_temporary_index: u32,
    #[get = "pub"]
    /// List of instructions contained in this basic block (in order)
    instructions: Vec<Instruction<'a>>,
    #[get = "pub"]
    /// Predecessor basic block entrypoint(s)
    prev_vip: Vec<Vip>,
    #[get = "pub"]
    /// Successor basic block entrypoint(s)
    next_vip: Vec<Vip>,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for BasicBlock<'a> {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
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

        Ok((
            BasicBlock {
                vip,
                sp_offset,
                sp_index,
                last_temporary_index,
                instructions,
                prev_vip,
                next_vip,
            },
            *offset,
        ))
    }
}

/// Alias for [`RoutineConvention`] for consistent naming
pub type SubroutineConvention = RoutineConvention;

#[derive(Debug, Getters)]
#[get = "pub(crate)"]
#[doc(hidden)]
pub struct VTILInner<'a> {
    header: Header,
    vip: Vip,
    routine_convention: RoutineConvention,
    subroutine_convention: SubroutineConvention,
    spec_subroutine_conventions: Vec<SubroutineConvention>,
    explored_blocks: Vec<BasicBlock<'a>>,
}

impl<'a> ctx::TryFromCtx<'a, Endian> for VTILInner<'a> {
    type Error = Error;

    fn try_from_ctx(source: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
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

        Ok((
            VTILInner {
                header,
                vip,
                routine_convention,
                subroutine_convention,
                spec_subroutine_conventions,
                explored_blocks,
            },
            *offset,
        ))
    }
}
