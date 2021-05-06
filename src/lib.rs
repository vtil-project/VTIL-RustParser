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
//! For a simple example of loading a VTIL file and reading out some basic data:
//! ```
//! # use vtil_parser::Result;
//! use vtil_parser::{VTILReader, ArchitectureIdentifier};
//!
//! # fn main() -> Result<()> {
//! let routine = VTILReader::from_path("resources/big.vtil")?;
//! assert_eq!(routine.header.arch_id, ArchitectureIdentifier::Amd64);
//! # Ok(())
//! # }
//! ```
//!
//! For a more complex example, iterating over IL instructions:
//! ```
//! # use vtil_parser::Result;
//! use vtil_parser::{VTILReader, Op, Operand, Reg, Imm, RegisterFlags};
//!
//! # fn main() -> Result<()> {
//! let routine = VTILReader::from_path("resources/big.vtil")?;
//!
//! for basic_block in routine.explored_blocks.iter().take(1) {
//!     for instr in basic_block.instructions.iter().take(1) {
//!         match &instr.op {
//!             Op::Ldd(_, Operand::Reg(op2), Operand::Imm(op3)) => {
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
use scroll::{Pread, Pwrite};
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

/// Reader for VTIL containers
pub struct VTILReader;

impl VTILReader {
    /// Tries to load VTIL from the given path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<VTIL> {
        let source = Box::new(unsafe { MmapOptions::new().map(&File::open(path.as_ref())?)? });
        source.pread_with::<VTIL>(0, scroll::LE)
    }

    /// Loads VTIL from a `Vec<u8>`
    pub fn from_vec<B: AsRef<[u8]>>(source: B) -> Result<VTIL> {
        source.as_ref().pread_with::<VTIL>(0, scroll::LE)
    }
}

impl VTIL {
    /// Build a new VTIL container
    pub fn new(
        arch_id: ArchitectureIdentifier,
        vip: Vip,
        routine_convention: RoutineConvention,
        subroutine_convention: SubroutineConvention,
    ) -> VTIL {
        VTIL {
            header: Header { arch_id },
            vip,
            routine_convention,
            subroutine_convention,
            spec_subroutine_conventions: vec![],
            explored_blocks: vec![],
        }
    }

    /// Serialize the VTIL container, consuming it
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::new();
        buffer.pwrite_with::<VTIL>(self, 0, scroll::LE)?;
        Ok(buffer)
    }
}
