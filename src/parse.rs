use getset::{CopyGetters, Getters};
use scroll::{ctx, Endian, Pread};
use std::fmt;

use super::{Error, Result};

const VTIL_MAGIC_1: u32 = 0x4c495456;
const VTIL_MAGIC_2: u16 = 0xdead;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArchitectureIdentifier {
    Amd64,
    Arm64,
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
pub struct Header {
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
    pub struct RegisterFlags: u64 {
        const VIRTUAL = 0;
        const PHYSICAL = 1 << 0;
        const LOCAL = 1 << 1;
        const FLAGS = 1 << 2;
        const STACK_POINTER = 1 << 3;
        const IMAGE_BASE = 1 << 4;
        const VOLATILE = 1 << 5;
        const READONLY = 1 << 6;
        const UNDEFINED = 1 << 7;
        const INTERNAL = 1 << 8;
        const SPECIAL = Self::FLAGS.bits | Self::STACK_POINTER.bits | Self::IMAGE_BASE.bits | Self::UNDEFINED.bits;
    }
}

#[derive(Debug, CopyGetters)]
#[get_copy = "pub"]
/// Describes a VTIL register in an operand
pub struct RegisterDesc {
    flags: RegisterFlags,
    combined_id: u64,
    bit_count: i32,
    bit_offset: i32,
}

impl RegisterDesc {
    pub fn local_id(&self) -> u64 {
        self.combined_id() & 0x00ffffffffffffff
    }

    pub fn architecture(&self) -> ArchitectureIdentifier {
        match self.combined_id() & 0xff {
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
            match self.architecture() {
                ArchitectureIdentifier::Amd64 => {
                    write!(f, "{}TODO_AMD64{}", prefix, suffix)?;
                    return Ok(());
                }
                ArchitectureIdentifier::Arm64 => {
                    write!(f, "{}TODO_ARM64{}", prefix, suffix)?;
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
        if combined_id & 0xff00000000000000 > 2 {
            println!("{:#x}", combined_id);
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
    volatile_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    param_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    retval_registers: Vec<RegisterDesc>,
    #[get = "pub"]
    frame_register: RegisterDesc,
    #[get_copy = "pub"]
    shadow_space: u64,
    #[get_copy = "pub"]
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
    pub u64: u64,
    pub i64: i64,
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
    bit_count: u32,
}

impl ImmediateDesc {
    pub fn u64(&self) -> u64 {
        self.value.u64()
    }

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
    Imm(ImmediateDesc),
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
    name: &'a str,
    #[get = "pub"]
    operands: Vec<Operand>,
    #[get_copy = "pub"]
    vip: u64,
    #[get_copy = "pub"]
    sp_offset: i64,
    #[get_copy = "pub"]
    sp_index: u32,
    #[get_copy = "pub"]
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
    vip: Vip,
    #[get_copy = "pub"]
    sp_offset: i64,
    #[get_copy = "pub"]
    sp_index: u32,
    #[get_copy = "pub"]
    last_temporary_index: u32,
    #[get = "pub"]
    instructions: Vec<Instruction<'a>>,
    #[get = "pub"]
    prev_vip: Vec<Vip>,
    #[get = "pub"]
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
