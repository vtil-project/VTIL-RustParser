#![allow(non_snake_case)]

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

pub type Result<T> = std::result::Result<T, error::Error>;

#[self_referencing]
pub struct VTIL<T: 'static> {
    source: Box<T>,
    #[borrows(source)]
    pub(crate) inner: VTILInner<'this>,
    phantom: PhantomData<T>,
}

impl<T> VTIL<T> {
    pub fn header(&self) -> &Header {
        self.inner.header()
    }

    pub fn vip(&self) -> &Vip {
        self.inner.vip()
    }

    pub fn routine_convention(&self) -> &RoutineConvention {
        self.inner.routine_convention()
    }

    pub fn subroutine_convention(&self) -> &SubroutineConvention {
        self.inner.subroutine_convention()
    }

    pub fn spec_subroutine_conventions(&self) -> &Vec<SubroutineConvention> {
        self.inner.spec_subroutine_conventions()
    }

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
