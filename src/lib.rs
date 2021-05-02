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
//! An in-place parser for VTIL files written in Rust.
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
//! assert_eq!(routine.header().arch_id(), ArchitectureIdentifier::Amd64);
//! # Ok(())
//! # }
//! ```
//!
//! For a more complex example, iterating over IL instructions:
//! ```
//! # use vtil_parser::Result;
//! use vtil_parser::{VTILReader, Operand, RegisterFlags};
//!
//! # fn main() -> Result<()> {
//! let routine = VTILReader::from_path("resources/big.vtil")?;
//!
//! for basic_block in routine.explored_blocks().iter().take(1) {
//!     for instr in basic_block.instructions().iter().take(1) {
//!         assert_eq!(instr.name(), "ldd");
//!         assert_eq!(instr.operands().len(), 3);
//!
//!         if let Operand::Reg(reg) = &instr.operands()[1] {
//!             assert!(reg.flags().contains(RegisterFlags::PHYSICAL));
//!         } else { unreachable!() }
//!
//!         if let Operand::Imm(imm) = &instr.operands()[2] {
//!             assert_eq!(imm.i64(), 0);
//!         } else { unreachable!() }
//!
//!         assert_eq!(instr.vip(), 0x9b833);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

use memmap::{Mmap, MmapOptions};
use ouroboros::self_referencing;
use scroll::Pread;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;

#[macro_use]
extern crate bitflags;

mod error;
pub use error::Error;

mod parse;
pub use parse::*;

mod arch_info;

#[doc(hidden)]
pub type Result<T> = std::result::Result<T, error::Error>;

/// VTIL container
#[self_referencing(no_doc)]
pub struct VTIL<T: 'static> {
    source: Box<T>,
    #[borrows(source)]
    #[covariant]
    pub(crate) inner: VTILInner<'this>,
    phantom: PhantomData<T>,
}

impl<T> VTIL<T> {
    /// Header containing metadata about the VTIL container
    pub fn header(&self) -> &Header {
        self.borrow_inner().header()
    }

    /// The entry virtual instruction pointer for this VTIL routine
    pub fn vip(&self) -> &Vip {
        self.borrow_inner().vip()
    }

    /// Metadata regarding the calling conventions of the VTIL routine
    pub fn routine_convention(&self) -> &RoutineConvention {
        self.borrow_inner().routine_convention()
    }

    /// Metadata regarding the calling conventions of the VTIL subroutine
    pub fn subroutine_convention(&self) -> &SubroutineConvention {
        self.borrow_inner().subroutine_convention()
    }

    /// All special subroutine calling conventions in the top-level VTIL routine
    pub fn spec_subroutine_conventions(&self) -> &Vec<SubroutineConvention> {
        self.borrow_inner().spec_subroutine_conventions()
    }

    /// Reachable [`BasicBlock`]s generated during a code-discovery analysis
    /// pass
    pub fn explored_blocks(&self) -> &Vec<BasicBlock> {
        self.borrow_inner().explored_blocks()
    }
}

/// Reader for VTIL containers
pub struct VTILReader;

impl VTILReader {
    /// Tries to load VTIL from the given path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<VTIL<Mmap>> {
        let source = Box::new(unsafe { MmapOptions::new().map(&File::open(path.as_ref())?)? });
        VTILTryBuilder {
            source,
            inner_builder: |source| source.pread_with::<VTILInner>(0, scroll::LE),
            phantom: PhantomData,
        }
        .try_build()
    }

    /// Loads VTIL from a `Vec<u8>`
    pub fn from_vec<B: AsRef<[u8]>>(source: B) -> Result<VTIL<B>> {
        VTILTryBuilder {
            source: Box::new(source),
            inner_builder: |source| source.as_ref().pread_with::<VTILInner>(0, scroll::LE),
            phantom: PhantomData,
        }
        .try_build()
    }
}
