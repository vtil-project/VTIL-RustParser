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
    arch_info::{self, amd64, arm64},
    Error, Result,
};
use indexmap::map::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryInto, fmt};

/// Architecture for IL inside of VTIL routines
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Header {
    /// The architecture used by the VTIL routine
    pub arch_id: ArchitectureIdentifier,
}

/// VTIL instruction pointer
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Vip(pub u64);

impl Vip {
    /// Invalid instruction pointer, unassociated with [`BasicBlock`]
    pub fn invalid() -> Vip {
        Vip(!0)
    }
}

bitflags! {
    /// Flags describing register properties
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
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

// Mask for local ID in `combined_id`, invert for architecture ID
const LOCAL_ID_MASK: u64 = 0x00ffffffffffffff;

// Define a physical register, including bit count and bit offset
macro_rules! dr {
    ($arch_id:expr, $name:ident, $id:expr, $offset:expr, $count:expr, $doc:expr) => {
        #[doc = $doc]
        #[doc = " register"]
        pub const $name: RegisterDesc = RegisterDesc {
            flags: RegisterFlags::PHYSICAL,
            combined_id: (($arch_id as u64) << 56) | $id,
            bit_count: $count * 8,
            bit_offset: $offset * 8,
        };
    };

    ($name:ident, $id:expr, $offset:expr, $count:expr) => {
        dr!($name, $id, $offset, $count, stringify!($name));
    };
}

macro_rules! dr_amd64 {
    ($name:ident, $id:expr, $offset:expr, $count:expr) => {
        dr!(
            ArchitectureIdentifier::Amd64,
            $name,
            $id,
            $offset,
            $count,
            stringify!($name)
        );
    };
}

macro_rules! dr_arm64 {
    ($name:ident, $id:expr, $offset:expr, $count:expr) => {
        dr!(
            ArchitectureIdentifier::Arm64,
            $name,
            $id,
            $offset,
            $count,
            stringify!($name)
        );
    };
}

impl RegisterDesc {
    /// Undefined register
    pub const UNDEFINED: RegisterDesc = RegisterDesc {
        flags: RegisterFlags::from_bits_truncate(
            RegisterFlags::VOLATILE.bits() | RegisterFlags::UNDEFINED.bits(),
        ),
        combined_id: 0,
        bit_count: 64,
        bit_offset: 0,
    };

    /// Image base register
    pub const IMGBASE: RegisterDesc = RegisterDesc {
        flags: RegisterFlags::from_bits_truncate(
            RegisterFlags::READONLY.bits() | RegisterFlags::IMAGE_BASE.bits(),
        ),
        combined_id: 0,
        bit_count: 64,
        bit_offset: 0,
    };

    /// Flags register
    pub const FLAGS: RegisterDesc = RegisterDesc {
        flags: RegisterFlags::from_bits_truncate(
            RegisterFlags::PHYSICAL.bits() | RegisterFlags::FLAGS.bits(),
        ),
        combined_id: 0,
        bit_count: 64,
        bit_offset: 0,
    };

    /// Stack pointer register
    pub const SP: RegisterDesc = RegisterDesc {
        flags: RegisterFlags::from_bits_truncate(
            RegisterFlags::PHYSICAL.bits() | RegisterFlags::STACK_POINTER.bits(),
        ),
        combined_id: 0,
        bit_count: 64,
        bit_offset: 0,
    };

    dr_amd64!(X86_REG_RAX, amd64::X86_REG_RAX, 0, 8);
    dr_amd64!(X86_REG_EAX, amd64::X86_REG_RAX, 0, 4);
    dr_amd64!(X86_REG_AX, amd64::X86_REG_RAX, 0, 2);
    dr_amd64!(X86_REG_AH, amd64::X86_REG_RAX, 1, 1);
    dr_amd64!(X86_REG_AL, amd64::X86_REG_RAX, 0, 1);

    dr_amd64!(X86_REG_RBX, amd64::X86_REG_RBX, 0, 8);
    dr_amd64!(X86_REG_EBX, amd64::X86_REG_RBX, 0, 4);
    dr_amd64!(X86_REG_BX, amd64::X86_REG_RBX, 0, 2);
    dr_amd64!(X86_REG_BH, amd64::X86_REG_RBX, 1, 1);
    dr_amd64!(X86_REG_BL, amd64::X86_REG_RBX, 0, 1);

    dr_amd64!(X86_REG_RCX, amd64::X86_REG_RCX, 0, 8);
    dr_amd64!(X86_REG_ECX, amd64::X86_REG_RCX, 0, 4);
    dr_amd64!(X86_REG_CX, amd64::X86_REG_RCX, 0, 2);
    dr_amd64!(X86_REG_CH, amd64::X86_REG_RCX, 1, 1);
    dr_amd64!(X86_REG_CL, amd64::X86_REG_RCX, 0, 1);

    dr_amd64!(X86_REG_RDX, amd64::X86_REG_RDX, 0, 8);
    dr_amd64!(X86_REG_EDX, amd64::X86_REG_RDX, 0, 4);
    dr_amd64!(X86_REG_DX, amd64::X86_REG_RDX, 0, 2);
    dr_amd64!(X86_REG_DH, amd64::X86_REG_RDX, 1, 1);
    dr_amd64!(X86_REG_DL, amd64::X86_REG_RDX, 0, 1);

    dr_amd64!(X86_REG_RDI, amd64::X86_REG_RDI, 0, 8);
    dr_amd64!(X86_REG_EDI, amd64::X86_REG_RDI, 0, 4);
    dr_amd64!(X86_REG_DI, amd64::X86_REG_RDI, 0, 2);
    dr_amd64!(X86_REG_DIL, amd64::X86_REG_RDI, 0, 1);

    dr_amd64!(X86_REG_RSI, amd64::X86_REG_RSI, 0, 8);
    dr_amd64!(X86_REG_ESI, amd64::X86_REG_RSI, 0, 4);
    dr_amd64!(X86_REG_SI, amd64::X86_REG_RSI, 0, 2);
    dr_amd64!(X86_REG_SIL, amd64::X86_REG_RSI, 0, 1);

    dr_amd64!(X86_REG_RBP, amd64::X86_REG_RBP, 0, 8);
    dr_amd64!(X86_REG_EBP, amd64::X86_REG_RBP, 0, 4);
    dr_amd64!(X86_REG_BP, amd64::X86_REG_RBP, 0, 2);
    dr_amd64!(X86_REG_BPL, amd64::X86_REG_RBP, 0, 1);

    dr_amd64!(X86_REG_RSP, amd64::X86_REG_RSP, 0, 8);
    dr_amd64!(X86_REG_ESP, amd64::X86_REG_RSP, 0, 4);
    dr_amd64!(X86_REG_SP, amd64::X86_REG_RSP, 0, 2);
    dr_amd64!(X86_REG_SPL, amd64::X86_REG_RSP, 0, 1);

    dr_amd64!(X86_REG_R8, amd64::X86_REG_R8, 0, 8);
    dr_amd64!(X86_REG_R8D, amd64::X86_REG_R8, 0, 4);
    dr_amd64!(X86_REG_R8W, amd64::X86_REG_R8, 0, 2);
    dr_amd64!(X86_REG_R8B, amd64::X86_REG_R8, 0, 1);

    dr_amd64!(X86_REG_R9, amd64::X86_REG_R9, 0, 8);
    dr_amd64!(X86_REG_R9D, amd64::X86_REG_R9, 0, 4);
    dr_amd64!(X86_REG_R9W, amd64::X86_REG_R9, 0, 2);
    dr_amd64!(X86_REG_R9B, amd64::X86_REG_R9, 0, 1);

    dr_amd64!(X86_REG_R10, amd64::X86_REG_R10, 0, 8);
    dr_amd64!(X86_REG_R10D, amd64::X86_REG_R10, 0, 4);
    dr_amd64!(X86_REG_R10W, amd64::X86_REG_R10, 0, 2);
    dr_amd64!(X86_REG_R10B, amd64::X86_REG_R10, 0, 1);

    dr_amd64!(X86_REG_R11, amd64::X86_REG_R11, 0, 8);
    dr_amd64!(X86_REG_R11D, amd64::X86_REG_R11, 0, 4);
    dr_amd64!(X86_REG_R11W, amd64::X86_REG_R11, 0, 2);
    dr_amd64!(X86_REG_R11B, amd64::X86_REG_R11, 0, 1);

    dr_amd64!(X86_REG_R12, amd64::X86_REG_R12, 0, 8);
    dr_amd64!(X86_REG_R12D, amd64::X86_REG_R12, 0, 4);
    dr_amd64!(X86_REG_R12W, amd64::X86_REG_R12, 0, 2);
    dr_amd64!(X86_REG_R12B, amd64::X86_REG_R12, 0, 1);

    dr_amd64!(X86_REG_R13, amd64::X86_REG_R13, 0, 8);
    dr_amd64!(X86_REG_R13D, amd64::X86_REG_R13, 0, 4);
    dr_amd64!(X86_REG_R13W, amd64::X86_REG_R13, 0, 2);
    dr_amd64!(X86_REG_R13B, amd64::X86_REG_R13, 0, 1);

    dr_amd64!(X86_REG_R14, amd64::X86_REG_R14, 0, 8);
    dr_amd64!(X86_REG_R14D, amd64::X86_REG_R14, 0, 4);
    dr_amd64!(X86_REG_R14W, amd64::X86_REG_R14, 0, 2);
    dr_amd64!(X86_REG_R14B, amd64::X86_REG_R14, 0, 1);

    dr_amd64!(X86_REG_R15, amd64::X86_REG_R15, 0, 8);
    dr_amd64!(X86_REG_R15D, amd64::X86_REG_R15, 0, 4);
    dr_amd64!(X86_REG_R15W, amd64::X86_REG_R15, 0, 2);
    dr_amd64!(X86_REG_R15B, amd64::X86_REG_R15, 0, 1);

    dr_amd64!(X86_REG_EFLAGS, amd64::X86_REG_EFLAGS, 0, 8);

    dr_arm64!(ARM64_REG_X0, arm64::ARM64_REG_X0, 0, 8);
    dr_arm64!(ARM64_REG_W0, arm64::ARM64_REG_X0, 0, 4);

    dr_arm64!(ARM64_REG_X1, arm64::ARM64_REG_X1, 0, 8);
    dr_arm64!(ARM64_REG_W1, arm64::ARM64_REG_X1, 0, 4);

    dr_arm64!(ARM64_REG_X2, arm64::ARM64_REG_X2, 0, 8);
    dr_arm64!(ARM64_REG_W2, arm64::ARM64_REG_X2, 0, 4);

    dr_arm64!(ARM64_REG_X3, arm64::ARM64_REG_X3, 0, 8);
    dr_arm64!(ARM64_REG_W3, arm64::ARM64_REG_X3, 0, 4);

    dr_arm64!(ARM64_REG_X4, arm64::ARM64_REG_X4, 0, 8);
    dr_arm64!(ARM64_REG_W4, arm64::ARM64_REG_X4, 0, 4);

    dr_arm64!(ARM64_REG_X5, arm64::ARM64_REG_X5, 0, 8);
    dr_arm64!(ARM64_REG_W5, arm64::ARM64_REG_X5, 0, 4);

    dr_arm64!(ARM64_REG_X6, arm64::ARM64_REG_X6, 0, 8);
    dr_arm64!(ARM64_REG_W6, arm64::ARM64_REG_X6, 0, 4);

    dr_arm64!(ARM64_REG_X7, arm64::ARM64_REG_X7, 0, 8);
    dr_arm64!(ARM64_REG_W7, arm64::ARM64_REG_X7, 0, 4);

    dr_arm64!(ARM64_REG_X8, arm64::ARM64_REG_X8, 0, 8);
    dr_arm64!(ARM64_REG_W8, arm64::ARM64_REG_X8, 0, 4);

    dr_arm64!(ARM64_REG_X9, arm64::ARM64_REG_X9, 0, 8);
    dr_arm64!(ARM64_REG_W9, arm64::ARM64_REG_X9, 0, 4);

    dr_arm64!(ARM64_REG_X10, arm64::ARM64_REG_X10, 0, 8);
    dr_arm64!(ARM64_REG_W10, arm64::ARM64_REG_X10, 0, 4);

    dr_arm64!(ARM64_REG_X11, arm64::ARM64_REG_X11, 0, 8);
    dr_arm64!(ARM64_REG_W11, arm64::ARM64_REG_X11, 0, 4);

    dr_arm64!(ARM64_REG_X12, arm64::ARM64_REG_X12, 0, 8);
    dr_arm64!(ARM64_REG_W12, arm64::ARM64_REG_X12, 0, 4);

    dr_arm64!(ARM64_REG_X13, arm64::ARM64_REG_X13, 0, 8);
    dr_arm64!(ARM64_REG_W13, arm64::ARM64_REG_X13, 0, 4);

    dr_arm64!(ARM64_REG_X14, arm64::ARM64_REG_X14, 0, 8);
    dr_arm64!(ARM64_REG_W14, arm64::ARM64_REG_X14, 0, 4);

    dr_arm64!(ARM64_REG_X15, arm64::ARM64_REG_X15, 0, 8);
    dr_arm64!(ARM64_REG_W15, arm64::ARM64_REG_X15, 0, 4);

    dr_arm64!(ARM64_REG_X16, arm64::ARM64_REG_X16, 0, 8);
    dr_arm64!(ARM64_REG_W16, arm64::ARM64_REG_X16, 0, 4);

    dr_arm64!(ARM64_REG_X17, arm64::ARM64_REG_X17, 0, 8);
    dr_arm64!(ARM64_REG_W17, arm64::ARM64_REG_X17, 0, 4);

    dr_arm64!(ARM64_REG_X18, arm64::ARM64_REG_X18, 0, 8);
    dr_arm64!(ARM64_REG_W18, arm64::ARM64_REG_X18, 0, 4);

    dr_arm64!(ARM64_REG_X19, arm64::ARM64_REG_X19, 0, 8);
    dr_arm64!(ARM64_REG_W19, arm64::ARM64_REG_X19, 0, 4);

    dr_arm64!(ARM64_REG_X20, arm64::ARM64_REG_X20, 0, 8);
    dr_arm64!(ARM64_REG_W20, arm64::ARM64_REG_X20, 0, 4);

    dr_arm64!(ARM64_REG_X21, arm64::ARM64_REG_X21, 0, 8);
    dr_arm64!(ARM64_REG_W21, arm64::ARM64_REG_X21, 0, 4);

    dr_arm64!(ARM64_REG_X22, arm64::ARM64_REG_X22, 0, 8);
    dr_arm64!(ARM64_REG_W22, arm64::ARM64_REG_X22, 0, 4);

    dr_arm64!(ARM64_REG_X23, arm64::ARM64_REG_X23, 0, 8);
    dr_arm64!(ARM64_REG_W23, arm64::ARM64_REG_X23, 0, 4);

    dr_arm64!(ARM64_REG_X24, arm64::ARM64_REG_X24, 0, 8);
    dr_arm64!(ARM64_REG_W24, arm64::ARM64_REG_X24, 0, 4);

    dr_arm64!(ARM64_REG_X25, arm64::ARM64_REG_X25, 0, 8);
    dr_arm64!(ARM64_REG_W25, arm64::ARM64_REG_X25, 0, 4);

    dr_arm64!(ARM64_REG_X26, arm64::ARM64_REG_X26, 0, 8);
    dr_arm64!(ARM64_REG_W26, arm64::ARM64_REG_X26, 0, 4);

    dr_arm64!(ARM64_REG_X27, arm64::ARM64_REG_X27, 0, 8);
    dr_arm64!(ARM64_REG_W27, arm64::ARM64_REG_X27, 0, 4);

    dr_arm64!(ARM64_REG_X28, arm64::ARM64_REG_X28, 0, 8);
    dr_arm64!(ARM64_REG_W28, arm64::ARM64_REG_X28, 0, 4);

    dr_arm64!(ARM64_REG_X29, arm64::ARM64_REG_X29, 0, 8);
    dr_arm64!(ARM64_REG_FP, arm64::ARM64_REG_X29, 0, 8);
    dr_arm64!(ARM64_REG_W29, arm64::ARM64_REG_X29, 0, 4);

    dr_arm64!(ARM64_REG_X30, arm64::ARM64_REG_X30, 0, 8);
    dr_arm64!(ARM64_REG_LR, arm64::ARM64_REG_X30, 0, 8);
    dr_arm64!(ARM64_REG_W30, arm64::ARM64_REG_X30, 0, 4);

    dr_arm64!(ARM64_REG_XZR, arm64::ARM64_REG_XZR, 0, 8);
    dr_arm64!(ARM64_REG_WZR, arm64::ARM64_REG_XZR, 0, 4);

    dr_arm64!(ARM64_REG_SP, arm64::ARM64_REG_SP, 0, 8);
    dr_arm64!(ARM64_REG_WSP, arm64::ARM64_REG_SP, 0, 4);

    dr_arm64!(ARM64_REG_NZCV, arm64::ARM64_REG_NZCV, 0, 8);

    /// Local identifier that is intentionally unique to this register
    pub fn local_id(&self) -> u64 {
        self.combined_id & LOCAL_ID_MASK
    }

    /// The underlying architecture of this register
    pub fn arch_id(&self) -> ArchitectureIdentifier {
        match (self.combined_id & !LOCAL_ID_MASK) >> 56 {
            0 => ArchitectureIdentifier::Amd64,
            1 => ArchitectureIdentifier::Arm64,
            2 => ArchitectureIdentifier::Virtual,
            _ => unreachable!(),
        }
    }

    /// Operand size in bits, rounding up
    pub fn size(&self) -> usize {
        (self.bit_count as usize + 7) / 8
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
                        arch_info::amd64::REGISTER_NAME_MAPPING[self.local_id() as usize],
                        suffix
                    )?;
                    return Ok(());
                }
                ArchitectureIdentifier::Arm64 => {
                    write!(
                        f,
                        "{}{}{}",
                        prefix,
                        arch_info::arm64::REGISTER_NAME_MAPPING[self.local_id() as usize],
                        suffix
                    )?;
                    return Ok(());
                }
                _ => {}
            }
        }

        write!(f, "{}vr{}{}", prefix, self.local_id(), suffix)?;
        Ok(())
    }
}

