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

#[self_referencing(no_doc)]
/// VTIL container
pub struct VTIL<T: 'static> {
    source: Box<T>,
    #[borrows(source)]
    pub(crate) inner: VTILInner<'this>,
    phantom: PhantomData<T>,
}

impl<T> VTIL<T> {
    /// Header containing metadata about the VTIL container
    pub fn header(&self) -> &Header {
        self.inner.header()
    }

    /// The entry virtual instruction pointer for this VTIL routine
    pub fn vip(&self) -> &Vip {
        self.inner.vip()
    }

    /// Metadata regarding the calling conventions of the VTIL routine
    pub fn routine_convention(&self) -> &RoutineConvention {
        self.inner.routine_convention()
    }

    /// Metadata regarding the calling conventions of the VTIL subroutine
    pub fn subroutine_convention(&self) -> &SubroutineConvention {
        self.inner.subroutine_convention()
    }

    /// All special subroutine calling conventions in the top-level VTIL routine
    pub fn spec_subroutine_conventions(&self) -> &Vec<SubroutineConvention> {
        self.inner.spec_subroutine_conventions()
    }

    /// Reachable [`BasicBlock`]s generated during a code-discovery analysis
    /// pass
    pub fn explored_blocks(&self) -> &Vec<BasicBlock> {
        self.inner.explored_blocks()
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
