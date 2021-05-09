// Copyright © 2021 Keegan Saunders
//
// Permission to use, copy, modify, and/or distribute this software for
// any purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
// AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
// OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use vtil_parser::{
    ArchitectureIdentifier, ImmediateDesc, InstructionBuilder, Result, Routine, Vip,
};

fn main() -> Result<()> {
    let mut routine = Routine::new(ArchitectureIdentifier::Virtual);
    let basic_block = routine.create_block(Vip(0)).unwrap();
    let tmp0 = basic_block.tmp(64);
    let mut builder = InstructionBuilder::from(basic_block);
    builder.mov(tmp0, ImmediateDesc::new(0xA57E6F0335298D0u64, 64).into());
    std::fs::write("built.vtil", routine.into_bytes()?)?;
    Ok(())
}
