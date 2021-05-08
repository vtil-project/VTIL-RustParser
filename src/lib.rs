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
//! # VTIL-RustParser
//!
//! Read/write VTIL files in Rust.
//!
//! You can learn more about VTIL [here](https://github.com/vtil-project/VTIL-Core#introduction)
//! on the main GitHub page.
//!
//! # Examples
//! For a simple example of loading a VTIL routine and reading out some basic data:
//! ```
//! # use vtil_parser::Result;
//! use vtil_parser::{Routine, ArchitectureIdentifier};
//!
//! # fn main() -> Result<()> {
//! let routine = Routine::from_path("resources/big.vtil")?;
//! assert_eq!(routine.header.arch_id, ArchitectureIdentifier::Amd64);
//! # Ok(())
//! # }
//! ```
//!
//! For a more complex example, iterating over IL instructions:
//! ```
//! # use vtil_parser::Result;
//! use vtil_parser::{Routine, Op, Operand, RegisterDesc, ImmediateDesc, RegisterFlags};
//!
//! # fn main() -> Result<()> {
//! let routine = Routine::from_path("resources/big.vtil")?;
//!
//! for basic_block in routine.explored_blocks.iter().take(1) {
//!     for instr in basic_block.instructions.iter().take(1) {
//!         match &instr.op {
//!             Op::Ldd(_, Operand::RegisterDesc(op2), Operand::ImmediateDesc(op3)) => {
//!                 assert!(op2.flags.contains(RegisterFlags::PHYSICAL));
//!                 assert!(op3.i64() == 0);
//!             }
//!             _ => assert!(false)
//!         }
//!
//!         assert_eq!(instr.vip.0, 0x9b833);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![allow(clippy::upper_case_acronyms)]
#![deny(missing_docs)]

use memmap::MmapOptions;
use scroll::{ctx::SizeWith, Pread, Pwrite};

use std::fs::File;
use std::path::Path;

#[macro_use]
extern crate bitflags;

mod error;
pub use error::Error;

mod arch_info;

mod pod;
pub use pod::*;

mod serialize;
pub use serialize::*;

#[doc(hidden)]
pub type Result<T> = std::result::Result<T, error::Error>;

/// VTIL routine container
impl Routine {
    /// Build a new VTIL routine container
    pub fn new(arch_id: ArchitectureIdentifier) -> Routine {
        let (routine_convention, subroutine_convention) = match arch_id {
            ArchitectureIdentifier::Virtual => {
                let routine_convention = RoutineConvention {
                    volatile_registers: vec![],
                    param_registers: vec![],
                    retval_registers: vec![],
                    // Not used, so it doesn't matter
                    frame_register: RegisterDesc {
                        flags: RegisterFlags::VIRTUAL,
                        combined_id: 0,
                        bit_count: 0,
                        bit_offset: 0,
                    },
                    shadow_space: 0,
                    purge_stack: true,
                };
                (routine_convention.clone(), routine_convention)
            }
            _ => unimplemented!(),
        };
        Routine {
            header: Header { arch_id },
            vip: Vip(0),
            routine_convention,
            subroutine_convention,
            spec_subroutine_conventions: vec![],
            explored_blocks: vec![],
        }
    }

    /// Tries to load VTIL routine from the given path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Routine> {
        let source = Box::new(unsafe { MmapOptions::new().map(&File::open(path.as_ref())?)? });
        source.pread_with::<Routine>(0, scroll::LE)
    }

    /// Loads VTIL routine from a `Vec<u8>`
    pub fn from_vec(source: &[u8]) -> Result<Routine> {
        source.as_ref().pread_with::<Routine>(0, scroll::LE)
    }

    /// Serialize the VTIL routine container, consuming it
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let size = Routine::size_with(&self);
        let mut buffer = vec![0; size];
        buffer.pwrite_with::<Routine>(self, 0, scroll::LE)?;
        Ok(buffer)
    }
}