/// Routine calling convention information and associated metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg(feature = "serde")]
impl Serialize for Immediate {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.i64())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Immediate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Immediate, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Immediate {
            i64: i64::deserialize(deserializer)?,
        })
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateDesc {
    pub(crate) value: Immediate,
    /// The bit count of this register (e.g.: 32)
    pub bit_count: u32,
}

impl From<i64> for ImmediateDesc {
    fn from(imm: i64) -> ImmediateDesc {
        ImmediateDesc::new_signed(imm, 64)
    }
}

impl From<u64> for ImmediateDesc {
    fn from(imm: u64) -> ImmediateDesc {
        ImmediateDesc::new(imm, 64)
    }
}

impl From<i32> for ImmediateDesc {
    fn from(imm: i32) -> ImmediateDesc {
        ImmediateDesc::new_signed(imm, 32)
    }
}

impl From<u32> for ImmediateDesc {
    fn from(imm: u32) -> ImmediateDesc {
        ImmediateDesc::new(imm, 32)
    }
}

impl From<i16> for ImmediateDesc {
    fn from(imm: i16) -> ImmediateDesc {
        ImmediateDesc::new_signed(imm, 16)
    }
}

impl From<u16> for ImmediateDesc {
    fn from(imm: u16) -> ImmediateDesc {
        ImmediateDesc::new(imm, 16)
    }
}

