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
pub struct Reg {
    /// Flags describing the register
    pub flags: RegisterFlags,
    /// Identifier for this register, use [`Reg::local_id`]
    pub combined_id: u64,
    /// The bit count of this register (e.g.: 32)
    pub bit_count: i32,
    /// The bit offset of register access
    pub bit_offset: i32,
}

impl Reg {
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

impl fmt::Display for Reg {
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
    pub volatile_registers: Vec<Reg>,
    /// List of regsiters that this routine wlil read from as a way of taking arguments
    /// * Additional arguments will be passed at `[$sp + shadow_space + n * 8]`
    pub param_registers: Vec<Reg>,
    /// List of registers that are used to store the return value of the routine and
    /// thus will change during routine execution but must be considered "used" by return
    pub retval_registers: Vec<Reg>,
    /// Register that is generally used to store the stack frame if relevant
    pub frame_register: Reg,
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
pub struct Imm {
    pub(crate) value: Immediate,
    /// The bit count of this register (e.g.: 32)
    pub bit_count: u32,
}

impl Imm {
    /// Immediate from a `u64`
    pub fn new<T: Into<u64>>(value: T, bit_count: u32) -> Imm {
        assert!(bit_count % 8 == 0);
        Imm {
            value: Immediate { u64: value.into() },
            bit_count,
        }
    }

    /// Immediate from an `i64`
    pub fn new_signed<T: Into<i64>>(value: T, bit_count: u32) -> Imm {
        assert!(bit_count % 8 == 0);
        Imm {
            value: Immediate { i64: value.into() },
            bit_count,
        }
    }

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
    Imm(Imm),
    /// Register operand containing a register description
    Reg(Reg),
}

/// VTIL instruction and associated metadata
#[derive(Debug)]
pub struct Instruction {
    /// Instruction operation and operators
    pub op: Op,
    /// The virtual instruction pointer of this instruction
    pub vip: Vip,
    /// Stack pointer offset at this instruction
    pub sp_offset: i64,
    /// Stack instance index
    pub sp_index: u32,
    /// If the stack pointer is reset at this instruction
    pub sp_reset: bool,
}

/// VTIL operator and operands
#[derive(Debug)]
enum Op {
    // Data/Memory instructions
    /// OP1 = ZX(OP2)
    Mov(Reg, Operand),
    /// OP1 = SX(OP2)
    Movsx(Reg, Operand),
    /// [OP1+OP2] <= OP3
    Str(Reg, Imm, Operand),
    /// OP1 <= [OP2+OP3]
    Ldd(Reg, Reg, Imm),

    // Arithmetic instructions
    /// OP1 = -OP1
    Neg(Reg),
    /// OP1 = OP1 + OP2
    Add(Reg, Operand),
    /// OP1 = OP1 - OP2
    Sub(Reg, Operand),
    /// OP1 = OP1 * OP2
    Mul(Reg, Operand),
    /// OP1 = [OP1 * OP2]>>N
    Mulhi(Reg, Operand),
    /// OP1 = OP1 * OP2 (Signed)
    Imul(Reg, Operand),
    /// OP1 = [OP1 * OP2]>>N (Signed)
    Imulhi(Reg, Operand),
    /// OP1 = [OP2:OP1] / OP3
    Div(Reg, Operand, Operand),
    /// OP1 = [OP2:OP1] % OP3
    Rem(Reg, Operand, Operand),
    /// OP1 = [OP2:OP1] / OP3 (Signed)
    Idiv(Reg, Operand, Operand),
    /// OP1 = [OP2:OP1] % OP3 (Signed)
    Irem(Reg, Operand, Operand),

    // Bitwise instructions
    /// OP1 = popcnt OP1
    Popcnt(Reg),
    /// OP1 = OP1 ? BitScanForward OP1 + 1 : 0
    Bsf(Reg),
    /// OP1 = OP1 ? BitScanReverse OP1 + 1 : 0
    Bsr(Reg),
    /// OP1 = ~OP1
    Not(Reg),
    /// OP1 >>= OP2
    Shr(Reg, Operand),
    /// OP1 <<= OP2
    Shl(Reg, Operand),
    /// OP1 ^= OP2
    Xor(Reg, Operand),
    /// OP1 |= OP2
    Or(Reg, Operand),
    /// OP1 &= OP2
    And(Reg, Operand),
    /// OP1 = (OP1>>OP2) | (OP1<<(N-OP2))
    Ror(Reg, Operand),
    /// OP1 = (OP1<<OP2) | (OP1>>(N-OP2))
    Rol(Reg, Operand),

    // Conditional instructions
    /// OP1 = OP2 > OP3
    Tg(Reg, Operand, Operand),
    /// OP1 = OP2 >= OP3
    Tge(Reg, Operand, Operand),
    /// OP1 = OP2 == OP3
    Te(Reg, Operand, Operand),
    /// OP1 = OP2 != OP3
    Tne(Reg, Operand, Operand),
    /// OP1 = OP2 < OP3
    Tl(Reg, Operand, Operand),
    /// OP1 = OP2 <= OP3
    Tle(Reg, Operand, Operand),
    /// OP1 = OP2 <= OP3
    Tug(Reg, Operand, Operand),
    /// OP1 = OP2   u>=  OP3
    Tuge(Reg, Operand, Operand),
    /// OP1 = OP2   u<   OP3
    Tul(Reg, Operand, Operand),
    /// OP1 = OP2   u<=  OP3
    Tule(Reg, Operand, Operand),
    /// OP1 = OP2 ? OP3 : 0
    Ifs(Reg, Operand, Operand),

    // Control flow instructions
    /// Jumps to OP1 ? OP2 : OP3, continues virtual execution
    Js(Reg, Operand, Operand),
    /// Jumps to OP1, continues virtual execution
    Jmp(Operand),
    /// Jumps to OP1, continues real execution
    Vexit(Operand),
    /// Calls into OP1, pauses virtual execution until the call returns
    Vxcall(Operand),

    // Special instructions
    /// Placeholder
    Nop,
    /// Assumes all memory is read from
    Sfence,
    /// Assumes all memory is written to
    Lfence,
    /// Emits the opcode as is to the final instruction stream
    Vemit(Imm),
    /// Pins the register for read
    Vpinr(Reg),
    /// Pins the register for write
    Vpinw(Reg),
    /// Pins the memory location for read, with size = OP3
    Vpinrm(Reg, Imm, Imm),
    /// Pins the memory location for write, with size = OP3
    Vpinwm(Reg, Imm, Imm),
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
