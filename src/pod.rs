use crate::arch_info;
use std::fmt;

/// Architecture for IL inside of VTIL routines
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArchitectureIdentifier {
    /// AMD64 (otherwise known as x86_64) architecture
    Amd64,
    /// AArch64 architecture
    Arm64,
    /// Virtual architecture (contains no physical register access)
    Virtual,
}

/// Header containing metadata regarding the VTIL container
#[derive(Debug)]
pub struct Header {
    /// The architecture used by the VTIL routine
    pub arch_id: ArchitectureIdentifier,
}

/// VTIL instruction pointer
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Vip(pub u64);

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

/// Describes a VTIL register in an operand
#[derive(Debug)]
pub struct RegisterDesc {
    /// Flags describing the register
    pub flags: RegisterFlags,
    /// Identifier for this register, use [`RegisterDesc::local_id`]
    pub combined_id: u64,
    /// The bit count of this register (e.g.: 32)
    pub bit_count: i32,
    /// The bit offset of register access
    pub bit_offset: i32,
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

        if self.bit_offset != 0 {
            suffix = format!("@{}", self.bit_offset);
        }

        if self.bit_count != 64 {
            suffix.push_str(&format!(":{}", self.bit_count));
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

        if self.flags.contains(RegisterFlags::PHYSICAL) {
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

/// Routine calling convention information and associated metadata
#[derive(Debug)]
pub struct RoutineConvention {
    /// List of registers that may change as a result of the routine execution but
    /// will be considered trashed
    pub volatile_registers: Vec<RegisterDesc>,
    /// List of regsiters that this routine wlil read from as a way of taking arguments
    /// * Additional arguments will be passed at `[$sp + shadow_space + n * 8]`
    pub param_registers: Vec<RegisterDesc>,
    /// List of registers that are used to store the return value of the routine and
    /// thus will change during routine execution but must be considered "used" by return
    pub retval_registers: Vec<RegisterDesc>,
    /// Register that is generally used to store the stack frame if relevant
    pub frame_register: RegisterDesc,
    /// Size of the shadow space
    pub shadow_space: u64,
    /// Purges any writes to stack that will be end up below the final stack pointer
    pub purge_stack: bool,
}

#[derive(Clone, Copy)]
pub(crate) union Immediate {
    pub(crate) u64: u64,
    pub(crate) i64: i64,
}

impl Immediate {
    pub(crate) fn u64(&self) -> u64 {
        unsafe { self.u64 }
    }

    pub(crate) fn set_u64(&mut self, imm: u64) {
        self.u64 = imm;
    }

    pub(crate) fn i64(&self) -> i64 {
        unsafe { self.i64 }
    }

    pub(crate) fn set_i64(&mut self, imm: i64) {
        self.i64 = imm;
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Immediate")
            .field("u64", &self.u64())
            .field("i64", &self.i64())
            .finish()
    }
}

/// Describes a VTIL immediate value in an operand
#[derive(Debug)]
pub struct ImmediateDesc {
    pub(crate) value: Immediate,
    /// The bit count of this register (e.g.: 32)
    pub bit_count: u32,
}

impl ImmediateDesc {
    /// Access the underlying immediate as a `u64`
    pub fn u64(&self) -> u64 {
        self.value.u64()
    }

    /// Set the value of the underlying immediate as a `u64`
    pub fn set_u64(&mut self, imm: u64) {
        self.value.set_u64(imm);
    }

    /// Access the underlying immediate as an `i64`
    pub fn i64(&self) -> i64 {
        self.value.i64()
    }

    /// Set the value of the underlying immediate as an `i64`
    pub fn set_i64(&mut self, imm: i64) {
        self.value.set_i64(imm);
    }
}

/// VTIL instruction operand
#[derive(Debug)]
pub enum Operand {
    /// Immediate operand containing a sized immediate value
    Imm(ImmediateDesc),
    /// Register operand containing a register description
    Reg(RegisterDesc),
}

/// VTIL instruction and associated metadata
#[derive(Debug)]
pub struct Instruction {
    /// The name of the instruction (e.g.: `ldd`)
    pub name: String,
    /// List of operands used in this instruction (in order)
    pub operands: Vec<Operand>,
    /// The virtual instruction pointer of this instruction
    pub vip: Vip,
    /// Stack pointer offset at this instruction
    pub sp_offset: i64,
    /// Stack instance index
    pub sp_index: u32,
    /// If the stack pointer is reset at this instruction
    pub sp_reset: bool,
}

/// Basic block containing a linear sequence of VTIL instructions
#[derive(Debug)]
pub struct BasicBlock {
    /// The virtual instruction pointer at entry
    pub vip: Vip,
    /// The stack pointer offset at entry
    pub sp_offset: i64,
    /// The stack instance index at entry
    pub sp_index: u32,
    /// Last temporary index used
    pub last_temporary_index: u32,
    /// List of instructions contained in this basic block (in order)
    pub instructions: Vec<Instruction>,
    /// Predecessor basic block entrypoint(s)
    pub prev_vip: Vec<Vip>,
    /// Successor basic block entrypoint(s)
    pub next_vip: Vec<Vip>,
}

/// Alias for [`RoutineConvention`] for consistent naming
pub type SubroutineConvention = RoutineConvention;

/// VTIL container
#[derive(Debug)]
pub struct VTIL {
    /// Header containing metadata about the VTIL container
    pub header: Header,
    /// The entry virtual instruction pointer for this VTIL routine
    pub vip: Vip,
    /// Metadata regarding the calling conventions of the VTIL routine
    pub routine_convention: RoutineConvention,
    /// Metadata regarding the calling conventions of the VTIL subroutine
    pub subroutine_convention: SubroutineConvention,
    /// All special subroutine calling conventions in the top-level VTIL routine
    pub spec_subroutine_conventions: Vec<SubroutineConvention>,
    /// Reachable [`BasicBlock`]s generated during a code-discovery analysis pass
    pub explored_blocks: Vec<BasicBlock>,
}
