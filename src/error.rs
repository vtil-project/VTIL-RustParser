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

use std::{fmt, io, num, str};
use thiserror::Error;

/// Custom `Error` for VTIL parsing
#[derive(Error, Debug)]
pub enum Error {
    /// An error occured during parsing due to a malformed VTIL file
    #[error("Malformed VTIL file")]
    Malformed(String),

    /// An I/O error occured
    #[error("I/O error")]
    Io(#[from] io::Error),

    /// Error inside of [Scroll](https://docs.rs/scroll) occured
    #[error("Scroll error")]
    Scroll(#[from] scroll::Error),

    /// Error during UTF-8 decoding, VTIL file is possibly malformed
    #[error("UTF-8 decoding error")]
    Utf8(#[from] str::Utf8Error),

    /// Error during internal formatting
    #[error("Formatting error")]
    Formatting(#[from] fmt::Error),

    /// Overflowing during writing
    #[error("Encoding error, value overflowed")]
    TryFromInt(#[from] num::TryFromIntError),
}