impl From<i8> for ImmediateDesc {
    fn from(imm: i8) -> ImmediateDesc {
        ImmediateDesc::new_signed(imm, 8)
    }
}

impl From<u8> for ImmediateDesc {
    fn from(imm: u8) -> ImmediateDesc {
        ImmediateDesc::new(imm, 8)
    }
}

impl ImmediateDesc {
    /// Immediate from a `u64`
    pub fn new<T: Into<u64>>(value: T, bit_count: u32) -> ImmediateDesc {
        ImmediateDesc {
            value: Immediate { u64: value.into() },
            bit_count,
        }
    }

    /// Immediate from an `i64`
    pub fn new_signed<T: Into<i64>>(value: T, bit_count: u32) -> ImmediateDesc {
        ImmediateDesc {
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

    /// Operand size in bits, rounding up
    pub fn size(&self) -> usize {
        (self.bit_count as usize + 7) / 8
    }
}

/// VTIL instruction operand
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    /// Immediate operand containing a sized immediate value
    ImmediateDesc(ImmediateDesc),
    /// Register operand containing a register description
    RegisterDesc(RegisterDesc),
}

impl From<i64> for Operand {
    fn from(imm: i64) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<u64> for Operand {
    fn from(imm: u64) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<i32> for Operand {
    fn from(imm: i32) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<u32> for Operand {
    fn from(imm: u32) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<i16> for Operand {
    fn from(imm: i16) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<u16> for Operand {
    fn from(imm: u16) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<i8> for Operand {
    fn from(imm: i8) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl From<u8> for Operand {
    fn from(imm: u8) -> Operand {
        Operand::ImmediateDesc(imm.into())
    }
}

impl Operand {
    /// Operand size in bits, rounding up
    pub fn size(&self) -> usize {
        match self {
            Operand::ImmediateDesc(i) => i.size(),
            Operand::RegisterDesc(r) => r.size(),
        }
    }
}

impl From<RegisterDesc> for Operand {
    fn from(register_desc: RegisterDesc) -> Self {
        Operand::RegisterDesc(register_desc)
    }
}

impl From<ImmediateDesc> for Operand {
    fn from(immediate_desc: ImmediateDesc) -> Self {
        Operand::ImmediateDesc(immediate_desc)
    }
}

impl<'a, 'b> TryInto<&'b ImmediateDesc> for &'a Operand
where
    'a: 'b,
{
    type Error = Error;

    fn try_into(self) -> Result<&'a ImmediateDesc> {
        match self {
            Operand::ImmediateDesc(ref i) => Ok(i),
            _ => Err(Error::OperandTypeMismatch),
        }
    }
}

impl<'a, 'b> TryInto<&'b RegisterDesc> for &'a Operand
where
    'a: 'b,
{
    type Error = Error;

    fn try_into(self) -> Result<&'a RegisterDesc> {
        match self {
            Operand::RegisterDesc(r) => Ok(r),
            _ => Err(Error::OperandTypeMismatch),
        }
    }
}

/// VTIL instruction and associated metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum Op {
    // Data/Memory instructions
    /// OP1 = ZX(OP2)
    Mov(Operand, Operand),
    /// OP1 = SX(OP2)
    Movsx(Operand, Operand),
    /// \[OP1+OP2\] <= OP3
    Str(Operand, Operand, Operand),
    /// OP1 <= \[OP2+OP3\]
    Ldd(Operand, Operand, Operand),

    // Arithmetic instructions
    /// OP1 = -OP1
    Neg(Operand),
    /// OP1 = OP1 + OP2
    Add(Operand, Operand),
    /// OP1 = OP1 - OP2
    Sub(Operand, Operand),
    /// OP1 = OP1 * OP2
    Mul(Operand, Operand),
    /// OP1 = \[OP1 * OP2\]>>N
    Mulhi(Operand, Operand),
    /// OP1 = OP1 * OP2 (Signed)
    Imul(Operand, Operand),
    /// OP1 = \[OP1 * OP2\]>>N (Signed)
    Imulhi(Operand, Operand),
    /// OP1 = \[OP2:OP1\] / OP3
    Div(Operand, Operand, Operand),
    /// OP1 = \[OP2:OP1\] % OP3
    Rem(Operand, Operand, Operand),
    /// OP1 = \[OP2:OP1\] / OP3 (Signed)
    Idiv(Operand, Operand, Operand),
    /// OP1 = \[OP2:OP1\] % OP3 (Signed)
    Irem(Operand, Operand, Operand),

    // Bitwise instructions
    /// OP1 = popcnt OP1
    Popcnt(Operand),
    /// OP1 = OP1 ? BitScanForward OP1 + 1 : 0
    Bsf(Operand),
    /// OP1 = OP1 ? BitScanReverse OP1 + 1 : 0
    Bsr(Operand),
    /// OP1 = ~OP1
    Not(Operand),
    /// OP1 >>= OP2
    Shr(Operand, Operand),
    /// OP1 <<= OP2
    Shl(Operand, Operand),
    /// OP1 ^= OP2
    Xor(Operand, Operand),
    /// OP1 |= OP2
    Or(Operand, Operand),
    /// OP1 &= OP2
    And(Operand, Operand),
    /// OP1 = (OP1>>OP2) | (OP1<<(N-OP2))
    Ror(Operand, Operand),
    /// OP1 = (OP1<<OP2) | (OP1>>(N-OP2))
    Rol(Operand, Operand),

    // Conditional instructions
    /// OP1 = OP2 > OP3
    Tg(Operand, Operand, Operand),
    /// OP1 = OP2 >= OP3
    Tge(Operand, Operand, Operand),
    /// OP1 = OP2 == OP3
    Te(Operand, Operand, Operand),
    /// OP1 = OP2 != OP3
    Tne(Operand, Operand, Operand),
    /// OP1 = OP2 < OP3
    Tl(Operand, Operand, Operand),
    /// OP1 = OP2 <= OP3
    Tle(Operand, Operand, Operand),
    /// OP1 = OP2 <= OP3
    Tug(Operand, Operand, Operand),
    /// OP1 = OP2   u>=  OP3
    Tuge(Operand, Operand, Operand),
    /// OP1 = OP2   u<   OP3
    Tul(Operand, Operand, Operand),
    /// OP1 = OP2   u<=  OP3
    Tule(Operand, Operand, Operand),
    /// OP1 = OP2 ? OP3 : 0
    Ifs(Operand, Operand, Operand),

    // Control flow instructions
    /// Jumps to OP1 ? OP2 : OP3, continues virtual execution
    Js(Operand, Operand, Operand),
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
    Vemit(Operand),
    /// Pins the register for read
    Vpinr(Operand),
    /// Pins the register for write
    Vpinw(Operand),
    /// Pins the memory location for read, with size = OP3
    Vpinrm(Operand, Operand, Operand),
    /// Pins the memory location for write, with size = OP3
    Vpinwm(Operand, Operand, Operand),
}

impl Op {
    /// Name of the operand
    pub fn name(&self) -> &'static str {
        match self {
            Op::Mov(_, _) => "mov",
            Op::Movsx(_, _) => "movsx",
            Op::Str(_, _, _) => "str",
            Op::Ldd(_, _, _) => "ldd",
            Op::Neg(_) => "neg",
            Op::Add(_, _) => "add",
            Op::Sub(_, _) => "sub",
            Op::Mul(_, _) => "mul",
            Op::Mulhi(_, _) => "mulhi",
            Op::Imul(_, _) => "imul",
            Op::Imulhi(_, _) => "imulhi",
            Op::Div(_, _, _) => "div",
            Op::Rem(_, _, _) => "rem",
            Op::Idiv(_, _, _) => "idiv",
            Op::Irem(_, _, _) => "irem",
            Op::Popcnt(_) => "popcnt",
            Op::Bsf(_) => "bsf",
            Op::Bsr(_) => "bsr",
            Op::Not(_) => "not",
            Op::Shr(_, _) => "shr",
            Op::Shl(_, _) => "shl",
            Op::Xor(_, _) => "xor",
            Op::Or(_, _) => "or",
            Op::And(_, _) => "and",
            Op::Ror(_, _) => "ror",
            Op::Rol(_, _) => "rol",
            Op::Tg(_, _, _) => "tg",
            Op::Tge(_, _, _) => "tge",
            Op::Te(_, _, _) => "te",
            Op::Tne(_, _, _) => "tne",
            Op::Tl(_, _, _) => "tl",
            Op::Tle(_, _, _) => "tle",
            Op::Tug(_, _, _) => "tug",
            Op::Tuge(_, _, _) => "tuge",
            Op::Tul(_, _, _) => "tul",
            Op::Tule(_, _, _) => "tule",
            Op::Ifs(_, _, _) => "ifs",
            Op::Js(_, _, _) => "js",
            Op::Jmp(_) => "jmp",
            Op::Vexit(_) => "vexit",
            Op::Vxcall(_) => "vxcall",
            Op::Nop => "nop",
            Op::Sfence => "sfence",
            Op::Lfence => "lfence",
            Op::Vemit(_) => "vemit",
            Op::Vpinr(_) => "vpinr",
            Op::Vpinw(_) => "vpinw",
            Op::Vpinrm(_, _, _) => "vpinrm",
            Op::Vpinwm(_, _, _) => "vpinwm",
        }
    }

    /// Operands for operator
    pub fn operands(&self) -> Vec<&Operand> {
        match *self {
            Op::Nop | Op::Sfence | Op::Lfence => vec![],
            Op::Neg(ref op1)
            | Op::Popcnt(ref op1)
            | Op::Bsf(ref op1)
            | Op::Bsr(ref op1)
            | Op::Not(ref op1)
            | Op::Jmp(ref op1)
            | Op::Vexit(ref op1)
            | Op::Vxcall(ref op1)
            | Op::Vemit(ref op1)
            | Op::Vpinr(ref op1)
            | Op::Vpinw(ref op1) => vec![op1],
            Op::Mov(ref op1, ref op2)
            | Op::Movsx(ref op1, ref op2)
            | Op::Add(ref op1, ref op2)
            | Op::Sub(ref op1, ref op2)
            | Op::Mul(ref op1, ref op2)
            | Op::Mulhi(ref op1, ref op2)
            | Op::Imul(ref op1, ref op2)
            | Op::Imulhi(ref op1, ref op2)
            | Op::Shr(ref op1, ref op2)
            | Op::Shl(ref op1, ref op2)
            | Op::Xor(ref op1, ref op2)
            | Op::Or(ref op1, ref op2)
            | Op::And(ref op1, ref op2)
            | Op::Ror(ref op1, ref op2)
            | Op::Rol(ref op1, ref op2) => vec![op1, op2],
            Op::Str(ref op1, ref op2, ref op3)
            | Op::Ldd(ref op1, ref op2, ref op3)
            | Op::Div(ref op1, ref op2, ref op3)
            | Op::Rem(ref op1, ref op2, ref op3)
            | Op::Idiv(ref op1, ref op2, ref op3)
            | Op::Irem(ref op1, ref op2, ref op3)
            | Op::Tg(ref op1, ref op2, ref op3)
            | Op::Tge(ref op1, ref op2, ref op3)
            | Op::Te(ref op1, ref op2, ref op3)
            | Op::Tne(ref op1, ref op2, ref op3)
            | Op::Tl(ref op1, ref op2, ref op3)
            | Op::Tle(ref op1, ref op2, ref op3)
            | Op::Tug(ref op1, ref op2, ref op3)
            | Op::Tuge(ref op1, ref op2, ref op3)
            | Op::Tul(ref op1, ref op2, ref op3)
            | Op::Tule(ref op1, ref op2, ref op3)
            | Op::Ifs(ref op1, ref op2, ref op3)
            | Op::Js(ref op1, ref op2, ref op3)
            | Op::Vpinrm(ref op1, ref op2, ref op3)
            | Op::Vpinwm(ref op1, ref op2, ref op3) => vec![op1, op2, op3],
        }
    }

    /// Mutable operands for operator
    pub fn operands_mut(&mut self) -> Vec<&mut Operand> {
        match *self {
            Op::Nop | Op::Sfence | Op::Lfence => vec![],
            Op::Neg(ref mut op1)
            | Op::Popcnt(ref mut op1)
            | Op::Bsf(ref mut op1)
            | Op::Bsr(ref mut op1)
            | Op::Not(ref mut op1)
            | Op::Jmp(ref mut op1)
            | Op::Vexit(ref mut op1)
            | Op::Vxcall(ref mut op1)
            | Op::Vemit(ref mut op1)
            | Op::Vpinr(ref mut op1)
            | Op::Vpinw(ref mut op1) => vec![op1],
            Op::Mov(ref mut op1, ref mut op2)
            | Op::Movsx(ref mut op1, ref mut op2)
            | Op::Add(ref mut op1, ref mut op2)
            | Op::Sub(ref mut op1, ref mut op2)
            | Op::Mul(ref mut op1, ref mut op2)
            | Op::Mulhi(ref mut op1, ref mut op2)
            | Op::Imul(ref mut op1, ref mut op2)
            | Op::Imulhi(ref mut op1, ref mut op2)
            | Op::Shr(ref mut op1, ref mut op2)
            | Op::Shl(ref mut op1, ref mut op2)
            | Op::Xor(ref mut op1, ref mut op2)
            | Op::Or(ref mut op1, ref mut op2)
            | Op::And(ref mut op1, ref mut op2)
            | Op::Ror(ref mut op1, ref mut op2)
            | Op::Rol(ref mut op1, ref mut op2) => vec![op1, op2],
            Op::Str(ref mut op1, ref mut op2, ref mut op3)
            | Op::Ldd(ref mut op1, ref mut op2, ref mut op3)
            | Op::Div(ref mut op1, ref mut op2, ref mut op3)
            | Op::Rem(ref mut op1, ref mut op2, ref mut op3)
            | Op::Idiv(ref mut op1, ref mut op2, ref mut op3)
            | Op::Irem(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tg(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tge(ref mut op1, ref mut op2, ref mut op3)
            | Op::Te(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tne(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tl(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tle(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tug(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tuge(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tul(ref mut op1, ref mut op2, ref mut op3)
            | Op::Tule(ref mut op1, ref mut op2, ref mut op3)
            | Op::Ifs(ref mut op1, ref mut op2, ref mut op3)
            | Op::Js(ref mut op1, ref mut op2, ref mut op3)
            | Op::Vpinrm(ref mut op1, ref mut op2, ref mut op3)
            | Op::Vpinwm(ref mut op1, ref mut op2, ref mut op3) => vec![op1, op2, op3],
        }
    }

    /// Returns if the instruction is volatile
    pub fn is_volatile(&self) -> bool {
        matches!(
            self,
            Op::Sfence
                | Op::Lfence
                | Op::Vemit(_)
                | Op::Vpinr(_)
                | Op::Vpinw(_)
                | Op::Vpinrm(_, _, _)
                | Op::Vpinwm(_, _, _)
        )
    }
}

/// Basic block containing a linear sequence of VTIL instructions
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl BasicBlock {
    /// Allocate a temporary register for this basic block
    pub fn tmp(&mut self, bit_count: i32) -> RegisterDesc {
        let reg = RegisterDesc {
            flags: RegisterFlags::LOCAL,
            combined_id: self.last_temporary_index as u64,
            bit_count,
            bit_offset: 0,
        };
        self.last_temporary_index += 1;
        reg
    }
}

/// Alias for [`RoutineConvention`] for consistent naming
pub type SubroutineConvention = RoutineConvention;

/// VTIL routine container
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Routine {
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
    pub explored_blocks: IndexMap<Vip, BasicBlock>,
}
